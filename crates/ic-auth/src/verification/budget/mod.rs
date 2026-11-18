//! Retained host-owned work allowance; no ambient clock or persistence.

#[cfg(test)]
mod tests;

pub use crate::work_budget::BudgetError;
use crate::work_budget::WorkBucket;
use ic_auth_protocol_types::PortableMechanism;

/// Relative verification work costs, chosen from protected host measurements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerificationCosts {
    /// Units consumed by one attempted Ed25519 verification.
    pub ed25519: u32,
    /// Units consumed by one attempted certified verification.
    pub certified: u32,
}

/// One receiver-wide allowance shared across callers and proof mechanisms.
///
/// Starts empty and refills one unit per interval. Keep it across requests;
/// failed cryptography consumes credit. Hosts retain charges on normal refusal
/// and storage failure; an enclosing IC trap can roll back the whole message.
/// This bounds charged work, not ingress traffic or total instructions.
#[derive(Debug, Eq, PartialEq)]
pub struct VerificationBudget {
    bucket: WorkBucket,
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
        if costs.ed25519 == 0
            || costs.certified == 0
            || costs.ed25519 > capacity
            || costs.certified > capacity
        {
            return Err(BudgetError::InvalidPolicy);
        }
        Ok(Self {
            bucket: WorkBucket::new(capacity, interval_ns, now_ns)?,
            costs,
        })
    }

    /// Retained units after the last attempt; this does not observe time/refill.
    #[must_use]
    pub const fn available(&self) -> u32 {
        self.bucket.available()
    }

    /// Most recent attempted-charge time, including exhausted attempts.
    #[must_use]
    pub const fn observed_at_ns(&self) -> u64 {
        self.bucket.observed_at_ns()
    }

    pub(super) fn charge(
        &mut self,
        now_ns: u64,
        mechanism: PortableMechanism,
    ) -> Result<(), BudgetError> {
        let cost = match mechanism {
            PortableMechanism::Ed25519 => self.costs.ed25519,
            PortableMechanism::Certified => self.costs.certified,
        };
        self.bucket.charge(now_ns, cost)
    }
}
