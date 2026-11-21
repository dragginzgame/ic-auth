//! One synchronous publication boundary; certification storage remains host-owned.
use super::{
    Completion, IssuedCredential, IssuerAuthority, IssuerError, IssuerSigning, IssuerTicket,
    MemoryIssuer, Phase,
};
use crate::certified_message::certified_signature_message;
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

///
/// CertifiedPublication
///
/// Exact approved leaf input, borrowed only during synchronous host publication.
/// Publish an empty leaf at `/sig/seed_hash/message_hash`, compose the host root
/// and retain the completion in one transaction. This is not a query witness.
///

#[derive(Debug)]
pub struct CertifiedPublication<'a> {
    /// Original intent identity for the host's transaction.
    pub ticket: IssuerTicket,
    /// Original canonical claims. Do not regenerate them when assembling witnesses.
    pub claims: &'a [u8],
    /// SHA-256 of the seed admitted by the captured protected authority.
    pub seed_hash: [u8; 32],
    /// SHA-256 of the fixed IC domain framing and original claims digest.
    pub message_hash: [u8; 32],
    /// Original exclusive admission deadline. Leaf retention cannot renew it.
    pub admission_deadline_ns: u64,
}

///
/// CertifiedCompletionError
///
/// Auth refusals precede publication. A host failure can follow writes and closes
/// this issuer; the host must roll back the entire transaction, including its root.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertifiedCompletionError<E> {
    /// Refused before invoking the host; ordinary Auth error semantics apply.
    Issuer(IssuerError),
    /// Host publication failed; the issuer is terminally closed, not rolled back.
    Publication(E),
}

impl<E: fmt::Display> fmt::Display for CertifiedCompletionError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Issuer(e) => e.fmt(f),
            Self::Publication(e) => write!(f, "certified publication failed: {e}"),
        }
    }
}

impl<E: Error + 'static> Error for CertifiedCompletionError<E> {}
impl<E> From<IssuerError> for CertifiedCompletionError<E> {
    fn from(value: IssuerError) -> Self {
        Self::Issuer(value)
    }
}

impl MemoryIssuer {
    /// Publish the approved certified leaf and retain completion synchronously.
    /// No external dispatch is needed. The host callback must prepare the exact
    /// leaf and publish its composed certification root in the SAME transaction
    /// as this call; success attests both. Preflight host capacity before calling.
    /// This API neither authenticates delegation enrollment nor implements rollback.
    ///
    /// A returned host error or unwind leaves this instance closed because the
    /// host may already have written. Never serve a partial root or catch a failure
    /// as successful IC commit. After native failure discard the enclosing host
    /// state; an IC host must trap to roll back. Exact completed retries skip the
    /// callback, even after expiry/revocation/restart or signature-leaf pruning.
    ///
    /// # Panics
    /// Host callback panic or interrupted publication leaves this instance closed.
    /// The enclosing host transaction must roll back or be discarded.
    pub fn complete_certified<E>(
        &mut self,
        ticket: IssuerTicket,
        authority: IssuerAuthority,
        now_ns: u64,
        publish: impl FnOnce(CertifiedPublication<'_>) -> Result<(), E>,
    ) -> Result<IssuedCredential<'_>, CertifiedCompletionError<E>> {
        self.observe(now_ns)?;
        let op = self.operation(ticket)?;
        let IssuerSigning::Certified { seed_hash } = ticket.authority.signing else {
            return Err(IssuerError::Invalid.into());
        };
        if !matches!(op.phase, Phase::Completed(..)) {
            self.check_pending(op, authority, now_ns)?;
            let Phase::Approved(a) = &op.phase else {
                return Err(IssuerError::Pending.into());
            };
            let message_hash = Sha256::digest(certified_signature_message(
                Sha256::digest(&a.claims).into(),
            ))
            .into();
            // All ordinary checks are complete. Keep closed across the callback
            // and both local writes; there is no fallible mutation tail.
            self.open = false;
            let op =
                &self.accounts[&ticket.account].operations[&(ticket.epoch, ticket.operation_id)];
            let Phase::Approved(a) = &op.phase else {
                unreachable!("approved publication")
            };
            publish(CertifiedPublication {
                ticket,
                claims: &a.claims,
                seed_hash,
                message_hash,
                admission_deadline_ns: op.admission,
            })
            .map_err(CertifiedCompletionError::Publication)?;
            super::after_write!();
            self.finish(ticket, Completion::Certified);
        }
        self.completed(ticket).map_err(Into::into)
    }
}
