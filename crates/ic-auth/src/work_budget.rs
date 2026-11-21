//! Shared retained refill arithmetic for separately owned operation budgets.
use std::{error::Error, fmt};

///
/// BudgetError
///
/// Failure before a charged operation begins.
///

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
            Self::InvalidPolicy => "invalid work budget policy",
            Self::Exhausted => "work budget exhausted",
            Self::ClockRollback => "work budget clock moved backwards",
        })
    }
}

impl Error for BudgetError {}

///
/// WorkBucket
///
/// Tracks available work credit and its refill interval.
///

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "receiver-stable", feature = "issuer-stable"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub(crate) struct WorkBucket {
    capacity: u32,
    available: u32,
    interval_ns: u64,
    refill_at_ns: u64,
    observed_at_ns: u64,
}

impl WorkBucket {
    #[cfg(any(feature = "issuer-stable", feature = "receiver-stable"))]
    pub(crate) fn matches(&self, capacity: u32, interval: u64, now: u64) -> bool {
        capacity > 0
            && interval > 0
            && self.capacity == capacity
            && self.interval_ns == interval
            && self.available <= capacity
            && self.refill_at_ns <= self.observed_at_ns
            && self.observed_at_ns <= now
    }

    pub(crate) fn new(capacity: u32, interval_ns: u64, now_ns: u64) -> Result<Self, BudgetError> {
        if capacity == 0 || interval_ns == 0 {
            return Err(BudgetError::InvalidPolicy);
        }
        Ok(Self {
            capacity,
            available: 0,
            interval_ns,
            refill_at_ns: now_ns,
            observed_at_ns: now_ns,
        })
    }

    pub(crate) const fn available(&self) -> u32 {
        self.available
    }

    #[cfg(feature = "issuer-memory")]
    pub(crate) fn new_full(
        capacity: u32,
        interval_ns: u64,
        now_ns: u64,
    ) -> Result<Self, BudgetError> {
        let mut bucket = Self::new(capacity, interval_ns, now_ns)?;
        bucket.available = capacity;
        Ok(bucket)
    }

    pub(crate) const fn observed_at_ns(&self) -> u64 {
        self.observed_at_ns
    }

    pub(crate) fn charge(&mut self, now_ns: u64, cost: u32) -> Result<(), BudgetError> {
        if cost == 0 || cost > self.capacity {
            return Err(BudgetError::InvalidPolicy);
        }
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
        self.available = self
            .available
            .checked_sub(cost)
            .ok_or(BudgetError::Exhausted)?;
        Ok(())
    }
}
