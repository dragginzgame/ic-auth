# Engine changelog

Covers public `ic-auth` and the unpublished backend conformance, qualification and
native tooling packages. The independent series starts after the
[last coordinated release, 0.29.2](../history.md).

## [0.32.0] - Unreleased

### ⚠️ Breaking

- Align grant APIs with Protocol's checked scope labels and 64-byte roles.
  `canonical::validate_scopes` accepts `ScopeLabel` slices and checks strict
  ordering; individual label admission belongs to the passive types. Malformed
  portable scope text is refused as `InvalidShape` while decoding. Signed bytes,
  verification algorithms, authority checks and valid stored formats are unchanged.

### ⬆️ Dependencies

- Adopt the incoming lock selections for Runtime 0.18.1, Testkit 0.80.1,
  Wire/derive 0.15.1, Storage 0.65.2, Metrics 0.27.2, Build 0.74.1 and Host
  FS/Process/Artifacts 0.74.2. Keep the independent file CLI pin at 0.74.1.
  Advance ctutils to 0.4.8.
  Regenerated browser contracts are unchanged.

### 🧹 Cleanup

- Remove unused direct Metrics and Serde-bytes dependencies from the private
  qualification canister. Keep metrics with the shared fixtures and certificate
  envelope bytes with Wire; tighten direct Metrics ownership accordingly.
- Share private canister codec bindings and use each method's canonical request
  ceiling before copying input. Consolidate login record serialization without
  exposing unchecked deserialization or changing its stored bytes.
- Consume checkpoint buffers during restoration and profile a borrowed session.
  Move owned role strings through fixture projections and declare the private
  enrollment field mapping once. Share primitive browser u64 admission while
  preserving each owner's clock, expiry and positivity rules.

### 🧪 Testing

- Derive the CI manifest seal from Cargo's admitted workspace inventory, including
  newly added members. Refuse incomplete discovery and changed preparation inputs;
  qualify paths with spaces and preserve the complete CI/release stage roster.

## [0.31.0] - 2026-11-02

### ⚠️ Breaking

- Require Rust 1.99.1 for all packages, matching the pinned development/release
  compiler ([#124](https://github.com/dragginzgame/ic-auth/issues/124)).
- Align public Principal and crypto/certificate APIs with Protocol's canonical
  Types 0.17 and Platform 0.18. Consumers must select matching Rust identities;
  cryptographic checks, signed bytes and the trust model are unchanged.
- Carry shared certificate lifetime and grant errors directly in issuance errors;
  callers match `Lifetime` and `Grants` rather than duplicate flattened variants.
- Private qualification controls use typed refusal codes instead of text. All
  native callers and codec hooks change together; public Auth Wire and retained
  checkpoint/record formats are unchanged.
- Adopt Storage 0.65's consolidated allocator and checksummed collections in the
  private fixture. Fresh backing is required; previous manager/ledger images are
  refused without reset. Auth's payload/record bytes and crypto remain unchanged.

### ⬆️ Dependencies

- Replace Memory/Storage 0.63 with published `ic-storage` 0.65.0, removing both
  `ic-memory` and `ic-stable-structures` from the lock. Use checked Storage leases,
  explicit Cell load ceilings and typed write errors; retain one maintained
  Runtime/generator/guard through its direct Wasm Runtime dependency.
- Select Metrics 0.27.0 in private qualification, preserving admission counters.
  Testkit/Host retain independent arithmetic-only Metrics 0.26.1 internally; no
  counter type crosses that boundary and native report metadata stays disabled.
- Adopt Build 0.74.0 and Host FS/Process/Artifacts 0.74.1, including the explicitly
  prepared file CLI and release editor. Adopt Testkit 0.80.0 and the canonical
  independent PocketIC 0.18 client with the unchanged 16.1 server pairing. Remove
  the old Host 0.73, Metrics 0.25 and management DTO 0.8 providers.
- Adopt Shared Tooling 0.27.28 at `8c7fae6605c69a930e3b1f14e1ff5c4c9dafbcb7`.
  Retain Auth's complete CI/release roster; optional release-group plans are not
  activated. Existing tool retention and release custody remain unchanged.
- Align public Auth, Storage and Testkit with Platform 0.18, Types 0.17,
  Runtime 0.18 and Wire/derive 0.15. The new Storage and Wire releases resolve
  the prior runtime-guard and Principal-schema blockers. Preserve one host
  Runtime/generator/guard, all public capability boundaries, schemas and bytes.

### 🧹 Cleanup

- Construct fresh sessions from the engine's checked, immutable admission snapshot;
  retain complete validation for exact retries, later reads and storage commits.
- Decode checkpoints from borrowed stable payloads, and use maintained digest
  operations for digest-only certified roots. Keep witness construction separate.
- Select logout dispatch/wait/completion in one strict journal transaction while
  preserving durable leases, post-signing checks and the existing stored profile.
- Share valid native signing/context setup, count decoded replay records without
  a temporary vector, and reuse the owned Wire logout refusal conversion.
- Keep Auth-specific group/recovery/publication fixtures while leaving generic
  version arithmetic and repeated delivery permutations with Shared Tooling.
- Use one compiler setup/cache and the isolated archive feature matrix; retain
  native and Wasm Clippy, proc-macro and real runtime qualification instead of
  repeating those compilations under a second compiler.

### 🧪 Testing

- Qualify the consolidated Storage host in the boundary caller fixture, including
  missing/duplicate Storage, retired Memory/stable-structures and native-client
  refusal alongside the existing Runtime/generator/guard checks.

## [0.30.0] - 2026-11-01

### ⚠️ Breaking

- Align exposed Principal and codec identities with Protocol's Types 0.16 and
  Wire 0.14 generation. Consumers exchanging these Rust values must align their
  dependencies. Signed bytes, verification checks, stored formats and TEE/non-TEE
  trust policy are unchanged.

### ⬆️ Dependencies

- Adopt Platform 0.17.0, Wire/derive 0.14.2 and Testkit 0.78.0 from the registry.
  The qualification host selects Runtime/macros/guard 0.17.1 directly; native
  Testkit uses Platform's owned PocketIC client and the matching Wire limits.
- Align private admission metrics with Testkit's Metrics 0.25.0 selection,
  retaining one arithmetic provider and unchanged counters/Serde admission.
- Adopt Memory/Storage 0.62.1 in the private qualification host, aligning ledger
  codec errors with Wire 0.14 and removing the remaining Wire 0.13 providers.
  Preserve checked allocation handles, checkpoint bytes, bounds and fresh/restore
  refusals; no retained-state reset or migration is introduced.

### 🧪 Testing

- Isolate release fixture Make controls from an enclosing release's group, so
  missing-group admission and independent Engine selection remain exercised.
- Verify the three local library archives together in the read-only package gate,
  so pending Protocol identity changes do not accidentally compile Engine against
  the older registry Protocol. Selected-group publication keeps normal registry
  admission; cover package refusal and Make descriptor/unsafe-mode behavior.

### ♻️ Refactoring

- Select canonical Types directly for engine self-authentication; make Platform
  optional and enable it explicitly through crypto capabilities. Default and
  session-only graphs exclude the facade. See the
  [Protocol boundary change](../protocol/CHANGELOG.md).
- Release Engine independently of Protocol. Bind validation and publication to
  the selected group and saved policy, retaining the complete gate and uncertain
  upload recovery. Compiler/package and backend archive qualification admit
  independent Engine and Protocol versions.
- Enforce facade-free passive/default/session graphs and reject the independent
  Runtime owner in public and conformance graphs. Packaged-consumer compiler
  qualification witnesses the canonical Types identity directly.
