use candid::{CandidType, decode_one, encode_one};
use ic_auth_protocol_types::*;
use serde::Deserialize;

mod proofs;

// Independent description of Canic's current Candid boundary: identifiers and
// roles are text, despite being validated value types inside the library.
#[derive(CandidType, Clone, Debug, Deserialize, PartialEq)]
enum SourceAudience {
    Fleet(SourceKey),
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq)]
struct SourceKey {
    canonical_network_id: String,
    fleet_id: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq)]
struct SourceGrant {
    target: String,
    scopes: Vec<String>,
}

// Independent wire fixtures reviewed against Canic c4c046f947b2b28f4342cbf6efe9221ba1ed5f70,
// crates/canic-core/src/dto/auth/{common,token}.rs. These describe transport,
// not proof verification or a production Canic adapter.
#[derive(CandidType, Clone, Debug, Deserialize, PartialEq)]
struct SourceMetadata {
    request_id: [u8; 32],
    ttl_ns: u64,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourcePrepareRequest {
    metadata: Option<SourceMetadata>,
    aud: SourceAudience,
    grants: Vec<SourceGrant>,
    ttl_ns: u64,
    ext: Option<Vec<u8>>,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceClaims {
    presenter: Principal,
    subject: Principal,
    issuer_pid: Principal,
    cert_hash: [u8; 32],
    issued_at_ns: u64,
    expires_at_ns: u64,
    aud: SourceAudience,
    grants: Vec<SourceGrant>,
    nonce: [u8; 16],
    ext: Option<Vec<u8>>,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourcePrepareResponse {
    claims: SourceClaims,
    claims_hash: [u8; 32],
    retrieval_expires_at_ns: u64,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceGetRequest {
    claims_hash: [u8; 32],
}

fn source_audience() -> SourceAudience {
    SourceAudience::Fleet(SourceKey {
        canonical_network_id: "ab".repeat(32),
        fleet_id: "cd".repeat(32),
    })
}

fn source_grants() -> Vec<SourceGrant> {
    vec![SourceGrant {
        target: "project_instance".into(),
        scopes: vec!["app:read".into(), "app:write".into()],
    }]
}

fn source_claims(ext: Option<Vec<u8>>) -> SourceClaims {
    SourceClaims {
        presenter: Principal::from_slice(&[1; 29]),
        subject: Principal::from_slice(&[1; 29]),
        issuer_pid: Principal::from_slice(&[2; 29]),
        cert_hash: [0xff; 32],
        issued_at_ns: u64::MAX - 100,
        expires_at_ns: u64::MAX,
        aud: source_audience(),
        grants: source_grants(),
        nonce: [0x80; 16],
        ext,
    }
}

#[test]
fn prepare_request_preserves_complete_canic_envelope_in_both_directions() {
    for metadata in [
        None,
        Some(SourceMetadata {
            request_id: [0x80; 32],
            ttl_ns: u64::MAX,
        }),
    ] {
        for ext in [None, Some(vec![]), Some(vec![0, 1, 0xff])] {
            let source = SourcePrepareRequest {
                metadata: metadata.clone(),
                aud: source_audience(),
                grants: source_grants(),
                ttl_ns: u64::MAX,
                ext,
            };
            let source_bytes = encode_one(&source).unwrap();
            let checked: DelegatedTokenPrepareRequest = decode_one(&source_bytes).unwrap();
            assert_eq!(checked.ttl_ns, source.ttl_ns);
            assert_eq!(checked.ext, source.ext);
            assert_eq!(
                checked.metadata.map(|m| (m.request_id, m.ttl_ns)),
                source.metadata.as_ref().map(|m| (m.request_id, m.ttl_ns))
            );
            let checked_bytes = encode_one(&checked).unwrap();
            assert_eq!(checked_bytes, source_bytes);
            assert_eq!(
                decode_one::<SourcePrepareRequest>(&checked_bytes).unwrap(),
                source
            );
        }
    }
}

#[test]
fn claims_and_prepare_response_preserve_canic_envelopes_in_both_directions() {
    for ext in [None, Some(vec![]), Some(vec![0, 1, 0xff])] {
        let source = SourcePrepareResponse {
            claims: source_claims(ext),
            claims_hash: [0x80; 32],
            retrieval_expires_at_ns: u64::MAX,
        };
        let source_bytes = encode_one(&source).unwrap();
        let checked: DelegatedTokenPrepareResponse = decode_one(&source_bytes).unwrap();
        let checked_bytes = encode_one(&checked).unwrap();
        assert_eq!(checked_bytes, source_bytes);
        assert_eq!(
            decode_one::<SourcePrepareResponse>(&checked_bytes).unwrap(),
            source
        );

        let claims_bytes = encode_one(&source.claims).unwrap();
        let claims: DelegatedTokenClaims = decode_one(&claims_bytes).unwrap();
        let checked_claims_bytes = encode_one(&claims).unwrap();
        assert_eq!(checked_claims_bytes, claims_bytes);
        assert_eq!(
            decode_one::<SourceClaims>(&checked_claims_bytes).unwrap(),
            source.claims
        );
    }
}

#[test]
fn retrieval_request_preserves_canic_hash_in_both_directions() {
    for claims_hash in [[0; 32], [0x80; 32], [0xff; 32]] {
        let source = SourceGetRequest { claims_hash };
        let source_bytes = encode_one(&source).unwrap();
        let checked: DelegatedTokenGetRequest = decode_one(&source_bytes).unwrap();
        assert_eq!(checked.claims_hash, claims_hash);
        let checked_bytes = encode_one(&checked).unwrap();
        assert_eq!(checked_bytes, source_bytes);
        assert_eq!(
            decode_one::<SourceGetRequest>(&checked_bytes).unwrap(),
            source
        );
    }
}

#[test]
fn nested_envelopes_cannot_bypass_role_or_audience_validation() {
    let mut request = SourcePrepareRequest {
        metadata: None,
        aud: source_audience(),
        grants: source_grants(),
        ttl_ns: 100,
        ext: None,
    };
    request.grants[0].target = "Root".into();
    assert!(decode_one::<DelegatedTokenPrepareRequest>(&encode_one(&request).unwrap()).is_err());
    request.grants = source_grants();
    let SourceAudience::Fleet(key) = &mut request.aud;
    key.canonical_network_id = "AB".repeat(32);
    assert!(decode_one::<DelegatedTokenPrepareRequest>(&encode_one(&request).unwrap()).is_err());

    let mut response = SourcePrepareResponse {
        claims: source_claims(None),
        claims_hash: [1; 32],
        retrieval_expires_at_ns: 100,
    };
    response.claims.grants[0].target = "app/read".into();
    assert!(decode_one::<DelegatedTokenPrepareResponse>(&encode_one(&response).unwrap()).is_err());
    response.claims.grants = source_grants();
    let SourceAudience::Fleet(key) = &mut response.claims.aud;
    key.fleet_id = "cd".into();
    assert!(decode_one::<DelegatedTokenPrepareResponse>(&encode_one(&response).unwrap()).is_err());
}

#[test]
fn checked_audience_preserves_existing_candid_bytes() {
    let source = SourceAudience::Fleet(SourceKey {
        canonical_network_id: "ab".repeat(32),
        fleet_id: "cd".repeat(32),
    });
    let bytes = encode_one(&source).unwrap();
    let projected: DelegationAudience = decode_one(&bytes).unwrap();
    assert_eq!(encode_one(&projected).unwrap(), bytes);
    assert_eq!(
        decode_one::<SourceAudience>(&encode_one(projected).unwrap()).unwrap(),
        source
    );
}

#[test]
fn checked_role_preserves_existing_candid_bytes() {
    let source = SourceGrant {
        target: "project_instance".into(),
        scopes: vec!["app:read".into()],
    };
    let bytes = encode_one(&source).unwrap();
    let grant: DelegatedRoleGrant = decode_one(&bytes).unwrap();
    assert_eq!(grant.target.as_str(), "project_instance");
    assert_eq!(encode_one(grant).unwrap(), bytes);
}

#[test]
fn decoding_cannot_bypass_identifier_validation() {
    for bad in ["", "ab", &"AB".repeat(32), &"g0".repeat(32)] {
        assert_eq!(
            bad.parse::<CanonicalId>(),
            Err(IdentifierError::NonCanonicalId)
        );
        assert!(decode_one::<CanonicalId>(&encode_one(bad).unwrap()).is_err());
    }
    for bad in ["", "Root", "app/read", "é", "read "] {
        assert_eq!(bad.parse::<AuthRole>(), Err(IdentifierError::InvalidRole));
        assert!(decode_one::<AuthRole>(&encode_one(bad).unwrap()).is_err());
    }
}

#[test]
fn identities_keep_network_and_audience_distinct() {
    let original = AudienceId {
        canonical_network_id: CanonicalId::from_bytes([1; 32]),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    };
    assert_ne!(
        original,
        AudienceId {
            canonical_network_id: CanonicalId::from_bytes([3; 32]),
            ..original
        }
    );
    assert_ne!(
        original,
        AudienceId {
            fleet_id: CanonicalId::from_bytes([3; 32]),
            ..original
        }
    );
    assert_eq!(
        original,
        decode_one::<AudienceId>(&encode_one(original).unwrap()).unwrap()
    );
}
