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
    ($contract:ty) => {{
        let (source, policy) = generate(
            <$contract>::NAME,
            <$contract>::ID,
            <$contract>::REQUEST_LIMITS,
            <$contract>::RESPONSE_LIMITS,
            <$contract>::encode_request,
            <$contract>::decode_response,
        )
        .map_err(|error| format!("IC Wire generation refused: {error:?}"))?;
        (source, (<$contract>::ID.to_string(), policy))
    }};
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
    // Record actual publications, so the driver needs no parallel roster.
    let mut artifacts = String::new();
    let mut publish =
        |kind: &str, name: &str, bytes: &[u8]| -> Result<(), Box<dyn std::error::Error>> {
            save(name, bytes)?;
            use std::fmt::Write;
            writeln!(artifacts, "{kind}\t{name}")?;
            Ok(())
        };
    if matches!(mode, crate::ContractMode::Generate) {
        let mut policies = serde_json::Map::new();
        for (name, (source, (id, limits)), policy) in [
            (
                "credential.ts",
                binding!(CredentialEnvelope),
                format!(
                    "export const defaultMaximumBytes = {};\nexport const maximumConfiguredBytes = {};\n",
                    ic_auth_wire::BROWSER_DEFAULT_CREDENTIAL_BYTES,
                    ic_auth_wire::BROWSER_MAX_CONFIGURED_CREDENTIAL_BYTES,
                ),
            ),
            ("establish.ts", binding!(EstablishSession), String::new()),
            ("read.ts", binding!(ReadSession), String::new()),
            (
                "logout.ts",
                binding!(Logout),
                format!(
                    "export const maximumResponseBytes = {};\n",
                    ic_auth_wire::BROWSER_LOGOUT_RESPONSE_BYTES,
                ),
            ),
        ] {
            publish("source", name, format!("{source}{policy}").as_bytes())?;
            policies.insert(id, limits);
        }
        publish(
            "source",
            "wire-runtime.ts",
            wire::typescript::RUNTIME.as_bytes(),
        )?;
        // Policy evidence is carried alongside each generated binding.
        publish(
            "evidence",
            "codec-policy.json",
            &serde_json::to_vec(&policies)?,
        )?;
    }
    // One typed declaration owns each vector's value, direction, bound and
    // filenames in both Rust generation and TypeScript-result admission.
    macro_rules! vector {
        ($name:literal, $limits:expr, $encode:path, $decode:path, $expected:expr) => {
            match mode {
                crate::ContractMode::Generate => {
                    let bytes = $encode(&$expected)?;
                    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
                    publish(
                        "evidence",
                        concat!($name, "-vector.hex"),
                        format!("{hex}\n").as_bytes(),
                    )?;
                }
                crate::ContractMode::Check => {
                    let bytes = read_file_no_follow(
                        &directory.join(concat!($name, "-from-ts.bin")),
                        $limits.bytes,
                    )?;
                    if $decode(&bytes)? != $expected {
                        return Err(concat!($name, " vector mismatch").into());
                    }
                }
            }
        };
    }
    vector!(
        "credential",
        CredentialEnvelope::REQUEST_LIMITS,
        CredentialEnvelope::encode_request,
        CredentialEnvelope::decode_request,
        credential()
    );
    vector!(
        "logout",
        Logout::RESPONSE_LIMITS,
        Logout::encode_response,
        Logout::decode_response,
        model::LogoutResult::Revoked(9_007_199_254_740_993)
    );
    vector!(
        "admission",
        EstablishSession::RESPONSE_LIMITS,
        EstablishSession::encode_response,
        EstablishSession::decode_response,
        Err(model::AdmissionRefusal::NotYetValid)
    );
    vector!(
        "read",
        ReadSession::RESPONSE_LIMITS,
        ReadSession::encode_response,
        ReadSession::decode_response,
        Err(model::SessionReadRefusal::Revoked)
    );
    if matches!(mode, crate::ContractMode::Generate) {
        save("artifacts.tsv", artifacts.as_bytes())?;
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
) -> Result<(String, serde_json::Value), wire::typescript::GenerationError> {
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
    let policy = wire::typescript::method_limits(request, response)?;
    let mut method = policy.clone();
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
    Ok((source, policy))
}

#[cfg(test)]
mod tests;
