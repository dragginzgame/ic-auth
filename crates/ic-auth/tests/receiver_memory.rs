//! Public receiver operations driven by independent Node/OpenSSL signed bytes.
#![cfg(all(feature = "receiver-memory", feature = "ed25519-verification"))]
use ic_auth::{
    receiver_memory::{AdmissionHost, MemoryConfig, MemoryError as E, MemoryReceiver},
    receiver_policy::{
        LoginLimits, LogoutError, LogoutReceipt, SessionAuthority, SessionCapacity,
        SessionPolicyError as P, UnknownLogoutLimits,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
    verification::{
        AuthorityGrants, CredentialError, Ed25519Key, VerificationContext, VerificationCosts,
        VerificationError, VerificationProfile,
    },
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, CanonicalId, Credential, CredentialProof, DelegatedRoleGrant,
    PortableMechanism, Principal,
};
fn bytes(name: &str) -> Vec<u8> {
    let hex = include_str!("fixtures/memory.tsv")
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .find(|(n, _)| *n == name)
        .unwrap()
        .1;
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn credential(name: &str) -> Credential {
    Credential {
        claims: bytes(&format!("{name}-claims")),
        proof: CredentialProof::Ed25519(bytes(&format!("{name}-signature"))),
    }
}
fn request() -> SessionRequest {
    SessionRequest::new(vec!["read".into()], None).unwrap()
}
fn account() -> (Principal, Principal) {
    (Principal::from_slice(&[1]), Principal::from_slice(&[2]))
}
struct Host {
    audience: AudienceId,
    grants: AuthorityGrants,
    role: AuthRole,
    key: Ed25519Key,
}
impl Host {
    fn new() -> Self {
        Self {
            audience: AudienceId {
                canonical_network_id: CanonicalId::from_bytes([0x11; 32]),
                fleet_id: CanonicalId::from_bytes([0x22; 32]),
            },
            grants: AuthorityGrants::new(vec![DelegatedRoleGrant {
                target: "user".parse().unwrap(),
                scopes: vec!["read".parse().unwrap()],
            }])
            .unwrap(),
            role: "user".parse().unwrap(),
            key: Ed25519Key::new(bytes("public-key").try_into().unwrap()).unwrap(),
        }
    }
    fn context(&self, name: &str, now_ns: u64) -> VerificationContext<'_> {
        VerificationContext {
            audience: &self.audience,
            grants: &self.grants,
            issuer: account().0,
            mechanism: PortableMechanism::Ed25519,
            key_binding_id: bytes("key-binding").try_into().unwrap(),
            caller: Principal::from_slice(&bytes(&format!("{name}-caller"))),
            now_ns,
            authority_not_before_ns: 0,
            authority_expires_at_ns: 200,
            enrollment_expires_at_ns: 150,
            min_login_started_at_ns: 0,
            min_proof_issued_at_ns: 0,
            max_login_ttl_ns: 100,
            max_proof_ttl_ns: 60,
        }
    }
    fn admission<'a>(&'a self, context: &'a VerificationContext<'a>) -> AdmissionHost<'a> {
        AdmissionHost {
            verifier: VerificationProfile::Ed25519 {
                context,
                key: &self.key,
            },
            role: &self.role,
            ttl: SessionTtlPolicy::new(60, 1800).unwrap(),
            generation: 7,
            enabled: true,
            subject_admissible: true,
        }
    }
    fn read(&self, name: &str, now_ns: u64) -> SessionAuthority<'_> {
        let c = self.context(name, now_ns);
        SessionAuthority {
            caller: c.caller,
            now_ns,
            audience: self.audience,
            issuer: c.issuer,
            mechanism: c.mechanism,
            key_binding_id: c.key_binding_id,
            role: &self.role,
            grants: self.grants.as_slice(),
            generation: 7,
            not_before_ns: 0,
            expires_at_ns: 200,
            enrollment_expires_at_ns: 150,
            min_login_started_at_ns: 0,
            max_session_ttl_secs: 1800,
            enabled: true,
            subject_admissible: true,
        }
    }
    fn config(&self) -> MemoryConfig {
        MemoryConfig {
            unknown_logout: UnknownLogoutLimits {
                max_denials: 2,
                capacity: 2,
                refill_interval_ns: 1,
            },
            audience: self.audience,
            login: LoginLimits {
                max_device_slots: 8,
                max_login_ttl_ns: 100,
                max_admitted_logins: 1,
            },
            capacity: SessionCapacity {
                max_sessions: 1,
                max_replays: 2,
                max_replays_per_account: 2,
                max_encoded_bytes: 4096,
            },
            verification_capacity: 10,
            refill_interval_ns: 1,
            verification_costs: VerificationCosts {
                ed25519: 2,
                certified: 3,
            },
        }
    }
    fn receiver(&self) -> MemoryReceiver {
        let mut r = MemoryReceiver::new_fresh(self.config(), 0).unwrap();
        r.activate_fresh(7).unwrap();
        r
    }
}

#[test]
fn memory_admission_retry_read_and_exclusive_lease() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let first = r
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert!(!first.exact_retry);
    assert_eq!(first.receipt.expires_at_ns, 100);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).replays, 1);
    assert_eq!(r.verification_budget().available(), 8);
    let usage = r.usage(account());
    let mut altered = credential("initial");
    altered.proof = CredentialProof::Ed25519(vec![]);
    let later = h.context("initial", 50); // proof expired at 30, alternate malformed proof is irrelevant
    let retry = r.admit(&altered, request(), h.admission(&later)).unwrap();
    assert!(retry.exact_retry);
    assert_eq!(retry.receipt, first.receipt);
    assert_eq!(r.usage(account()), usage);
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(r.verification_budget().observed_at_ns(), 25);
    assert_eq!(
        r.read(&h.read("initial", 50), &["read".parse().unwrap()]),
        Ok(first.receipt)
    );
    assert_eq!(
        r.read(&h.read("initial", 50), &["write".parse().unwrap()]),
        Err(E::Policy(P::ScopeRejected))
    );
    assert_eq!(
        r.admit(
            &altered,
            SessionRequest::new(vec!["read".into()], Some(60)).unwrap(),
            h.admission(&later)
        ),
        Err(E::Policy(P::ReplayConflict))
    );
    assert_eq!(
        r.read(&h.read("initial", 100), &[]),
        Err(E::Policy(P::MissingOrExpired))
    );
    assert_eq!(
        r.read(&h.read("initial", 99), &[]),
        Err(E::Policy(P::ClockRollback))
    );
    assert_eq!(r.usage(account()), usage);
}

#[test]
fn memory_invalid_proof_retains_charge_without_reservation() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let mut bad = credential("initial");
    if let CredentialProof::Ed25519(sig) = &mut bad.proof {
        sig[0] ^= 1;
    }
    let empty = r.usage(account());
    assert_eq!(
        r.admit(&bad, request(), h.admission(&c)),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::InvalidSignature
        )))
    );
    assert_eq!(r.usage(account()), empty);
    assert_eq!(r.known_logins(), 0);
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(r.observed_at_ns(), 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(r.verification_budget().available(), 6);
    let rebound = h.context("rebound", 25);
    assert_eq!(
        r.admit(&credential("rebound"), request(), h.admission(&rebound)),
        Err(E::Policy(P::BindingConflict))
    );
    assert_eq!(r.verification_budget().available(), 6);
}

#[test]
fn memory_full_pool_replacement_is_verified_before_displacement() {
    let h = Host::new();
    let mut r = h.receiver();
    let first = h.context("initial", 25);
    let original = r
        .admit(&credential("initial"), request(), h.admission(&first))
        .unwrap();
    let replacement = h.context("replacement", 25);
    let mut bad = credential("replacement");
    if let CredentialProof::Ed25519(sig) = &mut bad.proof {
        sig[0] ^= 1;
    }
    let before = r.usage(account());
    assert!(r.admit(&bad, request(), h.admission(&replacement)).is_err());
    assert_eq!(r.usage(account()), before);
    assert_eq!(r.read(&h.read("initial", 25), &[]), Ok(original.receipt));
    let next = r
        .admit(
            &credential("replacement"),
            request(),
            h.admission(&replacement),
        )
        .unwrap();
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).sessions, 1);
    assert_eq!(r.usage(account()).replays_for_account, 2);
    assert_eq!(r.read(&h.read("replacement", 25), &[]), Ok(next.receipt));
    assert_eq!(
        r.read(&h.read("initial", 25), &[]),
        Err(E::Policy(P::MissingOrExpired))
    );
    assert_eq!(
        r.admit(&credential("initial"), request(), h.admission(&first)),
        Err(E::Policy(P::ReplayConflict))
    );
    let third = h.context("third", 25);
    let credit = r.verification_budget().available();
    assert_eq!(
        r.admit(&credential("third"), request(), h.admission(&third)),
        Err(E::Policy(P::Capacity))
    );
    assert_eq!(r.verification_budget().available(), credit);
    // Replay expiry cannot erase the slot generation fence. This old browser
    // presents a separately signed, still-fresh credential after both replays go.
    assert_eq!(r.prune(40, 2).unwrap().replays, 2);
    let late_old = h.context("late-old", 40);
    assert_eq!(
        r.admit(&credential("late-old"), request(), h.admission(&late_old)),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(r.verification_budget().available(), credit);
}

#[test]
fn memory_logout_pruning_and_device_fences_survive_expiry() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(
        r.logout(c.caller, 25),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(r.usage(account()).sessions, 0);
    assert_eq!(r.usage(account()).replays, 1);
    assert_eq!(
        r.admit(&credential("renewal"), request(), h.admission(&c)),
        Err(E::Policy(P::Revoked))
    );
    let replacement = h.context("replacement", 25);
    r.admit(
        &credential("replacement"),
        request(),
        h.admission(&replacement),
    )
    .unwrap();
    assert_eq!(r.prune(100, 1).unwrap().sessions, 1);
    assert_eq!(r.usage(account()).replays, 2);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.prune(100, 1).unwrap().replays, 1);
    assert_eq!(r.usage(account()).replays_for_account, 1);
    assert_eq!(r.prune(100, 1).unwrap().replays, 1);
    assert_eq!(r.usage(account()).encoded_bytes, 0);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(
        r.logout(replacement.caller, 100),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(
        r.logout(c.caller, 100),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 200 })
    );
}

#[test]
fn memory_renewal_reuses_session_but_retains_account_replay_quota() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let first = r
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    let renewed = r
        .admit(&credential("renewal"), request(), h.admission(&c))
        .unwrap();
    assert!(!renewed.exact_retry);
    assert_eq!(renewed.receipt.expires_at_ns, first.receipt.expires_at_ns);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).sessions, 1);
    assert_eq!(r.usage(account()).replays_for_account, 2);
    assert_eq!(
        r.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Policy(P::ReplayConflict))
    );
}

#[test]
fn memory_readiness_generation_and_byte_capacity_are_enforced() {
    let h = Host::new();
    let c = h.context("initial", 25);
    let mut fresh = MemoryReceiver::new_fresh(h.config(), 0).unwrap();
    assert_eq!(
        fresh.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Closed)
    );
    fresh.activate_fresh(7).unwrap();
    fresh
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(fresh.advance_generation(25), Ok(8));
    assert_eq!(
        fresh.read(&h.read("initial", 25), &[]),
        Err(E::Policy(P::StaleAuthority))
    );
    let mut current = h.read("initial", 25);
    current.generation = 8;
    assert_eq!(fresh.read(&current, &[]), Err(E::Policy(P::StaleAuthority)));
    fresh.close();
    assert_eq!(fresh.activate_fresh(9), Err(E::Closed));
    assert_eq!(fresh.prune(100, 1), Err(E::Closed));
    let mut config = h.config();
    config.capacity.max_encoded_bytes = 1;
    let mut tiny = MemoryReceiver::new_fresh(config, 0).unwrap();
    tiny.activate_fresh(7).unwrap();
    assert_eq!(
        tiny.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Policy(P::ByteCapacity))
    );
    assert_eq!(tiny.verification_budget().observed_at_ns(), 0);
    assert_eq!(tiny.known_logins(), 0);
    assert_eq!(tiny.observed_at_ns(), 25);
}

#[test]
fn memory_logical_record_bytes_match_reference_field_sizes() {
    // Independently summed Postcard representations of the reference SessionData:
    // header 11; principals 30+2+2; audience byte strings 33+33; mechanism 1;
    // fixed key 32; role 5; scope vector 6; seven single-byte numeric fields;
    // fingerprint/request hash 32+32. Replay: 11+32+30+2+2+1+1.
    const SESSION: usize = 226;
    const REPLAY: usize = 79;
    let h = Host::new();
    let c = h.context("initial", 25);
    for (generation, expected) in [(7, SESSION + REPLAY), (u64::MAX, SESSION + REPLAY + 18)] {
        // u64::MAX takes ten bytes in each record instead of one.
        for ceiling in [expected - 1, expected] {
            let mut config = h.config();
            config.capacity.max_encoded_bytes = ceiling;
            let mut r = MemoryReceiver::new_fresh(config, 0).unwrap();
            r.activate_fresh(generation).unwrap();
            let mut host = h.admission(&c);
            host.generation = generation;
            let result = r.admit(&credential("initial"), request(), host);
            if ceiling < expected {
                assert_eq!(result, Err(E::Policy(P::ByteCapacity)));
                assert_eq!(r.usage(account()).encoded_bytes, 0);
                assert_eq!(r.verification_budget().observed_at_ns(), 0);
            } else {
                result.unwrap();
                assert_eq!(r.usage(account()).encoded_bytes, expected);
                if generation == u64::MAX {
                    assert_eq!(r.advance_generation(25), Err(E::Policy(P::StaleAuthority)));
                }
            }
        }
    }
}

#[test]
fn memory_replay_pruning_does_not_extend_or_destroy_live_exact_retry() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let first = r
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(r.prune(30, 1).unwrap().replays, 1);
    assert_eq!(r.usage(account()).encoded_bytes, 226);
    let later = h.context("initial", 30);
    assert_eq!(
        r.admit(&credential("initial"), request(), h.admission(&later))
            .unwrap()
            .receipt,
        first.receipt
    );
    assert_eq!(r.verification_budget().available(), 8);
    let mut separate = h.receiver();
    assert!(matches!(
        separate.admit(&credential("initial"), request(), h.admission(&later)),
        Err(E::Context(_))
    ));
    assert_eq!(separate.known_logins(), 0);
    assert_eq!(separate.verification_budget().observed_at_ns(), 0);
}

#[test]
fn memory_uncharged_refusals_and_empty_budget_cannot_publish() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let empty = r.usage(account());
    let mut malformed = credential("initial");
    malformed.proof = CredentialProof::Ed25519(vec![0; 63]);
    assert_eq!(
        r.admit(&malformed, request(), h.admission(&c)),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::MalformedSignature
        )))
    );
    assert_eq!(r.verification_budget().observed_at_ns(), 0);
    assert_eq!(r.observed_at_ns(), 25);
    assert_eq!(r.usage(account()), empty);
    let mut disabled = h.admission(&c);
    disabled.enabled = false;
    assert_eq!(
        r.admit(&credential("initial"), request(), disabled),
        Err(E::Policy(P::Disabled))
    );
    // A receiver created at this same trusted time starts with no work credit.
    let mut fresh = MemoryReceiver::new_fresh(h.config(), 25).unwrap();
    fresh.activate_fresh(7).unwrap();
    assert!(matches!(
        fresh.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::Budget(_)
        )))
    ));
    assert_eq!(fresh.known_logins(), 0);
    assert_eq!(fresh.usage(account()), empty);
    // A wrong authenticated caller never establishes the submitted login.
    let wrong = h.context("replacement", 26);
    assert!(matches!(
        r.admit(&credential("initial"), request(), h.admission(&wrong)),
        Err(E::Context(_))
    ));
    assert_eq!(r.observed_at_ns(), 25);
    assert_eq!(r.known_logins(), 0);
}

fn unknown_caller(byte: u8) -> Principal {
    let mut bytes = [byte; 29];
    bytes[28] = 2;
    Principal::from_slice(&bytes)
}

#[test]
fn logout_unknown_retries_remain_unconfirmed_without_sliding_or_charging() {
    let h = Host::new();
    let mut r = h.receiver();
    let caller = unknown_caller(1);
    assert_eq!(r.logout(caller, 0), Err(LogoutError::Capacity)); // starts empty
    assert_eq!(r.unknown_logins(), 0);
    let original = LogoutReceipt::Unconfirmed { deny_until_ns: 101 };
    assert_eq!(r.logout(caller, 1), Ok(original));
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.logout(caller, 100), Ok(original));
    assert_eq!(r.unknown_logout_credit(), 0); // retry does not even refill
    assert_eq!(r.unknown_logins(), 1);
    assert_eq!(r.known_logins(), 0);
    assert_eq!(r.verification_budget().observed_at_ns(), 0);
    assert_eq!(
        r.logout(unknown_caller(2), 100),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 200 })
    );
    assert_eq!(r.unknown_logout_credit(), 1);
    assert_eq!(r.logout(unknown_caller(3), 100), Err(LogoutError::Capacity));
    assert_eq!(r.unknown_logout_credit(), 1); // pool refusal precedes a charge
    assert_eq!(r.logout(caller, 100), Ok(original));
    assert_eq!(r.unknown_logins(), 2);
}

#[test]
fn logout_unknown_flood_cannot_spend_known_capacity_or_proof_credit() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    for caller in [unknown_caller(10), unknown_caller(11)] {
        r.logout(caller, 26).unwrap();
    }
    assert_eq!(r.logout(unknown_caller(12), 26), Err(LogoutError::Capacity));
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.unknown_logins(), 2);
    assert_eq!(
        r.logout(c.caller, 26),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(
        r.logout(c.caller, 101),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(r.unknown_logins(), 2);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).sessions, 0);
    assert_eq!(r.usage(account()).replays, 1);
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.verification_budget().available(), 8);
}

#[test]
fn logout_expired_unknown_reuses_its_slot_and_replaces_only_its_expiry() {
    let h = Host::new();
    let mut config = h.config();
    config.unknown_logout.max_denials = 1;
    let mut r = MemoryReceiver::new_fresh(config, 0).unwrap();
    r.activate_fresh(7).unwrap();
    let caller = unknown_caller(4);
    assert_eq!(
        r.logout(caller, 1),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 101 })
    );
    assert_eq!(r.logout(unknown_caller(5), 101), Err(LogoutError::Capacity)); // expired physical row still counts
    assert_eq!(
        r.logout(caller, 101),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 201 })
    );
    assert_eq!(r.unknown_logins(), 1);
    assert_eq!(r.prune(101, 128).unwrap().unknown_denials, 0); // old expiry removed
    assert_eq!(r.prune(201, 1).unwrap().unknown_denials, 1);
    assert_eq!(r.unknown_logins(), 0);
    assert_eq!(r.known_logins(), 0);
}

#[test]
fn logout_unconfirmed_cannot_exclude_unseen_future_credentials() {
    let h = Host::new();
    for prune_first in [false, true] {
        let mut r = h.receiver();
        let future = h.context("future", 125);
        assert_eq!(
            r.logout(future.caller, 1),
            Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 101 })
        );
        let early = h.context("future", 100);
        assert_eq!(
            r.admit(&credential("future"), request(), h.admission(&early)),
            Err(E::Policy(P::Revoked))
        );
        if prune_first {
            assert_eq!(r.prune(101, 1).unwrap().unknown_denials, 1);
        }
        let mut invalid = credential("future");
        if let CredentialProof::Ed25519(sig) = &mut invalid.proof {
            sig[0] ^= 1;
        }
        assert!(r.admit(&invalid, request(), h.admission(&future)).is_err());
        assert_eq!(r.known_logins(), 0);
        assert_eq!(r.unknown_logins(), usize::from(!prune_first));
        // Same independently signed future credential becomes admissible after
        // the temporary denial. No caller-selected time or fabricated verifier.
        let result = r
            .admit(&credential("future"), request(), h.admission(&future))
            .unwrap();
        assert_eq!(result.receipt.subject, Principal::from_slice(&[3]));
        assert_eq!(r.known_logins(), 1);
        assert_eq!(r.unknown_logins(), 0);
        assert_eq!(r.verification_budget().available(), 6);
        assert_eq!(r.prune(125, 128).unwrap().unknown_denials, 0);
        r.logout(unknown_caller(6), 125).unwrap();
        assert_eq!(r.unknown_logins(), 1);
    }
}

#[test]
fn logout_expired_unknown_transfer_refuses_without_freeing_either_pool() {
    let h = Host::new();
    let mut r = h.receiver();
    let initial = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&initial))
        .unwrap();
    let future = h.context("future", 125);
    r.logout(future.caller, 25).unwrap();
    assert_eq!(
        r.admit(&credential("future"), request(), h.admission(&future)),
        Err(E::Policy(P::Capacity))
    );
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.unknown_logins(), 1);
    assert_eq!(r.verification_budget().available(), 8);
    let pruned = r.prune(125, 128).unwrap();
    assert_eq!(pruned.unknown_denials, 1);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.unknown_logins(), 0);
    assert_eq!(
        r.logout(initial.caller, 125),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
}

#[test]
fn logout_invalid_clock_overflow_and_closed_state_never_acknowledge() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    for caller in [
        Principal::anonymous(),
        Principal::management_canister(),
        Principal::from_slice(&[2]),
    ] {
        assert_eq!(r.logout(caller, 26), Err(LogoutError::InvalidCaller));
    }
    assert_eq!(r.observed_at_ns(), 25);
    assert_eq!(r.logout(c.caller, 24), Err(LogoutError::Clock));
    assert_eq!(
        r.logout(unknown_caller(7), u64::MAX),
        Err(LogoutError::Clock)
    );
    assert_eq!(r.observed_at_ns(), u64::MAX);
    assert_eq!(r.unknown_logins(), 0);
    assert_eq!(r.unknown_logout_credit(), 0);
    // Known expiry is retained even when computing a new temporary lease would overflow.
    assert_eq!(
        r.logout(c.caller, u64::MAX),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    r.close();
    assert_eq!(r.logout(c.caller, u64::MAX), Err(LogoutError::Closed));
    let mut fresh = MemoryReceiver::new_fresh(h.config(), 0).unwrap();
    assert_eq!(fresh.logout(c.caller, 1), Err(LogoutError::Closed));
}

#[test]
fn logout_operation_budget_is_separate_and_retains_refill_remainder() {
    let h = Host::new();
    let mut config = h.config();
    config.unknown_logout = UnknownLogoutLimits {
        max_denials: 4,
        capacity: 2,
        refill_interval_ns: 3,
    };
    let mut r = MemoryReceiver::new_fresh(config, 0).unwrap();
    r.activate_fresh(7).unwrap();
    assert_eq!(r.logout(unknown_caller(1), 2), Err(LogoutError::Capacity));
    assert_eq!(
        r.logout(unknown_caller(1), 5),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 105 })
    );
    assert_eq!(r.logout(unknown_caller(2), 5), Err(LogoutError::Capacity));
    assert_eq!(
        r.logout(unknown_caller(2), 6),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 106 })
    );
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.verification_budget().observed_at_ns(), 0);
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(r.unknown_logout_credit(), 0);
    r.logout(c.caller, 25).unwrap();
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.verification_budget().available(), 8);
}

#[test]
fn logout_admission_orderings_deny_without_cross_receiver_effects() {
    let h = Host::new();
    let c = h.context("initial", 25);
    let mut before = h.receiver();
    let mut after = h.receiver();
    assert_eq!(
        before.logout(c.caller, 25),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 125 })
    );
    assert_eq!(
        before.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(before.verification_budget().available(), 0);
    after
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(
        after.logout(c.caller, 25),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(
        after.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(before.unknown_logins(), 1);
    assert_eq!(after.unknown_logins(), 0);
    assert_eq!(before.known_logins(), 0);
    assert_eq!(after.known_logins(), 1);
}
