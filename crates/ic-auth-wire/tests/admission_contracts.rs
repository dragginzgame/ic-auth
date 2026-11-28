//! Independent browser frames fix every nested field and discriminant order.

use ic_auth_protocol_types::Principal;
use ic_auth_wire::{
    BROWSER_DEFAULT_CREDENTIAL_BYTES, BROWSER_MAX_CONFIGURED_CREDENTIAL_BYTES, CredentialEnvelope,
    EstablishSession, SessionInput, SessionView,
    model::{
        AdmissionRefusal, AudienceId, CertifiedProof, ChainKeyAlgorithm, ChainKeyBatchHeaderV1,
        ChainKeyBatchWitnessStepV1, ChainKeyBatchWitnessV1, ChainKeyDelegationCertV1,
        ChainKeyKeyId, ChainKeyRootSignatureV1, DelegatedRoleGrant, DelegationAudience,
        DelegationCert, DelegationProof, IcCanisterSignatureProofV1,
        IcChainKeyBatchSignatureProofV1, IssuerProofAlgorithm, IssuerProofBinding,
        PortableCredential, PortableProof, RootProof,
    },
    wire::{self, Bytes, Direction, Encoding, Error, LimitKind, List, Text},
};

fn fixture(name: &str) -> Vec<u8> {
    let (_, hex) = include_str!("fixtures/admission-frames.tsv")
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .find(|(key, _)| *key == name)
        .expect("named browser fixture");
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn bytes<const N: usize>(input: &[u8]) -> Bytes<N> {
    Bytes::new(input.to_vec()).unwrap()
}

fn text<const N: usize>(input: &str) -> Text<N> {
    Text::new(input.into()).unwrap()
}

fn audience() -> DelegationAudience {
    DelegationAudience::Fleet(AudienceId {
        canonical_network_id: [1; 32],
        fleet_id: [2; 32],
    })
}

fn grants() -> List<DelegatedRoleGrant, 16> {
    List::new(vec![DelegatedRoleGrant {
        target: text("project_instance"),
        scopes: List::new(vec![text("write"), text("read")]).unwrap(),
    }])
    .unwrap()
}

fn ed25519() -> PortableCredential {
    PortableCredential {
        claims: bytes(&[0, 128, 255]),
        proof: PortableProof::Ed25519(Bytes::new((0..64).collect()).unwrap()),
    }
}

fn certified() -> PortableCredential {
    PortableCredential {
        claims: bytes(&[128, 0, 255, 1]),
        proof: PortableProof::Certified(CertifiedProof {
            delegation: DelegationProof {
                cert: DelegationCert {
                    root_pid: Principal::from_slice(&[3]),
                    issuer_pid: Principal::from_slice(&[4]),
                    issuer_proof_alg: IssuerProofAlgorithm::IcCanisterSignatureV1,
                    issuer_proof_binding_hash: [5; 32],
                    issuer_proof_binding: IssuerProofBinding::IcCanisterSignatureV1([6; 32]),
                    issued_at_ns: 127,
                    not_before_ns: 128,
                    expires_at_ns: 129,
                    max_token_ttl_ns: 130,
                    aud: audience(),
                    grants: grants(),
                },
                root_proof: RootProof::IcChainKeyBatchSignatureV1(
                    IcChainKeyBatchSignatureProofV1 {
                        header: ChainKeyBatchHeaderV1 {
                            schema_version: 257,
                            root_canister_id: Principal::from_slice(&[7]),
                            batch_id: [8; 32],
                            proof_epoch: 131,
                            registry_epoch: 132,
                            registry_hash: [9; 32],
                            tree_root: [10; 32],
                            not_before_ns: 133,
                            expires_at_ns: 134,
                            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
                            key_id: ChainKeyKeyId {
                                name: text("root-key"),
                            },
                            derivation_path_hash: [11; 32],
                            key_version: 135,
                        },
                        delegation_cert: ChainKeyDelegationCertV1 {
                            root_canister_id: Principal::from_slice(&[12]),
                            issuer_canister_id: Principal::from_slice(&[13]),
                            proof_epoch: 136,
                            issuer_proof_algorithm: IssuerProofAlgorithm::IcCanisterSignatureV1,
                            issuer_proof_binding_hash: [14; 32],
                            issuer_proof_binding: IssuerProofBinding::IcCanisterSignatureV1(
                                [15; 32],
                            ),
                            max_token_ttl_ns: 137,
                            audience: audience(),
                            grants: grants(),
                            not_before_ns: 138,
                            expires_at_ns: 139,
                            registry_epoch: 140,
                            registry_hash: [16; 32],
                        },
                        issuer_witness: ChainKeyBatchWitnessV1 {
                            steps: List::new(vec![
                                ChainKeyBatchWitnessStepV1::LeftSibling([17; 32]),
                                ChainKeyBatchWitnessStepV1::RightSibling([18; 32]),
                            ])
                            .unwrap(),
                        },
                        signature: ChainKeyRootSignatureV1 {
                            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
                            key_id: ChainKeyKeyId {
                                name: text("signature-key"),
                            },
                            derivation_path: List::new(vec![bytes(&[]), bytes(&[0, 128, 255])])
                                .unwrap(),
                            public_key: bytes(&[19, 20]),
                            signature: bytes(&[21, 22, 23]),
                        },
                    },
                ),
            },
            issuer_signature: IcCanisterSignatureProofV1 {
                signature_cbor: bytes(&[24, 25, 26, 27]),
                public_key_der: bytes(&[28, 29]),
            },
        }),
    }
}

#[test]
fn both_proofs_cross_encode_every_nested_field_with_current_fixtures() {
    for (name, value) in [("ed25519", ed25519()), ("certified", certified())] {
        let request = fixture(&format!("credential-{name}-request"));
        let response = fixture(&format!("credential-{name}-response"));
        assert_eq!(CredentialEnvelope::encode_request(&value).unwrap(), request);
        assert_eq!(CredentialEnvelope::decode_request(&request).unwrap(), value);
        assert_eq!(
            CredentialEnvelope::encode_response(&value).unwrap(),
            response
        );
        assert_eq!(
            CredentialEnvelope::decode_response(&response).unwrap(),
            value
        );
    }
    let empty = PortableCredential {
        claims: bytes(&[]),
        proof: PortableProof::Ed25519(bytes(&[])),
    };
    assert_eq!(
        CredentialEnvelope::encode_request(&empty).unwrap(),
        fixture("credential-empty")
    );
    assert_eq!(
        CredentialEnvelope::decode_request(&fixture("credential-empty")).unwrap(),
        empty
    );
}

#[test]
fn admission_preserves_unsigned_scope_text_and_optional_ttl() {
    for (name, credential, scopes, ttl_secs) in [
        (
            "establish-none",
            ed25519(),
            ["write", "read", "write"],
            None,
        ),
        (
            "establish-some",
            certified(),
            ["", "Read", "read::write"],
            Some(u64::MAX),
        ),
    ] {
        let input = SessionInput {
            credential,
            scopes: List::new(scopes.into_iter().map(text).collect()).unwrap(),
            ttl_secs,
        };
        let frame = fixture(name);
        assert_eq!(EstablishSession::encode_request(&input).unwrap(), frame);
        assert_eq!(EstablishSession::decode_request(&frame).unwrap(), input);
    }
}

#[test]
fn every_admission_reply_matches_current_fixtures() {
    let success = Ok(SessionView {
        ext: None,
        caller: Principal::from_slice(&[30]),
        issuer: Principal::from_slice(&[31]),
        subject: Principal::from_slice(&[32]),
        expires_at_ns: u64::MAX,
    });
    let mut cases = vec![("establish-success", success)];
    cases.extend([
        ("establish-NotYetValid", Err(AdmissionRefusal::NotYetValid)),
        ("establish-Expired", Err(AdmissionRefusal::Expired)),
        (
            "establish-InvalidCredential",
            Err(AdmissionRefusal::InvalidCredential),
        ),
        (
            "establish-AuthorityUnavailable",
            Err(AdmissionRefusal::AuthorityUnavailable),
        ),
        ("establish-BeforeReset", Err(AdmissionRefusal::BeforeReset)),
        (
            "establish-ScopeRejected",
            Err(AdmissionRefusal::ScopeRejected),
        ),
        (
            "establish-ReplayConflict",
            Err(AdmissionRefusal::ReplayConflict),
        ),
        ("establish-Revoked", Err(AdmissionRefusal::Revoked)),
        ("establish-Capacity", Err(AdmissionRefusal::Capacity)),
        ("establish-Throttled", Err(AdmissionRefusal::Throttled)),
        ("establish-Unavailable", Err(AdmissionRefusal::Unavailable)),
    ]);
    for (name, value) in cases {
        let frame = fixture(name);
        assert_eq!(
            EstablishSession::encode_response(&value).unwrap(),
            frame,
            "{name}"
        );
        assert_eq!(
            EstablishSession::decode_response(&frame).unwrap(),
            value,
            "{name}"
        );
    }
}

#[test]
fn identities_allowances_and_browser_caps_remain_separate() {
    assert_eq!(CredentialEnvelope::ID, 0x415554480101);
    assert_eq!(CredentialEnvelope::NAME, "credential");
    assert_eq!(EstablishSession::ID, 0x415554480112);
    assert_eq!(EstablishSession::NAME, "establish_portable_session");
    for limits in [
        CredentialEnvelope::REQUEST_LIMITS,
        CredentialEnvelope::RESPONSE_LIMITS,
        EstablishSession::REQUEST_LIMITS,
    ] {
        assert_eq!(
            (limits.bytes, limits.allocation, limits.nodes, limits.depth),
            (524288, 2097152, 65536, 32)
        );
    }
    let reply = EstablishSession::RESPONSE_LIMITS;
    assert_eq!(
        (reply.bytes, reply.allocation, reply.nodes, reply.depth),
        (8192, 131072, 128, 8)
    );
    assert_eq!(BROWSER_DEFAULT_CREDENTIAL_BYTES, 131072);
    assert_eq!(BROWSER_MAX_CONFIGURED_CREDENTIAL_BYTES, 1048576);
}

#[test]
fn malformed_frames_refuse_without_fallback() {
    for name in ["credential-ed25519-request", "credential-certified-request"] {
        let frame = fixture(name);
        for end in 0..frame.len() {
            assert!(
                CredentialEnvelope::decode_request(&frame[..end]).is_err(),
                "{name}:{end}"
            );
        }
        for offset in [0, 1, 2, 9, 10] {
            let mut changed = frame.clone();
            changed[offset] ^= 1;
            assert!(CredentialEnvelope::decode_request(&changed).is_err());
        }
        assert!(EstablishSession::decode_request(&frame).is_err());
        let mut trailing = frame;
        trailing.push(0);
        assert!(CredentialEnvelope::decode_request(&trailing).is_err());
    }
    for name in ["establish-none", "establish-some"] {
        let frame = fixture(name);
        for end in 0..frame.len() {
            assert!(EstablishSession::decode_request(&frame[..end]).is_err());
        }
        assert!(CredentialEnvelope::decode_request(&frame).is_err());
        let mut bad_option = frame;
        *bad_option.last_mut().unwrap() = 2;
        assert!(EstablishSession::decode_request(&bad_option).is_err());
    }
    let mut bad_proof = fixture("credential-empty");
    bad_proof[12] = 2;
    assert!(CredentialEnvelope::decode_request(&bad_proof).is_err());
    let mut bad_reply = fixture("establish-Unavailable");
    bad_reply[12] = 11;
    assert!(EstablishSession::decode_response(&bad_reply).is_err());
    let mut oversized = fixture("credential-empty");
    oversized.resize(524289, 0);
    assert_eq!(
        CredentialEnvelope::decode_request(&oversized),
        Err(Error::Limit(LimitKind::Bytes))
    );
}

#[test]
fn outer_length_normalization_never_changes_signed_claims() {
    let canonical = fixture("credential-ed25519-request");
    let mut redundant = canonical.clone();
    redundant.splice(11..12, [0x83, 0]);
    let decoded = CredentialEnvelope::decode_request(&redundant).unwrap();
    assert_eq!(decoded, ed25519());
    assert_eq!(
        CredentialEnvelope::encode_request(&decoded).unwrap(),
        canonical
    );
    for invalid_length in [vec![0x80; 10], vec![0xff; 10], vec![0x81, 0x80, 4]] {
        let mut changed = canonical.clone();
        changed.splice(11..12, invalid_length);
        assert!(CredentialEnvelope::decode_request(&changed).is_err());
    }
}

// Use an unbounded test core under the supplier's sparse root to submit fields
// larger than the typed constructors accept.
fn raw_frame(value: &impl serde::Serialize, encoding: Encoding, limits: wire::Limits) -> Vec<u8> {
    let mut frame = Vec::new();
    wire::evolution::encode_projected_into(&mut frame, encoding, limits, |_| {
        Ok(wire::evolution::Projected::empty(value))
    })
    .unwrap();
    frame
}

#[test]
fn decoder_enforces_claim_signature_scope_and_text_bounds() {
    let limits = CredentialEnvelope::REQUEST_LIMITS;
    let credential_encoding = Encoding::Frame {
        method: CredentialEnvelope::ID,
        direction: Direction::Request,
    };
    for size in [65535, 65536, 65537] {
        let raw = (
            wire::ByteBuf::from(vec![255_u8; size]),
            PortableProof::Ed25519(bytes(&[])),
        );
        let frame = raw_frame(&raw, credential_encoding, limits);
        assert_eq!(
            CredentialEnvelope::decode_request(&frame).is_ok(),
            size <= 65536
        );
    }

    // A single-variant fixture serializes tag zero; set the fixed Ed25519 tag one.
    for size in [63, 64, 65] {
        let frame = raw_frame(
            &(
                Vec::<u8>::new(),
                (1u8, wire::ByteBuf::from(vec![0u8; size])),
            ),
            credential_encoding,
            limits,
        );
        assert_eq!(
            CredentialEnvelope::decode_request(&frame).is_ok(),
            size <= 64
        );
    }
    let establish_encoding = Encoding::Frame {
        method: EstablishSession::ID,
        direction: Direction::Request,
    };
    for count in [15, 16, 17] {
        let frame = raw_frame(
            &((ed25519(), 0u8), vec!["read"; count], None::<u64>),
            establish_encoding,
            limits,
        );
        assert_eq!(
            EstablishSession::decode_request(&frame).is_ok(),
            count <= 16
        );
    }
    for length in [63, 64, 65] {
        let frame = raw_frame(
            &((ed25519(), 0u8), vec!["a".repeat(length)], None::<u64>),
            establish_encoding,
            limits,
        );
        assert_eq!(
            EstablishSession::decode_request(&frame).is_ok(),
            length <= 64
        );
    }
}

#[test]
fn aggregate_allowance_and_browser_retention_are_independent() {
    let mut credential = certified();
    let PortableProof::Certified(proof) = &mut credential.proof else {
        unreachable!()
    };
    proof.issuer_signature.signature_cbor = Bytes::new(vec![5; 131072]).unwrap();
    let frame = CredentialEnvelope::encode_request(&credential).unwrap();
    assert!(frame.len() > BROWSER_DEFAULT_CREDENTIAL_BYTES);
    assert!(frame.len() < CredentialEnvelope::REQUEST_LIMITS.bytes);
    assert_eq!(
        CredentialEnvelope::decode_request(&frame).unwrap(),
        credential
    );
    // Both fields are within their individual bounds; their combined allocation
    // charge exceeds the complete-frame allowance. It must not be relaxed.
    credential.claims = Bytes::new(vec![255; 65536]).unwrap();
    assert_eq!(
        CredentialEnvelope::encode_request(&credential),
        Err(Error::Limit(LimitKind::Allocation))
    );
}
