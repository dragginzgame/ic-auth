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
}
