//! Private controller-installed portable-session qualification contracts.
use ic_auth::{delegation::RootKeyPolicy, portable::ValidatedGrants};
use ic_auth_protocol_types::{AudienceId, AuthRole, Principal};
pub use ic_auth_qualification_contracts::model::{AdmissionProfile, SessionProfile};
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
    Ed25519(Box<ic_auth::portable_signature::PreparedIssuerKey>),
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
        match self {
            Self::Ed25519(key) => key.binding(),
            Self::Certified { binding, .. } => *binding,
        }
    }
}

// The private enrollment projection shares its unchanged field roster. Key
// preparation remains explicit at the receiver authority boundary below.
macro_rules! root_projection {
    ($($field:ident),* $(,)?) => {
        fn root_to_wire(root: RootKeyPolicy) -> Result<ic_auth_qualification_contracts::model::RootEnrollment, ic_auth_wire::wire::Error> {
            Ok(ic_auth_qualification_contracts::model::RootEnrollment {
                $($field: root.$field,)*
                algorithm: root.algorithm.into(),
                key_name: ic_auth_wire::wire::Text::new(root.key_id.name)?,
                public_key: ic_auth_wire::wire::Bytes::new(root.public_key)?,
            })
        }
        fn root_from_wire(root: ic_auth_qualification_contracts::model::RootEnrollment) -> RootKeyPolicy {
            RootKeyPolicy {
                $($field: root.$field,)*
                algorithm: root.algorithm.into(),
                key_id: ic_auth_protocol_types::ChainKeyKeyId { name: root.key_name.into_string() },
                public_key: root.public_key.into_vec(),
            }
        }
    };
}
root_projection!(
    root_canister_id,
    derivation_path_hash,
    key_version,
    min_accepted_key_version,
    min_accepted_proof_epoch,
    min_accepted_registry_epoch,
    valid_from_ns,
    accept_until_ns,
    max_revocation_latency_ns,
);

impl TryFrom<ReceiverAuthority> for ic_auth_qualification_contracts::model::ReceiverAuthority {
    type Error = ic_auth_wire::wire::Error;
    fn try_from(value: ReceiverAuthority) -> Result<Self, Self::Error> {
        use ic_auth_qualification_contracts::model as w;
        use ic_auth_wire::wire::Bytes;
        Ok(match value {
            ReceiverAuthority::Ed25519(key) => w::ReceiverAuthority::Ed25519(*key.bytes()),
            ReceiverAuthority::Certified {
                binding,
                root,
                network_key,
            } => w::ReceiverAuthority::Certified(w::CertifiedAuthority {
                binding,
                root: root_to_wire(*root)?,
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
        Ok(match value {
            w::ReceiverAuthority::Ed25519(key) => ReceiverAuthority::Ed25519(Box::new(
                ic_auth::portable_signature::PreparedIssuerKey::new(key)
                    .map_err(|_| ic_auth_wire::wire::Error::Malformed)?,
            )),
            w::ReceiverAuthority::Certified(w::CertifiedAuthority {
                binding,
                root,
                network_key,
            }) => ReceiverAuthority::Certified {
                binding,
                root: Box::new(root_from_wire(root)),
                network_key: network_key.into_vec(),
            },
        })
    }
}

impl TryFrom<ReceiverConfig> for ic_auth_qualification_contracts::model::ReceiverConfig {
    type Error = ic_auth_wire::wire::Error;
    fn try_from(value: ReceiverConfig) -> Result<Self, Self::Error> {
        use ic_auth_wire::wire::{List, Text};
        Ok(Self {
            audience: value.audience.try_into()?,
            issuer: value.issuer,
            authority: value.authority.try_into()?,
            role: Text::new(value.role.into_string())?,
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
            issuer: value.issuer,
            authority: value.authority.try_into()?,
            role: AuthRole::try_from(value.role.into_string()).map_err(|_| Error::Malformed)?,
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
mod tests;
