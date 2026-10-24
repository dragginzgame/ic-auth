//! Generate bounded Rust-owned IC Wire contracts; no endpoint is installed.
use ic_auth_wire::{CredentialEnvelope, EstablishSession, Logout, ReadSession, model, wire};
use wire::Bytes;

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
                    generate(
                        CredentialEnvelope::NAME,
                        CredentialEnvelope::ID,
                        CredentialEnvelope::REQUEST_LIMITS,
                        CredentialEnvelope::RESPONSE_LIMITS,
                        CredentialEnvelope::encode_request,
                        CredentialEnvelope::decode_response,
                    )
                    .map_err(|error| format!("IC Wire generation refused: {error:?}"))?,
                ),
                (
                    "establish.ts",
                    generate(
                        EstablishSession::NAME,
                        EstablishSession::ID,
                        EstablishSession::REQUEST_LIMITS,
                        EstablishSession::RESPONSE_LIMITS,
                        EstablishSession::encode_request,
                        EstablishSession::decode_response,
                    )
                    .map_err(|error| format!("IC Wire generation refused: {error:?}"))?,
                ),
                (
                    "read.ts",
                    generate(
                        ReadSession::NAME,
                        ReadSession::ID,
                        ReadSession::REQUEST_LIMITS,
                        ReadSession::RESPONSE_LIMITS,
                        ReadSession::encode_request,
                        ReadSession::decode_response,
                    )
                    .map_err(|error| format!("IC Wire generation refused: {error:?}"))?,
                ),
                (
                    "logout.ts",
                    generate(
                        Logout::NAME,
                        Logout::ID,
                        Logout::REQUEST_LIMITS,
                        Logout::RESPONSE_LIMITS,
                        Logout::encode_request,
                        Logout::decode_response,
                    )
                    .map_err(|error| format!("IC Wire generation refused: {error:?}"))?,
                ),
            ] {
                save(name, source.as_bytes())?;
            }
            save("wire-runtime.ts", wire::typescript::RUNTIME.as_bytes())?;
            for (name, bytes) in [
                (
                    "credential-vector.hex",
                    CredentialEnvelope::encode_request(&credential())?,
                ),
                (
                    "logout-vector.hex",
                    Logout::encode_response(&model::LogoutResult::Revoked(9_007_199_254_740_993))?,
                ),
                (
                    "admission-vector.hex",
                    EstablishSession::encode_response(&Err(model::AdmissionRefusal::NotYetValid))?,
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
            if CredentialEnvelope::decode_request(&bytes)? != credential() {
                return Err("credential vector mismatch".into());
            }
            let bytes = read_file_no_follow(
                &directory.join("logout-from-ts.bin"),
                Logout::RESPONSE_LIMITS.bytes,
            )?;
            if Logout::decode_response(&bytes)?
                != model::LogoutResult::Revoked(9_007_199_254_740_993)
            {
                return Err("logout vector mismatch".into());
            }
            let bytes = read_file_no_follow(
                &directory.join("admission-from-ts.bin"),
                EstablishSession::RESPONSE_LIMITS.bytes,
            )?;
            if EstablishSession::decode_response(&bytes)?
                != Err(model::AdmissionRefusal::NotYetValid)
            {
                return Err("admission vector mismatch".into());
            }
        }
        _ => return Err("client-contracts requires generate|check DIRECTORY".into()),
    }
    Ok(())
}

// Metadata comes from the DTO's Serde layout; identities and budgets stay with Auth.
fn generate<Request: wire::typescript::Wire, Response: wire::typescript::Wire>(
    name: &str,
    id: u64,
    request: wire::Limits,
    response: wire::Limits,
    _encode_request: fn(&Request) -> Result<Vec<u8>, wire::Error>,
    _decode_response: fn(&[u8]) -> Result<Response, wire::Error>,
) -> Result<String, wire::typescript::GenerationError> {
    use serde_json::json;
    let types = std::collections::BTreeMap::from([
        (
            "Request".into(),
            wire::typescript::native_schema::<Request>()?,
        ),
        (
            "Response".into(),
            wire::typescript::native_schema::<Response>()?,
        ),
    ]);
    let method = json!({"name":"contract", "mode":"update", "id":id.to_string(),
        "request":{"k":"ref","name":"Request"}, "response":{"k":"ref","name":"Response"},
        "requestBytes":request.bytes, "requestNodes":request.nodes, "requestDepth":request.depth,
        "responseBytes":response.bytes, "responseNodes":response.nodes, "responseDepth":response.depth});
    let mut source = wire::typescript::generate_registry(&types, &[method], "./wire-runtime.js")?;
    source.push_str(&format!("\nexport const methodName = {};\nexport const {{ encodeRequest, decodeRequest, encodeResponse, decodeResponse }} = contract;\n", json!(name)));
    Ok(source)
}
