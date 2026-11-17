//! Installed-cost fixture using independently OpenSSL-signed root/claims records.
#[cfg(target_family = "wasm")]
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, CertifiedProofMaterial, Credential, CredentialProof,
    DelegatedRoleGrant, DelegationCertificate, DelegationLeaf, IssuerDelegation, MerkleStep,
    Principal, RootBatchHeader, RootDelegation, RootSignatureMaterial,
};
use serde::{Deserialize, Serialize};

#[cfg(target_family = "wasm")]
pub(crate) mod canister;
/// Isolated test measurement endpoint identity.
pub const MEASURE: u64 = 0x4155_5448_5155_000d;
/// Public fixture seed, not a signing secret.
pub const SEED: &[u8] = b"auth-qualification-issuer";
/// One second in the installed fixture clock.
pub const SECOND: u64 = 1_000_000_000;
/// Fixed simulator time with independently signed windows.
#[must_use]
pub fn at(seconds: u64) -> u64 {
    crate::receiver::TIME_ORIGIN_NS + seconds * SECOND
}
/// Frozen independently assembled and OpenSSL-signed inputs.
/// # Panics
/// Panics if the selected frozen record is missing or malformed.
#[must_use]
pub fn bytes(name: &str) -> Vec<u8> {
    let records = include_str!(
        "../../../ic-auth-certified-conformance/tests/fixtures/certified-installed.tsv"
    );
    let (_, value) = records
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .find(|(k, _)| *k == name)
        .unwrap();
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
/// Fixed hash from the independent records.
/// # Panics
/// Panics if the selected frozen record is missing or malformed.
#[must_use]
pub fn hash(name: &str) -> [u8; 32] {
    bytes(name).try_into().unwrap()
}
/// Protected fixture audience.
#[cfg(target_family = "wasm")]
pub(crate) fn audience() -> AudienceId {
    AudienceId {
        canonical_network_id: CanonicalId::from_bytes([3; 32]),
        fleet_id: CanonicalId::from_bytes([4; 32]),
    }
}
/// Protected grants, not inferred from submitted claims.
#[cfg(target_family = "wasm")]
pub(crate) fn grants() -> Vec<DelegatedRoleGrant> {
    vec![DelegatedRoleGrant {
        target: "app".parse().unwrap(),
        scopes: vec!["read".parse().unwrap(), "write".parse().unwrap()],
    }]
}
/// Reconstruct the independently signed delegation; submission confers no authority.
#[cfg(target_family = "wasm")]
pub(crate) fn credential(signature: Vec<u8>, der: Vec<u8>) -> Credential {
    let d = IssuerDelegation {
        root: Principal::from_slice(&[1; 10]),
        issuer: Principal::from_slice(&[2; 10]),
        binding_hash: hash("binding"),
        seed_hash: hash("seed_hash"),
        max_token_ttl_ns: 60 * SECOND,
        audience: audience(),
        grants: grants(),
        not_before_ns: at(100),
        expires_at_ns: at(200),
    };
    Credential {
        claims: bytes("claims"),
        proof: CredentialProof::Certified(Box::new(CertifiedProofMaterial {
            delegation: RootDelegation {
                certificate: DelegationCertificate {
                    delegation: d.clone(),
                    issued_at_ns: at(100),
                },
                leaf: DelegationLeaf {
                    delegation: d,
                    proof_epoch: 2,
                    registry_epoch: 3,
                    registry_hash: [9; 32],
                },
                header: RootBatchHeader {
                    schema_version: 1,
                    root: Principal::from_slice(&[1; 10]),
                    batch_id: [10; 32],
                    proof_epoch: 2,
                    registry_epoch: 3,
                    registry_hash: [9; 32],
                    tree_root: hash("tree_root"),
                    not_before_ns: at(100),
                    expires_at_ns: at(200),
                    key_name: "test_key_1".into(),
                    derivation_path_hash: hash("path_hash"),
                    key_version: 4,
                },
                witness: vec![MerkleStep::Left([11; 32]), MerkleStep::Right([12; 32])],
                signature: RootSignatureMaterial {
                    key_name: "test_key_1".into(),
                    derivation_path: vec![b"canic".to_vec(), b"delegation".to_vec()],
                    public_key: bytes("public_key"),
                    signature: bytes("signature"),
                },
            },
            issuer_signature_cbor: signature,
            issuer_public_key_der: der,
        })),
    }
}
/// Test operation: only proof bytes are ingress-controlled; trust is installed separately.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    /// Submit current certificate/witness bytes against fixed signed claims.
    Admit(Vec<u8>),
    /// Recheck current retained session without cryptographic verification.
    Authorize,
    /// Revoke the actual caller's login.
    Logout,
}
/// Operation cost and retained accounting at the same host boundary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Measurement {
    /// Ordinary receiver outcome.
    pub result: Result<crate::receiver::Outcome, crate::receiver::Refusal>,
    /// Synchronous control interval including fixture proof assembly.
    pub instructions: u64,
    /// Linear-memory pages, not live allocations.
    pub heap_pages: (u64, u64),
    /// Session/replay occupancy after the operation.
    pub usage: (u64, u64),
    /// Remaining verification credit.
    pub credit: u32,
    /// Ordinary outcome encoding size, excluding measurement metadata.
    pub response_bytes: u64,
}
