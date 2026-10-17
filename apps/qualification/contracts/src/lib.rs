//! Unpublished adapter for the existing Candid qualification and browser fixtures.
//! Public Auth libraries do not depend on this package. It is not a second auth
//! flow: it projects transport only, leaving validation and verification upstream.
use candid::{
    CandidType,
    types::{Type, TypeId},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned, ser::Error};
use std::ops::Deref;

pub mod model;

/// The domain value is deserialized directly, preserving checked identifier rules.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct Candid<T>(pub T);

impl<T> Deref for Candid<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T> From<T> for Candid<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Only the explicitly enumerated fixture contracts have a transport projection.
pub trait FixtureModel {
    type Model: CandidType + DeserializeOwned;
}
impl<T: FixtureModel + Serialize> CandidType for Candid<T> {
    fn id() -> TypeId {
        T::Model::id()
    }
    fn _ty() -> Type {
        T::Model::_ty()
    }
    fn _ty_doc() -> candid::types::internal::TypeDoc {
        T::Model::_ty_doc()
    }
    fn idl_serialize<S: candid::types::Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        // Internal output projection only. Incoming Candid decodes directly into
        // T; it never bypasses domain validation through an unchecked DTO cast.
        let bytes = serde_cbor::to_vec(&self.0).map_err(S::Error::custom)?;
        let model: T::Model = serde_cbor::from_slice(&bytes).map_err(S::Error::custom)?;
        model.idl_serialize(serializer)
    }
}
macro_rules! fixture_models {
    ($($name:ident),* $(,)?) => {$(
        impl FixtureModel for ic_auth_protocol_types::$name { type Model = model::$name; }
    )*};
}
fixture_models!(
    DelegationAudience,
    AudienceId,
    DelegatedRoleGrant,
    AuthRequestMetadata,
    DelegatedTokenPrepareRequest,
    DelegatedTokenClaims,
    DelegatedTokenPrepareResponse,
    DelegatedTokenGetRequest,
    RootProof,
    IssuerProof,
    IcCanisterSignatureProofV1,
    ChainKeyAlgorithm,
    ChainKeyKeyId,
    IcChainKeyBatchSignatureProofV1,
    ChainKeyBatchHeaderV1,
    ChainKeyDelegationCertV1,
    ChainKeyRootSignatureV1,
    ChainKeyBatchWitnessV1,
    ChainKeyBatchWitnessStepV1,
    IssuerProofAlgorithm,
    IssuerProofBinding,
    DelegationCert,
    DelegationProof,
    DelegatedToken,
    PortableCredential,
    PortableProof,
    LogoutRequest,
    LogoutResult,
    LogoutUnavailable,
);
impl FixtureModel for ic_auth_protocol_types::CanonicalId {
    type Model = String;
}
impl FixtureModel for ic_auth_protocol_types::AuthRole {
    type Model = String;
}

impl<T: FixtureModel> FixtureModel for Vec<T> {
    type Model = Vec<T::Model>;
}
