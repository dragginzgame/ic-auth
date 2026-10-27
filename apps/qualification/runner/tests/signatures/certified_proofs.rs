//! Consumer-independent portable certified qualification. Root signing uses a native
//! test key; issuer certification and upgrade use the actual Testkit canister.

use super::{Fixture, SEED};
use ic_auth::{
    admission_budget::SessionAdmissionBudget,
    canister_signature::CanisterSignatureError,
    canonical::{
        chain_key_batch_header_hash, chain_key_delegation_cert_hash,
        chain_key_derivation_path_hash, issuer_proof_binding_hash,
    },
    chain_key_batch::merkle_root_and_witnesses,
    delegation::{DelegationProofError, RootKeyPolicy},
    portable::{
        PortableClaimsContext, PortableClaimsError, browser_principal, claims_hash, encode_claims,
    },
    portable_certified::{
        CERTIFIED_SIGNATURE_DOMAIN, CertifiedProofLimits, PortableCertifiedContext,
        PortableCertifiedError as Error, verify_certified_claims,
    },
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, CanonicalId, ChainKeyAlgorithm, ChainKeyBatchHeaderV1,
    ChainKeyDelegationCertV1, ChainKeyKeyId, ChainKeyRootSignatureV1, DelegatedRoleGrant,
    DelegationAudience, DelegationCert, DelegationProof, IcCanisterSignatureProofV1,
    IcChainKeyBatchSignatureProofV1, IssuerProofAlgorithm, IssuerProofBinding, PortableClaims,
    PortableMechanism, RootProof,
};
use ic_platform::canister_sig_creation::{
    IC_ROOT_PK_DER, extract_raw_root_pk_from_der, hash_bytes,
};
use ic_platform::ecdsa::{PrehashSigner, Signature, SigningKey};
use std::time::Duration;

const SECOND: u64 = 1_000_000_000;

pub(super) fn root_proof(
    cert: &DelegationCert,
    key: &SigningKey,
    policy: &RootKeyPolicy,
) -> RootProof {
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
fn portable_proof_rechecks_live_authority_and_real_certification_after_upgrade() {
    let fixture = Fixture::new();
    // Freeze signed timestamps as well as topology/keys for matched Wasm costs.
    fixture
        .pic
        .set_time((std::time::UNIX_EPOCH + Duration::from_secs(1_800_000_000)).into());
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
    let proof = DelegationProof {
        root_proof: root_proof(&cert, &key, &policy),
        cert,
    };
    let browser_key = ic_platform::ed25519::SigningKey::from_bytes(&[11; 32])
        .verifying_key()
        .to_bytes();
    let claims = PortableClaims {
        mechanism: PortableMechanism::Certified,
        audience,
        issuer: fixture.signer,
        key_binding_id: ic_auth::canonical::cert_hash(&proof.cert).unwrap(),
        browser_public_key: browser_key,
        account_subject: fixture.owner,
        login_started_at_ns: started,
        login_deadline_ns: started + 120 * SECOND,
        issued_at_ns: started,
        admission_expires_at_ns: started + 60 * SECOND,
        nonce: [4; 16],
        grants: vec![DelegatedRoleGrant {
            target: role.clone(),
            scopes: vec!["read".into()],
        }],
    };
    let encoded = encode_claims(&claims).unwrap();
    let payload = claims_hash(&claims).unwrap();
    fixture
        .prepare(CERTIFIED_SIGNATURE_DOMAIN, &payload, 60 * SECOND)
        .unwrap();
    let mut signature = fixture
        .retrieve(CERTIFIED_SIGNATURE_DOMAIN, &payload)
        .unwrap();
    let unchanged = signature.clone();
    let network = extract_raw_root_pk_from_der(&fixture.pic.root_key().unwrap()).unwrap();
    let mut context = PortableCertifiedContext {
        claims: PortableClaimsContext {
            authority: ic_auth::portable::PortableAuthority {
                audience: &audience,
                issuer: fixture.signer,
                mechanism: PortableMechanism::Certified,
                key_binding_id: claims.key_binding_id,
                grants: &proof.cert.grants,
                caller: browser_principal(&browser_key),
                now_ns: fixture.now(),
                authority_not_before_ns: started,
                authority_expires_at_ns: proof.cert.expires_at_ns,
                enrollment_expires_at_ns: proof.cert.expires_at_ns,
                min_login_started_at_ns: 0,
            },
            min_proof_issued_at_ns: 0,
            max_login_ttl_ns: 120 * SECOND,
            max_proof_ttl_ns: 60 * SECOND,
        },
        root_key: &policy,
        ic_root_public_key_raw: &network,
        limits: CertifiedProofLimits {
            max_cert_ttl_ns: 120 * SECOND,
            max_token_ttl_ns: 60 * SECOND,
            max_certificate_age_ns: 10 * SECOND,
            max_variable_bytes: 256 * 1024,
            max_issuer_signature_bytes: 128 * 1024,
            max_witness_steps: 6,
        },
    };
    let verify = |signature: &IcCanisterSignatureProofV1,
                  context: &PortableCertifiedContext<'_>| {
        verify_certified_claims(
            &mut SessionAdmissionBudget::new(64, 1, 0).unwrap(),
            &encoded,
            &proof,
            signature,
            context,
        )
    };
    let verified = verify(&signature, &context).unwrap();
    assert_eq!(verified.fingerprint(), payload);
    assert_eq!(verified.claims(), &claims);
    assert_ne!(context.claims.authority.caller, claims.account_subject);
    context.claims.authority.caller = fixture.controller;
    assert_eq!(
        verify(&signature, &context).unwrap_err(),
        Error::Claims(PortableClaimsError::CallerMismatch)
    );
    context.claims.authority.caller = browser_principal(&browser_key);
    let other_audience = AudienceId {
        fleet_id: CanonicalId::from_bytes([3; 32]),
        ..audience
    };
    context.claims.authority.audience = &other_audience;
    assert_eq!(
        verify(&signature, &context).unwrap_err(),
        Error::Claims(PortableClaimsError::AudienceMismatch)
    );
    context.claims.authority.audience = &audience;
    let narrowed = vec![DelegatedRoleGrant {
        target: role.clone(),
        scopes: vec!["write".into()],
    }];
    context.claims.authority.grants = &narrowed;
    assert_eq!(
        verify(&signature, &context).unwrap_err(),
        Error::Claims(PortableClaimsError::GrantsExceeded)
    );
    context.claims.authority.grants = &proof.cert.grants;
    context.claims.authority.enrollment_expires_at_ns = context.claims.authority.now_ns;
    assert_eq!(
        verify(&signature, &context).unwrap_err(),
        Error::Claims(PortableClaimsError::StaleAuthority)
    );
    context.claims.authority.enrollment_expires_at_ns = proof.cert.expires_at_ns;
    for field in ["key_version", "proof_epoch", "registry_epoch"] {
        let mut stale = policy.clone();
        match field {
            "key_version" => stale.min_accepted_key_version += 1,
            "proof_epoch" => stale.min_accepted_proof_epoch += 1,
            _ => stale.min_accepted_registry_epoch += 1,
        }
        let context = PortableCertifiedContext {
            root_key: &stale,
            ..context
        };
        assert_eq!(
            verify(&signature, &context).unwrap_err(),
            Error::Delegation(DelegationProofError::StaleAuthority { field })
        );
    }
    context.ic_root_public_key_raw = &network[..95];
    assert_eq!(
        verify(&signature, &context).unwrap_err(),
        Error::Signature(CanisterSignatureError::InvalidRootKeyLength)
    );
    let other_network = extract_raw_root_pk_from_der(IC_ROOT_PK_DER).unwrap();
    assert_ne!(network, other_network);
    context.ic_root_public_key_raw = &other_network;
    assert!(matches!(
        verify(&signature, &context),
        Err(Error::Signature(CanisterSignatureError::InvalidSignature(
            _
        )))
    ));
    context.ic_root_public_key_raw = &network;
    assert_eq!(signature, unchanged);
    fixture.pic.advance_time(Duration::from_secs(30));
    context.claims.authority.now_ns = fixture.now();
    assert_eq!(
        verify(&signature, &context).unwrap_err(),
        Error::Signature(CanisterSignatureError::CertificateTooOld)
    );
    fixture.set_asset(8);
    fixture.upgrade();
    assert_eq!(fixture.asset_version(), 8);
    assert!(
        fixture
            .retrieve(CERTIFIED_SIGNATURE_DOMAIN, &payload)
            .is_err()
    );
    fixture
        .prepare(CERTIFIED_SIGNATURE_DOMAIN, &payload, 60 * SECOND)
        .unwrap();
    signature = fixture
        .retrieve(CERTIFIED_SIGNATURE_DOMAIN, &payload)
        .unwrap();
    context.claims.authority.now_ns = fixture.now();
    assert_eq!(verify(&signature, &context).unwrap().fingerprint(), payload);
    assert_eq!(encode_claims(&claims).unwrap(), encoded);
    super::sessions::qualify_certified(&fixture, &proof, &context, &role);
}
