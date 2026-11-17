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
use std::panic::{AssertUnwindSafe, catch_unwind};

// Same independently signed public-API driver as the integration tests.
// Included in this test module so injection controls remain entirely private.
mod fixture {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/receiver_memory/memory_fixture.rs"
    ));
}
use fixture::*;

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
            assert_eq!(r.known_logins(), 1);
            assert_eq!(r.usage(account()).encoded_bytes, 0);
        },
    );
}
