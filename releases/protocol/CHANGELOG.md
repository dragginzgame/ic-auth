# Protocol changelog

Covers `ic-auth-protocol-types` and `ic-auth-wire`, released together.
The independent series starts after the [last coordinated release, 0.29.2](../history.md).

## [0.39.0] - 2026-11-11

### ⚠️ Breaking

- Select IC Wire 0.21.1 for public transport containers, limits and codec traits.
  Align consumers exchanging those Rust values. Wire now follows Postcard's
  acceptance of redundant outer length varints; encoding, method identities,
  traversal budgets and the original opaque signed claims/proofs stay unchanged.
  Retain original transport bytes when exact frame identity matters.
- Regenerate and review all five browser files. Auth's explicit Request/Response
  names preserve its four bindings across Wire's readable-name migration; the
  shared runtime updates alias-cycle detection without changing valid frames.

## [0.38.0] - 2026-11-10

### ⚠️ Breaking

- Add signed device slot (0–7) and nonzero generation to portable claims. The
  canonical credential schema advances to 2; schema 1 is rejected. Issuers must
  allocate slots under account authentication, preserve generations on renewal,
  and advance them with fresh browser keys for replacement. Wire envelopes are
  unchanged; coordinated Engine/issuer adoption is required.

## [0.37.0] - 2026-11-09

### ⚠️ Breaking

- Select canonical Platform Types 0.19 and IC Wire 0.19. Rust consumers exchanging
  Principal, transport containers, limits, errors or schema traits must align
  these dependency identities. Method identities, budgets, encoded bytes and
  stored formats are unchanged.

### ⬆️ Dependencies

- Regenerate and review all five browser files with Wire 0.19's generator. The
  four Auth contracts are unchanged; the shared runtime adds upstream finite
  packed-float support, which Auth's schemas do not select.

## [0.36.0] - 2026-11-09

### ⚠️ Breaking

- Select IC Wire 0.18 for exposed transport containers, limits and codec errors.
  Rust consumers exchanging Wire values must align that dependency identity.
  Method identities, budgets, encoded bytes and generated browser contracts
  remain unchanged; retained records require no format migration.

### 🛠️ Tooling

- Inherit the workspace's exact Toko Tooling 0.36 dev-dependency and source-bundle
  cargo-machete exemption. Production dependency graphs and protocol bytes are
  unchanged.

## [0.35.1] - 2026-11-08

### ⬆️ Dependencies

- Select Wire/derive 0.17.13 as the compatible minimum. Auth does not select the
  new packed 16-byte ID encoding; existing DTO bytes, method identities, bounds
  and generated browser method contracts remain unchanged.

## [0.35.0] - 2026-11-06

### ⚠️ Breaking

- Distinguish confirmed known-login revocation from bounded unknown-login denial
  with `Unconfirmed`. Retire the logout Wire identity; receivers and generated
  clients must update together for the clock-independent
  [#127 contract](https://github.com/dragginzgame/ic-auth/issues/127).

### ⬆️ Dependencies

- Select Wire/derive 0.17.4. Regenerate and review all five browser contract/runtime
  files. The logout binding changes with the current confirmation contract;
  other transport budgets and identities are unchanged.

## [0.34.0] - 2026-11-05

### ⚠️ Breaking

- Adopt Wire/derive 0.17. Rust consumers exchanging Wire limits, errors, bounded
  DTO fields or schema traits must align to that identity. Existing schema
  feature forwarding selects the now-optional metadata derive. Auth frames,
  method identities, budgets and stored formats are unchanged.

## [0.33.1] - 2026-11-04

### 🧪 Testing

- Move Principal parsing and scope-label unit tests into adjacent `tests.rs`
  modules. Keep assertions, public types and transport contracts unchanged.

## [0.33.0] - 2026-11-03

### ⚠️ Breaking

- Adopt canonical Types 0.18 and Wire/derive 0.16. Rust consumers exchanging
  Principal, parse errors, schema traits or codec limits must align those identities.
  Auth uses no Account DTOs: its frames, contract IDs, signed material and stored
  records are unchanged; all five regenerated browser files are unchanged.

## [0.32.0] - 2026-11-02

### ⚠️ Breaking

- Grant scopes use checked `ScopeLabel` values instead of strings, and `AuthRole`
  enforces the existing transport's 64-byte ceiling at construction and decoding.
  Rust callers must parse labels or use their fallible owned constructors. Grant
  cardinality, ordering and authority remain explicit engine/host checks.
- Valid label text, signed bytes and Wire frames are unchanged. Invalid grant
  labels fail during construction or decoding, before grant-set policy checks.
  The maintainer confirmed no retained roles above 64 bytes need preservation;
  no data reset or alternate reader is introduced.

## [0.31.0] - 2026-11-02

### ⚠️ Breaking

- Adopt canonical Types 0.17 and Wire/derive 0.15. Consumers exchanging
  Principal, parse errors, schema traits or codec limits must align those Rust
  identities. Principal derivation, schemas, transport bytes and budgets remain
  unchanged; generated browser contracts are unchanged.
- Require Rust 1.99.1, matching the pinned development/release compiler
  ([#124](https://github.com/dragginzgame/ic-auth/issues/124)). Retain independent
  native/Wasm archive consumers and passive dependency isolation.

## [0.30.0] - 2026-11-01

### ⚠️ Breaking

- Adopt canonical Types 0.16 and Wire/derive 0.14.2. Consumers exchanging Principal,
  parse errors, Wire traits or codec limits must align these Rust dependency
  identities. Principal derivation, schema and transport bytes are unchanged;
  external Principal conversions require the explicit `principal-external` feature.

### ♻️ Refactoring

- Depend directly on canonical Types in the passive protocol. Engine crypto
  remains independently selected; passive schemas need no facade or runtime.
- Release Protocol independently of Engine using explicit group selection and
  shared Cargo metadata preparation. Preserve existing package versions and history;
  Engine-only releases leave these package identities and this ledger unchanged.
