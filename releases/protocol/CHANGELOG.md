# Protocol changelog

Covers `ic-auth-protocol-types` and `ic-auth-wire`, released together.
The independent series starts after the [last coordinated release, 0.29.2](../history.md).

## [0.33.0] - Unreleased

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
