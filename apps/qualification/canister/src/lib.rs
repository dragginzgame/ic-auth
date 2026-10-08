//! Internal qualification host, never a deployed wallet login provider.
//! Controllers authorize signing and certified branch changes. Installation
//! configures the resource owner; user ingress cannot choose that authority.

use candid::{CandidType, Principal};
use ic_auth::signature_store::{
    SignatureInputs, SignatureLimits, SignatureSibling, SignatureStore,
};
use ic_auth_protocol_types::IcCanisterSignatureProofV1;
use ic_certification::{Hash, fork_hash, labeled, leaf};
use serde::Deserialize;
use std::cell::RefCell;

const SEED: &[u8] = b"qualification-identity";

/// Stable metadata belongs to this fixture's host. Pending signatures are
/// deliberately volatile; upgrade clears them and republishes the combined root.
#[derive(CandidType, Deserialize)]
struct Metadata {
    owner: Principal,
    asset_version: u64,
}

struct State {
    metadata: Metadata,
    signatures: SignatureStore,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

impl State {
    fn new(metadata: Metadata) -> Self {
        Self {
            metadata,
            signatures: SignatureStore::new(
                ic_cdk::api::canister_self(),
                SignatureLimits {
                    max_signatures: 16,
                    max_message_bytes: 1024,
                    max_certificate_bytes: 64 * 1024,
                    max_signature_bytes: 128 * 1024,
                },
            )
            .expect("fixed qualification limits"),
        }
    }

    fn asset_root(&self) -> Hash {
        labeled(b"assets", leaf(self.metadata.asset_version.to_be_bytes())).digest()
    }

    fn status_root(&self) -> Hash {
        labeled(b"status", leaf(b"qualification".to_vec())).digest()
    }

    fn publish(&self) {
        ic_cdk::api::certified_data_set(fork_hash(
            &fork_hash(&self.asset_root(), &self.signatures.root_hash()),
            &self.status_root(),
        ));
    }
}

fn with_state<T>(f: impl FnOnce(&mut State) -> T) -> T {
    STATE.with(|state| f(state.borrow_mut().as_mut().expect("initialized host")))
}

fn controller() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    if caller == Principal::anonymous() || !ic_cdk::api::is_controller(&caller) {
        return Err("controller required".into());
    }
    Ok(())
}

#[ic_cdk::init]
fn init(owner: Principal) {
    assert_ne!(owner, Principal::anonymous());
    STATE.with(|state| {
        *state.borrow_mut() = Some(State::new(Metadata {
            owner,
            asset_version: 1,
        }));
    });
    with_state(|state| state.publish());
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    with_state(|state| {
        ic_cdk::storage::stable_save((&state.metadata,)).expect("save fixture metadata");
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    let (metadata,): (Metadata,) =
        ic_cdk::storage::stable_restore().expect("restore fixture metadata");
    STATE.with(|state| *state.borrow_mut() = Some(State::new(metadata)));
    with_state(|state| state.publish());
}

#[ic_cdk::update]
fn prepare(domain: Vec<u8>, message: Vec<u8>, retention_ns: u64) -> Result<u64, String> {
    controller()?;
    with_state(|state| {
        let prepared = state
            .signatures
            .prepare(
                SignatureInputs {
                    seed: SEED,
                    domain: &domain,
                    message: &message,
                },
                ic_cdk::api::time(),
                retention_ns,
            )
            .map_err(|error| error.to_string())?;
        state.publish();
        Ok(prepared.retrieval_expires_at_ns)
    })
}

#[ic_cdk::query]
fn retrieve(domain: Vec<u8>, message: Vec<u8>) -> Result<IcCanisterSignatureProofV1, String> {
    let certificate = ic_cdk::api::data_certificate().ok_or("query certificate absent")?;
    with_state(|state| {
        state
            .signatures
            .retrieve(
                SignatureInputs {
                    seed: SEED,
                    domain: &domain,
                    message: &message,
                },
                ic_cdk::api::time(),
                &certificate,
                &[
                    SignatureSibling::Left(state.asset_root()),
                    SignatureSibling::Right(state.status_root()),
                ],
            )
            .map_err(|error| error.to_string())
    })
}

#[ic_cdk::update]
fn prune(budget: u32) -> Result<u32, String> {
    controller()?;
    with_state(|state| {
        let removed = state
            .signatures
            .prune(ic_cdk::api::time(), budget as usize)
            .map_err(|error| error.to_string())?;
        state.publish();
        Ok(removed as u32)
    })
}

#[ic_cdk::update]
fn set_asset(version: u64) -> Result<(), String> {
    controller()?;
    with_state(|state| {
        state.metadata.asset_version = version;
        state.publish();
    });
    Ok(())
}

#[ic_cdk::query]
fn asset_version() -> u64 {
    with_state(|state| state.metadata.asset_version)
}

#[ic_cdk::update]
fn owner_checked_operation() -> Result<Principal, String> {
    let caller = ic_cdk::api::msg_caller();
    with_state(|state| {
        if caller == state.metadata.owner {
            Ok(caller)
        } else {
            Err("resource owner required".into())
        }
    })
}

ic_cdk::export_candid!();
