# Current status

[0.2.0](https://github.com/dragginzgame/ic-auth-new/releases/tag/v0.2.0) is the
GitHub source milestone for Wire and Ed25519 verification. Every crate remains
unpublished. No complete Auth replacement or consumer cutover is claimed.

Product crates supply passive claims/labels, canonical schema-2 claims and request
identity/TTL, all four Wire contracts with generated browser codecs, and optional
Ed25519 proof verification against protected host inputs. See the
[request](../development/request-boundary.md), [claims](../development/claims-boundary.md),
[Wire](../development/wire-boundary.md) and
[verification](../development/verification-boundary.md) evidence.

The verifier uses published Platform 0.33.7, checks current caller/audience/key/
grant/time bindings, retains weighted attempt charges and returns immutable proof
evidence. Eight integration tests, a shared-budget sequence and four compile-fail
cases pass. The reference OpenSSL vector is independently reproduced. A separate
qualification package proves canonical-scalar refusal even when an activated
upstream legacy feature accepts the noncanonical control.

Wire retains 14 integration cases and 37 fixed frames compared across reference/
current native and generated browser codecs. Its independent qualification
package compares all four resource boundaries and aggregate allocation refusal.
Node codec evidence is separate from actual browser/SDK lifecycle qualification.

Rust 1.100.0 and authenticated Toko Tooling 0.44.4 remain selected. Product and
qualification workspaces have separate locks. Reference Wire and the adversarial
legacy-scalar activator stay outside product dependencies. Focused tests, lint,
docs, formatting and Wasm library compilation pass for implemented capabilities.

Certified proof verification is next, including complete root/delegation/issuer
checks and bounded proof work. Receiver policy, storage, issuer and browser
lifecycle remain unimplemented. Published `ic-storage` is interim; `ic-storage-new`
orthogonal persistence remains the durable destination subject to its own gates.
The original repository remains production owner. Consumer API inventory and full
replacement qualification remain open; no crates.io publication, deployment or
live storage migration is authorized by this work.
