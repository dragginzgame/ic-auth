use thiserror::Error;

/// Protected relative work units charged before cryptographic verification.
/// Choose from host measurements; these are not instruction or cycle ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "persistent", derive(serde::Serialize, serde::Deserialize))]
pub struct VerificationCosts {
    /// Work units per attempted Ed25519 verification.
    pub ed25519: u32,
    /// Work units per attempted certified-proof verification.
    pub certified: u32,
}

#[cfg(all(
    test,
    any(
        feature = "portable-signature-verification",
        feature = "certified-proof-verification"
    )
))]
#[path = "admission_budget/tests.rs"]
mod tests;

/// Refusal by the receiver-wide expensive-verification budget.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AdmissionBudgetError {
    #[error("invalid admission budget configuration")]
    /// The bucket policy is invalid or does not authorize proof verification.
    InvalidPolicy,
    #[error("fresh verification budget is exhausted")]
    /// Insufficient retained work units; verification must not start.
    Exhausted,
    #[error("admission clock moved backwards")]
    /// Trusted time precedes the bucket's last observation.
    ClockRollback,
}

/// Receiver-wide token bucket shared by verification and bounded-state admission.
/// Use separate instances for fresh proof verification and unknown-login logout;
/// each instance is shared across all callers of its operation class.
/// Keep one host-owned instance across callers, proof mechanisms and endpoints, outside
/// session transactions. Failed attempts consume credit too. Weighted verification
/// bounds charged work units, not total instructions, ingress traffic or fairness.
/// There is no implicit clock, allocation, persistence or per-principal map.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "persistent", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionAdmissionBudget {
    burst: u32,
    available: u32,
    refill_interval_ns: u64,
    refill_at_ns: u64,
    observed_at_ns: u64,
    costs: Option<VerificationCosts>,
}

impl SessionAdmissionBudget {
    #[cfg(feature = "persistent")]
    pub(crate) fn stored_policy_valid(&self, burst: u32, interval: u64, observed: u64) -> bool {
        self.burst == burst
            && self.refill_interval_ns == interval
            && burst > 0
            && interval > 0
            && self.available <= burst
            && self.refill_at_ns <= self.observed_at_ns
            && self.observed_at_ns <= observed
            && self.costs_valid()
    }

    fn costs_valid(&self) -> bool {
        self.costs.is_none_or(|costs| {
            costs.ed25519 > 0
                && costs.ed25519 <= self.burst
                && costs.certified > 0
                && costs.certified <= self.burst
        })
    }

    /// Configure one shared verification bucket with explicit mechanism costs.
    /// Burst and refill are work units; one unit refills per interval. Both
    /// costs must fit the burst. Operation buckets cannot verify proofs.
    pub fn for_verification(
        burst: u32,
        refill_interval_ns: u64,
        costs: VerificationCosts,
        now_ns: u64,
    ) -> Result<Self, AdmissionBudgetError> {
        let mut budget = Self::for_operations(burst, refill_interval_ns, now_ns)?;
        budget.costs = Some(costs);
        if !budget.costs_valid() {
            return Err(AdmissionBudgetError::InvalidPolicy);
        }
        Ok(budget)
    }

    #[cfg(feature = "persistent")]
    pub(crate) fn has_costs(&self, expected: VerificationCosts) -> bool {
        self.costs == Some(expected)
    }

    #[cfg(feature = "persistent")]
    pub(crate) fn is_operations(&self) -> bool {
        self.costs.is_none()
    }

    // A newly created issuer admits its configured burst immediately. Reopen
    // and restart retain the remaining allowance; only explicit creation fills it.
    #[cfg(feature = "persistent")]
    pub(crate) fn for_issuance(
        burst: u32,
        interval: u64,
        now: u64,
    ) -> Result<Self, AdmissionBudgetError> {
        let mut budget = Self::for_operations(burst, interval, now)?;
        budget.available = burst;
        Ok(budget)
    }

    #[cfg(any(
        feature = "portable-signature-verification",
        feature = "certified-proof-verification"
    ))]
    pub(crate) fn consume_verification(
        &mut self,
        now_ns: u64,
        mechanism: ic_auth_protocol_types::PortableMechanism,
    ) -> Result<(), AdmissionBudgetError> {
        let costs = self.costs.ok_or(AdmissionBudgetError::InvalidPolicy)?;
        let cost = match mechanism {
            ic_auth_protocol_types::PortableMechanism::Ed25519 => costs.ed25519,
            ic_auth_protocol_types::PortableMechanism::Certified => costs.certified,
        };
        self.consume_units(now_ns, cost)
    }
    /// Configure a unit-cost operation bucket, which refuses proof verification.
    /// Verification hosts must explicitly select `for_verification` costs.
    /// Starts empty: after creation/reset, wait one interval before admission.
    /// Hosts supply trusted time and must not recreate this for each request.
    pub fn for_operations(
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
            costs: None,
        })
    }

    pub(crate) fn consume(&mut self, now_ns: u64) -> Result<(), AdmissionBudgetError> {
        self.consume_units(now_ns, 1)
    }

    fn consume_units(&mut self, now_ns: u64, cost: u32) -> Result<(), AdmissionBudgetError> {
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
        if self.available < cost {
            return Err(AdmissionBudgetError::Exhausted);
        }
        self.available -= cost;
        Ok(())
    }
}
