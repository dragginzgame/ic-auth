#![cfg(feature = "token-verification")]

use ic_auth::{
    canister_signature::{CanisterSignatureError, domain_separated_message},
    canonical::*,
    token::{
        RootKeyPolicy, TokenVerificationContext, TokenVerificationError as Error,
        TokenVerificationLimits, verify_token,
    },
};
use ic_auth_protocol_types::*;
use ic_canister_sig_creation::CanisterSigPublicKey;
use ic_certification::{Certificate, HashTree, fork, labeled, leaf};
use ic_verify_bls_signature::PrivateKey;
use k256::ecdsa::{Signature, SigningKey, signature::hazmat::PrehashSigner};
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};

const SEED: &[u8] = b"canic-issuer-delegated-token";
const NOW: u64 = 150;

#[cfg(feature = "sessions")]
mod sessions;

fn p(id: u8) -> Principal {
    Principal::from_slice(&[id; 10])
}
fn role() -> AuthRole {
    "app_service".parse().unwrap()
}
fn audience() -> AudienceId {
    AudienceId {
        canonical_network_id: CanonicalId::from_bytes([1; 32]),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    }
}
fn grant(scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role(),
        scopes: scopes.iter().map(|s| s.to_string()).collect(),
    }
}
fn root_signing_key() -> SigningKey {
    SigningKey::from_slice(&[7; 32]).unwrap()
}
fn bls_key(value: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = value;
    PrivateKey::deserialize(&bytes).unwrap()
}
fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn cbor<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut bytes);
    serializer.self_describe().unwrap();
    value.serialize(&mut serializer).unwrap();
    bytes
}
#[derive(Serialize)]
struct Envelope {
    certificate: ByteBuf,
    tree: HashTree,
}

fn issuer_proof(claims: &DelegatedTokenClaims, seed: &[u8], time: u64) -> IssuerProof {
    let message = domain_separated_message(SEED, claims_hash(claims).unwrap()).unwrap();
    let tree = labeled(
        b"sig".to_vec(),
        labeled(hash(seed), labeled(hash(&message), leaf(vec![]))),
    );
    let mut time_bytes = Vec::new();
    leb128::write::unsigned(&mut time_bytes, time).unwrap();
    let certificate_tree = fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                claims.issuer_pid.as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(tree.digest())),
            ),
        ),
        labeled(b"time".to_vec(), leaf(time_bytes)),
    );
    let mut message = b"\x0Dic-state-root".to_vec();
    message.extend_from_slice(&certificate_tree.digest());
    let certificate = Certificate {
        tree: certificate_tree,
        signature: bls_key(1).sign(&message).serialize().to_vec(),
        delegation: None,
    };
    IssuerProof::IcCanisterSignatureV1(IcCanisterSignatureProofV1 {
        signature_cbor: cbor(&Envelope {
            certificate: ByteBuf::from(cbor(&certificate)),
            tree,
        }),
        public_key_der: CanisterSigPublicKey::new(claims.issuer_pid, seed.to_vec()).to_der(),
    })
}

fn merkle_node(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut bytes = vec![1];
    bytes.extend_from_slice(&left);
    bytes.extend_from_slice(&right);
    hash(&bytes)
}

fn root_proof(cert: &DelegationCert, policy: &RootKeyPolicy) -> RootProof {
    let leaf = ChainKeyDelegationCertV1 {
        root_canister_id: cert.root_pid,
        issuer_canister_id: cert.issuer_pid,
        proof_epoch: 2,
        registry_epoch: 3,
        registry_hash: [8; 32],
        issuer_proof_algorithm: cert.issuer_proof_alg,
        issuer_proof_binding_hash: cert.issuer_proof_binding_hash,
        issuer_proof_binding: cert.issuer_proof_binding,
        max_token_ttl_ns: cert.max_token_ttl_ns,
        audience: cert.aud.clone(),
        grants: cert.grants.clone(),
        not_before_ns: cert.not_before_ns,
        expires_at_ns: cert.expires_at_ns,
    };
    let leaf_hash = chain_key_delegation_cert_hash(&leaf).unwrap();
    let header = ChainKeyBatchHeaderV1 {
        schema_version: 1,
        root_canister_id: cert.root_pid,
        batch_id: [7; 32],
        proof_epoch: 2,
        registry_epoch: 3,
        registry_hash: [8; 32],
        tree_root: merkle_node(merkle_node([42; 32], leaf_hash), [43; 32]),
        not_before_ns: cert.not_before_ns,
        expires_at_ns: cert.expires_at_ns,
        algorithm: policy.algorithm,
        key_id: policy.key_id.clone(),
        derivation_path_hash: policy.derivation_path_hash,
        key_version: policy.key_version,
    };
    let sig: Signature = root_signing_key()
        .sign_prehash(&chain_key_batch_header_hash(&header).unwrap())
        .unwrap();
    RootProof::IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1 {
        header,
        delegation_cert: leaf,
        issuer_witness: ChainKeyBatchWitnessV1 {
            steps: vec![
                ChainKeyBatchWitnessStepV1::LeftSibling([42; 32]),
                ChainKeyBatchWitnessStepV1::RightSibling([43; 32]),
            ],
        },
        signature: ChainKeyRootSignatureV1 {
            algorithm: policy.algorithm,
            key_id: policy.key_id.clone(),
            derivation_path: vec![b"canic".to_vec(), b"delegation".to_vec()],
            public_key: policy.public_key.clone(),
            signature: sig.to_bytes().to_vec(),
        },
    })
}

struct Fixture {
    token: DelegatedToken,
    policy: RootKeyPolicy,
    ic_root: [u8; 96],
    role: AuthRole,
    allowed: Vec<String>,
    required: Vec<String>,
}
impl Fixture {
    fn new() -> Self {
        let policy = RootKeyPolicy {
            root_canister_id: p(1),
            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
            key_id: ChainKeyKeyId {
                name: "test_key_1".into(),
            },
            derivation_path_hash: chain_key_derivation_path_hash(&[
                b"canic".to_vec(),
                b"delegation".to_vec(),
            ])
            .unwrap(),
            public_key: root_signing_key()
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes()
                .to_vec(),
            key_version: 4,
            min_accepted_key_version: 4,
            min_accepted_proof_epoch: 2,
            min_accepted_registry_epoch: 3,
            valid_from_ns: 1,
            accept_until_ns: 1000,
            max_revocation_latency_ns: 500,
        };
        let binding = IssuerProofBinding::IcCanisterSignatureV1 {
            seed_hash: hash(SEED),
        };
        let cert = DelegationCert {
            root_pid: p(1),
            issuer_pid: p(2),
            issuer_proof_alg: IssuerProofAlgorithm::IcCanisterSignatureV1,
            issuer_proof_binding_hash: issuer_proof_binding_hash(
                p(2),
                IssuerProofAlgorithm::IcCanisterSignatureV1,
                binding,
            )
            .unwrap(),
            issuer_proof_binding: binding,
            issued_at_ns: 100,
            not_before_ns: 100,
            expires_at_ns: 500,
            max_token_ttl_ns: 100,
            aud: DelegationAudience::Fleet(audience()),
            grants: vec![grant(&["read", "write"])],
        };
        let claims = DelegatedTokenClaims {
            presenter: p(3),
            subject: p(3),
            issuer_pid: p(2),
            cert_hash: cert_hash(&cert).unwrap(),
            issued_at_ns: 140,
            expires_at_ns: 220,
            aud: cert.aud.clone(),
            grants: vec![grant(&["read"])],
            nonce: [9; 16],
            ext: Some(vec![1, 2, 3]),
        };
        Self {
            token: DelegatedToken {
                issuer_proof: issuer_proof(&claims, SEED, NOW),
                claims,
                proof: DelegationProof {
                    root_proof: root_proof(&cert, &policy),
                    cert,
                },
            },
            policy,
            ic_root: bls_key(1).public_key().serialize(),
            role: role(),
            allowed: vec!["read".into(), "write".into()],
            required: vec!["read".into()],
        }
    }
    fn context(&self) -> TokenVerificationContext<'_> {
        TokenVerificationContext {
            caller: p(3),
            audience: audience(),
            role: &self.role,
            allowed_scopes: &self.allowed,
            required_scopes: &self.required,
            root_key: &self.policy,
            ic_root_public_key_raw: &self.ic_root,
            now_ns: NOW,
            limits: TokenVerificationLimits {
                max_cert_ttl_ns: 600,
                max_token_ttl_ns: 120,
                max_future_skew_ns: 10,
                max_certificate_age_ns: 100,
                max_variable_bytes: 64 * 1024,
                max_issuer_signature_bytes: 16 * 1024,
                max_witness_steps: 32,
            },
        }
    }
    fn assert_error(&self, expected: Error) {
        assert_eq!(
            verify_token(&self.token, &self.context()).unwrap_err(),
            expected
        );
    }
    fn rebuild_proofs(&mut self) {
        self.token.claims.cert_hash = cert_hash(&self.token.proof.cert).unwrap();
        self.token.proof.root_proof = root_proof(&self.token.proof.cert, &self.policy);
        self.token.issuer_proof = issuer_proof(&self.token.claims, SEED, NOW);
    }
}

#[test]
fn complete_real_proof_chain_returns_only_authenticated_local_grants() {
    let f = Fixture::new();
    let verified = verify_token(&f.token, &f.context()).unwrap();
    assert_eq!(verified.claims(), &f.token.claims);
    assert_eq!(verified.role(), &f.role);
    assert_eq!(verified.scopes(), &["read"]);
    assert_eq!(
        verified.claims_hash(),
        claims_hash(&f.token.claims).unwrap()
    );
    assert_eq!(verified.expires_at_ns(), 220);
}

#[test]
fn protected_policy_deadline_and_epoch_floors_are_rechecked_without_cache() {
    let mut f = Fixture::new();
    assert!(verify_token(&f.token, &f.context()).is_ok());
    f.policy.accept_until_ns = 210;
    assert_eq!(
        verify_token(&f.token, &f.context())
            .unwrap()
            .expires_at_ns(),
        210
    );
    let mut ctx = f.context();
    ctx.now_ns = 210;
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::Expired {
            target: "root_key_policy"
        }
    );
    f.policy.accept_until_ns = NOW;
    f.assert_error(Error::Expired {
        target: "root_key_policy",
    });
    f.policy.accept_until_ns = 1000;
    f.policy.valid_from_ns = NOW + 11;
    f.assert_error(Error::NotYetValid {
        target: "root_key_policy",
    });
    f.policy.valid_from_ns = 1000;
    f.assert_error(Error::InvalidWindow {
        target: "root_key_policy",
    });
    for field in ["key_version", "proof_epoch", "registry_epoch"] {
        let mut f = Fixture::new();
        match field {
            "key_version" => f.policy.min_accepted_key_version = 5,
            "proof_epoch" => f.policy.min_accepted_proof_epoch = 3,
            _ => f.policy.min_accepted_registry_epoch = 4,
        }
        f.assert_error(Error::StaleAuthority { field });
    }
}

#[test]
fn caller_subject_anonymous_and_issuer_bindings_are_enforced() {
    let f = Fixture::new();
    let mut ctx = f.context();
    ctx.caller = p(4);
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::PresenterMismatch
    );
    ctx.caller = Principal::anonymous();
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::AnonymousPrincipal { field: "caller" }
    );
    let mut f = Fixture::new();
    f.token.claims.subject = p(4);
    f.assert_error(Error::SubjectMismatch);
    f.token.claims.subject = Principal::anonymous();
    f.assert_error(Error::AnonymousPrincipal { field: "subject" });
    let mut f = Fixture::new();
    f.token.claims.issuer_pid = p(9);
    f.assert_error(Error::BindingMismatch {
        field: "issuer_canister_id",
    });
    let mut f = Fixture::new();
    f.token.proof.cert.root_pid = p(9);
    f.assert_error(Error::BindingMismatch {
        field: "root_canister_id",
    });
    for field in ["presenter", "issuer", "root"] {
        let mut f = Fixture::new();
        match field {
            "presenter" => f.token.claims.presenter = Principal::anonymous(),
            "issuer" => f.token.proof.cert.issuer_pid = Principal::anonymous(),
            _ => f.token.proof.cert.root_pid = Principal::anonymous(),
        }
        f.assert_error(Error::AnonymousPrincipal { field });
    }
}

#[test]
fn protects_network_audience_local_role_scope_ceiling_and_endpoint_requirements() {
    let mut f = Fixture::new();
    let mut ctx = f.context();
    ctx.audience.canonical_network_id = CanonicalId::from_bytes([9; 32]);
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::AudienceRejected
    );
    ctx.audience = audience();
    ctx.audience.fleet_id = CanonicalId::from_bytes([9; 32]);
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::AudienceRejected
    );
    f.role = "other_service".parse().unwrap();
    f.assert_error(Error::RoleRejected);
    f.role = role();
    f.allowed = vec!["write".into()];
    f.assert_error(Error::ScopeRejected {
        scope: "read".into(),
    });
    f.allowed = vec!["read".into(), "write".into()];
    f.required = vec!["write".into()];
    f.assert_error(Error::ScopeRejected {
        scope: "write".into(),
    });
    f.required = vec!["Read".into()];
    assert!(matches!(
        verify_token(&f.token, &f.context()),
        Err(Error::Canonical(CanonicalAuthError::InvalidScope { .. }))
    ));
}

#[test]
fn rejects_grant_broadening_and_noncanonical_or_empty_grants() {
    let mut f = Fixture::new();
    f.token.claims.grants = vec![grant(&["admin"])];
    f.assert_error(Error::GrantsNotSubset);
    let mut f = Fixture::new();
    f.token.claims.grants[0].target = "other_service".parse().unwrap();
    f.assert_error(Error::GrantsNotSubset);
    for claims in [true, false] {
        let mut f = Fixture::new();
        let grants = if claims {
            &mut f.token.claims.grants
        } else {
            &mut f.token.proof.cert.grants
        };
        grants.clear();
        f.assert_error(Error::InvalidGrants);
        let mut f = Fixture::new();
        f.token.claims.grants[0].scopes.clear();
        f.assert_error(Error::InvalidGrants);
    }
    let mut f = Fixture::new();
    f.token.claims.grants = vec![grant(&["read"]); 17];
    f.assert_error(Error::InvalidGrants);
    let mut f = Fixture::new();
    f.token.claims.grants[0].scopes = vec!["read".into(); 33];
    f.assert_error(Error::InvalidGrants);
    let mut f = Fixture::new();
    f.token.claims.grants[0].scopes = vec!["write".into(), "read".into()];
    f.assert_error(Error::Canonical(CanonicalAuthError::NonCanonicalScopes));
    let mut f = Fixture::new();
    f.token.claims.grants.push(grant(&["read"]));
    f.assert_error(Error::Canonical(CanonicalAuthError::NonCanonicalRoles));
}

#[test]
fn token_and_certificate_windows_and_ttl_are_bounded() {
    let mut f = Fixture::new();
    f.token.claims.expires_at_ns = 240;
    assert!(verify_token(&f.token, &f.context()).is_err()); // claims changed: issuer signature also binds expiry
    f.rebuild_proofs();
    assert!(verify_token(&f.token, &f.context()).is_ok()); // exact TTL = 100
    f.token.claims.expires_at_ns += 1;
    f.assert_error(Error::TtlExceeded {
        target: "token",
        ttl_ns: 101,
        max_ttl_ns: 100,
    });
    let mut f = Fixture::new();
    f.token.claims.expires_at_ns = f.token.claims.issued_at_ns;
    f.assert_error(Error::InvalidWindow { target: "token" });
    let mut f = Fixture::new();
    f.token.claims.issued_at_ns = 99;
    f.token.claims.expires_at_ns = 190;
    f.assert_error(Error::BindingMismatch {
        field: "token_certificate_window",
    });
    let mut f = Fixture::new();
    f.token.claims.issued_at_ns = 450;
    f.token.claims.expires_at_ns = 501;
    f.assert_error(Error::BindingMismatch {
        field: "token_certificate_window",
    });
    let mut f = Fixture::new();
    f.token.proof.cert.issued_at_ns = 101;
    f.assert_error(Error::BindingMismatch {
        field: "cert_issued_at",
    });
    let mut f = Fixture::new();
    f.token.proof.cert.max_token_ttl_ns = 0;
    f.assert_error(Error::InvalidWindow {
        target: "max_token_ttl",
    });
    let mut f = Fixture::new();
    f.token.proof.cert.max_token_ttl_ns = 121;
    f.assert_error(Error::TtlExceeded {
        target: "max_token_ttl",
        ttl_ns: 121,
        max_ttl_ns: 120,
    });
    let f = Fixture::new();
    let mut ctx = f.context();
    ctx.limits.max_cert_ttl_ns = 399;
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::TtlExceeded {
            target: "certificate",
            ttl_ns: 400,
            max_ttl_ns: 399
        }
    );
}

#[test]
fn future_skew_and_exclusive_expiry_use_exact_host_time_boundaries() {
    let mut f = Fixture::new();
    f.token.claims.issued_at_ns = NOW + 10;
    f.token.claims.expires_at_ns = NOW + 80;
    f.rebuild_proofs();
    assert!(verify_token(&f.token, &f.context()).is_ok());
    f.token.claims.issued_at_ns += 1;
    f.assert_error(Error::NotYetValid { target: "token" });
    let f = Fixture::new();
    let mut ctx = f.context();
    ctx.now_ns = 220;
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::Expired { target: "token" }
    );
    ctx.now_ns = 500;
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::Expired {
            target: "certificate"
        }
    );
    ctx.now_ns = 89;
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::NotYetValid {
            target: "certificate"
        }
    );
}

#[test]
fn limits_proof_material_witnesses_and_extension_before_acceptance() {
    let f = Fixture::new();
    let mut ctx = f.context();
    ctx.limits.max_variable_bytes = 0;
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::InputTooLarge
    );
    ctx = f.context();
    ctx.limits.max_witness_steps = 1;
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::InputTooLarge
    );
    ctx = f.context();
    let IssuerProof::IcCanisterSignatureV1(proof) = &f.token.issuer_proof;
    ctx.limits.max_issuer_signature_bytes = proof.signature_cbor.len() - 1;
    assert_eq!(
        verify_token(&f.token, &ctx).unwrap_err(),
        Error::InputTooLarge
    );
    let mut f = Fixture::new();
    f.token.claims.ext = Some(vec![0; MAX_TOKEN_EXT_BYTES + 1]);
    assert!(matches!(
        verify_token(&f.token, &f.context()),
        Err(Error::Canonical(
            CanonicalAuthError::TokenExtTooLarge { .. }
        ))
    ));
}

#[test]
fn authenticates_full_claim_bytes_and_the_exact_root_authorized_seed() {
    for field in ["nonce", "extension", "certificate_hash"] {
        let mut f = Fixture::new();
        match field {
            "nonce" => f.token.claims.nonce[0] ^= 1,
            "extension" => f.token.claims.ext.as_mut().unwrap().push(4),
            _ => f.token.claims.cert_hash[0] ^= 1,
        }
        assert!(verify_token(&f.token, &f.context()).is_err());
    }
    let mut f = Fixture::new();
    f.token.issuer_proof = issuer_proof(&f.token.claims, b"other seed", NOW);
    f.assert_error(Error::IssuerSignature(CanisterSignatureError::SeedMismatch));
    let mut f = Fixture::new();
    f.token.issuer_proof = issuer_proof(&f.token.claims, SEED, NOW - 101);
    f.assert_error(Error::IssuerSignature(
        CanisterSignatureError::CertificateTooOld,
    ));
    let mut f = Fixture::new();
    f.ic_root = bls_key(2).public_key().serialize();
    assert!(matches!(
        verify_token(&f.token, &f.context()),
        Err(Error::IssuerSignature(
            CanisterSignatureError::InvalidSignature(_)
        ))
    ));
}

#[test]
fn rejects_root_crypto_failure_invalid_scalars_and_high_s() {
    for signature in [vec![0; 64], vec![1; 63], vec![0xff; 64]] {
        let mut f = Fixture::new();
        let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
        root.signature.signature = signature;
        f.assert_error(Error::RootSignatureInvalid);
    }
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    let sig = Signature::from_slice(&root.signature.signature).unwrap();
    let (r, s) = sig.split_scalars();
    root.signature.signature = Signature::from_scalars(r.to_bytes(), (-*s).to_bytes())
        .unwrap()
        .to_bytes()
        .to_vec();
    f.assert_error(Error::HighSSignature);
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.signature.signature[0] ^= 1;
    f.assert_error(Error::RootSignatureInvalid);
    let mut f = Fixture::new();
    f.policy.public_key = vec![0; 33];
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.signature.public_key = f.policy.public_key.clone();
    f.assert_error(Error::RootSignatureInvalid);
}

#[test]
fn binds_root_header_leaf_key_identity_derivation_and_merkle_direction() {
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.issuer_witness.steps[0] = ChainKeyBatchWitnessStepV1::RightSibling([42; 32]);
    f.assert_error(Error::InvalidMerkleWitness);
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.signature.derivation_path[0].push(0);
    f.assert_error(Error::BindingMismatch {
        field: "derivation_path_hash",
    });
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.delegation_cert.registry_hash[0] ^= 1;
    f.assert_error(Error::BindingMismatch {
        field: "registry_hash",
    });
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.delegation_cert.grants = vec![grant(&["read"])];
    f.assert_error(Error::BindingMismatch {
        field: "leaf_grants",
    });
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.header.key_id.name = "another_key".into();
    f.assert_error(Error::BindingMismatch { field: "key_id" });
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.header.batch_id[0] ^= 1;
    f.assert_error(Error::RootSignatureInvalid);
    let mut f = Fixture::new();
    f.policy.max_revocation_latency_ns = 399;
    f.assert_error(Error::TtlExceeded {
        target: "batch",
        ttl_ns: 400,
        max_ttl_ns: 399,
    });
}

#[test]
fn valid_signatures_do_not_override_forged_delegation_material() {
    let mut f = Fixture::new();
    f.token.proof.cert.grants = vec![grant(&["read", "write", "zadmin"])];
    f.token.claims.cert_hash = cert_hash(&f.token.proof.cert).unwrap();
    f.token.issuer_proof = issuer_proof(&f.token.claims, SEED, NOW);
    // The original valid root proof does not delegate the changed certificate.
    f.assert_error(Error::BindingMismatch {
        field: "leaf_grants",
    });
    let mut f = Fixture::new();
    f.token.proof.cert.issuer_proof_binding_hash[0] ^= 1;
    f.assert_error(Error::BindingMismatch {
        field: "issuer_proof_binding_hash",
    });
}
