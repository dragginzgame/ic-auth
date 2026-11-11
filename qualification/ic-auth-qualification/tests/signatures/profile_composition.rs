use super::*;

// These are existing external protocol identities, not new library defaults.
const ROOT_PROFILE: &[u8] = b"canic-root-role-attestation";
const ISSUER_PROFILE: &[u8] = b"canic-issuer-delegated-token";
const SECOND: u64 = 1_000_000_000;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn root_and_issuer_profiles_share_one_composed_store_without_crossing_authority() {
    let fixture = Fixture::new().await;
    // Identical payloads must remain distinct under each approved seed/domain.
    let payload = [42; 32];
    let root_deadline = fixture
        .prepare_with_seed(ROOT_PROFILE, ROOT_PROFILE, &payload, 5 * SECOND)
        .await
        .unwrap();
    fixture
        .prepare_with_seed(ISSUER_PROFILE, ISSUER_PROFILE, &payload, 60 * SECOND)
        .await
        .unwrap();
    let root = fixture
        .retrieve_with_seed(ROOT_PROFILE, ROOT_PROFILE, &payload)
        .await
        .unwrap();
    let issuer = fixture
        .retrieve_with_seed(ISSUER_PROFILE, ISSUER_PROFILE, &payload)
        .await
        .unwrap();
    for (profile, proof) in [(ROOT_PROFILE, &root), (ISSUER_PROFILE, &issuer)] {
        assert_eq!(
            proof.public_key_der,
            CanisterSigPublicKey::new(fixture.signer, profile.to_vec()).to_der()
        );
        assert_eq!(
            fixture
                .verify_result(profile, profile, payload, proof)
                .await,
            Ok(())
        );
    }
    assert_ne!(root.public_key_der, issuer.public_key_der);
    assert_eq!(
        fixture
            .verify_result(ROOT_PROFILE, ROOT_PROFILE, payload, &issuer)
            .await,
        Err(CanisterSignatureError::SeedMismatch)
    );
    assert!(matches!(
        fixture
            .verify_result(ISSUER_PROFILE, ROOT_PROFILE, payload, &issuer)
            .await,
        Err(CanisterSignatureError::InvalidSignature(_))
    ));
    assert!(
        fixture
            .retrieve_with_seed(ROOT_PROFILE, ISSUER_PROFILE, &payload)
            .await
            .is_err()
    );

    // The fixture's protected 16-leaf limit is shared across both profiles.
    for byte in 0..14 {
        fixture
            .prepare_with_seed(ROOT_PROFILE, ROOT_PROFILE, &[byte; 32], 60 * SECOND)
            .await
            .unwrap();
    }
    assert!(
        fixture
            .prepare_with_seed(ISSUER_PROFILE, ISSUER_PROFILE, &[99; 32], 60 * SECOND)
            .await
            .is_err()
    );
    assert_eq!(
        fixture
            .prepare_with_seed(ROOT_PROFILE, ROOT_PROFILE, &payload, 60 * SECOND)
            .await,
        Ok(root_deadline)
    );
    fixture.set_asset(9).await;
    for profile in [ROOT_PROFILE, ISSUER_PROFILE] {
        let proof = fixture
            .retrieve_with_seed(profile, profile, &payload)
            .await
            .unwrap();
        assert_eq!(
            fixture
                .verify_result(profile, profile, payload, &proof)
                .await,
            Ok(())
        );
    }

    fixture
        .pic
        .instance()
        .set_time(Time::from_nanos_since_unix_epoch(root_deadline))
        .await;
    assert!(
        fixture
            .retrieve_with_seed(ROOT_PROFILE, ROOT_PROFILE, &payload)
            .await
            .is_err()
    );
    let removed: Result<u32, controls::Refusal> = update_control::<u32, u32>(
        fixture.pic.instance(),
        fixture.signer,
        fixture.controller,
        spec!(controls::Prune),
        128_u32,
    )
    .await;
    assert_eq!(removed, Ok(1));
    let live_issuer = fixture
        .retrieve_with_seed(ISSUER_PROFILE, ISSUER_PROFILE, &payload)
        .await
        .unwrap();
    assert_eq!(
        fixture
            .verify_result(ISSUER_PROFILE, ISSUER_PROFILE, payload, &live_issuer)
            .await,
        Ok(())
    );
    fixture
        .prepare_with_seed(ISSUER_PROFILE, ISSUER_PROFILE, &[99; 32], 60 * SECOND)
        .await
        .unwrap();
    assert_eq!(fixture.asset_version().await, 9);

    fixture.upgrade().await;
    assert_eq!(fixture.asset_version().await, 9);
    for (profile, original) in [(ROOT_PROFILE, root), (ISSUER_PROFILE, issuer)] {
        assert!(
            fixture
                .retrieve_with_seed(profile, profile, &payload)
                .await
                .is_err()
        );
        fixture
            .prepare_with_seed(profile, profile, &payload, 60 * SECOND)
            .await
            .unwrap();
        let fresh = fixture
            .retrieve_with_seed(profile, profile, &payload)
            .await
            .unwrap();
        assert_eq!(fresh.public_key_der, original.public_key_der);
        assert_eq!(
            fixture
                .verify_result(profile, profile, payload, &fresh)
                .await,
            Ok(())
        );
    }
    fixture.pic.close().await;
}
