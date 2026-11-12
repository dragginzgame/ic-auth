//! Installed scaling of Auth-owned durable bookkeeping, separate from proof crypto.
use super::*;
use controls::persistent::{Command, Report};

// Interim maintainer acceptance of the measured 1.223M cold lookup on published
// Storage 0.73.4. Keep a regression bound while Storage optimizes page locality;
// neither a new supplier nor a sub-1M lookup is required for this Auth batch.
const ORDINARY_AUTHENTICATION_INSTRUCTION_LIMIT: u64 = 1_300_000;

async fn command(f: &Fixture, command: Command) -> Report {
    update_control(
        f.pic.instance(),
        f.resource,
        f.controller,
        spec!(controls::PersistentIssuer),
        command,
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn indexed_issuer_cost_stays_bounded_through_one_hundred_thousand_logins() {
    let f = Fixture::new().await;
    command(&f, Command::Initialize).await;
    let maximum = std::env::var("IC_AUTH_PERSISTENCE_TEST_LOGINS")
        .map(|s| {
            s.parse::<u32>()
                .expect("positive retained-login test target")
        })
        .unwrap_or(100_000);
    assert!((1..=100_000).contains(&maximum));
    let mut filled = 0;
    for target in [1, 1000, 10_000, 100_000] {
        let target = target.min(maximum);
        if target <= filled {
            continue;
        }
        let mut fill_max = 0;
        while filled < target {
            let count = 64.min(target - filled);
            let report = command(
                &f,
                Command::Fill {
                    start: filled + 1,
                    count,
                    certified: target % 2 == 0,
                },
            )
            .await;
            fill_max = fill_max.max(report.samples[0].1);
            filled += count;
            assert_eq!(report.logins, filled as u64);
        }
        let report = command(
            &f,
            Command::Profile {
                account: target,
                certified: target % 2 == 0,
            },
        )
        .await;
        eprintln!(
            "Auth persistent issuer installed: retained={target}, fill_max_individual={fill_max}, samples={:?}",
            report.samples
        );
        assert!(
            fill_max < 10_000_000,
            "individual prepare exceeded the small-record bookkeeping regression budget"
        );
        for (operation, instructions) in &report.samples {
            assert!(
                *instructions < 10_000_000,
                "{operation} exceeded small-record bookkeeping regression budget: {instructions}"
            );
        }
    }
    f.pic
        .instance()
        .upgrade_canister(
            PocketPrincipal::from_slice(f.resource.as_slice()),
            f.wasm.clone(),
            controls::Upgrade::encode_request(&()).unwrap(),
            Some(PocketPrincipal::from_slice(f.controller.as_slice())),
        )
        .await
        .unwrap();
    let report = command(&f, Command::Prune { budget: 0 }).await;
    assert_eq!(report.logins, maximum as u64);
    assert!(
        report.epoch >= 2,
        "same-release restore closes the previous operation epoch"
    );
    f.pic.close().await;
}

fn receiver_records(
    f: &Fixture,
    n: u32,
    now: u64,
) -> (controls::persistent::StoredSession, [u8; 32]) {
    use ic_platform::ed25519::{Signer, SigningKey};
    let key = SigningKey::from_bytes(&[42; 32]);
    let mut seed = [1; 32];
    seed[..4].copy_from_slice(&n.to_be_bytes());
    let claims = ic_auth_protocol_types::PortableClaims {
        mechanism: ic_auth_protocol_types::PortableMechanism::Ed25519,
        audience: ic_auth_protocol_types::AudienceId {
            canonical_network_id: ic_auth_protocol_types::CanonicalId::from_bytes([1; 32]),
            fleet_id: ic_auth_protocol_types::CanonicalId::from_bytes([2; 32]),
        },
        issuer: f.resource,
        key_binding_id: ic_auth::portable_signature::PreparedIssuerKey::new(
            key.verifying_key().to_bytes(),
        )
        .unwrap()
        .binding(),
        browser_public_key: SigningKey::from_bytes(&seed).verifying_key().to_bytes(),
        account_subject: Principal::from_slice(&n.to_be_bytes()),
        device_slot: 0,
        device_generation: 1,
        login_started_at_ns: now,
        login_deadline_ns: now + 90 * 86_400_000_000_000,
        issued_at_ns: now,
        admission_expires_at_ns: now + 60_000_000_000,
        nonce: [1; 16],
        grants: vec![ic_auth_protocol_types::DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }],
    };
    let credential = ic_auth_protocol_types::PortableCredential {
        claims: ic_auth::portable::encode_claims(&claims).unwrap(),
        proof: ic_auth_protocol_types::PortableProof::Ed25519 {
            signature: key
                .sign(&ic_auth::portable::claims_hash(&claims).unwrap())
                .to_bytes()
                .to_vec(),
        },
    };
    let (session, replay, login) = ic_auth_backend_conformance::admitted_records(
        &claims,
        &credential,
        key.verifying_key().to_bytes(),
        now,
    );
    (
        controls::persistent::StoredSession {
            session: Bytes::new(session).unwrap(),
            replay: Bytes::new(replay).unwrap(),
            login: Bytes::new(login).unwrap(),
        },
        claims.browser_public_key,
    )
}

#[tokio::test]
async fn ordinary_persistent_authentication_is_a_point_operation_at_scale() {
    let f = Fixture::new().await;
    command(&f, Command::Initialize).await;
    let maximum = std::env::var("IC_AUTH_PERSISTENCE_TEST_LOGINS")
        .map(|s| s.parse::<u32>().expect("retained-login test target"))
        .unwrap_or(100_000);
    assert!((1..=100_000).contains(&maximum));
    let issued_ns = f.now().await;
    let mut filled = 0;
    let mut last_browser = [0; 32];
    let mut overruns = Vec::new();
    for target in [1, 1000, 10_000, 100_000] {
        let target = target.min(maximum);
        if target <= filled {
            continue;
        }
        while filled < target {
            let count = 32.min(target - filled);
            let prepared: Vec<_> = (filled + 1..=filled + count)
                .map(|n| receiver_records(&f, n, issued_ns))
                .collect();
            last_browser = prepared.last().unwrap().1;
            let records = prepared.into_iter().map(|(record, _)| record).collect();
            f.pic
                .instance()
                .advance_time(Duration::from_nanos(64))
                .await;
            let report = command(&f, Command::FillReceiver { records }).await;
            filled += count;
            assert_eq!(report.logins, filled as u64);
        }
        let report = command(
            &f,
            Command::ProfileReceiver {
                browser_public_key: last_browser,
            },
        )
        .await;
        eprintln!(
            "Auth persistent receiver installed: retained={target}, samples={:?}",
            report.samples
        );
        if report.samples[0].1 >= ORDINARY_AUTHENTICATION_INSTRUCTION_LIMIT {
            overruns.push((target, report.samples[0].1));
        }
    }
    f.pic
        .instance()
        .upgrade_canister(
            PocketPrincipal::from_slice(f.resource.as_slice()),
            f.wasm.clone(),
            controls::Upgrade::encode_request(&()).unwrap(),
            Some(PocketPrincipal::from_slice(f.controller.as_slice())),
        )
        .await
        .unwrap();
    let report = command(
        &f,
        Command::ProfileReceiver {
            browser_public_key: last_browser,
        },
    )
    .await;
    assert_eq!(report.logins, maximum as u64);
    eprintln!(
        "Auth persistent receiver after upgrade: retained={maximum}, samples={:?}",
        report.samples
    );
    if report.samples[0].1 >= ORDINARY_AUTHENTICATION_INSTRUCTION_LIMIT {
        overruns.push((maximum, report.samples[0].1));
    }
    f.pic.close().await;
    assert!(
        overruns.is_empty(),
        "ordinary authentication exceeded interim {ORDINARY_AUTHENTICATION_INSTRUCTION_LIMIT}-instruction budget: {overruns:?}"
    );
}
