use ic_auth::canonical::*;
use ic_auth_protocol_types::*;

// Frozen projection of Canic's mainnet fixture; not a runtime trust anchor.
fn audience() -> DelegationAudience {
    DelegationAudience::Fleet(AudienceId {
        canonical_network_id: "402b3681453fb9cfa356b6ea7abde2de53cc4c665caaf438535bddc1e1679f60"
            .parse()
            .unwrap(),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    })
}

fn p(id: u8) -> Principal {
    Principal::from_slice(&[id; 29])
}

fn sample_cert() -> DelegationCert {
    let issuer_proof_alg = IssuerProofAlgorithm::IcCanisterSignatureV1;
    let issuer_proof_binding = IssuerProofBinding::IcCanisterSignatureV1 { seed_hash: [8; 32] };
    let issuer_proof_binding_hash =
        issuer_proof_binding_hash(p(3), issuer_proof_alg, issuer_proof_binding).unwrap();

    DelegationCert {
        root_pid: p(1),
        issuer_pid: p(3),
        issuer_proof_alg,
        issuer_proof_binding_hash,
        issuer_proof_binding,
        issued_at_ns: 100,
        not_before_ns: 100,
        expires_at_ns: 200,
        max_token_ttl_ns: 60,
        aud: audience(),
        grants: vec![grant("project_instance", &["read", "write"])],
    }
}

fn grant(role: &str, scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role.parse().unwrap(),
        scopes: scopes.iter().map(|scope| (*scope).to_string()).collect(),
    }
}

fn chain_key_proof() -> IcChainKeyBatchSignatureProofV1 {
    let key_id = ChainKeyKeyId {
        name: "test_key_1".to_string(),
    };

    IcChainKeyBatchSignatureProofV1 {
        header: ChainKeyBatchHeaderV1 {
            schema_version: 1,
            root_canister_id: p(1),
            batch_id: [31; 32],
            proof_epoch: 2,
            registry_epoch: 3,
            registry_hash: [32; 32],
            tree_root: [33; 32],
            not_before_ns: 100,
            expires_at_ns: 200,
            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
            key_id: key_id.clone(),
            derivation_path_hash: [34; 32],
            key_version: 4,
        },
        delegation_cert: ChainKeyDelegationCertV1 {
            root_canister_id: p(1),
            issuer_canister_id: p(3),
            proof_epoch: 2,
            issuer_proof_algorithm: IssuerProofAlgorithm::IcCanisterSignatureV1,
            issuer_proof_binding_hash: [35; 32],
            issuer_proof_binding: IssuerProofBinding::IcCanisterSignatureV1 {
                seed_hash: [36; 32],
            },
            max_token_ttl_ns: 60,
            audience: audience(),
            grants: vec![grant("project_instance", &["read", "write"])],
            not_before_ns: 100,
            expires_at_ns: 200,
            registry_epoch: 3,
            registry_hash: [32; 32],
        },
        issuer_witness: ChainKeyBatchWitnessV1 {
            steps: vec![
                ChainKeyBatchWitnessStepV1::LeftSibling([37; 32]),
                ChainKeyBatchWitnessStepV1::RightSibling([38; 32]),
            ],
        },
        signature: ChainKeyRootSignatureV1 {
            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
            key_id,
            derivation_path: vec![b"canic".to_vec(), b"delegation".to_vec()],
            public_key: vec![39; 33],
            signature: vec![40; 64],
        },
    }
}

#[test]
fn cert_hash_rejects_noncanonical_scope_order() {
    let mut cert = sample_cert();
    cert.grants = vec![grant("project_instance", &["write", "read"])];

    assert_eq!(
        cert_hash(&cert),
        Err(CanonicalAuthError::NonCanonicalScopes)
    );
}

#[test]
fn claims_hash_rejects_noncanonical_scopes() {
    let claims = DelegatedTokenClaims {
        presenter: p(10),
        subject: p(10),
        issuer_pid: p(11),
        cert_hash: [12; 32],
        issued_at_ns: 100,
        expires_at_ns: 120,
        aud: audience(),
        grants: vec![DelegatedRoleGrant {
            target: "project_instance".parse().unwrap(),
            scopes: vec!["Read".to_string()],
        }],
        nonce: [14; 16],
        ext: None,
    };

    assert_eq!(
        claims_hash(&claims),
        Err(CanonicalAuthError::InvalidScope {
            scope: "Read".to_string(),
        })
    );
}

#[test]
fn claims_hash_rejects_noncanonical_scope_order() {
    let left = DelegatedTokenClaims {
        presenter: p(10),
        subject: p(10),
        issuer_pid: p(11),
        cert_hash: [12; 32],
        issued_at_ns: 100,
        expires_at_ns: 120,
        aud: audience(),
        grants: vec![grant("project_instance", &["write", "read"])],
        nonce: [14; 16],
        ext: None,
    };

    assert_eq!(
        claims_hash(&left),
        Err(CanonicalAuthError::NonCanonicalScopes)
    );
}

#[test]
fn claims_hash_binds_signed_presenter() {
    let claims = DelegatedTokenClaims {
        presenter: p(10),
        subject: p(10),
        issuer_pid: p(11),
        cert_hash: [12; 32],
        issued_at_ns: 100,
        expires_at_ns: 120,
        aud: audience(),
        grants: vec![grant("project_instance", &["read"])],
        nonce: [14; 16],
        ext: None,
    };
    let mut changed_presenter = claims.clone();
    changed_presenter.presenter = p(9);

    assert_ne!(
        claims_hash(&claims).unwrap(),
        claims_hash(&changed_presenter).unwrap()
    );
}

#[test]
fn chain_key_root_proof_hash_binds_witness_direction_and_public_key() {
    let proof = chain_key_proof();
    let delegation_proof = DelegationProof {
        cert: sample_cert(),
        root_proof: RootProof::IcChainKeyBatchSignatureV1(proof.clone()),
    };
    let base_hash = proof_hash(&delegation_proof).unwrap();
    let mut changed_witness = proof.clone();
    changed_witness.issuer_witness.steps[0] = ChainKeyBatchWitnessStepV1::RightSibling([36; 32]);
    let mut changed_public_key = proof;
    changed_public_key.signature.public_key[0] ^= 1;

    assert_ne!(
        base_hash,
        proof_hash(&DelegationProof {
            cert: sample_cert(),
            root_proof: RootProof::IcChainKeyBatchSignatureV1(changed_witness),
        })
        .unwrap()
    );
    assert_ne!(
        base_hash,
        proof_hash(&DelegationProof {
            cert: sample_cert(),
            root_proof: RootProof::IcChainKeyBatchSignatureV1(changed_public_key),
        })
        .unwrap()
    );
}

#[test]
fn claims_hash_binds_ext_bytes() {
    let mut left = DelegatedTokenClaims {
        presenter: p(10),
        subject: p(10),
        issuer_pid: p(11),
        cert_hash: [12; 32],
        issued_at_ns: 100,
        expires_at_ns: 120,
        aud: audience(),
        grants: vec![grant("project_instance", &["read"])],
        nonce: [14; 16],
        ext: Some(b"user=1".to_vec()),
    };
    let mut right = left.clone();
    right.ext = Some(b"user=2".to_vec());

    assert_ne!(claims_hash(&left).unwrap(), claims_hash(&right).unwrap());
    left.ext = None;
    assert_ne!(claims_hash(&left).unwrap(), claims_hash(&right).unwrap());
}

#[test]
fn claims_hash_rejects_oversized_ext() {
    let claims = DelegatedTokenClaims {
        presenter: p(10),
        subject: p(10),
        issuer_pid: p(11),
        cert_hash: [12; 32],
        issued_at_ns: 100,
        expires_at_ns: 120,
        aud: audience(),
        grants: vec![grant("project_instance", &["read"])],
        nonce: [14; 16],
        ext: Some(vec![1; MAX_TOKEN_EXT_BYTES + 1]),
    };

    assert_eq!(
        claims_hash(&claims),
        Err(CanonicalAuthError::TokenExtTooLarge {
            len: MAX_TOKEN_EXT_BYTES + 1,
            max: MAX_TOKEN_EXT_BYTES,
        })
    );
}

#[test]
fn issuer_proof_hash_binds_signature_and_public_key() {
    let proof = IssuerProof::IcCanisterSignatureV1(IcCanisterSignatureProofV1 {
        signature_cbor: vec![1, 2, 3],
        public_key_der: vec![4, 5, 6],
    });
    let mut changed_signature = proof.clone();
    let mut changed_public_key = proof.clone();
    let IssuerProof::IcCanisterSignatureV1(changed) = &mut changed_signature;
    changed.signature_cbor[0] ^= 1;
    let IssuerProof::IcCanisterSignatureV1(changed) = &mut changed_public_key;
    changed.public_key_der[0] ^= 1;

    assert_ne!(
        issuer_proof_hash(&proof),
        issuer_proof_hash(&changed_signature)
    );
    assert_ne!(
        issuer_proof_hash(&proof),
        issuer_proof_hash(&changed_public_key)
    );
}

#[test]
fn issuer_proof_binding_hash_binds_authority_context() {
    let binding = IssuerProofBinding::IcCanisterSignatureV1 { seed_hash: [7; 32] };
    let base =
        issuer_proof_binding_hash(p(1), IssuerProofAlgorithm::IcCanisterSignatureV1, binding);

    assert_ne!(
        base,
        issuer_proof_binding_hash(p(2), IssuerProofAlgorithm::IcCanisterSignatureV1, binding)
    );
    assert_ne!(
        base,
        issuer_proof_binding_hash(
            p(1),
            IssuerProofAlgorithm::IcCanisterSignatureV1,
            IssuerProofBinding::IcCanisterSignatureV1 { seed_hash: [8; 32] },
        )
    );
}

#[test]
fn canic_signed_hash_vectors_are_unchanged() {
    let proof = chain_key_proof();
    assert_eq!(
        chain_key_batch_header_hash(&proof.header).unwrap(),
        [
            231, 134, 199, 186, 130, 244, 250, 243, 254, 252, 150, 140, 3, 154, 230, 252, 45, 52,
            89, 215, 119, 228, 233, 231, 245, 96, 54, 45, 33, 18, 44, 192
        ]
    );
    assert_eq!(
        chain_key_delegation_cert_hash(&proof.delegation_cert).unwrap(),
        [
            229, 125, 46, 9, 210, 247, 67, 53, 147, 231, 131, 196, 39, 154, 47, 102, 180, 230, 40,
            138, 15, 36, 24, 77, 76, 112, 166, 245, 86, 22, 94, 216
        ]
    );
    assert_eq!(
        proof_hash(&DelegationProof {
            cert: sample_cert(),
            root_proof: RootProof::IcChainKeyBatchSignatureV1(proof)
        })
        .unwrap(),
        [
            40, 0, 43, 111, 76, 97, 74, 157, 46, 216, 136, 161, 160, 142, 247, 203, 124, 187, 2,
            145, 177, 196, 48, 209, 112, 103, 122, 135, 249, 222, 243, 111
        ]
    );
}

#[test]
fn duplicate_roles_and_scopes_are_rejected() {
    let mut cert = sample_cert();
    cert.grants.push(cert.grants[0].clone());
    assert_eq!(cert_hash(&cert), Err(CanonicalAuthError::NonCanonicalRoles));
    cert.grants = vec![grant("app", &["read", "read"])];
    assert_eq!(
        cert_hash(&cert),
        Err(CanonicalAuthError::NonCanonicalScopes)
    );
}

#[test]
fn scope_grammar_and_boundaries_match_the_source_contract() {
    for good in ["read", "app:read", "a_b:c-d", "0"] {
        assert!(validate_scope_label(good).is_ok());
    }
    assert!(validate_scope_label(&"a".repeat(64)).is_ok());
    for bad in [
        "", ":read", "read:", "a::b", "a:_b", "Read", "é", "a/b", "*",
    ] {
        assert!(validate_scope_label(bad).is_err());
    }
    assert!(validate_scope_label(&"a".repeat(65)).is_err());
}
