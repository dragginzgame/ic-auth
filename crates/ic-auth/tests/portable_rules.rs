use ic_auth::portable::{
    PortableClaimsContext, PortableClaimsError as Error, ValidatedGrants, browser_principal,
    validate_claims_context,
};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};

fn claims() -> PortableClaims {
    PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: Principal::from_slice(&[1]),
        key_binding_id: [3; 32],
        browser_public_key: [4; 32],
        account_subject: Principal::from_slice(&[2]),
        login_started_at_ns: 10,
        login_deadline_ns: 110,
        issued_at_ns: 20,
        admission_expires_at_ns: 80,
        nonce: [5; 16],
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".into()],
        }],
    }
}

fn context<'a>(
    value: &'a PortableClaims,
    grants: &'a ValidatedGrants,
) -> PortableClaimsContext<'a> {
    PortableClaimsContext {
        authority: ic_auth::portable::PortableAuthority {
            audience: &value.audience,
            issuer: value.issuer,
            mechanism: value.mechanism,
            key_binding_id: value.key_binding_id,
            grants,
            caller: browser_principal(&value.browser_public_key),
            now_ns: 30,
            authority_not_before_ns: 0,
            authority_expires_at_ns: 200,
            enrollment_expires_at_ns: 150,
            min_login_started_at_ns: 10,
        },
        min_proof_issued_at_ns: 20,
        max_login_ttl_ns: 100,
        max_proof_ttl_ns: 60,
    }
}

#[test]
fn both_mechanisms_use_identical_strict_time_rules() {
    for mechanism in [PortableMechanism::Certified, PortableMechanism::Ed25519] {
        let mut value = claims();
        value.mechanism = mechanism;
        let checked_grants = ValidatedGrants::new(value.grants.clone()).unwrap();
        let mut host = context(&value, &checked_grants);
        for (now, expected) in [
            (19, Err(Error::NotYetValid)),
            (20, Ok(())),
            (79, Ok(())),
            (80, Err(Error::Expired)),
            (110, Err(Error::Expired)),
        ] {
            host.authority.now_ns = now;
            assert_eq!(validate_claims_context(&value, &host), expected);
        }
    }
}

#[test]
fn reset_floor_rejects_fresh_renewal_of_an_old_login() {
    let original = claims();
    let checked_grants = ValidatedGrants::new(original.grants.clone()).unwrap();
    let mut host = context(&original, &checked_grants);
    assert_eq!(validate_claims_context(&original, &host), Ok(()));
    host.authority.min_login_started_at_ns = 11;
    let mut renewal = original.clone();
    renewal.issued_at_ns = 30;
    renewal.admission_expires_at_ns = 90;
    assert_eq!(
        validate_claims_context(&renewal, &host),
        Err(Error::BeforeReset)
    );
    host.authority.min_login_started_at_ns = 10;
    host.min_proof_issued_at_ns = 21;
    assert_eq!(
        validate_claims_context(&original, &host),
        Err(Error::BeforeReset)
    );
    assert_eq!(validate_claims_context(&renewal, &host), Ok(()));
}

#[test]
fn live_authority_and_grant_ceiling_are_independent_of_claims() {
    let value = claims();
    for expiry in [29, 30] {
        let checked_grants = ValidatedGrants::new(value.grants.clone()).unwrap();
        let mut host = context(&value, &checked_grants);
        host.authority.authority_expires_at_ns = expiry;
        assert_eq!(
            validate_claims_context(&value, &host),
            Err(Error::StaleAuthority)
        );
        let checked_grants = ValidatedGrants::new(value.grants.clone()).unwrap();
        host = context(&value, &checked_grants);
        host.authority.enrollment_expires_at_ns = expiry;
        assert_eq!(
            validate_claims_context(&value, &host),
            Err(Error::StaleAuthority)
        );
    }
    let checked_grants = ValidatedGrants::new(value.grants.clone()).unwrap();
    let mut host = context(&value, &checked_grants);
    host.authority.authority_not_before_ns = 21;
    assert_eq!(
        validate_claims_context(&value, &host),
        Err(Error::StaleAuthority)
    );
    let mut escalated = value.clone();
    escalated.grants[0].scopes.push("write".into());
    assert_eq!(
        validate_claims_context(
            &escalated,
            &context(&value, &ValidatedGrants::new(value.grants.clone()).unwrap())
        ),
        Err(Error::GrantsExceeded)
    );
    escalated.grants[0].target = "admin".parse().unwrap();
    assert_eq!(
        validate_claims_context(
            &escalated,
            &context(&value, &ValidatedGrants::new(value.grants.clone()).unwrap())
        ),
        Err(Error::GrantsExceeded)
    );
    let malformed = vec![value.grants[0].clone(), value.grants[0].clone()];
    assert!(ValidatedGrants::new(malformed).is_err());
}

#[test]
fn caller_is_the_browser_principal_and_account_is_issuer_scoped() {
    let value = claims();
    for caller in [
        Principal::anonymous(),
        value.account_subject,
        browser_principal(&[9; 32]),
    ] {
        let checked_grants = ValidatedGrants::new(value.grants.clone()).unwrap();
        let mut host = context(&value, &checked_grants);
        host.authority.caller = caller;
        assert_eq!(
            validate_claims_context(&value, &host),
            Err(Error::CallerMismatch)
        );
    }
    // Two devices can share an issuer/account while having independent callers.
    let mut second_device = value.clone();
    second_device.browser_public_key = [9; 32];
    assert_eq!(
        validate_claims_context(
            &second_device,
            &context(
                &second_device,
                &ValidatedGrants::new(second_device.grants.clone()).unwrap()
            )
        ),
        Ok(())
    );
    assert_eq!(
        validate_claims_context(
            &second_device,
            &context(&value, &ValidatedGrants::new(value.grants.clone()).unwrap())
        ),
        Err(Error::CallerMismatch)
    );
    // An equal account subject does not allow another issuer to use this authority.
    let mut second_issuer = value.clone();
    second_issuer.issuer = Principal::from_slice(&[3]);
    assert_eq!(
        validate_claims_context(
            &second_issuer,
            &context(&value, &ValidatedGrants::new(value.grants.clone()).unwrap())
        ),
        Err(Error::IssuerMismatch)
    );
}

#[test]
fn impossible_or_overlong_windows_and_policy_are_refused_without_wrapping() {
    let trusted = claims();
    let mutations: [fn(&mut PortableClaims); 7] = [
        |c| c.login_deadline_ns = c.login_started_at_ns,
        |c| c.login_deadline_ns = 111,
        |c| c.admission_expires_at_ns = c.issued_at_ns,
        |c| c.admission_expires_at_ns = 81,
        |c| c.login_started_at_ns = 21,
        |c| c.login_deadline_ns = 79,
        |c| c.issued_at_ns = u64::MAX,
    ];
    for change in mutations {
        let mut value = trusted.clone();
        change(&mut value);
        assert_eq!(
            validate_claims_context(
                &value,
                &context(
                    &trusted,
                    &ValidatedGrants::new(trusted.grants.clone()).unwrap()
                )
            ),
            Err(Error::InvalidWindow)
        );
    }
    for max in [0, 86_400_000_000_001, u64::MAX] {
        let checked_grants = ValidatedGrants::new(trusted.grants.clone()).unwrap();
        let mut host = context(&trusted, &checked_grants);
        host.max_login_ttl_ns = max;
        assert_eq!(
            validate_claims_context(&trusted, &host),
            Err(Error::InvalidContext)
        );
    }
    for max in [0, 60_000_000_001, u64::MAX] {
        let checked_grants = ValidatedGrants::new(trusted.grants.clone()).unwrap();
        let mut host = context(&trusted, &checked_grants);
        host.max_proof_ttl_ns = max;
        assert_eq!(
            validate_claims_context(&trusted, &host),
            Err(Error::InvalidContext)
        );
    }
    let mut boundary = trusted.clone();
    boundary.login_started_at_ns = u64::MAX - 100;
    boundary.login_deadline_ns = u64::MAX;
    boundary.issued_at_ns = u64::MAX - 60;
    boundary.admission_expires_at_ns = u64::MAX;
    let checked_grants = ValidatedGrants::new(boundary.grants.clone()).unwrap();
    let mut host = context(&boundary, &checked_grants);
    host.authority.now_ns = u64::MAX - 1;
    host.authority.authority_expires_at_ns = u64::MAX;
    host.authority.enrollment_expires_at_ns = u64::MAX;
    assert_eq!(validate_claims_context(&boundary, &host), Ok(()));
}

#[test]
fn reset_cutoffs_cover_issuer_ahead_clocks_and_reject_renewed_old_logins() {
    for mechanism in [PortableMechanism::Certified, PortableMechanism::Ed25519] {
        let mut original = claims();
        original.mechanism = mechanism;
        // At receiver boot time 30 the issuer could already have issued at 35.
        original.login_started_at_ns = 35;
        original.issued_at_ns = 35;
        let checked_grants = ValidatedGrants::new(original.grants.clone()).unwrap();
        let mut host = context(&original, &checked_grants);
        host.advance_reset_floors(5).unwrap();
        assert_eq!(
            (
                host.authority.min_login_started_at_ns,
                host.min_proof_issued_at_ns
            ),
            (36, 31)
        );
        host.authority.now_ns = 40;
        assert_eq!(
            validate_claims_context(&original, &host),
            Err(Error::BeforeReset)
        );
        let mut renewed = original.clone();
        renewed.issued_at_ns = 40;
        renewed.admission_expires_at_ns = 100;
        assert_eq!(
            validate_claims_context(&renewed, &host),
            Err(Error::BeforeReset)
        );
        // A fresh login beyond the margin is eligible, with a distinct browser key.
        let mut fresh = renewed;
        fresh.browser_public_key = [9; 32];
        fresh.login_started_at_ns = 36;
        host.authority.caller = browser_principal(&fresh.browser_public_key);
        assert_eq!(validate_claims_context(&fresh, &host), Ok(()));
        host.authority.now_ns = 39;
        assert_eq!(
            validate_claims_context(&fresh, &host),
            Err(Error::NotYetValid)
        );
        host.authority.now_ns = 100;
        assert_eq!(validate_claims_context(&fresh, &host), Err(Error::Expired));
    }
}

#[test]
fn reset_cutoffs_never_decrease_or_partially_change_on_overflow() {
    let c = claims();
    let checked_grants = ValidatedGrants::new(c.grants.clone()).unwrap();
    let mut host = context(&c, &checked_grants);
    host.advance_reset_floors(5).unwrap();
    host.advance_reset_floors(0).unwrap();
    assert_eq!(
        (
            host.authority.min_login_started_at_ns,
            host.min_proof_issued_at_ns
        ),
        (36, 31)
    );
    host.authority.now_ns = 50;
    host.advance_reset_floors(0).unwrap();
    assert_eq!(
        (
            host.authority.min_login_started_at_ns,
            host.min_proof_issued_at_ns
        ),
        (51, 51)
    );
    for (now, skew) in [(u64::MAX, 0), (u64::MAX - 1, 1), (50, u64::MAX)] {
        host.authority.now_ns = now;
        assert_eq!(host.advance_reset_floors(skew), Err(Error::InvalidContext));
        assert_eq!(
            (
                host.authority.min_login_started_at_ns,
                host.min_proof_issued_at_ns
            ),
            (51, 51)
        );
    }
    host.authority.now_ns = u64::MAX - 1;
    host.advance_reset_floors(0).unwrap();
    assert_eq!(
        (
            host.authority.min_login_started_at_ns,
            host.min_proof_issued_at_ns
        ),
        (u64::MAX, u64::MAX)
    );
    let checked_grants = ValidatedGrants::new(c.grants.clone()).unwrap();
    let mut origin = context(&c, &checked_grants);
    origin.authority.now_ns = 0;
    origin.authority.min_login_started_at_ns = 0;
    origin.min_proof_issued_at_ns = 0;
    origin.advance_reset_floors(0).unwrap();
    assert_eq!(
        (
            origin.authority.min_login_started_at_ns,
            origin.min_proof_issued_at_ns
        ),
        (1, 1)
    );
}
