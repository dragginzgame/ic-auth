#![cfg(all(feature = "sessions", feature = "portable-signature-verification"))]
use ed25519_dalek::{Signer, SigningKey};
use ic_auth::{
    admission_budget::SessionAdmissionBudget,
    portable::{
        LoginRevocationError, MemoryLoginRevocations, PortableClaimsContext, PortableClaimsError,
        browser_principal, claims_hash, encode_claims,
    },
    portable_signature::verify_ed25519_claims,
    session::*,
    session_request::SessionRequest,
};
use ic_auth_protocol_types::*;
use sha2::{Digest, Sha256};
const SECOND: u64 = 1_000_000_000;
const NOW: u64 = 10 * SECOND;

struct Fixture {
    key: SigningKey,
    claims: PortableClaims,
    role: AuthRole,
}
impl Fixture {
    fn new(device: u8) -> Self {
        let key = SigningKey::from_bytes(&[42; 32]);
        let role: AuthRole = "user".parse().unwrap();
        let claims = PortableClaims {
            mechanism: PortableMechanism::Ed25519,
            audience: AudienceId {
                canonical_network_id: CanonicalId::from_bytes([1; 32]),
                fleet_id: CanonicalId::from_bytes([2; 32]),
            },
            issuer: Principal::from_slice(&[1]),
            account_subject: Principal::from_slice(&[2]),
            key_binding_id: Sha256::digest(key.verifying_key().to_bytes()).into(),
            browser_public_key: SigningKey::from_bytes(&[device; 32])
                .verifying_key()
                .to_bytes(),
            login_started_at_ns: NOW - SECOND,
            login_deadline_ns: NOW + 60 * SECOND,
            issued_at_ns: NOW,
            admission_expires_at_ns: NOW + 2 * SECOND,
            nonce: [1; 16],
            grants: vec![DelegatedRoleGrant {
                target: role.clone(),
                scopes: vec!["read".into(), "write".into()],
            }],
        };
        Self { key, claims, role }
    }
    fn caller(&self) -> Principal {
        browser_principal(&self.claims.browser_public_key)
    }
    fn context(&self, now: u64) -> SessionContext<'_> {
        SessionContext {
            authority: PortableClaimsContext {
                audience: &self.claims.audience,
                issuer: self.claims.issuer,
                mechanism: self.claims.mechanism,
                key_binding_id: self.claims.key_binding_id,
                grants: &self.claims.grants,
                caller: self.caller(),
                now_ns: now,
                authority_not_before_ns: 0,
                authority_expires_at_ns: NOW + 120 * SECOND,
                enrollment_expires_at_ns: NOW + 90 * SECOND,
                min_login_started_at_ns: 0,
                min_proof_issued_at_ns: 0,
                max_login_ttl_ns: 86_400 * SECOND,
                max_proof_ttl_ns: 60 * SECOND,
            },
            role: &self.role,
            required_scopes: &[],
        }
    }
    fn admit(&self, receiver: &mut Receiver, now: u64) -> Result<SessionEstablished, SessionError> {
        let context = self.context(now);
        let bytes = encode_claims(&self.claims).unwrap();
        let sig = self
            .key
            .sign(&claims_hash(&self.claims).unwrap())
            .to_bytes();
        establish_session(
            &mut receiver.store,
            &mut receiver.ledger,
            &mut receiver.budget,
            &bytes,
            &request(),
            &context,
            policy(),
            |budget| {
                Ok(verify_ed25519_claims(
                    budget,
                    &bytes,
                    &sig,
                    &context.authority,
                    &self.key.verifying_key().to_bytes(),
                )?)
            },
        )
    }
}
struct Receiver {
    store: MemorySessionStore,
    ledger: MemoryLoginRevocations,
    budget: SessionAdmissionBudget,
    logout: SessionAdmissionBudget,
}
impl Receiver {
    fn new(f: &Fixture) -> Self {
        Self {
            store: MemorySessionStore::new(0, limits()).unwrap(),
            ledger: MemoryLoginRevocations::new(f.claims.audience, 4, 4, SECOND, 0).unwrap(),
            budget: SessionAdmissionBudget::new(32, 1, 0).unwrap(),
            logout: SessionAdmissionBudget::new(32, 1, 0).unwrap(),
        }
    }
    fn auth(&mut self, f: &Fixture, now: u64) -> Result<Session, SessionError> {
        authorize_session(&mut self.store, &mut self.ledger, &f.context(now), policy())
    }
    fn logout(&mut self, caller: Principal, now: u64) -> Result<u64, LogoutResult> {
        match logout_session(
            &mut self.store,
            &mut self.ledger,
            &mut self.logout,
            caller,
            now,
        ) {
            LogoutResult::Revoked { deny_until_ns } => Ok(deny_until_ns),
            other => Err(other),
        }
    }
}
fn policy() -> SessionPolicy {
    SessionPolicy {
        generation: 0,
        enabled: true,
        subject_admissible: true,
        default_ttl_secs: 5,
        max_ttl_secs: 30,
    }
}
fn limits() -> SessionLimits {
    SessionLimits {
        max_sessions: 4,
        max_replays: 8,
        max_replays_per_caller: 4,
        max_encoded_bytes: 16_384,
    }
}
fn request() -> SessionRequest {
    SessionRequest::new(vec!["read".into()], None).unwrap()
}
fn created(value: SessionEstablished) -> Session {
    match value {
        SessionEstablished::Created(s) => s,
        other => panic!("{other:?}"),
    }
}

#[test]
fn two_receivers_two_devices_retry_after_proof_expiry_and_local_logout() {
    let f = Fixture::new(1);
    let other = Fixture::new(2);
    let mut a = Receiver::new(&f);
    let mut b = Receiver::new(&f);
    let session = created(f.admit(&mut a, NOW).unwrap());
    created(f.admit(&mut b, NOW).unwrap());
    created(other.admit(&mut a, NOW).unwrap());
    assert_ne!(session.caller(), session.subject());
    assert_eq!(
        (session.issuer(), session.subject()),
        (f.claims.issuer, f.claims.account_subject)
    );
    let bytes = encode_claims(&f.claims).unwrap();
    let ctx = f.context(NOW + 3 * SECOND);
    assert_eq!(
        establish_session(
            &mut a.store,
            &mut a.ledger,
            &mut a.budget,
            &bytes,
            &request(),
            &ctx,
            policy(),
            |_| panic!("exact retry must not verify again")
        ),
        Ok(SessionEstablished::ExactRetry(session.clone()))
    );
    assert_eq!(a.auth(&f, NOW + 3 * SECOND), Ok(session));
    let until = a.logout(f.caller(), NOW + 3 * SECOND).unwrap();
    assert_eq!(a.logout(f.caller(), NOW + 4 * SECOND), Ok(until));
    assert!(matches!(
        a.auth(&f, NOW + 4 * SECOND),
        Err(SessionError::Revocation(LoginRevocationError::Revoked))
    ));
    assert!(other.admit(&mut a, NOW + 4 * SECOND).is_ok());
    assert!(b.auth(&f, NOW + 4 * SECOND).is_ok());
    assert!(matches!(
        f.admit(&mut a, NOW + 4 * SECOND),
        Err(SessionError::Revocation(LoginRevocationError::Revoked))
    ));
    b.logout(f.caller(), NOW + 4 * SECOND).unwrap();
    assert!(b.auth(&f, NOW + 4 * SECOND).is_err());
    a.store
        .transaction(|tx| {
            assert!(tx.replay_exists(claims_hash(&f.claims).unwrap())?);
            Ok(())
        })
        .unwrap();
}

#[test]
fn renewal_replaces_only_within_original_login_and_logout_blocks_fresh_proofs() {
    let mut f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let first = created(f.admit(&mut r, NOW).unwrap());
    f.claims.issued_at_ns = NOW + 4 * SECOND;
    f.claims.admission_expires_at_ns = NOW + 6 * SECOND;
    f.claims.nonce = [2; 16];
    let SessionEstablished::Replaced(second) = f.admit(&mut r, NOW + 4 * SECOND).unwrap() else {
        panic!("renewal")
    };
    assert!(second.expires_at_ns() > first.expires_at_ns());
    let original_subject = f.claims.account_subject;
    f.claims.account_subject = Principal::from_slice(&[9]);
    f.claims.nonce = [3; 16];
    assert!(matches!(
        f.admit(&mut r, NOW + 4 * SECOND),
        Err(SessionError::Revocation(
            LoginRevocationError::InvalidPolicy
        ))
    ));
    f.claims.account_subject = original_subject;
    r.logout(f.caller(), NOW + 4 * SECOND).unwrap();
    assert!(matches!(
        f.admit(&mut r, NOW + 4 * SECOND),
        Err(SessionError::Revocation(LoginRevocationError::Revoked))
    ));
    let mut absent = Receiver::new(&f);
    absent.logout(f.caller(), NOW + 4 * SECOND).unwrap();
    assert!(f.admit(&mut absent, NOW + 4 * SECOND).is_err());
}

struct FailingStore<'a>(&'a mut MemorySessionStore);
impl SessionStore for FailingStore<'_> {
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
// Fail only the write transaction, so proof verification and staging really happen.
struct FailWrite<'a>(&'a mut MemorySessionStore, usize);
impl SessionStore for FailWrite<'_> {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        self.1 += 1;
        if self.1 == 2 {
            FailingStore(self.0).transaction(operation)
        } else {
            self.0.transaction(operation)
        }
    }
}
#[test]
fn failed_commit_is_atomic_for_session_replay_reservation_and_logout() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let ctx = f.context(NOW);
    let bytes = encode_claims(&f.claims).unwrap();
    let signature = f.key.sign(&claims_hash(&f.claims).unwrap()).to_bytes();
    let mut budget = SessionAdmissionBudget::new(1, SECOND, NOW - SECOND).unwrap();
    assert_eq!(
        establish_session(
            &mut FailWrite(&mut r.store, 0),
            &mut r.ledger,
            &mut budget,
            &bytes,
            &request(),
            &ctx,
            policy(),
            |budget| Ok(verify_ed25519_claims(
                budget,
                &bytes,
                &signature,
                &ctx.authority,
                &f.key.verifying_key().to_bytes()
            )?)
        ),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(r.ledger.occupancy(), (0, 0));
    assert_eq!(r.store.encoded_bytes(), 0);
    r.store
        .transaction(|tx| {
            assert!(!tx.replay_exists(claims_hash(&f.claims).unwrap())?);
            Ok(())
        })
        .unwrap();
    assert!(
        verify_ed25519_claims(
            &mut budget,
            &bytes,
            &signature,
            &ctx.authority,
            &f.key.verifying_key().to_bytes()
        )
        .is_err()
    );
    let session = created(f.admit(&mut r, NOW).unwrap());
    let size = r.store.encoded_bytes();
    assert_eq!(
        logout_session(
            &mut FailingStore(&mut r.store),
            &mut r.ledger,
            &mut r.logout,
            f.caller(),
            NOW
        ),
        LogoutResult::Unavailable(LogoutUnavailable::Storage)
    );
    assert_eq!(r.store.encoded_bytes(), size);
    assert_eq!(r.auth(&f, NOW), Ok(session));
    assert!(!r.ledger.is_revoked(f.caller(), NOW).unwrap());
    r.logout(f.caller(), NOW).unwrap();
    assert!(r.auth(&f, NOW).is_err());
}

#[test]
fn live_authority_scope_generation_expiry_and_lost_ledger_are_enforced() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let session = created(f.admit(&mut r, NOW).unwrap());
    let mut ctx = f.context(NOW);
    let mut changed = policy();
    changed.enabled = false;
    assert_eq!(
        authorize_session(&mut r.store, &mut r.ledger, &ctx, changed),
        Err(SessionError::Disabled)
    );
    let required = ["write".into()];
    ctx.required_scopes = &required;
    assert_eq!(
        authorize_session(&mut r.store, &mut r.ledger, &ctx, policy()),
        Err(SessionError::ScopeRejected)
    );
    ctx = f.context(NOW);
    ctx.authority.key_binding_id = [99; 32];
    assert_eq!(
        authorize_session(&mut r.store, &mut r.ledger, &ctx, policy()),
        Err(SessionError::StaleAuthority)
    );
    ctx = f.context(NOW);
    ctx.authority.enrollment_expires_at_ns = NOW;
    assert_eq!(
        authorize_session(&mut r.store, &mut r.ledger, &ctx, policy()),
        Err(SessionError::StaleAuthority)
    );
    ctx = f.context(NOW);
    ctx.authority.authority_expires_at_ns = NOW;
    assert_eq!(
        authorize_session(&mut r.store, &mut r.ledger, &ctx, policy()),
        Err(SessionError::StaleAuthority)
    );
    let empty: [DelegatedRoleGrant; 0] = [];
    ctx = f.context(NOW);
    ctx.authority.grants = &empty;
    assert!(authorize_session(&mut r.store, &mut r.ledger, &ctx, policy()).is_err());
    let mut lost = MemoryLoginRevocations::new(f.claims.audience, 4, 4, SECOND, 0).unwrap();
    assert!(authorize_session(&mut r.store, &mut lost, &f.context(NOW), policy()).is_err());
    assert_eq!(
        r.auth(&f, session.expires_at_ns()),
        Err(SessionError::MissingOrExpired)
    );
    assert!(r.logout(f.caller(), NOW + 200 * SECOND).is_ok()); // no live enrollment required
    let mut r = Receiver::new(&f);
    f.admit(&mut r, NOW).unwrap();
    let mut p = policy();
    p.generation = r.store.advance_generation().unwrap();
    assert_eq!(
        authorize_session(&mut r.store, &mut r.ledger, &f.context(NOW), p),
        Err(SessionError::StaleAuthority)
    );
}

#[test]
fn replay_capacity_narrowing_and_reset_refuse_before_verification() {
    let mut f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let bytes = encode_claims(&f.claims).unwrap();
    let excessive = SessionRequest::new(vec!["admin".into()], None).unwrap();
    assert_eq!(
        establish_session(
            &mut r.store,
            &mut r.ledger,
            &mut r.budget,
            &bytes,
            &excessive,
            &f.context(NOW),
            policy(),
            |_| panic!("scope preflight")
        ),
        Err(SessionError::ScopeRejected)
    );
    created(f.admit(&mut r, NOW).unwrap());
    let changed = SessionRequest::new(vec!["write".into()], None).unwrap();
    assert_eq!(
        establish_session(
            &mut r.store,
            &mut r.ledger,
            &mut r.budget,
            &bytes,
            &changed,
            &f.context(NOW),
            policy(),
            |_| panic!("exact retry mismatch")
        ),
        Err(SessionError::ReplayConflict)
    );
    r.store = MemorySessionStore::new(
        0,
        SessionLimits {
            max_replays: 1,
            ..limits()
        },
    )
    .unwrap();
    f.admit(&mut r, NOW).unwrap();
    f.claims.nonce = [3; 16];
    let bytes = encode_claims(&f.claims).unwrap();
    assert_eq!(
        establish_session(
            &mut r.store,
            &mut r.ledger,
            &mut r.budget,
            &bytes,
            &request(),
            &f.context(NOW),
            policy(),
            |_| panic!("full store")
        ),
        Err(SessionError::Capacity)
    );
    let mut reset = Receiver::new(&f);
    let mut ctx = f.context(NOW);
    ctx.authority.advance_reset_floors(SECOND).unwrap();
    assert!(matches!(
        establish_session(
            &mut reset.store,
            &mut reset.ledger,
            &mut reset.budget,
            &bytes,
            &request(),
            &ctx,
            policy(),
            |_| panic!("pre-reset login")
        ),
        Err(SessionError::Claims(PortableClaimsError::BeforeReset))
    ));
}

#[cfg(feature = "session-restore")]
#[test]
fn portable_record_roundtrip_and_old_or_corrupt_records_are_rejected() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let session = created(f.admit(&mut r, NOW).unwrap());
    let bytes = session.encode().unwrap();
    assert_eq!(Session::decode(&bytes), Ok(session));
    assert!(Session::decode(&bytes[b"IC-AUTH-PORTABLE-SESSION\0".len()..]).is_err());
    assert!(Session::decode(&[0; 2049]).is_err());
    let mut truncated = bytes;
    truncated.pop();
    assert!(Session::decode(&truncated).is_err());
}

#[test]
fn invalid_signature_cannot_reserve_state_and_spends_verification_credit() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let ctx = f.context(NOW);
    let bytes = encode_claims(&f.claims).unwrap();
    let mut budget = SessionAdmissionBudget::new(1, SECOND, NOW - SECOND).unwrap();
    let result = establish_session(
        &mut r.store,
        &mut r.ledger,
        &mut budget,
        &bytes,
        &request(),
        &ctx,
        policy(),
        |budget| {
            Ok(verify_ed25519_claims(
                budget,
                &bytes,
                &[0; 64],
                &ctx.authority,
                &f.key.verifying_key().to_bytes(),
            )?)
        },
    );
    assert!(matches!(result, Err(SessionError::Signature(_))));
    assert_eq!(r.store.encoded_bytes(), 0);
    assert_eq!(r.ledger.occupancy(), (0, 0));
    assert!(matches!(
        verify_ed25519_claims(
            &mut budget,
            &bytes,
            &[0; 64],
            &ctx.authority,
            &f.key.verifying_key().to_bytes()
        ),
        Err(ic_auth::portable_signature::PortableSignatureError::Budget(
            _
        ))
    ));
}

#[test]
fn bounded_pruning_releases_exact_bytes_and_caller_replay_capacity() {
    let mut f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    r.store = MemorySessionStore::new(
        0,
        SessionLimits {
            max_replays_per_caller: 1,
            ..limits()
        },
    )
    .unwrap();
    let session = created(f.admit(&mut r, NOW).unwrap());
    let original_bytes = r.store.encoded_bytes();
    f.claims.nonce = [2; 16];
    assert_eq!(f.admit(&mut r, NOW), Err(SessionError::Capacity));
    assert_eq!(r.store.encoded_bytes(), original_bytes);
    assert_eq!(r.auth(&f, NOW), Ok(session.clone()));
    assert_eq!(r.store.prune(NOW + 2 * SECOND, 0), SessionPruned::default());
    assert_eq!(
        r.store.prune(NOW + 2 * SECOND, 1),
        SessionPruned {
            sessions: 0,
            replays: 1
        }
    );
    assert_eq!(r.store.encoded_bytes(), session.encode().unwrap().len());
    f.claims.issued_at_ns = NOW + 2 * SECOND;
    f.claims.admission_expires_at_ns = NOW + 4 * SECOND;
    assert!(matches!(
        f.admit(&mut r, NOW + 2 * SECOND),
        Ok(SessionEstablished::Replaced(_))
    ));
    assert_eq!(
        r.store.prune(NOW + 10 * SECOND, 1),
        SessionPruned {
            sessions: 0,
            replays: 1
        }
    );
    assert_eq!(
        r.store.prune(NOW + 10 * SECOND, 1),
        SessionPruned {
            sessions: 1,
            replays: 0
        }
    );
    assert_eq!(r.store.encoded_bytes(), 0);
    r.store
        .transaction(|tx| {
            assert_eq!(tx.occupancy(f.caller())?, SessionOccupancy::default());
            Ok(())
        })
        .unwrap();
    // Pruning session/replay storage cannot remove the immutable login binding.
    assert_eq!(r.ledger.occupancy(), (1, 0));
}

#[test]
fn byte_capacity_failure_preserves_previous_session_and_login_reservation() {
    let mut f = Fixture::new(1);
    let mut sizing = Receiver::new(&f);
    f.admit(&mut sizing, NOW).unwrap();
    let mut r = Receiver::new(&f);
    r.store = MemorySessionStore::new(
        0,
        SessionLimits {
            max_encoded_bytes: sizing.store.encoded_bytes(),
            ..limits()
        },
    )
    .unwrap();
    let session = created(f.admit(&mut r, NOW).unwrap());
    let before = r.store.encoded_bytes();
    f.claims.nonce = [2; 16];
    assert_eq!(f.admit(&mut r, NOW), Err(SessionError::ByteCapacity));
    assert_eq!(r.store.encoded_bytes(), before);
    assert_eq!(r.auth(&f, NOW), Ok(session));
    assert_eq!(r.ledger.occupancy(), (1, 0));
    r.store
        .transaction(|tx| {
            assert!(!tx.replay_exists(claims_hash(&f.claims).unwrap())?);
            Ok(())
        })
        .unwrap();
}

#[test]
fn logout_reports_typed_refusals_without_acknowledging_revocation() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    assert_eq!(
        r.logout(Principal::anonymous(), NOW),
        Err(LogoutResult::InvalidCaller)
    );
    assert_eq!(
        r.logout(Principal::from_slice(&[1]), NOW),
        Err(LogoutResult::InvalidCaller)
    );
    // Unknown credit starts empty; refusing it must not retain a tombstone.
    assert_eq!(
        r.logout(f.caller(), 0),
        Err(LogoutResult::Unavailable(LogoutUnavailable::Capacity))
    );
    assert!(!r.ledger.is_revoked(f.caller(), 0).unwrap());
    let until = r.logout(f.caller(), NOW).unwrap();
    assert_eq!(
        r.logout(f.caller(), NOW - 1),
        Err(LogoutResult::Unavailable(LogoutUnavailable::Clock))
    );
    assert_eq!(r.logout(f.caller(), NOW), Ok(until));
}
