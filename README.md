# IC Auth — new implementation

A clean restart of IC Auth around its current application-authentication design.
Volatile and durable storage are required. Published `ic-storage` is an interim
backend; the intended destination is `ic-storage-new` with orthogonal persistence.
Existing Wire layouts and signed credential bytes are compatibility constraints,
not redesign targets.

**Status: protocol foundation implemented.** `ic-auth-protocol-types` owns
passive claims, labels and Platform principal identity; `ic-auth` implements
schema-2 canonical claims encoding/decoding/hashing, unsigned session-request
identity and host-supplied TTL bounds. Both packages are unpublished. Wire,
verification, receiver/storage, issuer and browser replacement remain
unimplemented. The existing
`ic-auth` repository remains the implemented reference and canonical production
owner until the replacement is qualified and adoption is explicitly approved.

Version 0.1.0 is a GitHub source milestone. All crates retain `publish = false`;
crates.io publication is deferred until the replacement is complete. It does not
claim a working authentication engine or authorize a consumer cutover.

- [Architecture and qualification plan](docs/design/DESIGN.md)
- [Design and ownership overview](docs/design/authentication.md)
- [Compatibility and acceptance](docs/design/compatibility.md)
- [Current handoff](docs/status/current.md)
- [Developer tooling and focused checks](docs/development/tooling.md)

Run `make help`, `make toko-tooling-check`, and `make deps-list` from the root.
`make install-tools` prepares the pinned tools; `make tools-check` verifies them.
`make test-request` and `make test-claims` exercise the implemented contracts;
`make lint-protocol`, `make doc-protocol` and `make check-protocol-wasm`
cover their other focused checks.
The shared `make deps-update` maintains the lock; `toko-cloc -d 1` reports source
and test counts. See the [request boundary](docs/development/request-boundary.md)
and [claims boundary](docs/development/claims-boundary.md) for evidence and scope.

The objective is less maintained state and clearer ownership, not a promised
zero-debt result or a line-count quota. Keep rejection, atomicity and recovery
proofs when replacing implementation. Do not copy the old release system, task
history or compatibility layers merely because they exist.

Source provenance: `dragginzgame/ic-auth`, commit
`e52a40be75dd9809d0528189d84b43fe96952ad9`. See `reference.sha256` for the selected
reference files. Licensing is retained in [LICENSE.md](LICENSE.md).
