//! Portable receiver lifecycle against real canisters and signed ingress.
#[path = "sessions/costs.rs"]
mod costs;
use super::{
    Agent, BasicIdentity, CONTROL_REPLY_LIMITS, Fixture, Principal, decode_one, encode_args,
    encode_one,
};
#[path = "../../../sessions.rs"]
mod contract;
use contract::*;
use ed25519_dalek::{Signer, SigningKey};
use ic_auth::{
    portable::{browser_principal, claims_hash, encode_claims},
    session::*,
};
use ic_auth_protocol_types::*;
use ic_testkit::pic::CandidCallExt;
use sha2::{Digest, Sha256};
use std::time::Duration;
const SECOND: u64 = 1_000_000_000;

// Balance deltas include platform/update charges and fixture work. They are not
// instruction counters, production prices, issuer costs or subnet latency.
fn measured<T>(f: &Fixture, target: Principal, label: &str, operation: impl FnOnce() -> T) -> T {
    let before = f.pic.cycle_balance(target);
    let result = operation();
    let spent = before.checked_sub(f.pic.cycle_balance(target)).unwrap();
    assert!(spent > 0, "an update must consume cycles");
    eprintln!("portable receiver cycles: {label}={spent}");
    result
}

fn install(f: &Fixture, target: Principal, config: &ReceiverConfig) {
    let result: Result<(), String> = f
        .pic
        .update_candid(
            target,
            f.controller,
            "install_session_receiver",
            (config.clone(),),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    result.unwrap();
}
fn admission(
    f: &Fixture,
    target: Principal,
    caller: Principal,
    input: &SessionInput,
) -> Result<SessionView, String> {
    f.pic
        .update_candid(
            target,
            caller,
            "establish_portable_session",
            (input.clone(),),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport")
}
fn read(f: &Fixture, target: Principal, caller: Principal) -> Result<SessionView, String> {
    f.pic
        .update_candid(
            target,
            caller,
            "portable_session_read",
            (),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport")
}
fn logout(f: &Fixture, target: Principal, caller: Principal) -> Result<u64, LogoutResult> {
    let result: LogoutResult = f
        .pic
        .update_candid(
            target,
            caller,
            "logout_portable_session",
            (LogoutRequest {},),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    match result {
        LogoutResult::Revoked { deny_until_ns } => Ok(deny_until_ns),
        other => Err(other),
    }
}
fn input(claims: &PortableClaims, key: &SigningKey) -> SessionInput {
    SessionInput {
        credential: PortableCredential {
            claims: encode_claims(claims).unwrap(),
            proof: PortableProof::Ed25519 {
                signature: key.sign(&claims_hash(claims).unwrap()).to_bytes().to_vec(),
            },
        },
        scopes: vec!["read".into()],
        ttl_secs: None,
    }
}
#[test]
fn portable_sessions_cover_receivers_devices_logout_upgrade_and_signed_ingress() {
    let mut f = Fixture::new();
    eprintln!(
        "portable receiver artifact: bytes={}, sha256={:x}; Testkit application subnet; volatile storage; issuer signing excluded",
        f.wasm.len(),
        Sha256::digest(&f.wasm)
    );
    let now = f.now();
    let key = SigningKey::from_bytes(&[42; 32]);
    let config = ReceiverConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: f.signer,
        authority: ReceiverAuthority::Ed25519(key.verifying_key().to_bytes()),
        role: "user".parse().unwrap(),
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".into()],
        }],
        valid_from_ns: now,
        valid_until_ns: now + 120 * SECOND,
        enrollment_until_ns: now + 120 * SECOND,
        verification_burst: 16,
        verification_refill_ns: 1,
    };
    install(&f, f.signer, &config);
    install(&f, f.resource, &config);
    f.pic.advance_time(Duration::from_secs(1));
    let mut claims = PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: config.audience,
        issuer: config.issuer,
        key_binding_id: config.authority.binding(),
        browser_public_key: SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes(),
        account_subject: Principal::from_slice(&[99]),
        login_started_at_ns: f.now(),
        login_deadline_ns: f.now() + 60 * SECOND,
        issued_at_ns: f.now(),
        admission_expires_at_ns: f.now() + 2 * SECOND,
        nonce: [1; 16],
        grants: config.grants.clone(),
    };
    let caller = browser_principal(&claims.browser_public_key);
    let first = input(&claims, &key);
    let a = measured(&f, f.signer, "fresh_admission_a", || {
        admission(&f, f.signer, caller, &first)
    })
    .unwrap();
    let b = measured(&f, f.resource, "fresh_admission_b", || {
        admission(&f, f.resource, caller, &first)
    })
    .unwrap();
    assert_eq!(
        (a.caller, a.issuer, a.subject),
        (b.caller, b.issuer, b.subject)
    );
    assert!(b.expires_at_ns >= a.expires_at_ns);
    assert!(b.expires_at_ns <= claims.login_deadline_ns);
    let mut other = claims.clone();
    other.browser_public_key = SigningKey::from_bytes(&[2; 32]).verifying_key().to_bytes();
    let other_caller = browser_principal(&other.browser_public_key);
    let other_input = input(&other, &key);
    let mut forged = other_input.clone();
    let PortableProof::Ed25519 { signature } = &mut forged.credential.proof else {
        unreachable!()
    };
    signature[0] ^= 1;
    assert!(
        measured(&f, f.signer, "forged_signature", || {
            admission(&f, f.signer, other_caller, &forged)
        })
        .is_err()
    );
    assert!(read(&f, f.signer, other_caller).is_err());
    measured(&f, f.signer, "fresh_admission_second_device", || {
        admission(&f, f.signer, other_caller, &other_input)
    })
    .unwrap();
    f.pic.advance_time(Duration::from_secs(3));
    let mut retry = first.clone();
    retry.credential.proof = PortableProof::Ed25519 { signature: vec![] };
    assert_eq!(
        measured(&f, f.signer, "exact_retry_after_proof_expiry", || {
            admission(&f, f.signer, caller, &retry)
        })
        .unwrap(),
        a
    );
    let denial = measured(&f, f.signer, "admitted_logout", || {
        logout(&f, f.signer, caller)
    })
    .unwrap();
    assert_eq!(logout(&f, f.signer, caller).unwrap(), denial);
    assert!(read(&f, f.signer, caller).is_err());
    assert!(admission(&f, f.signer, caller, &first).is_err());
    assert!(read(&f, f.signer, other_caller).is_ok());
    assert!(
        measured(&f, f.resource, "ordinary_session_update", || {
            read(&f, f.resource, caller)
        })
        .is_ok()
    );
    logout(&f, f.resource, caller).unwrap();
    let profile: Result<SessionProfile, String> = f
        .pic
        .query_candid(
            f.signer,
            other_caller,
            "profile_portable_session",
            (),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    let profile = profile.unwrap();
    assert!(profile.record_bytes < 2048);
    assert!(profile.decode_instructions > 0);
    assert!(profile.authorization_instructions > 0);
    eprintln!("portable session Wasm profile: {profile:?}");
    let unknown_key = SigningKey::from_bytes(&[3; 32]).verifying_key().to_bytes();
    let unknown = browser_principal(&unknown_key);
    logout(&f, f.signer, unknown).unwrap();
    claims.browser_public_key = unknown_key;
    claims.issued_at_ns = f.now();
    claims.admission_expires_at_ns = f.now() + 2 * SECOND;
    assert!(admission(&f, f.signer, unknown, &input(&claims, &key)).is_err());
    let result: Result<(), String> = f
        .pic
        .update_candid(
            f.signer,
            f.controller,
            "invalidate_session_authority",
            (false,),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    result.unwrap();
    assert!(read(&f, f.signer, other_caller).is_err());
    logout(&f, f.signer, other_caller).unwrap();
    f.pic
        .upgrade_canister(
            f.resource,
            f.wasm.clone(),
            encode_args(()).unwrap(),
            Some(f.controller),
        )
        .unwrap();
    assert_eq!(
        read(&f, f.resource, caller).unwrap_err(),
        "receiver not ready"
    );
    assert_eq!(
        logout(&f, f.resource, caller),
        Err(LogoutResult::Unavailable(LogoutUnavailable::ResetPending))
    );
    install(&f, f.resource, &config);
    install(&f, f.signer, &config);
    f.pic.advance_time(Duration::from_secs(1));
    claims.browser_public_key = SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes();
    claims.issued_at_ns = f.now();
    claims.admission_expires_at_ns = f.now() + 2 * SECOND;
    assert!(
        admission(&f, f.resource, caller, &input(&claims, &key))
            .unwrap_err()
            .contains("reset")
    );
    claims.browser_public_key = SigningKey::from_bytes(&[4; 32]).verifying_key().to_bytes();
    claims.login_started_at_ns = f.now();
    let fresh = browser_principal(&claims.browser_public_key);
    let fresh_session = admission(&f, f.resource, fresh, &input(&claims, &key)).unwrap();
    let peer_session = admission(&f, f.signer, fresh, &input(&claims, &key)).unwrap();
    let url = f.pic.make_live(None).to_string();
    let network = f.pic.root_key().unwrap();
    for device in [4, 5] {
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            let agent = Agent::builder()
                .with_url(&url)
                .with_identity(BasicIdentity::from_raw_key(&[device; 32]))
                .with_ingress_expiry(Duration::from_secs(30))
                .build()
                .unwrap();
            agent.set_root_key(network.clone());
            let bytes = agent
                .update(&f.resource, "portable_session_read")
                .with_arg(encode_args(()).unwrap())
                .call_and_wait()
                .await
                .unwrap();
            decode_one::<Result<SessionView, String>>(&bytes).unwrap()
        });
        if device == 4 {
            assert_eq!(result.unwrap().caller, fresh)
        } else {
            assert!(result.is_err())
        }
    }
    assert_eq!(read(&f, f.resource, fresh).unwrap(), fresh_session);
    assert_eq!(read(&f, f.signer, fresh).unwrap(), peer_session);
    let mut sdk_claims = claims.clone();
    let sdk_now = f.now();
    sdk_claims.browser_public_key = SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes();
    sdk_claims.login_started_at_ns = sdk_now;
    sdk_claims.login_deadline_ns = sdk_now + 90 * SECOND;
    sdk_claims.issued_at_ns = sdk_now;
    sdk_claims.admission_expires_at_ns = sdk_now + 60 * SECOND;
    sdk_claims.nonce = [9; 16];
    let sdk_caller = browser_principal(&sdk_claims.browser_public_key);
    for receiver in [f.signer, f.resource] {
        assert!(read(&f, receiver, sdk_caller).is_err());
    }
    let sdk_credential = encode_one(input(&sdk_claims, &key).credential).unwrap();
    // Node uses the real SDK transport and trusted Testkit network root. The
    // journal uses fake IndexedDB; this does not qualify native browser storage.
    let mut command = std::process::Command::new(
        std::env::var_os("IC_AUTH_QUALIFICATION_NODE")
            .expect("make test-qualification supplies the admitted Node executable"),
    );
    command
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../packages/client/dist/test/qualification/logout.js"
        ))
        .arg(&url)
        .arg(f.signer.to_text())
        .arg(f.resource.to_text())
        .arg(fresh.to_text())
        .arg(
            network
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
        )
        .arg(f.now().to_string())
        .arg(
            sdk_credential
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
        )
        .arg(sdk_claims.account_subject.to_text());
    use ic_host_process::tool::{
        OutputLimit, OutputLimits, capture_command, format_failure_evidence,
    };
    let output = capture_command(
        &mut command,
        OutputLimits {
            stdout: OutputLimit::Terminate(65_536),
            stderr: OutputLimit::Terminate(65_536),
            timeout: Some(Duration::from_secs(30)),
        },
    )
    .and_then(|result| result.require_success())
    .and_then(|result| result.require_complete())
    .unwrap_or_else(|error| panic!("{}", format_failure_evidence("SDK logout fixture", &error)));
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(f.now() < fresh_session.expires_at_ns.min(peer_session.expires_at_ns));
    assert!(f.now() < config.enrollment_until_ns.min(config.valid_until_ns));
    assert!(f.now() < sdk_claims.admission_expires_at_ns);
    sdk_claims.issued_at_ns = f.now();
    sdk_claims.admission_expires_at_ns = f.now() + 2 * SECOND;
    sdk_claims.nonce = [10; 16];
    for receiver in [f.signer, f.resource] {
        assert!(read(&f, receiver, sdk_caller).is_err());
        assert!(admission(&f, receiver, sdk_caller, &input(&sdk_claims, &key)).is_err());
    }
    claims.issued_at_ns = f.now();
    claims.admission_expires_at_ns = f.now() + 2 * SECOND;
    claims.nonce = [4; 16];
    for receiver in [f.signer, f.resource] {
        assert!(read(&f, receiver, fresh).is_err());
        assert!(admission(&f, receiver, fresh, &input(&claims, &key)).is_err());
    }
}

pub(super) fn qualify_certified(
    f: &Fixture,
    token: &DelegatedToken,
    old: &ic_auth::token::TokenVerificationContext<'_>,
) {
    use ic_auth::{
        admission_budget::SessionAdmissionBudget,
        portable::{MemoryLoginRevocations, PortableClaimsContext},
        portable_certified::{CERTIFIED_SIGNATURE_DOMAIN, PortableCertifiedContext},
        session_request::SessionRequest,
    };
    costs::qualify(f, token, old);
    let now = f.now();
    let cert = &token.proof.cert;
    let claims = PortableClaims {
        mechanism: PortableMechanism::Certified,
        audience: old.audience,
        issuer: cert.issuer_pid,
        key_binding_id: ic_auth::canonical::cert_hash(cert).unwrap(),
        browser_public_key: SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes(),
        account_subject: f.owner,
        login_started_at_ns: now,
        login_deadline_ns: now + 30 * SECOND,
        issued_at_ns: now,
        admission_expires_at_ns: now + 2 * SECOND,
        nonce: [1; 16],
        grants: cert.grants.clone(),
    };
    let digest = claims_hash(&claims).unwrap();
    f.prepare(CERTIFIED_SIGNATURE_DOMAIN, &digest, 60 * SECOND)
        .unwrap();
    let signature = f.retrieve(CERTIFIED_SIGNATURE_DOMAIN, &digest).unwrap();
    let now = f.now();
    let authority = PortableClaimsContext {
        audience: &old.audience,
        issuer: cert.issuer_pid,
        mechanism: PortableMechanism::Certified,
        key_binding_id: claims.key_binding_id,
        grants: &cert.grants,
        caller: browser_principal(&claims.browser_public_key),
        now_ns: now,
        authority_not_before_ns: cert.not_before_ns,
        authority_expires_at_ns: cert.expires_at_ns.min(old.root_key.accept_until_ns),
        enrollment_expires_at_ns: cert.expires_at_ns,
        min_login_started_at_ns: 0,
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 86_400 * SECOND,
        max_proof_ttl_ns: 60 * SECOND,
    };
    let mut limits = old.limits;
    limits.max_future_skew_ns = 0;
    let proof_context = PortableCertifiedContext {
        claims: authority,
        root_key: old.root_key,
        ic_root_public_key_raw: old.ic_root_public_key_raw,
        limits,
    };
    let mut ctx = SessionContext {
        authority,
        role: old.role,
        required_scopes: &[],
    };
    let p = SessionPolicy {
        generation: 0,
        enabled: true,
        subject_admissible: true,
        default_ttl_secs: 10,
        max_ttl_secs: 30,
    };
    let mut store = MemorySessionStore::new(
        0,
        SessionLimits {
            max_sessions: 1,
            max_replays: 2,
            max_replays_per_caller: 2,
            max_encoded_bytes: 8192,
        },
    )
    .unwrap();
    let mut ledger = MemoryLoginRevocations::new(old.audience, 1, 1, 0, 0).unwrap();
    let mut budget = SessionAdmissionBudget::new(8, 1, 0).unwrap();
    let bytes = encode_claims(&claims).unwrap();
    let credential = PortableCredential {
        claims: bytes.clone(),
        proof: PortableProof::Certified {
            delegation: Box::new(token.proof.clone()),
            issuer_signature: signature,
        },
    };
    let request = SessionRequest::new(vec!["read".into()], None).unwrap();
    establish_session(
        &mut store,
        &mut ledger,
        &mut budget,
        &bytes,
        &request,
        &ctx,
        p,
        |budget, decoded| {
            Ok(decoded.verify(
                budget,
                &credential.proof,
                &ic_auth::portable_credential::PortableVerificationContext::Certified(
                    &proof_context,
                ),
            )?)
        },
    )
    .unwrap();
    f.pic.advance_time(Duration::from_secs(3));
    ctx.authority.now_ns = f.now();
    assert!(matches!(
        establish_session(
            &mut store,
            &mut ledger,
            &mut budget,
            &bytes,
            &request,
            &ctx,
            p,
            |_, _| panic!("retry must not verify")
        ),
        Ok(SessionEstablished::ExactRetry(_))
    ));
    assert!(authorize_session(&mut store, &mut ledger, &ctx, p).is_ok());
    assert!(matches!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut budget,
            authority.caller,
            ctx.authority.now_ns,
        ),
        LogoutResult::Revoked { .. }
    ));
    assert!(authorize_session(&mut store, &mut ledger, &ctx, p).is_err());
}

#[test]
fn checkpoint_upgrade_preserves_sessions_denials_and_requires_fresh_authority() {
    let f = Fixture::new();
    let now = f.now();
    let key = SigningKey::from_bytes(&[42; 32]);
    let config = ReceiverConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: f.signer,
        authority: ReceiverAuthority::Ed25519(key.verifying_key().to_bytes()),
        role: "user".parse().unwrap(),
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".into()],
        }],
        valid_from_ns: now,
        valid_until_ns: now + 120 * SECOND,
        enrollment_until_ns: now + 120 * SECOND,
        verification_burst: 16,
        verification_refill_ns: 1,
    };
    install(&f, f.resource, &config);
    install(&f, f.signer, &config);
    let enabled: Result<(), String> = f
        .pic
        .update_candid(
            f.resource,
            f.controller,
            "enable_session_checkpoint",
            (),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    enabled.unwrap();
    f.pic.advance_time(Duration::from_secs(1));
    let mut claims = PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: config.audience,
        issuer: config.issuer,
        key_binding_id: config.authority.binding(),
        browser_public_key: SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes(),
        account_subject: Principal::from_slice(&[99]),
        login_started_at_ns: f.now(),
        login_deadline_ns: f.now() + 60 * SECOND,
        issued_at_ns: f.now(),
        admission_expires_at_ns: f.now() + 2 * SECOND,
        nonce: [1; 16],
        grants: config.grants.clone(),
    };
    let live = browser_principal(&claims.browser_public_key);
    let original = input(&claims, &key);
    let session = admission(&f, f.resource, live, &original).unwrap();
    let peer_session = admission(&f, f.signer, live, &original).unwrap();
    claims.browser_public_key = SigningKey::from_bytes(&[2; 32]).verifying_key().to_bytes();
    let revoked = browser_principal(&claims.browser_public_key);
    admission(&f, f.resource, revoked, &input(&claims, &key)).unwrap();
    admission(&f, f.signer, revoked, &input(&claims, &key)).unwrap();
    let denied_until = logout(&f, f.resource, revoked).unwrap();
    assert!(read(&f, f.signer, revoked).is_ok());
    let unknown_key = SigningKey::from_bytes(&[3; 32]).verifying_key().to_bytes();
    let unknown = browser_principal(&unknown_key);
    let unknown_until = logout(&f, f.resource, unknown).unwrap();
    f.pic.advance_time(Duration::from_secs(3));
    assert!(f.now() >= claims.admission_expires_at_ns);
    f.pic
        .upgrade_canister(
            f.resource,
            f.wasm.clone(),
            encode_args(()).unwrap(),
            Some(f.controller),
        )
        .unwrap();
    assert_eq!(
        read(&f, f.resource, live).unwrap_err(),
        "receiver not ready"
    );
    assert_eq!(read(&f, f.signer, live).unwrap(), peer_session);
    assert!(read(&f, f.signer, revoked).is_ok());
    assert!(admission(&f, f.resource, live, &original).is_err());
    assert_eq!(
        logout(&f, f.resource, live),
        Err(LogoutResult::Unavailable(LogoutUnavailable::ResetPending))
    );
    let unauthorized: Result<(), String> = f
        .pic
        .update_candid(
            f.resource,
            live,
            "confirm_restored_session_authority",
            (config.clone(),),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    assert!(unauthorized.is_err());
    let mut mismatch = config.clone();
    mismatch.authority = ReceiverAuthority::Ed25519([7; 32]);
    let wrong: Result<(), String> = f
        .pic
        .update_candid(
            f.resource,
            f.controller,
            "confirm_restored_session_authority",
            (mismatch,),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    assert!(wrong.is_err());
    assert!(read(&f, f.resource, live).is_err());
    let confirmed: Result<(), String> = f
        .pic
        .update_candid(
            f.resource,
            f.controller,
            "confirm_restored_session_authority",
            (config.clone(),),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    confirmed.unwrap();
    assert_eq!(read(&f, f.resource, live).unwrap(), session);
    // Restoration retains the live exact retry and its original lease even
    // though the proof expired. It must not enter fresh verification.
    let mut retry = original.clone();
    retry.credential.proof = PortableProof::Ed25519 { signature: vec![] };
    assert_eq!(admission(&f, f.resource, live, &retry).unwrap(), session);
    assert!(read(&f, f.resource, revoked).is_err());
    assert_eq!(logout(&f, f.resource, revoked).unwrap(), denied_until);
    assert_eq!(logout(&f, f.resource, unknown).unwrap(), unknown_until);
    // Newly signed credentials for both revoked logins remain denied after boot.
    claims.issued_at_ns = f.now();
    claims.admission_expires_at_ns = f.now() + 2 * SECOND;
    claims.nonce = [2; 16];
    let renewed = input(&claims, &key);
    assert!(admission(&f, f.resource, revoked, &renewed).is_err());
    let renewed_peer = admission(&f, f.signer, revoked, &renewed).unwrap();
    assert_eq!(renewed_peer.caller, revoked);
    assert_eq!(renewed_peer.subject, session.subject);
    claims.browser_public_key = unknown_key;
    assert!(admission(&f, f.resource, unknown, &input(&claims, &key)).is_err());
    let disabled: Result<(), String> = f
        .pic
        .update_candid(
            f.resource,
            f.controller,
            "invalidate_session_authority",
            (false,),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    disabled.unwrap();
    f.pic
        .upgrade_canister(
            f.resource,
            f.wasm.clone(),
            encode_args(()).unwrap(),
            Some(f.controller),
        )
        .unwrap();
    let confirmed: Result<(), String> = f
        .pic
        .update_candid(
            f.resource,
            f.controller,
            "confirm_restored_session_authority",
            (config,),
            CONTROL_REPLY_LIMITS,
        )
        .expect("qualified Candid transport");
    confirmed.unwrap();
    assert!(read(&f, f.resource, live).is_err());
    assert_eq!(logout(&f, f.resource, revoked).unwrap(), denied_until);
    assert_eq!(read(&f, f.signer, live).unwrap(), peer_session);
    assert_eq!(read(&f, f.signer, revoked).unwrap(), renewed_peer);
    logout(&f, f.signer, revoked).unwrap();
    assert!(read(&f, f.signer, revoked).is_err());
    assert_eq!(read(&f, f.signer, live).unwrap(), peer_session);
    assert_eq!(logout(&f, f.resource, revoked).unwrap(), denied_until);
}
