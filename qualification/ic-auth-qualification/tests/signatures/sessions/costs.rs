//! Matched receiver execution; native credential production is outside the sample.
use super::*;
use ic_auth::portable_certified::PortableCertifiedContext;

// Cost samples deliberately start from empty disposable canisters. Use the real
// management reinstall rather than a receiver endpoint that discards safety state.
async fn reinstall_receiver(f: &Fixture, target: Principal, config: &ReceiverConfig) {
    f.pic
        .instance()
        .reinstall_canister(
            PocketPrincipal::from_slice(target.as_slice()),
            f.wasm.clone(),
            controls::Initialize::encode_request(&Bytes::new(f.owner.as_slice().to_vec()).unwrap())
                .unwrap(),
            Some(PocketPrincipal::from_slice(f.controller.as_slice())),
        )
        .await
        .unwrap();
    install(f, target, config).await;
}

async fn profile(
    f: &Fixture,
    target: Principal,
    caller: Principal,
    input: &SessionInput,
    label: &str,
) -> Result<SessionView, AdmissionRefusal> {
    let report: AdmissionProfile = measured(f, target, label, async || {
        update_wire::<ic_auth_wire::SessionInput, controls::model::AdmissionProfile>(
            f.pic.instance(),
            PocketPrincipal::from_slice(target.as_slice()),
            PocketPrincipal::from_slice(caller.as_slice()),
            spec!(controls::ProfileAdmission),
            input,
        )
        .await
        .expect("qualified Wire admission profile")
    })
    .await;
    assert!(report.instructions > 0);
    eprintln!(
        "portable receiver admission instructions: {label}={}",
        report.instructions
    );
    ic_auth_wire::EstablishSession::decode_response(report.reply.as_slice())
        .expect("bounded shared admission reply")
}

pub(super) async fn qualify(
    f: &Fixture,
    proof: &DelegationProof,
    old: &PortableCertifiedContext<'_>,
    role: &AuthRole,
) {
    eprintln!(
        "matched receiver artifact: bytes={}, sha256={}; same application subnet, endpoint, grants, lease and volatile store; three fresh logins per mechanism; native issuance excluded",
        f.wasm.len(),
        CanonicalId::from_bytes(Sha256::digest(&f.wasm).into())
    );
    let cert = &proof.cert;
    let key = SigningKey::from_bytes(&[42; 32]);
    for certified in [false, true] {
        let mechanism = if certified { "certified" } else { "ed25519" };
        let config = ReceiverConfig {
            audience: *old.claims.authority.audience,
            issuer: cert.issuer_pid,
            authority: if certified {
                let root = old.root_key.clone();
                ReceiverAuthority::Certified {
                    binding: ic_auth::canonical::cert_hash(cert).unwrap(),
                    root: Box::new(root),
                    network_key: old.ic_root_public_key_raw.to_vec(),
                }
            } else {
                ReceiverAuthority::Ed25519(Box::new(
                    ic_auth::portable_signature::PreparedIssuerKey::new(
                        key.verifying_key().to_bytes(),
                    )
                    .unwrap(),
                ))
            },
            role: role.clone(),
            grants: ic_auth::portable::ValidatedGrants::new(cert.grants.clone()).unwrap(),
            valid_from_ns: cert.not_before_ns,
            valid_until_ns: cert.expires_at_ns.min(old.root_key.accept_until_ns),
            enrollment_until_ns: cert.expires_at_ns.min(old.root_key.accept_until_ns),
            verification_burst: 16,
            verification_refill_ns: 1,
        };
        let make_claims = async |device: u8| PortableClaims {
            mechanism: config.authority.mechanism(),
            audience: config.audience,
            issuer: config.issuer,
            key_binding_id: config.authority.binding(),
            browser_public_key: SigningKey::from_bytes(&[device; 32])
                .verifying_key()
                .to_bytes(),
            device_slot: 0,
            device_generation: 1,
            account_subject: Principal::from_slice(&[device]),
            login_started_at_ns: f.now().await,
            login_deadline_ns: f.now().await + 60 * SECOND,
            issued_at_ns: f.now().await,
            admission_expires_at_ns: f.now().await + 2 * SECOND,
            nonce: [device; 16],
            grants: cert.grants.clone(),
        };
        let make_input = async |claims: &PortableClaims| {
            if !certified {
                return input(claims, &key);
            }
            let digest = claims_hash(claims).unwrap();
            let domain = ic_auth::portable_certified::CERTIFIED_SIGNATURE_DOMAIN;
            f.prepare(domain, &digest, 60 * SECOND).await.unwrap();
            SessionInput {
                credential: PortableCredential {
                    claims: encode_claims(claims).unwrap(),
                    proof: PortableProof::Certified {
                        delegation: Box::new(proof.clone()),
                        issuer_signature: f.retrieve(domain, &digest).await.unwrap(),
                    },
                }
                .try_into()
                .unwrap(),
                scopes: List::new(vec![Text::new("read".into()).unwrap()]).unwrap(),
                ttl_secs: None,
            }
        };
        // A valid-point, wrong-network certificate reaches BLS verification.
        // The Ed25519 case corrupts the issuer signature. Neither creates a session.
        let mut rejecting = config.clone();
        rejecting.verification_burst = 1;
        rejecting.verification_refill_ns = SECOND;
        if let ReceiverAuthority::Certified { network_key, .. } = &mut rejecting.authority {
            *network_key = ic_platform::canister_sig_creation::extract_raw_root_pk_from_der(
                ic_platform::canister_sig_creation::IC_ROOT_PK_DER,
            )
            .unwrap();
            assert_ne!(network_key, old.ic_root_public_key_raw);
        }
        reinstall_receiver(f, f.resource, &rejecting).await;
        f.pic.instance().advance_time(Duration::from_secs(1)).await;
        let rejected_claims = make_claims(19).await;
        let rejected_caller = browser_principal(&rejected_claims.browser_public_key);
        let mut rejected = make_input(&rejected_claims).await;
        if let ic_auth_wire::model::PortableProof::Ed25519(signature) =
            &mut rejected.credential.proof
        {
            let mut bytes = signature.as_slice().to_vec();
            bytes[0] ^= 1;
            *signature = Bytes::new(bytes).unwrap();
        }
        let label = format!("matched_{mechanism}_invalid_proof");
        assert_eq!(
            profile(f, f.resource, rejected_caller, &rejected, &label).await,
            Err(AdmissionRefusal::InvalidCredential)
        );
        assert!(read(f, f.resource, rejected_caller).await.is_err());
        let another = make_claims(18).await;
        let another_caller = browser_principal(&another.browser_public_key);
        let another_input = make_input(&another).await;
        let throttled = profile(
            f,
            f.resource,
            another_caller,
            &another_input,
            &format!("matched_{mechanism}_throttled_other_caller"),
        )
        .await;
        assert_eq!(throttled, Err(AdmissionRefusal::Throttled));
        assert!(read(f, f.resource, another_caller).await.is_err());
        // Fresh-verification exhaustion cannot consume the separate logout budget.
        assert!(matches!(
            logout(f, f.resource, rejected_caller).await,
            Err(LogoutResult::Unconfirmed { .. })
        ));

        // Reinstall into fresh fixture state for identical valid-admission occupancy.
        // This is an explicit test reset, never production enrollment refresh.
        reinstall_receiver(f, f.resource, &config).await;
        reinstall_receiver(f, f.signer, &config).await;
        f.pic.instance().advance_time(Duration::from_secs(1)).await;
        let mut first = None;
        for device in 20..23 {
            let claims = make_claims(device).await;
            let caller = browser_principal(&claims.browser_public_key);
            let input = make_input(&claims).await;
            let admitted = profile(
                f,
                f.resource,
                caller,
                &input,
                &format!("matched_{mechanism}_fresh_{device}"),
            )
            .await
            .unwrap();
            assert_eq!(
                (
                    admitted.caller.as_slice(),
                    admitted.issuer.as_slice(),
                    admitted.subject.as_slice()
                ),
                (
                    caller.as_slice(),
                    config.issuer.as_slice(),
                    claims.account_subject.as_slice()
                )
            );
            assert!(admitted.expires_at_ns <= claims.login_deadline_ns);
            if device == 20 {
                if certified {
                    components(f, caller, &input).await;
                    assert_eq!(read(f, f.resource, caller).await.unwrap(), admitted);
                }
                let peer = profile(
                    f,
                    f.signer,
                    caller,
                    &input,
                    &format!("matched_{mechanism}_second_receiver"),
                )
                .await
                .unwrap();
                assert_eq!(peer.subject, admitted.subject);
                first = Some((caller, input, admitted, claims));
            }
        }
        let (caller, input, mut admitted, original_claims) = first.unwrap();
        f.pic.instance().advance_time(Duration::from_secs(3)).await;
        assert_eq!(
            profile(
                f,
                f.resource,
                caller,
                &input,
                &format!("matched_{mechanism}_expired_proof_retry")
            )
            .await
            .unwrap(),
            admitted
        );
        // A new signed proof within the same login is a fresh renewal, even
        // while its previous lease remains live. Exact retries above skip it.
        let mut renewal = original_claims;
        renewal.issued_at_ns = f.now().await;
        renewal.admission_expires_at_ns = f.now().await + 2 * SECOND;
        renewal.nonce[0] ^= 1;
        let renewal_input = make_input(&renewal).await;
        let renewed = profile(
            f,
            f.resource,
            caller,
            &renewal_input,
            &format!("matched_{mechanism}_renewal"),
        )
        .await
        .unwrap();
        assert!(renewed.expires_at_ns > admitted.expires_at_ns);
        admitted = renewed;
        assert_eq!(
            measured(
                f,
                f.resource,
                &format!("matched_{mechanism}_session_read"),
                async || read(f, f.resource, caller).await
            )
            .await
            .unwrap(),
            admitted
        );
        let hot: Result<SessionProfile, controls::Refusal> =
            query_control::<(), controls::model::SessionProfile>(
                f.pic.instance(),
                f.resource,
                caller,
                spec!(controls::ProfileSession),
                (),
            )
            .await;
        let hot = hot.unwrap();
        assert!(hot.authorization_instructions > 0);
        eprintln!("matched {mechanism} session profile: {hot:?}");
        measured(
            f,
            f.resource,
            &format!("matched_{mechanism}_logout"),
            async || logout(f, f.resource, caller).await,
        )
        .await
        .unwrap();
        assert_eq!(
            admission(f, f.resource, caller, &input).await,
            Err(AdmissionRefusal::Revoked)
        );
        assert!(
            read(f, f.signer, caller).await.is_ok(),
            "local logout preserves peer lease"
        );
        if !certified {
            qualify_wide_grants(f, &config, &key).await;
        }
    }
}

async fn components(f: &Fixture, caller: Principal, input: &SessionInput) {
    let call = async |caller, input: &SessionInput| {
        update_wire::<
            ic_auth_wire::SessionInput,
            Result<controls::model::CertifiedProfile, controls::model::CertifiedProfileFailure>,
        >(
            f.pic.instance(),
            PocketPrincipal::from_slice(f.resource.as_slice()),
            caller,
            spec!(controls::ProfileCertified),
            input,
        )
        .await
        .expect("bounded certified component probe")
    };
    for sample in 0..3 {
        let report = call(PocketPrincipal::from_slice(caller.as_slice()), input)
            .await
            .unwrap();
        assert!(report.credential_instructions > 0);
        assert!(report.root_delegation_instructions > 0);
        assert!(report.issuer_instructions > 0);
        assert!(report.decode_instructions > 0);
        assert!(report.tree_hash_instructions > 0);
        assert!(report.signing_bls_instructions > 0);
        assert!(
            report.subnet_bls_instructions > 0,
            "real application-subnet delegation"
        );
        eprintln!("certified components sample {sample}: {report:?}");
    }
    assert!(
        matches!(
            call(
                PocketPrincipal::from_slice(Principal::anonymous().as_slice()),
                input
            )
            .await,
            Err(controls::model::CertifiedProfileFailure::Verification(_))
        ),
        "caller binding precedes component results"
    );
    let mut invalid = input.clone();
    let ic_auth_wire::model::PortableProof::Certified(proof) = &mut invalid.credential.proof else {
        panic!("certified fixture");
    };
    proof.issuer_signature.signature_cbor = Bytes::new(vec![0]).unwrap();
    assert!(
        matches!(
            call(PocketPrincipal::from_slice(caller.as_slice()), &invalid).await,
            Err(controls::model::CertifiedProfileFailure::Verification(_))
        ),
        "complete verification precedes component results"
    );
}

// Same executable and endpoints as the one-scope sample, with the credential's
// maximum 32 aggregate scope entries across 16 grants and 16 requested
// scopes. Protected enrollment stays broader. Native signing is excluded.
async fn qualify_wide_grants(f: &Fixture, base: &ReceiverConfig, key: &SigningKey) {
    let mut config = base.clone();
    let scopes: Vec<String> = std::iter::once("read".into())
        .chain((0..31).map(|n| format!("scope{n:02}")))
        .collect();
    config.grants = ic_auth::portable::ValidatedGrants::new(
        (0..16)
            .map(|n| DelegatedRoleGrant {
                target: format!("role{n:02}").parse().unwrap(),
                scopes: scopes.iter().map(|s| s.parse().unwrap()).collect(),
            })
            .collect(),
    )
    .unwrap();
    config.role = config.grants.as_slice().last().unwrap().target.clone();
    reinstall_receiver(f, f.resource, &config).await;
    f.pic.instance().advance_time(Duration::from_secs(1)).await;
    for device in 30..33 {
        let mut claims = PortableClaims {
            mechanism: config.authority.mechanism(),
            audience: config.audience,
            issuer: config.issuer,
            key_binding_id: config.authority.binding(),
            browser_public_key: SigningKey::from_bytes(&[device; 32])
                .verifying_key()
                .to_bytes(),
            device_slot: 0,
            device_generation: 1,
            account_subject: Principal::from_slice(&[device]),
            login_started_at_ns: f.now().await,
            login_deadline_ns: f.now().await + 60 * SECOND,
            issued_at_ns: f.now().await,
            admission_expires_at_ns: f.now().await + 2 * SECOND,
            nonce: [device; 16],
            grants: config.grants.as_slice().to_vec(),
        };
        assert_eq!(
            encode_claims(&claims),
            Err(
                ic_auth::portable::PortableEncodingError::TooManyScopeEntries {
                    found: 512,
                    max: 32,
                }
            )
        );
        for (index, grant) in claims.grants.iter_mut().enumerate() {
            grant.scopes.truncate(if index == 15 { 17 } else { 1 });
        }
        let caller = browser_principal(&claims.browser_public_key);
        let mut input = input(&claims, key);
        input.scopes = List::new(
            claims.grants.last().unwrap().scopes[..16]
                .iter()
                .map(|scope| Text::new(scope.to_string()).unwrap())
                .collect(),
        )
        .unwrap();
        assert_eq!(input.scopes.as_slice().len(), 16);
        let admitted = profile(
            f,
            f.resource,
            caller,
            &input,
            &format!("wide_ed25519_fresh_{device}"),
        )
        .await
        .unwrap();
        assert_eq!(read(f, f.resource, caller).await.unwrap(), admitted);
        let hot = query_control::<(), controls::model::SessionProfile>(
            f.pic.instance(),
            f.resource,
            caller,
            spec!(controls::ProfileSession),
            (),
        )
        .await
        .unwrap();
        eprintln!("wide ed25519 session profile {device}: {hot:?}");
        assert!(hot.authorization_instructions > 0);
    }
}
