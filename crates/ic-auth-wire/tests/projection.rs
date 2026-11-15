//! Frozen browser records constrain every projected field, without an Auth dependency.
use ic_auth_protocol_types as d;
use ic_auth_wire::{
    CredentialEnvelope, model as w,
    projection::ProjectionError,
    wire::{Bytes, List, Text},
};
fn fixture(name: &str) -> w::PortableCredential {
    let (_, hex) = include_str!("fixtures/admission-frames.tsv")
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .find(|(key, _)| *key == name)
        .unwrap();
    let bytes: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    CredentialEnvelope::decode_request(&bytes).unwrap()
}
fn certified() -> w::PortableCredential {
    fixture("credential-certified-request")
}
fn proof(value: &mut w::PortableCredential) -> &mut w::CertifiedProof {
    let w::PortableProof::Certified(value) = &mut value.proof else {
        panic!("certified fixture")
    };
    value
}
fn principal(byte: u8) -> d::Principal {
    d::Principal::from_slice(&[byte])
}
fn audience() -> d::AudienceId {
    d::AudienceId {
        canonical_network_id: d::CanonicalId::from_bytes([1; 32]),
        fleet_id: d::CanonicalId::from_bytes([2; 32]),
    }
}
fn grants() -> Vec<d::DelegatedRoleGrant> {
    vec![d::DelegatedRoleGrant {
        target: "project_instance".parse().unwrap(),
        scopes: vec!["write".parse().unwrap(), "read".parse().unwrap()],
    }]
}

#[test]
fn every_certified_field_preserves_the_independent_browser_record() {
    let actual = d::Credential::try_from(certified()).unwrap();
    let expected = d::Credential {
        claims: vec![128, 0, 255, 1],
        proof: d::CredentialProof::Certified(Box::new(d::CertifiedProofMaterial {
            delegation: d::RootDelegation {
                certificate: d::DelegationCertificate {
                    issued_at_ns: 127,
                    delegation: d::IssuerDelegation {
                        root: principal(3),
                        issuer: principal(4),
                        binding_hash: [5; 32],
                        seed_hash: [6; 32],
                        not_before_ns: 128,
                        expires_at_ns: 129,
                        max_token_ttl_ns: 130,
                        audience: audience(),
                        grants: grants(),
                    },
                },
                header: d::RootBatchHeader {
                    schema_version: 257,
                    root: principal(7),
                    batch_id: [8; 32],
                    proof_epoch: 131,
                    registry_epoch: 132,
                    registry_hash: [9; 32],
                    tree_root: [10; 32],
                    not_before_ns: 133,
                    expires_at_ns: 134,
                    key_name: "root-key".into(),
                    derivation_path_hash: [11; 32],
                    key_version: 135,
                },
                leaf: d::DelegationLeaf {
                    proof_epoch: 136,
                    registry_epoch: 140,
                    registry_hash: [16; 32],
                    delegation: d::IssuerDelegation {
                        root: principal(12),
                        issuer: principal(13),
                        binding_hash: [14; 32],
                        seed_hash: [15; 32],
                        max_token_ttl_ns: 137,
                        audience: audience(),
                        grants: grants(),
                        not_before_ns: 138,
                        expires_at_ns: 139,
                    },
                },
                witness: vec![
                    d::MerkleStep::Left([17; 32]),
                    d::MerkleStep::Right([18; 32]),
                ],
                signature: d::RootSignatureMaterial {
                    key_name: "signature-key".into(),
                    derivation_path: vec![vec![], vec![0, 128, 255]],
                    public_key: vec![19, 20],
                    signature: vec![21, 22, 23],
                },
            },
            issuer_signature_cbor: vec![24, 25, 26, 27],
            issuer_public_key_der: vec![28, 29],
        })),
    };
    assert_eq!(actual, expected);
}

#[test]
fn ed25519_projection_preserves_opaque_claims_and_short_signatures() {
    let actual = d::Credential::try_from(fixture("credential-ed25519-request")).unwrap();
    assert_eq!(
        actual,
        d::Credential {
            claims: vec![0, 128, 255],
            proof: d::CredentialProof::Ed25519((0..64).collect())
        }
    );
    for length in [0, 1, 63, 64] {
        let value = w::PortableCredential {
            claims: Bytes::new(vec![255]).unwrap(),
            proof: w::PortableProof::Ed25519(Bytes::new(vec![7; length]).unwrap()),
        };
        assert_eq!(
            d::Credential::try_from(value).unwrap(),
            d::Credential {
                claims: vec![255],
                proof: d::CredentialProof::Ed25519(vec![7; length])
            }
        );
    }
}

#[test]
fn projection_checks_label_grammar_in_both_copies_without_repair() {
    for leaf in [false, true] {
        for role in ["", "Upper", "a/b"] {
            let mut value = certified();
            let p = proof(&mut value);
            let w::RootProof::IcChainKeyBatchSignatureV1(root) = &mut p.delegation.root_proof;
            let target = if leaf {
                &mut root.delegation_cert.grants
            } else {
                &mut p.delegation.cert.grants
            };
            *target = List::new(vec![w::DelegatedRoleGrant {
                target: Text::new(role.into()).unwrap(),
                scopes: List::new(vec![]).unwrap(),
            }])
            .unwrap();
            assert!(matches!(
                d::Credential::try_from(value),
                Err(ProjectionError::Role(_))
            ));
        }
        for scope in ["", "Read", "a::b"] {
            let mut value = certified();
            let p = proof(&mut value);
            let w::RootProof::IcChainKeyBatchSignatureV1(root) = &mut p.delegation.root_proof;
            let target = if leaf {
                &mut root.delegation_cert.grants
            } else {
                &mut p.delegation.cert.grants
            };
            *target = List::new(vec![w::DelegatedRoleGrant {
                target: Text::new("role".into()).unwrap(),
                scopes: List::new(vec![Text::new(scope.into()).unwrap()]).unwrap(),
            }])
            .unwrap();
            assert!(matches!(
                d::Credential::try_from(value),
                Err(ProjectionError::Scope(_))
            ));
        }
    }
}

#[test]
fn duplicate_and_empty_grants_remain_untrusted_verifier_inputs() {
    let mut value = certified();
    let p = proof(&mut value);
    p.delegation.cert.grants = List::new(vec![
        w::DelegatedRoleGrant {
            target: Text::new("app".into()).unwrap(),
            scopes: List::new(vec![Text::new("read".into()).unwrap(); 2]).unwrap()
        };
        2
    ])
    .unwrap();
    let w::RootProof::IcChainKeyBatchSignatureV1(root) = &mut p.delegation.root_proof;
    root.delegation_cert.grants = List::new(vec![]).unwrap();
    let d::CredentialProof::Certified(p) = d::Credential::try_from(value).unwrap().proof else {
        unreachable!()
    };
    assert_eq!(p.delegation.certificate.delegation.grants.len(), 2);
    assert_eq!(
        p.delegation.certificate.delegation.grants[0].scopes.len(),
        2
    );
    assert!(p.delegation.leaf.delegation.grants.is_empty());
}

#[test]
fn projection_moves_large_buffers_and_preserves_label_storage() {
    let mut value = certified();
    value.claims = Bytes::new(vec![3; 65536]).unwrap();
    let claims = value.claims.as_slice().as_ptr();
    let p = proof(&mut value);
    p.issuer_signature.signature_cbor = Bytes::new(vec![4; 4096]).unwrap();
    let cbor = p.issuer_signature.signature_cbor.as_slice().as_ptr();
    let role = p.delegation.cert.grants.as_slice()[0]
        .target
        .as_str()
        .as_ptr();
    let w::RootProof::IcChainKeyBatchSignatureV1(root) = &p.delegation.root_proof;
    let path = root.signature.derivation_path.as_slice()[1]
        .as_slice()
        .as_ptr();
    let actual = d::Credential::try_from(value).unwrap();
    assert_eq!(actual.claims.as_ptr(), claims);
    let d::CredentialProof::Certified(p) = actual.proof else {
        unreachable!()
    };
    assert_eq!(p.issuer_signature_cbor.as_ptr(), cbor);
    assert_eq!(
        p.delegation.certificate.delegation.grants[0]
            .target
            .as_str()
            .as_ptr(),
        role
    );
    assert_eq!(p.delegation.signature.derivation_path[1].as_ptr(), path);
}
