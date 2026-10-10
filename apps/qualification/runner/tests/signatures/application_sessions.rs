//! Real certified token admission against the volatile reference store. This
//! does not qualify a consumer's stable-memory allocation or commit mechanism.

use super::{Fixture, SECOND};
use ic_auth::{
    canister_signature::CanisterSignatureError,
    session::{
        MemorySessionStore, SessionError, SessionEstablished, SessionLimits, SessionPolicy,
        SessionRequest, authorize_session, establish_session,
    },
    token::{TokenVerificationContext, TokenVerificationError, verify_token},
};
use ic_auth_protocol_types::DelegatedToken;
use ic_canister_sig_creation::{IC_ROOT_PK_DER, extract_raw_root_pk_from_der};
use std::time::Duration;

pub(super) fn qualify_admission(
    fixture: &Fixture,
    token: &DelegatedToken,
    mut context: TokenVerificationContext<'_>,
) {
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
    let mut policy = SessionPolicy {
        generation: 0,
        enabled: true,
        subject_admissible: true,
        default_ttl_secs: 60,
        max_ttl_secs: 60,
        max_proof_ttl_ns: 60 * SECOND,
    };
    let request = SessionRequest::new(vec!["read".into()], None).unwrap();

    // A cryptographically valid trust anchor for another network must fail
    // before any session/replay write. The identical token remains admissible.
    let other_network = extract_raw_root_pk_from_der(IC_ROOT_PK_DER).unwrap();
    let wrong_network = TokenVerificationContext {
        ic_root_public_key_raw: &other_network,
        ..context
    };
    assert!(matches!(
        establish_session(&mut store, token, &request, &wrong_network, policy),
        Err(SessionError::Token(
            TokenVerificationError::IssuerSignature(CanisterSignatureError::InvalidSignature(_))
        ))
    ));
    assert_eq!(store.encoded_bytes(), 0);
    let excessive = SessionRequest::new(vec!["write".into()], None).unwrap();
    assert_eq!(
        establish_session(&mut store, token, &excessive, &context, policy),
        Err(SessionError::ScopeRejected)
    );
    assert_eq!(store.encoded_bytes(), 0);

    let SessionEstablished::Created(session) =
        establish_session(&mut store, token, &request, &context, policy).unwrap()
    else {
        panic!("rejected admissions must leave the certified proof unconsumed");
    };
    assert_eq!(session.established_at_ns(), context.now_ns);
    assert_eq!(session.expires_at_ns(), context.now_ns + 60 * SECOND);
    let committed_bytes = store.encoded_bytes();
    assert!(committed_bytes > 0);

    // Move the actual host clock past proof expiry but within the session lease.
    // Retry returns the previously admitted authority without extending it.
    fixture.pic.advance_time(Duration::from_secs(31));
    context.now_ns = fixture.now();
    assert!(context.now_ns >= token.claims.expires_at_ns);
    assert!(context.now_ns < session.expires_at_ns());
    assert_eq!(
        verify_token(token, &context).unwrap_err(),
        TokenVerificationError::Expired { target: "token" }
    );
    assert_eq!(
        establish_session(&mut store, token, &request, &context, policy).unwrap(),
        SessionEstablished::ExactRetry(session.clone())
    );
    assert_eq!(
        authorize_session(&mut store, &context, policy).unwrap(),
        session
    );
    assert_eq!(store.encoded_bytes(), committed_bytes);

    let narrowed = TokenVerificationContext {
        allowed_scopes: &[],
        ..context
    };
    assert_eq!(
        authorize_session(&mut store, &narrowed, policy),
        Err(SessionError::ScopeRejected)
    );
    assert_eq!(
        establish_session(&mut store, token, &request, &narrowed, policy),
        Err(SessionError::ScopeRejected)
    );
    policy.generation = store.advance_generation().unwrap();
    assert_eq!(
        authorize_session(&mut store, &context, policy),
        Err(SessionError::StaleAuthority)
    );
    assert_eq!(
        establish_session(&mut store, token, &request, &context, policy),
        Err(SessionError::StaleAuthority)
    );
    assert_eq!(store.encoded_bytes(), committed_bytes);
    assert!(store.clear(context.caller));
    // Logout and generation invalidation do not erase the replay tombstone.
    assert_eq!(
        establish_session(&mut store, token, &request, &context, policy),
        Err(SessionError::ReplayConflict)
    );
}
