//! Passive authentication values. Valid shape never establishes authority.
//!
//! Labels and claim fields are unauthenticated data. Canonical encoding belongs
//! to Auth; transport, proof verification, host policy and persistence are separate.

mod claims;
mod credential;
mod delegation;
mod role;
mod scope;

pub use claims::{
    AudienceId, CanonicalId, DelegatedRoleGrant, MAX_TOKEN_EXT_BYTES, PortableClaims,
    PortableMechanism,
};
pub use credential::{
    CertifiedProofMaterial, Credential, CredentialProof, RootDelegation, RootSignatureMaterial,
};
pub use delegation::{
    DelegationCertificate, DelegationLeaf, IssuerDelegation, MerkleStep, RootBatchHeader,
};
pub use ic_platform_types::Principal;
pub use role::{AuthRole, RoleError};
pub use scope::{ScopeLabel, ScopeLabelError};
