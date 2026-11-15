//! Retained host-owned work allowance; no ambient clock or persistence.

use ic_auth_protocol_types::PortableMechanism;
use std::{error::Error, fmt};

/// Relative verification work costs, chosen from protected host measurements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerificationCosts {
    /// Units consumed by one attempted Ed25519 verification.
    pub ed25519: u32,
    /// Units consumed by one attempted certified verification.
    pub certified: u32,
}

#[cfg(test)]
mod tests {
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
}

/// Failure before expensive proof verification begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BudgetError {
    /// Capacity, refill interval or mechanism cost is zero or inconsistent.
    InvalidPolicy,
    /// Not enough retained work credit for this attempt.
    Exhausted,
    /// Trusted time precedes the most recently observed attempt.
    ClockRollback,
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPolicy => "invalid verification budget policy",
            Self::Exhausted => "verification budget exhausted",
            Self::ClockRollback => "verification clock moved backwards",
        })
    }
}
impl Error for BudgetError {}

/// One receiver-wide allowance shared across callers and proof mechanisms.
///
/// Starts empty and refills one unit per interval. Keep it across requests;
/// failed cryptography consumes credit. Hosts retain charges on normal refusal
/// and storage failure; an enclosing IC trap can roll back the whole message.
/// This bounds charged work, not ingress traffic or total instructions.
#[derive(Debug, Eq, PartialEq)]
pub struct VerificationBudget {
    capacity: u32,
    available: u32,
    interval_ns: u64,
    refill_at_ns: u64,
    observed_at_ns: u64,
    costs: VerificationCosts,
}

impl VerificationBudget {
    /// Configure positive costs within capacity and an initially empty bucket.
    pub fn new(
        capacity: u32,
        interval_ns: u64,
        costs: VerificationCosts,
        now_ns: u64,
    ) -> Result<Self, BudgetError> {
        if capacity == 0
            || interval_ns == 0
            || costs.ed25519 == 0
            || costs.certified == 0
            || costs.ed25519 > capacity
            || costs.certified > capacity
        {
            return Err(BudgetError::InvalidPolicy);
        }
        Ok(Self {
            capacity,
            available: 0,
            interval_ns,
            refill_at_ns: now_ns,
            observed_at_ns: now_ns,
            costs,
        })
    }

    /// Retained units after the last attempt; this does not observe time/refill.
    #[must_use]
    pub const fn available(&self) -> u32 {
        self.available
    }

    /// Most recent attempted-charge time, including exhausted attempts.
    #[must_use]
    pub const fn observed_at_ns(&self) -> u64 {
        self.observed_at_ns
    }

    pub(super) fn charge(
        &mut self,
        now_ns: u64,
        mechanism: PortableMechanism,
    ) -> Result<(), BudgetError> {
        if now_ns < self.observed_at_ns {
            return Err(BudgetError::ClockRollback);
        }
        let elapsed = now_ns - self.refill_at_ns;
        let added = elapsed / self.interval_ns;
        self.available = u64::from(self.available)
            .saturating_add(added)
            .min(u64::from(self.capacity)) as u32;
        self.refill_at_ns = now_ns - elapsed % self.interval_ns;
        self.observed_at_ns = now_ns;
        let cost = match mechanism {
            PortableMechanism::Ed25519 => self.costs.ed25519,
            PortableMechanism::Certified => self.costs.certified,
        };
        self.available = self
            .available
            .checked_sub(cost)
            .ok_or(BudgetError::Exhausted)?;
        Ok(())
    }
}
