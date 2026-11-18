//! Shared reconstruction of independently retained root fixture records.
use ic_auth::root_delegation::{
    DelegationLimits, RootDelegationProof, RootKeyPolicy, RootSignature,
};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, DelegationCertificate, DelegationLeaf,
    IssuerDelegation, MerkleStep, Principal, RootBatchHeader,
};

pub(crate) fn bytes(records: &str, name: &str) -> Vec<u8> {
    let (_, hex) = records
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .find(|(key, _)| *key == name)
        .unwrap();
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

fn hash(records: &str, name: &str) -> [u8; 32] {
    bytes(records, name).try_into().unwrap()
}

pub(crate) fn root() -> Principal {
    Principal::from_slice(&[1; 10])
}

pub(crate) fn issuer() -> Principal {
    Principal::from_slice(&[2; 10])
}

pub(crate) struct Fixture {
    pub(crate) cert: DelegationCertificate,
    pub(crate) leaf: DelegationLeaf,
    pub(crate) header: RootBatchHeader,
    pub(crate) steps: Vec<MerkleStep>,
    pub(crate) path: Vec<Vec<u8>>,
    pub(crate) key: Vec<u8>,
    pub(crate) signature: Vec<u8>,
    pub(crate) name: String,
}
impl Fixture {
    pub(crate) fn new(records: &str, seed_hash: [u8; 32]) -> Self {
        let delegation = IssuerDelegation {
            root: root(),
            issuer: issuer(),
            binding_hash: hash(records, "binding"),
            seed_hash,
            max_token_ttl_ns: 60,
            audience: AudienceId {
                canonical_network_id: CanonicalId::from_bytes([3; 32]),
                fleet_id: CanonicalId::from_bytes([4; 32]),
            },
            grants: vec![DelegatedRoleGrant {
                target: "app".parse().unwrap(),
                scopes: vec!["read".parse().unwrap(), "write".parse().unwrap()],
            }],
            not_before_ns: 100,
            expires_at_ns: 200,
        };
        Self {
            cert: DelegationCertificate {
                delegation: delegation.clone(),
                issued_at_ns: 100,
            },
            leaf: DelegationLeaf {
                delegation,
                proof_epoch: 2,
                registry_epoch: 3,
                registry_hash: [9; 32],
            },
            header: RootBatchHeader {
                schema_version: 1,
                root: root(),
                batch_id: [10; 32],
                proof_epoch: 2,
                registry_epoch: 3,
                registry_hash: [9; 32],
                tree_root: hash(records, "tree_root"),
                not_before_ns: 100,
                expires_at_ns: 200,
                key_name: "test_key_1".into(),
                derivation_path_hash: hash(records, "path_hash"),
                key_version: 4,
            },
            steps: vec![MerkleStep::Left([11; 32]), MerkleStep::Right([12; 32])],
            path: vec![b"canic".to_vec(), b"delegation".to_vec()],
            key: bytes(records, "public_key"),
            signature: bytes(records, "signature"),
            name: "test_key_1".into(),
        }
    }

    pub(crate) fn proof(&self) -> RootDelegationProof<'_> {
        RootDelegationProof {
            certificate: &self.cert,
            header: &self.header,
            leaf: &self.leaf,
            witness: &self.steps,
            signature: RootSignature {
                key_name: &self.name,
                derivation_path: &self.path,
                public_key: &self.key,
                signature: &self.signature,
            },
        }
    }
}

pub(crate) fn policy(key: &[u8]) -> RootKeyPolicy<'_> {
    RootKeyPolicy {
        root: root(),
        key_name: "test_key_1",
        derivation_path_hash: hash(include_str!("../fixtures/root-delegation.tsv"), "path_hash"),
        public_key: key,
        key_version: 4,
        min_key_version: 4,
        min_proof_epoch: 2,
        min_registry_epoch: 3,
        valid_from_ns: 90,
        accept_until_ns: 210,
        max_revocation_latency_ns: 100,
    }
}

pub(crate) fn limits() -> DelegationLimits {
    DelegationLimits {
        max_certificate_ttl_ns: 100,
        max_token_ttl_ns: 60,
        max_variable_bytes: 131072,
        max_witness_steps: 64,
    }
}
