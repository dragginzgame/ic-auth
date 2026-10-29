//! Valid mechanical setup shared by lifecycle tests. Independent signature
//! vectors and each suite's rejection mutations deliberately remain local.
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};
use ic_platform::ed25519::SigningKey;
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
            scopes: vec!["read".into()],
        }],
    }
}
