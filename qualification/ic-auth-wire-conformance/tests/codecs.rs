//! Check current Wire against independent current bytes and resource expectations.
//! Expectations were retained before removing the historical Wire dependency.

use ic_auth_wire::{
    CredentialEnvelope, EstablishSession, Logout, ReadSession, SessionInput, SessionView,
    model::{
        AdmissionRefusal, LogoutResult, PortableCredential, PortableProof, SessionReadRefusal,
    },
};
use ic_wire::{self as current, Direction, Encoding, Limits};
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

fn framing(id: u64, response: bool) -> Encoding {
    Encoding::Frame {
        method: id,
        direction: if response {
            Direction::Response
        } else {
            Direction::Request
        },
    }
}

fn encode<T: Serialize>(
    value: &T,
    id: u64,
    response: bool,
    limits: Limits,
) -> Result<Vec<u8>, current::Error> {
    current::encode(value, framing(id, response), limits)
}

fn decode<T: DeserializeOwned>(
    bytes: &[u8],
    id: u64,
    response: bool,
    limits: Limits,
) -> Result<T, current::Error> {
    current::decode(bytes, framing(id, response), limits)
}

fn check<T: Serialize + DeserializeOwned + PartialEq + Debug>(
    name: &str,
    bytes: &[u8],
    id: u64,
    limits: Limits,
) {
    let response = bytes[10] == 1;
    let original: T = decode(bytes, id, response, limits).unwrap();
    assert_eq!(encode(&original, id, response, limits).unwrap(), bytes);

    // Fixed values measured from published 0.22.2 before its removal. Never
    // derive expected admission from the current codec under test.
    let thresholds: Vec<_> = include_str!("fixtures/resource-thresholds.tsv")
        .lines()
        .filter_map(|line| {
            let columns: Vec<_> = line.split('\t').collect();
            assert_eq!(columns.len(), 4);
            (columns[0] == name).then(|| {
                (
                    columns[1].parse::<usize>().unwrap(),
                    columns[2].parse::<bool>().unwrap(),
                    columns[3].parse::<usize>().unwrap(),
                )
            })
        })
        .collect();
    assert_eq!(
        thresholds.len(),
        8,
        "missing or extra reference thresholds for {name}"
    );
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
            let matches: Vec<_> = thresholds
                .iter()
                .filter(|(f, d, _)| *f == field && *d == decode)
                .collect();
            assert_eq!(
                matches.len(),
                1,
                "duplicate or missing threshold for {name}"
            );
            let low = matches[0].2;
            assert!(low > 0);
            for amount in [low - 1, low, low + 1] {
                let now = if decode {
                    current::decode::<T>(bytes, framing(id, response), set(amount)).is_ok()
                } else {
                    current::encode(&original, framing(id, response), set(amount)).is_ok()
                };
                assert_eq!(
                    now,
                    amount >= low,
                    "frame={name}, field={field}, decode={decode}, allowance={amount}"
                );
            }
        }
    }
}

#[test]
fn current_frames_and_each_resource_boundary_match_fixed_expectations() {
    for (name, bytes) in fixtures() {
        if name.starts_with("credential-") {
            check::<PortableCredential>(
                name,
                &bytes,
                CredentialEnvelope::ID,
                CredentialEnvelope::REQUEST_LIMITS,
            );
        } else if name == "establish-none" || name == "establish-some" {
            check::<SessionInput>(
                name,
                &bytes,
                EstablishSession::ID,
                EstablishSession::REQUEST_LIMITS,
            );
        } else if name.starts_with("establish-") {
            check::<Result<SessionView, AdmissionRefusal>>(
                name,
                &bytes,
                EstablishSession::ID,
                EstablishSession::RESPONSE_LIMITS,
            );
        } else if name == "read-request" {
            check::<()>(name, &bytes, ReadSession::ID, ReadSession::REQUEST_LIMITS);
        } else if name == "logout-request" {
            check::<()>(name, &bytes, Logout::ID, Logout::REQUEST_LIMITS);
        } else if name.starts_with("read-") {
            check::<Result<SessionView, SessionReadRefusal>>(
                name,
                &bytes,
                ReadSession::ID,
                ReadSession::RESPONSE_LIMITS,
            );
        } else {
            check::<LogoutResult>(name, &bytes, Logout::ID, Logout::RESPONSE_LIMITS);
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
        decode(&frame, CredentialEnvelope::ID, false, limits).unwrap();
    let PortableProof::Certified(proof) = &mut value.proof else {
        unreachable!()
    };
    proof.issuer_signature.signature_cbor = current::Bytes::new(vec![0; 131072]).unwrap();
    let frame = encode(&value, CredentialEnvelope::ID, false, limits).unwrap();
    assert_eq!(CredentialEnvelope::encode_request(&value).unwrap(), frame);
    assert_eq!(CredentialEnvelope::decode_request(&frame).unwrap(), value);
    value.claims = current::Bytes::new(vec![0; 65536]).unwrap();
    assert!(encode(&value, CredentialEnvelope::ID, false, limits).is_err());
    assert_eq!(
        CredentialEnvelope::encode_request(&value),
        Err(current::Error::Limit(current::LimitKind::Allocation))
    );
    // Produce only test input with a larger allowance, then compare refusals at
    // the unchanged production allowance. No product cap is changed.
    let larger = Limits {
        allocation: limits.allocation * 2,
        ..limits
    };
    let frame = encode(&value, CredentialEnvelope::ID, false, larger).unwrap();
    assert!(decode::<PortableCredential>(&frame, CredentialEnvelope::ID, false, limits).is_err());
    assert_eq!(
        CredentialEnvelope::decode_request(&frame),
        Err(current::Error::Limit(current::LimitKind::Allocation))
    );
}
