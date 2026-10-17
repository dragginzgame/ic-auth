use super::*;
use ic_auth::portable::{
    PortableClaimsContext, PortableClaimsError, browser_principal, encode_claims,
};
use ic_auth::portable_certified::{
    CERTIFIED_SIGNATURE_DOMAIN, PortableCertifiedContext, PortableCertifiedError,
    verify_certified_claims,
};

fn claims(f: &Fixture) -> PortableClaims {
    PortableClaims {
        mechanism: PortableMechanism::Certified,
        audience: audience(),
        issuer: f.proof.cert.issuer_pid,
        key_binding_id: cert_hash(&f.proof.cert).unwrap(),
        browser_public_key: [6; 32],
        account_subject: p(8),
        login_started_at_ns: 100,
        login_deadline_ns: 1000,
        issued_at_ns: 140,
        admission_expires_at_ns: 160,
        nonce: [7; 16],
        grants: vec![grant(&["read"])],
    }
}
fn context<'a>(f: &'a Fixture, trusted: &'a PortableClaims) -> PortableCertifiedContext<'a> {
    let cert = &f.proof.cert;
    let mut limits = limits();
    limits.max_future_skew_ns = 0;
    PortableCertifiedContext {
        claims: PortableClaimsContext {
            audience: &trusted.audience,
            issuer: cert.issuer_pid,
            mechanism: PortableMechanism::Certified,
            key_binding_id: trusted.key_binding_id,
            grants: &cert.grants,
            caller: browser_principal(&[6; 32]),
            now_ns: NOW,
            authority_not_before_ns: cert.not_before_ns,
            authority_expires_at_ns: cert.expires_at_ns,
            enrollment_expires_at_ns: 400,
            min_login_started_at_ns: 0,
            min_proof_issued_at_ns: 0,
            max_login_ttl_ns: 1000,
            max_proof_ttl_ns: 60,
        },
        root_key: &f.policy,
        ic_root_public_key_raw: &f.ic_root,
        limits,
    }
}
fn proof(c: &PortableClaims, domain: &[u8], seed: &[u8], time: u64) -> IcCanisterSignatureProofV1 {
    let digest = ic_auth::portable::claims_hash(c).unwrap();
    let message = domain_separated_message(domain, digest).unwrap();
    issuer_proof_message(c.issuer, &message, seed, time)
}

#[test]
fn complete_portable_chain_authenticates_distinct_account_and_browser() {
    let f = Fixture::new();
    let c = claims(&f);
    let signature = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let result = verify_certified_claims(
        &mut budget(),
        &encode_claims(&c).unwrap(),
        &f.proof,
        &signature,
        &context(&f, &c),
    )
    .unwrap();
    assert_eq!(result.claims(), &c);
    assert_eq!(
        result.fingerprint(),
        ic_auth::portable::claims_hash(&c).unwrap()
    );
    assert_eq!(result.verified_at_ns(), NOW);
    assert_ne!(c.account_subject, browser_principal(&c.browser_public_key));
}

#[test]
fn domain_seed_signature_and_claim_tampering_fail() {
    let f = Fixture::new();
    let c = claims(&f);
    let encoded = encode_claims(&c).unwrap();
    for signature in [
        proof(&c, SEED, SEED, NOW),
        proof(&c, CERTIFIED_SIGNATURE_DOMAIN, b"wrong-seed", NOW),
    ] {
        assert!(matches!(
            verify_certified_claims(
                &mut budget(),
                &encoded,
                &f.proof,
                &signature,
                &context(&f, &c)
            ),
            Err(PortableCertifiedError::Signature(_))
        ));
    }
    let mut signature = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let mut changed = c.clone();
    changed.nonce[0] ^= 1;
    assert!(matches!(
        verify_certified_claims(
            &mut budget(),
            &encode_claims(&changed).unwrap(),
            &f.proof,
            &signature,
            &context(&f, &c)
        ),
        Err(PortableCertifiedError::Signature(_))
    ));
    signature.signature_cbor[0] ^= 1;
    assert!(
        verify_certified_claims(
            &mut budget(),
            &encoded,
            &f.proof,
            &signature,
            &context(&f, &c)
        )
        .is_err()
    );
}

#[test]
fn root_proof_is_required_even_with_a_valid_issuer_signature() {
    let mut f = Fixture::new();
    let c = claims(&f);
    let signature = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
    root.signature.signature[0] ^= 1;
    assert!(matches!(
        verify_certified_claims(
            &mut budget(),
            &encode_claims(&c).unwrap(),
            &f.proof,
            &signature,
            &context(&f, &c)
        ),
        Err(PortableCertifiedError::Delegation(_))
    ));
}

#[test]
fn shared_checks_and_current_authority_cannot_be_bypassed_by_valid_signatures() {
    let f = Fixture::new();
    let c = claims(&f);
    let bytes = encode_claims(&c).unwrap();
    let signature = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    for (now, error) in [
        (139, PortableClaimsError::NotYetValid),
        (160, PortableClaimsError::Expired),
    ] {
        let mut ctx = context(&f, &c);
        ctx.claims.now_ns = now;
        assert_eq!(
            verify_certified_claims(&mut budget(), &bytes, &f.proof, &signature, &ctx),
            Err(PortableCertifiedError::Claims(error))
        );
    }
    let mut ctx = context(&f, &c);
    ctx.claims.caller = c.account_subject;
    assert_eq!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &signature, &ctx),
        Err(PortableCertifiedError::Claims(
            PortableClaimsError::CallerMismatch
        ))
    );
    ctx = context(&f, &c);
    ctx.claims.enrollment_expires_at_ns = NOW;
    assert_eq!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &signature, &ctx),
        Err(PortableCertifiedError::Claims(
            PortableClaimsError::StaleAuthority
        ))
    );
    ctx = context(&f, &c);
    ctx.claims.min_login_started_at_ns = 101;
    assert_eq!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &signature, &ctx),
        Err(PortableCertifiedError::Claims(
            PortableClaimsError::BeforeReset
        ))
    );
    ctx = context(&f, &c);
    ctx.limits.max_future_skew_ns = 1;
    assert_eq!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &signature, &ctx),
        Err(PortableCertifiedError::BindingMismatch)
    );
}

#[test]
fn delegation_ceiling_binding_and_certificate_time_are_enforced() {
    let f = Fixture::new();
    let c = claims(&f);
    let mut elevated = c.clone();
    elevated.grants[0].scopes.push("zz".into());
    let mut ctx = context(&f, &c);
    ctx.claims.grants = &elevated.grants;
    let sig = proof(&elevated, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    assert_eq!(
        verify_certified_claims(
            &mut budget(),
            &encode_claims(&elevated).unwrap(),
            &f.proof,
            &sig,
            &ctx
        ),
        Err(PortableCertifiedError::BindingMismatch)
    );
    let bytes = encode_claims(&c).unwrap();
    let sig = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let mut ctx = context(&f, &c);
    ctx.claims.authority_expires_at_ns += 1;
    assert_eq!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &sig, &ctx),
        Err(PortableCertifiedError::BindingMismatch)
    );
    for time in [NOW + 1, NOW - 101] {
        let sig = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, time);
        assert!(matches!(
            verify_certified_claims(&mut budget(), &bytes, &f.proof, &sig, &context(&f, &c)),
            Err(PortableCertifiedError::Signature(_))
        ));
    }
}

#[test]
fn aggregate_and_individual_bounds_precede_proof_work() {
    let f = Fixture::new();
    let c = claims(&f);
    let bytes = encode_claims(&c).unwrap();
    let sig = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let mut ctx = context(&f, &c);
    ctx.limits.max_variable_bytes = bytes.len();
    assert_eq!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &sig, &ctx),
        Err(PortableCertifiedError::Delegation(Error::InputTooLarge))
    );
    ctx = context(&f, &c);
    ctx.limits.max_issuer_signature_bytes = sig.signature_cbor.len() - 1;
    assert_eq!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &sig, &ctx),
        Err(PortableCertifiedError::Delegation(Error::InputTooLarge))
    );
    ctx = context(&f, &c);
    ctx.limits.max_variable_bytes = usize::MAX;
    let oversized = vec![0; 131_073];
    assert_eq!(
        verify_certified_claims(&mut budget(), &oversized, &f.proof, &sig, &ctx),
        Err(PortableCertifiedError::Delegation(Error::InputTooLarge))
    );
}

#[test]
fn current_root_policy_network_key_and_mechanism_are_not_proof_selected() {
    let mut f = Fixture::new();
    let c = claims(&f);
    let bytes = encode_claims(&c).unwrap();
    let sig = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    f.policy.min_accepted_proof_epoch = 3;
    assert!(matches!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &sig, &context(&f, &c)),
        Err(PortableCertifiedError::Delegation(_))
    ));
    f.policy.min_accepted_proof_epoch = 2;
    f.ic_root = bls_key(2).public_key().serialize();
    assert!(matches!(
        verify_certified_claims(&mut budget(), &bytes, &f.proof, &sig, &context(&f, &c)),
        Err(PortableCertifiedError::Signature(_))
    ));
    let mut other = c.clone();
    other.mechanism = PortableMechanism::Ed25519;
    assert_eq!(
        verify_certified_claims(
            &mut budget(),
            &encode_claims(&other).unwrap(),
            &f.proof,
            &sig,
            &context(&f, &c)
        ),
        Err(PortableCertifiedError::Claims(
            PortableClaimsError::MechanismMismatch
        ))
    );
}

#[test]
fn certified_result_uses_the_same_reservation_and_logout_gate() {
    use ic_auth::portable::{LoginRevocationError, MemoryLoginRevocations};
    let f = Fixture::new();
    let c = claims(&f);
    let sig = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let checked = verify_certified_claims(
        &mut budget(),
        &encode_claims(&c).unwrap(),
        &f.proof,
        &sig,
        &context(&f, &c),
    )
    .unwrap();
    let caller = browser_principal(&c.browser_public_key);
    let mut ledger = MemoryLoginRevocations::new(c.audience, 1, 1, 0, NOW).unwrap();
    ledger
        .with_reservation(caller, &checked, NOW, |_| Ok(()))
        .unwrap();
    ledger
        .with_revocation(&mut budget(), caller, NOW, |_| Ok(()))
        .unwrap();
    assert_eq!(
        ledger.with_reservation::<()>(caller, &checked, NOW, |_| panic!("post-logout retry")),
        Err(LoginRevocationError::Revoked)
    );
}

#[test]
fn certified_lease_uses_the_same_deadline_caps() {
    use ic_auth::session_request::SessionRequest;
    let f = Fixture::new();
    let c = claims(&f);
    let sig = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let encoded = encode_claims(&c).unwrap();
    let request = SessionRequest::new(vec!["read".into()], Some(1)).unwrap();
    let mut host = context(&f, &c);
    let checked = verify_certified_claims(&mut budget(), &encoded, &f.proof, &sig, &host).unwrap();
    assert_eq!(
        checked.session_expires_at_ns(&request, &role(), 1, 1800),
        Ok(400)
    );
    assert!(400 > c.admission_expires_at_ns);
    let write = SessionRequest::new(vec!["write".into()], Some(1)).unwrap();
    assert_eq!(
        checked.session_expires_at_ns(&write, &role(), 1, 1800),
        Err(ic_auth::session_request::SessionRequestError::ScopeRejected)
    );
    assert_eq!(
        checked.session_expires_at_ns(&request, &"other".parse().unwrap(), 1, 1800),
        Err(ic_auth::session_request::SessionRequestError::ScopeRejected)
    );
    host.claims.authority_expires_at_ns = 250;
    let shorter = verify_certified_claims(&mut budget(), &encoded, &f.proof, &sig, &host).unwrap();
    assert_eq!(
        shorter.session_expires_at_ns(&request, &role(), 1, 1800),
        Ok(250)
    );
    assert_eq!(
        checked.session_expires_at_ns(&request, &role(), 1, 1800),
        Ok(400)
    );
}

// Crypto fixtures prepare credit; budget behavior tests retain a single instance.
fn budget() -> ic_auth::admission_budget::SessionAdmissionBudget {
    ic_auth::admission_budget::SessionAdmissionBudget::new(4096, 1, 0).unwrap()
}

#[test]
fn certified_signature_failure_spends_credit_after_cheap_checks() {
    use ic_auth::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
    let f = Fixture::new();
    let c = claims(&f);
    let bytes = encode_claims(&c).unwrap();
    let mut host = context(&f, &c);
    let wrong = proof(&c, b"wrong-domain", SEED, NOW);
    let mut budget = SessionAdmissionBudget::new(1, 1, NOW - 1).unwrap();
    host.claims.caller = Principal::anonymous();
    assert!(matches!(
        verify_certified_claims(&mut budget, &bytes, &f.proof, &wrong, &host),
        Err(PortableCertifiedError::Claims(
            PortableClaimsError::CallerMismatch
        ))
    ));
    host = context(&f, &c);
    assert!(matches!(
        verify_certified_claims(&mut budget, &bytes, &f.proof, &wrong, &host),
        Err(PortableCertifiedError::Signature(_))
    ));
    let valid = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    assert_eq!(
        verify_certified_claims(&mut budget, &bytes, &f.proof, &valid, &host),
        Err(PortableCertifiedError::Budget(
            AdmissionBudgetError::Exhausted
        ))
    );
    host.claims.now_ns += 1;
    assert!(verify_certified_claims(&mut budget, &bytes, &f.proof, &valid, &host).is_ok());
}

#[cfg(feature = "portable-signature-verification")]
#[test]
fn switching_proof_mechanisms_cannot_obtain_another_allowance() {
    use ed25519_dalek::{Signer, SigningKey};
    use ic_auth::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
    use ic_auth::portable_signature::{PortableSignatureError, verify_ed25519_claims};
    use sha2::{Digest, Sha256};
    let f = Fixture::new();
    let c = claims(&f);
    let certified_host = context(&f, &c);
    let certified_bytes = encode_claims(&c).unwrap();
    let certified_signature = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let key = SigningKey::from_bytes(&[42; 32]);
    let public = key.verifying_key().to_bytes();
    let mut ed = c.clone();
    ed.mechanism = PortableMechanism::Ed25519;
    ed.key_binding_id = Sha256::digest(public).into();
    let ed_host = PortableClaimsContext {
        mechanism: ed.mechanism,
        key_binding_id: ed.key_binding_id,
        ..certified_host.claims
    };
    let ed_bytes = encode_claims(&ed).unwrap();
    let ed_signature = key
        .sign(&ic_auth::portable::claims_hash(&ed).unwrap())
        .to_bytes();
    for ed_first in [true, false] {
        let mut budget = SessionAdmissionBudget::new(1, 1, NOW - 1).unwrap();
        if ed_first {
            verify_ed25519_claims(&mut budget, &ed_bytes, &ed_signature, &ed_host, &public)
                .unwrap();
            assert_eq!(
                verify_certified_claims(
                    &mut budget,
                    &certified_bytes,
                    &f.proof,
                    &certified_signature,
                    &certified_host
                ),
                Err(PortableCertifiedError::Budget(
                    AdmissionBudgetError::Exhausted
                ))
            );
        } else {
            verify_certified_claims(
                &mut budget,
                &certified_bytes,
                &f.proof,
                &certified_signature,
                &certified_host,
            )
            .unwrap();
            assert_eq!(
                verify_ed25519_claims(&mut budget, &ed_bytes, &ed_signature, &ed_host, &public),
                Err(PortableSignatureError::Budget(
                    AdmissionBudgetError::Exhausted
                ))
            );
        }
    }
}

#[cfg(feature = "sessions")]
#[test]
fn certified_proof_uses_the_shared_session_lifecycle() {
    use ic_auth::{portable::MemoryLoginRevocations, session::*, session_request::SessionRequest};
    let f = Fixture::new();
    let c = claims(&f);
    let encoded = encode_claims(&c).unwrap();
    let signature = proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW);
    let credential = PortableCredential {
        claims: encoded.clone(),
        proof: PortableProof::Certified {
            delegation: Box::new(f.proof.clone()),
            issuer_signature: signature,
        },
    };
    let host = context(&f, &c);
    let mut ctx = SessionContext {
        authority: host.claims,
        role: &role(),
        required_scopes: &[],
    };
    let policy = SessionPolicy {
        generation: 0,
        enabled: true,
        subject_admissible: true,
        default_ttl_secs: 1,
        max_ttl_secs: 30,
    };
    let mut store = MemorySessionStore::new(
        0,
        SessionLimits {
            max_sessions: 2,
            max_replays: 4,
            max_replays_per_caller: 2,
            max_encoded_bytes: 8192,
        },
    )
    .unwrap();
    let mut ledger = MemoryLoginRevocations::new(c.audience, 2, 2, 0, 0).unwrap();
    let mut budget = budget();
    let request = SessionRequest::new(vec!["read".into()], None).unwrap();
    let SessionEstablished::Created(session) = establish_session(
        &mut store,
        &mut ledger,
        &mut budget,
        &encoded,
        &request,
        &ctx,
        policy,
        |budget, decoded| {
            Ok(decoded.verify(
                budget,
                &credential.proof,
                &ic_auth::portable_credential::PortableVerificationContext::Certified(&host),
            )?)
        },
    )
    .unwrap() else {
        panic!("created")
    };
    assert_eq!(session.subject(), c.account_subject);
    // A second receiver has independent session, revocation and budget custody.
    // The same signed envelope is admitted without issuing another credential.
    let mut second = MemorySessionStore::new(
        0,
        SessionLimits {
            max_sessions: 2,
            max_replays: 4,
            max_replays_per_caller: 2,
            max_encoded_bytes: 8192,
        },
    )
    .unwrap();
    let mut second_ledger = MemoryLoginRevocations::new(c.audience, 2, 2, 0, 0).unwrap();
    let mut second_budget =
        ic_auth::admission_budget::SessionAdmissionBudget::new(1, 1, NOW - 1).unwrap();
    let second_session = establish_session(
        &mut second,
        &mut second_ledger,
        &mut second_budget,
        &credential.claims,
        &request,
        &ctx,
        policy,
        |budget, decoded| {
            Ok(decoded.verify(
                budget,
                &credential.proof,
                &ic_auth::portable_credential::PortableVerificationContext::Certified(&host),
            )?)
        },
    )
    .unwrap();
    assert_eq!(second_session, SessionEstablished::Created(session.clone()));
    ctx.authority.now_ns = 170;
    assert_eq!(
        establish_session(
            &mut store,
            &mut ledger,
            &mut budget,
            &encoded,
            &request,
            &ctx,
            policy,
            |_, _| panic!("expired proof exact retry")
        ),
        Ok(SessionEstablished::ExactRetry(session.clone()))
    );
    assert_eq!(
        authorize_session(&mut store, &mut ledger, &ctx, policy),
        Ok(session.clone())
    );
    assert!(matches!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut budget,
            ctx.authority.caller,
            170,
        ),
        ic_auth_protocol_types::LogoutResult::Revoked { .. }
    ));
    assert!(authorize_session(&mut store, &mut ledger, &ctx, policy).is_err());
    assert_eq!(
        authorize_session(&mut second, &mut second_ledger, &ctx, policy),
        Ok(session)
    );
    assert_eq!(
        establish_session(
            &mut store,
            &mut ledger,
            &mut budget,
            &credential.claims,
            &request,
            &ctx,
            policy,
            |_, _| panic!("revoked login must fail before cryptography"),
        ),
        Err(SessionError::Revocation(
            ic_auth::portable::LoginRevocationError::Revoked
        ))
    );
}

#[test]
fn credential_variant_cannot_select_authority_or_spend_mismatched_budget() {
    use ic_auth::admission_budget::SessionAdmissionBudget;
    use ic_auth::portable_credential::{
        PortableCredentialError, PortableVerificationContext, verify_credential,
    };
    let f = Fixture::new();
    let c = claims(&f);
    let authority = context(&f, &c);
    let selected = PortableVerificationContext::Certified(&authority);
    let mut credential = PortableCredential {
        claims: encode_claims(&c).unwrap(),
        proof: PortableProof::Ed25519 {
            signature: vec![0; 64],
        },
    };
    let mut credit = SessionAdmissionBudget::new(1, 1, NOW - 1).unwrap();
    assert_eq!(
        verify_credential(&mut credit, &credential, &selected),
        Err(PortableCredentialError::MechanismMismatch)
    );
    let encoded = std::mem::replace(&mut credential.claims, vec![0]);
    assert_eq!(
        verify_credential(&mut credit, &credential, &selected),
        Err(PortableCredentialError::MechanismMismatch),
        "mechanism mismatch precedes decoding malformed claims"
    );
    credential.proof = PortableProof::Certified {
        delegation: Box::new(f.proof.clone()),
        issuer_signature: proof(&c, CERTIFIED_SIGNATURE_DOMAIN, SEED, NOW),
    };
    let mut bounded = context(&f, &c);
    bounded.limits.max_variable_bytes = 0;
    assert_eq!(
        verify_credential(
            &mut credit,
            &credential,
            &PortableVerificationContext::Certified(&bounded),
        ),
        Err(PortableCredentialError::Certified(
            PortableCertifiedError::Delegation(
                ic_auth::delegation::DelegationProofError::InputTooLarge
            )
        )),
        "aggregate proof bounds precede decoding malformed claims"
    );
    assert!(matches!(
        verify_credential(&mut credit, &credential, &selected),
        Err(PortableCredentialError::Certified(
            PortableCertifiedError::Encoding(_)
        ))
    ));
    credential.claims = encoded;
    assert_eq!(
        verify_credential(&mut credit, &credential, &selected)
            .unwrap()
            .claims(),
        &c
    );
    assert!(matches!(
        verify_credential(&mut credit, &credential, &selected),
        Err(PortableCredentialError::Certified(
            PortableCertifiedError::Budget(_)
        ))
    ));
    #[cfg(feature = "portable-signature-verification")]
    {
        let key = [0; 32];
        let wrong = PortableVerificationContext::Ed25519 {
            claims: &authority.claims,
            public_key: &key,
        };
        assert_eq!(
            verify_credential(&mut budget(), &credential, &wrong),
            Err(PortableCredentialError::MechanismMismatch)
        );
        // Both mechanisms draw from the same retained receiver-wide credit.
        use ed25519_dalek::{Signer, SigningKey};
        use sha2::{Digest, Sha256};
        let signing = SigningKey::from_bytes(&[42; 32]);
        let key = signing.verifying_key().to_bytes();
        let mut ed_claims = c.clone();
        ed_claims.mechanism = PortableMechanism::Ed25519;
        ed_claims.key_binding_id = Sha256::digest(key).into();
        let mut ed_authority = authority.claims;
        ed_authority.mechanism = PortableMechanism::Ed25519;
        ed_authority.key_binding_id = ed_claims.key_binding_id;
        let ed_context = PortableVerificationContext::Ed25519 {
            claims: &ed_authority,
            public_key: &key,
        };
        let ed_credential = PortableCredential {
            claims: encode_claims(&ed_claims).unwrap(),
            proof: PortableProof::Ed25519 {
                signature: signing
                    .sign(&ic_auth::portable::claims_hash(&ed_claims).unwrap())
                    .to_bytes()
                    .to_vec(),
            },
        };
        assert!(matches!(
            verify_credential(&mut credit, &ed_credential, &ed_context),
            Err(PortableCredentialError::Ed25519(
                ic_auth::portable_signature::PortableSignatureError::Budget(_)
            ))
        ));
        assert!(verify_credential(&mut budget(), &ed_credential, &ed_context).is_ok());
    }
}
