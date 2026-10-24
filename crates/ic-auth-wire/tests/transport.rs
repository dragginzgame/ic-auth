use ic_auth_protocol_types as domain;
use ic_auth_wire::{CredentialEnvelope, EstablishSession, Logout, ReadSession, model, wire};
use wire::{Bytes, Text};

fn credential() -> domain::PortableCredential {
    domain::PortableCredential {
        claims: vec![0, 128, 255],
        proof: domain::PortableProof::Ed25519 {
            signature: (0..64).collect(),
        },
    }
}

#[test]
fn signed_bytes_roundtrip_without_intermediate_encoding_or_authority() {
    let original = credential();
    let value: model::PortableCredential = original.clone().try_into().unwrap();
    let frame = CredentialEnvelope::encode_request(&value).unwrap();
    assert_eq!(&frame.as_slice()[..2], b"IW");
    let decoded = CredentialEnvelope::decode_request(frame.as_slice()).unwrap();
    assert_eq!(
        domain::PortableCredential::try_from(decoded).unwrap(),
        original
    );
    // Envelope admission does not authenticate opaque claims or a zero signature.
    let unsigned = domain::PortableCredential {
        claims: vec![],
        proof: domain::PortableProof::Ed25519 {
            signature: vec![0; 64],
        },
    };
    let value: model::PortableCredential = unsigned.clone().try_into().unwrap();
    assert_eq!(
        domain::PortableCredential::try_from(value).unwrap(),
        unsigned
    );
}

#[test]
fn single_pass_decode_preserves_scope_order_and_rejects_nonminimal_lengths() {
    let value: model::PortableCredential = credential().try_into().unwrap();
    let frame = CredentialEnvelope::encode_request(&value).unwrap();
    assert_eq!(frame[11], 3); // Opaque claims byte length, after the frame header.
    let mut overlong = frame[..11].to_vec();
    overlong.extend_from_slice(&[0x83, 0]);
    overlong.extend_from_slice(&frame[12..]);
    assert_eq!(
        CredentialEnvelope::decode_request(&overlong),
        Err(wire::Error::Malformed)
    );

    // Transport must not sort or deduplicate scopes. The session engine owns
    // semantic normalization/rejection and request identity after this boundary.
    let request = ic_auth_wire::SessionInput {
        credential: value,
        scopes: wire::List::new(
            ["write", "read", "write"]
                .into_iter()
                .map(|scope| Text::new(scope.into()).unwrap())
                .collect(),
        )
        .unwrap(),
        ttl_secs: Some(30),
    };
    let bytes = EstablishSession::encode_request(&request).unwrap();
    let decoded = EstablishSession::decode_request(&bytes).unwrap();
    assert_eq!(decoded, request);
    assert_eq!(EstablishSession::encode_request(&decoded).unwrap(), bytes);
}

#[test]
fn frame_identity_direction_exact_consumption_and_bounds_are_independent() {
    let value: model::PortableCredential = credential().try_into().unwrap();
    let encoded = CredentialEnvelope::encode_request(&value).unwrap();
    for end in 0..encoded.len() {
        assert!(CredentialEnvelope::decode_request(&encoded[..end]).is_err());
    }
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(CredentialEnvelope::decode_request(&trailing).is_err());
    assert!(EstablishSession::decode_request(&encoded).is_err());
    let wrong_direction = CredentialEnvelope::encode_response(&value).unwrap();
    assert!(CredentialEnvelope::decode_request(wrong_direction.as_slice()).is_err());
    let mut too_large = credential();
    too_large.claims = vec![0; 65537];
    assert!(model::PortableCredential::try_from(too_large).is_err());
    let mut too_large = credential();
    too_large.proof = domain::PortableProof::Ed25519 {
        signature: vec![0; 65],
    };
    assert!(model::PortableCredential::try_from(too_large).is_err());
    let mut maximum = credential();
    maximum.claims = vec![255; 65536];
    let value: model::PortableCredential = maximum.clone().try_into().unwrap();
    let bytes = CredentialEnvelope::encode_request(&value).unwrap();
    assert_eq!(
        domain::PortableCredential::try_from(
            CredentialEnvelope::decode_request(bytes.as_slice()).unwrap()
        )
        .unwrap(),
        maximum
    );
}

#[test]
fn checked_role_projection_rejects_invalid_text_instead_of_coercing_it() {
    let valid = domain::DelegatedRoleGrant {
        target: "project_instance".parse().unwrap(),
        scopes: vec!["read".into()],
    };
    let projected: model::DelegatedRoleGrant = valid.clone().try_into().unwrap();
    assert_eq!(
        domain::DelegatedRoleGrant::try_from(projected).unwrap(),
        valid
    );
    let invalid = model::DelegatedRoleGrant {
        target: Text::new("INVALID ROLE".into()).unwrap(),
        scopes: wire::List::new(vec![]).unwrap(),
    };
    assert_eq!(
        domain::DelegatedRoleGrant::try_from(invalid),
        Err(wire::Error::Malformed)
    );
}

#[test]
fn caller_only_requests_and_all_logout_outcomes_have_bounded_distinct_frames() {
    let request = Logout::encode_request(&()).unwrap();
    assert_eq!(request.as_slice().len(), 11);
    assert!(ReadSession::decode_request(request.as_slice()).is_err());
    for value in [
        model::LogoutResult::Revoked(u64::MAX),
        model::LogoutResult::InvalidCaller,
        model::LogoutResult::Unavailable(model::LogoutUnavailable::Capacity),
        model::LogoutResult::Unavailable(model::LogoutUnavailable::Storage),
        model::LogoutResult::Unavailable(model::LogoutUnavailable::Clock),
        model::LogoutResult::Unavailable(model::LogoutUnavailable::ResetPending),
    ] {
        let bytes = Logout::encode_response(&value).unwrap();
        assert_eq!(Logout::decode_response(bytes.as_slice()).unwrap(), value);
        assert!(Logout::decode_request(bytes.as_slice()).is_err());
    }
    let view = ic_auth_wire::SessionView {
        caller: Bytes::new(vec![1; 29]).unwrap(),
        issuer: Bytes::new(vec![2]).unwrap(),
        subject: Bytes::new(vec![3]).unwrap(),
        expires_at_ns: u64::MAX,
    };
    for response in [Ok(view), Err(model::AdmissionRefusal::Capacity)] {
        let bytes = EstablishSession::encode_response(&response).unwrap();
        assert_eq!(
            EstablishSession::decode_response(bytes.as_slice()).unwrap(),
            response
        );
    }
}

#[test]
fn admission_refusals_are_typed_distinct_and_reject_retired_text() {
    use model::AdmissionRefusal as R;
    let mut frames = std::collections::BTreeSet::new();
    for refusal in [
        R::NotYetValid,
        R::Expired,
        R::InvalidCredential,
        R::AuthorityUnavailable,
        R::BeforeReset,
        R::ScopeRejected,
        R::ReplayConflict,
        R::Revoked,
        R::Capacity,
        R::Throttled,
        R::Unavailable,
    ] {
        let result = Err(refusal);
        let encoded = EstablishSession::encode_response(&result).unwrap();
        assert_eq!(
            EstablishSession::decode_response(encoded.as_slice()).unwrap(),
            result
        );
        assert!(frames.insert(encoded));
    }
    let encoded = EstablishSession::encode_response(&Err(R::NotYetValid)).unwrap();
    // Retired text replies used a length followed by UTF-8 bytes. In particular,
    // empty text (zero) must not accidentally become the future-start hint.
    for message in ["", "capacity", "portable proof is not yet valid"] {
        let mut retired = encoded.as_slice()[..encoded.as_slice().len() - 1].to_vec();
        retired[2..10].copy_from_slice(&0x415554480002_u64.to_le_bytes());
        retired.push(u8::try_from(message.len()).unwrap());
        retired.extend_from_slice(message.as_bytes());
        assert!(EstablishSession::decode_response(&retired).is_err());
    }
}

#[test]
fn released_010_frames_survive_the_wire_05_rust_api_cut() {
    // Captured from released 0.10.7 with IC Wire 0.4.9, not generated by the
    // selected codec. The Rust bound change must not alter method/signed bytes.
    let decode = |hex: &str| -> Vec<u8> {
        let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
        assert!(remainder.is_empty());
        pairs
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    };
    let credential_frame = decode(
        "4957010048545541000000030080ff0140000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
    );
    let decoded = CredentialEnvelope::decode_request(&credential_frame).unwrap();
    assert_eq!(
        domain::PortableCredential::try_from(decoded.clone()).unwrap(),
        credential()
    );
    assert_eq!(
        CredentialEnvelope::encode_request(&decoded)
            .unwrap()
            .as_slice(),
        credential_frame
    );
    let logout_frame = decode("4957040048545541000000");
    Logout::decode_request(&logout_frame).unwrap();
    assert_eq!(
        Logout::encode_request(&()).unwrap().as_slice(),
        logout_frame
    );
    let read_frame = decode("4957030048545541000000");
    ReadSession::decode_request(&read_frame).unwrap();
    assert_eq!(
        ReadSession::encode_request(&()).unwrap().as_slice(),
        read_frame
    );
}

#[test]
fn retired_admission_identity_cannot_reinterpret_refusal_tags() {
    let value: Result<ic_auth_wire::SessionView, model::AdmissionRefusal> =
        Err(model::AdmissionRefusal::NotYetValid);
    let frame = EstablishSession::encode_response(&value).unwrap();
    assert_eq!(&frame[2..10], &0x415554480012_u64.to_le_bytes());
    assert_eq!(&frame[11..], &[1, 0]); // Serde Result::Err, then NotYetValid.
    let mut retired = frame.clone();
    retired[2..10].copy_from_slice(&0x415554480002_u64.to_le_bytes());
    retired[12] = 1; // The former NotYetValid tag now means Expired.
    assert_eq!(
        EstablishSession::decode_response(&retired),
        Err(wire::Error::Identity)
    );
    assert_eq!(EstablishSession::decode_response(&frame).unwrap(), value);
}

#[test]
fn native_traversal_budgets_refuse_without_relaxing_field_bounds() {
    let value: model::PortableCredential = credential().try_into().unwrap();
    let frame = CredentialEnvelope::encode_request(&value).unwrap();
    let baseline = CredentialEnvelope::REQUEST_LIMITS;
    for limits in [
        wire::Limits {
            bytes: frame.len() - 1,
            ..baseline
        },
        wire::Limits {
            allocation: baseline.bytes,
            ..baseline
        },
        wire::Limits {
            nodes: 1,
            ..baseline
        },
        wire::Limits {
            depth: 1,
            ..baseline
        },
    ] {
        assert_eq!(
            wire::native::encode_request(CredentialEnvelope::ID, &value, limits),
            Err(wire::Error::Limit)
        );
        assert_eq!(
            wire::native::decode_request::<model::PortableCredential>(
                CredentialEnvelope::ID,
                &frame,
                limits
            ),
            Err(wire::Error::Limit)
        );
    }
    assert_eq!(CredentialEnvelope::decode_request(&frame).unwrap(), value);
}
