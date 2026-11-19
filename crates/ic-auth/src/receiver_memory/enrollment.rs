//! Receiver-local admission of host-authenticated enrollment metadata.
use ic_auth_protocol_types::{AudienceId, Principal};
use std::{error::Error, fmt};

/// Installed host identities; never selected from browser input.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize, Eq, PartialEq)
)]
pub struct EnrollmentBinding {
    /// Actual receiver canister.
    pub receiver: Principal,
    /// Authority whose IC-authenticated response the host accepts.
    pub authority: Principal,
}

/// Metadata projected by the host from a complete authenticated snapshot.
///
/// The host owns response authentication, canonical snapshot/content digests,
/// entry validation, terminal key revocation and atomic installation of entries
/// alongside this receiver. This value alone authenticates none of those facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct EnrollmentMetadata {
    /// Receiver named by the authenticated response.
    pub receiver: Principal,
    /// Complete snapshot domain.
    pub audience: AudienceId,
    /// Echo of this receiver's outstanding host-generated challenge.
    pub challenge: [u8; 32],
    /// Monotonic authority snapshot revision, including freshness-only refreshes.
    pub revision: u64,
    /// Monotonic authority generation; changes invalidate retained sessions.
    pub generation: u64,
    /// Digest of the entire authenticated snapshot, including its metadata.
    pub snapshot_digest: [u8; 32],
    /// Digest of authority entries, revocations and policy, excluding freshness
    /// and response metadata. Changes require a strictly greater generation.
    pub authority_digest: [u8; 32],
    /// Authority issue time in nanoseconds.
    pub issued_at_ns: u64,
    /// Exclusive freshness deadline, at most one hour after issuance. Individual
    /// entries may have tighter approval deadlines in the host verifier context.
    pub expires_at_ns: u64,
}

/// Enrollment refusals never erase retained safety state or accepted metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnrollmentError {
    /// Invalid installed identities, challenge or receipt window.
    InvalidRequest,
    /// Host-authenticated response source differs from the installed authority.
    WrongAuthority,
    /// Response receiver/domain differs from the protected binding.
    WrongReceiver,
    /// No matching outstanding request (or exact accepted-response retry).
    ChallengeMismatch,
    /// Local receipt deadline has been reached.
    ReceiptExpired,
    /// Future issuance, expired/empty freshness or a window exceeding one hour.
    InvalidFreshness,
    /// Revision or generation would regress, including local invalidation.
    Regression,
    /// Same revision has different content, or authority changed without a new generation.
    ConflictingSnapshot,
}
impl fmt::Display for EnrollmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "enrollment refused: {self:?}")
    }
}
impl Error for EnrollmentError {}

#[derive(Clone, Copy)]
#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
struct Request {
    challenge: [u8; 32],
    receipt_deadline_ns: u64,
}

#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(super) struct Enrollment {
    binding: EnrollmentBinding,
    pending: Option<Request>,
    accepted: Option<(EnrollmentMetadata, Request)>,
}
impl Enrollment {
    pub(super) fn new(binding: EnrollmentBinding) -> Result<Self, EnrollmentError> {
        if [binding.receiver, binding.authority]
            .iter()
            .any(|p| *p == Principal::anonymous() || *p == Principal::management_canister())
        {
            return Err(EnrollmentError::InvalidRequest);
        }
        Ok(Self {
            binding,
            pending: None,
            accepted: None,
        })
    }

    pub(super) fn request(
        &mut self,
        now_ns: u64,
        challenge: [u8; 32],
        receipt_deadline_ns: u64,
    ) -> Result<(), EnrollmentError> {
        // The host supplies fresh unpredictable challenges across its whole
        // lifetime. These bounded checks additionally catch immediate reuse.
        if challenge == [0; 32]
            || receipt_deadline_ns <= now_ns
            || self.pending.is_some_and(|p| p.challenge == challenge)
            || self.accepted.is_some_and(|(s, _)| s.challenge == challenge)
        {
            return Err(EnrollmentError::InvalidRequest);
        }
        self.pending = Some(Request {
            challenge,
            receipt_deadline_ns,
        });
        Ok(())
    }

    pub(super) fn install(
        &mut self,
        audience: AudienceId,
        generation: u64,
        now_ns: u64,
        allow_accepted_retry: bool,
        authenticated_authority: Principal,
        snapshot: EnrollmentMetadata,
    ) -> Result<(), EnrollmentError> {
        use EnrollmentError as E;
        if authenticated_authority != self.binding.authority {
            return Err(E::WrongAuthority);
        }
        if snapshot.receiver != self.binding.receiver || snapshot.audience != audience {
            return Err(E::WrongReceiver);
        }
        let request = self
            .pending
            .or_else(|| {
                self.accepted
                    .filter(|(old, _)| allow_accepted_retry && *old == snapshot)
                    .map(|(_, request)| request)
            })
            .ok_or(E::ChallengeMismatch)?;
        if snapshot.challenge != request.challenge {
            return Err(E::ChallengeMismatch);
        }
        if now_ns >= request.receipt_deadline_ns {
            return Err(E::ReceiptExpired);
        }
        if snapshot.issued_at_ns > now_ns
            || now_ns >= snapshot.expires_at_ns
            || !snapshot
                .expires_at_ns
                .checked_sub(snapshot.issued_at_ns)
                .is_some_and(|duration| duration > 0 && duration <= 3_600_000_000_000)
        {
            return Err(E::InvalidFreshness);
        }
        if snapshot.generation < generation {
            return Err(E::Regression);
        }
        if let Some((old, _)) = self.accepted {
            if snapshot.revision < old.revision || snapshot.generation < old.generation {
                return Err(E::Regression);
            }
            if (snapshot.revision == old.revision && snapshot != old)
                || (snapshot.generation == old.generation
                    && snapshot.authority_digest != old.authority_digest)
            {
                return Err(E::ConflictingSnapshot);
            }
        }
        // All expected refusals precede these allocation-free publications.
        self.accepted = Some((snapshot, request));
        self.pending = None;
        Ok(())
    }

    #[cfg(feature = "receiver-stable")]
    pub(super) fn matches(
        &self,
        binding: EnrollmentBinding,
        audience: AudienceId,
        generation: u64,
    ) -> bool {
        self.binding == binding
            && self.accepted.is_none_or(|(snapshot, request)| {
                snapshot.receiver == binding.receiver
                    && snapshot.audience == audience
                    && snapshot.generation <= generation
                    && snapshot.challenge == request.challenge
            })
    }

    #[cfg(feature = "receiver-stable")]
    pub(super) fn discard_pending(&mut self) {
        self.pending = None;
    }

    pub(super) fn current(&self) -> Option<EnrollmentMetadata> {
        self.accepted.map(|(snapshot, _)| snapshot)
    }
}
