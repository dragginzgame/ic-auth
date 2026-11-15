# Current status

[0.2.0](https://github.com/dragginzgame/ic-auth-new/releases/tag/v0.2.0) remains the
tagged GitHub source milestone. Certified verification and Wire/domain admission
were pushed in [03d1a53](https://github.com/dragginzgame/ic-auth-new/commit/03d1a53)
at 0.42.0/0.41.0. Current development is `ic-auth` 0.43.0 and
protocol-types/Wire 0.42.0. Every crate remains
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

The next increment adds shared request narrowing to one protected role and
captures login/key/enrollment lease ceilings in proof evidence. Refreshed
enrollment cannot extend that captured result. Independent request cases and
both real proof mechanisms exercise the rules; they create no stored session.

Shared receiver policy now checks proof-free reads and exact retries against
current authority, retained login/device association and request identity. Seven
independent fact scenarios and a real two-mechanism fixture driver pass, including
retry after proof expiry with the original lease. See the
[receiver policy boundary](../development/receiver-policy-boundary.md).
These helpers neither observe clocks nor authenticate snapshots or commit state.

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

Next are fresh-admission/replacement decisions and atomic receiver storage.
Volatile/stable backends, issuer state and browser
lifecycle remain unimplemented. Published `ic-storage` is interim; `ic-storage-new`
orthogonal persistence remains the durable destination subject to its own gates.
The original repository remains production owner. Consumer inventory and full
replacement qualification remain open; this work authorizes no crates.io
publication, deployment or live storage migration.
