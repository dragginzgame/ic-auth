use thiserror::Error;

/// Refusal by the receiver-wide expensive-verification budget.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AdmissionBudgetError {
    #[error("invalid admission budget configuration")]
    InvalidPolicy,
    #[error("fresh verification budget is exhausted")]
    Exhausted,
    #[error("admission clock moved backwards")]
    ClockRollback,
}

/// Receiver-wide token bucket shared by verification and bounded-state admission.
/// Use separate instances for fresh proof verification and unknown-login logout;
/// each instance is shared across all callers of its operation class.
/// Keep one host-owned instance across callers, proof mechanisms and endpoints, outside
/// session transactions. Failed attempts consume credit too. This bounds
/// attempts, not total instructions, ingress traffic or fairness between users.
/// There is no implicit clock, allocation, persistence or per-principal map.
#[derive(Debug)]
pub struct SessionAdmissionBudget {
    burst: u32,
    available: u32,
    refill_interval_ns: u64,
    refill_at_ns: u64,
    observed_at_ns: u64,
}

impl SessionAdmissionBudget {
    /// Configure a burst ceiling and one new attempt per nonzero interval.
    /// Starts empty: after creation/reset, wait one interval before admission.
    /// Hosts supply trusted time and must not recreate this for each request.
    pub fn new(
        burst: u32,
        refill_interval_ns: u64,
        now_ns: u64,
    ) -> Result<Self, AdmissionBudgetError> {
        if burst == 0 || refill_interval_ns == 0 {
            return Err(AdmissionBudgetError::InvalidPolicy);
        }
        Ok(Self {
            burst,
            available: 0,
            refill_interval_ns,
            refill_at_ns: now_ns,
            observed_at_ns: now_ns,
        })
    }

    pub(crate) fn consume(&mut self, now_ns: u64) -> Result<(), AdmissionBudgetError> {
        if now_ns < self.observed_at_ns {
            return Err(AdmissionBudgetError::ClockRollback);
        }
        self.observed_at_ns = now_ns;
        let elapsed = now_ns - self.refill_at_ns;
        let refill = elapsed / self.refill_interval_ns;
        self.available = u64::from(self.available)
            .saturating_add(refill)
            .min(u64::from(self.burst)) as u32;
        self.refill_at_ns = now_ns - elapsed % self.refill_interval_ns;
        if self.available == 0 {
            return Err(AdmissionBudgetError::Exhausted);
        }
        self.available -= 1;
        Ok(())
    }
}
