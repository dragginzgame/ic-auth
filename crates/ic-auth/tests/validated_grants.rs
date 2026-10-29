//! Structural evidence is immutable and independent of authority/proof admission.
use ic_auth::{
    canonical::CanonicalAuthError, certificate_issuance::GrantValidationError as Error,
    portable::ValidatedGrants,
};
use ic_auth_protocol_types::DelegatedRoleGrant;

fn grants(roles: usize, scopes: usize) -> Vec<DelegatedRoleGrant> {
    (0..roles)
        .map(|role| DelegatedRoleGrant {
            target: format!("role{role:02}").parse().unwrap(),
            scopes: (0..scopes)
                .map(|scope| format!("scope{scope:02}"))
                .collect(),
        })
        .collect()
}

#[test]
fn exact_portable_cardinality_limits_preserve_original_grants() {
    for (roles, scopes) in [(1, 1), (16, 32)] {
        let original = grants(roles, scopes);
        let checked = ValidatedGrants::new(original.clone()).unwrap();
        assert_eq!(checked.as_slice(), original);
        assert_eq!(checked.clone(), checked);
        assert_eq!(checked.into_vec(), original);
    }
    assert_eq!(ValidatedGrants::new(vec![]), Err(Error::Empty));
    assert_eq!(
        ValidatedGrants::new(grants(17, 1)),
        Err(Error::TooMany { found: 17, max: 16 })
    );
    assert!(matches!(
        ValidatedGrants::new(grants(1, 0)),
        Err(Error::EmptyScopes { .. })
    ));
    assert!(matches!(
        ValidatedGrants::new(grants(1, 33)),
        Err(Error::TooManyScopes {
            found: 33,
            max: 32,
            ..
        })
    ));
}

#[test]
fn reject_noncanonical_or_invalid_labels_without_reordering_or_normalizing() {
    let original = grants(2, 2);
    let mut bad = original.clone();
    bad.reverse();
    assert_eq!(
        ValidatedGrants::new(bad),
        Err(Error::Canonical(CanonicalAuthError::NonCanonicalRoles))
    );
    let mut bad = original.clone();
    bad[1].target = bad[0].target.clone();
    assert_eq!(
        ValidatedGrants::new(bad),
        Err(Error::Canonical(CanonicalAuthError::NonCanonicalRoles))
    );
    for scopes in [
        vec!["write".into(), "read".into()],
        vec!["read".into(), "read".into()],
    ] {
        let mut bad = original.clone();
        bad[0].scopes = scopes;
        assert_eq!(
            ValidatedGrants::new(bad),
            Err(Error::Canonical(CanonicalAuthError::NonCanonicalScopes))
        );
    }
    for label in [
        "".to_owned(),
        "READ".into(),
        "read..write".into(),
        "x".repeat(65),
    ] {
        let mut bad = original.clone();
        bad[0].scopes = vec![label];
        assert!(matches!(
            ValidatedGrants::new(bad),
            Err(Error::Canonical(_))
        ));
    }
    assert_eq!(
        ValidatedGrants::new(original.clone()).unwrap().as_slice(),
        original
    );
}
