//! Current frames are independently assembled; historical provenance is retained.

use ic_auth_protocol_types::Principal;
use ic_auth_wire::{
    BROWSER_LOGOUT_RESPONSE_BYTES, Logout, ReadSession, SessionView,
    model::{LogoutResult, LogoutUnavailable, SessionReadRefusal},
    wire::{self, Direction, Encoding, Error, Limits},
};

fn fixture(name: &str) -> Vec<u8> {
    let (_, hex) = include_str!("fixtures/caller-frames.tsv")
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .find(|(key, _)| *key == name)
        .expect("named independent fixture");
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn maximum_view() -> SessionView {
    SessionView {
        ext: Some(ic_auth_wire::wire::Bytes::new(vec![0xab; 16]).unwrap()),
        caller: Principal::from_slice(&[1; 29]),
        issuer: Principal::from_slice(&[2]),
        subject: Principal::from_slice(&[3]),
        expires_at_ns: u64::MAX,
    }
}

fn read_cases() -> Vec<(&'static str, Result<SessionView, SessionReadRefusal>)> {
    vec![
        (
            "read-empty",
            Ok(SessionView {
                ext: None,
                caller: Principal::management_canister(),
                issuer: Principal::management_canister(),
                subject: Principal::management_canister(),
                expires_at_ns: 0,
            }),
        ),
        ("read-max", Ok(maximum_view())),
        (
            "read-MissingOrExpired",
            Err(SessionReadRefusal::MissingOrExpired),
        ),
        (
            "read-AuthorityUnavailable",
            Err(SessionReadRefusal::AuthorityUnavailable),
        ),
        ("read-ScopeRejected", Err(SessionReadRefusal::ScopeRejected)),
        ("read-Revoked", Err(SessionReadRefusal::Revoked)),
        ("read-Unavailable", Err(SessionReadRefusal::Unavailable)),
    ]
}

fn logout_cases() -> Vec<(&'static str, LogoutResult)> {
    vec![
        ("logout-Revoked-zero", LogoutResult::Revoked(0)),
        ("logout-Revoked-max", LogoutResult::Revoked(u64::MAX)),
        ("logout-Unconfirmed-zero", LogoutResult::Unconfirmed(0)),
        (
            "logout-Unconfirmed-max",
            LogoutResult::Unconfirmed(u64::MAX),
        ),
        ("logout-InvalidCaller", LogoutResult::InvalidCaller),
        (
            "logout-Capacity",
            LogoutResult::Unavailable(LogoutUnavailable::Capacity),
        ),
        (
            "logout-Storage",
            LogoutResult::Unavailable(LogoutUnavailable::Storage),
        ),
        (
            "logout-Clock",
            LogoutResult::Unavailable(LogoutUnavailable::Clock),
        ),
        (
            "logout-ResetPending",
            LogoutResult::Unavailable(LogoutUnavailable::ResetPending),
        ),
    ]
}

#[test]
fn current_fixture_frames_cross_encode_and_decode() {
    assert_eq!(
        ReadSession::encode_request(&()).unwrap(),
        fixture("read-request")
    );
    assert_eq!(
        Logout::encode_request(&()).unwrap(),
        fixture("logout-request")
    );
    ReadSession::decode_request(&fixture("read-request")).unwrap();
    Logout::decode_request(&fixture("logout-request")).unwrap();
    for (name, expected) in read_cases() {
        let bytes = fixture(name);
        assert_eq!(
            ReadSession::decode_response(&bytes).unwrap(),
            expected,
            "{name}"
        );
        assert_eq!(
            ReadSession::encode_response(&expected).unwrap(),
            bytes,
            "{name}"
        );
    }
    for (name, expected) in logout_cases() {
        let bytes = fixture(name);
        assert_eq!(Logout::decode_response(&bytes).unwrap(), expected, "{name}");
        assert_eq!(Logout::encode_response(&expected).unwrap(), bytes, "{name}");
        assert!(bytes.len() <= BROWSER_LOGOUT_RESPONSE_BYTES);
    }
}

#[test]
fn identities_and_all_four_allowances_are_frozen() {
    assert_eq!(ReadSession::ID, 0x415554480013);
    assert_eq!(Logout::ID, 0x415554480014);
    assert_eq!(ReadSession::NAME, "portable_session_read");
    assert_eq!(Logout::NAME, "logout_portable_session");
    for limits in [ReadSession::REQUEST_LIMITS, Logout::REQUEST_LIMITS] {
        assert_eq!(
            (limits.bytes, limits.allocation, limits.nodes, limits.depth),
            (11, 128, 8, 4)
        );
    }
    for (limits, expected) in [
        (ReadSession::RESPONSE_LIMITS, (8192, 65536, 128, 8)),
        (Logout::RESPONSE_LIMITS, (4096, 16384, 128, 8)),
    ] {
        assert_eq!(
            (limits.bytes, limits.allocation, limits.nodes, limits.depth),
            expected
        );
    }
    assert_eq!(BROWSER_LOGOUT_RESPONSE_BYTES, 1024);
}

#[test]
fn caller_identity_cannot_be_submitted_in_requests() {
    for (name, decode) in [
        (
            "read-request",
            ReadSession::decode_request as fn(&[u8]) -> Result<(), Error>,
        ),
        ("logout-request", Logout::decode_request),
    ] {
        let bytes = fixture(name);
        for end in 0..bytes.len() {
            assert!(decode(&bytes[..end]).is_err());
        }
        for offset in [0, 1, 2, 9, 10] {
            let mut bad = bytes.clone();
            bad[offset] ^= 1;
            assert!(decode(&bad).is_err());
        }
        let mut submitted_caller = bytes;
        submitted_caller.extend_from_slice(&[1, 2]);
        assert!(decode(&submitted_caller).is_err());
    }
    assert!(ReadSession::decode_request(&fixture("logout-request")).is_err());
    assert!(Logout::decode_request(&fixture("read-request")).is_err());
}

#[test]
fn malformed_or_misrouted_replies_refuse() {
    for (name, _) in read_cases() {
        let mut bytes = fixture(name);
        for end in 0..bytes.len() {
            assert!(
                ReadSession::decode_response(&bytes[..end]).is_err(),
                "{name}:{end}"
            );
        }
        assert!(Logout::decode_response(&bytes).is_err());
        bytes.push(0);
        assert!(ReadSession::decode_response(&bytes).is_err());
    }
    for (name, _) in logout_cases() {
        let mut bytes = fixture(name);
        for end in 0..bytes.len() {
            assert!(
                Logout::decode_response(&bytes[..end]).is_err(),
                "{name}:{end}"
            );
        }
        assert!(ReadSession::decode_response(&bytes).is_err());
        bytes.push(0);
        assert!(Logout::decode_response(&bytes).is_err());
    }
    let mut bad_tag = fixture("logout-InvalidCaller");
    bad_tag[11] = 4;
    assert!(Logout::decode_response(&bad_tag).is_err());
    let mut bad_reason = fixture("logout-Capacity");
    bad_reason[12] = 4;
    assert!(Logout::decode_response(&bad_reason).is_err());
    let mut bad_read = fixture("read-Unavailable");
    bad_read[12] = 5;
    assert!(ReadSession::decode_response(&bad_read).is_err());
    let mut long_principal = fixture("read-max");
    long_principal[12] = 30;
    long_principal.insert(13, 1);
    assert!(ReadSession::decode_response(&long_principal).is_err());
    let mut bad_integer = fixture("logout-Revoked-max");
    *bad_integer.last_mut().unwrap() = 2;
    assert!(Logout::decode_response(&bad_integer).is_err());
    let mut oversized = fixture("read-empty");
    oversized.resize(ReadSession::RESPONSE_LIMITS.bytes + 1, 0);
    assert_eq!(ReadSession::decode_response(&oversized), Err(Error::Limit));
}

#[test]
fn outer_varints_are_not_signed_claims_canonicalization() {
    // Reference Postcard accepts a redundant zero group on outer enum tags.
    let mut bytes = fixture("logout-InvalidCaller");
    bytes.splice(11..12, [0x82, 0]);
    assert_eq!(
        Logout::decode_response(&bytes).unwrap(),
        LogoutResult::InvalidCaller
    );
}

#[test]
fn every_resource_allowance_is_enforced_independently() {
    let value: Result<SessionView, SessionReadRefusal> = Ok(maximum_view());
    let bytes = fixture("read-max");
    let encoding = Encoding::Frame {
        method: ReadSession::ID,
        direction: Direction::Response,
    };
    let full = ReadSession::RESPONSE_LIMITS;
    // All other allowances remain sufficient while one is exhausted.
    for limits in [
        Limits {
            bytes: bytes.len() - 1,
            ..full
        },
        Limits {
            allocation: 0,
            ..full
        },
        Limits { nodes: 0, ..full },
        Limits { depth: 0, ..full },
    ] {
        assert_eq!(wire::encode(&value, encoding, limits), Err(Error::Limit));
        assert_eq!(
            wire::decode::<Result<SessionView, SessionReadRefusal>>(&bytes, encoding, limits),
            Err(Error::Limit)
        );
    }
    for size in [bytes.len(), bytes.len() + 1] {
        let limits = Limits {
            bytes: size,
            ..full
        };
        assert_eq!(wire::encode(&value, encoding, limits).unwrap(), bytes);
        assert_eq!(
            wire::decode::<Result<SessionView, SessionReadRefusal>>(&bytes, encoding, limits)
                .unwrap(),
            value
        );
    }
}

#[test]
fn both_session_replies_transport_a_full_application_extension() {
    use ic_auth_wire::{EstablishSession, wire::Bytes};
    let mut view = maximum_view();
    view.ext = Some(Bytes::new(vec![0xff; 4096]).unwrap());
    let read = ReadSession::encode_response(&Ok(view.clone())).unwrap();
    let admitted = EstablishSession::encode_response(&Ok(view.clone())).unwrap();
    assert_eq!(
        ReadSession::decode_response(&read).unwrap(),
        Ok(view.clone())
    );
    assert_eq!(
        EstablishSession::decode_response(&admitted).unwrap(),
        Ok(view)
    );
}
