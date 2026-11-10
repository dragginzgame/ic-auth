//! Native exact-retry allocations; fixture setup/signing and request construction
//! are outside measurement. No production allocator or verifier behavior changes.
mod allocation_profile;

use clap::Parser;
use ic_auth::{
    admission_budget::SessionAdmissionBudget,
    portable::{
        MemoryLoginRevocations, PortableAuthority, PortableClaimsContext, ValidatedGrants,
        browser_principal, claims_hash, decode_claims, encode_claims,
    },
    portable_credential::PortableVerificationContext,
    portable_signature::PreparedIssuerKey,
    session::{
        MemorySessionStore, SessionAdmissionContext, SessionEstablished, SessionLimits,
        SessionPolicy, establish_session,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
};
use ic_auth_protocol_types::*;
use ic_platform::ed25519::{Signer, SigningKey};
use serde_json::{Value, json};
use std::{hint::black_box, num::NonZeroU16};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[derive(Parser)]
#[command(about = "Observe native allocations of already-admitted exact retries")]
struct Arguments {
    // Cargo supplies this flag to harness-free benchmark executables.
    #[arg(long = "bench", hide = true)]
    _bench: bool,
    #[arg(long, default_value = "100")]
    iterations: NonZeroU16,
}
fn profile(name: &str, roles: usize, scopes: usize, iterations: u16) -> Value {
    const SECOND: u64 = 1_000_000_000;
    const NOW: u64 = 10 * SECOND;
    let key = SigningKey::from_bytes(&[42; 32]);
    let prepared = PreparedIssuerKey::new(key.verifying_key().to_bytes()).unwrap();
    let role: AuthRole = format!("role{:02}", roles - 1).parse().unwrap();
    let claims = PortableClaims {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        mechanism: PortableMechanism::Ed25519,
        issuer: Principal::from_slice(&[1]),
        device_slot: 0,
        device_generation: 1,
        account_subject: Principal::from_slice(&[2]),
        key_binding_id: prepared.binding(),
        browser_public_key: SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes(),
        login_started_at_ns: NOW - SECOND,
        login_deadline_ns: NOW + 60 * SECOND,
        issued_at_ns: NOW,
        admission_expires_at_ns: NOW + SECOND,
        nonce: [0; 16],
        grants: (0..roles)
            .map(|role| DelegatedRoleGrant {
                target: format!("role{role:02}").parse().unwrap(),
                scopes: (0..scopes)
                    .map(|scope| format!("scope{scope:02}").parse().unwrap())
                    .collect(),
            })
            .collect(),
    };
    let grants = ValidatedGrants::new(claims.grants.clone()).unwrap();
    let mut context = PortableClaimsContext {
        authority: PortableAuthority {
            audience: &claims.audience,
            issuer: claims.issuer,
            mechanism: claims.mechanism,
            key_binding_id: prepared.binding(),
            grants: &grants,
            caller: browser_principal(&claims.browser_public_key),
            now_ns: NOW,
            authority_not_before_ns: 0,
            authority_expires_at_ns: NOW + 120 * SECOND,
            enrollment_expires_at_ns: NOW + 90 * SECOND,
            min_login_started_at_ns: 0,
        },
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 120 * SECOND,
        max_proof_ttl_ns: 60 * SECOND,
    };
    let mut credential = PortableCredential {
        claims: encode_claims(&claims).unwrap(),
        proof: PortableProof::Ed25519 {
            signature: key.sign(&claims_hash(&claims).unwrap()).to_bytes().to_vec(),
        },
    };
    let mut store = MemorySessionStore::new(0, SessionLimits::new(4, 8, 4, 65_536).unwrap());
    let mut ledger =
        MemoryLoginRevocations::new(claims.audience, 4, 4, 86_400_000_000_000, NOW).unwrap();
    let mut budget = SessionAdmissionBudget::new(1, NOW, 0).unwrap();
    let policy = SessionPolicy {
        generation: 0,
        enabled: true,
        subject_admissible: true,
    };
    let request = SessionRequest::new(vec!["scope00".into()], None).unwrap();
    let receipt = establish_session(
        &mut store,
        &mut ledger,
        &mut budget,
        &credential,
        request.clone(),
        &SessionAdmissionContext {
            verification: PortableVerificationContext::Ed25519 {
                claims: &context,
                public_key: &prepared,
            },
            role: &role,
            ttl: SessionTtlPolicy::new(5, 30).unwrap(),
        },
        policy,
    )
    .unwrap()
    .receipt();
    // Proof expiry, malformed signature and exhausted credit must not prevent
    // the existing exact retry. All current session/authority checks still run.
    credential.proof = PortableProof::Ed25519 { signature: vec![] };
    context.authority.now_ns = NOW + 3 * SECOND;
    let admission = SessionAdmissionContext {
        verification: PortableVerificationContext::Ed25519 {
            claims: &context,
            public_key: &prepared,
        },
        role: &role,
        ttl: SessionTtlPolicy::new(5, 30).unwrap(),
    };
    let requests: Vec<_> = (0..=iterations).map(|_| request.clone()).collect();
    let mut requests = requests.into_iter();
    let (_, retry) = allocation_profile::measure(iterations, || {
        assert_eq!(
            black_box(establish_session(
                &mut store,
                &mut ledger,
                &mut budget,
                black_box(&credential),
                requests.next().unwrap(),
                &admission,
                policy
            ))
            .unwrap(),
            SessionEstablished::ExactRetry(receipt)
        );
    });
    let (_, decode) = allocation_profile::measure(iterations, || {
        drop(black_box(
            decode_claims(black_box(&credential.claims)).unwrap(),
        ));
    });
    json!({"profile": name, "roles": roles, "scopes_per_role": scopes,
        "claims_bytes": credential.claims.len(), "exact_retry": retry, "decode_only": decode})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Err("run this observation with --release".into());
    }
    let iterations = Arguments::parse().iterations.get();
    let profiles = [
        profile("small", 1, 2, iterations),
        profile("wide", 16, 32, iterations),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "kind": "native-exact-retry-allocation-observation", "profiler": "dhat 0.3.3",
            "profiles": profiles,
            "limits": "Synthetic valid initial Ed25519 admission, then exact retries after proof expiry using a volatile store. Setup, signing and owned request construction excluded. Native allocations only; not Wasm instructions/size, latency or production workload evidence. No retry optimization implemented."
        }))?
    );
    Ok(())
}
