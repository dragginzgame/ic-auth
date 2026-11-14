use super::*;
use ic_auth_protocol_types::PortableMechanism::{Certified, Ed25519};

#[test]
fn operation_buckets_cannot_silently_verify_at_unit_cost() {
    let mut bucket = SessionAdmissionBudget::for_operations(100, 1, 0).unwrap();
    let before = bucket.clone();
    for mechanism in [Ed25519, Certified] {
        assert_eq!(
            bucket.consume_verification(100, mechanism),
            Err(AdmissionBudgetError::InvalidPolicy)
        );
        assert_eq!(bucket, before);
    }
    bucket.consume(100).unwrap();
}

#[test]
fn weighted_verification_shares_units_and_never_partially_charges() {
    let costs = VerificationCosts {
        ed25519: 1,
        certified: 200,
    };
    let mut bucket = SessionAdmissionBudget::for_verification(201, 10, costs, 0).unwrap();
    assert_eq!(
        bucket.consume_verification(0, Ed25519),
        Err(AdmissionBudgetError::Exhausted)
    );
    assert_eq!(
        bucket.consume_verification(1990, Certified),
        Err(AdmissionBudgetError::Exhausted)
    );
    assert_eq!(bucket.available, 199);
    bucket.consume_verification(2000, Certified).unwrap();
    assert_eq!(bucket.available, 0);
    assert_eq!(
        bucket.consume_verification(2000, Ed25519),
        Err(AdmissionBudgetError::Exhausted)
    );
    bucket.consume_verification(2010, Ed25519).unwrap();
    assert_eq!(
        bucket.consume_verification(2009, Ed25519),
        Err(AdmissionBudgetError::ClockRollback)
    );
    bucket.consume_verification(u64::MAX, Certified).unwrap();
    bucket.consume_verification(u64::MAX, Ed25519).unwrap();
    assert_eq!(
        bucket.consume_verification(u64::MAX, Ed25519),
        Err(AdmissionBudgetError::Exhausted)
    );
}

#[test]
fn verification_costs_must_be_positive_and_fit_the_bucket() {
    for (ed25519, certified) in [(0, 1), (1, 0), (3, 1), (1, 3)] {
        assert_eq!(
            SessionAdmissionBudget::for_verification(
                2,
                1,
                VerificationCosts { ed25519, certified },
                0
            ),
            Err(AdmissionBudgetError::InvalidPolicy)
        );
    }
}
