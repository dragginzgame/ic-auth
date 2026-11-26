// Shared signed-fixture driver, never the sequence model oracle.
use super::*;

pub(super) fn bytes(name: &str) -> Vec<u8> {
    let hex = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/memory.tsv"
    ))
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

pub(super) fn credential(name: &str) -> Credential {
    Credential {
        claims: bytes(&format!("{name}-claims")),
        proof: CredentialProof::Ed25519(bytes(&format!("{name}-signature"))),
    }
}

pub(super) fn request() -> SessionRequest {
    SessionRequest::new(vec!["read".into()], None).unwrap()
}

pub(super) fn account() -> (Principal, Principal) {
    (Principal::from_slice(&[1]), Principal::from_slice(&[2]))
}

///
/// Host
///
/// Supplies the audience, authority and signed fixtures for receiver tests.
///

pub(super) struct Host {
    pub(super) audience: AudienceId,
    pub(super) grants: AuthorityGrants,
    pub(super) role: AuthRole,
    pub(super) key: Ed25519Key,
}

impl Host {
    pub(super) fn new() -> Self {
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

    pub(super) fn context(&self, name: &str, now_ns: u64) -> VerificationContext<'_> {
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

    pub(super) fn admission<'a>(
        &'a self,
        context: &'a VerificationContext<'a>,
    ) -> AdmissionHost<'a> {
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

    pub(super) fn read(&self, name: &str, now_ns: u64) -> SessionAuthority<'_> {
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

    pub(super) fn config(&self) -> MemoryConfig {
        MemoryConfig {
            enrollment: EnrollmentBinding {
                receiver: Principal::from_slice(&[10]),
                authority: Principal::from_slice(&[11]),
            },
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

    pub(super) fn receiver(&self) -> FixtureReceiver {
        let mut r = fresh_receiver(self.config(), 0).unwrap();
        self.enroll(&mut r, 7).unwrap();
        r
    }

    pub(super) fn snapshot(&self, generation: u64) -> EnrollmentMetadata {
        EnrollmentMetadata {
            receiver: self.config().enrollment.receiver,
            audience: self.audience,
            challenge: [1; 32],
            revision: 1,
            generation,
            snapshot_digest: [2; 32],
            authority_digest: [3; 32],
            issued_at_ns: 0,
            expires_at_ns: 150,
        }
    }

    pub(super) fn enroll(&self, r: &mut MemoryReceiver, generation: u64) -> Result<(), E> {
        let now = r.observed_at_ns();
        r.request_enrollment(now, [1; 32], now + 10)?;
        r.install_enrollment(
            now,
            self.config().enrollment.authority,
            self.snapshot(generation),
        )
    }
}
