//! Canonical claims acceptance against retained independent bytes, not round trips alone.

use ic_auth::claims::{ClaimsError as E, claims_hash, decode_claims, encode_claims};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};

fn fixture() -> Vec<u8> {
    let text = include_str!("fixtures/claims-v2.hex").trim();
    assert_eq!(text.len(), 500);
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn grant(role: &str, scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role.parse().unwrap(),
        scopes: scopes.iter().map(|label| label.parse().unwrap()).collect(),
    }
}

fn sample() -> PortableClaims {
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
        ext: None,
        device_slot: 0,
        device_generation: 1,
        login_started_at_ns: 0x0102030405060708,
        login_deadline_ns: 0x1112131415161718,
        issued_at_ns: 0x2122232425262728,
        admission_expires_at_ns: 0x3132333435363738,
        nonce: [0x55; 16],
        grants: vec![grant("user", &["read"])],
    }
}

#[test]
fn fixed_bytes_and_digest_bind_both_mechanism_tags() {
    let expected = fixture();
    let value = sample();
    assert_eq!(encode_claims(&value).unwrap(), expected);
    assert_eq!(decode_claims(&expected).unwrap(), value);
    let digest: String = claims_hash(&value)
        .unwrap()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        digest,
        "e0d179c866e21b8009fa5dba6bcf85ccc99065f83f141d9e9c2aeb2bc61a2e9d"
    );
    let mut certified_bytes = expected;
    certified_bytes[30] = 0;
    let mut certified = value;
    certified.mechanism = PortableMechanism::Certified;
    assert_eq!(encode_claims(&certified).unwrap(), certified_bytes);
    assert_eq!(decode_claims(&certified_bytes).unwrap(), certified);
}

#[test]
fn every_field_contributes_to_the_signed_identity() {
    let changes: &[fn(&mut PortableClaims)] = &[
        |v| v.mechanism = PortableMechanism::Certified,
        |v| v.audience.canonical_network_id = CanonicalId::from_bytes([0; 32]),
        |v| v.audience.fleet_id = CanonicalId::from_bytes([0; 32]),
        |v| v.issuer = Principal::management_canister(),
        |v| v.key_binding_id[1] ^= 1,
        |v| v.browser_public_key[2] ^= 1,
        |v| v.account_subject = Principal::from_slice(&[3]),
        |v| v.device_slot = 7,
        |v| v.device_generation = u64::MAX,
        |v| v.login_started_at_ns ^= 1,
        |v| v.login_deadline_ns ^= 1,
        |v| v.issued_at_ns ^= 1,
        |v| v.admission_expires_at_ns ^= 1,
        |v| v.nonce[3] ^= 1,
        |v| v.grants[0].target = "admin".parse().unwrap(),
        |v| v.grants[0].scopes[0] = "write".parse().unwrap(),
    ];
    let original = claims_hash(&sample()).unwrap();
    for change in changes {
        let mut value = sample();
        change(&mut value);
        assert_ne!(claims_hash(&value).unwrap(), original);
    }
}

#[test]
fn every_truncation_and_retired_or_unknown_domain_refuses() {
    let bytes = fixture();
    for end in 0..bytes.len() {
        assert_eq!(
            decode_claims(&bytes[..end]),
            Err(E::Truncated),
            "prefix {end}"
        );
    }
    for offset in [0, 25, 26, 27, 28, 29, 30] {
        let mut changed = bytes.clone();
        changed[offset] = 255;
        assert_eq!(decode_claims(&changed), Err(E::WrongDomain));
    }
    let mut retired = bytes.clone();
    retired[29] = 1;
    assert_eq!(decode_claims(&retired), Err(E::WrongDomain));
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(decode_claims(&trailing), Err(E::TrailingBytes));
}

#[test]
fn length_count_and_byte_budgets_are_admitted_before_allocation() {
    for offset in [95, 164, 226, 230, 238, 242] {
        let mut bytes = fixture();
        bytes[offset..offset + 4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(decode_claims(&bytes), Err(E::InvalidShape));
    }
    for offset in [226, 230, 238, 242] {
        let mut bytes = fixture();
        bytes[offset..offset + 4].fill(0);
        assert_eq!(decode_claims(&bytes), Err(E::InvalidShape));
    }
    assert_eq!(decode_claims(&vec![0; 65536]), Err(E::WrongDomain));
    assert_eq!(decode_claims(&vec![0; 65537]), Err(E::TooLarge));
    let mut grants_at_limit = fixture()[..226].to_vec();
    grants_at_limit.resize(226 + 4096, 0);
    assert_eq!(decode_claims(&grants_at_limit), Err(E::InvalidShape));
    grants_at_limit.push(0);
    assert_eq!(decode_claims(&grants_at_limit), Err(E::TooLarge));
}

#[test]
fn invalid_labels_and_noncanonical_order_are_never_repaired() {
    for offset in [234, 246] {
        for byte in [255, b'A', b' '] {
            let mut bytes = fixture();
            bytes[offset] = byte;
            assert_eq!(decode_claims(&bytes), Err(E::InvalidShape));
        }
    }
    // Independent payload spelling: replace the one-scope list with two labels.
    for second in [b"read", b"aaaa"] {
        let mut bytes = fixture();
        bytes[238..242].copy_from_slice(&2_u32.to_be_bytes());
        bytes.extend_from_slice(&4_u32.to_be_bytes());
        bytes.extend_from_slice(second);
        assert_eq!(decode_claims(&bytes), Err(E::NonCanonicalScopes));
    }
    let mut two_roles = fixture();
    two_roles[226..230].copy_from_slice(&2_u32.to_be_bytes());
    two_roles.extend_from_slice(&fixture()[230..]);
    assert_eq!(decode_claims(&two_roles), Err(E::NonCanonicalRoles));
    for scopes in [["write", "read"], ["read", "read"]] {
        let mut value = sample();
        value.grants = vec![grant("user", &scopes)];
        assert_eq!(encode_claims(&value), Err(E::NonCanonicalScopes));
        assert_eq!(claims_hash(&value), Err(E::NonCanonicalScopes));
        assert_eq!(value.grants[0].scopes[0].as_str(), scopes[0]);
    }
}

#[test]
fn device_subject_and_error_order_remain_structural() {
    let mut value = sample();
    value.device_generation = 0;
    value.account_subject = Principal::anonymous();
    value.grants.clear();
    assert_eq!(encode_claims(&value), Err(E::InvalidShape));
    value.device_generation = 1;
    assert_eq!(encode_claims(&value), Err(E::AnonymousSubject));
    assert_eq!(claims_hash(&value), Err(E::AnonymousSubject));
    value.account_subject = Principal::management_canister();
    assert_eq!(
        encode_claims(&value),
        Err(E::InvalidGrantCount { found: 0 })
    );
    let mut bytes = fixture();
    bytes[169] = 8;
    assert_eq!(decode_claims(&bytes), Err(E::InvalidShape));
    bytes[169] = 7;
    bytes[170..178].fill(0);
    assert_eq!(decode_claims(&bytes), Err(E::InvalidShape));
    bytes[170..178].fill(255);
    bytes[168] = 4;
    assert_eq!(decode_claims(&bytes), Err(E::AnonymousSubject));
}

#[test]
fn empty_and_full_principals_and_untrusted_time_are_not_authority_checks() {
    for principal in [
        Principal::management_canister(),
        Principal::from_slice(&[255; 29]),
    ] {
        let mut value = sample();
        value.issuer = principal;
        value.account_subject = principal;
        value.nonce = [0; 16];
        value.login_started_at_ns = u64::MAX;
        value.login_deadline_ns = 0;
        value.issued_at_ns = u64::MAX;
        value.admission_expires_at_ns = 0;
        assert_eq!(
            decode_claims(&encode_claims(&value).unwrap()).unwrap(),
            value
        );
    }
    let mut anonymous_issuer = sample();
    anonymous_issuer.issuer = Principal::anonymous();
    assert!(encode_claims(&anonymous_issuer).is_ok());
    for offset in [95, 164] {
        let mut bytes = fixture();
        bytes[offset..offset + 4].copy_from_slice(&30_u32.to_be_bytes());
        assert_eq!(decode_claims(&bytes), Err(E::InvalidShape));
    }
}

#[test]
fn maximum_grants_fit_and_the_33rd_entry_refuses_before_its_payload() {
    for roles in 1..=16 {
        let mut value = sample();
        value.grants = (0..roles)
            .map(|role| DelegatedRoleGrant {
                target: format!("r{role:02}{}", "x".repeat(61)).parse().unwrap(),
                scopes: (0..(32 / roles + usize::from(role < 32 % roles)))
                    .map(|scope| format!("s{scope:02}{}", "x".repeat(61)).parse().unwrap())
                    .collect(),
            })
            .collect();
        let bytes = encode_claims(&value).unwrap();
        assert_eq!(bytes.len(), 226 + 4 + roles * 72 + 32 * 68);
        assert_eq!(decode_claims(&bytes).unwrap(), value);
        if roles == 16 {
            assert_eq!(bytes.len() - 226, 3332);
            let count = bytes.len() - 2 * 68 - 4;
            let mut missing_payload = bytes[..count + 4].to_vec();
            missing_payload[count..].copy_from_slice(&3_u32.to_be_bytes());
            assert_eq!(
                decode_claims(&missing_payload),
                Err(E::TooManyScopeEntries { found: 33, max: 32 })
            );
            value
                .grants
                .last_mut()
                .unwrap()
                .scopes
                .push("z".parse().unwrap());
            assert_eq!(
                encode_claims(&value),
                Err(E::TooManyScopeEntries { found: 33, max: 32 })
            );
        }
    }
    let mut value = sample();
    value.grants = vec![grant("user", &["read"]); 17];
    assert_eq!(
        encode_claims(&value),
        Err(E::InvalidGrantCount { found: 17 })
    );
    value.grants = vec![grant("user", &[])];
    assert_eq!(
        encode_claims(&value),
        Err(E::InvalidScopeCount { found: 0 })
    );
    value.grants[0].scopes = vec!["read".parse().unwrap(); 33];
    assert_eq!(
        encode_claims(&value),
        Err(E::InvalidScopeCount { found: 33 })
    );
}

#[test]
fn accepted_mutations_have_no_alternate_encoding() {
    let original = fixture();
    for offset in 0..original.len() {
        for byte in [0, 1, 127, 255] {
            let mut changed = original.clone();
            changed[offset] = byte;
            if let Ok(value) = decode_claims(&changed) {
                assert_eq!(encode_claims(&value).unwrap(), changed);
            }
        }
    }
}

#[test]
fn application_extension_presence_length_and_bytes_are_signed() {
    let mut claims = sample();
    let original = claims_hash(&claims).unwrap();
    claims.ext = Some(Vec::new());
    let empty = claims_hash(&claims).unwrap();
    assert_ne!(original, empty);
    assert_eq!(
        decode_claims(&encode_claims(&claims).unwrap()).unwrap().ext,
        Some(Vec::new())
    );

    claims.ext = Some(vec![0x80; 4096]);
    let mut bytes = encode_claims(&claims).unwrap();
    assert_eq!(decode_claims(&bytes).unwrap(), claims);
    let start = bytes.windows(32).position(|w| w == [0x80; 32]).unwrap();
    assert_eq!(&bytes[start - 4..start], &4096u32.to_be_bytes());
    bytes[start - 4..start].copy_from_slice(&4097u32.to_be_bytes());
    assert!(decode_claims(&bytes).is_err());
    claims.ext.as_mut().unwrap().push(0);
    assert_eq!(encode_claims(&claims), Err(E::TooLarge));
    assert_eq!(claims_hash(&claims), Err(E::TooLarge));
}
