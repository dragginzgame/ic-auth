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
    let frame = wire::encode_request::<CredentialEnvelope>(&value).unwrap();
    assert_eq!(&frame.as_bytes()[..2], b"IW");
    let decoded = wire::decode_request::<CredentialEnvelope>(frame.as_bytes()).unwrap();
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
fn frame_identity_direction_exact_consumption_and_bounds_are_independent() {
    let value: model::PortableCredential = credential().try_into().unwrap();
    let encoded = wire::encode_request::<CredentialEnvelope>(&value)
        .unwrap()
        .into_bytes();
    for end in 0..encoded.len() {
        assert!(wire::decode_request::<CredentialEnvelope>(&encoded[..end]).is_err());
    }
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(wire::decode_request::<CredentialEnvelope>(&trailing).is_err());
    assert!(wire::decode_request::<EstablishSession>(&encoded).is_err());
    let wrong_direction = wire::encode_response::<CredentialEnvelope>(&value).unwrap();
    assert!(wire::decode_request::<CredentialEnvelope>(wrong_direction.as_bytes()).is_err());
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
    let bytes = wire::encode_request::<CredentialEnvelope>(&value).unwrap();
    assert_eq!(
        domain::PortableCredential::try_from(
            wire::decode_request::<CredentialEnvelope>(bytes.as_bytes()).unwrap()
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
    let request = wire::encode_request::<Logout>(&()).unwrap();
    assert_eq!(request.as_bytes().len(), 11);
    assert!(wire::decode_request::<ReadSession>(request.as_bytes()).is_err());
    for value in [
        model::LogoutResult::Revoked(u64::MAX),
        model::LogoutResult::InvalidCaller,
        model::LogoutResult::Unavailable(model::LogoutUnavailable::Capacity),
        model::LogoutResult::Unavailable(model::LogoutUnavailable::Storage),
        model::LogoutResult::Unavailable(model::LogoutUnavailable::Clock),
        model::LogoutResult::Unavailable(model::LogoutUnavailable::ResetPending),
    ] {
        let bytes = wire::encode_response::<Logout>(&value).unwrap();
        assert_eq!(
            wire::decode_response::<Logout>(bytes.as_bytes()).unwrap(),
            value
        );
        assert!(wire::decode_request::<Logout>(bytes.as_bytes()).is_err());
    }
    let view = ic_auth_wire::SessionView {
        caller: Bytes::new(vec![1; 29]).unwrap(),
        issuer: Bytes::new(vec![2]).unwrap(),
        subject: Bytes::new(vec![3]).unwrap(),
        expires_at_ns: u64::MAX,
    };
    for response in [Ok(view), Err(Text::new("capacity".into()).unwrap())] {
        let bytes = wire::encode_response::<EstablishSession>(&response).unwrap();
        assert_eq!(
            wire::decode_response::<EstablishSession>(bytes.as_bytes()).unwrap(),
            response
        );
    }
}
