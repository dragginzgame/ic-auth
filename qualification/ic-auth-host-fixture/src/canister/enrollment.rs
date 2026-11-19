//! Actual inter-canister fixture responder; never a production enrollment service.
use crate::{
    enrollment::{CONFIGURE, KEY, LIMITS, METHOD, Scenario, Snapshot},
    receiver::TIME_ORIGIN_NS,
};
use ic_platform_runtime::system as api;
use std::cell::Cell;

thread_local! { static SCENARIO: Cell<Scenario> = const { Cell::new(Scenario::Initial) }; }

/// Reject oversized ingress before copying or decoding the request.
pub(super) fn bounded() -> Result<(), String> {
    if api::argument_size() > LIMITS.bytes {
        Err("oversized enrollment frame".into())
    } else {
        Ok(())
    }
}

/// Require the controller caller before changing the fixture scenario.
pub(super) fn controller() -> Result<(), String> {
    bounded()?;
    if api::is_controller(&api::caller()) {
        Ok(())
    } else {
        Err("fixture controller required".into())
    }
}

pub(super) fn decode_scenario(bytes: &[u8]) -> Result<Scenario, ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: CONFIGURE,
            direction: ic_wire::Direction::Request,
        },
        LIMITS,
    )
}

pub(super) fn encode_unit((): ()) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &(),
        ic_wire::Encoding::Frame {
            method: CONFIGURE,
            direction: ic_wire::Direction::Response,
        },
        LIMITS,
    )
}

pub(super) fn configure(scenario: Scenario) {
    SCENARIO.set(scenario);
}

pub(super) fn decode_request(bytes: &[u8]) -> Result<[u8; 32], ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: METHOD,
            direction: ic_wire::Direction::Request,
        },
        LIMITS,
    )
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform owns the handler result"
)]
pub(super) fn encode_snapshot(value: Snapshot) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: METHOD,
            direction: ic_wire::Direction::Response,
        },
        LIMITS,
    )
}

pub(super) fn snapshot(challenge: [u8; 32]) -> Snapshot {
    let mut result = Snapshot {
        receiver: api::caller().as_slice().to_vec(),
        challenge,
        revision: 1,
        generation: 7,
        issued_at_ns: api::time(),
        expires_at_ns: TIME_ORIGIN_NS + 150_000_000_000,
        key: KEY,
        revoked: false,
    };
    match SCENARIO.get() {
        Scenario::Initial => {}
        Scenario::Refresh => {
            result.revision = 2;
            result.expires_at_ns += 30_000_000_000;
        }
        Scenario::WrongReceiver => result.receiver = vec![8],
        Scenario::WrongChallenge => result.challenge[0] ^= 1,
        Scenario::Expired => result.expires_at_ns = result.issued_at_ns,
        Scenario::WrongKey => result.key[0] ^= 1,
        Scenario::Regression => result.revision = 0,
        Scenario::Unavailable => api::trap("fixture authority unavailable"),
        Scenario::Revoke => {
            result.revision = 3;
            result.generation = 8;
            result.revoked = true;
        }
        Scenario::Reenable => {
            result.revision = 4;
            result.generation = 9;
        }
    }
    result
}
