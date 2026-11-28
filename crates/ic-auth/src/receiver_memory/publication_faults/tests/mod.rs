mod memory_fixture;

use super::REMAINING;
use crate::{
    receiver_memory::{
        AdmissionHost, EnrollmentBinding, EnrollmentMetadata, MemoryConfig, MemoryError as E,
        MemoryReceiver,
    },
    receiver_policy::{
        LoginLimits, LogoutError, SessionAuthority, SessionCapacity, UnknownLogoutLimits,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
    verification::{
        AuthorityGrants, Ed25519Key, VerificationContext, VerificationCosts, VerificationProfile,
    },
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, CanonicalId, Credential, CredentialProof, DelegatedRoleGrant,
    PortableMechanism, Principal,
};
use memory_fixture::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

type FixtureReceiver = MemoryReceiver;
fn fresh_receiver(config: MemoryConfig, now: u64) -> Result<FixtureReceiver, E> {
    MemoryReceiver::new_fresh(config, now)
}

///
/// Armed
///
/// Disarms the test interruption hook when the guard goes out of scope.
///

struct Armed;

impl Armed {
    fn after(steps: usize) -> Self {
        REMAINING.with(|remaining| {
            assert!(remaining.replace(Some(steps)).is_none());
        });
        Self
    }
}

impl Drop for Armed {
    fn drop(&mut self) {
        REMAINING.with(|remaining| remaining.set(None));
    }
}

fn exercise(
    name: &str,
    build: impl Fn(&Host) -> MemoryReceiver,
    operation: impl Fn(&Host, &mut MemoryReceiver),
) -> usize {
    let host = Host::new();
    for stage in 0..128 {
        let mut receiver = build(&host);
        let result = {
            let _armed = Armed::after(stage);
            catch_unwind(AssertUnwindSafe(|| operation(&host, &mut receiver)))
        };
        if let Err(payload) = result {
            assert_eq!(
                payload.downcast_ref::<&str>(),
                Some(&"auth-publication-fault"),
                "unexpected panic in {name} at stage {stage}"
            );
            // Every entrypoint that can consume or mutate identity/control
            // state must refuse after interrupted publication. No reset API.
            let context = host.context("initial", 125);
            assert_eq!(
                receiver.admit(&credential("initial"), request(), host.admission(&context)),
                Err(E::Closed)
            );
            assert_eq!(
                receiver.read(&host.read("initial", 125), &[]),
                Err(E::Closed)
            );
            assert_eq!(
                receiver.authorize(&host.read("initial", 125), &[], |_| {
                    panic!("projection invoked on poisoned receiver")
                }),
                Err(E::Closed)
            );
            assert_eq!(
                receiver.logout(context.caller, 125),
                Err(LogoutError::Closed)
            );
            assert_eq!(receiver.prune(125, 128), Err(E::Closed));
            assert_eq!(receiver.advance_generation(125), Err(E::Closed));
            assert_eq!(
                receiver.request_enrollment(125, [9; 32], 135),
                Err(E::Closed)
            );
            assert_eq!(
                receiver.install_enrollment(
                    125,
                    host.config().enrollment.authority,
                    host.snapshot(9)
                ),
                Err(E::Closed)
            );
            assert!(matches!(
                receiver.inspect_sessions(None, 32, 128, 125),
                Err(E::Closed)
            ));
        } else {
            assert!(stage > 0, "{name} exercised no publication boundary");
            // All earlier indexed boundaries failed; this is the first full
            // execution, so the normal path must have restored readiness.
            assert!(receiver.inspect_sessions(None, 32, 128, 125).is_ok());
            return stage;
        }
    }
    panic!("unbounded publication trace: {name}")
}

fn admit(host: &Host, receiver: &mut MemoryReceiver, name: &str, now: u64) {
    let context = host.context(name, now);
    receiver
        .admit(&credential(name), request(), host.admission(&context))
        .unwrap();
}

fn populated(host: &Host) -> MemoryReceiver {
    let mut receiver = host.receiver();
    admit(host, &mut receiver, "initial", 25);
    receiver
}

#[test]
fn publication_faults_close_every_admission_and_logout_path() {
    let mut stages = 0;
    stages += exercise("fresh admission", Host::receiver, |h, r| {
        admit(h, r, "initial", 25)
    });
    stages += exercise("renewal", populated, |h, r| admit(h, r, "renewal", 25));
    stages += exercise("replacement", populated, |h, r| {
        admit(h, r, "replacement", 25)
    });
    stages += exercise(
        "replacement after logout",
        |h| {
            let mut r = populated(h);
            r.logout(h.context("initial", 25).caller, 25).unwrap();
            r
        },
        |h, r| admit(h, r, "replacement", 25),
    );
    stages += exercise(
        "unknown-to-known transfer",
        |h| {
            let mut r = h.receiver();
            r.logout(h.context("future", 25).caller, 25).unwrap();
            r
        },
        |h, r| admit(h, r, "future", 125),
    );
    stages += exercise("known logout", populated, |h, r| {
        r.logout(h.context("initial", 25).caller, 25).unwrap();
    });
    stages += exercise("unknown logout", Host::receiver, |h, r| {
        r.logout(h.context("initial", 25).caller, 25).unwrap();
    });
    stages += exercise(
        "expired unknown logout",
        |h| {
            let mut r = h.receiver();
            r.logout(h.context("initial", 25).caller, 25).unwrap();
            r
        },
        |h, r| {
            r.logout(h.context("initial", 125).caller, 125).unwrap();
        },
    );
    assert!(stages >= 7); // every selected path reached at least one real publication point
}

#[test]
fn publication_faults_close_pruning_of_all_retained_record_kinds() {
    exercise(
        "combined prune",
        |h| {
            let mut r = populated(h);
            admit(h, &mut r, "renewal", 25);
            r.logout(h.context("replacement", 25).caller, 25).unwrap();
            r
        },
        |_, r| {
            let report = r.prune(125, 128).unwrap();
            assert_eq!(
                (report.sessions, report.replays, report.unknown_denials),
                (1, 2, 1)
            );
            assert_eq!(report.device_slots, 1);
            assert_eq!(r.known_logins(), 0);
            assert_eq!(r.usage(account()).encoded_bytes, 0);
        },
    );
}

#[cfg(feature = "receiver-stable")]
#[test]
fn reopen_rejects_resealed_session_size_and_application_binding_changes() {
    use crate::{receiver_memory::StableReceiver, stable_upgrade::UpgradeError};
    use ic_storage::{StableKey, VectorMemory, adapter::StableCell, testing::fixture_runtime};
    use sha2::{Digest, Sha256};
    let h = Host::new();
    for change in ["size", "drop", "same-length"] {
        let key = "auth.receiver.v1";
        let store = fixture_runtime(
            VectorMemory::default(),
            "auth",
            "auth.",
            &[key],
            Default::default(),
        )
        .unwrap();
        let key = StableKey::parse(key).unwrap();
        let lease = || store.open_memory(&key).unwrap();
        let mut stable = StableReceiver::new_fresh(lease(), h.config(), 0).unwrap();
        h.enroll(stable.state(), 7).unwrap();
        admit(&h, stable.state(), "registered", 25);
        let caller = h.context("registered", 25).caller;
        let r = stable.state();
        let s = r
            .callers
            .get_mut(&caller)
            .unwrap()
            .session
            .as_mut()
            .unwrap();
        match change {
            "size" => {
                s.bytes += 1;
                r.encoded_bytes += 1;
            }
            "drop" => s.ext = None,
            _ => {}
        }
        stable.seal(5).unwrap();
        drop(stable);
        if change == "same-length" {
            // Model an encoder attaching different bytes while keeping the length
            // and the outer version-bound digest internally consistent.
            let mut cell = StableCell::<Vec<u8>, _>::load(lease()).unwrap();
            let mut image = cell.get().clone();
            let (digest, payload) = image.split_at_mut(32);
            let offset = payload.windows(16).position(|w| w == [0xab; 16]).unwrap();
            payload[offset] ^= 1;
            digest.copy_from_slice(
                &Sha256::new()
                    .chain_update(b"ic-auth.checkpoint.receiver.sparse.v1")
                    .chain_update(6u64.to_le_bytes())
                    .chain_update(payload)
                    .finalize(),
            );
            cell.set(image);
        }
        assert!(
            matches!(
                StableReceiver::reopen(lease(), h.config(), 26, 6),
                Err(UpgradeError::InvalidCheckpoint)
            ),
            "{change}"
        );
    }
}

#[test]
fn starting_profile_round_trips_full_session_replay_and_device_populations() {
    use crate::{
        claims::decode_claims,
        receiver_memory::{CallerRecord, LoginBinding, Replay, Session, StableReceiver},
    };
    use ic_storage::{
        StableKey, VectorMemory, stable_cell_payload_length, testing::fixture_runtime,
    };

    let host = Host::new();
    let config = MemoryConfig::starting_profile(host.audience, host.config().enrollment);
    let backing = VectorMemory::default();
    let name = "auth.capacity.v1";
    let runtime = fixture_runtime(backing, "auth", "auth.", &[name], Default::default()).unwrap();
    let key = StableKey::parse(name).unwrap();
    let lease = || runtime.open_memory(&key).unwrap();
    let mut receiver = StableReceiver::new_fresh(lease(), config, 25).unwrap();
    host.enroll(&mut receiver, 7).unwrap();

    // Populate checkpoint records directly so the test measures checkpoint
    // capacity independently of signature throughput and work-credit refill.
    let mut claims = decode_claims(&credential("registered").claims).unwrap();
    claims.ext = Some(vec![0xab; 32]);
    let mut authority = host.read("registered", 25);
    authority.issuer = Principal::from_slice(&[0x42; 29]);
    let request = request();
    let numbered = |index: usize, tag: u8| {
        let mut bytes = [tag; 29];
        bytes[..4].copy_from_slice(&u32::try_from(index).unwrap().to_be_bytes());
        Principal::from_slice(&bytes)
    };
    for index in 0..config.login.max_retained_device_slots {
        let caller = numbered(index, 0x02);
        authority.caller = caller;
        // Exercise one full eight-device account among the sparse dormant accounts.
        let dense_start = config.login.max_retained_device_slots - 8;
        let subject = numbered(index.min(dense_start), 0x03);
        let device_slot = u8::try_from(index.saturating_sub(dense_start)).unwrap();
        let binding = LoginBinding {
            issuer: authority.issuer,
            subject,
            device_slot,
            device_generation: 1,
            started_at_ns: 10,
            deadline_ns: 100,
        };
        let mut fingerprint = [0; 32];
        fingerprint[..4].copy_from_slice(&u32::try_from(index).unwrap().to_be_bytes());
        let session =
            Session::prepare(&claims, request.clone(), &authority, 85, fingerprint).unwrap();
        let mut row = CallerRecord::new(caller, config.audience, binding, 100, session);
        if index < config.capacity.max_sessions {
            receiver.encoded_bytes += row.session.as_ref().unwrap().bytes;
            receiver.sessions += 1;
            receiver.session_expiry.insert((85, caller));
            for sequence in 0..4 {
                fingerprint[4] = sequence;
                let replay =
                    Replay::prepare(&fingerprint, (authority.issuer, subject), 30).unwrap();
                receiver.encoded_bytes += replay.bytes;
                receiver.replays.insert(fingerprint, replay);
                receiver.replay_expiry.insert((30, fingerprint));
            }
            receiver
                .accounts
                .entry((authority.issuer, subject))
                .or_default()
                .replay_count = 4;
        } else {
            row.session = None;
        }
        receiver
            .accounts
            .entry((authority.issuer, subject))
            .or_default()
            .set_slot(device_slot, caller);
        receiver.callers.insert(caller, row);
    }
    assert_eq!(receiver.replays.len(), config.capacity.max_replays);
    assert!(receiver.encoded_bytes <= config.capacity.max_encoded_bytes);
    receiver.seal(1).unwrap();
    drop(receiver);
    let encoded = stable_cell_payload_length(&lease(), 128 * 1024 * 1024).unwrap();
    let mut reopened = StableReceiver::reopen(lease(), config, 26, 2).unwrap();
    assert_eq!(
        reopened.known_logins(),
        config.login.max_retained_device_slots
    );
    assert_eq!(reopened.sessions, config.capacity.max_sessions);
    assert_eq!(reopened.replays.len(), config.capacity.max_replays);
    assert_eq!(reopened.session_expiry.len(), config.capacity.max_sessions);
    assert_eq!(reopened.replay_expiry.len(), config.capacity.max_replays);
    assert_eq!(
        reopened.callers[&numbered(0, 0x02)]
            .session
            .as_ref()
            .unwrap()
            .ext
            .as_deref(),
        Some([0xab; 32].as_slice()),
    );
    let dense_start = config.login.max_retained_device_slots - 8;
    for slot in 0..8 {
        assert_eq!(
            reopened.slot_owner((authority.issuer, numbered(dense_start, 0x03), slot)),
            Some(&numbered(dense_start + usize::from(slot), 0x02))
        );
    }
    reopened.request_enrollment(26, [2; 32], 50).unwrap();
    let mut snapshot = host.snapshot(7);
    snapshot.revision = 2;
    snapshot.challenge = [2; 32];
    reopened
        .install_enrollment(26, config.enrollment.authority, snapshot)
        .unwrap();
    assert_eq!(reopened.prune(30, 128).unwrap().replays, 128);
    assert_eq!(reopened.sessions, config.capacity.max_sessions);
    println!("receiver checkpoint payload bytes={encoded}");
}
