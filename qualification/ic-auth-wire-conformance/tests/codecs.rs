//! Compare published reference/current suppliers with Auth's fixed schemas.
//! Expected bytes originate from the independently executed reference browser.

use ic_auth_wire::{
    CredentialEnvelope, EstablishSession, Logout, ReadSession, SessionInput, SessionView,
    model::{
        AdmissionRefusal, LogoutResult, PortableCredential, PortableProof, SessionReadRefusal,
    },
    wire::{self as current, Direction, Encoding, Limits},
};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

fn fixtures() -> Vec<(&'static str, Vec<u8>)> {
    [
        include_str!("../../../crates/ic-auth-wire/tests/fixtures/caller-frames.tsv"),
        include_str!("../../../crates/ic-auth-wire/tests/fixtures/admission-frames.tsv"),
    ]
    .into_iter()
    .flat_map(str::lines)
    .map(|line| {
        let (name, hex) = line.split_once('\t').unwrap();
        (
            name,
            hex.as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect(),
        )
    })
    .collect()
}

fn old_limits(limits: Limits) -> ic_wire::Limits {
    ic_wire::Limits {
        bytes: limits.bytes,
        allocation: limits.allocation,
        nodes: limits.nodes,
        depth: limits.depth,
    }
}

fn old_encode<T: Serialize>(
    value: &T,
    id: u64,
    response: bool,
    limits: Limits,
) -> Result<Vec<u8>, ic_wire::Error> {
    if response {
        ic_wire::native::encode_response(id, value, old_limits(limits))
    } else {
        ic_wire::native::encode_request(id, value, old_limits(limits))
    }
}

fn old_decode<T: DeserializeOwned>(
    bytes: &[u8],
    id: u64,
    response: bool,
    limits: Limits,
) -> Result<T, ic_wire::Error> {
    if response {
        ic_wire::native::decode_response(id, bytes, old_limits(limits))
    } else {
        ic_wire::native::decode_request(id, bytes, old_limits(limits))
    }
}

fn check<T: Serialize + DeserializeOwned + PartialEq + Debug>(
    bytes: &[u8],
    id: u64,
    limits: Limits,
) {
    let response = bytes[10] == 1;
    let encoding = Encoding::Frame {
        method: id,
        direction: if response {
            Direction::Response
        } else {
            Direction::Request
        },
    };
    let original: T = old_decode(bytes, id, response, limits).unwrap();
    assert_eq!(
        current::decode::<T>(bytes, encoding, limits).unwrap(),
        original
    );
    assert_eq!(old_encode(&original, id, response, limits).unwrap(), bytes);
    assert_eq!(current::encode(&original, encoding, limits).unwrap(), bytes);

    // Independently find each old supplier admission boundary, then require the
    // current supplier to preserve below/at/above acceptance for encode AND decode.
    // Encode and decode have distinct accounting; compare them separately.
    for field in 0..4 {
        for decode in [false, true] {
            let set = |value| {
                let mut selected = limits;
                match field {
                    0 => selected.bytes = value,
                    1 => selected.allocation = value,
                    2 => selected.nodes = value,
                    _ => selected.depth = value,
                }
                selected
            };
            let maximum = [limits.bytes, limits.allocation, limits.nodes, limits.depth][field];
            let accepted = |amount| {
                if decode {
                    old_decode::<T>(bytes, id, response, set(amount)).is_ok()
                } else {
                    old_encode(&original, id, response, set(amount)).is_ok()
                }
            };
            let (mut low, mut high) = (0, maximum);
            while low < high {
                let middle = low + (high - low) / 2;
                if accepted(middle) {
                    high = middle;
                } else {
                    low = middle + 1;
                }
            }
            assert!(low > 0 && accepted(low));
            for amount in [low - 1, low, low + 1] {
                let now = if decode {
                    current::decode::<T>(bytes, encoding, set(amount)).is_ok()
                } else {
                    current::encode(&original, encoding, set(amount)).is_ok()
                };
                assert_eq!(
                    now,
                    accepted(amount),
                    "id={id:x}, response={response}, field={field}, decode={decode}, allowance={amount}"
                );
            }
        }
    }
}

#[test]
fn fixed_frames_and_each_resource_boundary_match_the_reference_supplier() {
    for (name, bytes) in fixtures() {
        if name.starts_with("credential-") {
            check::<PortableCredential>(
                &bytes,
                CredentialEnvelope::ID,
                CredentialEnvelope::REQUEST_LIMITS,
            );
        } else if name == "establish-none" || name == "establish-some" {
            check::<SessionInput>(
                &bytes,
                EstablishSession::ID,
                EstablishSession::REQUEST_LIMITS,
            );
        } else if name.starts_with("establish-") {
            check::<Result<SessionView, AdmissionRefusal>>(
                &bytes,
                EstablishSession::ID,
                EstablishSession::RESPONSE_LIMITS,
            );
        } else if name == "read-request" {
            check::<()>(&bytes, ReadSession::ID, ReadSession::REQUEST_LIMITS);
        } else if name == "logout-request" {
            check::<()>(&bytes, Logout::ID, Logout::REQUEST_LIMITS);
        } else if name.starts_with("read-") {
            check::<Result<SessionView, SessionReadRefusal>>(
                &bytes,
                ReadSession::ID,
                ReadSession::RESPONSE_LIMITS,
            );
        } else {
            check::<LogoutResult>(&bytes, Logout::ID, Logout::RESPONSE_LIMITS);
        }
    }
}

#[test]
fn aggregate_allocation_refusal_matches_without_changing_field_caps() {
    let (_, frame) = fixtures()
        .into_iter()
        .find(|(name, _)| *name == "credential-certified-request")
        .unwrap();
    let limits = CredentialEnvelope::REQUEST_LIMITS;
    let mut value: PortableCredential =
        old_decode(&frame, CredentialEnvelope::ID, false, limits).unwrap();
    let PortableProof::Certified(proof) = &mut value.proof else {
        unreachable!()
    };
    proof.issuer_signature.signature_cbor = current::Bytes::new(vec![0; 131072]).unwrap();
    let frame = old_encode(&value, CredentialEnvelope::ID, false, limits).unwrap();
    assert_eq!(CredentialEnvelope::encode_request(&value).unwrap(), frame);
    assert_eq!(CredentialEnvelope::decode_request(&frame).unwrap(), value);
    value.claims = current::Bytes::new(vec![0; 65536]).unwrap();
    assert!(old_encode(&value, CredentialEnvelope::ID, false, limits).is_err());
    assert_eq!(
        CredentialEnvelope::encode_request(&value),
        Err(current::Error::Limit)
    );
    // Produce only test input with a larger allowance, then compare refusals at
    // the unchanged production allowance. No product cap is changed.
    let larger = Limits {
        allocation: limits.allocation * 2,
        ..limits
    };
    let frame = old_encode(&value, CredentialEnvelope::ID, false, larger).unwrap();
    assert!(
        old_decode::<PortableCredential>(&frame, CredentialEnvelope::ID, false, limits).is_err()
    );
    assert_eq!(
        CredentialEnvelope::decode_request(&frame),
        Err(current::Error::Limit)
    );
}
