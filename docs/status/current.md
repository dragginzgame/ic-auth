# Current status

[0.2.0](https://github.com/dragginzgame/ic-auth-new/releases/tag/v0.2.0) remains the
GitHub source milestone. Current development is `ic-auth` 0.42.0 and
protocol-types/Wire 0.41.0, the selected next registry minors. Every crate remains
unpublished; no complete Auth replacement or consumer cutover is claimed.

Product crates supply passive claims/labels, canonical schema-2 claims and request
identity/TTL, all four Wire contracts with generated browser codecs, and optional
Ed25519 and complete certified proof verification. Canic owns endpoint wiring.
See [claims](../development/claims-boundary.md), [Wire](../development/wire-boundary.md),
[Ed25519](../development/verification-boundary.md) and
[certified verification](../development/certified-verification-boundary.md).

Both mechanisms share protected caller/audience/grant/window/reset policy,
immutable `VerifiedClaims` and one weighted receiver budget. Certified verification
binds claims to root-authorized grants and the issuer's BLS-backed IC signature,
with aggregate proof bounds and one charge retained across root/issuer failures.
The legacy cryptographic schemes, signing domains and canonical bytes are retained;
Platform 0.33.7 owns cryptographic primitives and IC certificate verification.

Evidence includes seven complete certified-proof cases and one real Ed25519/
certified shared-budget case, eight root cases, eight issuer cases, six delegation
commitment cases, existing Ed25519/context cases and immutable-result API checks.
OpenSSL independently constructs claims and root/Ed25519 signatures; real BLS
certificate fixtures remain qualification-only. The deliberately legacy-enabled
scalar negative control still proves Ed25519 canonical-scalar refusal.

Wire has 37 fixed reference frames and 296 frozen resource thresholds against
published IC Wire 0.24.1; the old codec dependency is removed. Generated Node codec
checks remain separate from actual browser/SDK lifecycle qualification. Consuming
Wire/domain conversion preserves signed material without copying large buffers;
five projection cases and three real Wire-to-verifier drivers cover both mechanisms,
unsigned request policy and retained failure charges.

Rust 1.100.0 and authenticated Toko Tooling 0.44.4 remain selected. Product and
qualification workspaces have separate locks. Codec-only, certified-only and
Ed25519-only graphs preserve their capability boundaries. Focused tests, lint,
docs, formatting and Wasm library compilation pass for implemented capabilities.

Next is receiver/session policy with replay,
revocation and atomic storage. Volatile/stable backends, issuer state and browser
lifecycle remain unimplemented. Published `ic-storage` is interim; `ic-storage-new`
orthogonal persistence remains the durable destination subject to its own gates.
The original repository remains production owner. Consumer inventory and full
replacement qualification remain open; this work authorizes no crates.io
publication, deployment or live storage migration.
