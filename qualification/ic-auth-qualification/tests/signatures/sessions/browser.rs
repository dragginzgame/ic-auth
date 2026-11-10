//! Opt-in real-browser qualification; native fixture issuance remains injected.
use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicitly prepared Chromium: make test-client-browser"]
async fn actual_browser_storage_signing_and_receiver_transport() {
    let mut f = Fixture::with_application_subnets(2).await;
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
        valid_from_ns: f.now().await,
        valid_until_ns: f.now().await + 120 * SECOND,
        enrollment_until_ns: f.now().await + 120 * SECOND,
        verification_burst: 32,
        verification_refill_ns: 1,
    };
    for receiver in [f.signer, f.resource] {
        install(&f, receiver, &config).await;
    }
    f.pic
        .instance()
        .advance_time(Duration::from_secs(1))
        .await
        .unwrap();
    let subject = browser_principal(&SigningKey::from_bytes(&[81; 32]).verifying_key().to_bytes());
    let now = f.now().await;
    let credentials: Vec<_> = [9, 10]
        .into_iter()
        .map(|device| {
            let claims = PortableClaims {
                mechanism: PortableMechanism::Ed25519,
                audience: config.audience,
                issuer: config.issuer,
                key_binding_id: config.authority.binding(),
                device_slot: 0,
                device_generation: 1,
                account_subject: browser_principal(
                    &SigningKey::from_bytes(&[device + 72; 32])
                        .verifying_key()
                        .to_bytes(),
                ),
                browser_public_key: SigningKey::from_bytes(&[device; 32])
                    .verifying_key()
                    .to_bytes(),
                login_started_at_ns: now,
                login_deadline_ns: now + 90 * SECOND,
                issued_at_ns: now,
                admission_expires_at_ns: now + 60 * SECOND,
                nonce: [device; 16],
                grants: config.grants.as_slice().to_vec(),
            };
            ic_auth_wire::CredentialEnvelope::encode_request(&input(&claims, &key).credential)
                .unwrap()
        })
        .collect();
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
        std::env::var_os("IC_AUTH_QUALIFICATION_NODE").expect("admitted Node"),
    );
    command
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../packages/client/test/browser/run.mjs"
        ))
        .arg(url)
        .arg(f.signer.to_text())
        .arg(f.resource.to_text())
        .arg(hex(&f.root_key().await))
        .arg(now.to_string())
        .arg(hex(&credentials[0]))
        .arg(subject.to_text())
        .arg((now + 60 * SECOND).to_string())
        .arg(hex(&credentials[1]));
    eprintln!(
        "{}",
        String::from_utf8_lossy(&run_node(&mut command, "actual browser qualification"))
    );
    for device in [9, 10] {
        let caller = browser_principal(
            &SigningKey::from_bytes(&[device; 32])
                .verifying_key()
                .to_bytes(),
        );
        for receiver in [f.signer, f.resource] {
            assert!(
                read(&f, receiver, caller).await.is_err(),
                "native receiver must retain browser logout"
            );
        }
    }
    f.pic
        .close()
        .await
        .expect("delete qualification instance before stopping its server");
}
