//! Generate bounded Rust-owned IC Wire contracts; no endpoint is installed.
use ic_auth_wire::{CredentialEnvelope, EstablishSession, Logout, ReadSession, model, wire};
use wire::{Bytes, Method};

fn credential() -> model::PortableCredential {
    model::PortableCredential {
        claims: Bytes::new(vec![0, 1, 127, 128, 255]).unwrap(),
        proof: model::PortableProof::Ed25519(Bytes::new((0..64).collect()).unwrap()),
    }
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
            for (name, source) in [
                (
                    "credential.ts",
                    wire::typescript::generate::<CredentialEnvelope>()
                        .map_err(|error| format!("IC Wire generation refused: {error:?}"))?,
                ),
                (
                    "establish.ts",
                    wire::typescript::generate::<EstablishSession>()
                        .map_err(|error| format!("IC Wire generation refused: {error:?}"))?,
                ),
                (
                    "read.ts",
                    wire::typescript::generate::<ReadSession>()
                        .map_err(|error| format!("IC Wire generation refused: {error:?}"))?,
                ),
                (
                    "logout.ts",
                    wire::typescript::generate::<Logout>()
                        .map_err(|error| format!("IC Wire generation refused: {error:?}"))?,
                ),
            ] {
                save(name, source.as_bytes())?;
            }
            save("wire-runtime.ts", wire::typescript::RUNTIME.as_bytes())?;
            for (name, bytes) in [
                (
                    "credential-vector.hex",
                    wire::encode_request::<CredentialEnvelope>(&credential())?.into_bytes(),
                ),
                (
                    "logout-vector.hex",
                    wire::encode_response::<Logout>(&model::LogoutResult::Revoked(
                        9_007_199_254_740_993,
                    ))?
                    .into_bytes(),
                ),
            ] {
                let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
                save(name, format!("{hex}\n").as_bytes())?;
            }
        }
        "check" => {
            let bytes = read_file_no_follow(
                &directory.join("credential-from-ts.bin"),
                CredentialEnvelope::REQUEST_LIMITS.bytes,
            )?;
            if wire::decode_request::<CredentialEnvelope>(&bytes)? != credential() {
                return Err("credential vector mismatch".into());
            }
            let bytes = read_file_no_follow(
                &directory.join("logout-from-ts.bin"),
                Logout::RESPONSE_LIMITS.bytes,
            )?;
            if wire::decode_response::<Logout>(&bytes)?
                != model::LogoutResult::Revoked(9_007_199_254_740_993)
            {
                return Err("logout vector mismatch".into());
            }
        }
        _ => return Err("client-contracts requires generate|check DIRECTORY".into()),
    }
    Ok(())
}
