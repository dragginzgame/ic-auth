//! Private diagnostic probes, never an alternate verifier or admission path.
//! Complete verification must succeed before independently repeating components.
//! Their timings overlap; only the complete-verifier timing is end-to-end work.
use super::*;
use controls::model::CertifiedProfileFailure as Failure;
use ic_auth::{
    canister_signature::{
        CanisterSignaturePolicy, domain_separated_message, verify_canister_signature,
    },
    delegation::{DelegationProofVerificationContext, verify_delegation_proof},
    portable_certified::{
        CERTIFIED_SIGNATURE_DOMAIN, PortableCertifiedContext, verify_certified_claims,
    },
    session::SessionError,
};
use ic_auth_protocol_types::{IssuerProofBinding, PortableProof};
use ic_platform::{
    bls::verify_bls_signature,
    canister_sig_creation::extract_raw_root_pk_from_der,
    certification::{Certificate, HashTree, LookupResult},
};
use serde::Deserialize;

impl From<ReceiverUnavailable> for Failure {
    fn from(_: ReceiverUnavailable) -> Self {
        Self::ReceiverUnavailable
    }
}

fn measure<T>(operation: impl FnOnce() -> T) -> (T, u64) {
    let start = ic_platform::ic0::performance_counter(0);
    let value = operation();
    (value, ic_platform::ic0::performance_counter(0) - start)
}

#[derive(Deserialize)]
struct Envelope {
    certificate: serde_bytes::ByteBuf,
    tree: HashTree,
}

fn bls_message(hash: [u8; 32]) -> Vec<u8> {
    // Exact IC state-root framing consumed by Platform's BLS primitive.
    [b"\x0Dic-state-root".as_slice(), &hash].concat()
}

#[ic_platform::cdk::update(
    guard = "super::admission_guard",
    decode_with = "crate::control::ProfileCertified::decode",
    encode_with = "crate::control::ProfileCertified::encode"
)]
fn profile_certified_proof(
    input: SessionInput,
) -> Result<controls::model::CertifiedProfile, Failure> {
    let credential: PortableCredential = input
        .credential
        .try_into()
        .map_err(|_| Failure::Credential)?;
    with_receiver(|host| profile(host, &credential))
}

fn profile(
    host: &mut Receiver,
    credential: &PortableCredential,
) -> Result<controls::model::CertifiedProfile, Failure> {
    let ReceiverAuthority::Certified {
        root, network_key, ..
    } = &host.inputs.config.authority
    else {
        return Err(Failure::CertifiedReceiverRequired);
    };
    let context = PortableCertifiedContext {
        claims: host.inputs.claims_context(
            ic_platform::cdk::api::msg_caller(),
            ic_platform::cdk::api::time(),
        ),
        root_key: root,
        ic_root_public_key_raw: network_key,
        limits: CERTIFIED_LIMITS,
    };
    let PortableProof::Certified {
        delegation,
        issuer_signature,
    } = &credential.proof
    else {
        return Err(Failure::CertifiedCredentialRequired);
    };
    let (verified, credential_instructions) = measure(|| {
        verify_certified_claims(
            &mut host.admission,
            &credential.claims,
            delegation,
            issuer_signature,
            &context,
        )
    });
    let verified = verified.map_err(|e| {
        Failure::Verification(
            SessionError::from(ic_auth::portable_credential::PortableCredentialError::from(
                e,
            ))
            .admission_refusal()
            .into(),
        )
    })?;
    // Every probe below consumes the very same, fully verified proof. No
    // timings or partial results are returned for a failed complete check.
    let (root_result, root_delegation_instructions) = measure(|| {
        verify_delegation_proof(
            delegation,
            &DelegationProofVerificationContext {
                expected_issuer: context.claims.authority.issuer,
                root_key: context.root_key,
                now_ns: context.claims.authority.now_ns,
                limits: context.limits.into(),
            },
        )
    });
    root_result.map_err(|_| Failure::RootDelegation)?;
    let IssuerProofBinding::IcCanisterSignatureV1 { seed_hash } =
        delegation.cert.issuer_proof_binding;
    let message = domain_separated_message(CERTIFIED_SIGNATURE_DOMAIN, verified.fingerprint())
        .map_err(|_| Failure::MessageFraming)?;
    let (issuer_result, issuer_instructions) = measure(|| {
        verify_canister_signature(
            &message,
            issuer_signature,
            &CanisterSignaturePolicy {
                signing_canister: context.claims.authority.issuer,
                seed_hash,
                ic_root_public_key_raw: network_key,
                now_ns: context.claims.authority.now_ns,
                max_certificate_age_ns: context.limits.max_certificate_age_ns,
                max_future_skew_ns: 0,
                max_signature_bytes: context.limits.max_issuer_signature_bytes,
                max_message_bytes: message.len(),
            },
        )
    });
    issuer_result.map_err(|_| Failure::IssuerSignature)?;

    let (decoded, decode_instructions) = measure(|| -> Result<_, serde_cbor::Error> {
        let envelope: Envelope = serde_cbor::from_slice(&issuer_signature.signature_cbor)?;
        let certificate: Certificate = serde_cbor::from_slice(&envelope.certificate)?;
        let subnet = certificate
            .delegation
            .as_ref()
            .map(|d| serde_cbor::from_slice::<Certificate>(&d.certificate))
            .transpose()?;
        Ok((envelope, certificate, subnet))
    });
    let (envelope, certificate, subnet) = decoded.map_err(|_| Failure::CertificateDecode)?;
    let ((signing_hash, subnet_hash), tree_hash_instructions) = measure(|| {
        // Keep witness hashing in the probe even though the complete
        // verifier has already checked its certified-data binding.
        std::hint::black_box(envelope.tree.digest());
        (
            certificate.tree.digest(),
            subnet.as_ref().map(|cert| cert.tree.digest()),
        )
    });
    let mut subnet_bls_instructions = 0;
    let mut subnet_certificate_bytes = 0;
    let signing_key = if let (Some(subnet), Some(hash), Some(delegated)) =
        (&subnet, subnet_hash, &certificate.delegation)
    {
        subnet_certificate_bytes = delegated.certificate.len() as u64;
        let message = bls_message(hash);
        let (result, instructions) =
            measure(|| verify_bls_signature(&subnet.signature, &message, network_key));
        result.map_err(|_| Failure::SubnetBls)?;
        subnet_bls_instructions = instructions;
        // Evidence extraction after full verification, not a substitute
        // for Platform's delegation, ranges, key or certificate checks.
        let LookupResult::Found(der) = subnet.tree.lookup_path([
            b"subnet".as_slice(),
            delegated.subnet_id.as_ref(),
            b"public_key".as_slice(),
        ]) else {
            return Err(Failure::SubnetKeyMissing);
        };
        extract_raw_root_pk_from_der(der).map_err(|_| Failure::SubnetKeyDecode)?
    } else {
        network_key.clone()
    };
    let signing_message = bls_message(signing_hash);
    let (result, signing_bls_instructions) =
        measure(|| verify_bls_signature(&certificate.signature, &signing_message, &signing_key));
    result.map_err(|_| Failure::SigningBls)?;
    Ok(controls::model::CertifiedProfile {
        credential_instructions,
        root_delegation_instructions,
        issuer_instructions,
        decode_instructions,
        tree_hash_instructions,
        signing_bls_instructions,
        subnet_bls_instructions,
        signature_bytes: issuer_signature.signature_cbor.len() as u64,
        certificate_bytes: envelope.certificate.len() as u64,
        subnet_certificate_bytes,
    })
}
