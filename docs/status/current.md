# Current status

[0.2.0](https://github.com/dragginzgame/ic-auth-new/releases/tag/v0.2.0) is the
GitHub source milestone for Wire and Ed25519 verification. Every crate remains
unpublished. No complete Auth replacement or consumer cutover is claimed.
Current development is `ic-auth` 0.42.0 and protocol-types/Wire 0.41.0, selected
as the next minor after each crate's existing registry version. Both workspace
locks and local dependency requirements follow these versions.

Product crates supply passive claims/labels, canonical schema-2 claims and request
identity/TTL, all four Wire contracts with generated browser codecs, and optional
Ed25519 proof verification against protected host inputs. See the
[request](../development/request-boundary.md), [claims](../development/claims-boundary.md),
[Wire](../development/wire-boundary.md) and
[verification](../development/verification-boundary.md) evidence.

The optional [certified issuer-signature component](../development/certified-issuer-boundary.md)
now checks protected issuer/seed/network, fixed signing bytes and certificate age.
Eight independent real-BLS fixture cases pass, including both subnet-range layouts.
It returns signing time only; it cannot admit a credential or establish Auth root
authorization of issuer grants. The two verifier feature graphs remain separate.

[Delegation signing commitments](../development/delegation-encoding-boundary.md)
now preserve certificate, issuer-binding, root-header, leaf and derivation-path
domains with six compatibility cases. Grants share one byte encoder with claims.
These structural operations establish no root authorization.

The verifier uses published Platform 0.33.7, checks current caller/audience/key/
grant/time bindings, retains weighted attempt charges and returns immutable proof
evidence. Eight integration tests, a shared-budget sequence and four compile-fail
cases pass. The reference OpenSSL vector is independently reproduced. A separate
qualification package proves canonical-scalar refusal even when an activated
upstream legacy feature accepts the noncanonical control.

Wire retains 14 integration cases and 37 fixed frames compared across reference/
current native and generated browser codecs. Its independent qualification
package now uses Wire 0.24.1, with 296 frozen reference thresholds across all four
resource boundaries and aggregate allocation refusal; the old dependency is gone.
Node codec evidence is separate from actual browser/SDK lifecycle qualification.

Rust 1.100.0 and authenticated Toko Tooling 0.44.4 remain selected. Product and
qualification workspaces have separate locks. The adversarial legacy-scalar
activator and certificate fixture builders stay outside product dependencies. Focused tests, lint,
docs, formatting and Wasm library compilation pass for implemented capabilities.

Complete certified proof verification is next, including Auth root authorization,
common claims context and aggregate proof/work bounds. Receiver policy, storage, issuer and browser
lifecycle remain unimplemented. Published `ic-storage` is interim; `ic-storage-new`
orthogonal persistence remains the durable destination subject to its own gates.
The original repository remains production owner. Consumer API inventory and full
replacement qualification remain open; no crates.io publication, deployment or
live storage migration is authorized by this work.
