use super::*;
use ic_auth::delegation::{DelegationField, DelegationTarget};
use ic_auth::delegation::{
    DelegationProofVerificationContext, VerifiedDelegationProof, verify_delegation_proof,
};

fn context(f: &Fixture) -> DelegationProofVerificationContext<'_> {
    DelegationProofVerificationContext {
        expected_issuer: p(2),
        root_key: &f.policy,
        now_ns: NOW,
        limits: f.context().limits,
    }
}

fn verify(f: &Fixture) -> Result<VerifiedDelegationProof<'_>, Error> {
    verify_delegation_proof(&f.proof, &context(f))
}

#[test]
fn root_delegation_authenticates_without_claims_or_an_issuer_signature() {
    let f = Fixture::new();
    let proof = f.proof.clone();
    let result = verify_delegation_proof(&proof, &context(&f)).unwrap();
    assert_eq!(result.certificate(), &proof.cert);
    assert_eq!(result.certificate_hash(), cert_hash(&proof.cert).unwrap());
    assert_eq!(result.expires_at_ns(), 500);
}

#[test]
fn expected_issuer_and_independent_root_policy_are_required() {
    let f = Fixture::new();
    let mut ctx = context(&f);
    for issuer in [p(9), Principal::anonymous()] {
        ctx.expected_issuer = issuer;
        assert_eq!(
            verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
            Error::BindingMismatch {
                field: DelegationField::IssuerCanisterId
            }
        );
    }
    for field in [DelegationField::Root, DelegationField::Issuer] {
        let mut f = Fixture::new();
        if field == DelegationField::Root {
            f.proof.cert.root_pid = Principal::anonymous();
        } else {
            f.proof.cert.issuer_pid = Principal::anonymous();
        }
        assert_eq!(verify(&f).unwrap_err(), Error::AnonymousPrincipal { field });
    }
    let mut f = Fixture::new();
    f.proof.cert.root_pid = p(9);
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::BindingMismatch {
            field: DelegationField::RootCanisterId
        }
    );
}

#[test]
fn certificate_shape_and_canonical_grants_are_checked_before_root_crypto() {
    for (grants, error) in [
        (vec![], Error::InvalidGrants),
        (vec![grant(&[])], Error::InvalidGrants),
        (vec![grant(&["read"]); 17], Error::InvalidGrants),
        (
            vec![grant(&["write", "read"])],
            Error::Canonical(CanonicalAuthError::NonCanonicalScopes),
        ),
        (
            vec![grant(&["read"]); 2],
            Error::Canonical(CanonicalAuthError::NonCanonicalRoles),
        ),
        (
            vec![grant(&["Read"])],
            Error::Canonical(CanonicalAuthError::Scope(
                ic_auth_protocol_types::ScopeLabelError::InvalidSegmentStart { offset: 0 },
            )),
        ),
    ] {
        let mut f = Fixture::new();
        f.proof.cert.grants = grants;
        assert_eq!(verify(&f).unwrap_err(), error);
    }
    let mut f = Fixture::new();
    f.proof.cert.issued_at_ns = 101;
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::BindingMismatch {
            field: DelegationField::CertIssuedAt
        }
    );
    f.proof.cert.issued_at_ns = 100;
    f.proof.cert.issuer_proof_binding_hash[0] ^= 1;
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::BindingMismatch {
            field: DelegationField::IssuerProofBindingHash
        }
    );
}

#[test]
fn standalone_certificate_ttl_and_time_use_explicit_exact_limits() {
    let f = Fixture::new();
    let mut ctx = context(&f);
    ctx.limits.max_cert_ttl_ns = 400;
    ctx.limits.max_token_ttl_ns = 100;
    assert!(verify_delegation_proof(&f.proof, &ctx).is_ok());
    ctx.limits.max_cert_ttl_ns = 399;
    assert_eq!(
        verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
        Error::TtlExceeded {
            target: DelegationTarget::Certificate,
            ttl_ns: 400,
            max_ttl_ns: 399
        }
    );
    ctx.limits.max_cert_ttl_ns = 400;
    ctx.limits.max_token_ttl_ns = 99;
    assert_eq!(
        verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
        Error::TtlExceeded {
            target: DelegationTarget::MaxTokenTtl,
            ttl_ns: 100,
            max_ttl_ns: 99
        }
    );
    ctx.limits.max_token_ttl_ns = 100;
    ctx.now_ns = 90;
    assert!(verify_delegation_proof(&f.proof, &ctx).is_ok());
    ctx.now_ns = 89;
    assert_eq!(
        verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
        Error::NotYetValid {
            target: DelegationTarget::Certificate
        }
    );
    ctx.now_ns = 499;
    assert!(verify_delegation_proof(&f.proof, &ctx).is_ok());
    ctx.now_ns = 500;
    assert_eq!(
        verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
        Error::Expired {
            target: DelegationTarget::Certificate
        }
    );
    ctx.now_ns = u64::MAX;
    assert_eq!(
        verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
        Error::Expired {
            target: DelegationTarget::Certificate
        }
    );
    for end in [99, 100] {
        let mut f = Fixture::new();
        f.proof.cert.expires_at_ns = end;
        assert_eq!(
            verify(&f).unwrap_err(),
            Error::InvalidWindow {
                target: DelegationTarget::Certificate
            }
        );
    }
    let mut f = Fixture::new();
    f.proof.cert.max_token_ttl_ns = 0;
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::InvalidWindow {
            target: DelegationTarget::MaxTokenTtl
        }
    );
    f.proof.cert.expires_at_ns = 190;
    f.proof.cert.max_token_ttl_ns = 100;
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::TtlExceeded {
            target: DelegationTarget::MaxTokenTtl,
            ttl_ns: 100,
            max_ttl_ns: 90
        }
    );
}

#[test]
fn every_variable_proof_component_and_witness_count_is_bounded_before_validation() {
    // Actual fixture material, independently counted for an inclusive boundary.
    let f = Fixture::new();
    let mut ctx = context(&f);
    // Two copies of grants (target + role entry + scopes + scope entries),
    // two witness steps, two key names, SEC1/signature bytes and path components.
    let bytes = 2 * (1 + "app_service".len() + 2 + "read".len() + "write".len())
        + 2 * 33
        + 2 * "test_key_1".len()
        + 33
        + 64
        + 2
        + "canic".len()
        + "delegation".len();
    ctx.limits.max_variable_bytes = bytes;
    ctx.limits.max_witness_steps = 2;
    assert!(verify_delegation_proof(&f.proof, &ctx).is_ok());
    ctx.limits.max_variable_bytes -= 1;
    assert_eq!(
        verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
        Error::InputTooLarge
    );
    ctx.limits.max_variable_bytes = usize::MAX;
    ctx.limits.max_witness_steps = 1;
    assert_eq!(
        verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
        Error::InputTooLarge
    );
    for field in [
        "cert_grants",
        "leaf_grants",
        "header_key",
        "signature_key",
        "key",
        "signature",
        "path",
        "path_entry",
        "witness",
    ] {
        let mut f = Fixture::new();
        let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
        match field {
            "cert_grants" => f.proof.cert.grants[0].scopes.push("a".repeat(2000)),
            "leaf_grants" => root.delegation_cert.grants[0].scopes.push("a".repeat(2000)),
            "header_key" => root.header.key_id.name = "a".repeat(2000),
            "signature_key" => root.signature.key_id.name = "a".repeat(2000),
            "key" => root.signature.public_key.resize(2000, 0),
            "signature" => root.signature.signature.resize(2000, 0),
            "path" => root.signature.derivation_path.push(vec![0; 2000]),
            "path_entry" => root.signature.derivation_path.resize(2000, vec![]),
            _ => root
                .issuer_witness
                .steps
                .resize(2000, ChainKeyBatchWitnessStepV1::LeftSibling([0; 32])),
        }
        let mut ctx = context(&f);
        ctx.limits.max_variable_bytes = 1000;
        ctx.limits.max_witness_steps = usize::MAX;
        assert_eq!(
            verify_delegation_proof(&f.proof, &ctx).unwrap_err(),
            Error::InputTooLarge,
            "{field}"
        );
    }
}

#[test]
fn previously_valid_root_proof_is_rejected_after_live_policy_changes() {
    let mut f = Fixture::new();
    assert!(verify(&f).is_ok());
    f.policy.accept_until_ns = 200;
    assert_eq!(verify(&f).unwrap().expires_at_ns(), 200);
    f.policy.accept_until_ns = NOW;
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::Expired {
            target: DelegationTarget::RootKeyPolicy
        }
    );
    f.policy.accept_until_ns = 1000;
    f.policy.valid_from_ns = NOW + 11;
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::NotYetValid {
            target: DelegationTarget::RootKeyPolicy
        }
    );
    for field in [
        DelegationField::KeyVersion,
        DelegationField::ProofEpoch,
        DelegationField::RegistryEpoch,
    ] {
        let mut f = Fixture::new();
        assert!(verify(&f).is_ok());
        match field {
            DelegationField::KeyVersion => f.policy.min_accepted_key_version += 1,
            DelegationField::ProofEpoch => f.policy.min_accepted_proof_epoch += 1,
            _ => f.policy.min_accepted_registry_epoch += 1,
        }
        assert_eq!(verify(&f).unwrap_err(), Error::StaleAuthority { field });
    }
    let mut f = Fixture::new();
    f.policy.max_revocation_latency_ns = 399;
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::TtlExceeded {
            target: DelegationTarget::Batch,
            ttl_ns: 400,
            max_ttl_ns: 399
        }
    );
}

#[test]
fn root_rejection_contracts_cover_bound_metadata_and_crypto() {
    for (field, error) in [
        (
            "schema",
            Error::BindingMismatch {
                field: DelegationField::SchemaVersion,
            },
        ),
        (
            "root",
            Error::BindingMismatch {
                field: DelegationField::RootCanisterId,
            },
        ),
        (
            "issuer",
            Error::BindingMismatch {
                field: DelegationField::IssuerCanisterId,
            },
        ),
        (
            "seed",
            Error::BindingMismatch {
                field: DelegationField::IssuerProofBinding,
            },
        ),
        (
            "epoch",
            Error::BindingMismatch {
                field: DelegationField::ProofEpoch,
            },
        ),
        (
            "registry",
            Error::BindingMismatch {
                field: DelegationField::RegistryHash,
            },
        ),
        (
            "audience",
            Error::BindingMismatch {
                field: DelegationField::LeafAudience,
            },
        ),
        (
            "grant",
            Error::BindingMismatch {
                field: DelegationField::LeafGrants,
            },
        ),
        (
            "ttl",
            Error::BindingMismatch {
                field: DelegationField::MaxTokenTtl,
            },
        ),
        (
            "key_name",
            Error::RootSignatureBindingMismatch {
                field: DelegationField::KeyId,
            },
        ),
        (
            "key_version",
            Error::BindingMismatch {
                field: DelegationField::KeyVersion,
            },
        ),
        (
            "key",
            Error::BindingMismatch {
                field: DelegationField::RootPublicKey,
            },
        ),
        (
            "path",
            Error::RootSignatureBindingMismatch {
                field: DelegationField::DerivationPathHash,
            },
        ),
        ("witness", Error::InvalidMerkleWitness),
        ("header", Error::RootSignatureInvalid),
        ("signature", Error::RootSignatureInvalid),
    ] {
        let mut f = Fixture::new();
        let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
        match field {
            "schema" => root.header.schema_version = 2,
            "root" => root.header.root_canister_id = p(9),
            "issuer" => root.delegation_cert.issuer_canister_id = p(9),
            "seed" => {
                root.delegation_cert.issuer_proof_binding =
                    IssuerProofBinding::IcCanisterSignatureV1 { seed_hash: [0; 32] }
            }
            "epoch" => root.delegation_cert.proof_epoch += 1,
            "registry" => root.delegation_cert.registry_hash[0] ^= 1,
            "audience" => {
                root.delegation_cert.audience = DelegationAudience::Fleet(AudienceId {
                    canonical_network_id: CanonicalId::from_bytes([9; 32]),
                    fleet_id: CanonicalId::from_bytes([9; 32]),
                })
            }
            "grant" => root.delegation_cert.grants = vec![grant(&["read"])],
            "ttl" => root.delegation_cert.max_token_ttl_ns -= 1,
            "key_name" => root.signature.key_id.name.push('x'),
            "key_version" => root.header.key_version += 1,
            "key" => root.signature.public_key[0] ^= 1,
            "path" => root.signature.derivation_path[0].push(0),
            "witness" => {
                root.issuer_witness.steps[0] = ChainKeyBatchWitnessStepV1::RightSibling([42; 32])
            }
            "header" => root.header.batch_id[0] ^= 1,
            _ => root.signature.signature[0] ^= 1,
        }
        assert_eq!(verify(&f).unwrap_err(), error, "standalone {field}");
    }
}

#[test]
fn real_root_signature_requires_valid_sec1_scalars_and_low_s() {
    for signature in [vec![0; 64], vec![1; 63], vec![0xff; 64]] {
        let mut f = Fixture::new();
        let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
        root.signature.signature = signature;
        assert_eq!(verify(&f).unwrap_err(), Error::RootSignatureInvalid);
    }
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
    let sig = Signature::from_slice(&root.signature.signature).unwrap();
    let (r, s) = sig.split_scalars();
    root.signature.signature = Signature::from_scalars(r.to_bytes(), (-*s).to_bytes())
        .unwrap()
        .to_bytes()
        .to_vec();
    assert_eq!(verify(&f).unwrap_err(), Error::HighSSignature);
    let mut f = Fixture::new();
    f.policy.public_key = vec![0; 33];
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
    root.signature.public_key = f.policy.public_key.clone();
    assert_eq!(verify(&f).unwrap_err(), Error::RootSignatureInvalid);
}

#[test]
fn root_authorized_other_issuer_still_cannot_install_on_this_host() {
    let mut f = Fixture::new();
    f.proof.cert.issuer_pid = p(9);
    f.proof.cert.issuer_proof_binding_hash = issuer_proof_binding_hash(
        p(9),
        f.proof.cert.issuer_proof_alg,
        f.proof.cert.issuer_proof_binding,
    )
    .unwrap();
    f.proof.root_proof = root_proof(&f.proof.cert, &f.policy);
    assert_eq!(
        verify(&f).unwrap_err(),
        Error::BindingMismatch {
            field: DelegationField::IssuerCanisterId
        }
    );
    let mut ctx = context(&f);
    ctx.expected_issuer = p(9);
    assert!(verify_delegation_proof(&f.proof, &ctx).is_ok());
}

#[test]
fn root_and_leaf_windows_cannot_override_the_certificate_or_enrollment() {
    for (field, error) in [
        (
            "batch_start",
            Error::NotYetValid {
                target: DelegationTarget::Batch,
            },
        ),
        (
            "batch_end",
            Error::Expired {
                target: DelegationTarget::Batch,
            },
        ),
        (
            "batch_invalid",
            Error::InvalidWindow {
                target: DelegationTarget::Batch,
            },
        ),
        (
            "leaf_start",
            Error::NotYetValid {
                target: DelegationTarget::IssuerLeaf,
            },
        ),
        (
            "leaf_end",
            Error::Expired {
                target: DelegationTarget::IssuerLeaf,
            },
        ),
        (
            "leaf_batch",
            Error::BindingMismatch {
                field: DelegationField::LeafBatchWindow,
            },
        ),
        (
            "leaf_cert",
            Error::BindingMismatch {
                field: DelegationField::LeafCertWindow,
            },
        ),
        (
            "policy_invalid",
            Error::InvalidWindow {
                target: DelegationTarget::RootKeyPolicy,
            },
        ),
    ] {
        let mut f = Fixture::new();
        let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
        match field {
            "batch_start" => root.header.not_before_ns = NOW + 11,
            "batch_end" => root.header.expires_at_ns = NOW,
            "batch_invalid" => root.header.expires_at_ns = root.header.not_before_ns,
            "leaf_start" => root.delegation_cert.not_before_ns = NOW + 11,
            "leaf_end" => root.delegation_cert.expires_at_ns = NOW,
            "leaf_batch" => root.delegation_cert.not_before_ns = 99,
            "leaf_cert" => root.delegation_cert.not_before_ns = 101,
            _ => f.policy.valid_from_ns = f.policy.accept_until_ns,
        }
        assert_eq!(verify(&f).unwrap_err(), error, "{field}");
    }
}

#[test]
fn certificate_issue_metadata_is_checked_but_not_part_of_the_signed_root_leaf() {
    let mut f = Fixture::new();
    let hash = verify(&f).unwrap().certificate_hash();
    f.proof.cert.issued_at_ns = 99;
    let result = verify(&f).unwrap();
    assert_ne!(hash, result.certificate_hash());
    assert_eq!(result.certificate_hash(), cert_hash(&f.proof.cert).unwrap());
    // This changes the certificate hash but not the authority signed by the root.
    // Portable claims bind the exact certificate separately through key_binding_id.
}
