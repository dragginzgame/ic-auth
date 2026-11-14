use ic_auth_protocol_types::{AudienceId, DelegatedRoleGrant, PortableMechanism, Principal};
use serde::{Deserialize, Serialize};

/// Protected deployment limits; encoded-byte accounting reserves completed rows,
/// not just small pending intents. Allocator/index overhead is governed by the host.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerConfig {
    /// Protected audience shared by every login retained in this store.
    #[serde(with = "crate::storage_codec::audience")]
    pub audience: AudienceId,
    /// Authenticated host issuer identity, not a request-supplied Principal.
    pub issuer: Principal,
    /// Maximum retained login records across all accounts.
    pub max_logins: u64,
    /// Maximum retained issuance and renewal operations across all accounts.
    pub max_operations: u64,
    /// Per-account bound on retained operation records.
    pub max_operations_per_account: u64,
    /// Total encoded-byte reservations, including space for completed results.
    pub max_reserved_bytes: u64,
    /// Maximum concurrent device slots per account, bounded by the protocol ceiling.
    pub max_devices_per_account: u8,
    /// Maximum accumulated fresh issuance/renewal attempts across all accounts.
    pub issuance_burst: u32,
    /// Trusted nanoseconds per fresh intent credit. Creation starts with a full burst.
    pub issuance_refill_interval_ns: u64,
}

/// Host-authorized signing identity, never taken from a submitted request.
/// The host validates enrollment, trust roots, grants and private-key custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerAuthority {
    /// Protected authority generation; changing it invalidates old callback tickets.
    pub generation: u64,
    /// Mechanism the host authorizes this issuer to use.
    pub mechanism: PortableMechanism,
    /// Protected signing-key binding included in issued claims.
    pub key_binding_id: [u8; 32],
    /// Exclusive authority deadline in trusted nanoseconds.
    pub expires_at_ns: u64,
}

/// Host-selected lifetimes retained with each login and issuance intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerPolicy {
    /// Absolute login lifetime from initial issuance; renewal does not extend it.
    pub login_ttl_ns: u64,
    /// Credential admission lifetime, capped by the login and authority deadlines.
    pub proof_ttl_ns: u64,
    /// Time allowed for completing the retained issuance operation.
    pub receipt_timeout_ns: u64,
    /// Additional retention for reconciling completed or interrupted effects.
    pub reconciliation_retention_ns: u64,
}

/// Exact requested credential. `account` is supplied separately by the
/// authenticated host, never inferred from these fields.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssueRequest {
    /// Open issuer operation epoch in which this request is admitted.
    pub epoch: u64,
    /// Idempotency identity scoped to the authenticated account and operation epoch.
    pub operation_id: [u8; 32],
    /// Epoch qualifying the logical login identity.
    pub login_epoch: u64,
    /// Logical login identifier shared by its credentials and renewals.
    pub login_id: [u8; 32],
    /// Raw Ed25519 browser key bound to the retained login.
    pub browser_public_key: [u8; 32],
    /// Host-approved grants; Auth validates their shape, not application permission.
    pub grants: Vec<DelegatedRoleGrant>,
}

/// Renewal selection authenticated against the retained browser/login binding.
/// Supplying an account here does not establish account ownership.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RenewRequest {
    /// Retained account whose browser binding must match the authenticated caller.
    pub account: Principal,
    /// Epoch of the original login, retained across renewals.
    pub login_epoch: u64,
    /// Original logical login identity.
    pub login_id: [u8; 32],
    /// Renewal sequence checked against retained progress and exact retries.
    pub sequence: u64,
    /// Grants selected by the host's current application-authorization policy.
    pub grants: Vec<DelegatedRoleGrant>,
}

/// Browser-bound original inputs for the host's application-permission check.
/// A projection reserves nothing and grants no permission to sign. `renew`
/// rechecks the live binding, sequence, authority and time after host effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenewalSelection {
    /// Account recovered from the browser-authenticated retained login.
    pub account: Principal,
    /// Original issuance inputs for the host's permission recheck.
    pub request: IssueRequest,
    /// Retained lifetime policy; this projection does not extend the login.
    pub policy: IssuerPolicy,
}

/// Exact retained reconciliation evidence, including expired or interrupted work.
/// This does not reauthorize issuance or prove that a credential is still live.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedRenewal {
    /// Original intent and callback binding for reconciliation.
    pub intent: IssuerIntent,
    /// Pending or completed result retained for that exact intent.
    pub status: IssuerStatus,
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
    /// Account to which the retained callback intent belongs.
    #[must_use]
    pub fn account(&self) -> Principal {
        self.account
    }
    /// Operation epoch captured when the intent was reserved.
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Exact operation identity; possession is not authorization to sign.
    #[must_use]
    pub fn operation_id(&self) -> [u8; 32] {
        self.operation_id
    }
    /// Protected authority snapshot that must still match at completion.
    #[must_use]
    pub fn authority(&self) -> IssuerAuthority {
        self.authority
    }
}

/// Original host-effect inputs for resuming a retained intent without a second
/// journal. Reading these fields does not authorize a new signing effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerIntent {
    /// Opaque binding checked against the retained operation before completion.
    pub ticket: IssuerTicket,
    /// Exact request accepted when the operation was reserved.
    pub request: IssueRequest,
    /// Original host-selected lifetimes for this operation.
    pub policy: IssuerPolicy,
    /// Reserved credential nonce, absent until the entropy stage completes.
    pub nonce: Option<[u8; 16]>,
    /// Host approval binding, absent until application authorization is recorded.
    pub approval: Option<[u8; 32]>,
    /// Exclusive credential admission deadline; retries cannot extend it.
    pub admission_deadline_ns: u64,
    /// Exclusive deadline for completing this intent's receipt.
    pub receipt_deadline_ns: u64,
}

/// Original completed bytes. A certified result contains claims only: the host
/// assembles its current witness/certificate through its one certification owner.
/// This is reconciliation evidence, not a current authentication result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuedCredential {
    /// Exact canonical signed claims retained for idempotent reconciliation.
    #[serde(with = "serde_bytes")]
    pub claims: Vec<u8>,
    /// Ed25519 signature when applicable; certified witnesses remain host-owned.
    #[serde(with = "crate::storage_codec::optional_bytes")]
    pub signature: Option<Vec<u8>>,
    /// Original exclusive credential admission deadline in nanoseconds.
    pub admission_deadline_ns: u64,
}

/// Retained issuance progress, not a fresh decision that the credential is valid.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssuerStatus {
    /// Work remains; any callback must use this exact retained binding.
    Pending(IssuerTicket),
    /// Exact completed bytes are available without another signing effect.
    Completed(IssuedCredential),
}

/// Constant-time retained counters, excluding allocator and index overhead.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerOccupancy {
    /// Number of retained logical login records.
    pub logins: u64,
    /// Number of retained pending or completed operations.
    pub operations: u64,
    /// Encoded-byte reservations charged to those records.
    pub reserved_bytes: u64,
}

/// At most the requested budget (hard ceiling 32) records are removed. Records
/// retaining work are not swept: their reference count controls expiry indexing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssuerPruned {
    /// Idle expired login records removed by this bounded call.
    pub logins: u32,
    /// Expired reconciliation operation records removed by this bounded call.
    pub operations: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Header {
    pub issuance_budget: crate::admission_budget::SessionAdmissionBudget,
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
