#![cfg(all(feature = "persistent", feature = "portable-signature-verification"))]
mod support;
use ic_auth::{
    persistent::{
        StoreError,
        receiver::{BudgetPolicy, LoginLimits, ReceiverConfig, ReceiverStore},
    },
    portable::{LoginRevocationError, ValidatedGrants},
    portable_credential::PortableVerificationContext,
    portable_signature::PreparedIssuerKey,
    session::{
        RequiredScopes, SessionAdmissionContext, SessionContext, SessionError, SessionEstablished,
        SessionLimits, SessionPolicy,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
};
use ic_auth_protocol_types::*;
use ic_platform::ed25519::SigningKey;
use ic_storage::{
    StableKey, StorageConfig, VectorMemory,
    testing::{MeteredMemory, fixture_runtime},
};

type Backing = MeteredMemory<VectorMemory>;
const NOW: u64 = 10_000_000_000;
fn memory(backing: Backing) -> ic_auth::persistent::receiver::ReceiverMemory<Backing> {
    let runtime = fixture_runtime(
        backing,
        "auth",
        "auth.",
        &["auth.receiver.v1", "auth.metadata.v1", "auth.callers.v1"],
        StorageConfig::default(),
    )
    .unwrap();
    let open = |key| {
        runtime
            .open_memory(&StableKey::parse(key).unwrap())
            .unwrap()
    };
    ic_auth::persistent::receiver::ReceiverMemory {
        state: ic_auth::persistent::StoreMemory {
            records: open("auth.receiver.v1"),
            metadata: open("auth.metadata.v1"),
        },
        callers: open("auth.callers.v1"),
    }
}
fn configuration(audience: AudienceId, sessions: usize, bytes: usize) -> ReceiverConfig {
    ReceiverConfig::new(
        audience,
        SessionLimits::new(sessions, 64, 32, bytes).unwrap(),
        LoginLimits {
            admitted: 16,
            unknown: 4,
            maximum_ttl_ns: 90 * 86_400_000_000_000,
            devices_per_account: 8,
        },
        BudgetPolicy {
            burst: 8,
            refill_interval_ns: 1,
        },
        BudgetPolicy {
            burst: 8,
            refill_interval_ns: 1,
        },
    )
    .unwrap()
}
fn policy(generation: u64) -> SessionPolicy {
    SessionPolicy {
        enabled: true,
        subject_admissible: true,
        generation,
    }
}
fn ttl() -> SessionTtlPolicy {
    SessionTtlPolicy::new(30, 60).unwrap()
}
fn claims(device: u8, slot: u8, generation: u64) -> PortableClaims {
    let mut claims = support::claims(
        SigningKey::from_bytes(&[device; 32])
            .verifying_key()
            .to_bytes(),
    );
    claims.device_slot = slot;
    claims.device_generation = generation;
    claims.login_started_at_ns = NOW - 1;
    claims.login_deadline_ns = NOW + 100_000_000_000;
    claims.issued_at_ns = NOW;
    claims.admission_expires_at_ns = NOW + 1_000_000_000;
    claims
}
fn credential(claims: &PortableClaims) -> PortableCredential {
    PortableCredential {
        claims: ic_auth::portable::encode_claims(claims).unwrap(),
        proof: PortableProof::Ed25519 {
            signature: support::signature(claims, &support::issuer_key()).to_vec(),
        },
    }
}
fn admit(
    store: &mut ReceiverStore<Backing>,
    claims: &PortableClaims,
    now: u64,
) -> Result<SessionEstablished, SessionError> {
    let grants = ValidatedGrants::new(claims.grants.clone()).unwrap();
    let context = support::claims_context(support::authority(claims, &grants, now));
    store.admit(
        &credential(claims),
        SessionRequest::new(vec!["read".into()], None).unwrap(),
        &SessionAdmissionContext {
            verification: PortableVerificationContext::Ed25519 {
                claims: &context,
                public_key: &PreparedIssuerKey::new(
                    support::issuer_key().verifying_key().to_bytes(),
                )
                .unwrap(),
            },
            role: &"user".parse().unwrap(),
            ttl: ttl(),
        },
        policy(store.generation()),
    )
}
fn authenticate(
    store: &mut ReceiverStore<Backing>,
    claims: &PortableClaims,
    now: u64,
    generation: u64,
) -> Result<Principal, SessionError> {
    let grants = ValidatedGrants::new(claims.grants.clone()).unwrap();
    store.authenticate(
        &SessionContext {
            authority: support::authority(claims, &grants, now),
            max_ttl: ttl().maximum(),
            role: &"user".parse().unwrap(),
            required_scopes: RequiredScopes::NONE,
        },
        policy(generation),
        |session| session.subject(),
    )
}
#[test]
fn durable_devices_replacement_logout_replay_and_authority_use_the_existing_engine() {
    let first = claims(1, 0, 1);
    let peer = claims(2, 1, 2);
    let next = claims(3, 0, 3);
    let backing = Backing::new(VectorMemory::default());
    let config = configuration(first.audience, 2, 65_536);
    let mut store =
        ReceiverStore::create(memory(backing.clone()), config.clone(), 0, NOW - 100).unwrap();
    assert!(matches!(
        admit(&mut store, &first, NOW).unwrap(),
        SessionEstablished::Created(_)
    ));
    assert!(matches!(
        admit(&mut store, &first, NOW).unwrap(),
        SessionEstablished::ExactRetry(_)
    ));
    admit(&mut store, &peer, NOW).unwrap();
    assert_eq!(store.occupancy().admitted_logins, 2);
    assert!(matches!(
        admit(&mut store, &next, NOW).unwrap(),
        SessionEstablished::Replaced(_)
    ));
    assert_eq!(store.occupancy().sessions, 2);
    assert_eq!(store.occupancy().replays, 3);
    assert!(authenticate(&mut store, &first, NOW, 0).is_err());
    assert_eq!(
        authenticate(&mut store, &next, NOW, 0),
        Ok(next.account_subject)
    );
    assert_eq!(
        authenticate(&mut store, &peer, NOW, 0),
        Ok(peer.account_subject)
    );
    let count = store.occupancy();
    drop(store);
    let mut store = ReceiverStore::open(memory(backing.clone()), &config, NOW).unwrap();
    assert_eq!(store.occupancy(), count);
    assert_eq!(
        authenticate(&mut store, &next, NOW, 0),
        Ok(next.account_subject)
    );
    assert!(matches!(
        store.logout(
            ic_auth::portable::browser_principal(&next.browser_public_key),
            NOW
        ),
        LogoutResult::Revoked { .. }
    ));
    assert!(matches!(
        authenticate(&mut store, &next, NOW, 0),
        Err(SessionError::Revocation(LoginRevocationError::Revoked))
    ));
    assert!(admit(&mut store, &next, NOW).is_err());
    assert_eq!(store.occupancy().replays, 3);
    assert_eq!(
        authenticate(&mut store, &peer, NOW, 0),
        Ok(peer.account_subject)
    );
    store.advance_generation().unwrap();
    assert_eq!(
        authenticate(&mut store, &peer, NOW, 1),
        Err(SessionError::StaleAuthority)
    );
    store.prune(NOW + 200_000_000_000, 32).unwrap();
    assert_eq!(
        (
            store.occupancy().sessions,
            store.occupancy().replays,
            store.occupancy().admitted_logins
        ),
        (0, 0, 2)
    );
    drop(store);
    let store = ReceiverStore::open(memory(backing), &config, NOW + 200_000_000_000).unwrap();
    assert_eq!(
        store.occupancy().admitted_logins,
        2,
        "known slot fences never expire into reusable capacity"
    );
}

#[test]
fn capacity_signature_clock_and_configuration_refusals_preserve_records() {
    let first = claims(1, 0, 1);
    let peer = claims(2, 1, 2);
    let backing = Backing::new(VectorMemory::default());
    let config = configuration(first.audience, 1, 65_536);
    let mut store =
        ReceiverStore::create(memory(backing.clone()), config.clone(), 0, NOW - 100).unwrap();
    admit(&mut store, &first, NOW).unwrap();
    let count = store.occupancy();
    assert_eq!(admit(&mut store, &peer, NOW), Err(SessionError::Capacity));
    assert_eq!(store.occupancy(), count);
    assert!(authenticate(&mut store, &first, NOW - 1, 0).is_err());
    drop(store);
    // Different limits are explicit configuration mismatch, never silent reset.
    let wrong = configuration(first.audience, 2, 65_536);
    assert!(matches!(
        ReceiverStore::open(memory(backing.clone()), &wrong, NOW),
        Err(StoreError::Corrupt)
    ));
    let mut store = ReceiverStore::open(memory(backing), &config, NOW).unwrap();
    assert_eq!(store.occupancy(), count);
    assert_eq!(
        authenticate(&mut store, &first, NOW, 0),
        Ok(first.account_subject)
    );
}

#[test]
fn invalid_signature_and_byte_refusals_leave_no_session_replay_or_login() {
    let first = claims(1, 0, 1);
    let backing = Backing::new(VectorMemory::default());
    let config = configuration(first.audience, 2, 65_536);
    let mut store = ReceiverStore::create(memory(backing), config, 0, NOW - 100).unwrap();
    let mut bad = credential(&first);
    let PortableProof::Ed25519 { signature } = &mut bad.proof else {
        unreachable!()
    };
    signature[0] ^= 1;
    let grants = ValidatedGrants::new(first.grants.clone()).unwrap();
    let context = support::claims_context(support::authority(&first, &grants, NOW));
    let result = store.admit(
        &bad,
        SessionRequest::new(vec!["read".into()], None).unwrap(),
        &SessionAdmissionContext {
            verification: PortableVerificationContext::Ed25519 {
                claims: &context,
                public_key: &PreparedIssuerKey::new(
                    support::issuer_key().verifying_key().to_bytes(),
                )
                .unwrap(),
            },
            role: &"user".parse().unwrap(),
            ttl: ttl(),
        },
        policy(0),
    );
    assert!(result.is_err());
    assert_eq!(store.occupancy(), Default::default());
    admit(&mut store, &first, NOW).unwrap();
    let backing = Backing::new(VectorMemory::default());
    let config = configuration(first.audience, 2, 1);
    let mut store =
        ReceiverStore::create(memory(backing.clone()), config.clone(), 0, NOW - 100).unwrap();
    assert_eq!(
        admit(&mut store, &first, NOW),
        Err(SessionError::ByteCapacity)
    );
    assert_eq!(store.occupancy(), Default::default());
    drop(store);
    assert_eq!(
        ReceiverStore::open(memory(backing), &config, NOW)
            .unwrap()
            .occupancy(),
        Default::default()
    );
}

#[test]
fn inspection_pages_preserve_expired_stale_sessions_and_skip_fences_after_reopen() {
    let backing = Backing::new(VectorMemory::default());
    let mut claims = [claims(1, 0, 1), claims(2, 1, 2), claims(3, 2, 3)];
    let caller =
        |claims: &PortableClaims| ic_auth::portable::browser_principal(&claims.browser_public_key);
    claims.sort_by_key(caller);
    let config = configuration(claims[0].audience, 4, 65_536);
    let mut store =
        ReceiverStore::create(memory(backing.clone()), config.clone(), 0, NOW - 100).unwrap();
    for claims in &claims {
        admit(&mut store, claims, NOW).unwrap();
    }
    backing.reset();
    let page = store.inspect_sessions(None, 1, 1, NOW).unwrap();
    assert_eq!(page.sessions.len(), 1);
    assert_eq!(page.sessions[0].caller(), caller(&claims[0]));
    assert_eq!(page.next_after, Some(caller(&claims[0])));
    assert_eq!(backing.counts().writes, 0);
    store.logout(caller(&claims[1]), NOW);
    store.advance_generation().unwrap();
    drop(store);
    let store = ReceiverStore::open(memory(backing.clone()), &config, NOW).unwrap();
    backing.reset();
    let expired_at = NOW + 61_000_000_000;
    let empty = store
        .inspect_sessions(page.next_after, 1, 1, expired_at)
        .unwrap();
    assert!(empty.sessions.is_empty());
    assert_eq!(empty.next_after, Some(caller(&claims[1])));
    let last = store
        .inspect_sessions(empty.next_after, 1, 1, expired_at)
        .unwrap();
    assert_eq!(last.sessions.len(), 1);
    assert_eq!(last.sessions[0].caller(), caller(&claims[2]));
    assert!(last.sessions[0].expires_at_ns() <= last.observed_at_ns);
    assert_ne!(last.sessions[0].generation(), last.generation);
    assert_eq!(last.next_after, None);
    assert_eq!(store.observed_at_ns(), NOW);
    assert_eq!((backing.counts().writes, backing.counts().grows), (0, 0));
}
