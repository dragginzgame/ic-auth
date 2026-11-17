use super::*;

#[test]
fn weighted_mechanisms_share_credit_and_refill_remainder() {
    use PortableMechanism::{Certified, Ed25519};
    let mut budget = VerificationBudget::new(
        10,
        3,
        VerificationCosts {
            ed25519: 2,
            certified: 3,
        },
        0,
    )
    .unwrap();
    for (now, mechanism, expected, remaining) in [
        (0, Ed25519, Err(BudgetError::Exhausted), 0),
        (12, Certified, Ok(()), 1),
        (14, Ed25519, Err(BudgetError::Exhausted), 1),
        (15, Ed25519, Ok(()), 0),
        (33, Certified, Ok(()), 3),
        (34, Ed25519, Ok(()), 1),
        (u64::MAX, Ed25519, Ok(()), 8),
        (u64::MAX - 1, Certified, Err(BudgetError::ClockRollback), 8),
    ] {
        assert_eq!(budget.charge(now, mechanism), expected);
        assert_eq!(budget.available(), remaining);
    }
    assert_eq!(budget.observed_at_ns(), u64::MAX);
}
