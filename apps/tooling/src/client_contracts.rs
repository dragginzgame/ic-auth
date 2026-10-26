//! Generate bounded Rust-owned IC Wire contracts; no endpoint is installed.
use ic_auth_wire::{CredentialEnvelope, EstablishSession, Logout, ReadSession, model, wire};
use wire::Bytes;

fn credential() -> model::PortableCredential {
    model::PortableCredential {
        claims: Bytes::new(vec![0, 1, 127, 128, 255]).unwrap(),
        proof: model::PortableProof::Ed25519(Bytes::new((0..64).collect()).unwrap()),
    }
}

// Infer request/response types through the canonical codecs and take all other
// metadata from the same contract. No independent registry/schema declarations.
macro_rules! binding {
    ($contract:ty) => {
        generate(
            <$contract>::NAME,
            <$contract>::ID,
            <$contract>::REQUEST_LIMITS,
            <$contract>::RESPONSE_LIMITS,
            <$contract>::encode_request,
            <$contract>::decode_response,
        )
        .map_err(|error| format!("IC Wire generation refused: {error:?}"))?
    };
}

pub(crate) fn run(
    mode: crate::ContractMode,
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
        crate::ContractMode::Generate => {
            for (name, source) in [
                ("credential.ts", binding!(CredentialEnvelope)),
                ("establish.ts", binding!(EstablishSession)),
                ("read.ts", binding!(ReadSession)),
                ("logout.ts", binding!(Logout)),
            ] {
                let policy = match name {
                    "credential.ts" => format!(
                        "export const defaultMaximumBytes = {};\nexport const maximumConfiguredBytes = {};\n",
                        ic_auth_wire::BROWSER_DEFAULT_CREDENTIAL_BYTES,
                        ic_auth_wire::BROWSER_MAX_CONFIGURED_CREDENTIAL_BYTES,
                    ),
                    "logout.ts" => format!(
                        "export const maximumResponseBytes = {};\n",
                        ic_auth_wire::BROWSER_LOGOUT_RESPONSE_BYTES,
                    ),
                    _ => String::new(),
                };
                save(name, format!("{source}{policy}").as_bytes())?;
            }
            save("wire-runtime.ts", wire::typescript::RUNTIME.as_bytes())?;
            // Structured policy evidence for observing the actual generated codec
            // configuration. Expected settings come directly from the Rust owner.
            let policy = |request, response| {
                wire::typescript::method_limits(request, response)
                    .map_err(|error| format!("IC Wire policy refused: {error:?}"))
            };
            let policies = serde_json::json!({
                CredentialEnvelope::ID.to_string(): policy(CredentialEnvelope::REQUEST_LIMITS, CredentialEnvelope::RESPONSE_LIMITS)?,
                EstablishSession::ID.to_string(): policy(EstablishSession::REQUEST_LIMITS, EstablishSession::RESPONSE_LIMITS)?,
                ReadSession::ID.to_string(): policy(ReadSession::REQUEST_LIMITS, ReadSession::RESPONSE_LIMITS)?,
                Logout::ID.to_string(): policy(Logout::REQUEST_LIMITS, Logout::RESPONSE_LIMITS)?,
            });
            save("codec-policy.json", &serde_json::to_vec(&policies)?)?;

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
        crate::ContractMode::Check => {
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
    let mut method = wire::typescript::method_limits(request, response)?;
    method
        .as_object_mut()
        .expect("Wire method limit object")
        .extend(
            json!({"name":"contract", "mode":"update", "id":id.to_string(),
            "request":{"k":"ref","name":"Request"}, "response":{"k":"ref","name":"Response"}})
            .as_object()
            .expect("method metadata object")
            .clone(),
        );
    let mut source = wire::typescript::generate_registry(
        &types,
        &[method],
        "./wire-runtime.js",
        &std::collections::BTreeMap::new(),
        |report| {
            if !report.is_clean() {
                eprintln!("IC Wire bulk advice for {name}:\n{report}");
            }
        },
    )?;
    source.push_str(&format!("\nexport const methodName = {};\nexport const {{ encodeRequest, decodeRequest, encodeResponse, decodeResponse }} = contract;\n", json!(name)));
    Ok(source)
}
