use ic_auth::{canonical::*, certificate_issuance::*};
use ic_auth_protocol_types::*;

fn p(byte: u8) -> Principal {
    Principal::from_slice(&[byte; 29])
}

fn grant(role: &str, scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role.parse().unwrap(),
        scopes: scopes.iter().map(|scope| scope.to_string()).collect(),
    }
}

fn cert() -> DelegationCert {
    let binding = IssuerProofBinding::IcCanisterSignatureV1 { seed_hash: [3; 32] };
    DelegationCert {
        root_pid: p(1),
        issuer_pid: p(2),
        issuer_proof_alg: IssuerProofAlgorithm::IcCanisterSignatureV1,
        issuer_proof_binding: binding,
        issuer_proof_binding_hash: issuer_proof_binding_hash(
            p(2),
            IssuerProofAlgorithm::IcCanisterSignatureV1,
            binding,
        )
        .unwrap(),
        issued_at_ns: 100,
        not_before_ns: 100,
        expires_at_ns: 500,
        max_token_ttl_ns: 120,
        aud: DelegationAudience::Fleet(AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        }),
        grants: vec![grant("app_service", &["read", "write"])],
    }
}

fn limits() -> CertificateIssuanceLimits {
    CertificateIssuanceLimits {
        max_cert_ttl_ns: 600,
        max_token_ttl_ns: 120,
        max_grants: 16,
        max_scopes_per_grant: 32,
    }
}

fn validate(cert: &DelegationCert) -> Result<(), CertificateIssuanceError> {
    validate_certificate_issuance(cert, p(1), limits())
}

#[test]
fn validation_preserves_frozen_certificate_hash_and_every_field() {
    let cert = cert();
    let original = cert.clone();
    let bytes = cert_bytes(&cert).unwrap();
    validate(&cert).unwrap();
    assert_eq!(cert, original);
    assert_eq!(cert_bytes(&cert).unwrap(), bytes);
    assert_eq!(
        cert_hash(&cert).unwrap(),
        *"84bc591011b0b30d301f904618ba512271746cd762a2a4275822bc12bf466f0b"
            .parse::<CanonicalId>()
            .unwrap()
            .as_bytes()
    );
}

#[test]
fn exact_host_bounds_and_delayed_validity_start_are_accepted() {
    let mut cert = cert();
    cert.issued_at_ns = 99;
    cert.max_token_ttl_ns = 400;
    validate_certificate_issuance(
        &cert,
        p(1),
        CertificateIssuanceLimits {
            max_cert_ttl_ns: 400,
            max_token_ttl_ns: 400,
            max_grants: 1,
            max_scopes_per_grant: 2,
        },
    )
    .unwrap();
    // Checked subtraction must also admit a valid window at the numeric ceiling.
    cert.issued_at_ns = u64::MAX - 400;
    cert.not_before_ns = cert.issued_at_ns;
    cert.expires_at_ns = u64::MAX;
    validate_certificate_issuance(
        &cert,
        p(1),
        CertificateIssuanceLimits {
            max_cert_ttl_ns: 400,
            max_token_ttl_ns: 400,
            max_grants: 1,
            max_scopes_per_grant: 2,
        },
    )
    .unwrap();
}

#[test]
fn malformed_windows_and_zero_host_bounds_refuse() {
    use CertificateIssuanceError as E;
    for (issued, start, end) in [(101, 100, 500), (100, 100, 99), (100, 100, 100)] {
        let mut cert = cert();
        cert.issued_at_ns = issued;
        cert.not_before_ns = start;
        cert.expires_at_ns = end;
        assert_eq!(validate(&cert), Err(E::InvalidCertWindow));
    }
    let cert = cert();
    let mut bounds = limits();
    bounds.max_cert_ttl_ns = 0;
    assert_eq!(
        validate_certificate_issuance(&cert, p(1), bounds),
        Err(E::CertTtlExceeded {
            ttl_ns: 400,
            max_ttl_ns: 0
        })
    );
    bounds = limits();
    bounds.max_token_ttl_ns = 0;
    assert_eq!(
        validate_certificate_issuance(&cert, p(1), bounds),
        Err(E::TokenTtlExceeded {
            ttl_ns: 120,
            max_ttl_ns: 0
        })
    );
}

#[test]
fn combined_invalid_certificate_preserves_issuance_rejection_order() {
    use CertificateIssuanceError as E;
    let mut cert = cert();
    cert.root_pid = p(9);
    cert.issued_at_ns = 101;
    cert.expires_at_ns = 900;
    cert.max_token_ttl_ns = 0;
    cert.grants.clear();
    cert.issuer_proof_binding_hash = [0; 32];
    assert_eq!(
        validate(&cert),
        Err(E::RootPidMismatch {
            expected: p(1),
            found: p(9)
        })
    );
    cert.root_pid = p(1);
    assert_eq!(validate(&cert), Err(E::InvalidCertWindow));
    cert.issued_at_ns = 100;
    assert_eq!(
        validate(&cert),
        Err(E::CertTtlExceeded {
            ttl_ns: 800,
            max_ttl_ns: 600
        })
    );
    cert.expires_at_ns = 150;
    assert_eq!(validate(&cert), Err(E::TokenTtlZero));
    cert.max_token_ttl_ns = 121;
    assert_eq!(
        validate(&cert),
        Err(E::TokenTtlExceeded {
            ttl_ns: 121,
            max_ttl_ns: 120
        })
    );
    cert.max_token_ttl_ns = 120;
    assert_eq!(
        validate(&cert),
        Err(E::TokenTtlOutlivesCert {
            token_ttl_ns: 120,
            cert_ttl_ns: 50
        })
    );
    cert.expires_at_ns = 500;
    assert_eq!(validate(&cert), Err(E::GrantsEmpty));
    cert.grants = vec![grant("app_service", &["read"])];
    assert_eq!(validate(&cert), Err(E::IssuerProofBindingHashMismatch));
}

#[test]
fn canonical_grants_and_scope_errors_precede_binding_hash_refusal() {
    use CertificateIssuanceError as E;
    let cases = [
        (
            vec![grant("app_service", &[])],
            E::EmptyGrantScopes {
                role: "app_service".parse().unwrap(),
            },
        ),
        (
            vec![grant("app_service", &[""])],
            E::Canonical(CanonicalAuthError::Scope(
                ic_auth_protocol_types::ScopeLabelError::Empty,
            )),
        ),
        (
            vec![grant("app_service", &["Read"])],
            E::Canonical(CanonicalAuthError::Scope(
                ic_auth_protocol_types::ScopeLabelError::InvalidSegmentStart { offset: 0 },
            )),
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
            vec![grant("app_service", &["read"]), grant("app_service", &[])],
            E::Canonical(CanonicalAuthError::NonCanonicalRoles),
        ),
        (
            vec![
                grant("other_service", &["read"]),
                grant("app_service", &["Bad"]),
            ],
            E::Canonical(CanonicalAuthError::NonCanonicalRoles),
        ),
        // Earlier grant scope failure precedes a later role-order error.
        (
            vec![grant("other_service", &["Bad"]), grant("app_service", &[])],
            E::Canonical(CanonicalAuthError::Scope(
                ic_auth_protocol_types::ScopeLabelError::InvalidSegmentStart { offset: 0 },
            )),
        ),
    ];
    for (grants, expected) in cases {
        let mut cert = cert();
        cert.grants = grants;
        cert.issuer_proof_binding_hash = [0; 32];
        let original = cert.clone();
        assert_eq!(validate(&cert), Err(expected));
        assert_eq!(cert, original);
    }
}

#[test]
fn host_grant_and_scope_limits_accept_equality_and_refuse_one_over() {
    use CertificateIssuanceError as E;
    let mut cert = cert();
    cert.grants = (0..16)
        .map(|i| grant(&format!("role_{i:02}"), &["read"]))
        .collect();
    cert.grants[0].scopes = (0..32).map(|i| format!("scope_{i:02}")).collect();
    validate(&cert).unwrap();
    cert.grants.push(grant("role_16", &[]));
    assert_eq!(
        validate(&cert),
        Err(E::TooManyGrants { found: 17, max: 16 })
    );
    cert.grants.pop();
    cert.grants[0].scopes.push("Bad".into());
    assert_eq!(
        validate(&cert),
        Err(E::TooManyGrantScopes {
            role: "role_00".parse().unwrap(),
            found: 33,
            max: 32
        })
    );
    let mut bounds = limits();
    bounds.max_grants = 0;
    assert_eq!(
        validate_certificate_issuance(&cert, p(1), bounds),
        Err(E::TooManyGrants { found: 16, max: 0 })
    );
    bounds = limits();
    bounds.max_scopes_per_grant = 0;
    assert_eq!(
        validate_certificate_issuance(&cert, p(1), bounds),
        Err(E::TooManyGrantScopes {
            role: "role_00".parse().unwrap(),
            found: 33,
            max: 0
        })
    );
}

#[test]
fn passive_role_type_enforces_grammar_before_certificate_validation() {
    for invalid in ["", "Upper", "bad role", "é"] {
        assert!(invalid.parse::<AuthRole>().is_err());
    }
    let mut cert = cert();
    cert.grants = vec![grant("project-instance:v1_2", &["read"])];
    validate(&cert).unwrap();
}

#[test]
fn issuer_identity_and_seed_are_bound_by_the_canonical_owner() {
    for change_issuer in [false, true] {
        let mut cert = cert();
        if change_issuer {
            cert.issuer_pid = p(9);
        } else {
            cert.issuer_proof_binding =
                IssuerProofBinding::IcCanisterSignatureV1 { seed_hash: [9; 32] };
        }
        assert_eq!(
            validate(&cert),
            Err(CertificateIssuanceError::IssuerProofBindingHashMismatch)
        );
    }
}

#[test]
fn shape_success_does_not_claim_clock_authority_or_principal_eligibility() {
    let mut cert = cert();
    cert.root_pid = Principal::anonymous();
    cert.issuer_pid = Principal::anonymous();
    cert.issuer_proof_binding_hash = issuer_proof_binding_hash(
        cert.issuer_pid,
        cert.issuer_proof_alg,
        cert.issuer_proof_binding,
    )
    .unwrap();
    // No signature, policy or clock is supplied. Valid shape can describe an
    // expired, not-yet-valid or wholly unauthorized certificate.
    validate_certificate_issuance(&cert, Principal::anonymous(), limits()).unwrap();
}
