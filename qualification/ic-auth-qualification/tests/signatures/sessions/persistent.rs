//! Real proofs through durable issuer/receiver stores. Controller effects are
//! fixture authority; ingress callers on admission/read/logout are actual callers.
use super::*;
use controls::persistent::{Command, IssuerAction, Report, StoredCredential};
use ic_auth::portable_certified::CERTIFIED_SIGNATURE_DOMAIN;
use ic_platform::canister_sig_creation::{extract_raw_root_pk_from_der, hash_bytes};

async fn command(
    f: &Fixture,
    target: Principal,
    command: Command,
) -> Result<Report, controls::Refusal> {
    update_control(
        f.pic.instance(),
        target,
        f.controller,
        spec!(controls::PersistentIssuer),
        command,
    )
    .await
}
async fn issuer(
    f: &Fixture,
    config: &ReceiverConfig,
    login: u32,
    sequence: u64,
    action: IssuerAction,
) -> Result<StoredCredential, controls::Refusal> {
    command(
        f,
        f.signer,
        Command::Lifecycle {
            action,
            login,
            sequence,
            browser_public_key: SigningKey::from_bytes(&[login as u8; 32])
                .verifying_key()
                .to_bytes(),
            key_binding_id: config.authority.binding(),
            certified: config.authority.mechanism() == PortableMechanism::Certified,
        },
    )
    .await
    .map(|r| r.credential.unwrap())
}
async fn admit(
    f: &Fixture,
    caller: Principal,
    input: &SessionInput,
) -> Result<SessionView, AdmissionRefusal> {
    update_wire(
        f.pic.instance(),
        PocketPrincipal::from_slice(f.resource.as_slice()),
        PocketPrincipal::from_slice(caller.as_slice()),
        spec!(controls::PersistentAdmission),
        input,
    )
    .await
    .unwrap()
}
async fn read(
    f: &Fixture,
    caller: Principal,
) -> Result<SessionView, ic_auth_wire::model::SessionReadRefusal> {
    update_wire(
        f.pic.instance(),
        PocketPrincipal::from_slice(f.resource.as_slice()),
        PocketPrincipal::from_slice(caller.as_slice()),
        spec!(controls::PersistentRead),
        &(),
    )
    .await
    .unwrap()
}
async fn logout(f: &Fixture, caller: Principal) -> ic_auth_wire::model::LogoutResult {
    update_wire(
        f.pic.instance(),
        PocketPrincipal::from_slice(f.resource.as_slice()),
        PocketPrincipal::from_slice(caller.as_slice()),
        spec!(controls::PersistentLogout),
        &(),
    )
    .await
    .unwrap()
}
async fn upgrade(f: &Fixture, target: Principal) {
    f.pic
        .instance()
        .upgrade_canister(
            PocketPrincipal::from_slice(target.as_slice()),
            f.wasm.clone(),
            controls::Upgrade::encode_request(&()).unwrap(),
            Some(PocketPrincipal::from_slice(f.controller.as_slice())),
        )
        .await
        .unwrap();
}
async fn issued(
    f: &Fixture,
    config: &ReceiverConfig,
    proof: Option<&DelegationProof>,
    key: &SigningKey,
    login: u32,
    sequence: u64,
) -> (SessionInput, StoredCredential) {
    let selected = issuer(f, config, login, sequence, IssuerAction::Prepare)
        .await
        .unwrap();
    let claims = ic_auth::portable::decode_claims(selected.claims.as_slice()).unwrap();
    let digest = claims_hash(&claims).unwrap();
    let signature = proof
        .is_none()
        .then(|| Bytes::new(key.sign(&digest).to_bytes().to_vec()).unwrap());
    let completed = issuer(
        f,
        config,
        login,
        sequence,
        IssuerAction::Complete {
            signature: signature.clone(),
        },
    )
    .await
    .unwrap();
    assert!(completed.completed);
    assert_eq!(completed.claims, selected.claims);
    assert_eq!(completed.signature, signature);
    let portable = if let Some(delegation) = proof {
        PortableProof::Certified {
            delegation: Box::new(delegation.clone()),
            issuer_signature: f
                .retrieve(CERTIFIED_SIGNATURE_DOMAIN, &digest)
                .await
                .unwrap(),
        }
    } else {
        PortableProof::Ed25519 {
            signature: signature.unwrap().into_vec(),
        }
    };
    (
        SessionInput {
            credential: PortableCredential {
                claims: completed.claims.as_slice().to_vec(),
                proof: portable,
            }
            .try_into()
            .unwrap(),
            scopes: List::new(vec![Text::new("read".into()).unwrap()]).unwrap(),
            ttl_secs: None,
        },
        completed,
    )
}

async fn configuration(
    f: &Fixture,
    certified: bool,
    key: &SigningKey,
) -> (ReceiverConfig, Option<DelegationProof>) {
    let now = f.now().await;
    let audience = AudienceId {
        canonical_network_id: CanonicalId::from_bytes([1; 32]),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    };
    let grants = vec![DelegatedRoleGrant {
        target: "user".parse().unwrap(),
        scopes: vec!["read".parse().unwrap()],
    }];
    let mut proof = None;
    let authority = if certified {
        let root_key = ic_platform::ecdsa::SigningKey::from_slice(&[7; 32]).unwrap();
        let root = ic_auth::delegation::RootKeyPolicy {
            root_canister_id: f.resource,
            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
            key_id: ChainKeyKeyId {
                name: "test_key_1".into(),
            },
            derivation_path_hash: ic_auth::canonical::chain_key_derivation_path_hash(&[
                b"canic".to_vec(),
                b"delegation".to_vec(),
            ])
            .unwrap(),
            public_key: root_key
                .verifying_key()
                .to_sec1_point(true)
                .as_bytes()
                .to_vec(),
            key_version: 4,
            min_accepted_key_version: 4,
            min_accepted_proof_epoch: 2,
            min_accepted_registry_epoch: 3,
            valid_from_ns: now,
            accept_until_ns: now + 120 * SECOND,
            max_revocation_latency_ns: 120 * SECOND,
        };
        let binding = IssuerProofBinding::IcCanisterSignatureV1 {
            seed_hash: hash_bytes(super::super::SEED),
        };
        let cert = DelegationCert {
            root_pid: f.resource,
            issuer_pid: f.signer,
            issuer_proof_alg: IssuerProofAlgorithm::IcCanisterSignatureV1,
            issuer_proof_binding_hash: ic_auth::canonical::issuer_proof_binding_hash(
                f.signer,
                IssuerProofAlgorithm::IcCanisterSignatureV1,
                binding,
            )
            .unwrap(),
            issuer_proof_binding: binding,
            issued_at_ns: now,
            not_before_ns: now,
            expires_at_ns: now + 120 * SECOND,
            max_token_ttl_ns: 60 * SECOND,
            aud: DelegationAudience::Fleet(audience),
            grants: grants.clone(),
        };
        let binding = ic_auth::canonical::cert_hash(&cert).unwrap();
        proof = Some(DelegationProof {
            root_proof: super::super::certified_proofs::root_proof(&cert, &root_key, &root),
            cert,
        });
        ReceiverAuthority::Certified {
            binding,
            root: Box::new(root),
            network_key: extract_raw_root_pk_from_der(&f.root_key().await).unwrap(),
        }
    } else {
        ReceiverAuthority::Ed25519(Box::new(
            ic_auth::portable_signature::PreparedIssuerKey::new(key.verifying_key().to_bytes())
                .unwrap(),
        ))
    };
    (
        ReceiverConfig {
            audience,
            issuer: f.signer,
            authority,
            role: "user".parse().unwrap(),
            grants: ic_auth::portable::ValidatedGrants::new(grants).unwrap(),
            valid_from_ns: now,
            valid_until_ns: now + 120 * SECOND,
            enrollment_until_ns: now + 120 * SECOND,
            verification_burst: 16,
            verification_refill_ns: 1,
        },
        proof,
    )
}

async fn lifecycle(certified: bool) {
    let f = Fixture::new().await;
    let key = SigningKey::from_bytes(&[42; 32]);
    let (config, proof) = configuration(&f, certified, &key).await;
    install(&f, f.resource, &config).await;
    // No persistent backing: protected configuration must not imply permission
    // to use the volatile store as a fallback.
    assert_eq!(
        read(&f, f.owner).await,
        Err(ic_auth_wire::model::SessionReadRefusal::Unavailable)
    );
    for target in [f.signer, f.resource] {
        command(&f, target, Command::Initialize).await.unwrap();
    }
    update_control::<(), ()>(
        f.pic.instance(),
        f.resource,
        f.controller,
        spec!(controls::EnableCheckpoint),
        (),
    )
    .await
    .unwrap();
    f.pic
        .instance()
        .advance_time(Duration::from_nanos(64))
        .await;
    let mut sessions = Vec::new();
    for login in 1..=8 {
        let (input, completed) = issued(&f, &config, proof.as_ref(), &key, login, 0).await;
        let claims = ic_auth::portable::decode_claims(completed.claims.as_slice()).unwrap();
        let caller = browser_principal(&claims.browser_public_key);
        assert!(admit(&f, f.owner, &input).await.is_err());
        let mut forged = input.clone();
        forged.credential.proof =
            ic_auth_wire::model::PortableProof::Ed25519(Bytes::new(vec![0; 64]).unwrap());
        assert!(admit(&f, caller, &forged).await.is_err());
        let session = admit(&f, caller, &input).await.unwrap();
        assert_eq!(session.subject, Principal::from_slice(&99u32.to_be_bytes()));
        assert_eq!(read(&f, caller).await.unwrap(), session);
        assert_eq!(admit(&f, caller, &input).await.unwrap(), session);
        sessions.push((caller, input, session));
    }
    assert!(
        issuer(&f, &config, 9, 0, IssuerAction::Prepare)
            .await
            .is_err(),
        "one account cannot take a ninth device"
    );
    let before = command(&f, f.signer, Command::Prune { budget: 0 })
        .await
        .unwrap();
    assert_eq!((before.logins, before.operations), (8, 8));
    f.pic.instance().advance_time(Duration::from_nanos(1)).await;
    let (renewed, renewal) = issued(&f, &config, proof.as_ref(), &key, 1, 1).await;
    let old = ic_auth::portable::decode_claims(sessions[0].1.credential.claims.as_slice()).unwrap();
    let new = ic_auth::portable::decode_claims(renewal.claims.as_slice()).unwrap();
    assert_eq!(old.login_deadline_ns, new.login_deadline_ns);
    assert_eq!(
        (old.device_slot, old.device_generation),
        (new.device_slot, new.device_generation)
    );
    let live = sessions[0].0;
    let renewed_session = admit(&f, live, &renewed).await.unwrap();
    f.pic.instance().advance_time(Duration::from_nanos(1)).await;
    let pending = issuer(&f, &config, 1, 2, IssuerAction::Prepare)
        .await
        .unwrap();
    assert!(!pending.completed);
    assert_eq!(
        issuer(&f, &config, 1, 1, IssuerAction::Recover)
            .await
            .unwrap(),
        renewal
    );
    for target in [f.signer, f.resource] {
        upgrade(&f, target).await;
    }
    assert_eq!(
        read(&f, live).await,
        Err(ic_auth_wire::model::SessionReadRefusal::Unavailable)
    );
    assert_eq!(
        issuer(&f, &config, 1, 1, IssuerAction::Recover)
            .await
            .unwrap(),
        renewal
    );
    assert!(
        !issuer(&f, &config, 1, 2, IssuerAction::Recover)
            .await
            .unwrap()
            .completed
    );
    let pending_claims = ic_auth::portable::decode_claims(pending.claims.as_slice()).unwrap();
    assert!(f.now().await < pending_claims.admission_expires_at_ns);
    assert!(
        issuer(
            &f,
            &config,
            1,
            2,
            IssuerAction::Complete {
                signature: (!certified).then(|| Bytes::new(vec![0; 64]).unwrap())
            }
        )
        .await
        .is_err(),
        "late callback after boot cannot complete"
    );
    f.pic.instance().advance_time(Duration::from_secs(3)).await;
    assert_eq!(
        issuer(&f, &config, 1, 1, IssuerAction::Recover)
            .await
            .unwrap(),
        renewal
    );
    update_control::<controls::model::ReceiverConfig, ()>(
        f.pic.instance(),
        f.resource,
        f.controller,
        spec!(controls::ConfirmAuthority),
        config.clone().try_into().unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(read(&f, live).await.unwrap(), renewed_session);
    assert_eq!(
        admit(&f, live, &renewed).await.unwrap(),
        renewed_session,
        "exact retry after expiry and upgrade retains original lease"
    );
    let revoked = logout(&f, live).await;
    assert!(matches!(
        revoked,
        ic_auth_wire::model::LogoutResult::Revoked(_)
    ));
    assert_eq!(logout(&f, live).await, revoked);
    assert!(read(&f, live).await.is_err());
    assert!(admit(&f, live, &renewed).await.is_err());
    assert!(read(&f, sessions[1].0).await.is_ok());
    update_control::<bool, ()>(
        f.pic.instance(),
        f.resource,
        f.controller,
        spec!(controls::InvalidateAuthority),
        false,
    )
    .await
    .unwrap();
    assert!(read(&f, sessions[1].0).await.is_err());
    upgrade(&f, f.resource).await;
    update_control::<controls::model::ReceiverConfig, ()>(
        f.pic.instance(),
        f.resource,
        f.controller,
        spec!(controls::ConfirmAuthority),
        config.try_into().unwrap(),
    )
    .await
    .unwrap();
    assert!(read(&f, sessions[1].0).await.is_err());
    assert_eq!(logout(&f, live).await, revoked);
    f.pic
        .instance()
        .advance_time(Duration::from_secs(121))
        .await;
    let pruned = command(&f, f.signer, Command::Prune { budget: 1 })
        .await
        .unwrap();
    assert_eq!(
        pruned.operations, 9,
        "one expired operation removed, not a history sweep"
    );
    let receiver = command(&f, f.resource, Command::PruneReceiver { budget: 1 })
        .await
        .unwrap();
    assert!(receiver.operations <= 7);
    eprintln!(
        "persistent installed lifecycle: certified={certified}; eight-device capacity, overflow, real proofs, retries, renewal, boot-fenced callback, logout, authority, bounded cleanup passed"
    );
    f.pic.close().await;
}

#[tokio::test]
async fn persistent_ed25519_lifecycle() {
    lifecycle(false).await;
}
#[tokio::test]
async fn persistent_certified_lifecycle() {
    lifecycle(true).await;
}
