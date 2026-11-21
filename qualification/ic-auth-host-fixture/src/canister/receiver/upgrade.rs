//! Fixture-owned stable allocation and upgrade transaction.
use super::{Capacity, MemoryConfig, MemoryReceiver, Principal, STATE, Snapshot, State, config};
use crate::canister::issuer;
use ic_auth::receiver_memory::StableReceiver;
use ic_platform_runtime::system as api;
use ic_storage::{
    AllocationGrants, DefaultMemoryImpl, GenericAllocationPolicy, Memory, MemoryAuthority,
    MemoryRequest, MemoryRuntime, RuntimeMemory, SealedDeclarationSnapshot, StableKey,
    adapter::StableCell,
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

pub(super) enum Backend {
    Volatile(MemoryReceiver),
    Stable(StableReceiver, StableCell<Vec<u8>>),
}

impl Deref for Backend {
    type Target = MemoryReceiver;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Volatile(r) => r,
            Self::Stable(r, _) => r.as_ref(),
        }
    }
}

impl DerefMut for Backend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Volatile(r) => r,
            Self::Stable(r, _) => r.state(),
        }
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

type Lease = RuntimeMemory<DefaultMemoryImpl>;
fn leases() -> (Lease, Lease, Lease) {
    let requests =
        [RECEIVER, HOST, "auth.issuer.v1"].map(|name| MemoryRequest::new("auth", name).unwrap());
    let source = SealedDeclarationSnapshot::new(requests.into()).unwrap();
    let grants =
        AllocationGrants::new(vec![MemoryAuthority::new("auth", "auth.").unwrap()]).unwrap();
    let mut runtime = MemoryRuntime::new(DefaultMemoryImpl::default()).unwrap();
    runtime
        .bootstrap(&source, &grants, &GenericAllocationPolicy)
        .unwrap();
    let open = |name| {
        runtime
            .open_memory(&StableKey::parse(name).unwrap())
            .unwrap()
    };
    (open(RECEIVER), open(HOST), open("auth.issuer.v1"))
}

pub(super) fn create(config: MemoryConfig) -> Backend {
    let (receiver, host, issuer) = leases();
    issuer::create_stable(issuer);
    Backend::Stable(
        StableReceiver::new_fresh(receiver, config, api::time()).unwrap(),
        StableCell::create(host, Vec::new()),
    )
}

pub(in crate::canister) fn upgrade_guard() -> Result<(), String> {
    Ok(())
}
pub(in crate::canister) fn decode_upgrade_fault(bytes: &[u8]) -> Result<bool, String> {
    match bytes {
        [] => Ok(false),
        [1] => Ok(true),
        _ => Err("unknown upgrade fault".into()),
    }
}

pub(in crate::canister) fn seal() {
    STATE.with_borrow_mut(|s| {
        let Some(Backend::Stable(receiver, host)) = &mut s.receiver else {
            return;
        };
        receiver.seal().unwrap();
        let bytes = ic_wire::encode(
            &HostState {
                authority: s.authority.unwrap(),
                capacity: s.capacity,
                sequence: s.sequence,
                entry: s.entry.clone(),
                issuer: issuer::seal(),
                application_writes: s.application_writes,
            },
            ic_wire::Encoding::Payload,
            crate::enrollment::LIMITS,
        )
        .unwrap();
        host.set(bytes);
    });
}

pub(in crate::canister) fn reopen(fault: bool) {
    if DefaultMemoryImpl::default().size() == 0 {
        issuer::lost_volatile_state();
        return;
    }
    let (receiver, host, issuer) = leases();
    let cell = StableCell::<Vec<u8>>::load(host).unwrap();
    let saved: HostState = ic_wire::decode(
        cell.get(),
        ic_wire::Encoding::Payload,
        crate::enrollment::LIMITS,
    )
    .unwrap();
    let receiver = StableReceiver::reopen(
        receiver,
        config(saved.authority, saved.capacity),
        api::time(),
    )
    .unwrap();
    issuer::reopen(issuer, &saved.issuer);
    STATE.with_borrow_mut(|s| {
        *s = State {
            receiver: Some(Backend::Stable(receiver, cell)),
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
