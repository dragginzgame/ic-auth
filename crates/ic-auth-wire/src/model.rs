//! Bounded transport projections. Conversion is structural, never verification.
use ic_auth_protocol_types as domain;
use ic_auth_protocol_types::limits::{
    MAX_CLAIMS_BYTES, MAX_GRANTS, MAX_LABEL_BYTES, MAX_SCOPES_PER_GRANT,
};
use ic_wire::{Bytes, Error, List, Text};

trait Field<D>: Sized {
    fn from_domain(value: D) -> Result<Self, Error>;
    fn into_domain(self) -> Result<D, Error>;
}
macro_rules! scalar {
    ($($ty:ty),*) => {$(impl Field<$ty> for $ty {
        fn from_domain(value: $ty) -> Result<Self, Error> { Ok(value) }
        fn into_domain(self) -> Result<$ty, Error> { Ok(self) }
    })*};
}
scalar!(u16, u64, [u8; 16], [u8; 32], domain::Principal);
impl<const N: usize> Field<Vec<u8>> for Bytes<N> {
    fn from_domain(value: Vec<u8>) -> Result<Self, Error> {
        Self::new(value)
    }
    fn into_domain(self) -> Result<Vec<u8>, Error> {
        Ok(self.into_vec())
    }
}
impl<const N: usize> Field<String> for Text<N> {
    fn from_domain(value: String) -> Result<Self, Error> {
        Self::new(value)
    }
    fn into_domain(self) -> Result<String, Error> {
        Ok(self.into_string())
    }
}
impl<D, W: Field<D>, const N: usize> Field<Vec<D>> for List<W, N> {
    fn from_domain(value: Vec<D>) -> Result<Self, Error> {
        if value.len() > N {
            return Err(Error::Limit);
        }
        Self::new(
            value
                .into_iter()
                .map(W::from_domain)
                .collect::<Result<_, _>>()?,
        )
    }
    fn into_domain(self) -> Result<Vec<D>, Error> {
        self.into_vec().into_iter().map(W::into_domain).collect()
    }
}
impl Field<domain::CanonicalId> for [u8; 32] {
    fn from_domain(value: domain::CanonicalId) -> Result<Self, Error> {
        Ok(*value.as_bytes())
    }
    fn into_domain(self) -> Result<domain::CanonicalId, Error> {
        Ok(domain::CanonicalId::from_bytes(self))
    }
}
impl Field<domain::AuthRole> for Text<MAX_LABEL_BYTES> {
    fn from_domain(value: domain::AuthRole) -> Result<Self, Error> {
        Self::new(value.into_string())
    }
    fn into_domain(self) -> Result<domain::AuthRole, Error> {
        domain::AuthRole::try_from(self.into_string()).map_err(|_| Error::Malformed)
    }
}
// One declaration owns each Serde transport field and both typed
// projections. Rust checks the domain field names/types; no intermediate codec.
macro_rules! record {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
        pub struct $name { $(pub $field: $ty),* }
        impl TryFrom<domain::$name> for $name {
            type Error = Error;
            fn try_from(value: domain::$name) -> Result<Self, Error> {
                Ok(Self { $($field: Field::from_domain(value.$field)?),* })
            }
        }
        impl TryFrom<$name> for domain::$name {
            type Error = Error;
            fn try_from(value: $name) -> Result<Self, Error> {
                Ok(Self { $($field: Field::into_domain(value.$field)?),* })
            }
        }
        impl Field<domain::$name> for $name {
            fn from_domain(value: domain::$name) -> Result<Self, Error> { value.try_into() }
            fn into_domain(self) -> Result<domain::$name, Error> { self.try_into() }
        }
    };
}
macro_rules! unit_enum {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
        pub enum $name { $($variant),* }
        impl Field<domain::$name> for $name {
            fn from_domain(value: domain::$name) -> Result<Self, Error> {
                Ok(value.into())
            }
            fn into_domain(self) -> Result<domain::$name, Error> {
                Ok(self.into())
            }
        }
        impl From<domain::$name> for $name {
            fn from(value: domain::$name) -> Self {
                match value { $(domain::$name::$variant => Self::$variant),* }
            }
        }
        impl From<$name> for domain::$name {
            fn from(value: $name) -> Self {
                match value { $($name::$variant => Self::$variant),* }
            }
        }
    };
}
macro_rules! tuple_enum {
    ($name:ident { $($variant:ident($ty:ty)),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
        pub enum $name { $($variant($ty)),* }
        impl Field<domain::$name> for $name {
            fn from_domain(value: domain::$name) -> Result<Self, Error> {
                Ok(match value { $(domain::$name::$variant(v) => Self::$variant(Field::from_domain(v)?)),* })
            }
            fn into_domain(self) -> Result<domain::$name, Error> {
                Ok(match self { $(Self::$variant(v) => domain::$name::$variant(Field::into_domain(v)?)),* })
            }
        }
    };
}
record!(AudienceId {
    canonical_network_id: [u8; 32],
    fleet_id: [u8; 32]
});
record!(DelegatedRoleGrant { target: Text<MAX_LABEL_BYTES>, scopes: List<Text<MAX_LABEL_BYTES>, MAX_SCOPES_PER_GRANT> });
tuple_enum!(DelegationAudience {
    Fleet(AudienceId)
});
unit_enum!(ChainKeyAlgorithm { EcdsaSecp256k1 });
record!(ChainKeyKeyId { name: Text<128> });
unit_enum!(IssuerProofAlgorithm {
    IcCanisterSignatureV1
});
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
pub enum IssuerProofBinding {
    IcCanisterSignatureV1([u8; 32]),
}
impl Field<domain::IssuerProofBinding> for IssuerProofBinding {
    fn from_domain(value: domain::IssuerProofBinding) -> Result<Self, Error> {
        let domain::IssuerProofBinding::IcCanisterSignatureV1 { seed_hash } = value;
        Ok(Self::IcCanisterSignatureV1(seed_hash))
    }
    fn into_domain(self) -> Result<domain::IssuerProofBinding, Error> {
        let Self::IcCanisterSignatureV1(seed_hash) = self;
        Ok(domain::IssuerProofBinding::IcCanisterSignatureV1 { seed_hash })
    }
}
record!(IcCanisterSignatureProofV1 { signature_cbor: Bytes<131072>, public_key_der: Bytes<256> });
record!(ChainKeyBatchHeaderV1 {
    schema_version: u16,
    root_canister_id: domain::Principal,
    batch_id: [u8; 32],
    proof_epoch: u64,
    registry_epoch: u64,
    registry_hash: [u8; 32],
    tree_root: [u8; 32],
    not_before_ns: u64,
    expires_at_ns: u64,
    algorithm: ChainKeyAlgorithm,
    key_id: ChainKeyKeyId,
    derivation_path_hash: [u8; 32],
    key_version: u64,
});
record!(ChainKeyDelegationCertV1 {
    root_canister_id: domain::Principal, issuer_canister_id: domain::Principal, proof_epoch: u64,
    issuer_proof_algorithm: IssuerProofAlgorithm, issuer_proof_binding_hash: [u8; 32],
    issuer_proof_binding: IssuerProofBinding, max_token_ttl_ns: u64,
    audience: DelegationAudience, grants: List<DelegatedRoleGrant, MAX_GRANTS>,
    not_before_ns: u64, expires_at_ns: u64, registry_epoch: u64, registry_hash: [u8; 32],
});
record!(ChainKeyRootSignatureV1 {
    algorithm: ChainKeyAlgorithm, key_id: ChainKeyKeyId,
    derivation_path: List<Bytes<256>, 32>, public_key: Bytes<65>, signature: Bytes<64>,
});
tuple_enum!(ChainKeyBatchWitnessStepV1 {
    LeftSibling([u8; 32]),
    RightSibling([u8; 32])
});
record!(ChainKeyBatchWitnessV1 { steps: List<ChainKeyBatchWitnessStepV1, 64> });
record!(IcChainKeyBatchSignatureProofV1 {
    header: ChainKeyBatchHeaderV1,
    delegation_cert: ChainKeyDelegationCertV1,
    issuer_witness: ChainKeyBatchWitnessV1,
    signature: ChainKeyRootSignatureV1,
});
tuple_enum!(RootProof {
    IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1)
});
record!(DelegationCert {
    root_pid: domain::Principal, issuer_pid: domain::Principal, issuer_proof_alg: IssuerProofAlgorithm,
    issuer_proof_binding_hash: [u8; 32], issuer_proof_binding: IssuerProofBinding,
    issued_at_ns: u64, not_before_ns: u64, expires_at_ns: u64,
    max_token_ttl_ns: u64, aud: DelegationAudience, grants: List<DelegatedRoleGrant, MAX_GRANTS>,
});
record!(DelegationProof {
    cert: DelegationCert,
    root_proof: RootProof
});
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
pub struct CertifiedProof {
    pub delegation: DelegationProof,
    pub issuer_signature: IcCanisterSignatureProofV1,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
pub enum PortableProof {
    Certified(Box<CertifiedProof>),
    Ed25519(Bytes<64>),
}
impl Field<domain::PortableProof> for PortableProof {
    fn from_domain(value: domain::PortableProof) -> Result<Self, Error> {
        Ok(match value {
            domain::PortableProof::Certified {
                delegation,
                issuer_signature,
            } => Self::Certified(Box::new(CertifiedProof {
                delegation: (*delegation).try_into()?,
                issuer_signature: issuer_signature.try_into()?,
            })),
            domain::PortableProof::Ed25519 { signature } => Self::Ed25519(Bytes::new(signature)?),
        })
    }
    fn into_domain(self) -> Result<domain::PortableProof, Error> {
        Ok(match self {
            Self::Certified(value) => domain::PortableProof::Certified {
                delegation: Box::new(value.delegation.try_into()?),
                issuer_signature: value.issuer_signature.try_into()?,
            },
            Self::Ed25519(signature) => domain::PortableProof::Ed25519 {
                signature: signature.into_vec(),
            },
        })
    }
}
record!(PortableCredential { claims: Bytes<MAX_CLAIMS_BYTES>, proof: PortableProof });
// Admission diagnostics are independent of proof validity and retry policy.
unit_enum!(AdmissionRefusal {
    NotYetValid,
    Expired,
    InvalidCredential,
    AuthorityUnavailable,
    BeforeReset,
    ScopeRejected,
    ReplayConflict,
    Revoked,
    Capacity,
    Throttled,
    Unavailable
});
unit_enum!(LogoutUnavailable {
    Capacity,
    Storage,
    Clock,
    ResetPending
});
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
pub enum LogoutResult {
    Revoked(u64),
    InvalidCaller,
    Unavailable(LogoutUnavailable),
}
impl From<domain::LogoutResult> for LogoutResult {
    fn from(value: domain::LogoutResult) -> Self {
        match value {
            domain::LogoutResult::Revoked { deny_until_ns } => Self::Revoked(deny_until_ns),
            domain::LogoutResult::InvalidCaller => Self::InvalidCaller,
            domain::LogoutResult::Unavailable(reason) => Self::Unavailable(reason.into()),
        }
    }
}
unit_enum!(SessionReadRefusal {
    MissingOrExpired,
    AuthorityUnavailable,
    ScopeRejected,
    Revoked,
    Unavailable
});
