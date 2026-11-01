# Protocol changelog

Covers `ic-auth-protocol-types` and `ic-auth-wire`, released together.
The independent series starts after the [last coordinated release, 0.29.2](../history.md).

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
