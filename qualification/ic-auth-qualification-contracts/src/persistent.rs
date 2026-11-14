//! Controller-only installed bookkeeping probes, not production issuer endpoints.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// Controller-only commands exercising Auth-owned durable stores on disposable backing.
pub enum Command {
    /// Create the fixture issuer and receiver stores with fixed bounded policies.
    Initialize,
    /// Populate issuer operations using deterministic test accounts.
    Fill {
        /// First deterministic account index to populate.
        start: u32,
        /// Number of deterministic accounts to populate in this invocation.
        count: u32,
        /// Select the certified test mechanism instead of Ed25519.
        certified: bool,
    },
    /// Measure issuer bookkeeping for one deterministic test account.
    Profile {
        /// Deterministic account index selected by the controller.
        account: u32,
        /// Select the certified test mechanism instead of Ed25519.
        certified: bool,
    },
    /// Run bounded expiry cleanup on the issuer store.
    Prune {
        /// Requested record-work budget, still bounded by the Auth owner.
        budget: u32,
    },
    /// Inject native-verified session/replay/login triples into trusted fixture backing.
    FillReceiver {
        /// Records to install through the controller-only qualification path.
        records: Vec<StoredSession>,
    },
    /// Measure point lookup and authorization for one retained browser identity.
    ProfileReceiver {
        /// Browser key selecting the deterministic test login or lookup identity.
        browser_public_key: [u8; 32],
    },
    /// Read a bounded receiver inspection page without an unbounded store scan.
    InspectReceiver {
        /// Exclusive caller cursor from the previous page, or start of the index.
        after: Option<ic_auth_protocol_types::Principal>,
        /// Maximum sessions requested in this page.
        limit: u16,
        /// Maximum index entries examined, independently bounding empty/filtered work.
        scan_budget: u16,
    },
    /// Exercise retained issuer preparation, completion or recovery.
    Lifecycle {
        /// Lifecycle stage to invoke under controller authority.
        action: IssuerAction,
        /// Deterministic logical-login index used by the scenario.
        login: u32,
        /// Browser key selecting the deterministic test login or lookup identity.
        browser_public_key: [u8; 32],
        /// Controller-selected signing-key binding for the test authority.
        key_binding_id: [u8; 32],
        /// Select the certified test mechanism instead of Ed25519.
        certified: bool,
        /// Renewal sequence used for retained-operation reconciliation.
        sequence: u64,
    },
    /// Run bounded cleanup on receiver session/replay/denial indexes.
    PruneReceiver {
        /// Requested record-work budget, still bounded by the Auth owner.
        budget: u32,
    },
}

/// Controller-only effect adapter. No application endpoint accepts this authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum IssuerAction {
    /// Reserve a retained issuer intent without completing its signing effect.
    Prepare,
    /// Complete the reserved intent with the supplied test signature material.
    Complete {
        /// Ed25519 signature when needed; certified completion retains claims only.
        signature: Option<ic_wire::Bytes<64>>,
    },
    /// Read the exact retained operation result without issuing again.
    Recover,
}

/// Controller-injected trusted backing records produced by native real admission.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StoredSession {
    /// Checked Auth session record encoded with its private storage domain.
    pub session: ic_wire::Bytes<2048>,
    /// Matching consumed-credential replay record.
    pub replay: ic_wire::Bytes<512>,
    /// Matching authenticated login reservation record.
    pub login: ic_wire::Bytes<1024>,
}
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
/// Bounded probe response containing occupancy and named instruction samples.
pub struct Report {
    /// Retained issuer login count observed by the probe.
    pub logins: u64,
    /// Retained issuer operation count observed by the probe.
    pub operations: u64,
    /// Issuer operation epoch observed by the probe.
    pub epoch: u64,
    /// Named instruction-counter samples; not worst-case cycle guarantees.
    pub samples: Vec<(String, u64)>,
    /// Receiver inspection result for commands that request a page.
    pub inspection: Option<Inspection>,
    /// Retained issuance result or pending material for lifecycle reconciliation.
    pub credential: Option<StoredCredential>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Controller-only projection of retained issuance material and completion state.
pub struct StoredCredential {
    /// Exact canonical claims bytes produced by the issuer scenario.
    pub claims: ic_wire::Bytes<8192>,
    /// Retained Ed25519 signature, absent for certified or unfinished work.
    pub signature: Option<ic_wire::Bytes<64>>,
    /// Whether the operation has reached its completed state.
    pub completed: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// Bounded caller projection from an Auth receiver inspection page.
pub struct Inspection {
    /// Caller identities returned by the inspected session rows.
    pub callers: Vec<ic_auth_protocol_types::Principal>,
    /// Exclusive continuation cursor when further index work may remain.
    pub next_after: Option<ic_auth_protocol_types::Principal>,
    /// Index entries examined, including entries not returned as sessions.
    pub scanned: u16,
}
