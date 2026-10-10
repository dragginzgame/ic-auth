//! Real certified token admission against the volatile reference store. This
//! does not qualify a consumer's stable-memory allocation or commit mechanism.

use super::{Fixture, SECOND};
use ic_auth::{
    canister_signature::CanisterSignatureError,
    canonical::claims_hash,
    session::{
        MemorySessionStore, SessionError, SessionEstablished, SessionLimits, SessionOccupancy,
        SessionPolicy, SessionPruned, SessionRequest, SessionStore, SessionTransaction,
        authorize_session, establish_session,
    },
    token::{TokenVerificationContext, TokenVerificationError, verify_token},
};
use ic_auth_protocol_types::DelegatedToken;
use ic_canister_sig_creation::{IC_ROOT_PK_DER, extract_raw_root_pk_from_der};
use std::time::Duration;

// Refuse after the actual library operation, before the reference backend
// commits its staged pair. This is a substituted host failure, not a durable
// storage implementation or a simulation of persistence/restart recovery.
struct RefuseCommit<'a>(&'a mut MemorySessionStore);

impl SessionStore for RefuseCommit<'_> {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        self.0.transaction(|tx| {
            operation(tx)?;
            Err(SessionError::StorageFailure)
        })
    }
}

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

    // Complete real-certificate verification and staging succeed, but a host
    // refusal must not expose a session or consume replay/capacity/accounting.
    assert_eq!(
        establish_session(
            &mut RefuseCommit(&mut store),
            token,
            &request,
            &context,
            policy
        ),
        Err(SessionError::StorageFailure)
    );
    let fingerprint = claims_hash(&token.claims).unwrap();
    store
        .transaction(|tx| {
            assert_eq!(tx.generation(), policy.generation);
            assert_eq!(tx.session(context.caller)?, None);
            assert!(!tx.replay_exists(fingerprint)?);
            assert_eq!(tx.occupancy(context.caller)?, SessionOccupancy::default());
            Ok(())
        })
        .unwrap();
    assert_eq!(store.encoded_bytes(), 0);
    assert_eq!(store.prune(u64::MAX, 128), SessionPruned::default());
    assert_eq!(
        authorize_session(&mut store, &context, policy),
        Err(SessionError::MissingOrExpired)
    );

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
    // Exact retry still obeys the host transaction result. Refusal preserves
    // the committed session/replay pair and cannot return cached authority.
    assert_eq!(
        establish_session(
            &mut RefuseCommit(&mut store),
            token,
            &request,
            &context,
            policy
        ),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(store.encoded_bytes(), committed_bytes);
    store
        .transaction(|tx| {
            assert_eq!(tx.generation(), policy.generation);
            assert_eq!(tx.session(context.caller)?, Some(session.clone()));
            assert!(tx.replay_exists(fingerprint)?);
            assert_eq!(
                tx.occupancy(context.caller)?,
                SessionOccupancy {
                    sessions: 1,
                    replays: 1,
                    replays_for_subject: 1,
                }
            );
            Ok(())
        })
        .unwrap();
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
