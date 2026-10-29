//! Private controller-installed portable-session qualification contracts.
use ic_auth::{delegation::RootKeyPolicy, portable::ValidatedGrants};
use ic_auth_protocol_types::{AudienceId, AuthRole};
pub use ic_auth_qualification_contracts::model::{AdmissionProfile, SessionProfile};
use ic_platform::Principal;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverConfig {
    pub audience: AudienceId,
    pub issuer: Principal,
    pub authority: ReceiverAuthority,
    pub role: AuthRole,
    pub grants: ValidatedGrants,
    pub valid_from_ns: u64,
    pub valid_until_ns: u64,
    pub enrollment_until_ns: u64,
    pub verification_burst: u32,
    pub verification_refill_ns: u64,
}
/// Controller-installed test authority; never selected from submitted proofs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiverAuthority {
    Ed25519([u8; 32]),
    Certified {
        binding: [u8; 32],
        root: Box<RootKeyPolicy>,
        network_key: Vec<u8>,
    },
}

impl ReceiverAuthority {
    pub fn mechanism(&self) -> ic_auth_protocol_types::PortableMechanism {
        match self {
            Self::Ed25519(_) => ic_auth_protocol_types::PortableMechanism::Ed25519,
            Self::Certified { .. } => ic_auth_protocol_types::PortableMechanism::Certified,
        }
    }
    pub fn binding(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        match self {
            Self::Ed25519(key) => Sha256::digest(key).into(),
            Self::Certified { binding, .. } => *binding,
        }
    }
}

impl TryFrom<ReceiverAuthority> for ic_auth_qualification_contracts::model::ReceiverAuthority {
    type Error = ic_auth_wire::wire::Error;
    fn try_from(value: ReceiverAuthority) -> Result<Self, Self::Error> {
        use ic_auth_qualification_contracts::model as w;
        use ic_auth_wire::wire::{Bytes, Text};
        Ok(match value {
            ReceiverAuthority::Ed25519(key) => w::ReceiverAuthority::Ed25519(key),
            ReceiverAuthority::Certified {
                binding,
                root,
                network_key,
            } => w::ReceiverAuthority::Certified(w::CertifiedAuthority {
                binding,
                root: w::RootEnrollment {
                    root_canister_id: Bytes::new(root.root_canister_id.as_slice().to_vec())?,
                    algorithm: root.algorithm.into(),
                    key_name: Text::new(root.key_id.name)?,
                    derivation_path_hash: root.derivation_path_hash,
                    public_key: Bytes::new(root.public_key)?,
                    key_version: root.key_version,
                    min_accepted_key_version: root.min_accepted_key_version,
                    min_accepted_proof_epoch: root.min_accepted_proof_epoch,
                    min_accepted_registry_epoch: root.min_accepted_registry_epoch,
                    valid_from_ns: root.valid_from_ns,
                    accept_until_ns: root.accept_until_ns,
                    max_revocation_latency_ns: root.max_revocation_latency_ns,
                },
                network_key: Bytes::new(network_key)?,
            }),
        })
    }
}
impl TryFrom<ic_auth_qualification_contracts::model::ReceiverAuthority> for ReceiverAuthority {
    type Error = ic_auth_wire::wire::Error;
    fn try_from(
        value: ic_auth_qualification_contracts::model::ReceiverAuthority,
    ) -> Result<Self, Self::Error> {
        use ic_auth_qualification_contracts::model as w;
        use ic_auth_wire::wire::Error;
        Ok(match value {
            w::ReceiverAuthority::Ed25519(key) => ReceiverAuthority::Ed25519(key),
            w::ReceiverAuthority::Certified(w::CertifiedAuthority {
                binding,
                root,
                network_key,
            }) => ReceiverAuthority::Certified {
                binding,
                root: Box::new(RootKeyPolicy {
                    root_canister_id: Principal::try_from_slice(root.root_canister_id.as_slice())
                        .map_err(|_| Error::Malformed)?,
                    algorithm: match root.algorithm {
                        ic_auth_wire::model::ChainKeyAlgorithm::EcdsaSecp256k1 => {
                            ic_auth_protocol_types::ChainKeyAlgorithm::EcdsaSecp256k1
                        }
                    },
                    key_id: ic_auth_protocol_types::ChainKeyKeyId {
                        name: root.key_name.into_string(),
                    },
                    derivation_path_hash: root.derivation_path_hash,
                    public_key: root.public_key.into_vec(),
                    key_version: root.key_version,
                    min_accepted_key_version: root.min_accepted_key_version,
                    min_accepted_proof_epoch: root.min_accepted_proof_epoch,
                    min_accepted_registry_epoch: root.min_accepted_registry_epoch,
                    valid_from_ns: root.valid_from_ns,
                    accept_until_ns: root.accept_until_ns,
                    max_revocation_latency_ns: root.max_revocation_latency_ns,
                }),
                network_key: network_key.into_vec(),
            },
        })
    }
}

impl TryFrom<ReceiverConfig> for ic_auth_qualification_contracts::model::ReceiverConfig {
    type Error = ic_auth_wire::wire::Error;
    fn try_from(value: ReceiverConfig) -> Result<Self, Self::Error> {
        use ic_auth_wire::wire::{Bytes, List, Text};
        Ok(Self {
            audience: value.audience.try_into()?,
            issuer: Bytes::new(value.issuer.as_slice().to_vec())?,
            authority: value.authority.try_into()?,
            role: Text::new(value.role.as_str().into())?,
            grants: List::new(
                value
                    .grants
                    .into_vec()
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            )?,
            valid_from_ns: value.valid_from_ns,
            valid_until_ns: value.valid_until_ns,
            enrollment_until_ns: value.enrollment_until_ns,
            verification_burst: value.verification_burst,
            verification_refill_ns: value.verification_refill_ns,
        })
    }
}
impl TryFrom<ic_auth_qualification_contracts::model::ReceiverConfig> for ReceiverConfig {
    type Error = ic_auth_wire::wire::Error;
    fn try_from(
        value: ic_auth_qualification_contracts::model::ReceiverConfig,
    ) -> Result<Self, Self::Error> {
        use ic_auth_wire::wire::Error;
        Ok(Self {
            audience: value.audience.try_into()?,
            issuer: Principal::try_from_slice(value.issuer.as_slice())
                .map_err(|_| Error::Malformed)?,
            authority: value.authority.try_into()?,
            role: value.role.as_str().parse().map_err(|_| Error::Malformed)?,
            grants: ValidatedGrants::new(
                value
                    .grants
                    .into_vec()
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            )
            .map_err(|_| Error::Malformed)?,
            valid_from_ns: value.valid_from_ns,
            valid_until_ns: value.valid_until_ns,
            enrollment_until_ns: value.enrollment_until_ns,
            verification_burst: value.verification_burst,
            verification_refill_ns: value.verification_refill_ns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_auth_protocol_types::{
        CanonicalId, ChainKeyAlgorithm, ChainKeyKeyId, DelegatedRoleGrant,
    };
    use ic_auth_qualification_contracts::{InstallReceiver, model as wire};

    #[test]
    fn wire_projection_preserves_protected_authority_and_checked_identifiers() {
        let config = ReceiverConfig {
            audience: AudienceId {
                canonical_network_id: CanonicalId::from_bytes([1; 32]),
                fleet_id: CanonicalId::from_bytes([2; 32]),
            },
            issuer: Principal::from_slice(&[3]),
            authority: ReceiverAuthority::Ed25519([4; 32]),
            role: "user".parse().unwrap(),
            grants: ValidatedGrants::new(vec![DelegatedRoleGrant {
                target: "user".parse().unwrap(),
                scopes: vec!["read".into()],
            }])
            .unwrap(),
            valid_from_ns: 5,
            valid_until_ns: 6,
            enrollment_until_ns: 7,
            verification_burst: 8,
            verification_refill_ns: 9,
        };
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
                    Err(ic_wire::Error::Limit)
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
            malformed.grants = ic_wire::List::new(vec![grant.clone(), grant]).unwrap();
            assert_eq!(
                ReceiverConfig::try_from(malformed),
                Err(ic_wire::Error::Malformed)
            );
            let mut malformed = transport.clone();
            let mut grant = malformed.grants.as_slice()[0].clone();
            grant.scopes = ic_wire::List::new(vec![
                ic_wire::Text::new("write".into()).unwrap(),
                ic_wire::Text::new("read".into()).unwrap(),
            ])
            .unwrap();
            malformed.grants = ic_wire::List::new(vec![grant]).unwrap();
            assert_eq!(
                ReceiverConfig::try_from(malformed),
                Err(ic_wire::Error::Malformed)
            );
            let mut invalid = transport;
            invalid.role = ic_wire::Text::new("Invalid Role".into()).unwrap();
            assert!(ReceiverConfig::try_from(invalid).is_err());
        }
    }
}
