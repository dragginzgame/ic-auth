use super::*;
use ic_auth_protocol_types::{CanonicalId, ChainKeyAlgorithm, ChainKeyKeyId, DelegatedRoleGrant};
use ic_auth_qualification_contracts::{InstallReceiver, model as wire};

#[test]
fn wire_projection_preserves_protected_authority_and_checked_identifiers() {
    let config = ReceiverConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: Principal::from_slice(&[3]),
        authority: ReceiverAuthority::Ed25519(Box::new(
            ic_auth::portable_signature::PreparedIssuerKey::new(
                ic_platform::ed25519::SigningKey::from_bytes(&[4; 32])
                    .verifying_key()
                    .to_bytes(),
            )
            .unwrap(),
        )),
        role: "user".parse().unwrap(),
        grants: ValidatedGrants::new(vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }])
        .unwrap(),
        valid_from_ns: 5,
        valid_until_ns: 6,
        enrollment_until_ns: 7,
        verification_burst: 8,
        verification_refill_ns: 9,
    };
    let mut invalid: wire::ReceiverConfig = config.clone().try_into().unwrap();
    invalid.authority = wire::ReceiverAuthority::Ed25519([0; 32]);
    assert_eq!(
        ReceiverConfig::try_from(invalid),
        Err(ic_auth_wire::wire::Error::Malformed)
    );
    let root = RootKeyPolicy {
        root_canister_id: Principal::from_slice(&[10]),
        algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
        key_id: ChainKeyKeyId {
            name: "test_key".into(),
        },
        derivation_path_hash: [11; 32],
        public_key: vec![12; 33],
        key_version: 13,
        min_accepted_key_version: 14,
        min_accepted_proof_epoch: 15,
        min_accepted_registry_epoch: 16,
        valid_from_ns: 17,
        accept_until_ns: 18,
        max_revocation_latency_ns: 19,
    };
    for authority in [
        config.authority.clone(),
        ReceiverAuthority::Certified {
            binding: [20; 32],
            root: Box::new(root),
            network_key: vec![21; 96],
        },
    ] {
        if let ReceiverAuthority::Certified {
            binding,
            mut root,
            network_key,
        } = authority.clone()
        {
            root.public_key = vec![0; 66];
            let oversized = ReceiverAuthority::Certified {
                binding,
                root,
                network_key,
            };
            assert_eq!(
                wire::ReceiverAuthority::try_from(oversized),
                Err(ic_auth_wire::wire::Error::Limit)
            );
        }
        let expected = ReceiverConfig {
            authority,
            ..config.clone()
        };
        let transport: wire::ReceiverConfig = expected.clone().try_into().unwrap();
        let authority: wire::ReceiverAuthority = expected.authority.clone().try_into().unwrap();
        assert_eq!(
            ReceiverAuthority::try_from(authority).unwrap(),
            expected.authority
        );
        let bytes = InstallReceiver::encode_request(&transport).unwrap();
        let decoded = InstallReceiver::decode_request(bytes.as_slice()).unwrap();
        assert_eq!(ReceiverConfig::try_from(decoded).unwrap(), expected);
        let mut malformed = transport.clone();
        let grant = malformed.grants.as_slice()[0].clone();
        malformed.grants = ic_auth_wire::wire::List::new(vec![grant.clone(), grant]).unwrap();
        assert_eq!(
            ReceiverConfig::try_from(malformed),
            Err(ic_auth_wire::wire::Error::Malformed)
        );
        let mut malformed = transport.clone();
        let mut grant = malformed.grants.as_slice()[0].clone();
        grant.scopes = ic_auth_wire::wire::List::new(vec![
            ic_auth_wire::wire::Text::new("write".into()).unwrap(),
            ic_auth_wire::wire::Text::new("read".into()).unwrap(),
        ])
        .unwrap();
        malformed.grants = ic_auth_wire::wire::List::new(vec![grant]).unwrap();
        assert_eq!(
            ReceiverConfig::try_from(malformed),
            Err(ic_auth_wire::wire::Error::Malformed)
        );
        let mut invalid = transport;
        invalid.role = ic_auth_wire::wire::Text::new("Invalid Role".into()).unwrap();
        assert!(ReceiverConfig::try_from(invalid).is_err());
    }
}
