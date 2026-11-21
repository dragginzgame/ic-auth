//! Checked physical session/replay accounting; no reservation or publication.

use super::SessionPolicyError as E;
use ic_auth_protocol_types::Principal;

///
/// SessionCapacity
///
/// Protected physical count and encoded-byte ceilings, all strictly positive.
///

#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize, Eq, PartialEq)
)]
pub struct SessionCapacity {
    /// Physical session rows, including expired rows not explicitly pruned.
    pub max_sessions: usize,
    /// Physical replay rows across all accounts, including expired unpruned rows.
    pub max_replays: usize,
    /// Physical replay rows for an issuer-qualified account across all devices.
    pub max_replays_per_account: usize,
    /// Encoded session and replay bytes, excluding login rows and heap/index costs.
    pub max_encoded_bytes: usize,
}

impl SessionCapacity {
    pub(crate) fn validate(self) -> Result<(), E> {
        if self.max_sessions == 0
            || self.max_replays == 0
            || self.max_replays_per_account == 0
            || self.max_encoded_bytes == 0
        {
            return Err(E::InvalidContext);
        }
        Ok(())
    }
}

///
/// SessionUsage
///
/// Physical usage from one isolated snapshot, or projected totals after admission.
/// These numbers attest no provenance or commit. Backends must validate their
/// accounting and index associations; missing rows cannot be inferred from expiry.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionUsage {
    /// Issuer and subject owning the selected replay count, never a browser key.
    pub account: (Principal, Principal),
    /// Physical session rows across all accounts.
    pub sessions: usize,
    /// Physical replay rows across all accounts.
    pub replays: usize,
    /// Physical replay rows for `account`, including after logout/replacement.
    pub replays_for_account: usize,
    /// Encoded bytes across all session/replay rows, including expired rows.
    pub encoded_bytes: usize,
}

///
/// AdmissionFootprint
///
/// Exact bounded record sizes computed by the backend before publication.
/// This is not a caller-selected replacement hint. `replaced_session_bytes` must
/// come from the affected retained caller/slot-owner session in the same snapshot.
/// A known login fence without a session does not free a session row or bytes.
///

#[derive(Clone, Copy, Debug)]
pub(crate) struct AdmissionFootprint {
    /// Encoded size of the proposed session, 1 through 2048 bytes.
    pub session_bytes: usize,
    /// Encoded size of the new consumed replay, 1 through 512 bytes.
    pub replay_bytes: usize,
    /// Exact size of the one displaced/replaced session, when it exists.
    pub replaced_session_bytes: Option<usize>,
}

/// Check one fresh admission and calculate its physical session/replay totals.
///
/// `account` is the proposed issuer-qualified account, checked against the selected
/// lookup. Session replacement reuses one physical session row; it never removes
/// a replay or replenishes the account's replay quota. New totals subtract only
/// the replaced session's exact size before adding the new session and replay.
/// All arithmetic is checked. Invalid facts, count limits and byte limits remain
/// distinct refusals. Exact retries bypass fresh accounting entirely.
///
/// Success is arithmetic evidence only, not permission to commit these totals.
/// The receiver still owns proof/context/replay/login policy and one atomic host
/// transaction. Login pool, actual backing, index and heap limits remain separate.
/// No bytes are encoded, allocated, reserved, pruned or written by this function.
pub(crate) fn assess_capacity(
    account: (Principal, Principal),
    usage: SessionUsage,
    footprint: AdmissionFootprint,
    limits: SessionCapacity,
) -> Result<SessionUsage, E> {
    limits.validate()?;
    if account.0 == Principal::anonymous()
        || account.1 == Principal::anonymous()
        || account != usage.account
        || usage.replays_for_account > usage.replays
        || !(1..=2048).contains(&footprint.session_bytes)
        || !(1..=512).contains(&footprint.replay_bytes)
        || footprint
            .replaced_session_bytes
            .is_some_and(|bytes| !(1..=2048).contains(&bytes) || usage.sessions == 0)
    {
        return Err(E::InvalidFacts);
    }
    if (footprint.replaced_session_bytes.is_none() && usage.sessions >= limits.max_sessions)
        || usage.replays >= limits.max_replays
        || usage.replays_for_account >= limits.max_replays_per_account
    {
        return Err(E::Capacity);
    }
    // Limits above prove both replay additions and a new session addition fit
    // usize. Retain checked arithmetic rather than relying on that implication.
    let sessions = usage
        .sessions
        .checked_add(usize::from(footprint.replaced_session_bytes.is_none()))
        .ok_or(E::Capacity)?;
    let replays = usage.replays.checked_add(1).ok_or(E::Capacity)?;
    let replays_for_account = usage
        .replays_for_account
        .checked_add(1)
        .ok_or(E::Capacity)?;
    let remaining = usage
        .encoded_bytes
        .checked_sub(footprint.replaced_session_bytes.unwrap_or(0))
        .ok_or(E::InvalidFacts)?;
    let encoded_bytes = remaining
        .checked_add(footprint.session_bytes)
        .and_then(|bytes| bytes.checked_add(footprint.replay_bytes))
        .filter(|bytes| *bytes <= limits.max_encoded_bytes)
        .ok_or(E::ByteCapacity)?;
    Ok(SessionUsage {
        account,
        sessions,
        replays,
        replays_for_account,
        encoded_bytes,
    })
}
