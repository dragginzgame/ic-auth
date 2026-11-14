//! Host-owned example: one volatile admission diagnostic window per receiver.
//! Observe the engine's final result once, before projecting the public reply.
//! No clock, identity labels, instruction samples or authentication decisions.
use ic_auth::{
    admission_budget::AdmissionBudgetError,
    session::{SessionError, SessionEstablished},
};
use ic_metrics::MeasurementCounter;

/// Diagnostic event counts, never verification credit or session authority.
/// Replace with `Default` to start a new window. Canic's metrics adapter owns
/// reporting and window identity. These counters are not checkpointed or DTOs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AdmissionMetrics {
    /// Engine results that committed a new session.
    pub created: MeasurementCounter,
    /// Engine results that replaced an existing session.
    pub replaced: MeasurementCounter,
    /// Exact retries returning the unchanged original receipt.
    pub exact_retry: MeasurementCounter,
    /// Fresh-proof refusals excluding budget exhaustion; cryptography may not have run.
    pub proof_refused: MeasurementCounter,
    /// Attempts refused because the shared verification budget was exhausted.
    pub budget_refused: MeasurementCounter,
    /// Other engine refusals, including invalid budget policy or clock rollback.
    pub other_refused: MeasurementCounter,
}

impl AdmissionMetrics {
    /// Count only calls that reached the engine. Transport/guard failures are
    /// outside this window. `Credential` refusals mean the fresh proof stage
    /// refused; they do not imply cryptography ran or verification credit was
    /// charged. Budget clock/config failures are other refusals, not exhaustion.
    /// Ordinary errors retain diagnostics and charges; an IC message trap rolls
    /// back the whole message, including both. Do not trap to report a refusal.
    pub fn observe(&mut self, result: &Result<SessionEstablished, SessionError>) {
        use SessionError as E;
        let counter = match result {
            Ok(SessionEstablished::Created(_)) => &mut self.created,
            Ok(SessionEstablished::Replaced(_)) => &mut self.replaced,
            Ok(SessionEstablished::ExactRetry(_)) => &mut self.exact_retry,
            Err(E::Credential(error)) => match error.budget_error() {
                Some(AdmissionBudgetError::Exhausted) => &mut self.budget_refused,
                Some(_) => &mut self.other_refused,
                None => &mut self.proof_refused,
            },
            Err(_) => &mut self.other_refused,
        };
        counter.add(1);
    }
}
