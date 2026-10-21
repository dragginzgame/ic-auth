use ic_auth::portable::{
    MAX_CLAIMS_BYTES, PortableEncodingError as Error, browser_principal, claims_hash,
    decode_claims, encode_claims,
};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};

// Independent protocol vector, assembled with Python struct.pack('>I' / '>Q')
// and hashlib, rather than the Rust encoder. Deliberately asymmetric timestamps
// exercise byte order; shape codecs do not enforce time/authority validity.
const VECTOR: &str = concat!(
    "49432d415554482d504f525441424c452d43524544454e5449414c00000101",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "2222222222222222222222222222222222222222222222222222222222222222",
    "0000000101",
    "3333333333333333333333333333333333333333333333333333333333333333",
    "4444444444444444444444444444444444444444444444444444444444444444",
    "0000000102",
    "0102030405060708111213141516171821222324252627283132333435363738",
    "55555555555555555555555555555555",
    "000000010000000475736572000000010000000472656164",
);

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn claims() -> PortableClaims {
    PortableClaims {
        mechanism: PortableMechanism::Ed25519,
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([0x11; 32]),
            fleet_id: CanonicalId::from_bytes([0x22; 32]),
        },
        issuer: Principal::from_slice(&[1]),
        key_binding_id: [0x33; 32],
        browser_public_key: [0x44; 32],
        account_subject: Principal::from_slice(&[2]),
        login_started_at_ns: 0x0102030405060708,
        login_deadline_ns: 0x1112131415161718,
        issued_at_ns: 0x2122232425262728,
        admission_expires_at_ns: 0x3132333435363738,
        nonce: [0x55; 16],
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".into()],
        }],
    }
}

#[test]
fn frozen_bytes_digest_and_direct_browser_principal() {
    let value = claims();
    let bytes = hex(VECTOR);
    assert_eq!(encode_claims(&value).unwrap(), bytes);
    assert_eq!(decode_claims(&bytes).unwrap(), value);
    assert_eq!(
        claims_hash(&value).unwrap().as_slice(),
        hex("3b70b4402a7318590764ab5cd2a9a2d62e4ae754cc62e2a897d67850ab1f7b76")
    );
    assert_eq!(
        browser_principal(&value.browser_public_key).as_slice(),
        hex("a95a917c9745c3c1fdf4d6459a6b8356ce5e86e0ee9ac0da7789741a02")
    );
    assert_ne!(
        browser_principal(&value.browser_public_key),
        value.account_subject
    );
    assert_ne!(
        browser_principal(&value.browser_public_key),
        browser_principal(&[0x45; 32])
    );
}

#[test]
fn every_semantic_field_is_bound_and_mechanisms_have_distinct_bytes() {
    let original = claims();
    let digest = claims_hash(&original).unwrap();
    let changes: &[fn(&mut PortableClaims)] = &[
        |c| c.mechanism = PortableMechanism::Certified,
        |c| c.audience.canonical_network_id = CanonicalId::from_bytes([0; 32]),
        |c| c.audience.fleet_id = CanonicalId::from_bytes([0; 32]),
        |c| c.issuer = Principal::from_slice(&[3]),
        |c| c.key_binding_id[0] ^= 1,
        |c| c.browser_public_key[0] ^= 1,
        |c| c.account_subject = Principal::from_slice(&[3]),
        |c| c.login_started_at_ns += 1,
        |c| c.login_deadline_ns += 1,
        |c| c.issued_at_ns += 1,
        |c| c.admission_expires_at_ns += 1,
        |c| c.nonce[0] ^= 1,
        |c| c.grants[0].target = "admin".parse().unwrap(),
        |c| c.grants[0].scopes[0] = "write".into(),
    ];
    for change in changes {
        let mut value = original.clone();
        change(&mut value);
        assert_ne!(claims_hash(&value).unwrap(), digest);
        let bytes = encode_claims(&value).unwrap();
        assert_eq!(decode_claims(&bytes).unwrap(), value);
    }
    let mut certified = original;
    certified.mechanism = PortableMechanism::Certified;
    let mut expected = hex(VECTOR);
    expected[30] = 0;
    assert_eq!(encode_claims(&certified).unwrap(), expected);
}

#[test]
fn malformed_lengths_domains_and_trailing_data_refuse() {
    let bytes = hex(VECTOR);
    for end in 0..bytes.len() {
        assert!(
            decode_claims(&bytes[..end]).is_err(),
            "accepted prefix {end}"
        );
    }
    for offset in [0, 27, 28, 29, 30] {
        let mut invalid = bytes.clone();
        invalid[offset] = 0xff;
        assert_eq!(decode_claims(&invalid), Err(Error::WrongDomain));
    }
    // Principal lengths, grant count, role length, scope count and scope length.
    for offset in [95, 164, 217, 221, 229, 233] {
        let mut invalid = bytes.clone();
        invalid[offset..offset + 4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(decode_claims(&invalid), Err(Error::InvalidShape));
    }
    for offset in [217, 221, 229, 233] {
        let mut invalid = bytes.clone();
        invalid[offset..offset + 4].fill(0);
        assert_eq!(decode_claims(&invalid), Err(Error::InvalidShape));
    }
    let mut invalid = bytes.clone();
    invalid.push(0);
    assert_eq!(decode_claims(&invalid), Err(Error::TrailingBytes));
    assert_eq!(
        decode_claims(&vec![0; MAX_CLAIMS_BYTES + 1]),
        Err(Error::TooLarge)
    );
    let mut anonymous = bytes.clone();
    anonymous[168] = 4;
    assert_eq!(decode_claims(&anonymous), Err(Error::AnonymousSubject));
    for offset in [225, 237] {
        let mut invalid = bytes.clone();
        invalid[offset] = 0xff;
        assert_eq!(decode_claims(&invalid), Err(Error::InvalidShape));
    }
}

#[test]
fn no_normalization_or_unbounded_grants() {
    let mut value = claims();
    value.grants[0].scopes = vec!["write".into(), "read".into()];
    assert!(encode_claims(&value).is_err());
    value.grants[0].scopes = vec!["read".into(), "read".into()];
    assert!(encode_claims(&value).is_err());
    value = claims();
    value.grants.push(value.grants[0].clone());
    assert!(encode_claims(&value).is_err());
    value = claims();
    value.grants[0].target = "a".repeat(65).parse().unwrap();
    assert_eq!(encode_claims(&value), Err(Error::InvalidShape));
    value = claims();
    value.grants[0].scopes = vec!["a".repeat(65)];
    assert!(encode_claims(&value).is_err());

    // Maximum legal shape, including principal lengths and label/count limits.
    value = claims();
    value.issuer = Principal::from_slice(&[1; 29]);
    value.account_subject = Principal::from_slice(&[2; 29]);
    value.grants = (0..16)
        .map(|i| DelegatedRoleGrant {
            target: format!("r{i:02}{}", "a".repeat(61)).parse().unwrap(),
            scopes: (0..32)
                .map(|j| format!("s{j:02}{}", "a".repeat(61)))
                .collect(),
        })
        .collect();
    let bytes = encode_claims(&value).unwrap();
    assert!(bytes.len() < MAX_CLAIMS_BYTES);
    assert_eq!(decode_claims(&bytes).unwrap(), value);
    // A grant may offer 32 scopes, while a session request narrows to at most 16.
    // Keep these independent protocol ceilings even though grants/requests each
    // happen to have a 16-item limit in different dimensions.
    use ic_auth::session_request::{SessionRequest, SessionRequestError};
    let offered = &value.grants[0].scopes;
    assert!(SessionRequest::new(offered[..16].to_vec(), None).is_ok());
    assert_eq!(
        SessionRequest::new(offered[..17].to_vec(), None),
        Err(SessionRequestError::InvalidRequest)
    );
    value.grants[0].scopes.push("z".into());
    assert!(encode_claims(&value).is_err());
    value.grants[0].scopes.pop();
    value.grants.push(DelegatedRoleGrant {
        target: "z".parse().unwrap(),
        scopes: vec!["read".into()],
    });
    assert!(encode_claims(&value).is_err());
}

#[test]
fn wire_order_duplicates_and_invalid_grammar_refuse_without_normalization() {
    let mut value = claims();
    value.grants[0].scopes = vec!["read".into(), "wear".into()];
    let mut bytes = encode_claims(&value).unwrap();
    // Equal-length last scope: neither duplicate nor descending order is admitted.
    let end = bytes.len();
    bytes[end - 4..].copy_from_slice(b"read");
    assert!(matches!(decode_claims(&bytes), Err(Error::Grants(_))));
    bytes[end - 4..].copy_from_slice(b"aaaa");
    assert!(matches!(decode_claims(&bytes), Err(Error::Grants(_))));
    bytes[end - 4..].copy_from_slice(b"READ");
    assert!(matches!(decode_claims(&bytes), Err(Error::Grants(_))));
    let mut bytes = hex(VECTOR);
    bytes[225..229].copy_from_slice(b"USER");
    assert_eq!(decode_claims(&bytes), Err(Error::InvalidShape));
}

#[test]
fn accepted_claim_bytes_are_the_unique_encoding_used_for_fingerprints() {
    use sha2::{Digest, Sha256};
    let mut wide = claims();
    wide.grants = (0..16)
        .map(|role| DelegatedRoleGrant {
            target: format!("role{role:02}").parse().unwrap(),
            scopes: (0..32)
                .map(|scope| format!("scope{scope:02}{}", "x".repeat(57)))
                .collect(),
        })
        .collect();
    for mut value in [claims(), wide] {
        for mechanism in [PortableMechanism::Ed25519, PortableMechanism::Certified] {
            value.mechanism = mechanism;
            let bytes = encode_claims(&value).unwrap();
            let decoded = decode_claims(&bytes).unwrap();
            assert_eq!(encode_claims(&decoded).unwrap(), bytes);
            assert_eq!(
                claims_hash(&decoded).unwrap(),
                <[u8; 32]>::from(Sha256::digest(&bytes))
            );
        }
    }
    // Exercise accepted mutations as well as malformed ones: no alternate byte
    // spelling may decode to a value whose canonical fingerprint differs.
    let original = hex(VECTOR);
    for offset in 0..original.len() {
        for replacement in [0, 1, 0x7f, 0xff] {
            let mut bytes = original.clone();
            bytes[offset] = replacement;
            if let Ok(decoded) = decode_claims(&bytes) {
                assert_eq!(encode_claims(&decoded).unwrap(), bytes);
                assert_eq!(
                    claims_hash(&decoded).unwrap(),
                    <[u8; 32]>::from(Sha256::digest(&bytes))
                );
            }
        }
    }
}
