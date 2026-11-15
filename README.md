# IC Auth — new implementation

A clean restart of IC Auth around its current application-authentication design.
Volatile and durable storage are required. Published `ic-storage` is an interim
backend; the intended destination is `ic-storage-new` with orthogonal persistence.
Existing Wire layouts and signed credential bytes are compatibility constraints,
not redesign targets.

**Status: protocol foundation implemented.** `ic-auth-protocol-types` owns
passive claims, labels and Platform principal identity; `ic-auth` implements
schema-2 canonical claims encoding/decoding/hashing, unsigned session-request
identity, host-supplied TTL bounds and optional Ed25519/certified proof verification.
Version 0.2.0 adds `ic-auth-wire`
with all four Wire contracts and generated browser codecs. All packages are
unpublished. The current development increment adds the certified issuer-signature
component, root-delegation signing commitments and standalone root authorization.
Complete certified verification now combines both components with shared claims
policy and work charging. Receiver/storage, issuer state and browser lifecycle
replacement remain unimplemented. The existing
`ic-auth` repository remains the implemented reference and canonical production
owner until the replacement is qualified and adoption is explicitly approved.

Version 0.2.0 is a GitHub source milestone. All crates retain `publish = false`;
crates.io publication is deferred until the replacement is complete. It does not
claim a working authentication engine or authorize a consumer cutover.
Canic owns endpoint wiring using IC Wire; the Auth crates supply contract schemas
and authentication decisions without installing endpoints.
Current development uses registry-aligned versions: `ic-auth` 0.42.0 and
`ic-auth-protocol-types` / `ic-auth-wire` 0.41.0. These are the next minor
versions after the existing registry releases; version selection reserves no
registry slot and does not publish a crate.

- [Architecture and qualification plan](docs/design/DESIGN.md)
- [Design and ownership overview](docs/design/authentication.md)
- [Compatibility and acceptance](docs/design/compatibility.md)
- [Current handoff](docs/status/current.md)
- [Developer tooling and focused checks](docs/development/tooling.md)

Run `make help`, `make toko-tooling-check`, and `make deps-list` from the root.
`make install-tools` prepares the pinned tools; `make tools-check` verifies them.
`make test-request`, `make test-claims` and `make test-wire` exercise the implemented contracts;
`make lint-protocol`, `make doc-protocol` and `make check-protocol-wasm`
cover their other focused checks.
The shared `make deps-update` maintains the lock; `toko-cloc -d 1` reports source
and test counts. See the [request boundary](docs/development/request-boundary.md)
and [claims boundary](docs/development/claims-boundary.md) for evidence and scope.
The [Wire boundary](docs/development/wire-boundary.md) records native budget and
old/new generated-browser comparisons. `make qualify-wire` runs the independent
native comparison; `make qualify-wire-browser` also needs Node 24.21.0 and the
pinned reference Git objects (`AUTH_REFERENCE_REPO` defaults to `../ic-auth`).
The [Ed25519 verification boundary](docs/development/verification-boundary.md)
records protected inputs, refusal/charging order and independent signatures.
Run `make test-verification` and `make qualify-ed25519` for its focused checks.
The [certified issuer boundary](docs/development/certified-issuer-boundary.md)
records that component's scope and real certificate fixtures;
`make qualify-certified-issuer` runs those checks.
The [delegation encoding boundary](docs/development/delegation-encoding-boundary.md)
records root signing commitments and fixtures; run `make test-delegation-encoding`.
The [root verification boundary](docs/development/root-delegation-boundary.md)
records enrollment, grant binding and proof limits; run `make qualify-root-delegation`.
The [complete certified boundary](docs/development/certified-verification-boundary.md)
records the combined path and legacy cryptographic compatibility; run
`make qualify-certified qualify-shared-verification`.
`make test-wire-projection qualify-wire-admission` covers consuming Wire/domain
projection and real proofs through the verifier; Canic still owns endpoint wiring.

The objective is less maintained state and clearer ownership, not a promised
zero-debt result or a line-count quota. Keep rejection, atomicity and recovery
proofs when replacing implementation. Do not copy the old release system, task
history or compatibility layers merely because they exist.

Source provenance: `dragginzgame/ic-auth`, commit
`e52a40be75dd9809d0528189d84b43fe96952ad9`. See `reference.sha256` for the selected
reference files. Licensing is retained in [LICENSE.md](LICENSE.md).
