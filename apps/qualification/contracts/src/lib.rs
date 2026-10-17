//! Unpublished adapter for the existing Candid qualification and browser fixtures.
//! Public Auth libraries do not depend on this package. It is not a second auth
//! flow: it projects transport only, leaving validation and verification upstream.
use candid::{
    CandidType,
    types::{Type, TypeId},
};
use serde::{Deserialize, Serialize};
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
    type Model: CandidType;
    fn model(&self) -> Self::Model;
}
impl<T: FixtureModel> CandidType for Candid<T> {
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
        self.0.model().idl_serialize(serializer)
    }
}

impl FixtureModel for ic_auth_protocol_types::AudienceId {
    type Model = model::AudienceId;
    fn model(&self) -> Self::Model {
        model::AudienceId {
            canonical_network_id: self.canonical_network_id.to_string(),
            fleet_id: self.fleet_id.to_string(),
        }
    }
}
impl FixtureModel for ic_auth_protocol_types::DelegatedRoleGrant {
    type Model = model::DelegatedRoleGrant;
    fn model(&self) -> Self::Model {
        model::DelegatedRoleGrant {
            target: self.target.as_str().to_owned(),
            scopes: self.scopes.clone(),
        }
    }
}
impl FixtureModel for ic_auth_protocol_types::IcCanisterSignatureProofV1 {
    type Model = model::IcCanisterSignatureProofV1;
    fn model(&self) -> Self::Model {
        model::IcCanisterSignatureProofV1 {
            signature_cbor: self.signature_cbor.clone(),
            public_key_der: self.public_key_der.clone(),
        }
    }
}
impl FixtureModel for ic_auth_protocol_types::ChainKeyAlgorithm {
    type Model = model::ChainKeyAlgorithm;
    fn model(&self) -> Self::Model {
        match self {
            Self::EcdsaSecp256k1 => model::ChainKeyAlgorithm::EcdsaSecp256k1,
        }
    }
}
impl FixtureModel for ic_auth_protocol_types::ChainKeyKeyId {
    type Model = model::ChainKeyKeyId;
    fn model(&self) -> Self::Model {
        model::ChainKeyKeyId {
            name: self.name.clone(),
        }
    }
}
impl FixtureModel for ic_auth_protocol_types::CanonicalId {
    type Model = String;
    fn model(&self) -> Self::Model {
        self.to_string()
    }
}
impl FixtureModel for ic_auth_protocol_types::AuthRole {
    type Model = String;
    fn model(&self) -> Self::Model {
        self.as_str().to_owned()
    }
}
impl<T: FixtureModel> FixtureModel for Vec<T> {
    type Model = Vec<T::Model>;
    fn model(&self) -> Self::Model {
        self.iter().map(FixtureModel::model).collect()
    }
}
