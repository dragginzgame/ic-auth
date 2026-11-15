# Current status

Version 0.1.0 is the initial GitHub-only source milestone. Both crates remain
unpublished with `publish = false`; no complete Auth replacement is claimed.

The protocol foundation is implemented in two unpublished product crates:
passive claim fields/labels in `ic-auth-protocol-types`, and canonical schema-2
claims encode/decode/hash plus unsigned request identity/TTL bounds in `ic-auth`.
See the [request evidence](../development/request-boundary.md) and
[claims evidence](../development/claims-boundary.md).
`qualification/` remains reserved for actual independent qualification packages;
ordinary contract tests live beside the product packages.

The [architecture](../design/DESIGN.md) selects a full Auth reimplementation,
with shared bounded receiver decisions and separate issuer/browser ownership.
Published `ic-storage` is interim; `ic-storage-new` orthogonal persistence is the
intended durable destination. Its native persistence design is not yet a
qualified supplier contract. The first receiver slice still covers memory and
actual stable collections; OP integration has separate lifecycle/rollback gates.

Toko Tooling 0.44.4 is installed from its authenticated registry archive.
Cargo.lock and all member declarations agree with the installed snapshot.
The root selects published `sha2` 0.11.1 for existing SHA-256 protocol domains and
`ic-platform-types` 0.20.4 (defaults disabled) for canonical principal identity. See
[tooling setup](../development/tooling.md) for the focused commands.

Validated on Rust 1.100.0: 17 integration tests and one compile-fail doctest;
retained request and claims vectors independently checked with Perl SHA-256;
Clippy and Rustdoc with warnings denied; wasm32 library compilation. Formatting
and Cargo tooling admission pass. The runtime graph contains Platform Types and
SHA-256, with no Storage, Canic, issuer, runtime executor or native tooling edge.

Exact Wire/browser codecs, complete verifiers, receiver/storage, issuer and
browser lifecycle replacement are not implemented. Continue the protocol boundary
with exact Wire vectors and cross-decoding/generated browser qualification
under the [compatibility contract](../design/compatibility.md). The full consumer
API inventory remains open; no API removal or cutover is qualified. The original
repository remains production owner; no crates.io publication or deployment is
authorized by this milestone.
