use ic_auth_protocol_types::{AudienceId, DelegatedRoleGrant, PortableMechanism, Principal};
use std::{error::Error, fmt};

///
/// IssuerConfig
///
/// Protected issuance/renewal bounds. Physical expired rows still count.
///

#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    feature = "issuer-stable",
    derive(serde::Serialize, serde::Deserialize, Eq, PartialEq)
)]
pub struct IssuerConfig {
    /// Protected network/application and issuer identities.
    pub audience: AudienceId,
    /// Host-selected issuer; cannot be anonymous.
    pub issuer: Principal,
    /// Global retained login limit.
    pub max_logins: u32,
    /// Global retained operation limit.
    pub max_operations: u32,
    /// Per-account retained operation limit.
    pub max_operations_per_account: u32,
    /// Per-account login limit, at most eight device slots.
    pub max_devices_per_account: u8,
    /// Exact canonical claims lengths, plus 64 bytes for each Ed25519 signature.
    /// Counts bound other records; this is not an allocator/heap byte measurement.
    pub max_result_bytes: u64,
    /// Fresh-intent credit starts full, unlike receiver proof/logout budgets.
    pub issuance_burst: u32,
    /// Trusted nanoseconds per replenished fresh-intent credit.
    pub issuance_refill_interval_ns: u64,
}

///
/// IssuerAuthority
///
/// Current host-authorized signing identity. Not request data.
/// The host owns enrollment, key custody and authentication of signing responses.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "issuer-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct IssuerAuthority {
    /// Protected mechanism and, for certification, root-authorized seed commitment.
    pub signing: IssuerSigning,
    /// Nonzero generation; the host must never reuse invalidated generations.
    pub generation: u64,
    /// Protected key binding included in canonical claims.
    pub key_binding_id: [u8; 32],
    /// Exclusive authority deadline.
    pub expires_at_ns: u64,
}

///
/// IssuerSigning
///
/// Host-selected signing capability, captured before any effect.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "issuer-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum IssuerSigning {
    /// Host-authenticated external Ed25519 signing response.
    Ed25519,
    /// Local signature-leaf publication under the enrolled issuer seed.
    Certified {
        /// SHA-256 of the seed authorized by the bound delegation certificate.
        seed_hash: [u8; 32],
    },
}

impl IssuerSigning {
    pub(super) const fn mechanism(self) -> PortableMechanism {
        match self {
            Self::Ed25519 => PortableMechanism::Ed25519,
            Self::Certified { .. } => PortableMechanism::Certified,
        }
    }
}

///
/// IssuerPolicy
///
/// Host-selected immutable lifetime bounds, in trusted nanoseconds.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "issuer-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct IssuerPolicy {
    /// Original absolute login lifetime, at most 90 days.
    pub login_ttl_ns: u64,
    /// Admission lifetime, at most 60 seconds and capped by login/authority expiry.
    pub proof_ttl_ns: u64,
    /// Exclusive interval for accepting completion; never extended on retry.
    pub receipt_timeout_ns: u64,
    /// Additional result/unknown-outcome retention after the later admission or
    /// receipt deadline. Positive; captured at reservation and never extended by retry.
    pub reconciliation_retention_ns: u64,
}

///
/// IssueRequest
///
/// Untrusted exact request. The authenticated account is a separate host input.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssueRequest {
    /// Current operation epoch.
    pub epoch: u64,
    /// Nonzero idempotency identity, scoped to account and epoch.
    pub operation_id: [u8; 32],
    /// Epoch qualifying the retained login identity.
    pub login_epoch: u64,
    /// Nonzero logical login identity.
    pub login_id: [u8; 32],
    /// Raw browser key; possession is not established by these bytes.
    pub browser_public_key: [u8; 32],
    /// Requested canonical grants; application permission is checked by the host.
    pub grants: Vec<DelegatedRoleGrant>,
}

///
/// RenewRequest
///
/// Browser-bound renewal request. Naming an account/login does not authenticate it.
/// Grants must match the retained login and be approved again by the application.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenewRequest {
    /// Account qualifying the retained login.
    pub account: Principal,
    /// Original login epoch; renewals do not replace it with the current epoch.
    pub login_epoch: u64,
    /// Original logical login identity.
    pub login_id: [u8; 32],
    /// Nonzero sequence, advancing exactly one step after completed prior work.
    pub sequence: u64,
    /// Exact original grants, subject to current host application permission.
    pub grants: Vec<DelegatedRoleGrant>,
}

///
/// RenewalSelection
///
/// Read-only browser-authenticated inputs for the host's application approval.
/// This reserves nothing and grants no signing authority; preparation rechecks
/// the live binding after any await. Do not use this account as account authentication.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenewalSelection {
    /// Account whose retained browser binding was checked.
    pub account: Principal,
    /// Exact selected operation/login identity and original browser/grants.
    pub request: IssueRequest,
    /// Original login policy, never replaced by renewal request fields.
    pub policy: IssuerPolicy,
}

///
/// RetainedRenewal
///
/// Original browser-authenticated reconciliation evidence, without fresh authority.
///

#[derive(Debug)]
pub struct RetainedRenewal<'a> {
    /// Original intent including its callback ticket and captured deadlines.
    pub intent: IssuerIntent<'a>,
    /// Retained progress or original completion, even after expiry/revocation.
    pub status: IssuerStatus<'a>,
}

///
/// IssuerTicket
///
/// Callback/effect identity, checked against the retained intent on every mutation.
/// Possession is not account authentication or permission to sign.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IssuerTicket {
    pub(super) account: Principal,
    pub(super) epoch: u64,
    pub(super) operation_id: [u8; 32],
    pub(super) authority: IssuerAuthority,
    pub(super) boot: u64,
}

impl IssuerTicket {
    /// Account qualifying this operation and its external-effect lookup.
    #[must_use]
    pub const fn account(self) -> Principal {
        self.account
    }

    /// Original operation epoch. Pruning closes consumed IDs independently of
    /// the private restart fence, so unrelated live callbacks remain valid.
    #[must_use]
    pub const fn epoch(self) -> u64 {
        self.epoch
    }

    /// Original idempotency identity; never replace it after an uncertain response.
    #[must_use]
    pub const fn operation_id(self) -> [u8; 32] {
        self.operation_id
    }
}

///
/// IssuerIntent
///
/// Retained inputs for the host's exact challenge/application-approval check.
/// This borrowed projection does not reserve or authorize an external effect.
///

#[derive(Debug)]
pub struct IssuerIntent<'a> {
    /// Exact callback binding.
    pub ticket: IssuerTicket,
    /// Original browser key.
    pub browser_public_key: [u8; 32],
    /// Original grants to authenticate before recording approval.
    pub grants: &'a [DelegatedRoleGrant],
    /// First accepted nonce, if entropy has been recorded.
    pub nonce: Option<[u8; 16]>,
    /// First accepted host-authenticated approval commitment.
    pub approval: Option<[u8; 32]>,
    /// Original exclusive admission deadline.
    pub admission_deadline_ns: u64,
    /// Original exclusive completion deadline.
    pub receipt_deadline_ns: u64,
}

///
/// SigningInput
///
/// Exact bytes returned only by the first successful dispatch transition.
/// The host must persist that transition before dispatching and must not redispatch
/// a copied value. Lost responses require lookup by the original ticket.
///

#[derive(Debug)]
pub struct SigningInput {
    pub(super) ticket: IssuerTicket,
    pub(super) claims: Vec<u8>,
}

impl SigningInput {
    /// Exact effect identity captured before external dispatch.
    #[must_use]
    pub const fn ticket(&self) -> IssuerTicket {
        self.ticket
    }

    /// Original canonical schema-2 claims; Ed25519 signs their SHA-256 digest.
    #[must_use]
    pub fn claims(&self) -> &[u8] {
        &self.claims
    }
}

///
/// IssuedProof
///
/// Retained proof evidence. Certified witnesses are assembled by the host at query
/// time; an absent/pruned leaf never permits publishing a replacement on retry.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssuedProof<'a> {
    /// Exact original host-authenticated signature.
    Ed25519(&'a [u8; 64]),
    /// Original certified seed; no query certificate or witness is retained here.
    Certified {
        /// Root-authorized seed commitment captured with the operation.
        seed_hash: [u8; 32],
    },
}

///
/// IssuedCredential
///
/// Exact retained result, not current authority or a renewed credential.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IssuedCredential<'a> {
    /// Original canonical claims.
    pub claims: &'a [u8],
    /// Original signature or certified publication evidence.
    pub proof: IssuedProof<'a>,
    /// Original exclusive admission deadline.
    pub admission_deadline_ns: u64,
}

///
/// IssuerStatus
///
/// Lookup-only progress. Neither pending variant authorizes signing.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssuerStatus<'a> {
    /// Intent retained before external signing dispatch.
    Pending(IssuerTicket),
    /// Dispatch recorded; an absent response is unknown, never permission to retry.
    Unknown(IssuerTicket),
    /// Exact original evidence, readable even after expiry/revocation/restart.
    Completed(IssuedCredential<'a>),
}

///
/// IssuerUsage
///
/// Constant-time physical counts and completed-result reservations.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(
    feature = "issuer-stable",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct IssuerUsage {
    /// Retained logical logins, including revoked and expired rows.
    pub logins: u32,
    /// Retained operations, including unresolved/completed rows.
    pub operations: u32,
    /// Reserved canonical claims and signature bytes.
    pub result_bytes: u64,
}

///
/// IssuerPruned
///
/// Records removed by one bounded maintenance call. Operations have priority;
/// logins newly made idle are eligible only on a later call.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IssuerPruned {
    /// Pending or completed operations past their captured reconciliation deadline.
    pub operations: u32,
    /// Expired logins with no retained operation references.
    pub logins: u32,
}

///
/// IssuerError
///
/// Typed refusal. Records remain unchanged on ordinary errors; trusted clock
/// observations and charged fresh-intent attempts are retained.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssuerError {
    /// Malformed request, configuration, policy or canonical claims.
    Invalid,
    /// Unknown ticket, revoked login or stale protected authority.
    Denied,
    /// An exact operation/login/nonce/approval/result conflicts with retained data.
    Conflict,
    /// Required nonce, approval or dispatch has not been recorded.
    Pending,
    /// Signing was already dispatched; only reconcile the original effect.
    Dispatched,
    /// An exclusive original deadline has elapsed.
    Expired,
    /// New work names a closed operation/login epoch.
    EpochClosed,
    /// Physical counts, result reservation or generation space are exhausted.
    Capacity,
    /// Fresh-intent credit is exhausted.
    Throttled,
    /// Trusted time moved backward.
    Clock,
    /// Explicit close or interrupted publication has disabled this instance.
    Closed,
}

impl fmt::Display for IssuerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid issuer input",
            Self::Denied => "issuer authority denied",
            Self::Conflict => "issuer intent conflicts",
            Self::Pending => "issuer stage pending",
            Self::Dispatched => "issuer signing outcome requires reconciliation",
            Self::Expired => "issuer deadline expired",
            Self::EpochClosed => "issuer epoch closed",
            Self::Capacity => "issuer capacity exhausted",
            Self::Throttled => "issuer attempt budget exhausted",
            Self::Clock => "issuer clock moved backwards",
            Self::Closed => "issuer closed",
        })
    }
}

impl Error for IssuerError {}
