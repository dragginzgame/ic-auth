//! Valid mechanical setup shared by lifecycle tests. Independent signature
//! vectors and each suite's rejection mutations deliberately remain local.
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};
use ic_platform::ed25519::{Signer, SigningKey};
use sha2::{Digest, Sha256};

pub fn issuer_key() -> SigningKey {
    SigningKey::from_bytes(&[42; 32])
}

pub fn claims(browser_public_key: [u8; 32]) -> PortableClaims {
    PortableClaims {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        mechanism: PortableMechanism::Ed25519,
        issuer: Principal::from_slice(&[1]),
        account_subject: Principal::from_slice(&[2]),
        key_binding_id: Sha256::digest(issuer_key().verifying_key().to_bytes()).into(),
        browser_public_key,
        login_started_at_ns: 0,
        login_deadline_ns: 1000,
        issued_at_ns: 0,
        admission_expires_at_ns: 1,
        nonce: [0; 16],
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }],
    }
}

/// Shared valid signing mechanics; independent/adversarial vectors remain local.
pub fn signature(claims: &PortableClaims, key: &SigningKey) -> [u8; 64] {
    key.sign(&ic_auth::portable::claims_hash(claims).unwrap())
        .to_bytes()
}
pub fn authority<'a>(
    claims: &'a PortableClaims,
    grants: &'a ic_auth::portable::ValidatedGrants,
    now_ns: u64,
) -> ic_auth::portable::PortableAuthority<'a> {
    ic_auth::portable::PortableAuthority {
        audience: &claims.audience,
        issuer: claims.issuer,
        mechanism: claims.mechanism,
        key_binding_id: claims.key_binding_id,
        grants,
        caller: ic_auth::portable::browser_principal(&claims.browser_public_key),
        now_ns,
        authority_not_before_ns: 0,
        authority_expires_at_ns: u64::MAX,
        enrollment_expires_at_ns: u64::MAX,
        min_login_started_at_ns: 0,
    }
}
pub fn claims_context(
    authority: ic_auth::portable::PortableAuthority<'_>,
) -> ic_auth::portable::PortableClaimsContext<'_> {
    ic_auth::portable::PortableClaimsContext {
        authority,
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 86_400_000_000_000,
        max_proof_ttl_ns: 60_000_000_000,
    }
}
