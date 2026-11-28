//! Fixture-owned stable allocation and upgrade transaction.
use super::{Capacity, MemoryConfig, MemoryReceiver, Principal, STATE, Snapshot, State, config};
use crate::canister::issuer;
use ic_auth::receiver_memory::StableReceiver;
use ic_platform_runtime::system as api;
use ic_storage::{
    AllocationGrants, DefaultMemoryImpl, GenericAllocationPolicy, MemoryAuthority, MemoryRequest,
    MemoryRuntime, RuntimeMemory, SealedDeclarationSnapshot, StableKey, adapter::StableCell,
};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

const RECEIVER: &str = "auth.receiver.v1";
const HOST: &str = "auth.fixture.v1";

///
/// Backend
///
/// Selects the volatile or stable Auth state used by the installed fixture.
///

pub(super) struct Backend(StableReceiver, StableCell<Vec<u8>>, bool);

impl Deref for Backend {
    type Target = MemoryReceiver;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for Backend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.state()
    }
}

///
/// HostState
///
/// Host enrollment and capacity fields saved alongside the receiver checkpoint.
///

#[derive(Serialize, Deserialize)]
struct HostState {
    authority: Principal,
    capacity: Capacity,
    sequence: u64,
    entry: Option<Snapshot>,
    application_writes: u32,
    issuer: issuer::HostState,
}

const HOST_CHECKPOINT_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 8192,
    allocation: 524_288,
    nodes: 65_536,
    depth: 32,
};

type Lease = RuntimeMemory<DefaultMemoryImpl>;
pub(in crate::canister) fn storage_runtime() -> MemoryRuntime<DefaultMemoryImpl> {
    let requests = [
        RECEIVER,
        HOST,
        "auth.issuer.v1",
        "auth.browser.receiver.v1",
        "auth.browser.issuer.v1",
        "auth.certified.receiver.v1",
    ]
    .map(|name| MemoryRequest::new("auth", name).unwrap());
    let source = SealedDeclarationSnapshot::new(requests.into()).unwrap();
    let grants =
        AllocationGrants::new(vec![MemoryAuthority::new("auth", "auth.").unwrap()]).unwrap();
    let mut runtime = MemoryRuntime::new(DefaultMemoryImpl::default()).unwrap();
    runtime
        .bootstrap(&source, &grants, &GenericAllocationPolicy)
        .unwrap();
    runtime
}

pub(in crate::canister) fn open_lease(
    runtime: &MemoryRuntime<DefaultMemoryImpl>,
    name: &str,
) -> Lease {
    runtime
        .open_memory(&StableKey::parse(name).unwrap())
        .unwrap()
}

fn leases(runtime: &MemoryRuntime<DefaultMemoryImpl>) -> (Lease, Lease, Lease) {
    (
        open_lease(runtime, RECEIVER),
        open_lease(runtime, HOST),
        open_lease(runtime, "auth.issuer.v1"),
    )
}

pub(super) fn create(
    runtime: &MemoryRuntime<DefaultMemoryImpl>,
    config: MemoryConfig,
    skip_checkpoint: bool,
) -> Backend {
    let (receiver, host, issuer) = leases(runtime);
    issuer::create_stable(issuer);
    Backend(
        StableReceiver::new_fresh(receiver, config, api::time()).unwrap(),
        StableCell::create(host, Vec::new()),
        skip_checkpoint,
    )
}

pub(in crate::canister) fn upgrade_guard() -> Result<(), String> {
    Ok(())
}
pub(in crate::canister) fn decode_upgrade_settings(bytes: &[u8]) -> Result<(bool, bool), String> {
    match bytes {
        [] => Ok((false, false)),
        [1] => Ok((true, true)),
        [2] => Ok((true, false)),
        _ => Err("unknown upgrade fault".into()),
    }
}

pub(in crate::canister) fn seal() {
    STATE.with_borrow_mut(|s| {
        let Some(Backend(receiver, host, skip_checkpoint)) = &mut s.receiver else {
            return;
        };
        if *skip_checkpoint {
            return;
        }
        receiver
            .seal(api::canister_version())
            .unwrap_or_else(|e| api::trap(&format!("receiver seal: {e:?}")));
        let bytes = crate::codec::encode(
            &HostState {
                authority: s.authority.unwrap(),
                capacity: s.capacity,
                sequence: s.sequence,
                entry: s.entry.clone(),
                issuer: issuer::seal(),
                application_writes: s.application_writes,
            },
            ic_wire::Encoding::Payload,
            HOST_CHECKPOINT_LIMITS,
        )
        .unwrap_or_else(|e| api::trap(&format!("host seal: {e:?}")));
        host.set(bytes);
    });
}

pub(in crate::canister) fn reopen((grow, fault): (bool, bool)) {
    let (receiver, host, issuer) = leases(&storage_runtime());
    let cell = StableCell::<Vec<u8>>::load(host).unwrap();
    if cell.get().is_empty() {
        issuer::lost_volatile_state();
        return;
    }
    let mut saved: HostState = crate::codec::decode(
        cell.get(),
        ic_wire::Encoding::Payload,
        HOST_CHECKPOINT_LIMITS,
    )
    .unwrap();
    if grow {
        saved.capacity.logins = saved.capacity.logins.checked_add(1).unwrap();
    }
    let receiver = StableReceiver::reopen(
        receiver,
        config(saved.authority, saved.capacity),
        api::time(),
        api::canister_version(),
    );
    let issuer = issuer::reopen(issuer, &saved.issuer);
    let receiver = match (receiver, issuer) {
        (Ok(receiver), Ok(())) => receiver,
        (receiver, issuer) => api::trap(&format!(
            "Auth reopen failed: receiver {:?}, issuer {:?}",
            receiver.err(),
            issuer.err()
        )),
    };
    STATE.with_borrow_mut(|s| {
        *s = State {
            receiver: Some(Backend(receiver, cell, false)),
            authority: Some(saved.authority),
            sequence: saved.sequence,
            entry: saved.entry,
            application_writes: saved.application_writes,
            capacity: saved.capacity,
            wide_scopes: saved.capacity.wide_scopes,
        }
    });
    if fault {
        api::trap("after stable receiver reopen");
    }
}
