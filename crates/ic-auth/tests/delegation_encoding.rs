//! Frozen root signing commitments and bounded structural admission.
use ic_auth::{
    claims::ClaimsError,
    delegation_encoding::{
        DelegationEncodingError, batch_header_hash, certificate_bytes, certificate_hash,
        delegation_leaf_hash, derivation_path_hash, issuer_binding_hash,
    },
};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, DelegationCertificate, DelegationLeaf,
    IssuerDelegation, Principal, RootBatchHeader,
};

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn expected(name: &str) -> Vec<u8> {
    let (_, value) = include_str!("fixtures/delegation.tsv")
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .find(|(key, _)| *key == name)
        .unwrap();
    hex(value)
}

fn delegation() -> IssuerDelegation {
    IssuerDelegation {
        root: Principal::from_slice(&[1; 29]),
        issuer: Principal::from_slice(&[3; 29]),
        binding_hash: expected("issuer_binding_hash").try_into().unwrap(),
        seed_hash: [8; 32],
        max_token_ttl_ns: 60,
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes(
                hex("402b3681453fb9cfa356b6ea7abde2de53cc4c665caaf438535bddc1e1679f60")
                    .try_into()
                    .unwrap(),
            ),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        grants: vec![DelegatedRoleGrant {
            target: "project_instance".parse().unwrap(),
            scopes: vec!["read".parse().unwrap(), "write".parse().unwrap()],
        }],
        not_before_ns: 100,
        expires_at_ns: 200,
    }
}

fn certificate() -> DelegationCertificate {
    DelegationCertificate {
        delegation: delegation(),
        issued_at_ns: 100,
    }
}

fn leaf() -> DelegationLeaf {
    let mut delegation = delegation();
    delegation.binding_hash = [35; 32];
    delegation.seed_hash = [36; 32];
    DelegationLeaf {
        delegation,
        proof_epoch: 2,
        registry_epoch: 3,
        registry_hash: [32; 32],
    }
}

fn header() -> RootBatchHeader {
    RootBatchHeader {
        schema_version: 1,
        root: Principal::from_slice(&[1; 29]),
        batch_id: [31; 32],
        proof_epoch: 2,
        registry_epoch: 3,
        registry_hash: [32; 32],
        tree_root: [33; 32],
        not_before_ns: 100,
        expires_at_ns: 200,
        key_name: "test_key_1".into(),
        derivation_path_hash: [34; 32],
        key_version: 4,
    }
}

#[test]
fn frozen_signed_bytes_and_all_hash_domains_match_independent_vectors() {
    let cert = certificate();
    assert_eq!(certificate_bytes(&cert).unwrap(), expected("certificate"));
    assert_eq!(
        certificate_hash(&cert).unwrap().as_slice(),
        expected("certificate_hash")
    );
    assert_eq!(
        issuer_binding_hash(cert.delegation.issuer, cert.delegation.seed_hash).as_slice(),
        expected("issuer_binding_hash")
    );
    assert_eq!(
        delegation_leaf_hash(&leaf()).unwrap().as_slice(),
        expected("leaf_hash")
    );
    assert_eq!(
        batch_header_hash(&header()).unwrap().as_slice(),
        expected("batch_header_hash")
    );
    assert_eq!(
        derivation_path_hash(&[b"canic".to_vec(), b"delegation".to_vec()])
            .unwrap()
            .as_slice(),
        expected("derivation_path_hash")
    );
}

#[test]
fn every_shared_authorization_field_affects_certificate_and_leaf() {
    let cert = certificate();
    let baseline = certificate_hash(&cert).unwrap();
    let leaf = DelegationLeaf {
        delegation: cert.delegation.clone(),
        ..leaf()
    };
    let leaf_baseline = delegation_leaf_hash(&leaf).unwrap();
    let changes: [fn(&mut IssuerDelegation); 11] = [
        |d| d.root = Principal::from_slice(&[2; 29]),
        |d| d.issuer = Principal::from_slice(&[4; 29]),
        |d| d.binding_hash[0] ^= 1,
        |d| d.seed_hash[0] ^= 1,
        |d| d.max_token_ttl_ns += 1,
        |d| d.audience.canonical_network_id = CanonicalId::from_bytes([3; 32]),
        |d| d.audience.fleet_id = CanonicalId::from_bytes([3; 32]),
        |d| d.grants[0].target = "other".parse().unwrap(),
        |d| d.grants[0].scopes[0] = "list".parse().unwrap(),
        |d| d.not_before_ns += 1,
        |d| d.expires_at_ns += 1,
    ];
    for change in changes {
        let mut changed_cert = cert.clone();
        change(&mut changed_cert.delegation);
        assert_ne!(certificate_hash(&changed_cert).unwrap(), baseline);
        let mut changed_leaf = leaf.clone();
        change(&mut changed_leaf.delegation);
        assert_ne!(delegation_leaf_hash(&changed_leaf).unwrap(), leaf_baseline);
    }
    let mut changed = cert;
    changed.issued_at_ns += 1;
    assert_ne!(certificate_hash(&changed).unwrap(), baseline);
    // Issued-at is not a leaf field: changing certificate metadata cannot change
    // the root's signed commitment, so later verification must check consistency.
    assert_eq!(changed.delegation, leaf.delegation);
    assert_eq!(delegation_leaf_hash(&leaf).unwrap(), leaf_baseline);
}

#[test]
fn every_batch_and_leaf_epoch_field_affects_its_signed_commitment() {
    let original = header();
    let baseline = batch_header_hash(&original).unwrap();
    let changes: [fn(&mut RootBatchHeader); 13] = [
        |h| h.schema_version += 1,
        |h| h.root = Principal::from_slice(&[2; 29]),
        |h| h.batch_id[0] ^= 1,
        |h| h.proof_epoch += 1,
        |h| h.registry_epoch += 1,
        |h| h.registry_hash[0] ^= 1,
        |h| h.tree_root[0] ^= 1,
        |h| h.not_before_ns += 1,
        |h| h.expires_at_ns += 1,
        |h| h.key_name.push('x'),
        |h| h.derivation_path_hash[0] ^= 1,
        |h| h.key_version += 1,
        |h| h.expires_at_ns = u64::MAX,
    ];
    for change in changes {
        let mut candidate = original.clone();
        change(&mut candidate);
        assert_ne!(batch_header_hash(&candidate).unwrap(), baseline);
    }
    let original = leaf();
    let baseline = delegation_leaf_hash(&original).unwrap();
    for change in [
        |l: &mut DelegationLeaf| l.proof_epoch += 1,
        |l: &mut DelegationLeaf| l.registry_epoch += 1,
        |l: &mut DelegationLeaf| l.registry_hash[0] ^= 1,
    ] {
        let mut candidate = original.clone();
        change(&mut candidate);
        assert_ne!(delegation_leaf_hash(&candidate).unwrap(), baseline);
    }
}

#[test]
fn strict_grants_are_shared_but_delegations_allow_more_than_32_total_scopes() {
    let cert = certificate();
    for (grants, error) in [
        (vec![], ClaimsError::InvalidGrantCount { found: 0 }),
        (
            vec![cert.delegation.grants[0].clone(); 17],
            ClaimsError::InvalidGrantCount { found: 17 },
        ),
        (
            vec![cert.delegation.grants[0].clone(); 2],
            ClaimsError::NonCanonicalRoles,
        ),
    ] {
        let candidate = DelegationCertificate {
            delegation: IssuerDelegation {
                grants,
                ..delegation()
            },
            ..certificate()
        };
        assert_eq!(
            certificate_bytes(&candidate),
            Err(DelegationEncodingError::Grants(error))
        );
        assert_eq!(
            delegation_leaf_hash(&DelegationLeaf {
                delegation: candidate.delegation,
                ..leaf()
            }),
            Err(DelegationEncodingError::Grants(error))
        );
    }
    for scopes in [vec!["write", "read"], vec!["read", "read"]] {
        let mut candidate = certificate();
        candidate.delegation.grants[0].scopes = scopes.iter().map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            certificate_hash(&candidate),
            Err(DelegationEncodingError::Grants(
                ClaimsError::NonCanonicalScopes
            ))
        );
    }
    for count in [0, 33] {
        let mut candidate = certificate();
        candidate.delegation.grants[0].scopes = vec!["read".parse().unwrap(); count];
        assert_eq!(
            certificate_hash(&candidate),
            Err(DelegationEncodingError::Grants(
                ClaimsError::InvalidScopeCount { found: count }
            ))
        );
    }
    let mut candidate = certificate();
    candidate.delegation.grants = (0..16)
        .map(|r| DelegatedRoleGrant {
            target: format!("role{r:02}").parse().unwrap(),
            scopes: (0..32)
                .map(|s| format!("scope{s:02}").parse().unwrap())
                .collect(),
        })
        .collect();
    assert!(certificate_hash(&candidate).is_ok());
    assert!(
        delegation_leaf_hash(&DelegationLeaf {
            delegation: candidate.delegation,
            ..leaf()
        })
        .is_ok()
    );
}

#[test]
fn path_and_key_bounds_preserve_empty_components_order_and_utf8_bytes() {
    let a = b"canic".to_vec();
    let b = b"delegation".to_vec();
    let base = derivation_path_hash(&[a.clone(), b.clone()]).unwrap();
    assert_ne!(base, derivation_path_hash(&[b, a]).unwrap());
    assert_ne!(
        base,
        derivation_path_hash(&[b"canicdelegation".to_vec()]).unwrap()
    );
    assert_ne!(
        derivation_path_hash(&[]).unwrap(),
        derivation_path_hash(&[vec![]]).unwrap()
    );
    assert!(derivation_path_hash(&vec![vec![0; 256]; 32]).is_ok());
    for path in [vec![vec![]; 33], vec![vec![0; 257]]] {
        assert_eq!(
            derivation_path_hash(&path),
            Err(DelegationEncodingError::DerivationPathTooLarge)
        );
    }
    let mut header = header();
    for name in [String::new(), "a".repeat(127), "é".repeat(64)] {
        header.key_name = name;
        assert!(batch_header_hash(&header).is_ok());
    }
    header.key_name.push('a');
    assert_eq!(
        batch_header_hash(&header),
        Err(DelegationEncodingError::KeyNameTooLong)
    );
}

#[test]
fn commitments_preserve_untrusted_binding_and_time_instead_of_repairing_them() {
    let mut cert = certificate();
    let binding = issuer_binding_hash(cert.delegation.issuer, cert.delegation.seed_hash);
    assert_ne!(
        binding,
        issuer_binding_hash(Principal::from_slice(&[4; 29]), cert.delegation.seed_hash)
    );
    assert_ne!(
        binding,
        issuer_binding_hash(cert.delegation.issuer, [9; 32])
    );
    cert.delegation.binding_hash = [0; 32];
    cert.issued_at_ns = u64::MAX;
    cert.delegation.expires_at_ns = 0;
    assert!(certificate_hash(&cert).is_ok()); // Encoding is not authority admission.
    assert_eq!(cert.delegation.binding_hash, [0; 32]);
}
