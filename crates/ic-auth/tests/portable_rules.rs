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
        device_slot: 0,
        device_generation: 1,
        account_subject: Principal::from_slice(&[2]),
        login_started_at_ns: 10,
        login_deadline_ns: 110,
        issued_at_ns: 20,
        admission_expires_at_ns: 80,
        nonce: [5; 16],
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
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

// Clock-independent admission bounds and lost-state counterexample (#127).
#[test]
fn clock_contract_signed_deadline_bounds_every_renewal_without_an_offset_estimate() {
    for mechanism in [PortableMechanism::Certified, PortableMechanism::Ed25519] {
        // Translate the issuer's whole window, rather than assuming its epoch is
        // near the receiver's. Only receiver-local comparisons decide validity.
        for start in [0, 1, 1_000, 86_400_000_000_000, u64::MAX - 200] {
            let mut value = claims();
            value.mechanism = mechanism;
            value.login_started_at_ns = start;
            value.login_deadline_ns = start + 100;
            let grants = ValidatedGrants::new(value.grants.clone()).unwrap();
            let trusted = value.clone();
            let mut host = context(&trusted, &grants);
            host.authority.authority_expires_at_ns = u64::MAX;
            host.authority.enrollment_expires_at_ns = u64::MAX;
            host.authority.min_login_started_at_ns = 0;
            host.min_proof_issued_at_ns = 0;
            // Enumerate every positive proof window within this small login.
            for issued in start..value.login_deadline_ns {
                value.issued_at_ns = issued;
                for expiry in issued + 1..=(issued + 60).min(value.login_deadline_ns) {
                    value.admission_expires_at_ns = expiry;
                    host.authority.now_ns = issued;
                    assert_eq!(validate_claims_context(&value, &host), Ok(()));
                    host.authority.now_ns = value.login_deadline_ns;
                    assert_eq!(validate_claims_context(&value, &host), Err(Error::Expired));
                }
            }
        }
    }
}

#[test]
fn local_timestamp_floor_cannot_fence_a_lost_login() {
    for mechanism in [PortableMechanism::Certified, PortableMechanism::Ed25519] {
        let mut old = claims();
        old.mechanism = mechanism;
        // An old login can already exist at issuer time 40 when receiver boots
        // at 30. A local time floor does not prove this login is new.
        old.login_started_at_ns = 40;
        old.issued_at_ns = 40;
        let grants = ValidatedGrants::new(old.grants.clone()).unwrap();
        let mut host = context(&old, &grants);
        host.authority.min_login_started_at_ns = 31;
        host.min_proof_issued_at_ns = 31;
        assert_eq!(
            validate_claims_context(&old, &host),
            Err(Error::NotYetValid)
        );
        host.authority.now_ns = 40;
        assert_eq!(validate_claims_context(&old, &host), Ok(()));
        // A fresh renewal of that same pre-reset login also passes this floor.
        let mut renewed = old.clone();
        renewed.issued_at_ns = 41;
        host.authority.now_ns = 41;
        assert_eq!(validate_claims_context(&renewed, &host), Ok(()));
    }
}

#[test]
fn ninety_day_login_ceiling_keeps_short_proofs_and_stricter_host_policy() {
    const LOGIN: u64 = 90 * 86_400_000_000_000;
    const PROOF: u64 = 60_000_000_000;
    for mechanism in [PortableMechanism::Certified, PortableMechanism::Ed25519] {
        let mut value = claims();
        value.mechanism = mechanism;
        value.login_deadline_ns = value.login_started_at_ns + LOGIN;
        value.admission_expires_at_ns = value.issued_at_ns + PROOF;
        let trusted = value.clone();
        let grants = ValidatedGrants::new(trusted.grants.clone()).unwrap();
        let mut host = context(&trusted, &grants);
        host.max_login_ttl_ns = LOGIN;
        host.max_proof_ttl_ns = PROOF;
        assert_eq!(validate_claims_context(&value, &host), Ok(()));
        host.max_login_ttl_ns = LOGIN - 1;
        assert_eq!(
            validate_claims_context(&value, &host),
            Err(Error::InvalidWindow)
        );
        host.max_login_ttl_ns = LOGIN;
        value.login_deadline_ns += 1;
        assert_eq!(
            validate_claims_context(&value, &host),
            Err(Error::InvalidWindow)
        );
        value.login_deadline_ns -= 1;
        value.admission_expires_at_ns += 1;
        assert_eq!(
            validate_claims_context(&value, &host),
            Err(Error::InvalidWindow)
        );
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
    escalated.grants[0].scopes.push("write".parse().unwrap());
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
    for max in [0, 7_776_000_000_000_001, u64::MAX] {
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
