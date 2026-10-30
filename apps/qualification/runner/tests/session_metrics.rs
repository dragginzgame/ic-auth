//! Native outcome semantics, not instruction/cycle measurements.
use ic_auth_qualification_support::session_metrics as metrics;
use sha2::{Digest, Sha256};

use ic_auth::{
    admission_budget::{AdmissionBudgetError, SessionAdmissionBudget},
    portable::{MemoryLoginRevocations, PortableClaimsContext, ValidatedGrants, browser_principal},
    portable_credential::{PortableCredentialError, PortableVerificationContext},
    portable_signature::PortableSignatureError,
    session::*,
    session_request::{SessionRequest, SessionTtlPolicy},
};
use ic_auth_protocol_types::*;
use ic_metrics::{MeasurementCounter, MeasurementCounterError};
use ic_platform::ed25519::{Signer, SigningKey};
use metrics::AdmissionMetrics;

const SECOND: u64 = 1_000_000_000;
const NOW: u64 = 10 * SECOND;

struct Store {
    inner: MemorySessionStore,
    fail_commit: bool,
}
impl SessionStore for Store {
    fn read<T>(
        &mut self,
        operation: impl FnOnce(&dyn SessionReader) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        self.inner.read(operation)
    }
    fn admit(
        &mut self,
        session: Session,
        replay: Replay,
        login: &ic_auth::portable::LoginRevocationChange,
    ) -> Result<bool, SessionError> {
        self.inner.admit_with_commit(session, replay, login, |_| {
            if self.fail_commit {
                Err(SessionError::StorageFailure)
            } else {
                Ok(())
            }
        })
    }
    fn logout(
        &mut self,
        login: &ic_auth::portable::LoginRevocationChange,
    ) -> Result<(), SessionError> {
        self.inner.logout(login)
    }
}

struct Host {
    claims: PortableClaims,
    grants: ValidatedGrants,
    key: SigningKey,
    store: Store,
    ledger: MemoryLoginRevocations,
    budget: SessionAdmissionBudget,
    metrics: AdmissionMetrics,
}
impl Host {
    fn new() -> Self {
        let key = SigningKey::from_bytes(&[42; 32]);
        let claims = PortableClaims {
            audience: AudienceId {
                canonical_network_id: CanonicalId::from_bytes([1; 32]),
                fleet_id: CanonicalId::from_bytes([2; 32]),
            },
            mechanism: PortableMechanism::Ed25519,
            issuer: Principal::from_slice(&[1]),
            account_subject: Principal::from_slice(&[2]),
            key_binding_id: Sha256::digest(key.verifying_key().to_bytes()).into(),
            browser_public_key: SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes(),
            login_started_at_ns: NOW - SECOND,
            login_deadline_ns: NOW + 60 * SECOND,
            issued_at_ns: NOW,
            admission_expires_at_ns: NOW + SECOND,
            nonce: [0; 16],
            grants: vec![DelegatedRoleGrant {
                target: "user".parse().unwrap(),
                scopes: vec!["read".into()],
            }],
        };
        Self {
            grants: ValidatedGrants::new(claims.grants.clone()).unwrap(),
            key,
            store: Store {
                inner: MemorySessionStore::new(
                    0,
                    SessionLimits::new(4, 8, 4, 16_384).expect("fixture limits"),
                ),
                fail_commit: false,
            },
            ledger: MemoryLoginRevocations::new(claims.audience, 4, 4, 0, 0).unwrap(),
            claims,
            budget: SessionAdmissionBudget::new(1, SECOND, NOW - SECOND).unwrap(),
            metrics: Default::default(),
        }
    }
    fn credential(&self) -> PortableCredential {
        PortableCredential {
            claims: ic_auth::portable::encode_claims(&self.claims).unwrap(),
            proof: PortableProof::Ed25519 {
                signature: self
                    .key
                    .sign(&ic_auth::portable::claims_hash(&self.claims).unwrap())
                    .to_bytes()
                    .to_vec(),
            },
        }
    }
    fn admit(
        &mut self,
        credential: &PortableCredential,
    ) -> Result<SessionEstablished, SessionError> {
        let context = PortableClaimsContext {
            authority: ic_auth::portable::PortableAuthority {
                audience: &self.claims.audience,
                issuer: self.claims.issuer,
                mechanism: self.claims.mechanism,
                key_binding_id: self.claims.key_binding_id,
                grants: &self.grants,
                caller: browser_principal(&self.claims.browser_public_key),
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
        let result = establish_session(
            &mut self.store,
            &mut self.ledger,
            &mut self.budget,
            credential,
            SessionRequest::new(vec!["read".into()], None).unwrap(),
            &SessionAdmissionContext {
                verification: PortableVerificationContext::Ed25519 {
                    claims: &context,
                    public_key: &ic_auth::portable_signature::PreparedIssuerKey::new(
                        self.key.verifying_key().to_bytes(),
                    )
                    .unwrap(),
                },
                role: &"user".parse().unwrap(),
                ttl: SessionTtlPolicy::new(5, 30).unwrap(),
            },
            SessionPolicy {
                generation: 0,
                enabled: true,
                subject_admissible: true,
            },
        );
        self.metrics.observe(&result);
        result
    }
}

#[test]
fn committed_outcomes_and_retry_fast_path_stay_distinct() {
    let mut host = Host::new();
    let mut credential = host.credential();
    assert!(matches!(
        host.admit(&credential),
        Ok(SessionEstablished::Created(_))
    ));
    // No credit remains. An exact retry still succeeds without inspecting a new
    // signature; diagnostics must not introduce a second verification.
    credential.proof = PortableProof::Ed25519 { signature: vec![] };
    assert!(matches!(
        host.admit(&credential),
        Ok(SessionEstablished::ExactRetry(_))
    ));
    assert_eq!(host.metrics.created.exact_total(), Ok(1));
    assert_eq!(host.metrics.exact_retry.exact_total(), Ok(1));
    assert_eq!(host.metrics.proof_refused.exact_total(), Ok(0));
    host.claims.nonce[0] += 1;
    assert!(host.admit(&host.credential()).is_err());
    assert_eq!(host.metrics.budget_refused.exact_total(), Ok(1));

    let mut host = Host::new();
    host.budget = SessionAdmissionBudget::new(2, 1, 0).unwrap();
    host.admit(&host.credential()).unwrap();
    host.claims.nonce[0] += 1;
    assert!(matches!(
        host.admit(&host.credential()),
        Ok(SessionEstablished::Replaced(_))
    ));
    assert_eq!(host.metrics.created.exact_total(), Ok(1));
    assert_eq!(host.metrics.replaced.exact_total(), Ok(1));
}

#[test]
fn proof_and_store_failures_keep_charges_and_never_count_a_commit() {
    let mut host = Host::new();
    let mut credential = host.credential();
    credential.proof = PortableProof::Ed25519 {
        signature: vec![0; 64],
    };
    assert!(matches!(
        host.admit(&credential),
        Err(SessionError::Credential(_))
    ));
    assert_eq!(host.metrics.proof_refused.exact_total(), Ok(1));
    assert_eq!(host.store.inner.encoded_bytes(), 0);
    assert_eq!(host.ledger.occupancy(), (0, 0));
    host.admit(&host.credential()).unwrap_err();
    assert_eq!(host.metrics.budget_refused.exact_total(), Ok(1));

    let mut host = Host::new();
    host.store.fail_commit = true;
    assert_eq!(
        host.admit(&host.credential()),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(host.metrics.created.exact_total(), Ok(0));
    assert_eq!(host.metrics.other_refused.exact_total(), Ok(1));
    assert_eq!(host.store.inner.encoded_bytes(), 0);
    assert_eq!(host.ledger.occupancy(), (0, 0));
    host.store.fail_commit = false;
    host.admit(&host.credential()).unwrap_err();
    assert_eq!(host.metrics.budget_refused.exact_total(), Ok(1));

    let mut host = Host::new();
    host.budget = SessionAdmissionBudget::new(2, 1, 0).unwrap();
    let original = host.credential();
    host.admit(&original).unwrap();
    host.claims.nonce[0] += 1;
    host.store.fail_commit = true;
    assert_eq!(
        host.admit(&host.credential()),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(host.metrics.replaced.exact_total(), Ok(0));
    assert_eq!(host.metrics.created.exact_total(), Ok(1));
    assert!(matches!(
        host.admit(&original),
        Ok(SessionEstablished::ExactRetry(_))
    ));
}

#[test]
fn diagnostic_reset_preserves_budget_replay_generation_and_revocation() {
    let mut host = Host::new();
    let credential = host.credential();
    host.admit(&credential).unwrap();
    host.metrics = AdmissionMetrics::default();
    assert!(matches!(
        host.admit(&credential),
        Ok(SessionEstablished::ExactRetry(_))
    ));
    host.claims.nonce[0] += 1;
    host.admit(&host.credential()).unwrap_err();
    assert_eq!(host.metrics.budget_refused.exact_total(), Ok(1));
    let caller = browser_principal(&host.claims.browser_public_key);
    let mut logout_budget = SessionAdmissionBudget::new(1, 1, 0).unwrap();
    assert!(matches!(
        logout_session(
            &mut host.store,
            &mut host.ledger,
            &mut logout_budget,
            caller,
            NOW
        ),
        LogoutResult::Revoked { .. }
    ));
    let occupancy = host.ledger.occupancy();
    let observed = host.ledger.observed_at_ns();
    host.metrics = AdmissionMetrics::default();
    assert!(host.admit(&credential).is_err());
    assert_eq!(
        host.store.read(|r| r.occupancy(caller)).unwrap(),
        SessionOccupancy {
            sessions: 0,
            replays: 1,
            replays_for_caller: 1,
        }
    );
    assert_eq!(host.store.read(|r| Ok(r.generation())).unwrap(), 0);
    assert_eq!(host.ledger.occupancy(), occupancy);
    assert_eq!(host.ledger.observed_at_ns(), observed);
    assert!(host.ledger.is_revoked(caller, NOW).unwrap());
    let mut host = Host::new();
    host.store.inner.advance_generation().unwrap();
    host.metrics = AdmissionMetrics::default();
    assert_eq!(
        host.admit(&host.credential()),
        Err(SessionError::StaleAuthority)
    );
    assert_eq!(host.store.inner.generation(), 1);
}

#[test]
fn both_budget_paths_and_cheap_refusals_are_bounded_categories() {
    let mut metrics = AdmissionMetrics::default();
    for reason in [
        AdmissionBudgetError::Exhausted,
        AdmissionBudgetError::ClockRollback,
        AdmissionBudgetError::InvalidPolicy,
    ] {
        metrics.observe(&Err(SessionError::Credential(
            PortableCredentialError::Ed25519(PortableSignatureError::Budget(reason)),
        )));
        metrics.observe(&Err(SessionError::Credential(
            PortableCredentialError::Certified(
                ic_auth::portable_certified::PortableCertifiedError::Budget(reason),
            ),
        )));
    }
    metrics.observe(&Err(SessionError::Encoding(
        ic_auth::portable::PortableEncodingError::InvalidShape,
    )));
    assert_eq!(metrics.budget_refused.exact_total(), Ok(2));
    assert_eq!(metrics.other_refused.exact_total(), Ok(5));
    assert_eq!(metrics.proof_refused.exact_total(), Ok(0));
}

#[test]
fn diagnostic_projection_preserves_wide_totals_loss_and_failure_isolation() {
    let mut host = Host::new();
    host.metrics.created = MeasurementCounter::from_parts(u128::MAX, false).unwrap();
    let limits = ic_wire::Limits {
        bytes: 4096,
        allocation: 16384,
        nodes: 256,
        depth: 8,
    };
    let bytes = ic_wire::native::encode(&host.metrics, limits).unwrap();
    assert_eq!(
        ic_wire::native::decode::<AdmissionMetrics>(&bytes, limits),
        Ok(host.metrics)
    );
    host.admit(&host.credential()).unwrap();
    assert_eq!(
        host.metrics.created.exact_total(),
        Err(MeasurementCounterError::TotalOverflow)
    );
    let before = host.metrics;
    let bytes = ic_wire::native::encode(&before, limits).unwrap();
    assert_eq!(
        ic_wire::native::decode::<AdmissionMetrics>(&bytes, limits),
        Ok(before)
    );
    // Unavailable bounded reports cannot turn a committed admission into a
    // refusal or alter the host's diagnostic state.
    assert_eq!(
        ic_wire::native::encode(&before, ic_wire::Limits { bytes: 1, ..limits }),
        Err(ic_wire::Error::Limit)
    );
    assert!(
        ic_wire::native::decode::<AdmissionMetrics>(&bytes[..bytes.len() - 1], limits).is_err()
    );
    #[derive(serde::Serialize)]
    struct InvalidCounter {
        total: u128,
        total_overflowed: bool,
    }
    #[derive(serde::Serialize)]
    struct InvalidReport {
        created: InvalidCounter,
        replaced: MeasurementCounter,
        exact_retry: MeasurementCounter,
        proof_refused: MeasurementCounter,
        budget_refused: MeasurementCounter,
        other_refused: MeasurementCounter,
    }
    let invalid = ic_wire::native::encode(
        &InvalidReport {
            created: InvalidCounter {
                total: 1,
                total_overflowed: true,
            },
            replaced: before.replaced,
            exact_retry: before.exact_retry,
            proof_refused: before.proof_refused,
            budget_refused: before.budget_refused,
            other_refused: before.other_refused,
        },
        limits,
    )
    .unwrap();
    assert_eq!(
        ic_wire::native::decode::<AdmissionMetrics>(&invalid, limits),
        Err(ic_wire::Error::Malformed)
    );
    assert_eq!(host.metrics, before);
    let caller = browser_principal(&host.claims.browser_public_key);
    assert_eq!(
        host.store.read(|r| r.occupancy(caller)).unwrap().sessions,
        1
    );
    assert!(matches!(
        host.admit(&host.credential()),
        Ok(SessionEstablished::ExactRetry(_))
    ));
}
