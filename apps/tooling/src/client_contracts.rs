//! Generate data contracts from their Rust owner; this declares no endpoints.
use candid::{Decode, Encode, pretty::candid::compile, types::internal::TypeContainer};
use ic_auth_protocol_types::{
    AudienceId, AuthRequestMetadata, CanonicalId, DelegatedRoleGrant, DelegatedToken,
    DelegatedTokenGetRequest, DelegatedTokenPrepareRequest, DelegatedTokenPrepareResponse,
    DelegationAudience, LogoutRequest, LogoutResult, LogoutUnavailable, PortableCredential,
    PortableProof,
};
use ic_auth_qualification_contracts::Candid;

fn request_vector() -> DelegatedTokenPrepareRequest {
    DelegatedTokenPrepareRequest {
        metadata: Some(AuthRequestMetadata {
            request_id: [7; 32],
            ttl_ns: 1_000_000_000,
        }),
        aud: DelegationAudience::Fleet(AudienceId {
            canonical_network_id: CanonicalId::from_bytes([0xaa; 32]),
            fleet_id: CanonicalId::from_bytes([0xbb; 32]),
        }),
        grants: vec![DelegatedRoleGrant {
            target: "users".parse().expect("canonical role"),
            scopes: vec!["read".into()],
        }],
        ttl_ns: 9_007_199_254_740_993,
        ext: Some(vec![0, 255]),
    }
}

// Deliberately opaque transport bytes, not a valid signed credential.
fn credential_vector() -> PortableCredential {
    PortableCredential {
        claims: vec![0, 1, 127, 128, 255],
        proof: PortableProof::Ed25519 {
            signature: (0..64).collect(),
        },
    }
}

fn logout_vector() -> Vec<LogoutResult> {
    vec![
        LogoutResult::Revoked {
            deny_until_ns: 9_007_199_254_740_993,
        },
        LogoutResult::InvalidCaller,
        LogoutResult::Unavailable(LogoutUnavailable::Capacity),
        LogoutResult::Unavailable(LogoutUnavailable::Storage),
        LogoutResult::Unavailable(LogoutUnavailable::Clock),
        LogoutResult::Unavailable(LogoutUnavailable::ResetPending),
    ]
}

pub(crate) fn run(
    mode: &str,
    directory: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use ic_host_fs::{
        durable::{PublicationMode, WriteOptions, write_with},
        read::read_file_no_follow,
    };
    use std::io::Write;
    let save = |name: &str, bytes: &[u8]| -> Result<(), Box<dyn std::error::Error>> {
        write_with(
            &directory.join(name),
            WriteOptions {
                mode: PublicationMode::Replace,
                permissions: 0o600,
            },
            |file| file.write_all(bytes),
        )?;
        Ok(())
    };
    match mode {
        "generate" => {
            let mut types = TypeContainer::new();
            types.add::<Candid<DelegatedToken>>();
            types.add::<Candid<PortableCredential>>();
            // Candid elides names for empty records; retain the public request alias.
            let logout_request = types.add::<Candid<LogoutRequest>>();
            types.env.0.insert("LogoutRequest".into(), logout_request);
            types.add::<Candid<LogoutResult>>();
            types.add::<Candid<DelegatedTokenGetRequest>>();
            types.add::<Candid<DelegatedTokenPrepareRequest>>();
            types.add::<Candid<DelegatedTokenPrepareResponse>>();
            save("protocol.did", compile(&types.env, &None).as_bytes())?;
            for (name, bytes) in [
                ("request-vector.hex", Encode!(&Candid(request_vector()))?),
                (
                    "credential-vector.hex",
                    Encode!(&Candid(credential_vector()))?,
                ),
                ("logout-vector.hex", Encode!(&Candid(logout_vector()))?),
            ] {
                let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
                save(name, format!("{hex}\n").as_bytes())?;
            }
        }
        "check" => {
            let bytes = read_file_no_follow(&directory.join("request-from-ts.bin"), 131_072)?;
            if Decode!(&bytes, Candid<DelegatedTokenPrepareRequest>)?.0 != request_vector() {
                return Err("request vector mismatch".into());
            }
            let bytes = read_file_no_follow(&directory.join("credential-from-ts.bin"), 131_072)?;
            if Decode!(&bytes, Candid<PortableCredential>)?.0 != credential_vector() {
                return Err("credential vector mismatch".into());
            }
            let bytes = read_file_no_follow(&directory.join("logout-from-ts.bin"), 131_072)?;
            if Decode!(&bytes, Candid<Vec<LogoutResult>>)?.0 != logout_vector() {
                return Err("logout vector mismatch".into());
            }
        }
        _ => return Err("client-contracts requires generate|check DIRECTORY".into()),
    }
    Ok(())
}
