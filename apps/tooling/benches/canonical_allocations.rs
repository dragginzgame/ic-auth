//! Native allocation observations only: no verifier or production allocator changes.
mod allocation_profile;

use clap::Parser;
use ic_auth::canonical::{
    cert_hash, chain_key_batch_header_hash, chain_key_delegation_cert_hash,
    chain_key_derivation_path_hash, issuer_proof_binding_hash, proof_hash,
};
use ic_auth::portable::claims_hash;
use ic_auth_protocol_types::*;
use serde_json::{Value, json};
use std::{hint::black_box, num::NonZeroU16};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[derive(Parser)]
#[command(about = "Measure unchanged canonical hashing allocations on synthetic native inputs")]
struct Arguments {
    // Cargo supplies this flag to harness-free benchmark executables.
    #[arg(long = "bench", hide = true)]
    _bench: bool,
    #[arg(long, default_value = "100")]
    iterations: NonZeroU16,
}

fn measure(name: &str, iterations: u16, operation: impl Fn() -> [u8; 32]) -> Value {
    // Input construction, one warmup, reporting and profiling setup are outside
    // the observed operations. DHAT includes reallocations in its byte totals.
    let (expected, mut stats) = allocation_profile::measure(iterations, || black_box(operation()));
    stats["operation"] = json!(name);
    stats["digest"] = json!(expected);
    stats
}

fn fixture(roles: usize, scopes: usize) -> DelegationProof {
    let p = |byte| Principal::from_slice(&[byte]);
    let audience = DelegationAudience::Fleet(AudienceId {
        canonical_network_id: CanonicalId::from_bytes([1; 32]),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    });
    let binding = IssuerProofBinding::IcCanisterSignatureV1 { seed_hash: [8; 32] };
    let algorithm = IssuerProofAlgorithm::IcCanisterSignatureV1;
    let binding_hash = issuer_proof_binding_hash(p(3), algorithm, binding).unwrap();
    let grants = (0..roles)
        .map(|role| DelegatedRoleGrant {
            target: format!("role{role:02}").parse().unwrap(),
            scopes: (0..scopes)
                .map(|scope| format!("scope{scope:02}").parse().unwrap())
                .collect(),
        })
        .collect::<Vec<_>>();
    let key_id = ChainKeyKeyId {
        name: "test_key_1".into(),
    };
    DelegationProof {
        cert: DelegationCert {
            root_pid: p(1),
            issuer_pid: p(3),
            issuer_proof_alg: algorithm,
            issuer_proof_binding_hash: binding_hash,
            issuer_proof_binding: binding,
            issued_at_ns: 100,
            not_before_ns: 100,
            expires_at_ns: 200,
            max_token_ttl_ns: 60,
            aud: audience.clone(),
            grants: grants.clone(),
        },
        root_proof: RootProof::IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1 {
            header: ChainKeyBatchHeaderV1 {
                schema_version: 1,
                root_canister_id: p(1),
                batch_id: [31; 32],
                proof_epoch: 2,
                registry_epoch: 3,
                registry_hash: [32; 32],
                tree_root: [33; 32],
                not_before_ns: 100,
                expires_at_ns: 200,
                algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
                key_id: key_id.clone(),
                derivation_path_hash: [34; 32],
                key_version: 4,
            },
            delegation_cert: ChainKeyDelegationCertV1 {
                root_canister_id: p(1),
                issuer_canister_id: p(3),
                proof_epoch: 2,
                issuer_proof_algorithm: algorithm,
                issuer_proof_binding_hash: binding_hash,
                issuer_proof_binding: binding,
                max_token_ttl_ns: 60,
                audience,
                grants,
                not_before_ns: 100,
                expires_at_ns: 200,
                registry_epoch: 3,
                registry_hash: [32; 32],
            },
            issuer_witness: ChainKeyBatchWitnessV1 {
                steps: vec![ChainKeyBatchWitnessStepV1::LeftSibling([37; 32]); 4],
            },
            signature: ChainKeyRootSignatureV1 {
                algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
                key_id,
                derivation_path: vec![b"canic".to_vec(), b"delegation".to_vec()],
                public_key: vec![39; 33],
                signature: vec![40; 64],
            },
        }),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Arguments::parse();
    if cfg!(debug_assertions) {
        return Err("run this observation with --release".into());
    }
    let mut profiles = Vec::new();
    for (name, roles, scopes) in [("small", 1, 2), ("wide", 16, 32)] {
        let proof = fixture(roles, scopes);
        let RootProof::IcChainKeyBatchSignatureV1(root) = &proof.root_proof;
        let DelegationAudience::Fleet(audience) = &proof.cert.aud;
        let claims = PortableClaims {
            audience: *audience,
            mechanism: PortableMechanism::Ed25519,
            issuer: proof.cert.issuer_pid,
            account_subject: Principal::from_slice(&[9]),
            key_binding_id: [5; 32],
            browser_public_key: [6; 32],
            login_started_at_ns: 100,
            login_deadline_ns: 200,
            issued_at_ns: 100,
            admission_expires_at_ns: 110,
            nonce: [7; 16],
            grants: proof.cert.grants.clone(),
        };
        let n = arguments.iterations.get();
        let observations = [
            measure("portable_claims", n, || {
                claims_hash(black_box(&claims)).unwrap()
            }),
            measure("certificate", n, || {
                cert_hash(black_box(&proof.cert)).unwrap()
            }),
            measure("complete_proof", n, || {
                proof_hash(black_box(&proof)).unwrap()
            }),
            measure("batch_header", n, || {
                chain_key_batch_header_hash(black_box(&root.header)).unwrap()
            }),
            measure("issuer_leaf", n, || {
                chain_key_delegation_cert_hash(black_box(&root.delegation_cert)).unwrap()
            }),
            measure("derivation_path", n, || {
                chain_key_derivation_path_hash(black_box(&root.signature.derivation_path)).unwrap()
            }),
        ];
        profiles.push(
            json!({"profile": name, "roles": roles, "scopes_per_role": scopes,
            "witness_steps": 4, "observations": observations}),
        );
    }
    println!("{}", serde_json::to_string_pretty(&json!({
        "kind": "native-canonical-allocation-observation", "auth_version": env!("CARGO_PKG_VERSION"),
        "profiler": "dhat 0.3.3", "profiles": profiles,
        "limits": "Synthetic canonical inputs; not valid signatures, verifier cost, Wasm size/instructions, or production workload evidence. No optimization implemented."
    })).unwrap());
    Ok(())
}
