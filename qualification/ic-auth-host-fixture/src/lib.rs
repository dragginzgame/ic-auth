//! Test-only Wire controls. Never install this canister as an application service.
pub mod browser;
pub mod certified;
pub mod enrollment;
pub mod receiver;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// The native harness shares the contracts below. Certification, instruction counters
// and Wasm linear-memory measurements require the installed canister.
#[cfg(target_arch = "wasm32")]
mod canister {
    mod browser;
    mod certified;
    mod enrollment;
    mod issuer;
    mod receiver;

    use ic_auth_wire::{
        SessionInput, SessionView,
        model::{AdmissionRefusal, LogoutResult, PortableCredential},
    };
    use ic_platform_runtime as runtime;

    // init
    // Initialize the issuer and receiver fixtures on a fresh installation.
    // guard  : reject oversized input before decoding
    // limits : 256-byte input cap, enrollment::LIMITS for nonempty input, no reply
    #[runtime::init(
        runtime = ic_platform_runtime,
        guard = issuer::bounded_request,
        decode = issuer::decode_install,
        max_bytes = 256,
        on_error = issuer::endpoint_error,
    )]
    fn init(installation: crate::Installation) {
        issuer::init(installation)
    }

    // seal_auth
    // Seal stable receiver, issuer and host state in the upgrade transaction.
    // guard  : lifecycle hook, no ingress arguments
    // limits : empty input and reply
    #[runtime::pre_upgrade(
        runtime = ic_platform_runtime,
        guard = receiver::upgrade_guard,
        decode = issuer::decode_upgrade,
        max_bytes = 0,
        on_error = issuer::endpoint_error,
    )]
    fn seal_auth((): ()) {
        receiver::seal()
    }

    // reopen_auth
    // Reopen stable Auth or clear certification after volatile state loss.
    // guard  : reject oversized input before decoding
    // limits : up to one fixture upgrade-settings byte, no reply
    #[runtime::post_upgrade(
        runtime = ic_platform_runtime,
        guard = issuer::bounded_request,
        decode = receiver::decode_upgrade_settings,
        max_bytes = 1,
        on_error = issuer::endpoint_error,
    )]
    fn reopen_auth(settings: (bool, bool)) {
        receiver::reopen(settings)
    }

    // auth_fixture_control
    // Exercise an issuer operation with the actual ingress caller and time.
    // guard  : reject oversized input before decoding
    // limits : request REQUEST, reply RESPONSE
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "auth_fixture_control",
        guard = issuer::bounded_request,
        decode = issuer::decode_command,
        encode = issuer::encode_result,
        max_bytes = 256,
        on_error = issuer::endpoint_error,
    )]
    fn auth_fixture_control(command: crate::Command) -> Result<(), crate::Refusal> {
        issuer::control(command)
    }

    // auth_fixture_observe
    // Read retained issuer state and its certification witness.
    // guard  : reject oversized input before decoding
    // limits : request REQUEST, reply RESPONSE
    #[runtime::query(
        runtime = ic_platform_runtime,
        name = "auth_fixture_observe",
        guard = issuer::bounded_request,
        decode = issuer::decode_unit,
        encode = issuer::encode_observation,
        max_bytes = 256,
        on_error = issuer::endpoint_error,
    )]
    fn auth_fixture_observe((): ()) -> crate::Observation {
        issuer::observe()
    }

    // auth_fixture_measure
    // Measure one issuer control operation inside an update.
    // guard  : reject oversized input before decoding
    // limits : request REQUEST, reply RESPONSE
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "auth_fixture_measure",
        guard = issuer::bounded_request,
        decode = issuer::decode_measurement,
        encode = issuer::encode_measurement,
        max_bytes = 256,
        on_error = issuer::endpoint_error,
    )]
    fn auth_fixture_measure(command: crate::Command) -> crate::Measurement {
        issuer::measure(command)
    }

    // auth_fixture_witness_measure
    // Measure witness construction in a query.
    // guard  : reject oversized input before decoding
    // limits : request REQUEST, reply RESPONSE
    #[runtime::query(
        runtime = ic_platform_runtime,
        name = "auth_fixture_witness_measure",
        guard = issuer::bounded_request,
        decode = issuer::decode_witness_measurement,
        encode = issuer::encode_witness_measurement,
        max_bytes = 256,
        on_error = issuer::endpoint_error,
    )]
    fn auth_fixture_witness_measure((): ()) -> crate::WitnessMeasurement {
        issuer::measure_witness()
    }

    // authority_fixture_configure
    // Select the authority response scenario for subsequent enrollment calls.
    // guard  : controller and bounded enrollment frame
    // limits : enrollment::LIMITS for request and reply
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "authority_fixture_configure",
        guard = enrollment::controller,
        decode = enrollment::decode_scenario,
        encode = enrollment::encode_unit,
        max_bytes = 2048,
        on_error = issuer::endpoint_error,
    )]
    fn authority_fixture_configure(scenario: crate::enrollment::Scenario) {
        enrollment::configure(scenario)
    }

    // authority_fixture_snapshot
    // Return the enrollment snapshot bound to the requesting receiver and challenge.
    // guard  : reject oversized input before decoding
    // limits : enrollment::LIMITS for request and reply
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "authority_fixture_snapshot",
        guard = enrollment::bounded,
        decode = enrollment::decode_request,
        encode = enrollment::encode_snapshot,
        max_bytes = 2048,
        on_error = issuer::endpoint_error,
    )]
    fn authority_fixture_snapshot(challenge: [u8; 32]) -> crate::enrollment::Snapshot {
        enrollment::snapshot(challenge)
    }

    // receiver_fixture_control
    // Exercise receiver operations with the authenticated IC caller.
    // guard  : reject oversized input before decoding
    // limits : request receiver::REQUEST, reply RESPONSE
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "receiver_fixture_control",
        guard = receiver::bounded_request,
        decode = receiver::decode_command,
        encode = receiver::encode_result,
        max_bytes = 16_384,
        on_error = issuer::endpoint_error,
    )]
    async fn receiver_fixture_control(
        command: crate::receiver::Command,
    ) -> Result<crate::receiver::Outcome, crate::receiver::Refusal> {
        receiver::control(command).await
    }

    // receiver_fixture_observe
    // Read receiver counters and enrollment without consuming verification credit.
    // guard  : reject oversized input before decoding
    // limits : request receiver::REQUEST, reply RESPONSE
    #[runtime::query(
        runtime = ic_platform_runtime,
        name = "receiver_fixture_observe",
        guard = receiver::bounded_request,
        decode = receiver::decode_unit,
        encode = receiver::encode_observation,
        max_bytes = 16_384,
        on_error = issuer::endpoint_error,
    )]
    fn receiver_fixture_observe((): ()) -> crate::receiver::Observation {
        receiver::observe()
    }

    // receiver_fixture_measure
    // Measure one receiver operation inside an update.
    // guard  : reject oversized input before decoding
    // limits : request receiver::REQUEST, reply RESPONSE
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "receiver_fixture_measure",
        guard = receiver::bounded_request,
        decode = receiver::decode_measurement,
        encode = receiver::encode_measurement,
        max_bytes = 16_384,
        on_error = issuer::endpoint_error,
    )]
    async fn receiver_fixture_measure(
        command: crate::receiver::Command,
    ) -> crate::receiver::Measurement {
        receiver::measure(command).await
    }

    // certified_fixture_measure
    // Measure verification of the installed certified credential fixture.
    // guard  : reject oversized input before decoding
    // limits : RESPONSE for request and reply
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "certified_fixture_measure",
        guard = certified::guard,
        decode = certified::decode,
        encode = certified::encode,
        max_bytes = 32768,
        on_error = issuer::endpoint_error,
    )]
    fn certified_fixture_measure(
        command: crate::certified::Command,
    ) -> crate::certified::Measurement {
        certified::measure(command)
    }

    // browser_fixture_credential
    // Issue or look up a credential for the fixed fixture account.
    // guard  : reject oversized input before decoding
    // limits : request REQUEST, reply CredentialEnvelope::REQUEST_LIMITS
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "browser_fixture_credential",
        guard = browser::bounded::<256>,
        decode = browser::decode_lookup,
        encode = browser::encode_credential,
        max_bytes = 256,
        on_error = issuer::endpoint_error,
    )]
    fn browser_fixture_credential(lookup: bool) -> PortableCredential {
        browser::credential(lookup)
    }

    // establish_portable_session
    // Admit the browser credential and return its session view.
    // guard  : reject oversized input before decoding
    // limits : EstablishSession request and reply limits
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "establish_portable_session",
        guard = browser::bounded::<524288>,
        decode = browser::decode_admit,
        encode = browser::encode_admit,
        max_bytes = 524288,
        on_error = issuer::endpoint_error,
    )]
    fn establish_portable_session(input: SessionInput) -> Result<SessionView, AdmissionRefusal> {
        browser::admit(input)
    }

    // browser_fixture_write
    // Apply a fixture write under the current caller’s session.
    // scope  : write
    // guard  : reject oversized input before decoding
    // limits : request REQUEST, reply RESPONSE
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "browser_fixture_write",
        guard = browser::bounded::<256>,
        decode = browser::decode_write,
        encode = browser::encode_write,
        max_bytes = 256,
        on_error = issuer::endpoint_error,
    )]
    fn browser_fixture_write((): ()) -> Result<u32, ()> {
        browser::write()
    }

    // logout_portable_session
    // Revoke the caller’s login and report the resulting logout receipt.
    // guard  : reject oversized input before decoding
    // limits : Logout request and reply limits
    #[runtime::update(
        runtime = ic_platform_runtime,
        name = "logout_portable_session",
        guard = browser::bounded::<11>,
        decode = browser::decode_logout,
        encode = browser::encode_logout,
        max_bytes = 11,
        on_error = issuer::endpoint_error,
    )]
    fn logout_portable_session((): ()) -> LogoutResult {
        browser::logout()
    }

    // browser_fixture_observe
    // Read the browser fixture’s observable state.
    // guard  : reject oversized input before decoding
    // limits : request REQUEST, reply RESPONSE
    #[runtime::query(
        runtime = ic_platform_runtime,
        name = "browser_fixture_observe",
        guard = browser::bounded::<256>,
        decode = browser::decode_observe,
        encode = browser::encode_observe,
        max_bytes = 256,
        on_error = issuer::endpoint_error,
    )]
    fn browser_fixture_observe((): ()) -> crate::browser::Observation {
        browser::observe()
    }
}

/// Compute the SHA-256 digest of fixture bytes on either host.
pub fn hash_bytes(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    Sha256::digest(bytes.as_ref()).into()
}

/// Qualification control identity; unrelated to product endpoint identities.
pub const CONTROL: u64 = 0x4155_5448_5155_0001;
/// Qualification observation identity.
pub const OBSERVE: u64 = 0x4155_5448_5155_0002;
/// Request limits are checked before copying and during Wire decoding.
pub const REQUEST: ic_wire::Limits = ic_wire::Limits {
    bytes: 256,
    allocation: 1024,
    nodes: 32,
    depth: 4,
};
/// Bounded certificate/claims observation, not a product credential envelope.
pub const RESPONSE: ic_wire::Limits = ic_wire::Limits {
    bytes: 32_768,
    allocation: 1_048_576,
    nodes: 65_536,
    depth: 8,
};
/// Public fixture seed; this is not a signing secret.
pub const SEED: &[u8] = b"auth-host-qualification";

///
/// Fault
///
/// Deliberate interruptions inside one synchronous update transaction.
///

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Fault {
    /// Commit normally.
    None,
    /// Trap after mutating the host's signature leaf.
    Leaf,
    /// Trap after publishing the system certification root.
    Root,
    /// Return a host error after root publication; the adapter must trap.
    HostError,
    /// Trap after Auth has retained completion.
    Completion,
}

///
/// Command
///
/// Controller-independent fixture account operations use the actual IC caller.
///

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Command {
    /// Reserve and approve a fixed fixture intent for the authenticated caller.
    Prepare,
    /// Complete that caller's retained intent, injecting an optional host fault.
    Complete(Fault),
    /// Account-authenticated revocation of the retained login.
    Revoke,
    /// Prepare and approve a specific renewal using the actual retained browser caller.
    Renew(u64),
    /// Run one bounded issuer maintenance batch.
    Prune,
}

///
/// Refusal
///
/// Fixed fixture refusal categories, separate from product Wire error projection.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Refusal {
    /// No surviving issuer state or terminal closure.
    Closed,
    /// Caller is not the original account, or the login is revoked.
    Denied,
    /// Completion exceeded the original deadline.
    Expired,
    /// Another ordinary Auth refusal.
    Auth,
}

///
/// Progress
///
/// Observable retained operation phase.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Progress {
    /// Genuinely new installation with no operation.
    Empty,
    /// Reserved/approved operation, without a published leaf.
    Pending,
    /// Original completion is retained.
    Completed,
    /// Lost/closed state cannot be reconstructed through these endpoints.
    Closed,
}

///
/// Observation
///
/// Query evidence from actual host certificate acquisition and retained state.
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    /// Retained progress, without creating work.
    pub progress: Progress,
    /// Physical login/operation/result-byte reservations.
    pub usage: (u32, u32, u64),
    /// Retained issuance credit.
    pub credit: u32,
    /// Exact completed claims, empty before completion.
    pub claims: Vec<u8>,
    /// Host signature subtree root, including the empty state.
    pub root: [u8; 32],
    /// Actual IC data certificate; never supplied by an ingress argument.
    pub certificate: Vec<u8>,
    /// Current canister-signature proof, empty without a retained leaf/completion.
    pub signature: Vec<u8>,
    /// Actual host query time.
    pub now_ns: u64,
}

/// Qualification-only synchronous issuer measurement identity.
pub const MEASURE: u64 = 0x4155_5448_5155_000b;

///
/// Measurement
///
/// Issuer fixture execution interval, excluding outer codecs and executor overhead.
///

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Measurement {
    /// Actual control outcome.
    pub result: Result<(), Refusal>,
    /// Runtime counter-zero delta around synchronous control execution.
    pub instructions: u64,
    /// Total Wasm linear-memory pages before and after execution.
    pub heap_pages: (u64, u64),
    /// Ordinary control reply bytes, excluding measurement metadata.
    pub response_bytes: u64,
}

/// Qualification-only query measurement identity.
pub const WITNESS_MEASURE: u64 = 0x4155_5448_5155_000c;

///
/// WitnessMeasurement
///
/// Current certificate/witness assembly, excluding outer reply encoding.
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WitnessMeasurement {
    /// Actual query evidence, including the current certificate.
    pub observation: Observation,
    /// Query counter-zero delta around evidence assembly.
    pub instructions: u64,
    /// Linear-memory pages before and after assembly.
    pub heap_pages: (u64, u64),
    /// Encoded ordinary observation response size.
    pub response_bytes: u64,
}

///
/// Installation
///
/// Installation-owned test authority and network trust; never accepted at admission.
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Installation {
    /// Enrollment authority principal bytes; empty selects this canister.
    pub authority: Vec<u8>,
    /// Optional test network BLS key for the independent certified-cost lane.
    pub certified_network_key: Vec<u8>,
    /// Optional installation-only receiver limits; ingress cannot change them.
    pub receiver_capacity: Option<receiver::Capacity>,
    /// Inject missing upgrade hooks for installed loss qualification.
    pub skip_auth_checkpoint: bool,
}
