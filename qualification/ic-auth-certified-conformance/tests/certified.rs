//! Complete certified proof from independent root/claims bytes and real BLS.
use ic_auth::{
    claims::{decode_claims, encode_claims},
    delegation_encoding::certificate_hash,
    issuer_signature::IssuerSignatureError as I,
    root_delegation::DelegationError as R,
    verification::{
        AuthorityGrants, BudgetError, CertifiedContext, CertifiedError as E, CertifiedLimits,
        CertifiedProof, ContextError as C, VerificationBudget, VerificationContext,
        VerificationCosts, verify_certified,
    },
};
use ic_auth_protocol_types::{MerkleStep, PortableMechanism, Principal};
use ic_platform::{
    bls::PrivateKey,
    canister_sig_creation::CanisterSigPublicKey,
    certification::{
        Certificate,
        cbor::{encode_canister_signature, encode_certificate},
        fork, labeled, leaf,
    },
};
use sha2::{Digest, Sha256};
#[cfg(feature = "receiver-memory")]
mod memory;
mod support;
use root_fixture::{Fixture as RootFixture, issuer, limits, policy};
use support::root as root_fixture;
const RECORDS: &str = include_str!("fixtures/certified.tsv");
fn bytes(name: &str) -> Vec<u8> {
    root_fixture::bytes(RECORDS, name)
}
fn hash(name: &str) -> [u8; 32] {
    bytes(name).try_into().unwrap()
}
const SEED: &[u8] = b"auth-qualification-issuer";
fn bls_key() -> PrivateKey {
    let mut s = [0; 32];
    s[31] = 1;
    PrivateKey::deserialize(&s).unwrap()
}
fn sha(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
fn issuer_cbor(digest: [u8; 32], time: Vec<u8>) -> Vec<u8> {
    issuer_cbor_for(issuer(), digest, time)
}
fn issuer_cbor_for(issuer: Principal, digest: [u8; 32], time: Vec<u8>) -> Vec<u8> {
    // Literal domain and frozen digest: no Auth message or claims builder.
    let message = [b"\x1bic-auth-portable-credential".as_slice(), &digest].concat();
    let witness = labeled(
        b"sig".to_vec(),
        labeled(sha(SEED), labeled(sha(&message), leaf(Vec::new()))),
    );
    let tree = fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                issuer.as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(witness.digest())),
            ),
        ),
        labeled(b"time".to_vec(), leaf(time)),
    );
    let signed = [b"\x0Dic-state-root".as_slice(), &tree.digest()].concat();
    let cert = Certificate {
        tree,
        signature: bls_key().sign(&signed).serialize().to_vec(),
        delegation: None,
    };
    encode_canister_signature(&encode_certificate(&cert, 16384).unwrap(), &witness, 16384).unwrap()
}
type FixtureChange = fn(&mut Fixture);
struct Fixture {
    root: RootFixture,
    claims: Vec<u8>,
    cbor: Vec<u8>,
    der: Vec<u8>,
    network: [u8; 96],
}
impl Fixture {
    fn new() -> Self {
        Self {
            root: RootFixture::new(RECORDS, hash("seed_hash")),
            claims: bytes("claims"),
            cbor: issuer_cbor(hash("claims_digest"), vec![0x96, 1]),
            der: CanisterSigPublicKey::new(issuer(), SEED.to_vec()).to_der(),
            network: bls_key().public_key().serialize(),
        }
    }
    fn proof(&self) -> CertifiedProof<'_> {
        CertifiedProof {
            delegation: self.root.proof(),
            issuer_signature_cbor: &self.cbor,
            issuer_public_key_der: &self.der,
        }
    }
}
fn grants(f: &Fixture) -> AuthorityGrants {
    AuthorityGrants::new(f.root.cert.delegation.grants.clone()).unwrap()
}
fn host<'a>(trusted: &'a Fixture, grants: &'a AuthorityGrants) -> CertifiedContext<'a> {
    CertifiedContext {
        claims: VerificationContext {
            audience: &trusted.root.cert.delegation.audience,
            issuer: issuer(),
            mechanism: PortableMechanism::Certified,
            key_binding_id: hash("certificate_hash"),
            grants,
            caller: Principal::from_slice(&bytes("caller")),
            now_ns: 150,
            authority_not_before_ns: 100,
            authority_expires_at_ns: 200,
            enrollment_expires_at_ns: 210,
            min_login_started_at_ns: 100,
            min_proof_issued_at_ns: 100,
            max_login_ttl_ns: 100,
            max_proof_ttl_ns: 60,
        },
        root_key: policy(&trusted.root.key),
        ic_root_public_key_raw: &trusted.network,
        limits: CertifiedLimits {
            delegation: limits(),
            max_issuer_signature_bytes: 16384,
            max_certificate_age_ns: 20,
        },
    }
}
fn budget(capacity: u32) -> VerificationBudget {
    VerificationBudget::new(
        capacity,
        1,
        VerificationCosts {
            ed25519: 2,
            certified: 3,
        },
        0,
    )
    .unwrap()
}
fn result(
    f: &Fixture,
    h: &CertifiedContext<'_>,
    b: &mut VerificationBudget,
) -> Result<[u8; 32], E> {
    verify_certified(&f.claims, f.proof(), h, b).map(|v| v.fingerprint())
}

#[test]
fn both_signatures_yield_shared_immutable_claims_with_one_charge() {
    let f = Fixture::new();
    let g = grants(&f);
    let h = host(&f, &g);
    let mut b = budget(3);
    let verified = verify_certified(&f.claims, f.proof(), &h, &mut b).unwrap();
    assert_eq!(verified.fingerprint(), hash("claims_digest"));
    assert_eq!(verified.claims(), &decode_claims(&bytes("claims")).unwrap());
    assert_eq!(verified.caller(), Principal::from_slice(&bytes("caller")));
    assert_eq!(verified.verified_at_ns(), 150);
    assert_eq!(b.available(), 0);
    assert_eq!(
        result(&f, &h, &mut b),
        Err(E::Budget(BudgetError::Exhausted))
    );
}

#[test]
fn aggregate_and_independent_limits_precede_decode_and_charging() {
    let f = Fixture::new();
    let g = grants(&f);
    let mut h = host(&f, &g);
    let exact = f.claims.len() + f.cbor.len() + f.der.len() + 230; // Independent root fixture accounting.
    h.limits.max_issuer_signature_bytes = f.cbor.len();
    for (cap, expected) in [
        (exact - 1, Err(E::InputTooLarge)),
        (exact, Ok(hash("claims_digest"))),
        (exact + 1, Ok(hash("claims_digest"))),
    ] {
        h.limits.delegation.max_variable_bytes = cap;
        let mut b = budget(3);
        assert_eq!(result(&f, &h, &mut b), expected);
        if expected.is_err() {
            assert_eq!(b, budget(3));
        }
    }
    h.limits.max_issuer_signature_bytes = f.cbor.len() - 1;
    let mut b = budget(3);
    assert_eq!(result(&f, &h, &mut b), Err(E::InputTooLarge));
    assert_eq!(b, budget(3));
    let mut candidate = Fixture::new();
    candidate.claims = vec![0; 131073];
    h.limits.delegation.max_variable_bytes = usize::MAX;
    assert_eq!(result(&candidate, &h, &mut b), Err(E::InputTooLarge));
    assert_eq!(b, budget(3));
    h.limits.max_issuer_signature_bytes = 16384;
    h.limits.delegation.max_witness_steps = 1;
    candidate.claims = vec![]; // Witness cap must win over malformed claims.
    assert_eq!(result(&candidate, &h, &mut b), Err(E::InputTooLarge));
    assert_eq!(b, budget(3));
    h.limits.delegation.max_witness_steps = 64;
    assert!(matches!(
        result(&candidate, &h, &mut b),
        Err(E::Encoding(_))
    ));
    assert_eq!(b, budget(3));
}

#[test]
fn common_context_and_certificate_bindings_refuse_without_observing_time() {
    let f = Fixture::new();
    let g = grants(&f);
    type Change = for<'a> fn(&mut CertifiedContext<'a>);
    let cases: [(Change, E); 7] = [
        (
            |h| h.claims.caller = issuer(),
            E::Context(C::CallerMismatch),
        ),
        (
            |h| h.claims.mechanism = PortableMechanism::Ed25519,
            E::BindingMismatch,
        ),
        (|h| h.claims.now_ns = 180, E::Context(C::Expired)),
        (
            |h| h.claims.min_proof_issued_at_ns = 141,
            E::Context(C::BeforeReset),
        ),
        (
            |h| h.claims.authority_not_before_ns = 99,
            E::BindingMismatch,
        ),
        (
            |h| h.claims.authority_expires_at_ns = 201,
            E::BindingMismatch,
        ),
        (|h| h.root_key.accept_until_ns = 199, E::BindingMismatch),
    ];
    for (change, error) in cases {
        let mut h = host(&f, &g);
        change(&mut h);
        let mut b = budget(10);
        assert_eq!(result(&f, &h, &mut b), Err(error));
        assert_eq!(b, budget(10));
    }
    let mut candidate = Fixture::new();
    candidate.root.cert.issued_at_ns = 99;
    let mut b = budget(10);
    assert_eq!(
        result(&candidate, &host(&f, &g), &mut b),
        Err(E::BindingMismatch)
    );
    assert_eq!(b, budget(10));
}

#[test]
fn delegated_grants_and_token_lifetime_constrain_independently_of_enrollment() {
    let trusted = Fixture::new();
    let mut candidate = Fixture::new();
    // Negative inputs only may use the replacement codec; valid signed fixture
    // bytes/digests above are independently retained, never generated by Auth.
    let mut broad = trusted.root.cert.delegation.grants.clone();
    broad[0].scopes.insert(0, "admin".parse().unwrap());
    let broad = AuthorityGrants::new(broad).unwrap();
    let mut h = host(&trusted, &broad);
    let mut claims = decode_claims(&candidate.claims).unwrap();
    claims.grants[0].scopes = vec!["admin".parse().unwrap()];
    candidate.claims = encode_claims(&claims).unwrap();
    let mut b = budget(10);
    assert_eq!(result(&candidate, &h, &mut b), Err(E::BindingMismatch));
    assert_eq!(b, budget(10));
    candidate = Fixture::new();
    candidate.root.cert.delegation.max_token_ttl_ns = 39;
    let binding = certificate_hash(&candidate.root.cert).unwrap();
    h.claims.key_binding_id = binding;
    let mut claims = decode_claims(&candidate.claims).unwrap();
    claims.key_binding_id = binding;
    candidate.claims = encode_claims(&claims).unwrap();
    assert_eq!(result(&candidate, &h, &mut b), Err(E::BindingMismatch));
    assert_eq!(b, budget(10));
}

#[test]
fn every_root_refusal_retains_one_charge_and_precedes_issuer_verification() {
    let trusted = Fixture::new();
    let g = grants(&trusted);
    let mut h = host(&trusted, &g);
    let cases: [(FixtureChange, R); 4] = [
        (
            |f: &mut Fixture| f.root.signature = root_fixture::bytes(RECORDS, "high_signature"),
            R::HighSignatureScalar,
        ),
        (
            |f: &mut Fixture| f.root.signature = vec![0; 64],
            R::InvalidSignature,
        ),
        (
            |f: &mut Fixture| f.root.steps[0] = MerkleStep::Right([11; 32]),
            R::InvalidWitness,
        ),
        (
            |f: &mut Fixture| f.root.header.schema_version = 2,
            R::ProofBindingMismatch,
        ),
    ];
    for (change, error) in cases {
        let mut f = Fixture::new();
        change(&mut f);
        f.cbor.clear();
        let mut b = budget(10);
        assert_eq!(result(&f, &h, &mut b), Err(E::Root(error)));
        assert_eq!(b.available(), 7);
        assert_eq!(b.observed_at_ns(), 150);
    }
    h.root_key.min_proof_epoch = 3;
    let mut b = budget(10);
    assert_eq!(
        result(&trusted, &h, &mut b),
        Err(E::Root(R::StaleAuthority))
    );
    assert_eq!(b.available(), 7);
}

#[test]
fn issuer_refusals_and_changed_claims_retain_the_same_single_charge() {
    let trusted = Fixture::new();
    let g = grants(&trusted);
    let h = host(&trusted, &g);
    let cases: [(FixtureChange, I); 6] = [
        (|f| f.cbor.clear(), I::InvalidSignature),
        (
            |f| f.der = CanisterSigPublicKey::new(issuer(), b"wrong seed".to_vec()).to_der(),
            I::SeedMismatch,
        ),
        (
            |f| f.cbor = issuer_cbor([0; 32], vec![0x96, 1]),
            I::InvalidSignature,
        ),
        (
            |f| f.cbor = issuer_cbor(hash("claims_digest"), vec![0x81, 1]),
            I::CertificateTooOld,
        ),
        (
            |f| f.cbor = issuer_cbor(hash("claims_digest"), vec![0x97, 1]),
            I::CertificateInFuture,
        ),
        (
            |f| f.cbor = issuer_cbor(hash("claims_digest"), vec![]),
            I::InvalidCertificateTime,
        ),
    ];
    for (change, error) in cases {
        let mut f = Fixture::new();
        change(&mut f);
        let mut b = budget(10);
        assert_eq!(result(&f, &h, &mut b), Err(E::Issuer(error)));
        assert_eq!(b.available(), 7);
    }
    let mut f = Fixture::new();
    let mut c = decode_claims(&f.claims).unwrap();
    c.nonce[0] ^= 1;
    f.claims = encode_claims(&c).unwrap();
    let mut b = budget(10);
    assert_eq!(result(&f, &h, &mut b), Err(E::Issuer(I::InvalidSignature)));
    assert_eq!(b.available(), 7);
    let mut h = h;
    h.ic_root_public_key_raw = &[0; 95];
    let mut b = budget(10);
    assert_eq!(
        result(&trusted, &h, &mut b),
        Err(E::Issuer(I::InvalidRootKeyLength))
    );
    assert_eq!(b.available(), 7);
}

#[test]
fn budget_exhaustion_and_clock_rollback_cannot_reach_either_crypto_component() {
    let trusted = Fixture::new();
    let g = grants(&trusted);
    let mut h = host(&trusted, &g);
    let mut b = budget(3);
    assert!(result(&trusted, &h, &mut b).is_ok());
    let mut f = Fixture::new();
    f.root.signature = vec![0; 64];
    f.cbor.clear();
    assert_eq!(
        result(&f, &h, &mut b),
        Err(E::Budget(BudgetError::Exhausted))
    );
    h.claims.now_ns = 149;
    assert_eq!(
        result(&f, &h, &mut b),
        Err(E::Budget(BudgetError::ClockRollback))
    );
    assert_eq!(b.available(), 0);
    assert_eq!(b.observed_at_ns(), 150);
    h.claims.now_ns = 153;
    assert_eq!(result(&f, &h, &mut b), Err(E::Root(R::InvalidSignature)));
    assert_eq!(b.available(), 0);
    assert_eq!(b.observed_at_ns(), 153);
}

#[test]
#[cfg(feature = "ed25519-comparison")]
fn real_ed25519_and_certified_proofs_consume_one_shared_weighted_bucket() {
    use ic_auth::verification::{Ed25519Key, VerificationError, verify_ed25519};
    let f = Fixture::new();
    let g = grants(&f);
    let h = host(&f, &g);
    let mut b = budget(5);
    assert!(result(&f, &h, &mut b).is_ok());
    assert_eq!(b.available(), 2);
    let mut ed = h.claims;
    ed.mechanism = PortableMechanism::Ed25519;
    ed.key_binding_id = hash("ed_key_binding");
    let key = Ed25519Key::new(hash("ed_public_key")).unwrap();
    assert!(
        verify_ed25519(
            &bytes("ed_claims"),
            &bytes("ed_signature"),
            &ed,
            &key,
            &mut b
        )
        .is_ok()
    );
    assert_eq!(b.available(), 0);
    assert_eq!(
        verify_ed25519(
            &bytes("ed_claims"),
            &bytes("ed_signature"),
            &ed,
            &key,
            &mut b
        ),
        Err(VerificationError::Budget(BudgetError::Exhausted))
    );
    assert_eq!(
        result(&f, &h, &mut b),
        Err(E::Budget(BudgetError::Exhausted))
    );
}

// Qualification host driver: construct transport from independently signed
// fixture values, then decode/project before invoking the verifier explicitly.
fn wire_credential(f: &Fixture) -> ic_auth_wire::model::PortableCredential {
    use ic_auth_wire::{
        model as w,
        wire::{Bytes, List, Text},
    };
    fn audience(d: &ic_auth_protocol_types::IssuerDelegation) -> w::DelegationAudience {
        w::DelegationAudience::Fleet(w::AudienceId {
            canonical_network_id: *d.audience.canonical_network_id.as_bytes(),
            fleet_id: *d.audience.fleet_id.as_bytes(),
        })
    }
    fn grants(d: &ic_auth_protocol_types::IssuerDelegation) -> List<w::DelegatedRoleGrant, 16> {
        List::new(
            d.grants
                .iter()
                .map(|g| w::DelegatedRoleGrant {
                    target: Text::new(g.target.as_str().into()).unwrap(),
                    scopes: List::new(
                        g.scopes
                            .iter()
                            .map(|s| Text::new(s.as_str().into()).unwrap())
                            .collect(),
                    )
                    .unwrap(),
                })
                .collect(),
        )
        .unwrap()
    }
    let c = &f.root.cert;
    let d = &c.delegation;
    let h = &f.root.header;
    let l = &f.root.leaf;
    let ld = &l.delegation;
    w::PortableCredential {
        claims: Bytes::new(f.claims.clone()).unwrap(),
        proof: w::PortableProof::Certified(Box::new(w::CertifiedProof {
            delegation: w::DelegationProof {
                cert: w::DelegationCert {
                    root_pid: d.root,
                    issuer_pid: d.issuer,
                    issuer_proof_alg: w::IssuerProofAlgorithm::IcCanisterSignatureV1,
                    issuer_proof_binding_hash: d.binding_hash,
                    issuer_proof_binding: w::IssuerProofBinding::IcCanisterSignatureV1(d.seed_hash),
                    issued_at_ns: c.issued_at_ns,
                    not_before_ns: d.not_before_ns,
                    expires_at_ns: d.expires_at_ns,
                    max_token_ttl_ns: d.max_token_ttl_ns,
                    aud: audience(d),
                    grants: grants(d),
                },
                root_proof: w::RootProof::IcChainKeyBatchSignatureV1(
                    w::IcChainKeyBatchSignatureProofV1 {
                        header: w::ChainKeyBatchHeaderV1 {
                            schema_version: h.schema_version,
                            root_canister_id: h.root,
                            batch_id: h.batch_id,
                            proof_epoch: h.proof_epoch,
                            registry_epoch: h.registry_epoch,
                            registry_hash: h.registry_hash,
                            tree_root: h.tree_root,
                            not_before_ns: h.not_before_ns,
                            expires_at_ns: h.expires_at_ns,
                            algorithm: w::ChainKeyAlgorithm::EcdsaSecp256k1,
                            key_id: w::ChainKeyKeyId {
                                name: Text::new(h.key_name.clone()).unwrap(),
                            },
                            derivation_path_hash: h.derivation_path_hash,
                            key_version: h.key_version,
                        },
                        delegation_cert: w::ChainKeyDelegationCertV1 {
                            root_canister_id: ld.root,
                            issuer_canister_id: ld.issuer,
                            proof_epoch: l.proof_epoch,
                            issuer_proof_algorithm: w::IssuerProofAlgorithm::IcCanisterSignatureV1,
                            issuer_proof_binding_hash: ld.binding_hash,
                            issuer_proof_binding: w::IssuerProofBinding::IcCanisterSignatureV1(
                                ld.seed_hash,
                            ),
                            max_token_ttl_ns: ld.max_token_ttl_ns,
                            audience: audience(ld),
                            grants: grants(ld),
                            not_before_ns: ld.not_before_ns,
                            expires_at_ns: ld.expires_at_ns,
                            registry_epoch: l.registry_epoch,
                            registry_hash: l.registry_hash,
                        },
                        issuer_witness: w::ChainKeyBatchWitnessV1 {
                            steps: List::new(
                                f.root
                                    .steps
                                    .iter()
                                    .map(|s| match s {
                                        MerkleStep::Left(h) => {
                                            w::ChainKeyBatchWitnessStepV1::LeftSibling(*h)
                                        }
                                        MerkleStep::Right(h) => {
                                            w::ChainKeyBatchWitnessStepV1::RightSibling(*h)
                                        }
                                    })
                                    .collect(),
                            )
                            .unwrap(),
                        },
                        signature: w::ChainKeyRootSignatureV1 {
                            algorithm: w::ChainKeyAlgorithm::EcdsaSecp256k1,
                            key_id: w::ChainKeyKeyId {
                                name: Text::new(f.root.name.clone()).unwrap(),
                            },
                            derivation_path: List::new(
                                f.root
                                    .path
                                    .iter()
                                    .map(|p| Bytes::new(p.clone()).unwrap())
                                    .collect(),
                            )
                            .unwrap(),
                            public_key: Bytes::new(f.root.key.clone()).unwrap(),
                            signature: Bytes::new(f.root.signature.clone()).unwrap(),
                        },
                    },
                ),
            },
            issuer_signature: w::IcCanisterSignatureProofV1 {
                signature_cbor: Bytes::new(f.cbor.clone()).unwrap(),
                public_key_der: Bytes::new(f.der.clone()).unwrap(),
            },
        })),
    }
}

#[test]
fn wire_admission_projects_into_real_certified_verification_and_unsigned_request_policy() {
    use ic_auth::session_request::SessionRequest;
    use ic_auth::verification::{VerificationProfile, verify_credential};
    use ic_auth_protocol_types::{Credential, CredentialProof};
    use ic_auth_wire::{
        EstablishSession, SessionInput,
        wire::{List, Text},
    };
    let f = Fixture::new();
    let g = grants(&f);
    let h = host(&f, &g);
    let frame = EstablishSession::encode_request(&SessionInput {
        credential: wire_credential(&f),
        scopes: List::new(vec![
            Text::new("write".into()).unwrap(),
            Text::new("read".into()).unwrap(),
        ])
        .unwrap(),
        ttl_secs: Some(30),
    })
    .unwrap();
    let received = EstablishSession::decode_request(&frame).unwrap();
    let request = SessionRequest::new(
        received
            .scopes
            .into_vec()
            .into_iter()
            .map(Text::into_string)
            .collect(),
        received.ttl_secs,
    )
    .unwrap();
    assert_eq!(
        request
            .scopes()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        ["read", "write"]
    );
    assert_eq!(request.ttl_secs(), Some(30));
    let credential = Credential::try_from(received.credential).unwrap();
    assert_eq!(credential.claims, bytes("claims"));
    let CredentialProof::Certified(material) = &credential.proof else {
        panic!("fixture is certified")
    };
    assert_eq!(material.delegation.certificate, f.root.cert);
    assert_eq!(material.delegation.header, f.root.header);
    assert_eq!(material.delegation.leaf, f.root.leaf);
    let view = CertifiedProof::from(material.as_ref());
    assert_eq!(
        view.issuer_signature_cbor.as_ptr(),
        material.issuer_signature_cbor.as_ptr()
    );
    let mut b = budget(3);
    let verified =
        verify_credential(&credential, VerificationProfile::Certified(&h), &mut b).unwrap();
    assert_eq!(verified.fingerprint(), hash("claims_digest"));
    assert_eq!(b.available(), 0);
    // Protected selection, not the projected variant, chooses the verifier.
    let mut wrong_host = h;
    wrong_host.claims.mechanism = PortableMechanism::Ed25519;
    let mut b = budget(3);
    assert_eq!(
        verify_certified(&credential.claims, view, &wrong_host, &mut b),
        Err(E::BindingMismatch)
    );
    assert_eq!(b, budget(3));
}

#[test]
fn wire_projection_preserves_binding_and_signature_failures_for_auth_admission() {
    use ic_auth::verification::{CredentialError, VerificationProfile, verify_credential};
    use ic_auth_protocol_types::Credential;
    use ic_auth_wire::CredentialEnvelope;
    let trusted = Fixture::new();
    let g = grants(&trusted);
    let h = host(&trusted, &g);
    type Change = fn(&mut Fixture);
    let cases: [(Change, E, u32); 2] = [
        (|f| f.root.cert.issued_at_ns = 99, E::BindingMismatch, 0),
        (
            |f| f.root.signature = vec![0; 64],
            E::Root(R::InvalidSignature),
            150,
        ),
    ];
    for (change, error, observed) in cases {
        let mut f = Fixture::new();
        change(&mut f);
        let frame = CredentialEnvelope::encode_request(&wire_credential(&f)).unwrap();
        let projected =
            Credential::try_from(CredentialEnvelope::decode_request(&frame).unwrap()).unwrap();
        let mut b = budget(3);
        assert_eq!(
            verify_credential(&projected, VerificationProfile::Certified(&h), &mut b),
            Err(CredentialError::Certified(error))
        );
        assert_eq!(b.observed_at_ns(), u64::from(observed));
    }
}

#[test]
#[cfg(feature = "ed25519-comparison")]
fn wire_ed25519_projection_preserves_independent_signature_and_preflight_refusal() {
    use ic_auth::verification::{
        CredentialError, Ed25519Key, VerificationError, VerificationProfile, verify_credential,
    };
    use ic_auth_protocol_types::Credential;
    use ic_auth_wire::{
        CredentialEnvelope,
        model::{PortableCredential, PortableProof},
        wire::Bytes,
    };
    let f = Fixture::new();
    let g = grants(&f);
    let mut h = host(&f, &g).claims;
    h.mechanism = PortableMechanism::Ed25519;
    h.key_binding_id = hash("ed_key_binding");
    let key = Ed25519Key::new(hash("ed_public_key")).unwrap();
    for len in [63, 64] {
        let frame = CredentialEnvelope::encode_request(&PortableCredential {
            claims: Bytes::new(bytes("ed_claims")).unwrap(),
            proof: PortableProof::Ed25519(
                Bytes::new(bytes("ed_signature")[..len].to_vec()).unwrap(),
            ),
        })
        .unwrap();
        let credential =
            Credential::try_from(CredentialEnvelope::decode_request(&frame).unwrap()).unwrap();
        let mut b = budget(3);
        let result = verify_credential(
            &credential,
            VerificationProfile::Ed25519 {
                context: &h,
                key: &key,
            },
            &mut b,
        );
        if len == 63 {
            assert_eq!(
                result,
                Err(CredentialError::Ed25519(
                    VerificationError::MalformedSignature
                ))
            );
            assert_eq!(b, budget(3));
        } else {
            assert!(result.is_ok());
            assert_eq!(b.available(), 1);
        }
    }
}

#[test]
fn session_deadline_is_shared_by_complete_proofs_and_captures_enrollment() {
    use ic_auth::session_request::{SessionRequest, SessionRequestError, SessionTtlPolicy};
    let f = Fixture::new();
    let g = grants(&f);
    let role = "app".parse().unwrap();
    let request = SessionRequest::new(vec!["write".into(), "read".into()], None).unwrap();
    let ttl = SessionTtlPolicy::new(30, 60).unwrap();
    // Independent fixture: original login ends at 190; proof ends at 180.
    for (key_end, enrollment_end, expected) in [(200, 210, 190), (185, 210, 185), (200, 175, 175)] {
        let mut h = host(&f, &g);
        h.claims.authority_expires_at_ns = key_end;
        h.claims.enrollment_expires_at_ns = enrollment_end;
        let verified = verify_certified(&f.claims, f.proof(), &h, &mut budget(3)).unwrap();
        assert_eq!(
            verified.session_deadline(&request, &role, ttl),
            Ok(expected)
        );
        assert_eq!(
            verified.session_deadline(&request, &"other".parse().unwrap(), ttl),
            Err(SessionRequestError::ScopeRejected),
        );
        #[cfg(feature = "ed25519-comparison")]
        {
            use ic_auth::verification::{Ed25519Key, verify_ed25519};
            let mut ed = h.claims;
            ed.mechanism = PortableMechanism::Ed25519;
            ed.key_binding_id = hash("ed_key_binding");
            let key = Ed25519Key::new(hash("ed_public_key")).unwrap();
            let verified_ed = verify_ed25519(
                &bytes("ed_claims"),
                &bytes("ed_signature"),
                &ed,
                &key,
                &mut budget(3),
            )
            .unwrap();
            assert_eq!(
                verified_ed.session_deadline(&request, &role, ttl),
                Ok(expected)
            );
        }
    }
}

#[test]
fn credential_profile_refuses_untrusted_mechanism_switches_before_decoding_or_charging() {
    use ic_auth::verification::{CredentialError, VerificationProfile, verify_credential};
    use ic_auth_protocol_types::{Credential, CredentialProof};
    let f = Fixture::new();
    let g = grants(&f);
    let h = host(&f, &g);
    let wrong = Credential {
        claims: vec![255],
        proof: CredentialProof::Ed25519(vec![]),
    };
    let mut b = budget(3);
    assert_eq!(
        verify_credential(&wrong, VerificationProfile::Certified(&h), &mut b),
        Err(CredentialError::ProofMismatch)
    );
    assert_eq!(b, budget(3));
    let credential = Credential::try_from(wire_credential(&f)).unwrap();
    let verified =
        verify_credential(&credential, VerificationProfile::Certified(&h), &mut b).unwrap();
    assert_eq!(verified.fingerprint(), hash("claims_digest"));
    assert_eq!(b.available(), 0);
    #[cfg(feature = "ed25519-comparison")]
    {
        use ic_auth::verification::Ed25519Key;
        let mut ed = h.claims;
        ed.mechanism = PortableMechanism::Ed25519;
        ed.key_binding_id = hash("ed_key_binding");
        let key = Ed25519Key::new(hash("ed_public_key")).unwrap();
        let mut credential = credential;
        credential.claims = vec![255];
        let mut b = budget(3);
        assert_eq!(
            verify_credential(
                &credential,
                VerificationProfile::Ed25519 {
                    context: &ed,
                    key: &key
                },
                &mut b
            ),
            Err(CredentialError::ProofMismatch)
        );
        assert_eq!(b, budget(3));
    }
}

#[cfg(feature = "receiver-memory")]
#[test]
fn memory_certified_admission_retains_failed_work_and_retries_after_proof_expiry() {
    use ic_auth::{
        receiver_memory::{AdmissionHost, MemoryConfig, MemoryReceiver},
        receiver_memory::{LoginLimits, SessionAuthority, SessionCapacity},
        session_request::{SessionRequest, SessionTtlPolicy},
        verification::VerificationProfile,
    };
    use ic_auth_protocol_types::{Credential, CredentialProof};
    let f = Fixture::new();
    let g = grants(&f);
    let h = host(&f, &g);
    let role = "app".parse().unwrap();
    let mut receiver = MemoryReceiver::new_fresh(
        MemoryConfig {
            enrollment: ic_auth::receiver_memory::EnrollmentBinding {
                receiver: Principal::from_slice(&[10]),
                authority: Principal::from_slice(&[11]),
            },
            unknown_logout: ic_auth::receiver_memory::UnknownLogoutLimits {
                max_denials: 1,
                capacity: 1,
                refill_interval_ns: 1,
            },
            audience: *h.claims.audience,
            login: LoginLimits {
                max_device_slots: 8,
                max_login_ttl_ns: 100,
                max_admitted_logins: 1,
            },
            capacity: SessionCapacity {
                max_sessions: 1,
                max_replays: 1,
                max_replays_per_account: 1,
                max_encoded_bytes: 4096,
            },
            verification_capacity: 10,
            refill_interval_ns: 1,
            verification_costs: VerificationCosts {
                ed25519: 2,
                certified: 3,
            },
        },
        0,
    )
    .unwrap();
    receiver.request_enrollment(0, [1; 32], 10).unwrap();
    receiver
        .install_enrollment(
            0,
            Principal::from_slice(&[11]),
            ic_auth::receiver_memory::EnrollmentMetadata {
                receiver: Principal::from_slice(&[10]),
                audience: *h.claims.audience,
                challenge: [1; 32],
                revision: 1,
                generation: 9,
                snapshot_digest: [2; 32],
                authority_digest: [3; 32],
                issued_at_ns: 0,
                expires_at_ns: 210,
            },
        )
        .unwrap();
    let input = |context| AdmissionHost {
        verifier: VerificationProfile::Certified(context),
        role: &role,
        ttl: SessionTtlPolicy::new(60, 1800).unwrap(),
        generation: 9,
        enabled: true,
        subject_admissible: true,
    };
    let request = || SessionRequest::new(vec!["read".into()], None).unwrap();
    let credential = Credential::try_from(wire_credential(&f)).unwrap();
    let account = (issuer(), Principal::from_slice(&[5; 10]));
    for break_root in [true, false] {
        let mut bad = credential.clone();
        let CredentialProof::Certified(proof) = &mut bad.proof else {
            panic!("certified fixture")
        };
        if break_root {
            proof.delegation.signature.signature[0] ^= 1;
        } else {
            proof.issuer_signature_cbor[0] ^= 1;
        }
        assert!(receiver.admit(&bad, request(), input(&h)).is_err());
        assert_eq!(receiver.known_logins(), 0);
        assert_eq!(receiver.usage(account).encoded_bytes, 0);
    }
    assert_eq!(receiver.verification_budget().available(), 4);
    let admitted = receiver.admit(&credential, request(), input(&h)).unwrap();
    assert!(!admitted.exact_retry);
    assert_eq!(admitted.receipt.expires_at_ns, 190);
    assert_eq!(receiver.verification_budget().available(), 1);
    let usage = receiver.usage(account);
    assert_eq!(usage.replays_for_account, 1);
    let page = receiver.inspect_sessions(None, 1, 1, 185).unwrap();
    assert_eq!((page.scanned, page.next_after), (1, None));
    assert_eq!(page.sessions.len(), 1);
    assert_eq!(page.sessions[0].mechanism, PortableMechanism::Certified);
    assert_eq!(
        page.sessions[0].expires_at_ns,
        admitted.receipt.expires_at_ns
    );
    assert_eq!(receiver.observed_at_ns(), 150);
    assert_eq!(receiver.verification_budget().available(), 1);
    let h = CertifiedContext {
        claims: VerificationContext {
            now_ns: 185,
            ..h.claims
        },
        ..h
    }; // proof expired at 180
    let mut retry = credential;
    retry.proof = CredentialProof::Ed25519(vec![]);
    let repeated = receiver.admit(&retry, request(), input(&h)).unwrap();
    assert!(repeated.exact_retry);
    assert_eq!(repeated.receipt, admitted.receipt);
    assert_eq!(receiver.usage(account), usage);
    assert_eq!(receiver.verification_budget().available(), 1);
    let authority = SessionAuthority {
        caller: h.claims.caller,
        now_ns: 185,
        audience: *h.claims.audience,
        issuer: h.claims.issuer,
        mechanism: h.claims.mechanism,
        key_binding_id: h.claims.key_binding_id,
        role: &role,
        grants: g.as_slice(),
        generation: 9,
        not_before_ns: 100,
        expires_at_ns: 200,
        enrollment_expires_at_ns: 210,
        min_login_started_at_ns: 100,
        max_session_ttl_secs: 1800,
        enabled: true,
        subject_admissible: true,
    };
    assert_eq!(
        receiver.read(&authority, &["read".parse().unwrap()]),
        Ok(admitted.receipt)
    );
    assert_eq!(
        receiver.authorize(&authority, &["read".parse().unwrap()], |receipt| {
            (receipt.issuer, receipt.subject)
        }),
        Ok(account)
    );
    // Qualification-only Canic-style result projection; Auth installs no endpoint
    // and the Wire crate acquires no dependency on the receiver implementation.
    fn wire_reply(
        result: Result<
            ic_auth::receiver_memory::LogoutReceipt,
            ic_auth::receiver_memory::LogoutError,
        >,
    ) -> ic_auth_wire::model::LogoutResult {
        use ic_auth::receiver_memory::{LogoutError as E, LogoutReceipt as R};
        use ic_auth_wire::model::{LogoutResult as W, LogoutUnavailable as U};
        match result {
            Ok(R::Revoked { deny_until_ns }) => W::Revoked(deny_until_ns),
            Ok(R::Unconfirmed { deny_until_ns }) => W::Unconfirmed(deny_until_ns),
            Err(E::InvalidCaller) => W::InvalidCaller,
            Err(E::Capacity) => W::Unavailable(U::Capacity),
            Err(E::Clock) => W::Unavailable(U::Clock),
            Err(E::Closed) => W::Unavailable(U::ResetPending),
            Err(E::InvalidFacts) => W::Unavailable(U::Storage),
        }
    }
    fn check_wire(
        result: Result<
            ic_auth::receiver_memory::LogoutReceipt,
            ic_auth::receiver_memory::LogoutError,
        >,
        payload: &[u8],
    ) {
        let reply = wire_reply(result);
        let frame = ic_auth_wire::Logout::encode_response(&reply).unwrap();
        // Independent fixed ID/direction and Postcard variant/deadline bytes.
        let mut expected = vec![b'I', b'W', 0x14, 0, 0x48, 0x54, 0x55, 0x41, 0, 0, 1];
        expected.extend_from_slice(payload);
        assert_eq!(frame, expected);
        assert_eq!(
            ic_auth_wire::Logout::decode_response(&frame).unwrap(),
            reply
        );
    }
    check_wire(receiver.logout(h.claims.caller, 185), &[0, 0xbe, 1]); // Revoked(190)
    let mut unknown_bytes = [6; 29];
    unknown_bytes[28] = 2;
    let unknown = Principal::from_slice(&unknown_bytes);
    check_wire(receiver.logout(unknown, 185), &[1, 0x9d, 2]); // Unconfirmed(285)
    check_wire(receiver.logout(unknown, 186), &[1, 0x9d, 2]); // same deadline, still unconfirmed
    check_wire(receiver.logout(Principal::anonymous(), 186), &[2]);
    check_wire(receiver.logout(unknown, 185), &[3, 2]); // clock rollback
    unknown_bytes[0] = 7;
    check_wire(
        receiver.logout(Principal::from_slice(&unknown_bytes), 186),
        &[3, 0],
    ); // capacity
    assert_eq!(receiver.verification_budget().available(), 1);

    assert_eq!(receiver.usage(account).replays, 1);
    let revoked_host = CertifiedContext {
        claims: VerificationContext {
            now_ns: 186,
            ..h.claims
        },
        ..h
    };
    assert_eq!(
        receiver.admit(&retry, request(), input(&revoked_host)),
        Err(ic_auth::receiver_memory::MemoryError::Policy(
            ic_auth::receiver_memory::SessionPolicyError::Revoked
        ))
    );
    assert_eq!(
        receiver.authorize(
            &SessionAuthority {
                now_ns: 186,
                ..authority
            },
            &[],
            |_| { panic!("certified revoked session reached application projection") }
        ),
        Err(ic_auth::receiver_memory::MemoryError::Policy(
            ic_auth::receiver_memory::SessionPolicyError::Revoked
        ))
    );
    receiver.close();
    check_wire(receiver.logout(unknown, 186), &[3, 3]); // recovery pending
}
