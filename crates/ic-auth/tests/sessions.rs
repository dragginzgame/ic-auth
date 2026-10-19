#![cfg(all(feature = "sessions", feature = "portable-signature-verification"))]
use ic_auth::{
    admission_budget::SessionAdmissionBudget,
    portable::{
        LoginRevocationError, MemoryLoginRevocations, PortableClaimsContext, PortableClaimsError,
        browser_principal, claims_hash, encode_claims,
    },
    portable_credential::PortableVerificationContext,
    portable_signature::verify_ed25519_claims,
    session::*,
    session_request::SessionRequest,
};
use ic_auth_protocol_types::*;
use ic_platform::ed25519::{Signer, SigningKey};
use sha2::{Digest, Sha256};
const SECOND: u64 = 1_000_000_000;
const NOW: u64 = 10 * SECOND;

#[cfg(feature = "session-restore")]
#[test]
fn storage_record_domains_and_capacity_errors_remain_distinct() {
    use ic_auth::portable::LoginRevocationChange;
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    f.admit(&mut r, NOW).unwrap();
    let session = r.store.session_records().next().unwrap().encode().unwrap();
    let replay = r.store.replay_records().next().unwrap().encode().unwrap();
    let login = r.ledger.records().next().unwrap().encode().unwrap();
    assert_eq!(
        Session::decode(&session).unwrap().encode().unwrap(),
        session
    );
    assert_eq!(Replay::decode(&replay).unwrap().encode().unwrap(), replay);
    assert_eq!(
        LoginRevocationChange::decode(&login)
            .unwrap()
            .encode()
            .unwrap(),
        login
    );
    for bytes in [&replay, &login] {
        assert_eq!(
            Session::decode(bytes).unwrap_err(),
            SessionError::InvalidPolicyOrRecord
        );
    }
    for bytes in [&session, &login] {
        assert_eq!(
            Replay::decode(bytes).unwrap_err(),
            if bytes.len() > 512 {
                SessionError::ByteCapacity
            } else {
                SessionError::InvalidPolicyOrRecord
            }
        );
    }
    for bytes in [&session, &replay] {
        assert!(matches!(
            LoginRevocationChange::decode(bytes),
            Err(LoginRevocationError::InvalidPolicy)
        ));
    }
    // Outer limits precede wrong-domain and malformed-CBOR errors. At the exact
    // limit, malformed input must still be a framing error, not a capacity error.
    assert_eq!(
        Session::decode(&[0; 2048]).unwrap_err(),
        SessionError::InvalidPolicyOrRecord
    );
    assert_eq!(
        Session::decode(&[0; 2049]).unwrap_err(),
        SessionError::ByteCapacity
    );
    assert_eq!(
        Replay::decode(&[0; 512]).unwrap_err(),
        SessionError::InvalidPolicyOrRecord
    );
    assert_eq!(
        Replay::decode(&[0; 513]).unwrap_err(),
        SessionError::ByteCapacity
    );
    assert!(matches!(
        LoginRevocationChange::decode(&[0; 1025]),
        Err(LoginRevocationError::InvalidPolicy)
    ));
}

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
            |budget, decoded| {
                Ok(decoded.verify(
                    budget,
                    &PortableProof::Ed25519 {
                        signature: sig.to_vec(),
                    },
                    &PortableVerificationContext::Ed25519 {
                        claims: &context.authority,
                        public_key: &self.key.verifying_key().to_bytes(),
                    },
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
            |_, _| panic!("exact retry must not verify again")
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
        .transaction(None, |tx| {
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
        change: Option<&ic_auth::portable::LoginRevocationChange>,
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        self.0
            .transaction_with_commit(change, operation, |_| Err(SessionError::StorageFailure))
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
            &mut FailingStore(&mut r.store),
            &mut r.ledger,
            &mut budget,
            &bytes,
            &request(),
            &ctx,
            policy(),
            |budget, decoded| Ok(decoded.verify(
                budget,
                &PortableProof::Ed25519 {
                    signature: signature.to_vec()
                },
                &PortableVerificationContext::Ed25519 {
                    claims: &ctx.authority,
                    public_key: &f.key.verifying_key().to_bytes()
                }
            )?)
        ),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(r.ledger.occupancy(), (0, 0));
    assert_eq!(r.store.encoded_bytes(), 0);
    r.store
        .transaction(None, |tx| {
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
            |_, _| panic!("scope preflight")
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
            |_, _| panic!("exact retry mismatch")
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
            |_, _| panic!("full store")
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
            |_, _| panic!("pre-reset login")
        ),
        Err(SessionError::Claims(PortableClaimsError::BeforeReset))
    ));
}

#[cfg(feature = "session-restore")]
#[test]
fn restored_session_selects_exact_enrollment_among_same_issuer_keys() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let admitted = created(f.admit(&mut r, NOW).unwrap());
    let restored = Session::decode(&admitted.encode().unwrap()).unwrap();
    let identity = (
        restored.issuer(),
        restored.mechanism(),
        restored.key_binding_id(),
    );
    let approved = (f.claims.issuer, f.claims.mechanism, f.claims.key_binding_id);
    let other_key = (f.claims.issuer, f.claims.mechanism, [99; 32]);
    let other_mechanism = (
        f.claims.issuer,
        PortableMechanism::Certified,
        f.claims.key_binding_id,
    );
    let enrollment = [other_key, other_mechanism, approved];
    assert_eq!(
        enrollment.iter().find(|entry| **entry == identity),
        Some(&approved)
    );

    r.store = restored_store(&r.store);
    assert_eq!(r.auth(&f, NOW), Ok(restored));
    // The selected record cannot authorize another key or mechanism of that issuer.
    for (_, mechanism, key_binding_id) in [other_key, other_mechanism] {
        let mut context = f.context(NOW);
        context.authority.mechanism = mechanism;
        context.authority.key_binding_id = key_binding_id;
        assert_eq!(
            authorize_session(&mut r.store, &mut r.ledger, &context, policy()),
            Err(SessionError::StaleAuthority)
        );
    }
    // Selection is not authorization: expiry still closes the same authority.
    let mut context = f.context(NOW);
    context.authority.enrollment_expires_at_ns = NOW;
    assert_eq!(
        authorize_session(&mut r.store, &mut r.ledger, &context, policy()),
        Err(SessionError::StaleAuthority)
    );
    assert!(
        r.auth(&f, NOW).is_ok(),
        "refusals preserve the retained session"
    );
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
fn callback_result_must_match_original_claims_and_current_clock() {
    for changed_claims in [true, false] {
        let f = Fixture::new(1);
        let mut receiver = Receiver::new(&f);
        receiver.budget = SessionAdmissionBudget::new(1, SECOND, NOW - SECOND).unwrap();
        let bytes = encode_claims(&f.claims).unwrap();
        let context = f.context(NOW);
        let mut other = f.claims.clone();
        if changed_claims {
            other.nonce = [9; 16];
        }
        let other_bytes = encode_claims(&other).unwrap();
        let signature = f.key.sign(&claims_hash(&other).unwrap()).to_bytes();
        let mut verification = f.context(NOW);
        if !changed_claims {
            verification.authority.now_ns += 1;
        }
        assert_eq!(
            establish_session(
                &mut receiver.store,
                &mut receiver.ledger,
                &mut receiver.budget,
                &bytes,
                &request(),
                &context,
                policy(),
                |budget, _decoded| Ok(verify_ed25519_claims(
                    budget,
                    &other_bytes,
                    &signature,
                    &verification.authority,
                    &f.key.verifying_key().to_bytes(),
                )?),
            ),
            Err(SessionError::InvalidPolicyOrRecord),
        );
        assert_eq!(receiver.store.encoded_bytes(), 0);
        assert_eq!(receiver.ledger.occupancy(), (0, 0));
        // Refused callback results must not undo work already charged by verification.
        assert!(matches!(
            verify_ed25519_claims(
                &mut receiver.budget,
                &bytes,
                &f.key.sign(&claims_hash(&f.claims).unwrap()).to_bytes(),
                &verification.authority,
                &f.key.verifying_key().to_bytes(),
            ),
            Err(ic_auth::portable_signature::PortableSignatureError::Budget(
                _
            ))
        ));
    }
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
        |budget, decoded| {
            Ok(decoded.verify(
                budget,
                &PortableProof::Ed25519 {
                    signature: [0; 64].to_vec(),
                },
                &PortableVerificationContext::Ed25519 {
                    claims: &ctx.authority,
                    public_key: &f.key.verifying_key().to_bytes(),
                },
            )?)
        },
    );
    assert!(matches!(result, Err(SessionError::Credential(_))));
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
        .transaction(None, |tx| {
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
    r.budget = SessionAdmissionBudget::new(2, SECOND, 0).unwrap();
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
    assert_eq!(r.auth(&f, NOW).as_ref(), Ok(&session));
    assert_eq!(r.ledger.occupancy(), (1, 0));
    // The failed stage keeps its verification charge, but consumes no replay.
    assert_eq!(
        f.admit(&mut r, NOW),
        Err(SessionError::Credential(
            ic_auth::portable_credential::PortableCredentialError::Ed25519(
                ic_auth::portable_signature::PortableSignatureError::Budget(
                    ic_auth::admission_budget::AdmissionBudgetError::Exhausted,
                ),
            ),
        )),
    );
    f.claims.nonce = [1; 16];
    assert_eq!(
        f.admit(&mut r, NOW),
        Ok(SessionEstablished::ExactRetry(session))
    );
    f.claims.nonce = [2; 16];
    r.store
        .transaction(None, |tx| {
            assert_eq!(
                tx.occupancy(f.caller())?,
                SessionOccupancy {
                    sessions: 1,
                    replays: 1,
                    replays_for_caller: 1,
                }
            );
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

#[test]
fn transaction_refuses_missing_or_mismatched_logout_change_without_partial_effects() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let session = created(f.admit(&mut r, NOW).unwrap());
    assert_eq!(
        r.store.transaction(None, |tx| tx.remove(f.caller())),
        Err(SessionError::InvalidPolicyOrRecord)
    );
    let other = Fixture::new(2);
    assert_eq!(
        r.ledger
            .with_revocation(&mut r.logout, other.caller(), NOW, |change| {
                r.store
                    .transaction(Some(change), |tx| tx.remove(f.caller()))
                    .map_err(|_| LoginRevocationError::Storage)
            }),
        Err(LoginRevocationError::Storage)
    );
    assert!(!r.ledger.is_revoked(other.caller(), NOW).unwrap());
    let mut foreign_audience = f.claims.audience;
    foreign_audience.fleet_id = CanonicalId::from_bytes([77; 32]);
    let mut foreign = MemoryLoginRevocations::new(foreign_audience, 4, 4, SECOND, 0).unwrap();
    assert_eq!(
        foreign.with_revocation(&mut r.logout, f.caller(), NOW, |change| {
            r.store
                .transaction(Some(change), |tx| tx.remove(f.caller()))
                .map_err(|_| LoginRevocationError::Storage)
        }),
        Err(LoginRevocationError::Storage)
    );
    assert!(!foreign.is_revoked(f.caller(), NOW).unwrap());
    assert_eq!(r.auth(&f, NOW), Ok(session));
    r.logout(f.caller(), NOW).unwrap();
    assert!(r.auth(&f, NOW).is_err());
}

#[cfg(feature = "session-restore")]
#[test]
fn restored_login_records_gate_retained_sessions_and_reject_revoked_retries() {
    use ic_auth::portable::LoginRevocationChange;
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let session = created(f.admit(&mut r, NOW).unwrap());
    let bytes = encode_claims(&f.claims).unwrap();
    let signature = f.key.sign(&claims_hash(&f.claims).unwrap()).to_bytes();
    let verified = verify_ed25519_claims(
        &mut r.budget,
        &bytes,
        &signature,
        &f.context(NOW).authority,
        &f.key.verifying_key().to_bytes(),
    )
    .unwrap();
    let mut stored = Vec::new();
    r.ledger
        .with_reservation(f.caller(), &verified, NOW, |change| {
            stored = change.encode().unwrap();
            Ok(())
        })
        .unwrap();
    let restore = |bytes: &[u8]| {
        MemoryLoginRevocations::new(f.claims.audience, 4, 4, SECOND, NOW)
            .unwrap()
            .restore_records(NOW, (1, 0), [LoginRevocationChange::decode(bytes).unwrap()])
            .unwrap()
    };
    r.ledger = restore(&stored);
    assert_eq!(r.auth(&f, NOW), Ok(session));
    assert!(matches!(
        f.admit(&mut r, NOW).unwrap(),
        SessionEstablished::ExactRetry(_)
    ));
    // Deliberately retain the session to prove that restored denial gates it too.
    r.ledger
        .with_revocation(&mut r.logout, f.caller(), NOW, |change| {
            stored = change.encode().unwrap();
            Ok(())
        })
        .unwrap();
    r.ledger = restore(&stored);
    assert!(r.auth(&f, NOW).is_err());
    assert!(f.admit(&mut r, NOW).is_err());
}

#[cfg(feature = "session-restore")]
fn restored_store(store: &MemorySessionStore) -> MemorySessionStore {
    MemorySessionStore::new(store.generation(), limits())
        .unwrap()
        .restore_records(
            (store.session_records().len(), store.replay_records().len()),
            store.encoded_bytes(),
            store
                .session_records()
                .map(|s| Session::decode(&s.encode().unwrap()).unwrap()),
            store
                .replay_records()
                .map(|r| Replay::decode(&r.encode().unwrap()).unwrap()),
        )
        .unwrap()
}

#[cfg(feature = "session-restore")]
#[test]
fn restored_store_preserves_sessions_replay_history_and_pruning_indexes() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let session = created(f.admit(&mut r, NOW).unwrap());
    r.store = restored_store(&r.store);
    assert_eq!(r.auth(&f, NOW), Ok(session.clone()));
    assert_eq!(
        f.admit(&mut r, NOW),
        Ok(SessionEstablished::ExactRetry(session))
    );
    // Expired replay can be pruned before the still-live session; restore must
    // support that valid state without inventing a replacement replay record.
    assert_eq!(r.store.prune(NOW + 2 * SECOND, 1).replays, 1);
    r.store = restored_store(&r.store);
    assert!(r.auth(&f, NOW + 2 * SECOND).is_ok());

    let mut r = Receiver::new(&f);
    f.admit(&mut r, NOW).unwrap();
    r.logout(f.caller(), NOW).unwrap();
    r.store = restored_store(&r.store);
    assert_eq!(r.store.session_records().len(), 0);
    assert_eq!(r.store.replay_records().len(), 1);
    // Even a lost login ledger must not erase consumed-proof evidence.
    r.ledger = MemoryLoginRevocations::new(f.claims.audience, 4, 4, SECOND, NOW).unwrap();
    assert_eq!(f.admit(&mut r, NOW), Err(SessionError::ReplayConflict));
    r.store.advance_generation().unwrap();
    r.store = restored_store(&r.store);
    assert_eq!(r.store.generation(), 1);
    assert_eq!(r.store.replay_records().len(), 1);
    assert_eq!(r.store.prune(NOW + 2 * SECOND, 1).replays, 1);
    assert_eq!(r.store.encoded_bytes(), 0);
}

#[cfg(feature = "session-restore")]
#[test]
fn store_restore_rejects_incomplete_duplicate_future_and_over_capacity_records() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let session = created(f.admit(&mut r, NOW).unwrap());
    let replay = r.store.replay_records().next().unwrap().clone();
    let bytes = r.store.encoded_bytes();
    let restore = |counts, bytes, sessions: Vec<Session>, replays: Vec<Replay>| {
        MemorySessionStore::new(0, limits())
            .unwrap()
            .restore_records(counts, bytes, sessions, replays)
    };
    assert!(restore((1, 1), bytes, vec![], vec![replay.clone()]).is_err());
    assert!(restore((1, 1), bytes, vec![session.clone()], vec![]).is_err());
    assert!(
        restore(
            (1, 1),
            bytes - 1,
            vec![session.clone()],
            vec![replay.clone()]
        )
        .is_err()
    );
    assert!(
        restore(
            (1, 1),
            bytes + 1,
            vec![session.clone()],
            vec![replay.clone()]
        )
        .is_err()
    );
    assert!(
        restore(
            (2, 1),
            bytes * 2,
            vec![session.clone(), session.clone()],
            vec![replay.clone()]
        )
        .is_err()
    );
    assert!(
        restore(
            (1, 2),
            bytes * 2,
            vec![session.clone()],
            vec![replay.clone(), replay.clone()]
        )
        .is_err()
    );
    assert!(restore((0, 1), bytes, vec![session.clone()], vec![replay.clone()]).is_err());
    let mut tight = limits();
    tight.max_encoded_bytes = bytes - 1;
    assert!(
        MemorySessionStore::new(0, tight)
            .unwrap()
            .restore_records((1, 1), bytes, [session.clone()], [replay.clone()])
            .is_err()
    );
    assert!(
        r.store
            .restore_records((1, 1), bytes, [session.clone()], [replay.clone()])
            .is_err()
    );

    let prefix = b"IC-AUTH-PORTABLE-REPLAY\0";
    let encoded = replay.encode().unwrap();
    let mut value: serde_cbor::Value = serde_cbor::from_slice(&encoded[prefix.len()..]).unwrap();
    let serde_cbor::Value::Map(fields) = &mut value else {
        panic!()
    };
    fields.insert(
        serde_cbor::Value::Text("generation".into()),
        serde_cbor::Value::Integer(1),
    );
    let mut encoded = prefix.to_vec();
    encoded.extend(serde_cbor::to_vec(&value).unwrap());
    let future = Replay::decode(&encoded).unwrap();
    assert!(restore((1, 1), bytes, vec![session.clone()], vec![future.clone()]).is_err());
    // Even when both generations are historical, a retained matching fingerprint
    // cannot disagree with the session's caller/generation.
    assert!(
        MemorySessionStore::new(1, limits())
            .unwrap()
            .restore_records((1, 1), bytes, [session], [future])
            .is_err()
    );

    let mut f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    f.admit(&mut r, NOW).unwrap();
    f.claims.nonce = [2; 16];
    f.admit(&mut r, NOW).unwrap();
    let mut tight = limits();
    tight.max_replays_per_caller = 1;
    assert!(
        MemorySessionStore::new(0, tight)
            .unwrap()
            .restore_records(
                (1, 2),
                r.store.encoded_bytes(),
                r.store.session_records().cloned(),
                r.store.replay_records().cloned(),
            )
            .is_err()
    );
}

#[cfg(feature = "session-restore")]
#[test]
fn replay_storage_decoder_rejects_wrong_domain_truncation_and_invalid_fields() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let session = created(f.admit(&mut r, NOW).unwrap());
    let record = r.store.replay_records().next().unwrap();
    let bytes = record.encode().unwrap();
    assert_eq!(Replay::decode(&bytes).unwrap(), *record);
    assert!(Replay::decode(&session.encode().unwrap()).is_err());
    assert!(Replay::decode(&[0; 513]).is_err());
    for cut in 0..bytes.len() {
        assert!(Replay::decode(&bytes[..cut]).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(Replay::decode(&trailing).is_err());
    let prefix = b"IC-AUTH-PORTABLE-REPLAY\0";
    let value: serde_cbor::Value = serde_cbor::from_slice(&bytes[prefix.len()..]).unwrap();
    for (field, replacement) in [
        ("remove_at_ns", serde_cbor::Value::Integer(0)),
        (
            "caller",
            serde_cbor::value::to_value(Principal::anonymous()).unwrap(),
        ),
        ("unexpected", serde_cbor::Value::Null),
    ] {
        let mut value = value.clone();
        let serde_cbor::Value::Map(fields) = &mut value else {
            panic!()
        };
        fields.insert(serde_cbor::Value::Text(field.into()), replacement);
        let mut invalid = prefix.to_vec();
        invalid.extend(serde_cbor::to_vec(&value).unwrap());
        assert!(Replay::decode(&invalid).is_err());
    }
}

#[cfg(feature = "session-restore")]
#[test]
fn commit_hook_receives_only_validated_delta_and_exact_post_commit_totals() {
    let mut f = Fixture::new(1);
    let mut source = Receiver::new(&f);
    let mut store = MemorySessionStore::new(0, limits()).unwrap();
    let mut commits = 0;
    // Exercise insertion and replacement; only the touched pair is supplied.
    for nonce in [1, 2] {
        f.claims.nonce = [nonce; 16];
        let session = match f.admit(&mut source, NOW).unwrap() {
            SessionEstablished::Created(s) | SessionEstablished::Replaced(s) => s,
            _ => panic!("new proof"),
        };
        let replay = source
            .store
            .replay_records()
            .find(|r| r.fingerprint() == session.fingerprint())
            .unwrap()
            .clone();
        let verified = verify_ed25519_claims(
            &mut source.budget,
            &encode_claims(&f.claims).unwrap(),
            &f.key.sign(&claims_hash(&f.claims).unwrap()).to_bytes(),
            &f.context(NOW).authority,
            &f.key.verifying_key().to_bytes(),
        )
        .unwrap();
        let mut expected_bytes = 0;
        source
            .ledger
            .with_reservation(f.caller(), &verified, NOW, |login| {
                store
                    .transaction_with_commit(
                        Some(login),
                        |tx| tx.stage(session.clone(), replay.clone()),
                        |delta| {
                            assert!(!delta.login.revoked());
                            assert_eq!(delta.login.caller(), f.caller());
                            assert_eq!(delta.session_count, 1);
                            assert_eq!(delta.replay_count, nonce as usize);
                            let (s, r) = delta.admission.unwrap();
                            assert_eq!(s, &session);
                            assert_eq!(r, &replay);
                            // A durable owner has all changed records before heap publication.
                            assert!(!delta.login.encode().unwrap().is_empty());
                            assert!(!s.encode().unwrap().is_empty());
                            assert!(!r.encode().unwrap().is_empty());
                            expected_bytes = delta.encoded_bytes;
                            commits += 1;
                            Ok(())
                        },
                    )
                    .map_err(|_| LoginRevocationError::Storage)
            })
            .unwrap();
        assert_eq!(store.encoded_bytes(), expected_bytes);
        assert_eq!(store.session_records().len(), 1);
        assert_eq!(store.replay_records().len(), nonce as usize);
    }
    // Reads and invalid writes must never invoke external persistence.
    store
        .transaction_with_commit(None, |tx| tx.session(f.caller()), |_| panic!("read commit"))
        .unwrap();
    assert_eq!(
        store.transaction_with_commit(
            None,
            |tx| tx.remove(f.caller()),
            |_| panic!("invalid commit")
        ),
        Err(SessionError::InvalidPolicyOrRecord)
    );
    let before = store.encoded_bytes();
    let session = store.session_records().next().unwrap().clone();
    assert_eq!(
        source
            .ledger
            .with_revocation(&mut source.logout, f.caller(), NOW, |login| {
                store
                    .transaction_with_commit(
                        Some(login),
                        |tx| tx.remove(f.caller()),
                        |_| Err(SessionError::StorageFailure),
                    )
                    .map_err(|_| LoginRevocationError::Storage)
            }),
        Err(LoginRevocationError::Storage)
    );
    assert_eq!(store.encoded_bytes(), before);
    assert_eq!(store.session_records().next().unwrap(), &session);
    assert!(!source.ledger.is_revoked(f.caller(), NOW).unwrap());
    for caller in [f.caller(), Fixture::new(2).caller()] {
        source
            .ledger
            .with_revocation(&mut source.logout, caller, NOW, |login| {
                store
                    .transaction_with_commit(
                        Some(login),
                        |tx| tx.remove(caller),
                        |delta| {
                            assert!(delta.login.revoked());
                            assert_eq!(delta.login.caller(), caller);
                            assert!(delta.admission.is_none());
                            assert_eq!((delta.session_count, delta.replay_count), (0, 2));
                            assert_eq!(
                                delta.encoded_bytes,
                                before - session.encode().unwrap().len()
                            );
                            commits += 1;
                            Ok(())
                        },
                    )
                    .map_err(|_| LoginRevocationError::Storage)
            })
            .unwrap();
    }
    assert_eq!(commits, 4);
    assert_eq!(store.session_records().len(), 0);
    assert_eq!(store.replay_records().len(), 2);
}

#[cfg(feature = "session-restore")]
#[test]
fn durable_pruning_preserves_records_on_failure_and_commits_exact_deletions() {
    let f = Fixture::new(1);
    let other = Fixture::new(2);
    let mut r = Receiver::new(&f);
    f.admit(&mut r, NOW).unwrap();
    other.admit(&mut r, NOW).unwrap();
    let before = r.store.encoded_bytes();
    assert_eq!(
        r.store
            .prune_with_commit(NOW, 128, |_| panic!("unexpired commit"))
            .unwrap(),
        SessionPruned::default()
    );
    assert_eq!(
        r.store
            .prune_with_commit(NOW + 60 * SECOND, 0, |_| panic!("zero budget commit"))
            .unwrap(),
        SessionPruned::default()
    );
    let mut failed_key = None;
    assert_eq!(
        r.store.prune_with_commit(NOW + 60 * SECOND, 1, |delta| {
            assert!(delta.callers.is_empty());
            assert_eq!(delta.fingerprints.len(), 1);
            assert_eq!((delta.session_count, delta.replay_count), (2, 1));
            failed_key = Some(delta.fingerprints[0]);
            Err(SessionError::StorageFailure)
        }),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(r.store.encoded_bytes(), before);
    assert_eq!(r.store.replay_records().len(), 2);
    assert!(r.auth(&f, NOW).is_ok());
    let mut after = 0;
    assert_eq!(
        r.store
            .prune_with_commit(NOW + 60 * SECOND, 1, |delta| {
                assert_eq!(Some(delta.fingerprints[0]), failed_key);
                after = delta.encoded_bytes;
                Ok(())
            })
            .unwrap(),
        SessionPruned {
            sessions: 0,
            replays: 1
        }
    );
    assert_eq!(r.store.encoded_bytes(), after);
    let pruned = r
        .store
        .prune_with_commit(NOW + 60 * SECOND, usize::MAX, |delta| {
            assert_eq!(delta.callers.len(), 2);
            assert_eq!(delta.fingerprints.len(), 1);
            assert_eq!(
                (delta.session_count, delta.replay_count, delta.encoded_bytes),
                (0, 0, 0)
            );
            Ok(())
        })
        .unwrap();
    assert_eq!(
        pruned,
        SessionPruned {
            sessions: 2,
            replays: 1
        }
    );
    assert_eq!(r.store.encoded_bytes(), 0);
    assert_eq!(
        r.store
            .prune_with_commit(u64::MAX, 128, |_| panic!("empty commit"))
            .unwrap(),
        SessionPruned::default()
    );
    // Reconstructed indexes and per-caller counts are empty too.
    r.store = restored_store(&r.store);
    assert_eq!(r.store.replay_records().len(), 0);
}

#[test]
fn authority_generation_changes_only_after_host_commit() {
    let f = Fixture::new(1);
    let mut r = Receiver::new(&f);
    let session = created(f.admit(&mut r, NOW).unwrap());
    assert_eq!(
        r.store.advance_generation_with_commit(|next| {
            assert_eq!(next, 1);
            Err(SessionError::StorageFailure)
        }),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(r.store.generation(), 0);
    assert_eq!(r.auth(&f, NOW), Ok(session));
    assert_eq!(
        r.store
            .advance_generation_with_commit(|next| {
                assert_eq!(next, 1);
                Ok(())
            })
            .unwrap(),
        1
    );
    assert!(r.auth(&f, NOW).is_err());
    let mut exhausted = MemorySessionStore::new(u64::MAX, limits()).unwrap();
    assert_eq!(
        exhausted.advance_generation_with_commit(|_| panic!("overflow commit")),
        Err(SessionError::GenerationExhausted)
    );
}

#[cfg(feature = "session-restore")]
#[test]
fn custom_store_checks_login_binding_through_public_api() {
    let fixture = Fixture::new(1);
    let mut receiver = Receiver::new(&fixture);
    let session = created(fixture.admit(&mut receiver, NOW).unwrap());
    let change = receiver.ledger.records().next().unwrap();
    assert!(session.check_login_change(&change).is_ok());

    // Each record is from a completely verified admission, but only the exact
    // caller/domain/account/login binding belongs to this proposed session.
    for field in 0..6 {
        let mut other = Fixture::new(1);
        match field {
            0 => other = Fixture::new(2),
            1 => other.claims.audience.fleet_id = CanonicalId::from_bytes([3; 32]),
            2 => other.claims.issuer = Principal::from_slice(&[3]),
            3 => other.claims.account_subject = Principal::from_slice(&[3]),
            4 => other.claims.login_started_at_ns -= 1,
            5 => other.claims.login_deadline_ns += 1,
            _ => unreachable!(),
        }
        let mut host = Receiver::new(&other);
        other.admit(&mut host, NOW).unwrap();
        let change = host.ledger.records().next().unwrap();
        assert_eq!(
            session.check_login_change(&change),
            Err(SessionError::InvalidPolicyOrRecord),
            "binding field {field}"
        );
    }

    receiver.logout(fixture.caller(), NOW).unwrap();
    let tombstone = receiver.ledger.records().next().unwrap();
    assert!(tombstone.revoked());
    assert!(session.check_login_change(&tombstone).is_ok());

    // Proofless logout can remove an old retained session without a reservation.
    let mut unknown = Receiver::new(&fixture);
    unknown.logout(fixture.caller(), NOW).unwrap();
    let tombstone = unknown.ledger.records().next().unwrap();
    assert_eq!(tombstone.binding(), None);
    assert!(session.check_login_change(&tombstone).is_ok());

    for field in 0..2 {
        let mut other = Fixture::new(1);
        if field == 0 {
            other = Fixture::new(2);
        } else {
            other.claims.audience.fleet_id = CanonicalId::from_bytes([3; 32]);
        }
        let mut host = Receiver::new(&other);
        host.logout(other.caller(), NOW).unwrap();
        assert_eq!(
            session.check_login_change(&host.ledger.records().next().unwrap()),
            Err(SessionError::InvalidPolicyOrRecord)
        );
    }
}
