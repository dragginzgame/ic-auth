# Changelog

## [0.1.0] - 2026-11-15

Initial GitHub source milestone for the replacement protocol foundation.
This is not a complete authentication engine or a crates.io publication.

### ✨ Added

- Begin the replacement protocol boundary with validated scope labels, exact
  unsigned session-request hashes and bounded host TTL calculation. Fixed
  reference vectors preserve request identity, including absent versus explicit
  TTL; malformed inputs and deadline overflow refuse without normalization of
  labels. This does not implement credential verification or session admission.
- Add the bounded schema-2 claims codec with exact signed bytes and SHA-256,
  passive claims/roles and published Platform principal identity. Preserve the
  independent golden vector, strict signed-grant ordering and all count/byte
  limits; decoding remains separate from proof and authority verification.

### 🛠️ Tooling

- Establish the Rust 1.100.0 product workspace and product/qualification
  directories, with authenticated Toko Tooling 0.44.4, shared dependency commands
  and the standalone LOC reporter. The first real packages now inherit the
  tooling selection, produce a lockfile and have focused native/lint/docs/Wasm
  checks. Both packages remain unpublished.

### ▶️ Try it

From the repository root:

```sh
make install-tools
make tools-check
make toko-tooling-check
make deps-list
make test-request
make test-claims
```

Setup may download pinned tools. Prepare locked crate sources with
`cargo fetch --locked` on a fresh checkout before running offline checks.
The tooling check verifies Cargo selection and installed bytes; the request
target runs seven contract tests and one compile-fail doctest; the claims target
adds ten integration cases. Use `lint-protocol`, `doc-protocol` and
`check-protocol-wasm` for the now-expanded protocol boundary.
