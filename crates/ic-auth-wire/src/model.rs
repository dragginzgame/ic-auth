//! Bounded transport projections. Conversion is structural, never verification.
use ic_auth_protocol_types as domain;
use ic_wire::{Bytes, Error, List, Text, Wire};

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
scalar!(u16, u64, [u8; 16], [u8; 32]);
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
impl Field<domain::Principal> for Bytes<29> {
    fn from_domain(value: domain::Principal) -> Result<Self, Error> {
        Self::new(value.as_slice().to_vec())
    }
    fn into_domain(self) -> Result<domain::Principal, Error> {
        domain::Principal::try_from_slice(self.as_slice()).map_err(|_| Error::Malformed)
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
impl Field<domain::AuthRole> for Text<64> {
    fn from_domain(value: domain::AuthRole) -> Result<Self, Error> {
        Self::new(value.as_str().to_owned())
    }
    fn into_domain(self) -> Result<domain::AuthRole, Error> {
        self.as_str().parse().map_err(|_| Error::Malformed)
    }
}
// One declaration owns each transport field, its explicit ID and both typed
// projections. Rust checks the domain field names/types; no intermediate codec.
macro_rules! record {
    ($name:ident { $($id:literal: $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq, Wire)]
        pub struct $name { $(#[wire(id = $id)] pub $field: $ty),* }
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
    ($name:ident { $($id:literal: $variant:ident),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq, Wire)]
        pub enum $name { $(#[wire(id = $id)] $variant),* }
        impl Field<domain::$name> for $name {
            fn from_domain(value: domain::$name) -> Result<Self, Error> {
                Ok(match value { $(domain::$name::$variant => Self::$variant),* })
            }
            fn into_domain(self) -> Result<domain::$name, Error> {
                Ok(match self { $(Self::$variant => domain::$name::$variant),* })
            }
        }
    };
}
macro_rules! tuple_enum {
    ($name:ident { $($id:literal: $variant:ident($ty:ty)),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq, Wire)]
        pub enum $name { $(#[wire(id = $id)] $variant($ty)),* }
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
record!(AudienceId { 1: canonical_network_id: [u8; 32], 2: fleet_id: [u8; 32] });
record!(DelegatedRoleGrant { 1: target: Text<64>, 2: scopes: List<Text<64>, 32> });
tuple_enum!(DelegationAudience {
    0: Fleet(AudienceId)
});
unit_enum!(ChainKeyAlgorithm { 0: EcdsaSecp256k1 });
record!(ChainKeyKeyId { 1: name: Text<128> });
unit_enum!(IssuerProofAlgorithm {
    0: IcCanisterSignatureV1
});
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub enum IssuerProofBinding {
    #[wire(id = 0)]
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
record!(IcCanisterSignatureProofV1 { 1: signature_cbor: Bytes<131072>, 2: public_key_der: Bytes<256> });
record!(ChainKeyBatchHeaderV1 {
    1: schema_version: u16, 2: root_canister_id: Bytes<29>, 3: batch_id: [u8; 32],
    4: proof_epoch: u64, 5: registry_epoch: u64, 6: registry_hash: [u8; 32],
    7: tree_root: [u8; 32], 8: not_before_ns: u64, 9: expires_at_ns: u64,
    10: algorithm: ChainKeyAlgorithm, 11: key_id: ChainKeyKeyId,
    12: derivation_path_hash: [u8; 32], 13: key_version: u64,
});
record!(ChainKeyDelegationCertV1 {
    1: root_canister_id: Bytes<29>, 2: issuer_canister_id: Bytes<29>, 3: proof_epoch: u64,
    4: issuer_proof_algorithm: IssuerProofAlgorithm, 5: issuer_proof_binding_hash: [u8; 32],
    6: issuer_proof_binding: IssuerProofBinding, 7: max_token_ttl_ns: u64,
    8: audience: DelegationAudience, 9: grants: List<DelegatedRoleGrant, 16>,
    10: not_before_ns: u64, 11: expires_at_ns: u64, 12: registry_epoch: u64, 13: registry_hash: [u8; 32],
});
record!(ChainKeyRootSignatureV1 {
    1: algorithm: ChainKeyAlgorithm, 2: key_id: ChainKeyKeyId,
    3: derivation_path: List<Bytes<256>, 32>, 4: public_key: Bytes<65>, 5: signature: Bytes<64>,
});
tuple_enum!(ChainKeyBatchWitnessStepV1 {
    0: LeftSibling([u8; 32]),
    1: RightSibling([u8; 32])
});
record!(ChainKeyBatchWitnessV1 { 1: steps: List<ChainKeyBatchWitnessStepV1, 64> });
record!(IcChainKeyBatchSignatureProofV1 {
    1: header: ChainKeyBatchHeaderV1, 2: delegation_cert: ChainKeyDelegationCertV1,
    3: issuer_witness: ChainKeyBatchWitnessV1, 4: signature: ChainKeyRootSignatureV1,
});
tuple_enum!(RootProof {
    0: IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1)
});
record!(DelegationCert {
    1: root_pid: Bytes<29>, 2: issuer_pid: Bytes<29>, 3: issuer_proof_alg: IssuerProofAlgorithm,
    4: issuer_proof_binding_hash: [u8; 32], 5: issuer_proof_binding: IssuerProofBinding,
    6: issued_at_ns: u64, 7: not_before_ns: u64, 8: expires_at_ns: u64,
    9: max_token_ttl_ns: u64, 10: aud: DelegationAudience, 11: grants: List<DelegatedRoleGrant, 16>,
});
record!(DelegationProof { 1: cert: DelegationCert, 2: root_proof: RootProof });
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub struct CertifiedProof {
    #[wire(id = 1)]
    pub delegation: DelegationProof,
    #[wire(id = 2)]
    pub issuer_signature: IcCanisterSignatureProofV1,
}
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
#[allow(
    clippy::large_enum_variant,
    reason = "The upstream derive has no Box shape; retain its bounded canonical schema instead of adding a private codec"
)]
pub enum PortableProof {
    #[wire(id = 0)]
    Certified(CertifiedProof),
    #[wire(id = 1)]
    Ed25519(Bytes<64>),
}
impl Field<domain::PortableProof> for PortableProof {
    fn from_domain(value: domain::PortableProof) -> Result<Self, Error> {
        Ok(match value {
            domain::PortableProof::Certified {
                delegation,
                issuer_signature,
            } => Self::Certified(CertifiedProof {
                delegation: (*delegation).try_into()?,
                issuer_signature: issuer_signature.try_into()?,
            }),
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
record!(PortableCredential { 1: claims: Bytes<65536>, 2: proof: PortableProof });
// Admission diagnostics are independent of proof validity and retry policy.
unit_enum!(AdmissionRefusal {
    1: NotYetValid,
    2: Expired,
    3: InvalidCredential,
    4: AuthorityUnavailable,
    5: BeforeReset,
    6: ScopeRejected,
    7: ReplayConflict,
    8: Revoked,
    9: Capacity,
    10: Throttled,
    11: Unavailable
});
impl From<domain::AdmissionRefusal> for AdmissionRefusal {
    fn from(value: domain::AdmissionRefusal) -> Self {
        use domain::AdmissionRefusal as R;
        match value {
            R::NotYetValid => Self::NotYetValid,
            R::Expired => Self::Expired,
            R::InvalidCredential => Self::InvalidCredential,
            R::AuthorityUnavailable => Self::AuthorityUnavailable,
            R::BeforeReset => Self::BeforeReset,
            R::ScopeRejected => Self::ScopeRejected,
            R::ReplayConflict => Self::ReplayConflict,
            R::Revoked => Self::Revoked,
            R::Capacity => Self::Capacity,
            R::Throttled => Self::Throttled,
            R::Unavailable => Self::Unavailable,
        }
    }
}
unit_enum!(LogoutUnavailable {
    0: Capacity,
    1: Storage,
    2: Clock,
    3: ResetPending
});
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub enum LogoutResult {
    #[wire(id = 0)]
    Revoked(u64),
    #[wire(id = 1)]
    InvalidCaller,
    #[wire(id = 2)]
    Unavailable(LogoutUnavailable),
}
impl From<domain::LogoutResult> for LogoutResult {
    fn from(value: domain::LogoutResult) -> Self {
        match value {
            domain::LogoutResult::Revoked { deny_until_ns } => Self::Revoked(deny_until_ns),
            domain::LogoutResult::InvalidCaller => Self::InvalidCaller,
            domain::LogoutResult::Unavailable(reason) => Self::Unavailable(match reason {
                domain::LogoutUnavailable::Capacity => LogoutUnavailable::Capacity,
                domain::LogoutUnavailable::Storage => LogoutUnavailable::Storage,
                domain::LogoutUnavailable::Clock => LogoutUnavailable::Clock,
                domain::LogoutUnavailable::ResetPending => LogoutUnavailable::ResetPending,
            }),
        }
    }
}
