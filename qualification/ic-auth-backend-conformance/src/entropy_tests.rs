//! Reference issuer composition only; fixed seeds are public test data.
use super::*;
use ic_entropy::{Generator, SeedPool};

// The fixture already owns signing. Entropy refusal must precede changing claims
// or calling that issuer; this is not a production nonce/entropy provider API.
fn issue(
    issuer: &mut Fixture,
    generator: Option<&mut Generator>,
) -> Result<PortableCredential, ()> {
    let mut nonce = [0; 16];
    generator
        .ok_or(())?
        .fill_bytes(&mut nonce)
        .map_err(|_| ())?;
    issuer.claims.nonce = nonce;
    Ok(issuer.credential())
}

fn admit(
    issuer: &Fixture,
    credential: &PortableCredential,
    store: &mut MemorySessionStore,
    ledger: &mut MemoryLoginRevocations,
    budget: &mut SessionAdmissionBudget,
) -> Result<SessionEstablished, SessionError> {
    let claims = PortableClaimsContext {
        authority: issuer.authority(NOW),
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: issuer.max_login_ttl_ns,
        max_proof_ttl_ns: 60 * SECOND,
    };
    establish_session(
        store,
        ledger,
        budget,
        credential,
        SessionRequest::new(vec!["read".into()], None).unwrap(),
        &SessionAdmissionContext {
            verification: PortableVerificationContext::Ed25519 {
                claims: &claims,
                public_key: &issuer.prepared,
            },
            role: &issuer.role,
            ttl: SessionTtlPolicy::new(5, 30).unwrap(),
        },
        policy(0),
    )
}

#[test]
fn generated_nonces_preserve_admission_exact_retry_and_replay_refusal() {
    let mut issuer = Fixture::new(1);
    let mut generator = Generator::from_seed([7; 32], 32);
    let first = issue(&mut issuer, Some(&mut generator)).unwrap();
    let first_nonce = issuer.claims.nonce;
    let mut store = MemorySessionStore::new(0, limits(4, 65_536));
    let mut ledger = issuer.ledger();
    let mut budget = SessionAdmissionBudget::new(4, SECOND, 0).unwrap();
    let created = admit(&issuer, &first, &mut store, &mut ledger, &mut budget).unwrap();
    assert!(matches!(created, SessionEstablished::Created(_)));
    assert_eq!(
        admit(&issuer, &first, &mut store, &mut ledger, &mut budget),
        Ok(SessionEstablished::ExactRetry(created.receipt()))
    );
    assert_eq!(generator.remaining_bytes(), 16, "retry draws no new nonce");

    let second = issue(&mut issuer, Some(&mut generator)).unwrap();
    assert_ne!(issuer.claims.nonce, first_nonce);
    assert_ne!(first.claims, second.claims);
    let replacement = admit(&issuer, &second, &mut store, &mut ledger, &mut budget).unwrap();
    assert!(matches!(replacement, SessionEstablished::Replaced(_)));
    assert_eq!(
        admit(&issuer, &first, &mut store, &mut ledger, &mut budget),
        Err(SessionError::ReplayConflict)
    );
    assert_eq!(
        admit(&issuer, &second, &mut store, &mut ledger, &mut budget),
        Ok(SessionEstablished::ExactRetry(replacement.receipt()))
    );
    assert_eq!(store.replay_records().len(), 2);
    assert_eq!(ledger.occupancy(), (1, 0));
    assert!(issuer.authorize(&mut store, &mut ledger, NOW, 0).is_ok());
}

#[test]
fn unavailable_or_short_providers_return_no_credential_or_changed_claims() {
    let mut issuer = Fixture::new(1);
    let before = issuer.claims.clone();
    let mut pool = SeedPool::<1>::new();
    let mut empty = pool.take().map(|seed| Generator::from_seed(seed, 16));
    assert_eq!(issue(&mut issuer, empty.as_mut()), Err(()));
    assert_eq!(issuer.claims, before);
    assert_eq!(issue(&mut issuer, None), Err(()));
    assert_eq!(issuer.claims, before);

    for available in [0, 15] {
        let mut generator = Generator::from_seed([7; 32], available);
        assert_eq!(issue(&mut issuer, Some(&mut generator)), Err(()));
        assert_eq!(issuer.claims, before);
        assert_eq!(generator.remaining_bytes(), available);
    }
}

#[test]
fn exhausted_provider_cannot_return_the_previous_credential_as_new_issuance() {
    let mut issuer = Fixture::new(1);
    let mut generator = Generator::from_seed([7; 32], 16);
    let retained = issue(&mut issuer, Some(&mut generator)).unwrap();
    let before = issuer.claims.clone();
    assert_eq!(issue(&mut issuer, Some(&mut generator)), Err(()));
    assert_eq!(issuer.claims, before);
    // An explicit exact-result lookup can retain this result. An attempted new
    // issuance must report exhaustion instead of implicitly reusing it.
    assert_eq!(retained, issuer.credential());
}

#[test]
fn buffered_seeds_are_single_use_and_restarting_the_old_seed_replays_output() {
    let mut pool = SeedPool::<1>::new();
    pool.insert([7; 32]).unwrap();
    let mut generator = Generator::from_seed(pool.take().unwrap(), 16);
    let mut issuer = Fixture::new(1);
    let first = issue(&mut issuer, Some(&mut generator)).unwrap();
    assert!(pool.take().is_none());
    let mut store = MemorySessionStore::new(0, limits(4, 65_536));
    let mut ledger = issuer.ledger();
    let mut budget = SessionAdmissionBudget::new(4, SECOND, 0).unwrap();
    admit(&issuer, &first, &mut store, &mut ledger, &mut budget).unwrap();

    // Public deterministic test seeds establish stream identity, not entropy
    // quality or fresh-after-commitment unpredictability.
    pool.insert([8; 32]).unwrap();
    generator.reseed(pool.take().unwrap(), 16);
    let second = issue(&mut issuer, Some(&mut generator)).unwrap();
    assert_ne!(first, second);
    admit(&issuer, &second, &mut store, &mut ledger, &mut budget).unwrap();

    generator = Generator::from_seed([7; 32], 16);
    let replayed = issue(&mut issuer, Some(&mut generator)).unwrap();
    assert_eq!(
        replayed, first,
        "restoring a seed does not create freshness"
    );
    assert_eq!(
        admit(&issuer, &replayed, &mut store, &mut ledger, &mut budget),
        Err(SessionError::ReplayConflict)
    );
    // Replay tracks the complete claims, not a nonce whitelist. A new signed
    // timestamp can hide stream reuse from replay detection: lifecycle must
    // prevent it, even when the receiver retained every old replay record.
    issuer.claims.issued_at_ns -= 1;
    let changed_claims = issuer.credential();
    assert_ne!(changed_claims.claims, replayed.claims);
    assert!(matches!(
        admit(
            &issuer,
            &changed_claims,
            &mut store,
            &mut ledger,
            &mut budget
        ),
        Ok(SessionEstablished::Replaced(_))
    ));
}

#[test]
fn all_zero_nonce_is_valid_signed_material_not_an_unseeded_provider() {
    let mut issuer = Fixture::new(1);
    issuer.claims.nonce = [0; 16];
    let credential = issuer.credential();
    let mut store = MemorySessionStore::new(0, limits(4, 65_536));
    let mut ledger = issuer.ledger();
    let mut budget = SessionAdmissionBudget::new(1, SECOND, NOW - SECOND).unwrap();
    assert!(matches!(
        admit(&issuer, &credential, &mut store, &mut ledger, &mut budget),
        Ok(SessionEstablished::Created(_))
    ));
}
