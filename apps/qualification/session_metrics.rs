//! Host-owned example: one volatile admission diagnostic window per receiver.
//! Observe the engine's final result once, before projecting the public reply.
//! No clock, identity labels, instruction samples or authentication decisions.
use ic_auth::{
    admission_budget::AdmissionBudgetError,
    portable_certified::PortableCertifiedError,
    portable_credential::PortableCredentialError,
    portable_signature::PortableSignatureError,
    session::{SessionError, SessionEstablished},
};
use ic_metrics::MeasurementCounter;

/// Diagnostic event counts, never verification credit or session authority.
/// Copy for a report; replace with `Default` to start a new window. Hosts own
/// report authorization/window identity. These counters are not checkpointed.
/// Serde delegates numeric admission to Metrics, preserving u128 and actual loss.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionMetrics {
    pub created: MeasurementCounter,
    pub replaced: MeasurementCounter,
    pub exact_retry: MeasurementCounter,
    pub proof_refused: MeasurementCounter,
    pub budget_refused: MeasurementCounter,
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
        use PortableCredentialError as C;
        use SessionError as E;
        let counter = match result {
            Ok(SessionEstablished::Created(_)) => &mut self.created,
            Ok(SessionEstablished::Replaced(_)) => &mut self.replaced,
            Ok(SessionEstablished::ExactRetry(_)) => &mut self.exact_retry,
            Err(E::AdmissionThrottled)
            | Err(E::Credential(C::Ed25519(PortableSignatureError::Budget(
                AdmissionBudgetError::Exhausted,
            ))))
            | Err(E::Credential(C::Certified(PortableCertifiedError::Budget(
                AdmissionBudgetError::Exhausted,
            )))) => &mut self.budget_refused,
            Err(E::Credential(C::Ed25519(PortableSignatureError::Budget(_))))
            | Err(E::Credential(C::Certified(PortableCertifiedError::Budget(_)))) => {
                &mut self.other_refused
            }
            Err(E::Credential(_)) => &mut self.proof_refused,
            Err(_) => &mut self.other_refused,
        };
        counter.add(1);
    }
}
