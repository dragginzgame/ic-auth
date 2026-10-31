//! Shared private receiver configuration and diagnostics, without a runtime or store.
pub mod metrics;
pub mod sessions;

/// Source-cohort qualification helpers; never selected by the canister fixture.
#[cfg(all(feature = "backend-conformance", not(target_family = "wasm")))]
pub mod conformance;
