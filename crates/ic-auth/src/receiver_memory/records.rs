//! Receiver rows and byte accounting for the current checkpoint format.

use super::{Account, MemoryError, MemoryReceiver, SessionReceipt, Slot};
use crate::{
    receiver_policy::{
        DeviceFacts, LoginBinding, LoginFacts, ReceiverSnapshot, SessionAuthority, SessionFacts,
        SessionPolicyError,
    },
    session_request::SessionRequest,
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, PortableClaims, PortableMechanism, Principal, ScopeLabel,
};
use serde::Serialize;

///
/// Session
///
/// Retains the admitted mechanism, authority binding and lease for one session.
///

#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(super) struct Session {
    #[cfg_attr(
        feature = "receiver-stable",
        serde(with = "crate::stable_upgrade::extension")
    )]
    pub(super) ext: Option<Vec<u8>>,
    mechanism: PortableMechanism,
    key_binding_id: [u8; 32],
    role: AuthRole,
    scopes: Vec<ScopeLabel>,
    generation: u64,
    established_at_ns: u64,
    pub(super) expires_at_ns: u64,
    fingerprint: [u8; 32],
    request_hash: [u8; 32],
    #[serde(skip)]
    pub(super) bytes: usize,
}

impl Session {
    pub(super) fn prepare(
        claims: &PortableClaims,
        request: SessionRequest,
        authority: &SessionAuthority<'_>,
        expires_at_ns: u64,
        fingerprint: [u8; 32],
    ) -> Result<Self, MemoryError> {
        let request_hash = request.hash();
        let mut session = Self {
            ext: claims.ext.clone(),
            mechanism: claims.mechanism,
            key_binding_id: claims.key_binding_id,
            role: authority.role.clone(),
            scopes: request.into_scopes(),
            generation: authority.generation,
            established_at_ns: authority.now_ns,
            expires_at_ns,
            fingerprint,
            request_hash,
            bytes: 0,
        };
        session.bytes = session.encoded_size()?;
        Ok(session)
    }

    pub(super) fn encoded_size(&self) -> Result<usize, MemoryError> {
        measured(self, crate::receiver_policy::MAX_SESSION_BYTES)
    }
}

///
/// CallerRecord
///
/// Groups the caller’s login binding and optional live session.
///

#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(super) struct CallerRecord {
    pub(super) login: LoginFacts,
    pub(super) session: Option<Session>,
}

impl CallerRecord {
    pub(super) fn new(
        caller: Principal,
        audience: AudienceId,
        binding: LoginBinding,
        until_ns: u64,
        session: Session,
    ) -> Self {
        Self {
            login: LoginFacts {
                caller,
                audience,
                binding: Some(binding),
                until_ns,
                denied: false,
            },
            session: Some(session),
        }
    }

    pub(super) fn unknown(caller: Principal, audience: AudienceId, until_ns: u64) -> Self {
        Self {
            login: LoginFacts {
                caller,
                audience,
                binding: None,
                until_ns,
                denied: true,
            },
            session: None,
        }
    }

    fn binding(&self) -> LoginBinding {
        self.login
            .binding
            .expect("memory sessions always have a known binding")
    }

    // Older issuer generations started no later than this one. Their signed login
    // windows are bounded by the same immutable receiver lifetime limit.
    pub(super) fn reclaim_at(&self, max_login_ttl_ns: u64) -> u64 {
        self.binding()
            .started_at_ns
            .saturating_add(max_login_ttl_ns)
            .max(self.login.until_ns)
    }

    pub(super) fn slot(&self) -> Option<Slot> {
        self.login
            .binding
            .map(|b| (b.issuer, b.subject, b.device_slot))
    }

    pub(super) fn device(&self) -> Option<DeviceFacts> {
        let b = self.login.binding?;
        Some(DeviceFacts {
            issuer: b.issuer,
            subject: b.subject,
            slot: b.device_slot,
            generation: b.device_generation,
            caller: self.login.caller,
        })
    }

    pub(super) fn session_facts(&self) -> Option<SessionFacts<'_>> {
        self.session.as_ref().map(|s| SessionFacts {
            ext: s.ext.as_deref(),
            caller: self.login.caller,
            audience: self.login.audience,
            login: self.binding(),
            mechanism: s.mechanism,
            key_binding_id: s.key_binding_id,
            role: &s.role,
            scopes: &s.scopes,
            authority_generation: s.generation,
            established_at_ns: s.established_at_ns,
            expires_at_ns: s.expires_at_ns,
            fingerprint: s.fingerprint,
            request_hash: s.request_hash,
        })
    }
}

///
/// Replay
///
/// Retains the account and expiry of a consumed credential fingerprint.
///

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Replay {
    pub(super) account: Account,
    pub(super) remove_at_ns: u64,
    #[serde(skip)]
    pub(super) bytes: usize,
}

impl Replay {
    pub(super) fn prepare(
        fingerprint: &[u8; 32],
        account: Account,
        remove_at_ns: u64,
    ) -> Result<Self, MemoryError> {
        let mut replay = Self {
            account,
            remove_at_ns,
            bytes: 0,
        };
        replay.bytes = replay.encoded_size(fingerprint)?;
        Ok(replay)
    }

    pub(super) fn encoded_size(&self, fingerprint: &[u8; 32]) -> Result<usize, MemoryError> {
        measured(&(fingerprint, self), 512)
    }
}

///
/// Selected
///
/// Borrows the indexed facts used for a receiver policy decision.
///

pub(super) struct Selected<'a> {
    pub(super) session: Option<SessionFacts<'a>>,
    pub(super) login: Option<LoginFacts>,
    pub(super) device: Option<DeviceFacts>,
}

impl Selected<'_> {
    pub(super) fn snapshot<'a>(&'a self, receiver: &MemoryReceiver) -> ReceiverSnapshot<'a> {
        ReceiverSnapshot {
            audience: receiver.config.audience,
            authority_generation: receiver.generation,
            observed_at_ns: receiver.observed_at_ns,
            session: self.session.as_ref(),
            login: self.login.as_ref(),
            device: self.device.as_ref(),
        }
    }
}

impl SessionFacts<'_> {
    pub(super) fn receipt(&self) -> SessionReceipt {
        SessionReceipt {
            ext: self.ext.map(<[u8]>::to_vec),
            caller: self.caller,
            issuer: self.login.issuer,
            subject: self.login.subject,
            expires_at_ns: self.expires_at_ns,
        }
    }
}

fn measured(value: &impl Serialize, maximum: usize) -> Result<usize, MemoryError> {
    ic_wire::encoded_len(
        value,
        ic_wire::Encoding::Payload,
        ic_wire::Limits {
            bytes: maximum,
            allocation: 65_536,
            nodes: 1024,
            depth: 16,
        },
    )
    .map_err(|_| SessionPolicyError::ByteCapacity.into())
}
