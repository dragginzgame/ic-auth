//! Internal qualification host, never a deployed wallet login provider.
//! Controllers authorize signing and certified branch changes. Installation
//! configures the resource owner; user ingress cannot choose that authority.

use ic_auth::signature_store::{
    SignatureInputs, SignatureLimits, SignatureSibling, SignatureStore,
};
use ic_auth_qualification_contracts as controls;
use ic_platform::Principal;
use ic_platform::certification::{Hash, fork_hash, labeled, leaf};
use ic_wire::{Bytes, Limits, Method, Wire};
use std::cell::RefCell;

mod control;
mod sessions;
mod storage;

/// Stable metadata belongs to this fixture's host. Pending signatures are
/// deliberately volatile; upgrade clears them and republishes the combined root.
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
                ic_platform::cdk::api::canister_self(),
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
        ic_platform::cdk::api::certified_data_set(fork_hash(
            &fork_hash(&self.asset_root(), &self.signatures.root_hash()),
            &self.status_root(),
        ));
    }
}

fn with_state<T>(f: impl FnOnce(&mut State) -> T) -> T {
    STATE.with(|state| f(state.borrow_mut().as_mut().expect("initialized host")))
}

fn controller() -> Result<(), String> {
    let caller = ic_platform::cdk::api::msg_caller();
    if caller == Principal::anonymous() || !ic_platform::cdk::api::is_controller(&caller) {
        return Err("controller required".into());
    }
    Ok(())
}

#[ic_platform::cdk::init(decode_with = "control::decode::<controls::Initialize>")]
fn init(owner: Bytes<29>) {
    let owner = Principal::try_from_slice(owner.as_slice()).expect("owner principal");
    assert_ne!(owner, Principal::anonymous());
    storage::bootstrap(Default::default(), false);
    STATE.with(|state| {
        *state.borrow_mut() = Some(State::new(Metadata {
            owner,
            asset_version: 1,
        }));
    });
    with_state(|state| state.publish());
}

#[ic_platform::cdk::pre_upgrade]
fn pre_upgrade() {
    with_state(|state| {
        let bytes = ic_wire::encode_request::<Checkpoint>(&CheckpointData {
            owner: Bytes::new(state.metadata.owner.as_slice().to_vec()).expect("owner bound"),
            asset_version: state.metadata.asset_version,
            receiver: sessions::capture(),
        })
        .expect("encode Wire fixture checkpoint")
        .into_bytes();
        storage::save(&bytes);
    });
}

#[ic_platform::cdk::post_upgrade(decode_with = "control::decode::<controls::Upgrade>")]
fn post_upgrade() {
    storage::bootstrap(Default::default(), true);
    let CheckpointData {
        owner,
        asset_version,
        receiver,
    } = ic_wire::decode_request::<Checkpoint>(&storage::load())
        .expect("restore Wire fixture checkpoint");
    let metadata = Metadata {
        owner: Principal::try_from_slice(owner.as_slice()).expect("checkpoint owner"),
        asset_version,
    };
    STATE.with(|state| *state.borrow_mut() = Some(State::new(metadata)));
    with_state(|state| state.publish());
    sessions::restore(receiver);
}

#[ic_platform::cdk::update(
    guard = "control::guard",
    decode_with = "control::decode::<controls::Prepare>",
    encode_with = "control::encode_result::<controls::Prepare, _>"
)]
fn prepare(
    controls::PrepareInput {
        input,
        retention_ns,
    }: controls::PrepareInput,
) -> Result<u64, String> {
    let controls::SignatureInput {
        seed,
        domain,
        message,
    } = input;
    controller()?;
    with_state(|state| {
        let prepared = state
            .signatures
            .prepare(
                SignatureInputs {
                    seed: seed.as_slice(),
                    domain: domain.as_slice(),
                    message: message.as_slice(),
                },
                ic_platform::cdk::api::time(),
                retention_ns,
            )
            .map_err(|error| error.to_string())?;
        state.publish();
        Ok(prepared.retrieval_expires_at_ns)
    })
}

#[ic_platform::cdk::query(
    guard = "control::guard",
    decode_with = "control::decode::<controls::Retrieve>",
    encode_with = "control::encode_result::<controls::Retrieve, _>"
)]
fn retrieve(
    controls::SignatureInput {
        seed,
        domain,
        message,
    }: controls::SignatureInput,
) -> Result<ic_auth_wire::model::IcCanisterSignatureProofV1, String> {
    let certificate =
        ic_platform::cdk::api::data_certificate().ok_or("query certificate absent")?;
    with_state(|state| {
        state
            .signatures
            .retrieve(
                SignatureInputs {
                    seed: seed.as_slice(),
                    domain: domain.as_slice(),
                    message: message.as_slice(),
                },
                ic_platform::cdk::api::time(),
                &certificate,
                &[
                    SignatureSibling::Left(state.asset_root()),
                    SignatureSibling::Right(state.status_root()),
                ],
            )
            .map_err(|error| error.to_string())?
            .try_into()
            .map_err(|error: ic_wire::Error| error.to_string())
    })
}

#[ic_platform::cdk::update(
    guard = "control::guard",
    decode_with = "control::decode::<controls::Prune>",
    encode_with = "control::encode_result::<controls::Prune, _>"
)]
fn prune(budget: u32) -> Result<u32, String> {
    controller()?;
    with_state(|state| {
        let removed = state
            .signatures
            .prune(ic_platform::cdk::api::time(), budget as usize)
            .map_err(|error| error.to_string())?;
        state.publish();
        Ok(removed as u32)
    })
}

#[ic_platform::cdk::update(
    guard = "control::guard",
    decode_with = "control::decode::<controls::SetAsset>",
    encode_with = "control::encode_result::<controls::SetAsset, _>"
)]
fn set_asset(version: u64) -> Result<(), String> {
    controller()?;
    with_state(|state| {
        state.metadata.asset_version = version;
        state.publish();
    });
    Ok(())
}

#[ic_platform::cdk::query(
    guard = "control::guard",
    decode_with = "control::decode::<controls::AssetVersion>",
    encode_with = "control::encode::<controls::AssetVersion>"
)]
fn asset_version() -> u64 {
    with_state(|state| state.metadata.asset_version)
}

#[ic_platform::cdk::update(
    guard = "control::guard",
    decode_with = "control::decode::<controls::OwnerOperation>",
    encode_with = "control::encode_result::<controls::OwnerOperation, _>"
)]
fn owner_checked_operation() -> Result<Bytes<29>, String> {
    let caller = ic_platform::cdk::api::msg_caller();
    with_state(|state| {
        if caller == state.metadata.owner {
            Ok(Bytes::new(caller.as_slice().to_vec()).expect("caller bound"))
        } else {
            Err("resource owner required".into())
        }
    })
}

// The maintainer explicitly retired the old unpublished Candid checkpoint.
// Production storage profiles and IC Memory allocation are unaffected.
#[derive(Wire)]
struct CheckpointData {
    #[wire(id = 1)]
    owner: Bytes<29>,
    #[wire(id = 2)]
    asset_version: u64,
    #[wire(id = 3)]
    receiver: Option<sessions::Snapshot>,
}
struct Checkpoint;
impl Method for Checkpoint {
    type Request = CheckpointData;
    type Response = ();
    const ID: u64 = 0x41555448f100;
    const NAME: &'static str = "fixture_checkpoint_v2";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 2097152,
        allocation: 8388608,
        nodes: 65536,
        depth: 64,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
