//! Shared receiver fixture setup and observation transport; scenarios own assertions.
use super::{frame, receiver};
use ic_auth_host_fixture::{
    Installation, RESPONSE,
    enrollment::Fault,
    receiver::{CONTROL, Capacity, Command, MEASURE, Measurement, Outcome, REQUEST},
};
use ic_testkit::pocket_ic::{CanisterId, nonblocking::PocketIc};

pub(super) async fn install(pic: &PocketIc, wasm: &[u8], limits: Capacity) -> CanisterId {
    let id = pic.create_canister().await;
    pic.add_cycles(id, 5_000_000_000_000).await;
    let init = ic_wire::encode(
        &Installation {
            authority: Vec::new(),
            certified_network_key: Vec::new(),
            skip_auth_checkpoint: false,
            receiver_capacity: Some(limits),
        },
        ic_wire::Encoding::Payload,
        ic_auth_host_fixture::enrollment::LIMITS,
    )
    .unwrap();
    assert!(init.len() <= 256);
    pic.install_canister(id, wasm.to_vec(), init, None).await;
    id
}

pub(super) async fn enroll(pic: &PocketIc, id: CanisterId, caller: CanisterId) {
    assert_eq!(
        receiver::control(pic, id, caller, &Command::Enroll(Fault::None))
            .await
            .unwrap(),
        Ok(Outcome::Enrolled)
    );
}

/// Decode and check the measurement envelope without deciding the expected outcome.
pub(super) async fn measure(
    pic: &PocketIc,
    id: CanisterId,
    caller: CanisterId,
    command: &Command,
) -> (Measurement, usize) {
    let request = ic_wire::encode(
        command,
        frame(MEASURE, ic_wire::Direction::Request),
        REQUEST,
    )
    .unwrap();
    let request_bytes = request.len();
    let reply = pic
        .update_call(id, caller, "receiver_fixture_measure", request)
        .await
        .unwrap();
    let sample: Measurement = ic_wire::decode(
        &reply,
        frame(MEASURE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    assert!(sample.instructions > 0);
    assert!(sample.heap_pages.1 >= sample.heap_pages.0);
    let ordinary = ic_wire::encode(
        &sample.result,
        frame(CONTROL, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    assert_eq!(sample.response_bytes, ordinary.len() as u64);
    (sample, request_bytes)
}

pub(super) async fn measured(
    pic: &PocketIc,
    id: CanisterId,
    caller: CanisterId,
    command: &Command,
    size: u32,
    name: &str,
) -> Outcome {
    let (sample, request_bytes) = measure(pic, id, caller, command).await;
    println!(
        "receiver-cost size={size} {name} instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={}",
        sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
    );
    sample.result.unwrap()
}
