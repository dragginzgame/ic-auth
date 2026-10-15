use ic_auth::{canonical::*, token_preparation::*};
use ic_auth_protocol_types::*;

fn p(byte: u8) -> Principal {
    Principal::from_slice(&[byte; 29])
}

fn grant(role: &str, scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role.parse().unwrap(),
        scopes: scopes.iter().map(|s| s.to_string()).collect(),
    }
}

// Opaque Root material is intentionally unauthenticated: this pure boundary
// attaches an installed host proof; it cannot authorize it or sign anything.
fn proof() -> DelegationProof {
    let binding = IssuerProofBinding::IcCanisterSignatureV1 { seed_hash: [3; 32] };
    let cert = DelegationCert {
        root_pid: p(1),
        issuer_pid: p(2),
        issuer_proof_alg: IssuerProofAlgorithm::IcCanisterSignatureV1,
        issuer_proof_binding_hash: issuer_proof_binding_hash(
            p(2),
            IssuerProofAlgorithm::IcCanisterSignatureV1,
            binding,
        )
        .unwrap(),
        issuer_proof_binding: binding,
        issued_at_ns: 100,
        not_before_ns: 100,
        expires_at_ns: 500,
        max_token_ttl_ns: 120,
        aud: DelegationAudience::Fleet(AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        }),
        grants: vec![grant("app_service", &["read", "write"])],
    };
    let key_id = ChainKeyKeyId {
        name: "test_key_1".into(),
    };
    let root_proof = RootProof::IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1 {
        header: ChainKeyBatchHeaderV1 {
            schema_version: 1,
            root_canister_id: p(1),
            batch_id: [1; 32],
            proof_epoch: 2,
            registry_epoch: 3,
            registry_hash: [4; 32],
            tree_root: [5; 32],
            not_before_ns: 100,
            expires_at_ns: 500,
            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
            key_id: key_id.clone(),
            derivation_path_hash: [6; 32],
            key_version: 7,
        },
        delegation_cert: ChainKeyDelegationCertV1 {
            root_canister_id: p(1),
            issuer_canister_id: p(2),
            proof_epoch: 2,
            issuer_proof_algorithm: cert.issuer_proof_alg,
            issuer_proof_binding_hash: cert.issuer_proof_binding_hash,
            issuer_proof_binding: binding,
            max_token_ttl_ns: 120,
            audience: cert.aud.clone(),
            grants: cert.grants.clone(),
            not_before_ns: 100,
            expires_at_ns: 500,
            registry_epoch: 3,
            registry_hash: [4; 32],
        },
        issuer_witness: ChainKeyBatchWitnessV1 { steps: vec![] },
        signature: ChainKeyRootSignatureV1 {
            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
            key_id,
            derivation_path: vec![b"canic".to_vec()],
            public_key: vec![7; 33],
            signature: vec![8; 64],
        },
    });
    DelegationProof { cert, root_proof }
}

fn input(proof: &DelegationProof) -> PrepareDelegatedTokenInput<'_> {
    PrepareDelegatedTokenInput {
        proof,
        authority_expires_at_ns: 500,
        operation_id: [4; 32],
        prepared_by: p(9),
        audience: proof.cert.aud.clone(),
        grants: vec![grant("app_service", &["read"])],
        ttl_ns: 60,
        ext: Some(vec![1, 2]),
        now_ns: 120,
        limits: TokenPreparationLimits {
            max_grants: 16,
            max_scopes_per_grant: 32,
        },
    }
}

fn hash_vector(value: &str) -> [u8; 32] {
    *value.parse::<CanonicalId>().unwrap().as_bytes()
}

#[test]
fn frozen_canic_nonce_claim_bytes_and_hash_survive_preparation_and_finishing() {
    let proof = proof();
    let prepared = prepare_delegated_token(input(&proof)).unwrap();
    assert_eq!(prepared.claims().presenter, p(9));
    assert_eq!(prepared.claims().subject, p(9));
    assert_eq!(prepared.claims().issuer_pid, p(2));
    // Independently framed frozen vectors, including raw (unprefixed) principal
    // bytes in the nonce domain, BE lengths/times and extension presence.
    assert_eq!(
        prepared.claims().cert_hash,
        hash_vector("84bc591011b0b30d301f904618ba512271746cd762a2a4275822bc12bf466f0b")
    );
    assert_eq!(
        prepared.claims().nonce,
        0x2d9bf3fbd2e1e7957a66fede5be606ba_u128.to_be_bytes()
    );
    assert_eq!(
        prepared.claims_hash(),
        hash_vector("3b4f15a80a8bd56d4cc8c6fe34727a0adc8bb532fdbc737995e5af308a434ce2")
    );
    let bytes = claims_bytes(prepared.claims()).unwrap();
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(
        hex,
        concat!(
            "43414e49432d4155544800020000001d0909090909090909090909090909090909090909090909090909090909",
            "0000001d0909090909090909090909090909090909090909090909090909090909",
            "0000001d0202020202020202020202020202020202020202020202020202020202",
            "84bc591011b0b30d301f904618ba512271746cd762a2a4275822bc12bf466f0b",
            "000000000000007800000000000000b4010101010101010101010101010101010101010101010101010101010101010101",
            "0202020202020202020202020202020202020202020202020202020202020202",
            "000000010000000b6170705f73657276696365000000010000000472656164",
            "2d9bf3fbd2e1e7957a66fede5be606ba01000000020102"
        )
    );
    assert_eq!(prepared.proof(), &proof);
    let claims = prepared.claims().clone();
    let issuer = IssuerProof::IcCanisterSignatureV1(IcCanisterSignatureProofV1 {
        public_key_der: vec![],
        signature_cbor: vec![99],
    });
    let token = finish_delegated_token(prepared, issuer.clone());
    assert_eq!(token.claims, claims);
    assert_eq!(token.proof, proof);
    assert_eq!(token.issuer_proof, issuer);
}

#[test]
fn retained_retry_is_deterministic_and_each_nonce_binding_matters() {
    let proof = proof();
    let expected = prepare_delegated_token(input(&proof)).unwrap();
    assert_eq!(prepare_delegated_token(input(&proof)).unwrap(), expected);
    let mut changed = input(&proof);
    changed.operation_id[0] ^= 1;
    assert_ne!(
        prepare_delegated_token(changed).unwrap().claims().nonce,
        expected.claims().nonce
    );
    let mut changed = input(&proof);
    changed.prepared_by = p(10);
    assert_ne!(
        prepare_delegated_token(changed).unwrap().claims().nonce,
        expected.claims().nonce
    );
    for issuer_changed in [false, true] {
        let mut changed_proof = proof.clone();
        if issuer_changed {
            changed_proof.cert.issuer_pid = p(3);
        } else {
            changed_proof.cert.expires_at_ns += 1;
        }
        assert_ne!(
            prepare_delegated_token(input(&changed_proof))
                .unwrap()
                .claims()
                .nonce,
            expected.claims().nonce
        );
    }
    let mut changed = input(&proof);
    changed.now_ns += 1;
    let later = prepare_delegated_token(changed).unwrap();
    assert_eq!(later.claims().nonce, expected.claims().nonce);
    assert_ne!(later.claims_hash(), expected.claims_hash());
    // The host must retain the original clock/request or the prepared value.
}

#[test]
fn exclusive_authority_window_and_requested_ttl_are_not_silently_rewritten() {
    use PrepareDelegatedTokenError as E;
    let proof = proof();
    for (now, deadline, ttl, error) in [
        (99, 500, 0, E::CertNotYetValid),
        (120, 120, 0, E::CertExpired),
        (500, 1000, 1, E::CertExpired),
        (120, 179, 60, E::TokenOutlivesCert),
        (490, 1000, 11, E::TokenOutlivesCert),
    ] {
        let mut request = input(&proof);
        request.now_ns = now;
        request.authority_expires_at_ns = deadline;
        request.ttl_ns = ttl;
        assert_eq!(prepare_delegated_token(request).unwrap_err(), error);
    }
    for (now, deadline, ttl, expiry) in [
        (100, 500, 120, 220),
        (120, 180, 60, 180),
        (499, 1000, 1, 500),
    ] {
        let mut request = input(&proof);
        request.now_ns = now;
        request.authority_expires_at_ns = deadline;
        request.ttl_ns = ttl;
        let prepared = prepare_delegated_token(request).unwrap();
        assert_eq!(prepared.claims().expires_at_ns, expiry);
        assert_eq!(prepared.proof(), &proof);
    }
}

#[test]
fn zero_ttl_limit_and_arithmetic_overflow_have_stable_precedence() {
    use PrepareDelegatedTokenError as E;
    let mut proof = proof();
    proof.cert.expires_at_ns = u64::MAX;
    for (ttl, expected) in [
        (0, E::TokenTtlZero),
        (
            121,
            E::TokenTtlExceeded {
                ttl_ns: 121,
                max_ttl_ns: 120,
            },
        ),
        (2, E::TokenExpiresAtOverflow),
    ] {
        let mut request = input(&proof);
        request.now_ns = u64::MAX - 1;
        request.authority_expires_at_ns = u64::MAX;
        request.ttl_ns = ttl;
        request.grants.clear();
        assert_eq!(prepare_delegated_token(request).unwrap_err(), expected);
    }
    let mut request = input(&proof);
    request.now_ns = u64::MAX - 1;
    request.authority_expires_at_ns = u64::MAX;
    request.ttl_ns = 1;
    assert_eq!(
        prepare_delegated_token(request)
            .unwrap()
            .claims()
            .expires_at_ns,
        u64::MAX
    );
}

#[test]
fn request_shape_and_bounds_reject_before_audience_narrowing() {
    use PrepareDelegatedTokenError as E;
    let proof = proof();
    let cases = [
        (vec![], E::GrantsEmpty),
        (
            vec![grant("app_service", &[])],
            E::EmptyGrantScopes {
                role: "app_service".parse().unwrap(),
            },
        ),
        (
            vec![grant("app_service", &[""])],
            E::Canonical(CanonicalAuthError::EmptyScope),
        ),
        (
            vec![grant("app_service", &["Read"])],
            E::Canonical(CanonicalAuthError::InvalidScope {
                scope: "Read".into(),
            }),
        ),
        (
            vec![grant("app_service", &["read", "read"])],
            E::Canonical(CanonicalAuthError::NonCanonicalScopes),
        ),
        (
            vec![grant("app_service", &["write", "read"])],
            E::Canonical(CanonicalAuthError::NonCanonicalScopes),
        ),
        (
            vec![
                grant("app_service", &["read"]),
                grant("app_service", &["read"]),
            ],
            E::Canonical(CanonicalAuthError::NonCanonicalRoles),
        ),
        (
            vec![
                grant("other_service", &["read"]),
                grant("app_service", &["read"]),
            ],
            E::Canonical(CanonicalAuthError::NonCanonicalRoles),
        ),
    ];
    for (grants, expected) in cases {
        let mut request = input(&proof);
        request.grants = grants;
        request.audience = DelegationAudience::Fleet(AudienceId {
            canonical_network_id: CanonicalId::from_bytes([0; 32]),
            fleet_id: CanonicalId::from_bytes([0; 32]),
        });
        assert_eq!(prepare_delegated_token(request).unwrap_err(), expected);
    }
    let mut request = input(&proof);
    request.limits.max_grants = 0;
    assert_eq!(
        prepare_delegated_token(request).unwrap_err(),
        E::TooManyGrants { found: 1, max: 0 }
    );
    let mut request = input(&proof);
    request.grants[0].scopes = vec!["bad".into(); 33];
    assert_eq!(
        prepare_delegated_token(request).unwrap_err(),
        E::TooManyGrantScopes {
            role: "app_service".parse().unwrap(),
            found: 33,
            max: 32
        }
    );
}

#[test]
fn host_bounds_accept_exact_limits_and_preserve_extension_presence() {
    let mut proof = proof();
    proof
        .cert
        .grants
        .push(grant("other_service", &["read", "write"]));
    let prepare = |ext| {
        let mut request = input(&proof);
        request.grants = proof.cert.grants.clone();
        request.limits = TokenPreparationLimits {
            max_grants: 2,
            max_scopes_per_grant: 2,
        };
        request.ext = ext;
        prepare_delegated_token(request).unwrap()
    };
    let absent = prepare(None);
    let present = prepare(Some(vec![]));
    assert_eq!(absent.claims().grants, proof.cert.grants);
    assert_eq!(present.claims().grants, proof.cert.grants);
    assert_eq!(absent.claims().nonce, present.claims().nonce);
    assert_ne!(absent.claims_hash(), present.claims_hash());
    assert_eq!(absent.claims().ext, None);
    assert_eq!(present.claims().ext, Some(vec![]));
}

#[test]
fn audience_then_grant_narrowing_then_canonical_extension_have_stable_precedence() {
    use PrepareDelegatedTokenError as E;
    let proof = proof();
    let mut request = input(&proof);
    request.grants = vec![grant("app_service", &["admin"])];
    request.ext = Some(vec![0; MAX_TOKEN_EXT_BYTES + 1]);
    request.audience = DelegationAudience::Fleet(AudienceId {
        canonical_network_id: CanonicalId::from_bytes([0; 32]),
        fleet_id: CanonicalId::from_bytes([0; 32]),
    });
    assert_eq!(
        prepare_delegated_token(request).unwrap_err(),
        E::AudienceNotSubset
    );
    for grants in [
        vec![grant("app_service", &["admin"])],
        vec![grant("other_service", &["read"])],
    ] {
        let mut request = input(&proof);
        request.grants = grants;
        request.ext = Some(vec![0; MAX_TOKEN_EXT_BYTES + 1]);
        assert_eq!(
            prepare_delegated_token(request).unwrap_err(),
            E::GrantsNotSubset
        );
    }
    let mut request = input(&proof);
    request.ext = Some(vec![0; MAX_TOKEN_EXT_BYTES + 1]);
    assert_eq!(
        prepare_delegated_token(request).unwrap_err(),
        E::Canonical(CanonicalAuthError::TokenExtTooLarge {
            len: MAX_TOKEN_EXT_BYTES + 1,
            max: MAX_TOKEN_EXT_BYTES
        })
    );
    let mut request = input(&proof);
    request.ext = Some(vec![0; MAX_TOKEN_EXT_BYTES]);
    assert!(prepare_delegated_token(request).is_ok());
}

#[test]
fn duplicate_parent_roles_preserve_first_match_narrowing_before_canonical_refusal() {
    use PrepareDelegatedTokenError as E;
    let mut proof = proof();
    proof.cert.grants = vec![
        grant("app_service", &["write"]),
        grant("app_service", &["read"]),
    ];
    let mut request = input(&proof);
    request.ext = Some(vec![0; MAX_TOKEN_EXT_BYTES + 1]);
    assert_eq!(
        prepare_delegated_token(request).unwrap_err(),
        E::GrantsNotSubset
    );

    proof.cert.grants.reverse();
    let mut request = input(&proof);
    request.ext = Some(vec![0; MAX_TOKEN_EXT_BYTES + 1]);
    assert_eq!(
        prepare_delegated_token(request).unwrap_err(),
        E::Canonical(CanonicalAuthError::NonCanonicalRoles)
    );
}
