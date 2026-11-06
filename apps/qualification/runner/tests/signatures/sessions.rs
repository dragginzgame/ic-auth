//! Portable receiver lifecycle against real canisters and signed ingress.
mod application;
mod browser;
mod costs;
use super::{Agent, BasicIdentity, Fixture, Principal};
use super::{controls, query_control, update_control};
use contract::*;
use ic_auth::{
    portable::{browser_principal, claims_hash, encode_claims},
    session::*,
};
use ic_auth_protocol_types::*;
use ic_auth_qualification_fixtures::sessions as contract;
use ic_auth_wire::{
    SessionInput, SessionView,
    model::AdmissionRefusal,
    wire::{Bytes, List, Text},
};
use ic_platform::ed25519::{Signer, SigningKey};
use ic_testkit::pic::update_wire;
use sha2::{Digest, Sha256};
use std::time::Duration;
const SECOND: u64 = 1_000_000_000;

// Balance deltas include platform/update charges and fixture work. They are not
// instruction counters, production prices, issuer costs or subnet latency.
async fn measured<T>(
    f: &Fixture,
    target: Principal,
    label: &str,
    operation: impl AsyncFnOnce() -> T,
) -> T {
    let before = f.pic.instance().cycle_balance(target.into()).await.unwrap();
    let result = operation().await;
    let spent = before
        .checked_sub(f.pic.instance().cycle_balance(target.into()).await.unwrap())
        .unwrap();
    assert!(spent > 0, "an update must consume cycles");
    eprintln!("portable receiver cycles: {label}={spent}");
    result
}

async fn install(f: &Fixture, target: Principal, config: &ReceiverConfig) {
    let result: Result<(), controls::Refusal> =
        update_control::<controls::model::ReceiverConfig, ()>(
            f.pic.instance(),
            target,
            f.controller,
            spec!(controls::InstallReceiver),
            (config.clone()).try_into().unwrap(),
        )
        .await;
    result.unwrap();
}
async fn admission(
    f: &Fixture,
    target: Principal,
    caller: Principal,
    input: &SessionInput,
) -> Result<SessionView, AdmissionRefusal> {
    update_wire::<
        ic_auth_wire::SessionInput,
        Result<ic_auth_wire::SessionView, ic_auth_wire::model::AdmissionRefusal>,
    >(
        f.pic.instance(),
        target.into(),
        caller.into(),
        spec!(ic_auth_wire::EstablishSession),
        input,
    )
    .await
    .expect("qualified Wire admission")
}
async fn read(
    f: &Fixture,
    target: Principal,
    caller: Principal,
) -> Result<SessionView, SessionReadRefusal> {
    update_wire::<(), Result<ic_auth_wire::SessionView, ic_auth_wire::model::SessionReadRefusal>>(
        f.pic.instance(),
        target.into(),
        caller.into(),
        spec!(ic_auth_wire::ReadSession),
        &(),
    )
    .await
    .expect("qualified Wire session read")
    .map_err(Into::into)
}
async fn logout(f: &Fixture, target: Principal, caller: Principal) -> Result<u64, LogoutResult> {
    use ic_auth_wire::model as m;
    match update_wire::<(), ic_auth_wire::model::LogoutResult>(
        f.pic.instance(),
        target.into(),
        caller.into(),
        spec!(ic_auth_wire::Logout),
        &(),
    )
    .await
    .expect("qualified Wire logout")
    {
        m::LogoutResult::Revoked(until) => Ok(until),
        m::LogoutResult::Unconfirmed(deny_until_ns) => {
            Err(LogoutResult::Unconfirmed { deny_until_ns })
        }
        m::LogoutResult::InvalidCaller => Err(LogoutResult::InvalidCaller),
        m::LogoutResult::Unavailable(reason) => Err(LogoutResult::Unavailable(reason.into())),
    }
}

fn input(claims: &PortableClaims, key: &SigningKey) -> SessionInput {
    SessionInput {
        credential: PortableCredential {
            claims: encode_claims(claims).unwrap(),
            proof: PortableProof::Ed25519 {
                signature: key.sign(&claims_hash(claims).unwrap()).to_bytes().to_vec(),
            },
        }
        .try_into()
        .unwrap(),
        scopes: List::new(vec![Text::new("read".into()).unwrap()]).unwrap(),
        ttl_secs: None,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sdk_preserves_typed_refusals_and_retries_the_original_future_credential() {
    let mut f = Fixture::new().await;
    let key = SigningKey::from_bytes(&[42; 32]);
    let now = f.now().await;
    let config = ReceiverConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: f.signer,
        authority: ReceiverAuthority::Ed25519(Box::new(
            ic_auth::portable_signature::PreparedIssuerKey::new(key.verifying_key().to_bytes())
                .unwrap(),
        )),
        role: "user".parse().unwrap(),
        grants: ic_auth::portable::ValidatedGrants::new(vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }])
        .unwrap(),
        valid_from_ns: now,
        valid_until_ns: now + 120 * SECOND,
        enrollment_until_ns: now + 120 * SECOND,
        verification_burst: 16,
        verification_refill_ns: 1,
    };
    install(&f, f.signer, &config).await;
    install(
        &f,
        f.resource,
        &ReceiverConfig {
            enrollment_until_ns: now + SECOND,
            ..config.clone()
        },
    )
    .await;
    f.pic
        .instance()
        .advance_time(Duration::from_secs(1))
        .await
        .unwrap();
    let now = f.now().await;
    let valid = PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: config.audience,
        issuer: config.issuer,
        key_binding_id: config.authority.binding(),
        browser_public_key: SigningKey::from_bytes(&[71; 32]).verifying_key().to_bytes(),
        account_subject: Principal::from_slice(&[99]),
        login_started_at_ns: now,
        login_deadline_ns: now + 90 * SECOND,
        issued_at_ns: now,
        admission_expires_at_ns: now + 30 * SECOND,
        nonce: [71; 16],
        grants: config.grants.as_slice().to_vec(),
    };
    let future = input(
        &PortableClaims {
            issued_at_ns: now + 10 * SECOND,
            ..valid.clone()
        },
        &key,
    );
    let expired = input(
        &PortableClaims {
            admission_expires_at_ns: now + 1,
            ..valid.clone()
        },
        &key,
    );
    let mut invalid = input(&valid, &key);
    invalid.credential.proof =
        ic_auth_wire::model::PortableProof::Ed25519(Bytes::new(vec![0; 64]).unwrap());
    let url = f
        .pic
        .instance_mut()
        .make_live(None)
        .await
        .unwrap()
        .to_string();
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let mut command = std::process::Command::new(
        std::env::var_os("IC_AUTH_QUALIFICATION_NODE").expect("admitted Node executable"),
    );
    command
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../packages/client/dist/test/qualification/admission-refusals.js"
        ))
        .arg(&url)
        .arg(f.signer.to_text())
        .arg(f.resource.to_text())
        .arg(hex(&f.root_key().await));
    for value in [&future, &expired, &invalid] {
        let frame = ic_auth_wire::CredentialEnvelope::encode_request(&value.credential).unwrap();
        command.arg(hex(frame.as_slice()));
    }
    eprintln!(
        "{}",
        String::from_utf8_lossy(&run_node(&mut command, "SDK typed admission refusals"))
    );
    assert!(
        read(&f, f.signer, browser_principal(&valid.browser_public_key))
            .await
            .is_ok()
    );
    assert!(
        read(&f, f.resource, browser_principal(&valid.browser_public_key))
            .await
            .is_err()
    );
    f.pic
        .close()
        .await
        .expect("delete qualification instance before stopping its server");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issuer_scoped_accounts_survive_wire_projection_and_independent_logout() {
    let f = Fixture::new().await;
    let first_key = SigningKey::from_bytes(&[42; 32]);
    let second_key = SigningKey::from_bytes(&[43; 32]);
    let now = f.now().await;
    let first = ReceiverConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: f.signer,
        authority: ReceiverAuthority::Ed25519(Box::new(
            ic_auth::portable_signature::PreparedIssuerKey::new(
                first_key.verifying_key().to_bytes(),
            )
            .unwrap(),
        )),
        role: "user".parse().unwrap(),
        grants: ic_auth::portable::ValidatedGrants::new(vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }])
        .unwrap(),
        valid_from_ns: now,
        valid_until_ns: now + 120 * SECOND,
        enrollment_until_ns: now + 120 * SECOND,
        verification_burst: 16,
        verification_refill_ns: 1,
    };
    let second = ReceiverConfig {
        issuer: f.resource,
        authority: ReceiverAuthority::Ed25519(Box::new(
            ic_auth::portable_signature::PreparedIssuerKey::new(
                second_key.verifying_key().to_bytes(),
            )
            .unwrap(),
        )),
        ..first.clone()
    };
    install(&f, f.signer, &first).await;
    install(&f, f.resource, &second).await;
    f.pic
        .instance()
        .advance_time(Duration::from_secs(1))
        .await
        .unwrap();
    let first_claims = PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: first.audience,
        issuer: first.issuer,
        key_binding_id: first.authority.binding(),
        browser_public_key: SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes(),
        account_subject: Principal::from_slice(&[99]),
        login_started_at_ns: f.now().await,
        login_deadline_ns: f.now().await + 60 * SECOND,
        issued_at_ns: f.now().await,
        admission_expires_at_ns: f.now().await + 5 * SECOND,
        nonce: [1; 16],
        grants: first.grants.as_slice().to_vec(),
    };
    let second_claims = PortableClaims {
        issuer: second.issuer,
        key_binding_id: second.authority.binding(),
        browser_public_key: SigningKey::from_bytes(&[2; 32]).verifying_key().to_bytes(),
        ..first_claims.clone()
    };
    let first_caller = browser_principal(&first_claims.browser_public_key);
    let second_caller = browser_principal(&second_claims.browser_public_key);
    let a = admission(
        &f,
        f.signer,
        first_caller,
        &input(&first_claims, &first_key),
    )
    .await
    .unwrap();
    let b = admission(
        &f,
        f.resource,
        second_caller,
        &input(&second_claims, &second_key),
    )
    .await
    .unwrap();
    assert_eq!(a.subject, b.subject);
    assert_ne!(a.issuer, b.issuer);
    assert_ne!(a.caller, b.caller);
    assert_eq!(read(&f, f.signer, first_caller).await.unwrap(), a);
    assert_eq!(read(&f, f.resource, second_caller).await.unwrap(), b);
    assert!(
        admission(
            &f,
            f.signer,
            second_caller,
            &input(&second_claims, &second_key),
        )
        .await
        .is_err()
    );
    assert!(read(&f, f.signer, second_caller).await.is_err());
    assert_eq!(read(&f, f.signer, first_caller).await.unwrap(), a);
    logout(&f, f.signer, first_caller).await.unwrap();
    assert!(read(&f, f.signer, first_caller).await.is_err());
    assert_eq!(read(&f, f.resource, second_caller).await.unwrap(), b);
    f.pic
        .close()
        .await
        .expect("delete qualification instance before stopping its server");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn portable_sessions_cover_receivers_devices_logout_upgrade_and_signed_ingress() {
    let mut f = Fixture::with_application_subnets(2).await;
    assert_ne!(
        f.pic.instance().get_subnet(f.signer.into()).await.unwrap(),
        f.pic
            .instance()
            .get_subnet(f.resource.into())
            .await
            .unwrap()
    );
    eprintln!(
        "portable receiver artifact: bytes={}, sha256={:x}; two Testkit application subnets; volatile storage; issuer signing excluded",
        f.wasm.len(),
        Sha256::digest(&f.wasm)
    );
    let now = f.now().await;
    let key = SigningKey::from_bytes(&[42; 32]);
    let config = ReceiverConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: f.signer,
        authority: ReceiverAuthority::Ed25519(Box::new(
            ic_auth::portable_signature::PreparedIssuerKey::new(key.verifying_key().to_bytes())
                .unwrap(),
        )),
        role: "user".parse().unwrap(),
        grants: ic_auth::portable::ValidatedGrants::new(vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }])
        .unwrap(),
        valid_from_ns: now,
        valid_until_ns: now + 120 * SECOND,
        enrollment_until_ns: now + 120 * SECOND,
        verification_burst: 16,
        verification_refill_ns: 1,
    };
    install(&f, f.signer, &config).await;
    install(&f, f.resource, &config).await;
    for malformed in [
        vec![0],
        ic_auth_wire::ReadSession::encode_request(&()).unwrap(),
        vec![0; ic_auth_wire::EstablishSession::REQUEST_LIMITS.bytes + 1],
    ] {
        assert!(
            f.pic
                .instance()
                .update_call(
                    f.resource.into(),
                    f.owner.into(),
                    ic_auth_wire::EstablishSession::NAME,
                    malformed
                )
                .await
                .is_err()
        );
        assert!(read(&f, f.resource, f.owner).await.is_err());
    }
    f.pic
        .instance()
        .advance_time(Duration::from_secs(1))
        .await
        .unwrap();
    let mut claims = PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: config.audience,
        issuer: config.issuer,
        key_binding_id: config.authority.binding(),
        browser_public_key: SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes(),
        account_subject: Principal::from_slice(&[99]),
        login_started_at_ns: f.now().await,
        login_deadline_ns: f.now().await + 60 * SECOND,
        issued_at_ns: f.now().await,
        admission_expires_at_ns: f.now().await + 2 * SECOND,
        nonce: [1; 16],
        grants: config.grants.as_slice().to_vec(),
    };
    let caller = browser_principal(&claims.browser_public_key);
    let first = input(&claims, &key);
    let a = measured(&f, f.signer, "fresh_admission_a", async || {
        admission(&f, f.signer, caller, &first).await
    })
    .await
    .unwrap();
    let b = measured(&f, f.resource, "fresh_admission_b", async || {
        admission(&f, f.resource, caller, &first).await
    })
    .await
    .unwrap();
    assert_eq!(
        (&a.caller, &a.issuer, &a.subject),
        (&b.caller, &b.issuer, &b.subject)
    );
    assert!(b.expires_at_ns >= a.expires_at_ns);
    assert!(b.expires_at_ns <= claims.login_deadline_ns);
    let mut other = claims.clone();
    other.browser_public_key = SigningKey::from_bytes(&[2; 32]).verifying_key().to_bytes();
    let other_caller = browser_principal(&other.browser_public_key);
    let other_input = input(&other, &key);
    let mut forged = other_input.clone();
    let ic_auth_wire::model::PortableProof::Ed25519(signature) = &mut forged.credential.proof
    else {
        unreachable!()
    };
    let mut bytes = signature.as_slice().to_vec();
    bytes[0] ^= 1;
    *signature = Bytes::new(bytes).unwrap();
    assert!(
        measured(&f, f.signer, "forged_signature", async || {
            admission(&f, f.signer, other_caller, &forged).await
        })
        .await
        .is_err()
    );
    assert!(read(&f, f.signer, other_caller).await.is_err());
    measured(&f, f.signer, "fresh_admission_second_device", async || {
        admission(&f, f.signer, other_caller, &other_input).await
    })
    .await
    .unwrap();
    f.pic
        .instance()
        .advance_time(Duration::from_secs(3))
        .await
        .unwrap();
    let mut retry = first.clone();
    retry.credential.proof =
        ic_auth_wire::model::PortableProof::Ed25519(Bytes::new(vec![]).unwrap());
    assert_eq!(
        measured(&f, f.signer, "exact_retry_after_proof_expiry", async || {
            admission(&f, f.signer, caller, &retry).await
        })
        .await
        .unwrap(),
        a
    );
    let denial = measured(&f, f.signer, "admitted_logout", async || {
        logout(&f, f.signer, caller).await
    })
    .await
    .unwrap();
    assert_eq!(logout(&f, f.signer, caller).await.unwrap(), denial);
    assert!(read(&f, f.signer, caller).await.is_err());
    assert!(admission(&f, f.signer, caller, &first).await.is_err());
    assert!(read(&f, f.signer, other_caller).await.is_ok());
    assert!(
        measured(&f, f.resource, "ordinary_session_update", async || {
            read(&f, f.resource, caller).await
        })
        .await
        .is_ok()
    );
    logout(&f, f.resource, caller).await.unwrap();
    let profile: Result<SessionProfile, controls::Refusal> =
        query_control::<(), controls::model::SessionProfile>(
            f.pic.instance(),
            f.signer,
            other_caller,
            spec!(controls::ProfileSession),
            (),
        )
        .await;
    let profile = profile.unwrap();
    assert!(profile.record_bytes < 2048);
    assert!(profile.decode_instructions > 0);
    assert!(profile.authorization_instructions > 0);
    eprintln!("portable session Wasm profile: {profile:?}");
    let unknown_key = SigningKey::from_bytes(&[3; 32]).verifying_key().to_bytes();
    let unknown = browser_principal(&unknown_key);
    assert!(matches!(
        logout(&f, f.signer, unknown).await,
        Err(LogoutResult::Unconfirmed { .. })
    ));
    claims.browser_public_key = unknown_key;
    claims.issued_at_ns = f.now().await;
    claims.admission_expires_at_ns = f.now().await + 2 * SECOND;
    assert!(
        admission(&f, f.signer, unknown, &input(&claims, &key))
            .await
            .is_err()
    );
    let result: Result<(), controls::Refusal> = update_control::<bool, ()>(
        f.pic.instance(),
        f.signer,
        f.controller,
        spec!(controls::InvalidateAuthority),
        false,
    )
    .await;
    result.unwrap();
    assert!(read(&f, f.signer, other_caller).await.is_err());
    logout(&f, f.signer, other_caller).await.unwrap();
    let enabled: Result<(), controls::Refusal> = update_control::<bool, ()>(
        f.pic.instance(),
        f.signer,
        f.controller,
        spec!(controls::InvalidateAuthority),
        true,
    )
    .await;
    enabled.unwrap();
    claims.browser_public_key = SigningKey::from_bytes(&[4; 32]).verifying_key().to_bytes();
    claims.login_started_at_ns = f.now().await;
    claims.login_deadline_ns = claims.login_started_at_ns + 90 * SECOND;
    claims.issued_at_ns = claims.login_started_at_ns;
    claims.admission_expires_at_ns = claims.issued_at_ns + 2 * SECOND;
    let fresh = browser_principal(&claims.browser_public_key);
    // This device must outlive signed ingress plus the bounded SDK child on
    // slower hosts. Request the existing maximum without changing default TTLs;
    // the final pre-expiry assertion still distinguishes revocation from expiry.
    let mut fresh_input = input(&claims, &key);
    fresh_input.ttl_secs = Some(60);
    let fresh_session = admission(&f, f.resource, fresh, &fresh_input)
        .await
        .unwrap();
    let peer_session = admission(&f, f.signer, fresh, &fresh_input).await.unwrap();
    let url = f
        .pic
        .instance_mut()
        .make_live(None)
        .await
        .unwrap()
        .to_string();
    let network = f.root_key().await;
    for device in [4, 5] {
        let result = async {
            let agent = Agent::builder()
                .with_url(&url)
                .with_identity(BasicIdentity::from_raw_key(&[device; 32]))
                .with_ingress_expiry(Duration::from_secs(30))
                .build()
                .unwrap();
            agent.set_root_key(network.clone());
            let bytes = agent
                .update(&f.resource.into(), "portable_session_read")
                .with_arg(ic_auth_wire::ReadSession::encode_request(&()).unwrap())
                .call_and_wait()
                .await
                .unwrap();
            ic_auth_wire::ReadSession::decode_response(&bytes).unwrap()
        }
        .await;
        if device == 4 {
            assert_eq!(result.unwrap().caller.as_slice(), fresh.as_slice())
        } else {
            assert!(result.is_err())
        }
    }
    assert_eq!(read(&f, f.resource, fresh).await.unwrap(), fresh_session);
    assert_eq!(read(&f, f.signer, fresh).await.unwrap(), peer_session);
    let mut sdk_claims = claims.clone();
    let sdk_now = f.now().await;
    sdk_claims.browser_public_key = SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes();
    sdk_claims.account_subject =
        browser_principal(&SigningKey::from_bytes(&[81; 32]).verifying_key().to_bytes());
    sdk_claims.login_started_at_ns = sdk_now;
    sdk_claims.login_deadline_ns = sdk_now + 90 * SECOND;
    sdk_claims.issued_at_ns = sdk_now;
    sdk_claims.admission_expires_at_ns = sdk_now + 60 * SECOND;
    sdk_claims.nonce = [9; 16];
    let sdk_caller = browser_principal(&sdk_claims.browser_public_key);
    for receiver in [f.signer, f.resource] {
        assert!(read(&f, receiver, sdk_caller).await.is_err());
    }
    let sdk_credential =
        ic_auth_wire::CredentialEnvelope::encode_request(&input(&sdk_claims, &key).credential)
            .unwrap();
    let mut sdk_peer_claims = sdk_claims.clone();
    sdk_peer_claims.browser_public_key =
        SigningKey::from_bytes(&[10; 32]).verifying_key().to_bytes();
    sdk_peer_claims.nonce = [10; 16];
    let sdk_peer_caller = browser_principal(&sdk_peer_claims.browser_public_key);
    let sdk_peer_credential =
        ic_auth_wire::CredentialEnvelope::encode_request(&input(&sdk_peer_claims, &key).credential)
            .unwrap();
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
        .arg(f.now().await.to_string())
        .arg(
            sdk_credential
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
        )
        .arg(sdk_claims.account_subject.to_text())
        .arg(sdk_claims.admission_expires_at_ns.to_string())
        .arg(
            sdk_peer_credential
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
        );
    eprintln!(
        "{}",
        String::from_utf8_lossy(&run_node(&mut command, "SDK logout fixture"))
    );
    assert!(f.now().await < fresh_session.expires_at_ns.min(peer_session.expires_at_ns));
    assert!(f.now().await < config.enrollment_until_ns.min(config.valid_until_ns));
    assert!(f.now().await < sdk_claims.admission_expires_at_ns);
    sdk_claims.issued_at_ns = f.now().await;
    sdk_claims.admission_expires_at_ns = f.now().await + 2 * SECOND;
    sdk_claims.nonce = [10; 16];
    sdk_peer_claims.issued_at_ns = f.now().await;
    sdk_peer_claims.admission_expires_at_ns = f.now().await + 2 * SECOND;
    sdk_peer_claims.nonce = [11; 16];
    for receiver in [f.signer, f.resource] {
        assert!(read(&f, receiver, sdk_caller).await.is_err());
        assert!(
            admission(&f, receiver, sdk_caller, &input(&sdk_claims, &key))
                .await
                .is_err()
        );
        assert!(read(&f, receiver, sdk_peer_caller).await.is_err());
        assert!(
            admission(
                &f,
                receiver,
                sdk_peer_caller,
                &input(&sdk_peer_claims, &key)
            )
            .await
            .is_err()
        );
    }
    claims.issued_at_ns = f.now().await;
    claims.admission_expires_at_ns = f.now().await + 2 * SECOND;
    claims.nonce = [4; 16];
    for receiver in [f.signer, f.resource] {
        assert!(read(&f, receiver, fresh).await.is_err());
        assert!(
            admission(&f, receiver, fresh, &input(&claims, &key))
                .await
                .is_err()
        );
    }
    f.pic
        .instance()
        .upgrade_canister(
            f.resource.into(),
            f.wasm.clone(),
            controls::Upgrade::encode_request(&()).unwrap(),
            Some(f.controller.into()),
        )
        .await
        .unwrap();
    assert_eq!(
        read(&f, f.resource, caller).await.unwrap_err(),
        SessionReadRefusal::Unavailable
    );
    assert_eq!(
        logout(&f, f.resource, caller).await,
        Err(LogoutResult::Unavailable(LogoutUnavailable::ResetPending))
    );
    let reinstall: Result<(), controls::Refusal> =
        update_control::<controls::model::ReceiverConfig, ()>(
            f.pic.instance(),
            f.resource,
            f.controller,
            spec!(controls::InstallReceiver),
            config.clone().try_into().unwrap(),
        )
        .await;
    assert_eq!(reinstall, Err(controls::Refusal::ReceiverUnavailable));
    let confirm: Result<(), controls::Refusal> =
        update_control::<controls::model::ReceiverConfig, ()>(
            f.pic.instance(),
            f.resource,
            f.controller,
            spec!(controls::ConfirmAuthority),
            config.clone().try_into().unwrap(),
        )
        .await;
    assert_eq!(confirm, Err(controls::Refusal::AwaitingAuthority));
    assert_eq!(
        admission(&f, f.resource, fresh, &fresh_input).await,
        Err(AdmissionRefusal::Unavailable)
    );
    // A second upgrade must persist the closed marker, not reinterpret absence
    // of an active receiver as a never-configured installation.
    f.pic
        .instance()
        .upgrade_canister(
            f.resource.into(),
            f.wasm.clone(),
            controls::Upgrade::encode_request(&()).unwrap(),
            Some(f.controller.into()),
        )
        .await
        .unwrap();
    let reinstall: Result<(), controls::Refusal> =
        update_control::<controls::model::ReceiverConfig, ()>(
            f.pic.instance(),
            f.resource,
            f.controller,
            spec!(controls::InstallReceiver),
            config.clone().try_into().unwrap(),
        )
        .await;
    assert_eq!(reinstall, Err(controls::Refusal::ReceiverUnavailable));
    f.pic
        .close()
        .await
        .expect("delete qualification instance before stopping its server");
}

fn run_node(command: &mut std::process::Command, label: &str) -> Vec<u8> {
    use ic_host_process::tool::{
        OutputLimit, OutputLimits, capture_command, format_failure_evidence,
    };
    let output = capture_command(
        command,
        OutputLimits {
            stdout: OutputLimit::Terminate(65_536),
            stderr: OutputLimit::Terminate(65_536),
            timeout: Some(Duration::from_secs(30)),
        },
    )
    .and_then(|result| result.require_success())
    .and_then(|result| result.require_complete())
    .unwrap_or_else(|error| panic!("{}", format_failure_evidence(label, &error)));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    output.stdout
}

pub(super) async fn qualify_certified(
    f: &Fixture,
    proof: &DelegationProof,
    old: &ic_auth::portable_certified::PortableCertifiedContext<'_>,
    role: &AuthRole,
) {
    use ic_auth::{
        admission_budget::SessionAdmissionBudget,
        portable::{MemoryLoginRevocations, PortableClaimsContext},
        portable_certified::{CERTIFIED_SIGNATURE_DOMAIN, PortableCertifiedContext},
        session_request::SessionRequest,
    };
    costs::qualify(f, proof, old, role).await;
    let now = f.now().await;
    let cert = &proof.cert;
    let claims = PortableClaims {
        mechanism: PortableMechanism::Certified,
        audience: (*old.claims.authority.audience),
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
        .await
        .unwrap();
    let signature = f
        .retrieve(CERTIFIED_SIGNATURE_DOMAIN, &digest)
        .await
        .unwrap();
    let now = f.now().await;
    let checked_grants = ic_auth::portable::ValidatedGrants::new(cert.grants.clone()).unwrap();
    let authority = PortableClaimsContext {
        authority: ic_auth::portable::PortableAuthority {
            audience: old.claims.authority.audience,
            issuer: cert.issuer_pid,
            mechanism: PortableMechanism::Certified,
            key_binding_id: claims.key_binding_id,
            grants: &checked_grants,
            caller: browser_principal(&claims.browser_public_key),
            now_ns: now,
            authority_not_before_ns: cert.not_before_ns,
            authority_expires_at_ns: cert.expires_at_ns.min(old.root_key.accept_until_ns),
            enrollment_expires_at_ns: cert.expires_at_ns,
            min_login_started_at_ns: 0,
        },
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 86_400 * SECOND,
        max_proof_ttl_ns: 60 * SECOND,
    };
    let limits = old.limits;
    let mut proof_context = PortableCertifiedContext {
        claims: authority,
        root_key: old.root_key,
        ic_root_public_key_raw: old.ic_root_public_key_raw,
        limits,
    };
    let mut ctx = SessionContext {
        authority: authority.authority,
        max_ttl: ic_auth::session_request::SessionTtlLimit::new(30).unwrap(),
        role,
        required_scopes: RequiredScopes::NONE,
    };
    let p = SessionPolicy {
        generation: 0,
        enabled: true,
        subject_admissible: true,
    };
    let mut store = MemorySessionStore::new(
        0,
        SessionLimits::new(1, 2, 2, 8192).expect("fixture limits"),
    );
    let mut ledger =
        MemoryLoginRevocations::new(*old.claims.authority.audience, 1, 1, 86_400_000_000_000, 0)
            .unwrap();
    let mut budget = SessionAdmissionBudget::new(8, 1, 0).unwrap();
    let bytes = encode_claims(&claims).unwrap();
    let credential = PortableCredential {
        claims: bytes.clone(),
        proof: PortableProof::Certified {
            delegation: Box::new(proof.clone()),
            issuer_signature: signature,
        },
    };
    let frame =
        ic_auth_wire::CredentialEnvelope::encode_request(&credential.clone().try_into().unwrap())
            .unwrap();
    let hex: String = frame.iter().map(|byte| format!("{byte:02x}")).collect();
    let mut command = std::process::Command::new(
        std::env::var_os("IC_AUTH_QUALIFICATION_NODE").expect("admitted Node"),
    );
    command
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../packages/client/dist/test/qualification/credential.js"
        ))
        .arg(&hex);
    assert_eq!(
        String::from_utf8(run_node(&mut command, "certified Wire roundtrip")).unwrap(),
        hex
    );
    eprintln!("Certified Wire Rust/TypeScript/Rust projection retained exact signed/proof bytes");
    let request = SessionRequest::new(vec!["read".into()], None).unwrap();
    establish_session(
        &mut store,
        &mut ledger,
        &mut budget,
        &credential,
        request.clone(),
        &ic_auth::session::SessionAdmissionContext {
            verification: ic_auth::portable_credential::PortableVerificationContext::Certified(
                &proof_context,
            ),
            role,
            ttl: ic_auth::session_request::SessionTtlPolicy::new(10, 30).unwrap(),
        },
        p,
    )
    .unwrap();
    f.pic
        .instance()
        .advance_time(Duration::from_secs(3))
        .await
        .unwrap();
    ctx.authority.now_ns = f.now().await;
    proof_context.claims.authority.now_ns = ctx.authority.now_ns;
    assert!(matches!(
        establish_session(
            &mut store,
            &mut ledger,
            &mut budget,
            &credential,
            request.clone(),
            &ic_auth::session::SessionAdmissionContext {
                verification: ic_auth::portable_credential::PortableVerificationContext::Certified(
                    &proof_context
                ),
                role,
                ttl: ic_auth::session_request::SessionTtlPolicy::new(10, 30).unwrap()
            },
            p
        ),
        Ok(SessionEstablished::ExactRetry(_))
    ));
    assert!(authorize_session(&mut store, &mut ledger, &ctx, p, |_| ()).is_ok());
    assert!(matches!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut budget,
            authority.authority.caller,
            ctx.authority.now_ns,
        ),
        LogoutResult::Revoked { .. }
    ));
    assert!(authorize_session(&mut store, &mut ledger, &ctx, p, |_| ()).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_upgrade_preserves_sessions_denials_and_requires_fresh_authority() {
    let f = Fixture::new().await;
    let now = f.now().await;
    let key = SigningKey::from_bytes(&[42; 32]);
    let config = ReceiverConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: f.signer,
        authority: ReceiverAuthority::Ed25519(Box::new(
            ic_auth::portable_signature::PreparedIssuerKey::new(key.verifying_key().to_bytes())
                .unwrap(),
        )),
        role: "user".parse().unwrap(),
        grants: ic_auth::portable::ValidatedGrants::new(vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }])
        .unwrap(),
        valid_from_ns: now,
        valid_until_ns: now + 120 * SECOND,
        enrollment_until_ns: now + 120 * SECOND,
        verification_burst: 16,
        verification_refill_ns: 1,
    };
    install(&f, f.resource, &config).await;
    install(&f, f.signer, &config).await;
    let enabled: Result<(), controls::Refusal> = update_control::<(), ()>(
        f.pic.instance(),
        f.resource,
        f.controller,
        spec!(controls::EnableCheckpoint),
        (),
    )
    .await;
    enabled.unwrap();
    f.pic
        .instance()
        .advance_time(Duration::from_secs(1))
        .await
        .unwrap();
    let mut claims = PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: config.audience,
        issuer: config.issuer,
        key_binding_id: config.authority.binding(),
        browser_public_key: SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes(),
        account_subject: Principal::from_slice(&[99]),
        login_started_at_ns: f.now().await,
        login_deadline_ns: f.now().await + 60 * SECOND,
        issued_at_ns: f.now().await,
        admission_expires_at_ns: f.now().await + 2 * SECOND,
        nonce: [1; 16],
        grants: config.grants.as_slice().to_vec(),
    };
    let live = browser_principal(&claims.browser_public_key);
    let original = input(&claims, &key);
    let session = admission(&f, f.resource, live, &original).await.unwrap();
    let peer_session = admission(&f, f.signer, live, &original).await.unwrap();
    claims.browser_public_key = SigningKey::from_bytes(&[2; 32]).verifying_key().to_bytes();
    let revoked = browser_principal(&claims.browser_public_key);
    admission(&f, f.resource, revoked, &input(&claims, &key))
        .await
        .unwrap();
    admission(&f, f.signer, revoked, &input(&claims, &key))
        .await
        .unwrap();
    let denied_until = logout(&f, f.resource, revoked).await.unwrap();
    assert!(read(&f, f.signer, revoked).await.is_ok());
    let unknown_key = SigningKey::from_bytes(&[3; 32]).verifying_key().to_bytes();
    let unknown = browser_principal(&unknown_key);
    let Err(LogoutResult::Unconfirmed {
        deny_until_ns: unknown_until,
    }) = logout(&f, f.resource, unknown).await
    else {
        panic!("unseen login must remain unconfirmed")
    };
    f.pic
        .instance()
        .advance_time(Duration::from_secs(3))
        .await
        .unwrap();
    assert!(f.now().await >= claims.admission_expires_at_ns);
    let before_upgrade = f
        .pic
        .instance()
        .cycle_balance(f.resource.into())
        .await
        .unwrap();
    f.pic
        .instance()
        .upgrade_canister(
            f.resource.into(),
            f.wasm.clone(),
            controls::Upgrade::encode_request(&()).unwrap(),
            Some(f.controller.into()),
        )
        .await
        .unwrap();
    eprintln!(
        "checkpoint upgrade cycles: {}",
        before_upgrade
            - f.pic
                .instance()
                .cycle_balance(f.resource.into())
                .await
                .unwrap()
    );
    assert_eq!(
        read(&f, f.resource, live).await.unwrap_err(),
        SessionReadRefusal::Unavailable
    );
    assert_eq!(read(&f, f.signer, live).await.unwrap(), peer_session);
    assert!(read(&f, f.signer, revoked).await.is_ok());
    assert!(admission(&f, f.resource, live, &original).await.is_err());
    assert_eq!(
        logout(&f, f.resource, live).await,
        Err(LogoutResult::Unavailable(LogoutUnavailable::ResetPending))
    );
    let unauthorized: Result<(), controls::Refusal> =
        update_control::<controls::model::ReceiverConfig, ()>(
            f.pic.instance(),
            f.resource,
            live,
            spec!(controls::ConfirmAuthority),
            (config.clone()).try_into().unwrap(),
        )
        .await;
    assert!(unauthorized.is_err());
    let mut mismatch = config.clone();
    mismatch.authority = ReceiverAuthority::Ed25519(Box::new(
        ic_auth::portable_signature::PreparedIssuerKey::new(
            SigningKey::from_bytes(&[99; 32]).verifying_key().to_bytes(),
        )
        .unwrap(),
    ));
    let wrong: Result<(), controls::Refusal> =
        update_control::<controls::model::ReceiverConfig, ()>(
            f.pic.instance(),
            f.resource,
            f.controller,
            spec!(controls::ConfirmAuthority),
            (mismatch).try_into().unwrap(),
        )
        .await;
    assert!(wrong.is_err());
    assert!(read(&f, f.resource, live).await.is_err());
    let confirmed: Result<(), controls::Refusal> =
        update_control::<controls::model::ReceiverConfig, ()>(
            f.pic.instance(),
            f.resource,
            f.controller,
            spec!(controls::ConfirmAuthority),
            (config.clone()).try_into().unwrap(),
        )
        .await;
    confirmed.unwrap();
    assert_eq!(read(&f, f.resource, live).await.unwrap(), session);
    // Restoration retains the live exact retry and its original lease even
    // though the proof expired. It must not enter fresh verification.
    let mut retry = original.clone();
    retry.credential.proof =
        ic_auth_wire::model::PortableProof::Ed25519(Bytes::new(vec![]).unwrap());
    assert_eq!(
        admission(&f, f.resource, live, &retry).await.unwrap(),
        session
    );
    assert!(read(&f, f.resource, revoked).await.is_err());
    assert_eq!(logout(&f, f.resource, revoked).await.unwrap(), denied_until);
    assert_eq!(
        logout(&f, f.resource, unknown).await,
        Err(LogoutResult::Unconfirmed {
            deny_until_ns: unknown_until
        })
    );
    // Newly signed credentials for both revoked logins remain denied after boot.
    claims.issued_at_ns = f.now().await;
    claims.admission_expires_at_ns = f.now().await + 2 * SECOND;
    claims.nonce = [2; 16];
    let renewed = input(&claims, &key);
    assert!(admission(&f, f.resource, revoked, &renewed).await.is_err());
    let renewed_peer = admission(&f, f.signer, revoked, &renewed).await.unwrap();
    assert_eq!(renewed_peer.caller.as_slice(), revoked.as_slice());
    assert_eq!(renewed_peer.subject, session.subject);
    claims.browser_public_key = unknown_key;
    assert!(
        admission(&f, f.resource, unknown, &input(&claims, &key))
            .await
            .is_err()
    );
    let disabled: Result<(), controls::Refusal> = update_control::<bool, ()>(
        f.pic.instance(),
        f.resource,
        f.controller,
        spec!(controls::InvalidateAuthority),
        false,
    )
    .await;
    disabled.unwrap();
    f.pic
        .instance()
        .upgrade_canister(
            f.resource.into(),
            f.wasm.clone(),
            controls::Upgrade::encode_request(&()).unwrap(),
            Some(f.controller.into()),
        )
        .await
        .unwrap();
    let confirmed: Result<(), controls::Refusal> =
        update_control::<controls::model::ReceiverConfig, ()>(
            f.pic.instance(),
            f.resource,
            f.controller,
            spec!(controls::ConfirmAuthority),
            (config).try_into().unwrap(),
        )
        .await;
    confirmed.unwrap();
    assert!(read(&f, f.resource, live).await.is_err());
    assert_eq!(logout(&f, f.resource, revoked).await.unwrap(), denied_until);
    assert_eq!(read(&f, f.signer, live).await.unwrap(), peer_session);
    assert_eq!(read(&f, f.signer, revoked).await.unwrap(), renewed_peer);
    logout(&f, f.signer, revoked).await.unwrap();
    assert!(read(&f, f.signer, revoked).await.is_err());
    assert_eq!(read(&f, f.signer, live).await.unwrap(), peer_session);
    assert_eq!(logout(&f, f.resource, revoked).await.unwrap(), denied_until);
    f.pic
        .close()
        .await
        .expect("delete qualification instance before stopping its server");
}
