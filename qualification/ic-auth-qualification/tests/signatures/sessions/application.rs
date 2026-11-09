//! Ordinary application effects use a live local session, never another proof.
use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scoped_application_updates_use_only_the_live_caller_session() {
    let mut f = Fixture::with_application_subnets(2).await;
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
            scopes: vec!["read".parse().unwrap(), "write".parse().unwrap()],
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
    f.pic
        .instance()
        .advance_time(Duration::from_secs(1))
        .await
        .unwrap();
    let now = f.now().await;
    let reader_claims = PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: config.audience,
        issuer: config.issuer,
        key_binding_id: config.authority.binding(),
        browser_public_key: SigningKey::from_bytes(&[72; 32]).verifying_key().to_bytes(),
        account_subject: Principal::from_slice(&[99]),
        login_started_at_ns: now,
        login_deadline_ns: now + 90 * SECOND,
        issued_at_ns: now,
        admission_expires_at_ns: now + 10 * SECOND,
        nonce: [72; 16],
        grants: config.grants.as_slice().to_vec(),
    };
    let writer_claims = PortableClaims {
        browser_public_key: SigningKey::from_bytes(&[73; 32]).verifying_key().to_bytes(),
        nonce: [73; 16],
        ..reader_claims.clone()
    };
    let reader = browser_principal(&reader_claims.browser_public_key);
    let writer = browser_principal(&writer_claims.browser_public_key);
    let version = async |f: &Fixture, target: Principal| {
        ic_testkit::pic::query_wire::<(), u64>(
            f.pic.instance(),
            target,
            Principal::anonymous(),
            spec!(controls::AssetVersion),
            &(),
        )
        .await
        .unwrap()
    };
    let write = async |f: &Fixture, target: Principal, caller: Principal, value: u64| {
        update_control::<u64, ()>(
            f.pic.instance(),
            target,
            caller,
            spec!(controls::SetSessionAsset),
            value,
        )
        .await
    };
    assert!(
        write(&f, f.resource, Principal::anonymous(), 9)
            .await
            .is_err()
    );
    assert!(write(&f, f.resource, writer, 9).await.is_err());
    assert_eq!(version(&f, f.resource).await, 1);

    let mut reader_input = input(&reader_claims, &key);
    reader_input.ttl_secs = Some(60);
    admission(&f, f.resource, reader, &reader_input)
        .await
        .unwrap();
    let mut writer_input = input(&writer_claims, &key);
    writer_input.scopes = List::new(vec![Text::new("write".into()).unwrap()]).unwrap();
    writer_input.ttl_secs = Some(60);
    admission(&f, f.resource, writer, &writer_input)
        .await
        .unwrap();
    // One receiver's session is not another receiver's admission.
    assert!(write(&f, f.signer, writer, 9).await.is_err());
    assert_eq!(version(&f, f.signer).await, 1);
    admission(&f, f.signer, writer, &writer_input)
        .await
        .unwrap();

    f.pic
        .instance()
        .advance_time(Duration::from_secs(11))
        .await
        .unwrap();
    assert!(f.now().await > writer_claims.admission_expires_at_ns);
    let url = f
        .pic
        .instance_mut()
        .make_live(None)
        .await
        .unwrap()
        .to_string();
    let network = f.root_key().await;
    let receiver = f.resource;
    let signed_write = async |device: u8, value: u64| {
        async {
            let agent = Agent::builder()
                .with_url(&url)
                .with_identity(BasicIdentity::from_raw_key(&[device; 32]))
                .with_ingress_expiry(Duration::from_secs(30))
                .build()
                .unwrap();
            agent.set_root_key(network.clone());
            // The only application input is the new value, with no credential,
            // proof, account or purported caller in the request.
            let bytes = tokio::time::timeout(
                Duration::from_secs(10),
                agent
                    .update(
                        &receiver
                            .as_slice()
                            .try_into()
                            .expect("validated Principal bytes"),
                        controls::SetSessionAsset::NAME,
                    )
                    .with_arg(controls::SetSessionAsset::encode_request(&value).unwrap())
                    .call_and_wait(),
            )
            .await
            .expect("bounded signed application update")
            .expect("authenticated application reply");
            controls::SetSessionAsset::decode_response(&bytes).unwrap()
        }
        .await
    };
    // The credential grants both scopes; the reader's stored session narrows it.
    assert!(signed_write(72, 9).await.is_err());
    assert!(signed_write(74, 9).await.is_err());
    assert_eq!(version(&f, f.resource).await, 1);
    signed_write(73, 2).await.unwrap();
    assert_eq!(version(&f, f.resource).await, 2);

    logout(&f, f.resource, writer).await.unwrap();
    assert!(signed_write(73, 9).await.is_err());
    assert_eq!(version(&f, f.resource).await, 2);
    assert!(read(&f, f.resource, reader).await.is_ok());
    // Local logout leaves the other subnet's independently admitted session live.
    write(&f, f.signer, writer, 3).await.unwrap();
    assert_eq!(version(&f, f.signer).await, 3);
    update_control::<bool, ()>(
        f.pic.instance(),
        f.signer,
        f.controller,
        spec!(controls::InvalidateAuthority),
        true,
    )
    .await
    .unwrap();
    assert!(write(&f, f.signer, writer, 9).await.is_err());
    assert_eq!(version(&f, f.signer).await, 3);
    f.pic
        .close()
        .await
        .expect("delete qualification instance before stopping its server");
}
