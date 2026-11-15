//! Generate all four browser codecs and their matching supplier runtime.
//! Usage: cargo run -p ic-auth-wire --features typescript --example generate -- OUTPUT

use ic_auth_wire::{
    BROWSER_DEFAULT_CREDENTIAL_BYTES, BROWSER_LOGOUT_RESPONSE_BYTES,
    BROWSER_MAX_CONFIGURED_CREDENTIAL_BYTES, CredentialEnvelope, EstablishSession, Logout,
    ReadSession, SessionInput, SessionView,
    model::{AdmissionRefusal, LogoutResult, PortableCredential, SessionReadRefusal},
    wire::{
        Limits,
        typescript::{self, Method, PayloadLimits, SchemaLimits, SchemaRegistry, Wire},
    },
};
use std::{fs, path::Path};

fn payload(limits: Limits) -> PayloadLimits {
    PayloadLimits {
        bytes: u64::try_from(
            limits
                .bytes
                .checked_sub(11)
                .expect("frame header allowance"),
        )
        .unwrap(),
        nodes: u64::try_from(limits.nodes).unwrap(),
        depth: u64::try_from(limits.depth).unwrap(),
    }
}

fn module<Q: Wire, R: Wire>(id: u64, name: &str, request: Limits, response: Limits) -> String {
    let mut registry = SchemaRegistry::new(SchemaLimits::default()).expect("schema allowance");
    registry.register::<Q>("Request").expect("request schema");
    registry.register::<R>("Response").expect("response schema");
    let graph = registry.finish().expect("validated schema");
    let mut source = graph
        .method_module(
            &[Method {
                export: "codec",
                id,
                request: "Request",
                response: "Response",
                request_limits: payload(request),
                response_limits: payload(response),
            }],
            "./wire-runtime.js",
            &Default::default(),
            |report| eprint!("{report}"),
        )
        .expect("generated method");
    // Preserve the existing public convenience exports; the supplier owns all
    // schema/runtime output. Fixed arrays stay arrays to retain signed layouts.
    source.push_str(&format!("\nexport const contract = globalThis.Object.freeze({{ ...codec, methodName: 'contract', mode: 'update' as const }});\nexport const methodName = {name:?};\nexport const {{ encodeRequest, decodeRequest, encodeResponse, decodeResponse }} = contract;\n"));
    source
}

fn main() -> std::io::Result<()> {
    let destination = std::env::args_os()
        .nth(1)
        .expect("provide the generated output directory");
    let destination = Path::new(&destination);
    let mut credential = module::<PortableCredential, PortableCredential>(
        CredentialEnvelope::ID,
        CredentialEnvelope::NAME,
        CredentialEnvelope::REQUEST_LIMITS,
        CredentialEnvelope::RESPONSE_LIMITS,
    );
    credential.push_str(&format!("export const defaultMaximumBytes = {BROWSER_DEFAULT_CREDENTIAL_BYTES};\nexport const maximumConfiguredBytes = {BROWSER_MAX_CONFIGURED_CREDENTIAL_BYTES};\n"));
    let establish = module::<SessionInput, Result<SessionView, AdmissionRefusal>>(
        EstablishSession::ID,
        EstablishSession::NAME,
        EstablishSession::REQUEST_LIMITS,
        EstablishSession::RESPONSE_LIMITS,
    );
    let read = module::<(), Result<SessionView, SessionReadRefusal>>(
        ReadSession::ID,
        ReadSession::NAME,
        ReadSession::REQUEST_LIMITS,
        ReadSession::RESPONSE_LIMITS,
    );
    let mut logout = module::<(), LogoutResult>(
        Logout::ID,
        Logout::NAME,
        Logout::REQUEST_LIMITS,
        Logout::RESPONSE_LIMITS,
    );
    logout.push_str(&format!(
        "export const maximumResponseBytes = {BROWSER_LOGOUT_RESPONSE_BYTES};\n"
    ));
    fs::create_dir_all(destination)?;
    for (name, source) in [
        ("credential", credential),
        ("establish", establish),
        ("read", read),
        ("logout", logout),
    ] {
        fs::write(destination.join(format!("{name}.ts")), source)?;
    }
    fs::write(destination.join("wire-runtime.ts"), typescript::RUNTIME)
}
