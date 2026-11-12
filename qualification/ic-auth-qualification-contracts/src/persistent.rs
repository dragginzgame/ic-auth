//! Controller-only installed bookkeeping probes, not production issuer endpoints.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Initialize,
    Fill {
        start: u32,
        count: u32,
        certified: bool,
    },
    Profile {
        account: u32,
        certified: bool,
    },
    Prune {
        budget: u32,
    },
    FillReceiver {
        records: Vec<StoredSession>,
    },
    ProfileReceiver {
        browser_public_key: [u8; 32],
    },
    InspectReceiver {
        after: Option<ic_auth_protocol_types::Principal>,
        limit: u16,
        scan_budget: u16,
    },
    Lifecycle {
        action: IssuerAction,
        login: u32,
        browser_public_key: [u8; 32],
        key_binding_id: [u8; 32],
        certified: bool,
        sequence: u64,
    },
    PruneReceiver {
        budget: u32,
    },
}

/// Controller-only effect adapter. No application endpoint accepts this authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum IssuerAction {
    Prepare,
    Complete {
        signature: Option<ic_wire::Bytes<64>>,
    },
    Recover,
}

/// Controller-injected trusted backing records produced by native real admission.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StoredSession {
    pub session: ic_wire::Bytes<2048>,
    pub replay: ic_wire::Bytes<512>,
    pub login: ic_wire::Bytes<1024>,
}
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub logins: u64,
    pub operations: u64,
    pub epoch: u64,
    pub samples: Vec<(String, u64)>,
    pub inspection: Option<Inspection>,
    pub credential: Option<StoredCredential>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredCredential {
    pub claims: ic_wire::Bytes<8192>,
    pub signature: Option<ic_wire::Bytes<64>>,
    pub completed: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Inspection {
    pub callers: Vec<ic_auth_protocol_types::Principal>,
    pub next_after: Option<ic_auth_protocol_types::Principal>,
    pub scanned: u16,
}
