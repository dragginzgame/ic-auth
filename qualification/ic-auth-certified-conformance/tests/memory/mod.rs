//! Certified receiver lifecycle using independently assembled claims and root signatures.
use super::{
    Fixture, RECORDS, RootFixture, SEED, bls_key, grants, host, issuer_cbor_for, root_fixture, sha,
    wire_credential,
};
use ic_auth::{
    receiver_memory::{
        AdmissionHost, AdmissionOutcome, EnrollmentBinding, EnrollmentMetadata, LoginLimits,
        LogoutReceipt, MemoryConfig, MemoryError, SessionAuthority, SessionCapacity,
        SessionPolicyError, StableReceiver, UnknownLogoutLimits,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
    verification::{
        AuthorityGrants, CertifiedContext, ContextError, VerificationCosts, VerificationProfile,
    },
};
use ic_auth_protocol_types::{AuthRole, Credential, CredentialProof, Principal};
use ic_platform::certificate::construction::canister_key_der;

const MEMORY: &str = include_str!("../fixtures/certified-memory.tsv");
const OTHER: &str = include_str!("../fixtures/certified-other-issuer.tsv");

///
/// Case
///
/// Pairs a signed credential and caller with its independent certified fixture.
///

struct Case {
    fixture: Fixture,
    credential: Credential,
    caller: Principal,
    issuer: Principal,
    binding: [u8; 32],
}

impl Case {
    fn new(name: &str) -> Self {
        let records = if name == "other-issuer" {
            OTHER
        } else {
            RECORDS
        };
        let issuer = Principal::from_slice(&[if name == "other-issuer" { 3 } else { 2 }; 10]);
        let mut root = RootFixture::new(records, sha(SEED));
        root.cert.delegation.issuer = issuer;
        root.leaf.delegation.issuer = issuer;
        let claims = root_fixture::bytes(MEMORY, &format!("{name}-claims"));
        let caller = Principal::from_slice(&root_fixture::bytes(MEMORY, &format!("{name}-caller")));
        let digest = root_fixture::bytes(MEMORY, &format!("{name}-digest"));
        assert_eq!(sha(&claims).as_slice(), digest);
        let time = if matches!(name, "renewal" | "replacement" | "late-old") {
            181
        } else {
            150
        };
        let fixture = Fixture {
            root,
            claims,
            cbor: issuer_cbor_for(
                issuer,
                digest.try_into().unwrap(),
                vec![time as u8 | 0x80, 1],
            ),
            der: canister_key_der(issuer, SEED).unwrap(),
            network: bls_key().public_key().serialize(),
        };
        // Exercise the maintained Wire envelope and consuming domain projection.
        let frame =
            ic_auth_wire::CredentialEnvelope::encode_response(&wire_credential(&fixture)).unwrap();
        let credential = Credential::try_from(
            ic_auth_wire::CredentialEnvelope::decode_response(&frame).unwrap(),
        )
        .unwrap();
        Self {
            fixture,
            credential,
            caller,
            issuer,
            binding: root_fixture::bytes(records, "certificate_hash")
                .try_into()
                .unwrap(),
        }
    }

    fn context<'a>(&'a self, grants: &'a AuthorityGrants, now: u64) -> CertifiedContext<'a> {
        let mut context = host(&self.fixture, grants);
        context.claims.caller = self.caller;
        context.claims.issuer = self.issuer;
        context.claims.key_binding_id = self.binding;
        context.claims.now_ns = now;
        context
    }

    fn read<'a>(
        &self,
        grants: &'a AuthorityGrants,
        role: &'a AuthRole,
        now: u64,
    ) -> SessionAuthority<'a> {
        SessionAuthority {
            caller: self.caller,
            now_ns: now,
            audience: self.fixture.root.cert.delegation.audience,
            issuer: self.issuer,
            mechanism: ic_auth_protocol_types::PortableMechanism::Certified,
            key_binding_id: self.binding,
            role,
            grants: grants.as_slice(),
            generation: 9,
            not_before_ns: 100,
            expires_at_ns: 200,
            enrollment_expires_at_ns: 210,
            min_login_started_at_ns: 100,
            max_session_ttl_secs: 1800,
            enabled: true,
            subject_admissible: true,
        }
    }

    fn admit(
        &self,
        receiver: &mut StableReceiver,
        grants: &AuthorityGrants,
        role: &AuthRole,
        now: u64,
    ) -> Result<AdmissionOutcome, MemoryError> {
        receiver.admit(
            &self.credential,
            request(),
            input(&self.context(grants, now), role),
        )
    }
}

fn request() -> SessionRequest {
    SessionRequest::new(vec!["read".into()], None).unwrap()
}

fn input<'a>(context: &'a CertifiedContext<'a>, role: &'a AuthRole) -> AdmissionHost<'a> {
    AdmissionHost {
        verifier: VerificationProfile::Certified(context),
        role,
        ttl: SessionTtlPolicy::new(60, 1800).unwrap(),
        generation: 9,
        enabled: true,
        subject_admissible: true,
    }
}

fn receiver(case: &Case, id: u8) -> StableReceiver {
    let binding = EnrollmentBinding {
        receiver: Principal::from_slice(&[id]),
        authority: Principal::from_slice(&[11]),
    };
    let audience = case.fixture.root.cert.delegation.audience;
    let mut receiver = StableReceiver::new_fresh(
        super::lease("auth.certified.receiver.v1"),
        MemoryConfig {
            audience,
            enrollment: binding,
            login: LoginLimits {
                max_device_slots: 8,
                max_login_ttl_ns: 100,
                max_admitted_logins: 3,
            },
            capacity: SessionCapacity {
                max_sessions: 3,
                max_replays: 3,
                max_replays_per_account: 2,
                max_encoded_bytes: 4096,
            },
            unknown_logout: UnknownLogoutLimits {
                max_denials: 1,
                capacity: 1,
                refill_interval_ns: 1,
            },
            verification_capacity: 30,
            refill_interval_ns: 1,
            verification_costs: VerificationCosts {
                ed25519: 2,
                certified: 3,
            },
        },
        0,
    )
    .unwrap();
    receiver.request_enrollment(0, [id; 32], 10).unwrap();
    receiver
        .install_enrollment(
            0,
            binding.authority,
            EnrollmentMetadata {
                receiver: binding.receiver,
                audience,
                challenge: [id; 32],
                revision: 1,
                generation: 9,
                snapshot_digest: [2; 32],
                authority_digest: [3; 32],
                issued_at_ns: 0,
                expires_at_ns: 210,
            },
        )
        .unwrap();
    receiver
}

#[test]
fn memory_certified_accounts_devices_and_receivers_preserve_independent_lifecycles() {
    let initial = Case::new("initial");
    let device = Case::new("other-device");
    let other = Case::new("other-issuer");
    let replacement = Case::new("replacement");
    let renewal = Case::new("renewal");
    let early_renewal = Case::new("early-renewal");
    let late_old = Case::new("late-old");
    let grants = grants(&initial.fixture);
    let role = "app".parse().unwrap();
    let account = (initial.issuer, Principal::from_slice(&[5; 10]));
    let other_account = (other.issuer, account.1);
    let mut first = receiver(&initial, 10);
    let mut second = receiver(&initial, 12);
    for r in [&mut first, &mut second] {
        initial.admit(r, &grants, &role, 150).unwrap();
        device.admit(r, &grants, &role, 150).unwrap();
        assert_eq!(r.usage(account).replays_for_account, 2);
        // The account quota is full while one global replay/session slot remains.
        assert_eq!(
            early_renewal.admit(r, &grants, &role, 150),
            Err(MemoryError::Policy(SessionPolicyError::Capacity))
        );
        let mut wrong = other.context(&grants, 150);
        wrong.claims.issuer = initial.issuer;
        assert_eq!(
            r.admit(&other.credential, request(), input(&wrong, &role)),
            Err(MemoryError::Context(ContextError::IssuerMismatch))
        );
        other.admit(r, &grants, &role, 150).unwrap();
        assert_eq!(r.usage(other_account).replays_for_account, 1);
        assert_eq!(r.verification_budget().available(), 21);
        for case in [&initial, &device, &other] {
            assert_eq!(
                r.authorize(&case.read(&grants, &role, 150), &[], |s| (
                    s.issuer, s.subject
                )),
                Ok((case.issuer, account.1))
            );
        }
        let usage = r.usage(account);
        let context = initial.context(&grants, 150);
        assert_eq!(
            r.admit(
                &initial.credential,
                SessionRequest::new(vec!["read".into()], Some(60)).unwrap(),
                input(&context, &role)
            ),
            Err(MemoryError::Policy(SessionPolicyError::ReplayConflict))
        );
        let mut wrong = device.context(&grants, 150);
        wrong.claims.caller = other.caller;
        assert_eq!(
            r.admit(&device.credential, request(), input(&wrong, &role)),
            Err(MemoryError::Context(ContextError::CallerMismatch))
        );
        assert_eq!(r.usage(account), usage);
        assert_eq!(r.verification_budget().available(), 21);
    }

    let original_receipt = second
        .read(&initial.read(&grants, &role, 150), &[])
        .unwrap();
    // Full physical quotas still permit exact retry after both proof and certificate age expire.
    let context = initial.context(&grants, 181);
    let mut retry = initial.credential.clone();
    retry.proof = CredentialProof::Ed25519(vec![]);
    let repeated = second
        .admit(&retry, request(), input(&context, &role))
        .unwrap();
    assert!(repeated.exact_retry);
    assert_eq!(repeated.receipt, original_receipt);
    assert_eq!(repeated.receipt.expires_at_ns, 190);
    assert_eq!(second.verification_budget().available(), 21);
    assert_eq!(
        replacement.admit(&mut first, &grants, &role, 181),
        Err(MemoryError::Policy(SessionPolicyError::Capacity))
    );
    assert_eq!(first.verification_budget().available(), 21);
    assert_eq!(first.prune(181, 128).unwrap().replays, 3);
    let usage = first.usage(account);
    for break_root in [true, false] {
        let mut bad = replacement.credential.clone();
        let CredentialProof::Certified(proof) = &mut bad.proof else {
            unreachable!()
        };
        if break_root {
            proof.delegation.signature.signature[0] ^= 1;
        } else {
            proof.issuer_signature_cbor[0] ^= 1;
        }
        assert!(matches!(
            first.admit(
                &bad,
                request(),
                input(&replacement.context(&grants, 181), &role)
            ),
            Err(MemoryError::Proof(_))
        ));
        assert_eq!(first.usage(account), usage);
        assert!(
            first
                .authorize(&initial.read(&grants, &role, 181), &[], |_| ())
                .is_ok()
        );
    }
    assert_eq!(first.verification_budget().available(), 24); // refilled to 30, charged twice
    replacement.admit(&mut first, &grants, &role, 181).unwrap();
    assert_eq!(first.known_logins(), 3); // replace in the full login/session pool
    assert_eq!(first.usage(account).sessions, 3);
    assert_eq!(
        first.authorize(&initial.read(&grants, &role, 181), &[], |_| panic!(
            "displaced caller authorized"
        )),
        Err(MemoryError::Policy(SessionPolicyError::MissingOrExpired))
    );
    for case in [&device, &other] {
        assert!(
            first
                .authorize(&case.read(&grants, &role, 181), &[], |_| ())
                .is_ok()
        );
    }
    assert_eq!(
        late_old.admit(&mut first, &grants, &role, 181),
        Err(MemoryError::Policy(SessionPolicyError::Revoked))
    );
    assert_eq!(first.verification_budget().available(), 21);

    assert_eq!(second.prune(181, 128).unwrap().replays, 3);
    let renewed = renewal.admit(&mut second, &grants, &role, 181).unwrap();
    assert!(!renewed.exact_retry);
    assert_eq!(
        renewed.receipt.expires_at_ns,
        repeated.receipt.expires_at_ns
    );
    assert_eq!(
        first.logout(replacement.caller, 182),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 190 })
    );
    assert_eq!(first.usage(account).replays_for_account, 1);
    assert_eq!(
        replacement.admit(&mut first, &grants, &role, 182),
        Err(MemoryError::Policy(SessionPolicyError::Revoked))
    );
    assert!(
        second
            .authorize(&initial.read(&grants, &role, 182), &[], |_| ())
            .is_ok()
    );
    for case in [&device, &other] {
        assert!(
            first
                .authorize(&case.read(&grants, &role, 182), &[], |_| ())
                .is_ok()
        );
    }
    // A previously displaced browser is now unknown locally; its denial remains unconfirmed.
    assert_eq!(
        first.logout(initial.caller, 182),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 282 })
    );
    assert_eq!(
        first.logout(initial.caller, 183),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 282 })
    );
    assert_eq!(first.prune(190, 128).unwrap().sessions, 2);
    assert_eq!(first.known_logins(), 3);
    assert_eq!(first.usage(account).replays, 0);
    assert_eq!(
        second.authorize(&initial.read(&grants, &role, 190), &[], |_| panic!(
            "expired renewal authorized"
        )),
        Err(MemoryError::Policy(SessionPolicyError::MissingOrExpired))
    );
}

#[test]
fn memory_certified_enrollment_refresh_and_generation_preserve_captured_leases_and_fences() {
    let initial = Case::new("initial");
    let device = Case::new("other-device");
    let grants = grants(&initial.fixture);
    let role = "app".parse().unwrap();
    let mut r = receiver(&initial, 10);
    let snapshot = r.enrollment().unwrap();
    let source = Principal::from_slice(&[11]);
    r.request_enrollment(140, [20; 32], 149).unwrap();
    r.install_enrollment(
        140,
        source,
        EnrollmentMetadata {
            challenge: [20; 32],
            revision: 2,
            snapshot_digest: [20; 32],
            issued_at_ns: 140,
            expires_at_ns: 175,
            ..snapshot
        },
    )
    .unwrap();
    let admitted = initial.admit(&mut r, &grants, &role, 150).unwrap();
    assert_eq!(admitted.receipt.expires_at_ns, 190);
    let account = (initial.issuer, Principal::from_slice(&[5; 10]));
    let original_usage = r.usage(account);
    let credit = r.verification_budget().available();
    r.request_enrollment(151, [21; 32], 160).unwrap();
    r.install_enrollment(
        151,
        source,
        EnrollmentMetadata {
            challenge: [21; 32],
            revision: 3,
            snapshot_digest: [21; 32],
            issued_at_ns: 151,
            ..snapshot
        },
    )
    .unwrap();
    let retry = initial.admit(&mut r, &grants, &role, 151).unwrap();
    assert!(retry.exact_retry);
    assert_eq!(retry.receipt, admitted.receipt);
    assert_eq!(r.usage(account), original_usage);
    assert_eq!(r.verification_budget().available(), credit);
    assert_eq!(
        r.read(&initial.read(&grants, &role, 175), &[]),
        Ok(admitted.receipt),
    );
    // A distinct device still has independently valid proof and captured authority.
    // Replace the expired certificate witness with a current one for the same claims.
    let mut fresh = device.credential.clone();
    let CredentialProof::Certified(proof) = &mut fresh.proof else {
        unreachable!()
    };
    proof.issuer_signature_cbor = issuer_cbor_for(device.issuer, sha(&fresh.claims), vec![0xaf, 1]);
    r.admit(
        &fresh,
        request(),
        input(&device.context(&grants, 175), &role),
    )
    .unwrap();
    let usage = r.usage(account);
    let credit = r.verification_budget().available();
    r.request_enrollment(176, [22; 32], 185).unwrap();
    r.install_enrollment(
        176,
        source,
        EnrollmentMetadata {
            challenge: [22; 32],
            revision: 4,
            generation: 10,
            authority_digest: [22; 32],
            snapshot_digest: [22; 32],
            issued_at_ns: 176,
            ..snapshot
        },
    )
    .unwrap();
    let mut current = device.read(&grants, &role, 176);
    current.generation = 10;
    assert_eq!(
        r.authorize(&current, &[], |_| panic!("stale session authorized")),
        Err(MemoryError::Policy(SessionPolicyError::StaleAuthority))
    );
    assert_eq!(r.usage(account), usage);
    assert_eq!(r.verification_budget().available(), credit);
    // Even after enrollment expiry, intact known-login state supports local revocation.
    assert_eq!(
        r.logout(device.caller, 210),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 190 })
    );
    assert_eq!(r.known_logins(), 2);
    assert_eq!(r.usage(account).replays_for_account, 2);
    r.close();
    assert_eq!(
        r.request_enrollment(210, [23; 32], 220),
        Err(MemoryError::Closed)
    );
    assert_eq!(r.advance_generation(210), Err(MemoryError::Closed));
    assert_eq!(
        initial.admit(&mut r, &grants, &role, 210),
        Err(MemoryError::Closed)
    );
}

#[test]
fn memory_certified_unknown_logout_before_admission_cannot_install_identity() {
    let initial = Case::new("initial");
    let grants = grants(&initial.fixture);
    let role = "app".parse().unwrap();
    let mut denied = receiver(&initial, 10);
    let mut independent = receiver(&initial, 12);
    assert_eq!(
        denied.logout(initial.caller, 150),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 250 })
    );
    assert_eq!(
        initial.admit(&mut denied, &grants, &role, 150),
        Err(MemoryError::Policy(SessionPolicyError::Revoked))
    );
    assert_eq!(denied.known_logins(), 0);
    assert_eq!(denied.unknown_logins(), 1);
    assert_eq!(denied.verification_budget().available(), 0);
    assert_eq!(
        denied
            .usage((initial.issuer, Principal::from_slice(&[5; 10])))
            .encoded_bytes,
        0
    );
    initial
        .admit(&mut independent, &grants, &role, 150)
        .unwrap();
    assert!(
        independent
            .authorize(&initial.read(&grants, &role, 150), &[], |_| ())
            .is_ok()
    );
    assert_eq!(
        denied.logout(initial.caller, 180),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 250 })
    );
    assert_eq!(denied.unknown_logout_credit(), 0);
}

#[test]
fn memory_certified_replacement_of_denied_slot_still_requires_complete_proof() {
    let initial = Case::new("initial");
    let device = Case::new("other-device");
    let other = Case::new("other-issuer");
    let replacement = Case::new("replacement");
    let grants = grants(&initial.fixture);
    let role = "app".parse().unwrap();
    let account = (initial.issuer, Principal::from_slice(&[5; 10]));
    let mut r = receiver(&initial, 10);
    for case in [&initial, &device, &other] {
        case.admit(&mut r, &grants, &role, 150).unwrap();
    }
    r.logout(initial.caller, 150).unwrap();
    assert_eq!(r.known_logins(), 3); // denial retains the full known-login pool
    assert_eq!(r.prune(181, 128).unwrap().replays, 3);
    let usage = r.usage(account);
    for break_root in [true, false] {
        let mut bad = replacement.credential.clone();
        let CredentialProof::Certified(proof) = &mut bad.proof else {
            unreachable!()
        };
        if break_root {
            proof.delegation.signature.signature[0] ^= 1;
        } else {
            proof.issuer_signature_cbor[0] ^= 1;
        }
        assert!(matches!(
            r.admit(
                &bad,
                request(),
                input(&replacement.context(&grants, 181), &role)
            ),
            Err(MemoryError::Proof(_))
        ));
        assert_eq!(r.usage(account), usage);
        assert_eq!(r.known_logins(), 3);
        assert_eq!(
            r.authorize(&initial.read(&grants, &role, 181), &[], |_| panic!(
                "denied caller authorized"
            )),
            Err(MemoryError::Policy(SessionPolicyError::Revoked))
        );
        assert_eq!(
            r.read(&replacement.read(&grants, &role, 181), &[]),
            Err(MemoryError::Policy(SessionPolicyError::MissingOrExpired))
        );
    }
    assert_eq!(r.verification_budget().available(), 24);
    replacement.admit(&mut r, &grants, &role, 181).unwrap();
    assert_eq!(r.verification_budget().available(), 21);
    assert_eq!(r.known_logins(), 3);
    for case in [&replacement, &device, &other] {
        assert!(
            r.authorize(&case.read(&grants, &role, 181), &[], |_| ())
                .is_ok()
        );
    }
}
