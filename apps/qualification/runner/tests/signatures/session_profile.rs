//! Isolate actual Wasm SDK costs; no consumer stable/context acquisition costs.
use super::{DOMAIN, Fixture, application_tokens::root_proof};
#[path = "../../../session_profile.rs"]
mod contract;
use contract::{SessionProfileInput, SessionProfileResult};
use ic_auth::{
    session::{
        MemorySessionStore, SessionEstablished, SessionLimits, SessionPolicy, establish_session,
    },
    session_request::SessionRequest,
    token::TokenVerificationContext,
    token_preparation::{
        PrepareDelegatedTokenInput, TokenPreparationLimits, finish_delegated_token,
        prepare_delegated_token,
    },
};
use ic_auth_protocol_types::{DelegatedRoleGrant, DelegatedToken, DelegationProof, IssuerProof};
use ic_testkit::pic::CandidCallExt;
use k256::ecdsa::SigningKey;
use std::time::Duration;

const SECOND: u64 = 1_000_000_000;

fn install(fixture: &Fixture, input: &SessionProfileInput) {
    let result: Result<(), String> = fixture.pic.update_candid_as_or_panic(
        fixture.resource,
        fixture.controller,
        "install_session_profile",
        (input.clone(),),
    );
    result.unwrap();
}
fn measure(fixture: &Fixture, expected: &str) -> SessionProfileResult {
    let result: Result<SessionProfileResult, String> = fixture.pic.query_candid_as_or_panic(
        fixture.resource,
        fixture.owner,
        "profile_session",
        (),
    );
    let sample = result.unwrap();
    assert_eq!(sample.decision, expected);
    assert!(sample.authorization_instructions > 0);
    assert!(
        sample.total_instructions < 1_000_000,
        "isolated SDK segment exceeds the facade budget"
    );
    assert!(
        sample.total_instructions >= sample.decode_instructions + sample.authorization_instructions
    );
    eprintln!("session Wasm profile: {sample:?}");
    sample
}

pub(super) fn qualify(
    fixture: &Fixture,
    template: &DelegatedToken,
    context: &TokenVerificationContext<'_>,
    key: &SigningKey,
) -> u64 {
    let mut last_expiry = 0;
    for scopes in [
        vec!["read".to_owned()],
        (0..16)
            .map(|n| format!("app{n:02}:{}", "x".repeat(58)))
            .collect(),
    ] {
        let mut cert = template.proof.cert.clone();
        cert.grants = vec![DelegatedRoleGrant {
            target: context.role.clone(),
            scopes: scopes.clone(),
        }];
        let proof = DelegationProof {
            root_proof: root_proof(&cert, key, context.root_key),
            cert,
        };
        let prepared = prepare_delegated_token(PrepareDelegatedTokenInput {
            proof: &proof,
            authority_expires_at_ns: context.root_key.accept_until_ns,
            prepared_by: fixture.owner,
            operation_id: [scopes.len() as u8; 32],
            audience: proof.cert.aud.clone(),
            grants: proof.cert.grants.clone(),
            ttl_ns: 60 * SECOND,
            ext: None,
            now_ns: fixture.now(),
            limits: TokenPreparationLimits {
                max_grants: 1,
                max_scopes_per_grant: 16,
            },
        })
        .unwrap();
        let hash = prepared.claims_hash();
        fixture.prepare(DOMAIN, &hash, 60 * SECOND).unwrap();
        let token = finish_delegated_token(
            prepared,
            IssuerProof::IcCanisterSignatureV1(fixture.retrieve(DOMAIN, &hash).unwrap()),
        );
        let context = TokenVerificationContext {
            allowed_scopes: &scopes,
            required_scopes: &scopes,
            now_ns: fixture.now(),
            ..*context
        };
        let mut store = MemorySessionStore::new(
            0,
            SessionLimits {
                max_sessions: 1,
                max_replays: 1,
                max_replays_per_subject: 1,
                max_encoded_bytes: 4096,
            },
        )
        .unwrap();
        let policy = SessionPolicy {
            generation: 0,
            enabled: true,
            subject_admissible: true,
            default_ttl_secs: 60,
            max_ttl_secs: 60,
            max_proof_ttl_ns: 60 * SECOND,
        };
        let request = SessionRequest::new(scopes.clone(), None).unwrap();
        let SessionEstablished::Created(session) =
            establish_session(&mut store, &token, &request, &context, policy).unwrap()
        else {
            panic!("new certified profile session")
        };
        let record = session.encode().unwrap();
        last_expiry = session.expires_at_ns();
        let input = SessionProfileInput {
            record: Some(record.clone()),
            audience: context.audience,
            role: context.role.clone(),
            allowed_scopes: scopes.clone(),
            required_scope: scopes[0].clone(),
            root: context.root_key.root_canister_id,
            public_key: context.root_key.public_key.clone(),
            path_hash: context.root_key.derivation_path_hash,
            valid_from_ns: context.root_key.valid_from_ns,
            accept_until_ns: context.root_key.accept_until_ns,
            generation: 0,
            enabled: true,
        };
        let refused: Result<(), String> = fixture.pic.update_candid_as_or_panic(
            fixture.resource,
            fixture.owner,
            "install_session_profile",
            (input.clone(),),
        );
        assert_eq!(refused, Err("controller required".into()));
        install(fixture, &input);
        let first = measure(fixture, "allow");
        assert_eq!(first.record_bytes, record.len() as u64);
        assert!(first.decode_instructions > 0);
        let repeated = measure(fixture, "allow");
        assert_eq!(first.record_bytes, repeated.record_bytes);

        // Rejected host configuration must preserve the previously installed pair.
        let mut invalid = input.clone();
        invalid.record = Some(vec![0; 2049]);
        let rejected: Result<(), String> = fixture.pic.update_candid_as_or_panic(
            fixture.resource,
            fixture.controller,
            "install_session_profile",
            (invalid,),
        );
        assert_eq!(rejected, Err("invalid bounded profile inputs".into()));
        invalid = input.clone();
        invalid.public_key.pop();
        let rejected: Result<(), String> = fixture.pic.update_candid_as_or_panic(
            fixture.resource,
            fixture.controller,
            "install_session_profile",
            (invalid,),
        );
        assert_eq!(rejected, Err("invalid bounded profile inputs".into()));
        if scopes.len() > 1 {
            invalid = input.clone();
            invalid.allowed_scopes.reverse();
            let rejected: Result<(), String> = fixture.pic.update_candid_as_or_panic(
                fixture.resource,
                fixture.controller,
                "install_session_profile",
                (invalid,),
            );
            assert_eq!(rejected, Err("profile ceiling must be canonical".into()));
        }
        measure(fixture, "allow");

        let mut changed = input.clone();
        changed.generation = 1;
        install(fixture, &changed);
        measure(fixture, "stale_authority");
        changed = input.clone();
        changed.accept_until_ns = last_expiry - 1;
        install(fixture, &changed);
        measure(fixture, "stale_authority");
        changed = input.clone();
        changed.public_key = SigningKey::from_slice(&[8; 32])
            .unwrap()
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec();
        install(fixture, &changed);
        measure(fixture, "stale_authority");
        changed = input.clone();
        changed.generation = 1;
        changed.allowed_scopes = vec!["different".into()];
        install(fixture, &changed);
        measure(fixture, "stale_authority");
        changed = input.clone();
        changed.allowed_scopes = vec!["different".into()];
        install(fixture, &changed);
        measure(fixture, "scope_rejected");
        changed = input.clone();
        changed.required_scope = "not_granted".into();
        install(fixture, &changed);
        measure(fixture, "scope_rejected");
        changed = input.clone();
        changed.record = None;
        install(fixture, &changed);
        measure(fixture, "missing_or_expired");
        changed = input.clone();
        changed.enabled = false;
        install(fixture, &changed);
        measure(fixture, "disabled");
        install(fixture, &input);
        measure(fixture, "allow");
        assert_eq!(session.encode().unwrap(), record);
    }
    last_expiry
}

pub(super) fn qualify_expiry(fixture: &Fixture, expiry: u64) {
    fixture
        .pic
        .advance_time(Duration::from_nanos(expiry - fixture.now()));
    fixture.pic.tick();
    measure(fixture, "missing_or_expired");
}
