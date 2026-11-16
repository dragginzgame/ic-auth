//! Drive the real issuer to the frozen Node-packed claims, then certify its actual
//! leaf input with Platform fixtures. No product claims/framing builder is an oracle.
use super::{Fixture, SEED, bls_key, budget, bytes, grants, hash, host, issuer, result, sha};
use ic_auth::issuer_memory::{
    IssueRequest, IssuedProof, IssuerAuthority, IssuerConfig, IssuerPolicy, IssuerSigning,
    IssuerTicket, MemoryIssuer,
};
use ic_auth_protocol_types::Principal;
use ic_platform::certification::{
    Certificate, HashTree,
    cbor::{encode_canister_signature, encode_certificate},
    fork, labeled, leaf,
};

fn authority() -> IssuerAuthority {
    IssuerAuthority {
        signing: IssuerSigning::Certified {
            seed_hash: hash("seed_hash"),
        },
        generation: 1,
        key_binding_id: hash("certificate_hash"),
        expires_at_ns: 200,
    }
}
fn approved(f: &Fixture) -> (MemoryIssuer, IssuerTicket) {
    let cfg = IssuerConfig {
        audience: f.root.cert.delegation.audience,
        issuer: issuer(),
        max_logins: 16,
        max_operations: 16,
        max_operations_per_account: 4,
        max_devices_per_account: 2,
        max_result_bytes: 1_000_000,
        issuance_burst: 32,
        issuance_refill_interval_ns: 1,
    };
    let policy = IssuerPolicy {
        login_ttl_ns: 90,
        proof_ttl_ns: 40,
        receipt_timeout_ns: 40,
        reconciliation_retention_ns: 20,
    };
    let mut auth = MemoryIssuer::new_fresh(cfg, 0).unwrap();
    let account = Principal::from_slice(&[5; 10]);
    let mut request = IssueRequest {
        epoch: 1,
        operation_id: [1; 32],
        login_epoch: 1,
        login_id: [1; 32],
        browser_public_key: [9; 32],
        grants: f.root.cert.delegation.grants.clone(),
    };
    // Frozen claims select slot 1 / generation 7. Reserve six other bindings;
    // the sixth occupies this account's slot 0. No fixture bytes come from Auth.
    for n in 0..6 {
        let owner = if n == 5 {
            account
        } else {
            Principal::from_slice(&[20 + n])
        };
        auth.prepare_issue(owner, &request, policy, authority(), 100)
            .unwrap();
    }
    request.browser_public_key = [
        0x03, 0xa1, 0x07, 0xbf, 0xf3, 0xce, 0x10, 0xbe, 0x1d, 0x70, 0xdd, 0x18, 0xe7, 0x4b, 0xc0,
        0x99, 0x67, 0xe4, 0xd6, 0x30, 0x9b, 0xa5, 0x0d, 0x5f, 0x1d, 0xdc, 0x86, 0x64, 0x12, 0x55,
        0x31, 0xb8,
    ];
    request.operation_id = [2; 32];
    request.login_id = [2; 32];
    auth.prepare_issue(account, &request, policy, authority(), 100)
        .unwrap();
    request.operation_id = [3; 32];
    let t = auth
        .prepare_issue(account, &request, policy, authority(), 140)
        .unwrap();
    auth.record_nonce(t, authority(), [6; 16], 140).unwrap();
    auth.record_approval(t, authority(), [7; 32], 140).unwrap();
    (auth, t)
}

fn signed_witness(witness: &HashTree, time: u8) -> Vec<u8> {
    let tree = fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                issuer().as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(witness.digest())),
            ),
        ),
        labeled(b"time".to_vec(), leaf(vec![time, 1])), // Fixture times 150/151, canonical LEB128.
    );
    let signed = [b"\x0Dic-state-root".as_slice(), &tree.digest()].concat();
    let certificate = Certificate {
        tree,
        signature: bls_key().sign(&signed).serialize().to_vec(),
        delegation: None,
    };
    encode_canister_signature(
        &encode_certificate(&certificate, 16384).unwrap(),
        witness,
        16384,
    )
    .unwrap()
}

#[test]
fn certified_issuance_publishes_the_frozen_leaf_and_fresh_witnesses_keep_identity() {
    let mut f = Fixture::new();
    let (mut auth, ticket) = approved(&f);
    let mut published = None;
    let credential = auth
        .complete_certified(ticket, authority(), 140, |p| {
            assert_eq!(p.claims, bytes("claims"));
            assert_eq!(p.seed_hash, sha(SEED));
            assert_eq!(
                p.message_hash,
                sha(&[
                    b"\x1bic-auth-portable-credential".as_slice(),
                    &hash("claims_digest")
                ]
                .concat())
            );
            // Compose an independent host-owned application branch with the actual
            // publication input. The supplied leaf drives the certificate below.
            published = Some(fork(
                labeled(b"app".to_vec(), leaf(vec![42])),
                labeled(
                    b"sig".to_vec(),
                    labeled(p.seed_hash, labeled(p.message_hash, leaf(Vec::new()))),
                ),
            ));
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(credential.claims, bytes("claims"));
    assert_eq!(
        credential.proof,
        IssuedProof::Certified {
            seed_hash: sha(SEED)
        }
    );
    let original = credential.claims.to_vec();
    let witness = published.unwrap();
    let mut proofs = Vec::new();
    for time in [150, 151] {
        f.cbor = signed_witness(&witness, time as u8);
        f.claims = auth
            .complete_certified(ticket, authority(), time, |_| -> Result<(), ()> {
                panic!("witness refresh must not publish again")
            })
            .unwrap()
            .claims
            .to_vec();
        assert_eq!(f.claims, original);
        let grants = grants(&f);
        let mut h = host(&f, &grants);
        h.claims.now_ns = time;
        assert_eq!(result(&f, &h, &mut budget(10)), Ok(hash("claims_digest")));
        proofs.push(f.cbor.clone());
    }
    assert_ne!(proofs[0], proofs[1]);
}
