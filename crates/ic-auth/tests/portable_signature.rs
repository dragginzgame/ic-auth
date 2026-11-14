//! Strict Ed25519 issuer verification, protected key binding and failed-attempt budgeting.
#![cfg(feature = "portable-signature-verification")]

use ic_auth::portable::{
    MAX_CLAIMS_BYTES, PortableClaimsContext, PortableClaimsError as ClaimsError, ValidatedGrants,
    browser_principal, claims_hash, encode_claims,
};
use ic_auth::portable_signature::{PortableSignatureError as Error, verify_ed25519_claims};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};
use ic_platform::ed25519::{Signer, SigningKey};
use sha2::{Digest, Sha256};

fn hex<const N: usize>(value: &str) -> [u8; N] {
    let bytes: Vec<_> = value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    bytes.try_into().unwrap()
}

fn public_key() -> [u8; 32] {
    hex("03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8")
}

fn signature() -> [u8; 64] {
    hex(concat!(
        "c8e4c35496f1397a54faa16f8c438d3e7f238edea6bb289314fdbf32c33e1722",
        "e4ae146258e6abd7bcff480ba69567fb584149e38dbf0571d2ab80f9ebccf403",
    ))
}

fn claims() -> PortableClaims {
    PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([0x11; 32]),
            fleet_id: CanonicalId::from_bytes([0x22; 32]),
        },
        issuer: Principal::from_slice(&[1]),
        key_binding_id: Sha256::digest(public_key()).into(),
        browser_public_key: [0x44; 32],
        device_slot: 0,
        device_generation: 1,
        account_subject: Principal::from_slice(&[2]),
        login_started_at_ns: 10,
        login_deadline_ns: 100,
        issued_at_ns: 20,
        admission_expires_at_ns: 30,
        nonce: [0x55; 16],
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }],
    }
}

fn context<'a>(
    trusted: &'a PortableClaims,
    grants: &'a ValidatedGrants,
) -> PortableClaimsContext<'a> {
    PortableClaimsContext {
        authority: ic_auth::portable::PortableAuthority {
            audience: &trusted.audience,
            issuer: trusted.issuer,
            mechanism: PortableMechanism::Ed25519,
            key_binding_id: trusted.key_binding_id,
            grants,
            caller: browser_principal(&trusted.browser_public_key),
            now_ns: 25,
            authority_not_before_ns: 0,
            authority_expires_at_ns: 200,
            enrollment_expires_at_ns: 150,
            min_login_started_at_ns: 0,
        },
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 100,
        max_proof_ttl_ns: 60,
    }
}

fn verify(value: &PortableClaims, proof: &[u8]) -> Result<PortableClaims, Error> {
    let trusted = claims();
    verify_ed25519_claims(
        &mut budget(),
        &encode_claims(value).unwrap(),
        proof,
        &context(
            &trusted,
            &ValidatedGrants::new(trusted.grants.clone()).unwrap(),
        ),
        &public_key(),
    )
    .map(|verified| verified.claims().clone())
}

#[test]
fn independent_signature_vector_and_exact_signed_message() {
    // Perl pack and OpenSSL independently encoded and signed the schema-2 blob's
    // SHA-256 digest, using the test-only secret bytes 00..1f. No Rust signer or
    // encoder produced this frozen signature/hash vector.
    let value = claims();
    assert_eq!(
        claims_hash(&value).unwrap(),
        hex("55d41ea4b6d7f9ee02726c30241be325048c560a90261ebb8822c3745a0af057")
    );
    assert_eq!(verify(&value, &signature()).unwrap(), value);

    let signing = SigningKey::from_bytes(&std::array::from_fn(|i| i as u8));
    assert_eq!(signing.verifying_key().to_bytes(), public_key());
    let raw_blob_signature = signing.sign(&encode_claims(&value).unwrap());
    assert_eq!(
        verify(&value, &raw_blob_signature.to_bytes()),
        Err(Error::InvalidSignature)
    );
}

#[test]
fn protected_bindings_cannot_be_selected_by_the_proof() {
    let mut value = claims();
    value.mechanism = PortableMechanism::Certified;
    assert_eq!(
        verify(&value, &[]),
        Err(Error::Claims(ClaimsError::MechanismMismatch))
    );
    value = claims();
    value.audience.fleet_id = CanonicalId::from_bytes([9; 32]);
    assert_eq!(
        verify(&value, &[]),
        Err(Error::Claims(ClaimsError::AudienceMismatch))
    );
    value = claims();
    value.audience.canonical_network_id = CanonicalId::from_bytes([9; 32]);
    assert_eq!(
        verify(&value, &[]),
        Err(Error::Claims(ClaimsError::AudienceMismatch))
    );
    value = claims();
    value.issuer = Principal::anonymous();
    assert_eq!(
        verify(&value, &[]),
        Err(Error::Claims(ClaimsError::IssuerMismatch))
    );
    value = claims();
    value.key_binding_id[0] ^= 1;
    assert_eq!(
        verify(&value, &[]),
        Err(Error::Claims(ClaimsError::KeyMismatch))
    );

    let trusted = claims();
    assert_eq!(
        verify_ed25519_claims(
            &mut budget(),
            &encode_claims(&trusted).unwrap(),
            &signature(),
            &context(
                &trusted,
                &ValidatedGrants::new(trusted.grants.clone()).unwrap()
            ),
            &[42; 32]
        ),
        Err(Error::Claims(ClaimsError::KeyMismatch))
    );
}

#[test]
fn signed_claim_changes_fail_and_success_does_not_imply_admission() {
    let mutations: [fn(&mut PortableClaims); 8] = [
        |c| c.browser_public_key[0] ^= 1,
        |c| c.account_subject = Principal::from_slice(&[3]),
        |c| c.login_started_at_ns += 1,
        |c| c.login_deadline_ns += 1,
        |c| c.issued_at_ns += 1,
        |c| c.admission_expires_at_ns += 1,
        |c| c.nonce[0] ^= 1,
        |c| c.grants[0].scopes[0] = "write".parse().unwrap(),
    ];
    for change in mutations {
        let mut value = claims();
        change(&mut value);
        assert!(matches!(
            verify(&value, &signature()),
            Err(Error::InvalidSignature)
                | Err(Error::Claims(
                    ClaimsError::CallerMismatch | ClaimsError::GrantsExceeded
                ))
        ));
    }
    // A valid signature cannot override invalid time bounds.
    let mut value = claims();
    value.login_deadline_ns = 0;
    let signing = SigningKey::from_bytes(&std::array::from_fn(|i| i as u8));
    let proof = signing.sign(&claims_hash(&value).unwrap());
    assert_eq!(
        verify(&value, &proof.to_bytes()),
        Err(Error::Claims(ClaimsError::InvalidWindow))
    );
}

fn noncanonical_scalar(mut bad: [u8; 64]) -> [u8; 64] {
    let order: [u8; 32] = hex("edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010");
    let mut carry = 0u16;
    for (byte, add) in bad[32..].iter_mut().zip(order) {
        let sum = u16::from(*byte) + u16::from(add) + carry;
        *byte = sum as u8;
        carry = sum >> 8;
    }
    bad
}

#[test]
fn strict_signature_lengths_scalars_and_small_order_points() {
    for len in [0, 1, 32, 63, 65, 128] {
        assert_eq!(
            verify(&claims(), &vec![0; len]),
            Err(Error::MalformedSignature)
        );
    }
    for byte in 0..64 {
        let mut bad = signature();
        bad[byte] ^= 0x80;
        assert_eq!(verify(&claims(), &bad), Err(Error::InvalidSignature));
    }
    // S + the group order encodes the same scalar modulo l, but must fail even
    // when another consumer enables Dalek's legacy_compatibility feature.
    let mut bad = noncanonical_scalar(signature());
    assert_eq!(verify(&claims(), &bad), Err(Error::InvalidSignature));
    bad = signature();
    bad[32..].fill(255);
    assert_eq!(verify(&claims(), &bad), Err(Error::InvalidSignature));
    bad = signature();
    bad[..32].fill(0);
    bad[0] = 1; // identity R
    assert_eq!(verify(&claims(), &bad), Err(Error::InvalidSignature));

    for first_byte in [0, 1] {
        let mut weak_key = [0; 32];
        weak_key[0] = first_byte;
        let mut value = claims();
        value.key_binding_id = Sha256::digest(weak_key).into();
        assert_eq!(
            verify_ed25519_claims(
                &mut budget(),
                &encode_claims(&value).unwrap(),
                &signature(),
                &context(&value, &ValidatedGrants::new(value.grants.clone()).unwrap()),
                &weak_key
            ),
            Err(Error::InvalidPublicKey)
        );
    }
}

#[test]
fn malformed_claims_never_reach_signature_verification() {
    let value = claims();
    let valid = encode_claims(&value).unwrap();
    let mut trailing = valid.clone();
    trailing.push(0);
    let mut excess_grants = valid.clone();
    excess_grants.extend(vec![0; ic_auth::portable::MAX_CREDENTIAL_GRANT_BYTES]);
    for bytes in [
        &valid[..valid.len() - 1],
        &trailing,
        &excess_grants,
        &vec![0; MAX_CLAIMS_BYTES + 1],
    ] {
        assert!(matches!(
            verify_ed25519_claims(
                &mut budget(),
                bytes,
                &[],
                &context(&value, &ValidatedGrants::new(value.grants.clone()).unwrap()),
                &public_key()
            ),
            Err(Error::Encoding(_))
        ));
    }
}

#[test]
fn portable_lease_uses_login_and_captured_authority_not_proof_expiry() {
    use ic_auth::session_request::{SessionRequest, SessionRequestError};
    let request = SessionRequest::new(vec!["read".into()], Some(1)).unwrap();
    let key = SigningKey::from_bytes(&std::array::from_fn(|i| i as u8));
    let mut value = claims();
    value.login_deadline_ns = 10_000_000_000;
    let signature = key.sign(&claims_hash(&value).unwrap()).to_bytes();
    let checked_grants = ValidatedGrants::new(value.grants.clone()).unwrap();
    let mut host = context(&value, &checked_grants);
    host.max_login_ttl_ns = 10_000_000_000;
    host.authority.authority_expires_at_ns = 9_000_000_000;
    host.authority.enrollment_expires_at_ns = 8_000_000_000;
    let encoded = encode_claims(&value).unwrap();
    let checked =
        verify_ed25519_claims(&mut budget(), &encoded, &signature, &host, &public_key()).unwrap();
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(1, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&request, &"user".parse().unwrap(), ttl)),
        Ok(1_000_000_025)
    );
    // The proof expires at 30ns, but its admitted lease may live longer.
    assert!(
        ic_auth::session_request::SessionTtlPolicy::new(1, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&request, &"user".parse().unwrap(), ttl))
            .unwrap()
            > value.admission_expires_at_ns
    );
    let longer = SessionRequest::new(vec!["read".into()], None).unwrap();
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(30, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&longer, &"user".parse().unwrap(), ttl)),
        Ok(8_000_000_000)
    );
    host.authority.enrollment_expires_at_ns = 9_500_000_000;
    let refreshed =
        verify_ed25519_claims(&mut budget(), &encoded, &signature, &host, &public_key()).unwrap();
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(30, 1800).and_then(
            |ttl| refreshed.session_expires_at_ns(&longer, &"user".parse().unwrap(), ttl)
        ),
        Ok(9_000_000_000)
    );
    // A refreshed snapshot does not modify previously checked admission terms.
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(30, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&longer, &"user".parse().unwrap(), ttl)),
        Ok(8_000_000_000)
    );
    host.authority.authority_expires_at_ns = 20_000_000_000;
    host.authority.enrollment_expires_at_ns = 20_000_000_000;
    let login_limited =
        verify_ed25519_claims(&mut budget(), &encoded, &signature, &host, &public_key()).unwrap();
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(30, 1800).and_then(
            |ttl| login_limited.session_expires_at_ns(&longer, &"user".parse().unwrap(), ttl)
        ),
        Ok(value.login_deadline_ns)
    );
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(0, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&request, &"user".parse().unwrap(), ttl)),
        Err(SessionRequestError::InvalidLifetime)
    );
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(1801, 1801)
            .and_then(|ttl| checked.session_expires_at_ns(&longer, &"user".parse().unwrap(), ttl)),
        Err(SessionRequestError::InvalidLifetime)
    );
}

// Crypto fixtures prepare credit; budget behavior tests retain a single instance.
fn budget() -> ic_auth::admission_budget::SessionAdmissionBudget {
    ic_auth::admission_budget::SessionAdmissionBudget::for_verification(
        4096,
        1,
        ic_auth::admission_budget::VerificationCosts {
            ed25519: 1,
            certified: 1,
        },
        0,
    )
    .unwrap()
}

#[test]
fn failed_signatures_consume_shared_credit_and_cheap_refusals_do_not() {
    use ic_auth::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
    let c = claims();
    let bytes = encode_claims(&c).unwrap();
    let checked_grants = ValidatedGrants::new(c.grants.clone()).unwrap();
    let mut host = context(&c, &checked_grants);
    let mut budget = SessionAdmissionBudget::for_verification(
        1,
        5,
        ic_auth::admission_budget::VerificationCosts {
            ed25519: 1,
            certified: 1,
        },
        20,
    )
    .unwrap();
    // A caller mismatch is rejected without spending the available attempt.
    host.authority.caller = Principal::anonymous();
    assert_eq!(
        verify_ed25519_claims(&mut budget, &bytes, &signature(), &host, &public_key()),
        Err(Error::Claims(ClaimsError::CallerMismatch))
    );
    let checked_grants = ValidatedGrants::new(c.grants.clone()).unwrap();
    host = context(&c, &checked_grants);
    // Validly encoded signature for a different digest reaches cryptography.
    let bad = SigningKey::from_bytes(&[7; 32]).sign(&[9; 32]).to_bytes();
    assert_eq!(
        verify_ed25519_claims(&mut budget, &bytes, &bad, &host, &public_key()),
        Err(Error::InvalidSignature)
    );
    // Switching callers does not reset the receiver-wide allowance.
    let mut other = c.clone();
    other.browser_public_key = [8; 32];
    let checked_grants = ValidatedGrants::new(other.grants.clone()).unwrap();
    let other_host = context(&other, &checked_grants);
    assert_eq!(
        verify_ed25519_claims(
            &mut budget,
            &encode_claims(&other).unwrap(),
            &bad,
            &other_host,
            &public_key()
        ),
        Err(Error::Budget(AdmissionBudgetError::Exhausted))
    );
    host.authority.now_ns = 24;
    assert_eq!(
        verify_ed25519_claims(&mut budget, &bytes, &signature(), &host, &public_key()),
        Err(Error::Budget(AdmissionBudgetError::ClockRollback))
    );
}

#[test]
fn successful_verification_and_failed_storage_do_not_refund_credit() {
    use ic_auth::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
    use ic_auth::portable::{LoginRevocationError, MemoryLoginRevocations};
    let c = claims();
    let bytes = encode_claims(&c).unwrap();
    let checked_grants = ValidatedGrants::new(c.grants.clone()).unwrap();
    let host = context(&c, &checked_grants);
    let mut budget = SessionAdmissionBudget::for_verification(
        1,
        5,
        ic_auth::admission_budget::VerificationCosts {
            ed25519: 1,
            certified: 1,
        },
        20,
    )
    .unwrap();
    let checked =
        verify_ed25519_claims(&mut budget, &bytes, &signature(), &host, &public_key()).unwrap();
    let mut ledger = MemoryLoginRevocations::new(c.audience, 1, 1, 86_400_000_000_000, 0).unwrap();
    assert_eq!(
        ledger.with_reservation::<(), ic_auth::portable::LoginRevocationError>(
            host.authority.caller,
            &checked,
            host.authority.now_ns,
            |_| Err(LoginRevocationError::Storage)
        ),
        Err(LoginRevocationError::Storage)
    );
    assert_eq!(ledger.occupancy(), (0, 0));
    assert_eq!(
        verify_ed25519_claims(&mut budget, &bytes, &signature(), &host, &public_key()),
        Err(Error::Budget(AdmissionBudgetError::Exhausted))
    );
    // Reconstructing at current time never grants a fresh initial burst.
    let mut reset = SessionAdmissionBudget::for_verification(
        100,
        5,
        ic_auth::admission_budget::VerificationCosts {
            ed25519: 1,
            certified: 1,
        },
        host.authority.now_ns,
    )
    .unwrap();
    assert_eq!(
        verify_ed25519_claims(&mut reset, &bytes, &signature(), &host, &public_key()),
        Err(Error::Budget(AdmissionBudgetError::Exhausted))
    );
}

#[test]
fn session_request_narrows_one_protected_role_without_combining_grants() {
    use ic_auth::session_request::{SessionRequest, SessionRequestError};
    let key = SigningKey::from_bytes(&std::array::from_fn(|i| i as u8));
    let mut c = claims();
    c.grants = vec![
        DelegatedRoleGrant {
            target: "admin".parse().unwrap(),
            scopes: vec!["write".parse().unwrap()],
        },
        DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap(), "view".parse().unwrap()],
        },
    ];
    let encoded = encode_claims(&c).unwrap();
    let signature = key.sign(&claims_hash(&c).unwrap()).to_bytes();
    let checked = verify_ed25519_claims(
        &mut budget(),
        &encoded,
        &signature,
        &context(&c, &ValidatedGrants::new(c.grants.clone()).unwrap()),
        &public_key(),
    )
    .unwrap();
    let user = "user".parse().unwrap();
    let admin = "admin".parse().unwrap();
    let read = SessionRequest::new(vec!["read".into()], Some(1)).unwrap();
    let write = SessionRequest::new(vec!["write".into()], Some(1)).unwrap();
    let union = SessionRequest::new(vec!["read".into(), "write".into()], Some(1)).unwrap();
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(1, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&read, &user, ttl)),
        Ok(100)
    );
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(1, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&write, &admin, ttl)),
        Ok(100)
    );
    for (request, role) in [
        (&write, &user),
        (&read, &admin),
        (&union, &user),
        (&union, &admin),
    ] {
        assert_eq!(
            ic_auth::session_request::SessionTtlPolicy::new(1, 1800)
                .and_then(|ttl| checked.session_expires_at_ns(request, role, ttl)),
            Err(SessionRequestError::ScopeRejected)
        );
    }
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(1, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&read, &"unknown".parse().unwrap(), ttl)),
        Err(SessionRequestError::ScopeRejected)
    );
    let reordered = SessionRequest::new(vec!["view".into(), "read".into()], Some(1)).unwrap();
    assert_eq!(
        ic_auth::session_request::SessionTtlPolicy::new(1, 1800)
            .and_then(|ttl| checked.session_expires_at_ns(&reordered, &user, ttl)),
        Ok(100)
    );
    // Narrowing never rewrites signed grants, fingerprints or request identity.
    assert_eq!(checked.claims(), &c);
    assert_eq!(checked.fingerprint(), claims_hash(&c).unwrap());
    assert_eq!(
        read.scopes()
            .iter()
            .map(|scope| scope.as_str())
            .collect::<Vec<_>>(),
        ["read"]
    );
}

#[test]
fn credential_dispatch_preserves_signed_bytes_and_protected_key_checks() {
    use ic_auth::portable_credential::{
        PortableCredentialError, PortableVerificationContext, verify_credential,
    };
    use ic_auth_protocol_types::{PortableCredential, PortableProof};
    let trusted = claims();
    let checked_grants = ValidatedGrants::new(trusted.grants.clone()).unwrap();
    let authority = context(&trusted, &checked_grants);
    let key = public_key();
    let selected = PortableVerificationContext::Ed25519 {
        claims: &authority,
        public_key: &ic_auth::portable_signature::PreparedIssuerKey::new(key).unwrap(),
    };
    let mut credential = PortableCredential {
        claims: encode_claims(&trusted).unwrap(),
        proof: PortableProof::Ed25519 {
            signature: signature().to_vec(),
        },
    };
    assert_eq!(
        verify_credential(&mut budget(), &credential, &selected)
            .unwrap()
            .claims(),
        &trusted
    );
    credential.proof = PortableProof::Ed25519 {
        signature: vec![0; 63],
    };
    assert_eq!(
        verify_credential(&mut budget(), &credential, &selected),
        Err(PortableCredentialError::Ed25519(Error::MalformedSignature))
    );
    credential.proof = PortableProof::Ed25519 {
        signature: signature().to_vec(),
    };
    credential.claims.push(0);
    assert!(matches!(
        verify_credential(&mut budget(), &credential, &selected),
        Err(PortableCredentialError::Ed25519(Error::Encoding(_)))
    ));
    credential.claims = encode_claims(&trusted).unwrap();
    let wrong_key = [3; 32];
    let wrong = PortableVerificationContext::Ed25519 {
        claims: &authority,
        public_key: &ic_auth::portable_signature::PreparedIssuerKey::new(wrong_key).unwrap(),
    };
    assert!(verify_credential(&mut budget(), &credential, &wrong).is_err());
}

#[test]
fn prepared_issuer_key_retains_original_identity_and_refuses_weak_material() {
    use ic_auth::portable_signature::PreparedIssuerKey;
    let original = public_key();
    let mut supplied = original;
    let admitted = PreparedIssuerKey::new(supplied).unwrap();
    supplied.fill(0);
    assert_eq!(admitted.bytes(), &original);
    assert_eq!(
        admitted.binding(),
        <[u8; 32]>::from(Sha256::digest(original))
    );
    assert_eq!(
        PreparedIssuerKey::new(supplied),
        Err(Error::InvalidPublicKey)
    );
}

// This entrypoint is also exercised by the private feature-unification consumer.
// Ledger maintenance/restore tests remain in their normal suite, not this graph.
#[cfg(feature = "sessions")]
#[test]
fn session_admission_refuses_noncanonical_scalar_before_installing_state() {
    use ic_auth::{
        portable::MemoryLoginRevocations,
        portable_credential::{PortableCredentialError, PortableVerificationContext},
        session::{
            MemorySessionStore, SessionAdmissionContext, SessionError, SessionLimits,
            SessionPolicy, SessionReader, establish_session,
        },
        session_request::{SessionRequest, SessionTtlPolicy},
    };
    use ic_auth_protocol_types::{PortableCredential, PortableProof};
    let c = claims();
    let grants = ValidatedGrants::new(c.grants.clone()).unwrap();
    let ctx = context(&c, &grants);
    let key = ic_auth::portable_signature::PreparedIssuerKey::new(public_key()).unwrap();
    let admission = SessionAdmissionContext {
        verification: PortableVerificationContext::Ed25519 {
            claims: &ctx,
            public_key: &key,
        },
        role: &c.grants[0].target,
        ttl: SessionTtlPolicy::new(1, 1).unwrap(),
    };
    let mut store = MemorySessionStore::new(0, SessionLimits::new(1, 1, 1, 4096).unwrap());
    let mut ledger = MemoryLoginRevocations::new(c.audience, 1, 1, 100, 0).unwrap();
    let mut credit = budget();
    for (proof, accepted) in [
        (noncanonical_scalar(signature()), false),
        (signature(), true),
    ] {
        let credential = PortableCredential {
            claims: encode_claims(&c).unwrap(),
            proof: PortableProof::Ed25519 {
                signature: proof.to_vec(),
            },
        };
        let result = establish_session(
            &mut store,
            &mut ledger,
            &mut credit,
            &credential,
            SessionRequest::new(vec!["read".into()], None).unwrap(),
            &admission,
            SessionPolicy {
                generation: 0,
                enabled: true,
                subject_admissible: true,
            },
        );
        if accepted {
            assert!(result.is_ok());
            assert_eq!(ledger.occupancy(), (1, 0));
            assert!(store.session(ctx.authority.caller).unwrap().is_some());
        } else {
            assert_eq!(
                result,
                Err(SessionError::Credential(PortableCredentialError::Ed25519(
                    Error::InvalidSignature
                )))
            );
            assert_eq!(ledger.occupancy(), (0, 0));
            assert_eq!(store.encoded_bytes(), 0);
            assert!(store.session(ctx.authority.caller).unwrap().is_none());
        }
    }
}
