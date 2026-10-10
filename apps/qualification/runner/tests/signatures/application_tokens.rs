//! Consumer-independent complete-token qualification. Root signing uses a native
//! test key; issuer certification and upgrade use the actual Testkit canister.

use super::{DOMAIN, Fixture, SEED};
use ic_auth::{
    canister_signature::CanisterSignatureError,
    canonical::{
        cert_hash, chain_key_batch_header_hash, chain_key_delegation_cert_hash,
        chain_key_derivation_path_hash, claims_hash, issuer_proof_binding_hash,
    },
    chain_key_batch::merkle_root_and_witnesses,
    token::{
        RootKeyPolicy, TokenVerificationContext, TokenVerificationError as Error,
        TokenVerificationLimits, verify_token,
    },
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, CanonicalId, ChainKeyAlgorithm, ChainKeyBatchHeaderV1,
    ChainKeyDelegationCertV1, ChainKeyKeyId, ChainKeyRootSignatureV1, DelegatedRoleGrant,
    DelegatedToken, DelegatedTokenClaims, DelegationAudience, DelegationCert, DelegationProof,
    IcChainKeyBatchSignatureProofV1, IssuerProof, IssuerProofAlgorithm, IssuerProofBinding,
    RootProof,
};
use ic_canister_sig_creation::{IC_ROOT_PK_DER, extract_raw_root_pk_from_der, hash_bytes};
use k256::ecdsa::{Signature, SigningKey, signature::hazmat::PrehashSigner};
use std::time::Duration;

const SECOND: u64 = 1_000_000_000;

#[path = "application_sessions.rs"]
mod application_sessions;

fn root_proof(cert: &DelegationCert, key: &SigningKey, policy: &RootKeyPolicy) -> RootProof {
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
    // Preserve the authorized leaf order when selecting the returned witness.
    let hashes = [
        [42; 32],
        chain_key_delegation_cert_hash(&leaf).unwrap(),
        [43; 32],
    ];
    let (tree_root, mut witnesses) = merkle_root_and_witnesses(&hashes, 3).unwrap();
    let header = ChainKeyBatchHeaderV1 {
        schema_version: 1,
        root_canister_id: cert.root_pid,
        batch_id: [7; 32],
        proof_epoch: leaf.proof_epoch,
        registry_epoch: leaf.registry_epoch,
        registry_hash: leaf.registry_hash,
        tree_root,
        not_before_ns: cert.not_before_ns,
        expires_at_ns: cert.expires_at_ns,
        algorithm: policy.algorithm,
        key_id: policy.key_id.clone(),
        derivation_path_hash: policy.derivation_path_hash,
        key_version: policy.key_version,
    };
    let signature: Signature = key
        .sign_prehash(&chain_key_batch_header_hash(&header).unwrap())
        .unwrap();
    RootProof::IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1 {
        header,
        delegation_cert: leaf,
        issuer_witness: witnesses.remove(1),
        signature: ChainKeyRootSignatureV1 {
            algorithm: policy.algorithm,
            key_id: policy.key_id.clone(),
            derivation_path: vec![b"canic".to_vec(), b"delegation".to_vec()],
            public_key: policy.public_key.clone(),
            signature: signature.to_bytes().to_vec(),
        },
    })
}

#[test]
fn complete_token_rechecks_live_authority_and_real_certification_after_upgrade() {
    let fixture = Fixture::new();
    let started = fixture.now();
    let key = SigningKey::from_slice(&[7; 32]).unwrap();
    let policy = RootKeyPolicy {
        root_canister_id: fixture.resource,
        algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
        key_id: ChainKeyKeyId {
            name: "test_key_1".into(),
        },
        derivation_path_hash: chain_key_derivation_path_hash(&[
            b"canic".to_vec(),
            b"delegation".to_vec(),
        ])
        .unwrap(),
        public_key: key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec(),
        key_version: 4,
        min_accepted_key_version: 4,
        min_accepted_proof_epoch: 2,
        min_accepted_registry_epoch: 3,
        valid_from_ns: started,
        accept_until_ns: started + 120 * SECOND,
        max_revocation_latency_ns: 120 * SECOND,
    };
    let audience = AudienceId {
        canonical_network_id: CanonicalId::from_bytes([1; 32]),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    };
    let role: AuthRole = "app_service".parse().unwrap();
    let binding = IssuerProofBinding::IcCanisterSignatureV1 {
        seed_hash: hash_bytes(SEED),
    };
    let cert = DelegationCert {
        root_pid: policy.root_canister_id,
        issuer_pid: fixture.signer,
        issuer_proof_alg: IssuerProofAlgorithm::IcCanisterSignatureV1,
        issuer_proof_binding_hash: issuer_proof_binding_hash(
            fixture.signer,
            IssuerProofAlgorithm::IcCanisterSignatureV1,
            binding,
        )
        .unwrap(),
        issuer_proof_binding: binding,
        issued_at_ns: started,
        not_before_ns: started,
        expires_at_ns: started + 120 * SECOND,
        max_token_ttl_ns: 60 * SECOND,
        aud: DelegationAudience::Fleet(audience),
        grants: vec![DelegatedRoleGrant {
            target: role.clone(),
            scopes: vec!["read".into(), "write".into()],
        }],
    };
    let claims = DelegatedTokenClaims {
        presenter: fixture.owner,
        subject: fixture.owner,
        issuer_pid: fixture.signer,
        cert_hash: cert_hash(&cert).unwrap(),
        issued_at_ns: started,
        expires_at_ns: started + 60 * SECOND,
        aud: cert.aud.clone(),
        grants: vec![DelegatedRoleGrant {
            target: role.clone(),
            scopes: vec!["read".into()],
        }],
        nonce: [9; 16],
        ext: None,
    };
    let payload = claims_hash(&claims).unwrap();
    fixture.prepare(DOMAIN, &payload, 60 * SECOND).unwrap();
    let mut token = DelegatedToken {
        proof: DelegationProof {
            root_proof: root_proof(&cert, &key, &policy),
            cert,
        },
        claims,
        issuer_proof: IssuerProof::IcCanisterSignatureV1(
            fixture.retrieve(DOMAIN, &payload).unwrap(),
        ),
    };
    let unchanged = token.clone();
    let network = extract_raw_root_pk_from_der(&fixture.pic.root_key().unwrap()).unwrap();
    let allowed = vec!["read".into(), "write".into()];
    let required = vec!["read".into()];
    let mut context = TokenVerificationContext {
        caller: fixture.owner,
        audience,
        role: &role,
        allowed_scopes: &allowed,
        required_scopes: &required,
        root_key: &policy,
        ic_root_public_key_raw: &network,
        now_ns: fixture.now(),
        limits: TokenVerificationLimits {
            max_cert_ttl_ns: 120 * SECOND,
            max_token_ttl_ns: 60 * SECOND,
            max_future_skew_ns: 0,
            max_certificate_age_ns: 10 * SECOND,
            max_variable_bytes: 256 * 1024,
            max_issuer_signature_bytes: 128 * 1024,
            max_witness_steps: 6,
        },
    };
    let verified = verify_token(&token, &context).unwrap();
    assert_eq!(verified.claims_hash(), payload);
    assert_eq!(verified.scopes(), required);
    assert_eq!(verified.expires_at_ns(), token.claims.expires_at_ns);

    context.caller = fixture.controller;
    assert_eq!(
        verify_token(&token, &context).unwrap_err(),
        Error::PresenterMismatch
    );
    context.caller = fixture.owner;
    context.audience.fleet_id = CanonicalId::from_bytes([3; 32]);
    assert_eq!(
        verify_token(&token, &context).unwrap_err(),
        Error::AudienceRejected
    );
    context.audience = audience;
    context.allowed_scopes = &[];
    assert_eq!(
        verify_token(&token, &context).unwrap_err(),
        Error::ScopeRejected {
            scope: "read".into()
        }
    );
    context.allowed_scopes = &allowed;

    let mut changed_policy = policy.clone();
    changed_policy.accept_until_ns = context.now_ns;
    context.root_key = &changed_policy;
    assert_eq!(
        verify_token(&token, &context).unwrap_err(),
        Error::Expired {
            target: "root_key_policy"
        }
    );
    for field in ["key_version", "proof_epoch", "registry_epoch"] {
        let mut stale_policy = policy.clone();
        match field {
            "key_version" => stale_policy.min_accepted_key_version += 1,
            "proof_epoch" => stale_policy.min_accepted_proof_epoch += 1,
            _ => stale_policy.min_accepted_registry_epoch += 1,
        }
        let context = TokenVerificationContext {
            root_key: &stale_policy,
            ..context
        };
        assert_eq!(
            verify_token(&token, &context).unwrap_err(),
            Error::StaleAuthority { field }
        );
    }
    context.root_key = &policy;
    context.ic_root_public_key_raw = &network[..95];
    assert_eq!(
        verify_token(&token, &context).unwrap_err(),
        Error::IssuerSignature(CanisterSignatureError::InvalidRootKeyLength)
    );
    context.ic_root_public_key_raw = &network;
    // A valid BLS trust anchor for another network cannot authenticate this
    // actual Testkit certificate, even after an earlier successful verification.
    let other_network = extract_raw_root_pk_from_der(IC_ROOT_PK_DER).unwrap();
    assert_ne!(network, other_network);
    context.ic_root_public_key_raw = &other_network;
    assert!(matches!(
        verify_token(&token, &context),
        Err(Error::IssuerSignature(
            CanisterSignatureError::InvalidSignature(_)
        ))
    ));
    context.ic_root_public_key_raw = &network;
    assert_eq!(token, unchanged);

    fixture.pic.advance_time(Duration::from_secs(30));
    context.now_ns = fixture.now();
    assert_eq!(
        verify_token(&token, &context).unwrap_err(),
        Error::IssuerSignature(CanisterSignatureError::CertificateTooOld)
    );
    fixture.set_asset(8);
    fixture.upgrade();
    assert_eq!(fixture.asset_version(), 8);
    assert!(fixture.retrieve(DOMAIN, &payload).is_err());
    fixture.prepare(DOMAIN, &payload, 60 * SECOND).unwrap();
    token.issuer_proof =
        IssuerProof::IcCanisterSignatureV1(fixture.retrieve(DOMAIN, &payload).unwrap());
    context.now_ns = fixture.now();
    assert_eq!(
        verify_token(&token, &context).unwrap().claims_hash(),
        payload
    );
    assert_eq!(token.claims, unchanged.claims);
    assert_eq!(token.proof, unchanged.proof);
    application_sessions::qualify_admission(&fixture, &token, context);
}
