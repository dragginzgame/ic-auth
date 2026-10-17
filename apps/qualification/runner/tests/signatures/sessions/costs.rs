//! Matched receiver execution; native credential production is outside the sample.
use super::*;
use ic_auth::token::TokenVerificationContext;

fn profile(
    f: &Fixture,
    target: Principal,
    caller: Principal,
    input: &SessionInput,
    label: &str,
) -> AdmissionProfile {
    let report: AdmissionProfile = measured(f, target, label, || {
        f.pic
            .update_candid_as(
                target,
                caller,
                "profile_portable_admission",
                (input.clone(),),
                CONTROL_REPLY_LIMITS,
            )
            .expect("profile Candid transport")
    });
    assert!(report.instructions > 0);
    eprintln!(
        "portable receiver admission instructions: {label}={}",
        report.instructions
    );
    report
}

pub(super) fn qualify(f: &Fixture, token: &DelegatedToken, old: &TokenVerificationContext<'_>) {
    eprintln!(
        "matched receiver artifact: bytes={}, sha256={:x}; same application subnet, endpoint, grants, lease and volatile store; three fresh logins per mechanism; native issuance excluded",
        f.wasm.len(),
        Sha256::digest(&f.wasm)
    );
    let cert = &token.proof.cert;
    let key = SigningKey::from_bytes(&[42; 32]);
    for certified in [false, true] {
        let mechanism = if certified { "certified" } else { "ed25519" };
        let config = ReceiverConfig {
            audience: old.audience,
            issuer: cert.issuer_pid,
            authority: if certified {
                let root = RootEnrollment::from(old.root_key);
                assert_eq!(root.policy(), *old.root_key, "fixture enrollment roundtrip");
                ReceiverAuthority::Certified {
                    binding: ic_auth::canonical::cert_hash(cert).unwrap(),
                    root: Box::new(root),
                    network_key: old.ic_root_public_key_raw.to_vec(),
                }
            } else {
                ReceiverAuthority::Ed25519(key.verifying_key().to_bytes())
            },
            role: old.role.clone(),
            grants: cert.grants.clone(),
            valid_from_ns: cert.not_before_ns,
            valid_until_ns: cert.expires_at_ns.min(old.root_key.accept_until_ns),
            enrollment_until_ns: cert.expires_at_ns.min(old.root_key.accept_until_ns),
            verification_burst: 16,
            verification_refill_ns: 1,
        };
        let make_claims = |device: u8| PortableClaims {
            mechanism: config.authority.mechanism(),
            audience: config.audience,
            issuer: config.issuer,
            key_binding_id: config.authority.binding(),
            browser_public_key: SigningKey::from_bytes(&[device; 32])
                .verifying_key()
                .to_bytes(),
            account_subject: f.owner,
            login_started_at_ns: f.now(),
            login_deadline_ns: f.now() + 60 * SECOND,
            issued_at_ns: f.now(),
            admission_expires_at_ns: f.now() + 2 * SECOND,
            nonce: [device; 16],
            grants: cert.grants.clone(),
        };
        let make_input = |claims: &PortableClaims| {
            if !certified {
                return input(claims, &key);
            }
            let digest = claims_hash(claims).unwrap();
            let domain = ic_auth::portable_certified::CERTIFIED_SIGNATURE_DOMAIN;
            f.prepare(domain, &digest, 60 * SECOND).unwrap();
            SessionInput {
                credential: PortableCredential {
                    claims: encode_claims(claims).unwrap(),
                    proof: PortableProof::Certified {
                        delegation: Box::new(token.proof.clone()),
                        issuer_signature: f.retrieve(domain, &digest).unwrap(),
                    },
                },
                scopes: vec!["read".into()],
                ttl_secs: None,
            }
        };
        // A valid-point, wrong-network certificate reaches BLS verification.
        // The Ed25519 case corrupts the issuer signature. Neither creates a session.
        let mut rejecting = config.clone();
        rejecting.verification_burst = 1;
        rejecting.verification_refill_ns = SECOND;
        if let ReceiverAuthority::Certified { network_key, .. } = &mut rejecting.authority {
            *network_key = ic_canister_sig_creation::extract_raw_root_pk_from_der(
                ic_canister_sig_creation::IC_ROOT_PK_DER,
            )
            .unwrap();
            assert_ne!(network_key, old.ic_root_public_key_raw);
        }
        install(f, f.resource, &rejecting);
        f.pic.advance_time(Duration::from_secs(1));
        let rejected_claims = make_claims(19);
        let rejected_caller = browser_principal(&rejected_claims.browser_public_key);
        let mut rejected = make_input(&rejected_claims);
        if let PortableProof::Ed25519 { signature } = &mut rejected.credential.proof {
            signature[0] ^= 1;
        }
        let label = format!("matched_{mechanism}_invalid_proof");
        assert!(
            profile(f, f.resource, rejected_caller, &rejected, &label)
                .result
                .unwrap_err()
                .contains("signature")
        );
        assert!(read(f, f.resource, rejected_caller).is_err());
        let another = make_claims(18);
        let another_caller = browser_principal(&another.browser_public_key);
        let another_input = make_input(&another);
        let throttled = profile(
            f,
            f.resource,
            another_caller,
            &another_input,
            &format!("matched_{mechanism}_throttled_other_caller"),
        );
        assert!(
            throttled
                .result
                .unwrap_err()
                .contains("verification budget is exhausted")
        );
        assert!(read(f, f.resource, another_caller).is_err());
        // Fresh-verification exhaustion cannot consume the separate logout budget.
        logout(f, f.resource, rejected_caller).unwrap();

        // Reinstall into fresh fixture state for identical valid-admission occupancy.
        // This is an explicit test reset, never production enrollment refresh.
        install(f, f.resource, &config);
        install(f, f.signer, &config);
        f.pic.advance_time(Duration::from_secs(1));
        let mut first = None;
        for device in 20..23 {
            let claims = make_claims(device);
            let caller = browser_principal(&claims.browser_public_key);
            let input = make_input(&claims);
            let admitted = profile(
                f,
                f.resource,
                caller,
                &input,
                &format!("matched_{mechanism}_fresh_{device}"),
            )
            .result
            .unwrap();
            assert_eq!(
                (admitted.caller, admitted.issuer, admitted.subject),
                (caller, config.issuer, f.owner)
            );
            assert!(admitted.expires_at_ns <= claims.login_deadline_ns);
            if device == 20 {
                let peer = profile(
                    f,
                    f.signer,
                    caller,
                    &input,
                    &format!("matched_{mechanism}_second_receiver"),
                )
                .result
                .unwrap();
                assert_eq!(peer.subject, admitted.subject);
                first = Some((caller, input, admitted));
            }
        }
        let (caller, input, admitted) = first.unwrap();
        f.pic.advance_time(Duration::from_secs(3));
        assert_eq!(
            profile(
                f,
                f.resource,
                caller,
                &input,
                &format!("matched_{mechanism}_expired_proof_retry")
            )
            .result
            .unwrap(),
            admitted
        );
        assert_eq!(
            measured(
                f,
                f.resource,
                &format!("matched_{mechanism}_session_read"),
                || read(f, f.resource, caller)
            )
            .unwrap(),
            admitted
        );
        let hot: Result<SessionProfile, String> = f
            .pic
            .query_candid_as(
                f.resource,
                caller,
                "profile_portable_session",
                (),
                CONTROL_REPLY_LIMITS,
            )
            .unwrap();
        let hot = hot.unwrap();
        assert!(hot.authorization_instructions > 0);
        eprintln!("matched {mechanism} session profile: {hot:?}");
        measured(
            f,
            f.resource,
            &format!("matched_{mechanism}_logout"),
            || logout(f, f.resource, caller),
        )
        .unwrap();
        assert!(
            admission(f, f.resource, caller, &input)
                .unwrap_err()
                .contains("revoked")
        );
        assert!(
            read(f, f.signer, caller).is_ok(),
            "local logout preserves peer lease"
        );
    }
}
