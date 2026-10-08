use super::*;
use ic_auth::session::{
    MemorySessionStore, Session, SessionError, SessionEstablished, SessionLimits, SessionPolicy,
    SessionRequest, SessionStore, SessionTransaction, authorize_session, establish_session,
};

fn policy(generation: u64) -> SessionPolicy {
    SessionPolicy {
        generation,
        enabled: true,
        subject_admissible: true,
        default_ttl_secs: 1,
        max_ttl_secs: 2,
        max_proof_ttl_ns: 60_000_000_000,
    }
}
fn limits() -> SessionLimits {
    SessionLimits {
        max_sessions: 2048,
        max_replays: 4096,
        max_replays_per_subject: 256,
        max_encoded_bytes: 8 * 1024 * 1024,
    }
}
fn store() -> MemorySessionStore {
    MemorySessionStore::new(0, limits()).unwrap()
}
fn request() -> SessionRequest {
    SessionRequest::new(vec!["read".into()], None).unwrap()
}
fn created(result: SessionEstablished) -> Session {
    let SessionEstablished::Created(session) = result else {
        panic!("expected creation");
    };
    session
}
fn resign(f: &mut Fixture, nonce: u8) {
    f.token.claims.nonce = [nonce; 16];
    f.token.issuer_proof = issuer_proof(&f.token.claims, SEED, NOW);
}

#[test]
fn verified_admission_retry_after_proof_expiry_and_no_extension() {
    let f = Fixture::new();
    let mut s = store();
    let original =
        created(establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap());
    assert_eq!(original.expires_at_ns(), 1000); // Host root acceptance, not token expiry 220.
    assert_eq!(
        Session::decode(&original.encode().unwrap()).unwrap(),
        original
    );
    let before = s.encoded_bytes();
    let mut ctx = f.context();
    ctx.now_ns = 250;
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &ctx, policy(0)).unwrap(),
        SessionEstablished::ExactRetry(original.clone())
    );
    assert_eq!(
        authorize_session(&mut s, &ctx, policy(0)).unwrap(),
        original
    );
    assert_eq!(s.encoded_bytes(), before);
    ctx.now_ns = 1000;
    assert_eq!(
        authorize_session(&mut s, &ctx, policy(0)),
        Err(SessionError::MissingOrExpired)
    );
}

#[test]
fn conflicting_retry_and_replaced_or_logged_out_proof_stay_consumed() {
    let mut f = Fixture::new();
    let mut s = store();
    let old_token = f.token.clone();
    establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap();
    let different = SessionRequest::new(vec!["read".into()], Some(1)).unwrap();
    assert_eq!(
        establish_session(&mut s, &f.token, &different, &f.context(), policy(0)),
        Err(SessionError::ReplayConflict)
    );
    resign(&mut f, 10);
    assert!(matches!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap(),
        SessionEstablished::Replaced(_)
    ));
    assert_eq!(
        establish_session(&mut s, &old_token, &request(), &f.context(), policy(0)),
        Err(SessionError::ReplayConflict)
    );
    assert!(s.clear(p(3)));
    assert!(!s.clear(p(3)));
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)),
        Err(SessionError::ReplayConflict)
    );
    assert_eq!(
        authorize_session(&mut s, &f.context(), policy(0)),
        Err(SessionError::MissingOrExpired)
    );
}

#[test]
fn generation_invalidates_retry_and_preserves_replay() {
    let f = Fixture::new();
    let mut s = store();
    establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap();
    assert_eq!(s.advance_generation().unwrap(), 1);
    assert_eq!(
        authorize_session(&mut s, &f.context(), policy(0)),
        Err(SessionError::StaleAuthority)
    );
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(1)),
        Err(SessionError::StaleAuthority)
    );
    s.clear(p(3));
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(1)),
        Err(SessionError::ReplayConflict)
    );
    let mut exhausted = MemorySessionStore::new(u64::MAX, limits()).unwrap();
    assert_eq!(
        exhausted.advance_generation(),
        Err(SessionError::GenerationExhausted)
    );
    assert_eq!(exhausted.generation(), u64::MAX);
}

#[test]
fn live_caller_audience_role_scope_and_subject_policy_are_checked() {
    let f = Fixture::new();
    let mut s = store();
    establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap();
    let mut ctx = f.context();
    ctx.caller = p(4);
    assert_eq!(
        authorize_session(&mut s, &ctx, policy(0)),
        Err(SessionError::MissingOrExpired)
    );
    ctx.caller = Principal::anonymous();
    assert!(authorize_session(&mut s, &ctx, policy(0)).is_err());
    ctx = f.context();
    ctx.audience.fleet_id = CanonicalId::from_bytes([42; 32]);
    assert_eq!(
        authorize_session(&mut s, &ctx, policy(0)),
        Err(SessionError::StaleAuthority)
    );
    let other_role = "other".parse().unwrap();
    ctx = f.context();
    ctx.role = &other_role;
    assert_eq!(
        authorize_session(&mut s, &ctx, policy(0)),
        Err(SessionError::StaleAuthority)
    );
    ctx = f.context();
    ctx.allowed_scopes = &[];
    assert_eq!(
        authorize_session(&mut s, &ctx, policy(0)),
        Err(SessionError::ScopeRejected)
    );
    let required = vec!["write".into()];
    ctx = f.context();
    ctx.required_scopes = &required;
    assert_eq!(
        authorize_session(&mut s, &ctx, policy(0)),
        Err(SessionError::ScopeRejected)
    );
    let mut disabled = policy(0);
    disabled.enabled = false;
    assert_eq!(
        authorize_session(&mut s, &f.context(), disabled),
        Err(SessionError::Disabled)
    );
    disabled.enabled = true;
    disabled.subject_admissible = false;
    assert_eq!(
        authorize_session(&mut s, &f.context(), disabled),
        Err(SessionError::Disabled)
    );
}

#[test]
fn root_epochs_key_identity_and_acceptance_remain_live_on_retry() {
    let mut f = Fixture::new();
    let mut s = store();
    establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap();
    let saved = f.policy.clone();
    f.policy.min_accepted_registry_epoch += 1;
    assert_eq!(
        authorize_session(&mut s, &f.context(), policy(0)),
        Err(SessionError::StaleAuthority)
    );
    f.policy = saved.clone();
    f.policy.min_accepted_proof_epoch += 1;
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)),
        Err(SessionError::StaleAuthority)
    );
    f.policy = saved.clone();
    f.policy.public_key[1] ^= 1;
    assert_eq!(
        authorize_session(&mut s, &f.context(), policy(0)),
        Err(SessionError::StaleAuthority)
    );
    f.policy = saved.clone();
    f.policy.key_id.name = "changed".into();
    assert_eq!(
        authorize_session(&mut s, &f.context(), policy(0)),
        Err(SessionError::StaleAuthority)
    );
    f.policy = saved.clone();
    f.policy.accept_until_ns = 200;
    assert_eq!(
        authorize_session(&mut s, &f.context(), policy(0)),
        Err(SessionError::StaleAuthority)
    );
    f.policy = saved;
    f.policy.accept_until_ns = NOW;
    assert_eq!(
        authorize_session(&mut s, &f.context(), policy(0)),
        Err(SessionError::StaleAuthority)
    );
}

#[test]
fn forged_proof_or_ungranted_narrowing_never_consumes_state() {
    let mut f = Fixture::new();
    let mut s = store();
    let saved = f.token.clone();
    let RootProof::IcChainKeyBatchSignatureV1(proof) = &mut f.token.proof.root_proof;
    proof.signature.signature[0] ^= 1;
    assert!(matches!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)),
        Err(SessionError::Token(_))
    ));
    assert_eq!(s.encoded_bytes(), 0);
    f.token = saved;
    let ungranted = SessionRequest::new(vec!["write".into()], None).unwrap();
    assert_eq!(
        establish_session(&mut s, &f.token, &ungranted, &f.context(), policy(0)),
        Err(SessionError::ScopeRejected)
    );
    assert_eq!(s.encoded_bytes(), 0);
    assert!(establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).is_ok());
}

struct FailingStore(MemorySessionStore);
impl SessionStore for FailingStore {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        self.0.transaction(|tx| {
            let _ = operation(tx)?;
            Err(SessionError::StorageFailure)
        })
    }
}
#[test]
fn failure_after_staging_preserves_session_replay_counters_and_indexes() {
    let mut f = Fixture::new();
    let mut s = store();
    let previous =
        created(establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap());
    let bytes = s.encoded_bytes();
    resign(&mut f, 10);
    let mut failing = FailingStore(s);
    assert_eq!(
        establish_session(&mut failing, &f.token, &request(), &f.context(), policy(0)),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(failing.0.encoded_bytes(), bytes);
    assert_eq!(
        authorize_session(&mut failing.0, &f.context(), policy(0)).unwrap(),
        previous
    );
    assert!(matches!(
        establish_session(
            &mut failing.0,
            &f.token,
            &request(),
            &f.context(),
            policy(0)
        )
        .unwrap(),
        SessionEstablished::Replaced(_)
    ));
}

#[test]
fn global_subject_and_byte_capacity_fail_without_partial_replacement() {
    let mut f = Fixture::new();
    for quota in [
        SessionLimits {
            max_replays: 1,
            ..limits()
        },
        SessionLimits {
            max_replays_per_subject: 1,
            ..limits()
        },
    ] {
        let mut s = MemorySessionStore::new(0, quota).unwrap();
        let before = created(
            establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap(),
        );
        let bytes = s.encoded_bytes();
        let token = f.token.clone();
        resign(&mut f, 10);
        assert_eq!(
            establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)),
            Err(SessionError::Capacity)
        );
        assert_eq!(
            authorize_session(&mut s, &f.context(), policy(0)).unwrap(),
            before
        );
        assert_eq!(s.encoded_bytes(), bytes);
        f.token = token;
    }
    let mut s = MemorySessionStore::new(
        0,
        SessionLimits {
            max_encoded_bytes: 1,
            ..limits()
        },
    )
    .unwrap();
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)),
        Err(SessionError::ByteCapacity)
    );
    assert_eq!(s.encoded_bytes(), 0);
    let mut sample = store();
    let original = created(
        establish_session(&mut sample, &f.token, &request(), &f.context(), policy(0)).unwrap(),
    );
    let budget = sample.encoded_bytes();
    let mut exact = MemorySessionStore::new(
        0,
        SessionLimits {
            max_encoded_bytes: budget,
            ..limits()
        },
    )
    .unwrap();
    establish_session(&mut exact, &f.token, &request(), &f.context(), policy(0)).unwrap();
    let saved = f.token.clone();
    resign(&mut f, 10);
    assert_eq!(
        establish_session(&mut exact, &f.token, &request(), &f.context(), policy(0)),
        Err(SessionError::ByteCapacity)
    );
    assert_eq!(exact.encoded_bytes(), budget);
    assert_eq!(
        authorize_session(&mut exact, &f.context(), policy(0)).unwrap(),
        original
    );
    exact
        .transaction(|tx| {
            assert!(!tx.replay_exists(claims_hash(&f.token.claims).unwrap())?);
            assert_eq!(tx.occupancy(p(3))?.replays_for_subject, 1);
            Ok(())
        })
        .unwrap();
    f.token = saved;
    let mut s = MemorySessionStore::new(
        0,
        SessionLimits {
            max_sessions: 1,
            ..limits()
        },
    )
    .unwrap();
    establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap();
    f.token.claims.presenter = p(4);
    f.token.claims.subject = p(4);
    resign(&mut f, 11);
    let mut ctx = f.context();
    ctx.caller = p(4);
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &ctx, policy(0)),
        Err(SessionError::Capacity)
    );
}

#[test]
fn pruning_is_bounded_and_exact_retry_survives_replay_pruning() {
    let mut f = Fixture::new();
    let mut s = store();
    for nonce in 0..130 {
        resign(&mut f, nonce);
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap();
    }
    assert_eq!(s.prune(219, 128).replays, 0);
    let removed = s.prune(220, usize::MAX);
    assert_eq!(removed.replays, 128);
    assert_eq!(removed.sessions, 0);
    assert_eq!(s.prune(220, 10).replays, 2);
    let mut ctx = f.context();
    ctx.now_ns = 250;
    assert!(matches!(
        establish_session(&mut s, &f.token, &request(), &ctx, policy(0)).unwrap(),
        SessionEstablished::ExactRetry(_)
    ));
    assert_eq!(s.prune(1000, 0).sessions, 0);
    assert_eq!(s.prune(1000, 1).sessions, 1);
    assert_eq!(s.encoded_bytes(), 0);
}

#[test]
fn canonical_request_hash_preserves_exact_ttl_choice_and_scope_order() {
    let a = SessionRequest::new(vec!["write".into(), "read".into()], Some(30)).unwrap();
    let b = SessionRequest::new(vec!["read".into(), "write".into()], Some(30)).unwrap();
    assert_eq!(a.hash(), b.hash());
    assert_eq!(
        a.hash()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        "d99fc919563d8dba6db966bacbb256f2ba2437e64d12353106409934cc678551"
    );
    assert_ne!(
        request().hash(),
        SessionRequest::new(vec!["read".into()], Some(1))
            .unwrap()
            .hash()
    );
    assert!(SessionRequest::new(vec![], None).is_err());
    assert!(SessionRequest::new(vec!["read".into(), "read".into()], None).is_err());
    assert!(SessionRequest::new(vec!["bad scope".into()], None).is_err());
    assert!(SessionRequest::new((0..17).map(|n| format!("s{n}")).collect(), None).is_err());
    assert!(SessionRequest::new(vec!["read".into()], Some(0)).is_err());
    assert_eq!(
        Session::decode(&vec![0; 2049]),
        Err(SessionError::ByteCapacity)
    );
    assert_eq!(
        Session::decode(&[0]),
        Err(SessionError::InvalidPolicyOrRecord)
    );
}

#[test]
fn strict_proof_start_ttl_and_invalid_policy_do_not_mutate_state() {
    let mut f = Fixture::new();
    f.token.issuer_proof = issuer_proof(&f.token.claims, SEED, 135);
    let mut s = store();
    let mut ctx = f.context();
    ctx.now_ns = 135; // Signature verifier allows skew; session admission does not.
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &ctx, policy(0)),
        Err(SessionError::ProofIneligible)
    );
    let mut restricted = policy(0);
    restricted.max_proof_ttl_ns = 79;
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &f.context(), restricted),
        Err(SessionError::ProofIneligible)
    );
    let too_long = SessionRequest::new(vec!["read".into()], Some(3)).unwrap();
    assert_eq!(
        establish_session(&mut s, &f.token, &too_long, &f.context(), policy(0)),
        Err(SessionError::InvalidPolicyOrRecord)
    );
    for bad in [
        SessionPolicy {
            max_ttl_secs: 1801,
            ..policy(0)
        },
        SessionPolicy {
            default_ttl_secs: 0,
            ..policy(0)
        },
    ] {
        assert_eq!(
            establish_session(&mut s, &f.token, &request(), &f.context(), bad),
            Err(SessionError::InvalidPolicyOrRecord)
        );
    }
    assert_eq!(s.encoded_bytes(), 0);
}

#[test]
fn session_expiry_overflow_with_fully_signed_maximum_clock_is_atomic() {
    let mut f = Fixture::new();
    let now = u64::MAX - 100;
    f.policy.accept_until_ns = u64::MAX;
    let cert = &mut f.token.proof.cert;
    cert.issued_at_ns = now - 100;
    cert.not_before_ns = now - 100;
    cert.expires_at_ns = u64::MAX;
    cert.max_token_ttl_ns = 100;
    f.token.claims.issued_at_ns = now - 10;
    f.token.claims.expires_at_ns = now + 50;
    f.token.claims.cert_hash = cert_hash(cert).unwrap();
    f.token.proof.root_proof = root_proof(cert, &f.policy);
    f.token.issuer_proof = issuer_proof(&f.token.claims, SEED, now);
    let mut ctx = f.context();
    ctx.now_ns = now;
    assert!(verify_token(&f.token, &ctx).is_ok());
    let mut s = store();
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &ctx, policy(0)),
        Err(SessionError::InvalidPolicyOrRecord)
    );
    assert_eq!(s.encoded_bytes(), 0);
}

#[test]
fn retry_proof_transport_cannot_create_authority_after_logout() {
    let mut f = Fixture::new();
    let mut s = store();
    let original =
        created(establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap());
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.signature.signature.clear();
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap(),
        SessionEstablished::ExactRetry(original)
    );
    s.clear(p(3));
    assert_eq!(
        establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)),
        Err(SessionError::ReplayConflict)
    );
    s.prune(220, 128);
    let mut ctx = f.context();
    ctx.now_ns = 250;
    assert!(matches!(
        establish_session(&mut s, &f.token, &request(), &ctx, policy(0)),
        Err(SessionError::Token(_))
    ));
    assert_eq!(s.encoded_bytes(), 0);
}

#[test]
fn stored_record_decode_rejects_corruption_and_unknown_fields() {
    let f = Fixture::new();
    let mut s = store();
    let session =
        created(establish_session(&mut s, &f.token, &request(), &f.context(), policy(0)).unwrap());
    let encoded = session.encode().unwrap();
    let serde_cbor::Value::Map(mut fields) = serde_cbor::from_slice(&encoded).unwrap() else {
        panic!("record map");
    };
    fields.insert(
        serde_cbor::Value::Text("unexpected".into()),
        serde_cbor::Value::Bool(true),
    );
    assert_eq!(
        Session::decode(&serde_cbor::to_vec(&fields).unwrap()),
        Err(SessionError::InvalidPolicyOrRecord)
    );
    fields.remove(&serde_cbor::Value::Text("unexpected".into()));
    fields.insert(
        serde_cbor::Value::Text("scopes".into()),
        serde_cbor::Value::Array(vec![serde_cbor::Value::Text("read".into()); 2]),
    );
    assert_eq!(
        Session::decode(&serde_cbor::to_vec(&fields).unwrap()),
        Err(SessionError::InvalidPolicyOrRecord)
    );
}
