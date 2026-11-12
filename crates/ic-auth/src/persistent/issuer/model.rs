use ic_auth_protocol_types::{AudienceId, DelegatedRoleGrant, PortableMechanism, Principal};
use serde::{Deserialize, Serialize};

/// Protected deployment limits; encoded-byte accounting reserves completed rows,
/// not just small pending intents. Allocator/index overhead is governed by the host.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerConfig {
    #[serde(with = "crate::storage_codec::audience")]
    pub audience: AudienceId,
    pub issuer: Principal,
    pub max_logins: u64,
    pub max_operations: u64,
    pub max_operations_per_account: u64,
    pub max_reserved_bytes: u64,
    pub max_devices_per_account: u8,
}

/// Host-authorized signing identity, never taken from a submitted request.
/// The host validates enrollment, trust roots, grants and private-key custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerAuthority {
    pub generation: u64,
    pub mechanism: PortableMechanism,
    pub key_binding_id: [u8; 32],
    pub expires_at_ns: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerPolicy {
    pub login_ttl_ns: u64,
    pub proof_ttl_ns: u64,
    pub receipt_timeout_ns: u64,
    pub reconciliation_retention_ns: u64,
}

/// Exact requested credential. `account` is supplied separately by the
/// authenticated host, never inferred from these fields.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssueRequest {
    pub epoch: u64,
    pub operation_id: [u8; 32],
    pub login_epoch: u64,
    pub login_id: [u8; 32],
    pub browser_public_key: [u8; 32],
    pub grants: Vec<DelegatedRoleGrant>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RenewRequest {
    pub account: Principal,
    pub login_epoch: u64,
    pub login_id: [u8; 32],
    pub sequence: u64,
    pub grants: Vec<DelegatedRoleGrant>,
}

/// Opaque callback binding. Reconstructed tickets are checked against the exact
/// retained intent; possession alone is not signing authority.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerTicket {
    pub(super) account: Principal,
    pub(super) epoch: u64,
    pub(super) operation_id: [u8; 32],
    pub(super) boot: u64,
    pub(super) authority: IssuerAuthority,
}
impl IssuerTicket {
    pub fn account(&self) -> Principal {
        self.account
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn operation_id(&self) -> [u8; 32] {
        self.operation_id
    }
    pub fn authority(&self) -> IssuerAuthority {
        self.authority
    }
}

/// Original host-effect inputs for resuming a retained intent without a second
/// journal. Reading these fields does not authorize a new signing effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerIntent {
    pub ticket: IssuerTicket,
    pub request: IssueRequest,
    pub policy: IssuerPolicy,
    pub nonce: Option<[u8; 16]>,
    pub approval: Option<[u8; 32]>,
    pub admission_deadline_ns: u64,
    pub receipt_deadline_ns: u64,
}

/// Original completed bytes. A certified result contains claims only: the host
/// assembles its current witness/certificate through its one certification owner.
/// This is reconciliation evidence, not a current authentication result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuedCredential {
    #[serde(with = "serde_bytes")]
    pub claims: Vec<u8>,
    #[serde(with = "crate::storage_codec::optional_bytes")]
    pub signature: Option<Vec<u8>>,
    pub admission_deadline_ns: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssuerStatus {
    Pending(IssuerTicket),
    Completed(IssuedCredential),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerOccupancy {
    pub logins: u64,
    pub operations: u64,
    pub reserved_bytes: u64,
}

/// At most the requested budget (hard ceiling 32) records are removed. Records
/// retaining work are not swept: their reference count controls expiry indexing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerPruned {
    pub logins: u32,
    pub operations: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Header {
    pub schema: u64,
    pub config: IssuerConfig,
    pub epoch: u64,
    pub boot: u64,
    pub generation: u64,
    pub observed_ns: u64,
    pub occupancy: IssuerOccupancy,
    pub renewal_indexes: u64,
    pub idle_logins: u64,
    pub operation_accounts: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct LoginId {
    pub account: Principal,
    pub epoch: u64,
    pub id: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Login {
    pub id: LoginId,
    pub browser_public_key: [u8; 32],
    pub slot: u8,
    pub generation: u64,
    pub started_ns: u64,
    pub deadline_ns: u64,
    pub grants_hash: [u8; 32],
    pub policy: IssuerPolicy,
    pub sequence: u64,
    pub completed: bool,
    pub references: u64,
    pub revoked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Operation {
    pub ticket: IssuerTicket,
    pub request: IssueRequest,
    pub policy: IssuerPolicy,
    pub login: LoginId,
    pub slot: u8,
    pub device_generation: u64,
    pub started_ns: u64,
    pub deadline_ns: u64,
    pub issued_ns: u64,
    pub admission_ns: u64,
    pub receipt_ns: u64,
    pub remove_ns: u64,
    pub renewal: Option<u64>,
    pub nonce: Option<[u8; 16]>,
    pub approval: Option<[u8; 32]>,
    pub result: Option<IssuedCredential>,
    pub reserved_bytes: u64,
}
