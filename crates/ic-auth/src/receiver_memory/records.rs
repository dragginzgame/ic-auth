//! Backend-local records and compatibility byte accounting. No persisted codec.
//!
//! Field order/representations follow the pinned reference session/model.rs and
//! storage_codec/mod.rs. Wire measures the logical frame; no bytes are emitted.

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
use serde::{Serialize, Serializer, ser::SerializeSeq};

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
    mechanism: PortableMechanism,
    key_binding_id: [u8; 32],
    role: AuthRole,
    scopes: Vec<ScopeLabel>,
    generation: u64,
    established_at_ns: u64,
    pub(super) expires_at_ns: u64,
    fingerprint: [u8; 32],
    request_hash: [u8; 32],
    pub(super) bytes: usize,
}

impl Session {
    pub(super) fn prepare(
        claims: &PortableClaims,
        request: &SessionRequest,
        authority: &SessionAuthority<'_>,
        expires_at_ns: u64,
        fingerprint: [u8; 32],
    ) -> Result<Self, MemoryError> {
        let request_hash = request.hash();
        let bytes = measured(
            &SessionSize {
                caller: authority.caller,
                subject: claims.account_subject,
                issuer: claims.issuer,
                audience: Audience(claims.audience),
                mechanism: Mechanism(claims.mechanism),
                key_binding_id: &claims.key_binding_id,
                role: authority.role.as_str(),
                scopes: Scopes(request.scopes()),
                generation: authority.generation,
                device_slot: claims.device_slot,
                device_generation: claims.device_generation,
                login_started_at_ns: claims.login_started_at_ns,
                login_deadline_ns: claims.login_deadline_ns,
                established_at_ns: authority.now_ns,
                expires_at_ns,
                fingerprint: &fingerprint,
                request_hash: &request_hash,
            },
            2048,
        )?;
        Ok(Self {
            mechanism: claims.mechanism,
            key_binding_id: claims.key_binding_id,
            role: authority.role.clone(),
            scopes: Vec::new(),
            generation: authority.generation,
            established_at_ns: authority.now_ns,
            expires_at_ns,
            fingerprint,
            request_hash,
            bytes,
        })
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
        mut session: Session,
        scopes: Vec<ScopeLabel>,
    ) -> Self {
        session.scopes = scopes;
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
/// Retains the credential binding needed to decide an exact admission retry.
///

#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(super) struct Replay {
    pub(super) account: Account,
    caller: Principal,
    generation: u64,
    pub(super) remove_at_ns: u64,
    pub(super) bytes: usize,
}

impl Replay {
    pub(super) fn prepare(
        fingerprint: [u8; 32],
        caller: Principal,
        binding: LoginBinding,
        generation: u64,
        remove_at_ns: u64,
    ) -> Result<Self, MemoryError> {
        let mut replay = Self {
            account: (binding.issuer, binding.subject),
            caller,
            generation,
            remove_at_ns,
            bytes: 0,
        };
        replay.bytes = measured(
            &(
                fingerprint,
                replay.caller,
                replay.account.0,
                replay.account.1,
                replay.generation,
                replay.remove_at_ns,
            ),
            512,
        )?;
        Ok(replay)
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

    pub(super) fn receipt(&self) -> Option<SessionReceipt> {
        self.session.map(|s| SessionReceipt {
            caller: s.caller,
            issuer: s.login.issuer,
            subject: s.login.subject,
            expires_at_ns: s.expires_at_ns,
        })
    }
}

fn measured(value: &impl Serialize, maximum: usize) -> Result<usize, MemoryError> {
    // Only length is used. The fixed 11-byte frame overhead is retained without
    // declaring a storage identity, writing a format or importing an old reader.
    ic_wire::encoded_len(
        value,
        ic_wire::Encoding::Frame {
            method: 0,
            direction: ic_wire::Direction::Request,
        },
        ic_wire::Limits {
            bytes: maximum,
            allocation: 65_536,
            nodes: 1024,
            depth: 16,
        },
    )
    .map_err(|_| SessionPolicyError::ByteCapacity.into())
}

///
/// SessionSize
///
/// Projects the retained session fields into their accounting encoding.
///

#[derive(Serialize)]
struct SessionSize<'a> {
    caller: Principal,
    subject: Principal,
    issuer: Principal,
    audience: Audience,
    mechanism: Mechanism,
    key_binding_id: &'a [u8; 32],
    role: &'a str,
    scopes: Scopes<'a>,
    generation: u64,
    device_slot: u8,
    device_generation: u64,
    login_started_at_ns: u64,
    login_deadline_ns: u64,
    established_at_ns: u64,
    expires_at_ns: u64,
    fingerprint: &'a [u8; 32],
    request_hash: &'a [u8; 32],
}

///
/// Audience
///
/// Serializes the network and fleet identifiers for session-size accounting.
///

struct Audience(AudienceId);

impl Serialize for Audience {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (
            ic_wire::ByteArray::new(*self.0.canonical_network_id.as_bytes()),
            ic_wire::ByteArray::new(*self.0.fleet_id.as_bytes()),
        )
            .serialize(s)
    }
}

///
/// Mechanism
///
/// Serializes the mechanism tag used for session-size accounting.
///

struct Mechanism(PortableMechanism);

impl Serialize for Mechanism {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            PortableMechanism::Certified => {
                s.serialize_unit_variant("PortableMechanism", 0, "Certified")
            }
            PortableMechanism::Ed25519 => {
                s.serialize_unit_variant("PortableMechanism", 1, "Ed25519")
            }
        }
    }
}

///
/// Scopes
///
/// Serializes the borrowed scope labels for session-size accounting.
///

struct Scopes<'a>(&'a [ScopeLabel]);

impl Serialize for Scopes<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for scope in self.0 {
            seq.serialize_element(scope.as_str())?;
        }
        seq.end()
    }
}
