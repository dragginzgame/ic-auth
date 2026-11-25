//! Small independent ledger for the fixed signed, single-slot fixture scenario.
//! No Auth policy/encoding/verification helper supplies expected decisions.
//! Limits deliberately avoid refill saturation: credit is elapsed time minus
//! charged operations, with one work unit per operation and one per nanosecond.
use super::*;

const NAMES: [&str; 4] = ["initial", "renewal", "replacement", "third"];
const WHO: [usize; 4] = [0, 0, 1, 2];
const DEVICE: [u64; 4] = [1, 1, 2, 3];
const PROOF_END: [u64; 4] = [30, 31, 32, 33];

///
/// Op
///
/// One receiver operation applied to the implementation and reference model.
///

#[derive(Clone, Copy, Debug)]
enum Op {
    Admit {
        fixture: usize,
        changed: bool,
        valid: bool,
    },
    Logout(usize),
    Prune,
    AuthorityChange,
    Wait(u64),
}
const INITIAL: Op = Op::Admit {
    fixture: 0,
    changed: false,
    valid: true,
};
const BAD: Op = Op::Admit {
    fixture: 0,
    changed: false,
    valid: false,
};
const RENEW: Op = Op::Admit {
    fixture: 1,
    changed: false,
    valid: true,
};
const REPLACE: Op = Op::Admit {
    fixture: 2,
    changed: false,
    valid: true,
};
const THIRD: Op = Op::Admit {
    fixture: 3,
    changed: false,
    valid: true,
};
const CONFLICT: Op = Op::Admit {
    fixture: 0,
    changed: true,
    valid: true,
};
const OPS: [Op; 12] = [
    INITIAL,
    BAD,
    RENEW,
    REPLACE,
    THIRD,
    CONFLICT,
    Op::Logout(0),
    Op::Logout(1),
    Op::Logout(2),
    Op::Prune,
    Op::AuthorityChange,
    Op::Wait(1),
];

///
/// Answer
///
/// Observable result compared between the receiver and reference model.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Answer {
    New,
    Retry,
    Refused,
    Revoked,
    Unconfirmed(u64),
    Done,
}

///
/// Lease
///
/// Session metadata retained by the reference model.
///

#[derive(Clone, Copy, Debug)]
struct Lease {
    fixture: usize,
    changed: bool,
    epoch: u64,
    established: u64,
}

///
/// Known
///
/// Known device and denial state in the reference model.
///

#[derive(Clone, Copy, Debug)]
struct Known {
    who: usize,
    device: u64,
    denied: bool,
}

///
/// Ledger
///
/// Independent model of leases, replay, denial and budget accounting.
///

#[derive(Debug)]
struct Ledger {
    known: Option<Known>,
    lease: Option<Lease>,
    consumed: [bool; 4],
    unknown: [Option<u64>; 3],
    epoch: u64,
    proof_charges: u64,
    proof_observation: u64,
    logout_charges: u64,
    logout_observation: u64,
}

impl Ledger {
    fn new() -> Self {
        Self {
            known: None,
            lease: None,
            consumed: [false; 4],
            unknown: [None; 3],
            epoch: 7,
            proof_charges: 0,
            proof_observation: 0,
            logout_charges: 0,
            logout_observation: 0,
        }
    }

    fn denied(&self, who: usize, now: u64) -> bool {
        self.unknown[who].is_some_and(|end| now < end)
            || self.known.is_some_and(|k| k.who == who && k.denied)
    }

    fn authorized(&self, who: usize, now: u64) -> bool {
        self.lease
            .is_some_and(|s| WHO[s.fixture] == who && s.epoch == self.epoch && now < 100)
            && !self.denied(who, now)
    }

    fn apply(&mut self, op: Op, now: u64) -> Answer {
        match op {
            Op::Admit {
                fixture,
                changed,
                valid,
            } => {
                let who = WHO[fixture];
                if let Some(old) = self.lease.filter(|s| s.fixture == fixture) {
                    return if old.changed == changed && self.authorized(who, now) {
                        Answer::Retry
                    } else {
                        Answer::Refused
                    };
                }
                // Declarative admission conditions for these independently signed
                // fixtures: all have one account/slot, start 10, absolute end 100.
                let device_allowed = self.known.is_none_or(|k| {
                    (k.who == who && k.device == DEVICE[fixture] && !k.denied)
                        || (k.who != who && DEVICE[fixture] > k.device)
                });
                let eligible = now < PROOF_END[fixture]
                    && !self.denied(who, now)
                    && !self.consumed[fixture]
                    && self.consumed.iter().filter(|b| **b).count() < 2
                    && device_allowed
                    && !(self
                        .known
                        .is_some_and(|k| k.who != who && k.denied && now < 100)
                        && self.unknown[who].is_none()
                        && self.unknown.iter().flatten().count() == 2);
                if !eligible {
                    return Answer::Refused;
                }
                self.proof_observation = now;
                if self.proof_charges == now {
                    return Answer::Refused;
                }
                self.proof_charges += 1;
                if !valid {
                    return Answer::Refused;
                }
                if let Some(k) = self.known.filter(|k| k.who != who && k.denied && now < 100) {
                    self.unknown[k.who] = Some(100);
                }
                self.unknown[who] = None;
                self.known = Some(Known {
                    who,
                    device: DEVICE[fixture],
                    denied: false,
                });
                self.lease = Some(Lease {
                    fixture,
                    changed,
                    epoch: self.epoch,
                    established: now,
                });
                self.consumed[fixture] = true;
                Answer::New
            }
            Op::Logout(who) => {
                if let Some(k) = self.known.as_mut().filter(|k| k.who == who) {
                    k.denied = true;
                    self.lease = None;
                    return Answer::Revoked;
                }
                if let Some(end) = self.unknown[who].filter(|end| now < *end) {
                    return Answer::Unconfirmed(end);
                }
                if self.unknown[who].is_none() && self.unknown.iter().flatten().count() == 2 {
                    return Answer::Refused;
                }
                self.logout_observation = now;
                if self.logout_charges == now {
                    return Answer::Refused;
                }
                self.logout_charges += 1;
                self.unknown[who] = Some(now + 100);
                Answer::Unconfirmed(now + 100)
            }
            Op::Prune => {
                if now >= 100 {
                    self.lease = None;
                }
                for (i, used) in self.consumed.iter_mut().enumerate() {
                    if now >= PROOF_END[i] {
                        *used = false;
                    }
                }
                for end in &mut self.unknown {
                    if end.is_some_and(|end| now >= end) {
                        *end = None;
                    }
                }
                Answer::Done
            }
            Op::AuthorityChange => {
                self.epoch += 1;
                Answer::Done
            }
            Op::Wait(_) => Answer::Done,
        }
    }
}

///
/// Driver
///
/// Applies the same operations to both receivers and their reference ledgers.
///

struct Driver {
    host: Host,
    receivers: [MemoryReceiver; 2],
    ledgers: [Ledger; 2],
    now: u64,
    answers: [usize; 6],
}

impl Driver {
    fn new() -> Self {
        let host = Host::new();
        let mut config = host.config();
        config.verification_capacity = 1_000;
        config.verification_costs = VerificationCosts {
            ed25519: 1,
            certified: 1,
        };
        config.unknown_logout.capacity = 1_000;
        let receivers = std::array::from_fn(|_| {
            let mut r = MemoryReceiver::new_fresh(config, 0).unwrap();
            host.enroll(&mut r, 7).unwrap();
            r
        });
        Self {
            host,
            receivers,
            ledgers: std::array::from_fn(|_| Ledger::new()),
            now: 25,
            answers: [0; 6],
        }
    }

    fn step(&mut self, target: usize, op: Op) {
        if let Op::Wait(delta) = op {
            self.now = (self.now + delta).min(125);
        }
        let expected = self.ledgers[target].apply(op, self.now);
        let epoch = self.ledgers[target].epoch;
        let r = &mut self.receivers[target];
        let actual = match op {
            Op::Admit {
                fixture,
                changed,
                valid,
            } => {
                let mut credential = credential(NAMES[fixture]);
                if !valid {
                    let CredentialProof::Ed25519(signature) = &mut credential.proof else {
                        unreachable!()
                    };
                    signature[0] ^= 1; // same independent bad-R control as the focused fixture test
                }
                let context = self.host.context(NAMES[fixture], self.now);
                let mut host = self.host.admission(&context);
                host.generation = epoch;
                let request =
                    SessionRequest::new(vec!["read".into()], changed.then_some(60)).unwrap();
                match r.admit(&credential, request, host) {
                    Ok(result) => {
                        assert_eq!(result.receipt.expires_at_ns, 100);
                        assert_eq!(result.receipt.caller, context.caller);
                        assert_eq!((result.receipt.issuer, result.receipt.subject), account());
                        if result.exact_retry {
                            Answer::Retry
                        } else {
                            Answer::New
                        }
                    }
                    Err(_) => Answer::Refused,
                }
            }
            Op::Logout(who) => {
                let caller = self.host.context(NAMES[[0, 2, 3][who]], self.now).caller;
                match r.logout(caller, self.now) {
                    Ok(LogoutReceipt::Revoked { deny_until_ns }) => {
                        assert_eq!(deny_until_ns, 100);
                        Answer::Revoked
                    }
                    Ok(LogoutReceipt::Unconfirmed { deny_until_ns }) => {
                        Answer::Unconfirmed(deny_until_ns)
                    }
                    Err(_) => Answer::Refused,
                }
            }
            Op::Prune => {
                r.prune(self.now, 128).unwrap();
                Answer::Done
            }
            Op::AuthorityChange => {
                let revision = epoch - 6;
                let challenge = [revision as u8; 32];
                r.request_enrollment(self.now, challenge, self.now + 10)
                    .unwrap();
                r.install_enrollment(
                    self.now,
                    self.host.config().enrollment.authority,
                    EnrollmentMetadata {
                        revision,
                        challenge,
                        snapshot_digest: [revision as u8; 32],
                        authority_digest: [epoch as u8; 32],
                        issued_at_ns: self.now,
                        ..self.host.snapshot(epoch)
                    },
                )
                .unwrap();
                Answer::Done
            }
            Op::Wait(_) => Answer::Done,
        };
        assert_eq!(
            actual, expected,
            "target={target}, now={}, op={op:?}, model={:?}",
            self.now, self.ledgers[target]
        );
        self.answers[match expected {
            Answer::New => 0,
            Answer::Retry => 1,
            Answer::Refused => 2,
            Answer::Revoked => 3,
            Answer::Unconfirmed(_) => 4,
            Answer::Done => 5,
        }] += 1;
        // Check both receivers after every operation, including the untouched one.
        // Read checks below are ordinary host operations at the current time;
        // they may observe the clock but must not charge or alter identity state.
        for i in 0..2 {
            self.check(i);
        }
    }

    fn check(&mut self, target: usize) {
        let m = &self.ledgers[target];
        let r = &mut self.receivers[target];
        let used = m.consumed.iter().filter(|b| **b).count();
        let usage = r.usage(account());
        assert_eq!(
            (usage.sessions, usage.replays, usage.replays_for_account),
            (usize::from(m.lease.is_some()), used, used),
            "model receiver occupancy mismatch"
        );
        // Independent Postcard field-size sum from the retained reference;
        // all fixture generations/times here remain single-byte varints.
        assert_eq!(
            usage.encoded_bytes,
            226 * usize::from(m.lease.is_some()) + 79 * used
        );
        assert_eq!(r.known_logins(), usize::from(m.known.is_some()));
        assert_eq!(r.unknown_logins(), m.unknown.iter().flatten().count());
        assert_eq!(
            r.verification_budget().observed_at_ns(),
            m.proof_observation
        );
        assert_eq!(
            u64::from(r.verification_budget().available()),
            m.proof_observation - m.proof_charges
        );
        assert_eq!(
            u64::from(r.unknown_logout_credit()),
            m.logout_observation - m.logout_charges
        );
        let page = r.inspect_sessions(None, 32, 128, self.now).unwrap();
        assert_eq!(page.generation, m.epoch);
        assert_eq!(page.sessions.len(), usize::from(m.lease.is_some()));
        if let Some(s) = m.lease {
            let facts = &page.sessions[0];
            assert_eq!(facts.authority_generation, s.epoch);
            assert_eq!(facts.login.device_generation, DEVICE[s.fixture]);
            assert_eq!(
                (facts.established_at_ns, facts.expires_at_ns),
                (s.established, 100)
            );
        }
        for (who, fixture) in [0, 2, 3].into_iter().enumerate() {
            let mut authority = self.host.read(NAMES[fixture], self.now);
            authority.generation = m.epoch;
            assert_eq!(
                r.read(&authority, &[]).is_ok(),
                m.authorized(who, self.now),
                "target={target}, who={who}, now={}, model={m:?}",
                self.now
            );
        }
    }
}

#[test]
fn independent_model_covers_generated_receiver_sequences() {
    let mut coverage = [0; 6];
    // Every ordered pair, both starting receiver choices, then a fixed tail
    // crossing proof expiry, pruning, the absolute login deadline and denial expiry.
    for first in OPS {
        for second in OPS {
            for target in 0..2 {
                let mut d = Driver::new();
                for (receiver, op) in [
                    (target, first),
                    (target, second),
                    (1 - target, INITIAL),
                    (target, Op::Wait(6)),
                    (target, Op::Prune),
                    (target, REPLACE),
                    (target, Op::Wait(69)),
                    (target, Op::Prune),
                    (target, Op::Logout(0)),
                    (target, Op::Wait(25)),
                    (target, Op::Prune),
                ] {
                    d.step(receiver, op);
                }
                for (count, observed) in coverage.iter_mut().zip(d.answers) {
                    *count += observed;
                }
            }
        }
    }
    // A fixed integer mixer selects repeatable longer traces, not production randomness.
    for seed in 1u64..=64 {
        let mut d = Driver::new();
        let mut value = seed;
        for _ in 0..64 {
            value = value
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            d.step(
                ((value >> 32) & 1) as usize,
                OPS[((value >> 33) % OPS.len() as u64) as usize],
            );
        }
        for (count, observed) in coverage.iter_mut().zip(d.answers) {
            *count += observed;
        }
    }
    let mut exhausted = Driver::new();
    for _ in 0..40 {
        exhausted.step(0, BAD);
    }
    assert_eq!(exhausted.receivers[0].verification_budget().available(), 0);
    exhausted.step(0, INITIAL); // no credit, no authority
    exhausted.step(0, Op::Wait(1));
    exhausted.step(0, INITIAL); // one newly earned unit, successful admission
    assert!(
        coverage.iter().all(|count| *count > 0),
        "outcome coverage: {coverage:?}"
    );
}

#[test]
#[should_panic(expected = "model receiver occupancy mismatch")]
fn independent_model_rejects_write_then_error_adapter() {
    let mut d = Driver::new();
    d.step(0, INITIAL);
    let context = d.host.context("replacement", 25);
    // Deliberately broken native adapter: publishes replacement and then labels
    // it a recoverable refusal, with no rollback. This is qualification machinery,
    // not a product transaction callback or evidence of installed host rollback.
    let mut broken_commit = || -> Result<(), &'static str> {
        d.receivers[0]
            .admit(
                &credential("replacement"),
                request(),
                d.host.admission(&context),
            )
            .unwrap();
        Err("recoverable commit refusal after publication")
    };
    assert_eq!(
        broken_commit(),
        Err("recoverable commit refusal after publication")
    );
    // A normal commit refusal may retain charged work, but must keep the original
    // login/session/replay/account state. Only the expected work charge changes.
    d.ledgers[0].proof_charges += 1;
    d.check(0); // must detect the changed replay occupancy, not accept the refusal label
}
