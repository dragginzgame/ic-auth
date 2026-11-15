# Current status

[0.2.0](https://github.com/dragginzgame/ic-auth-new/releases/tag/v0.2.0) remains the
tagged GitHub source milestone. This source checkpoint contains `ic-auth` 0.43.0
and protocol-types/Wire 0.42.0, extending the earlier
[policy increment](https://github.com/dragginzgame/ic-auth-new/commit/25817fd).
Every crate remains unpublished; no complete replacement or cutover is claimed.

Product crates supply passive claims/labels, canonical schema-2 claims and request
identity/TTL, all four Wire contracts with generated browser codecs, and optional
Ed25519 and complete certified verification. Canic owns endpoint wiring. See the
[Wire](../development/wire-boundary.md), [Ed25519](../development/verification-boundary.md)
and [certified](../development/certified-verification-boundary.md) boundaries.
Legacy cryptographic schemes, signing domains and canonical bytes are retained;
Platform 0.33.7 owns primitives and IC certificate verification.

The first [memory receiver](../development/memory-receiver-boundary.md) is now
implemented behind `receiver-memory` plus at least one complete verifier feature.
It owns bounded caller/device/replay/account/expiry state and one work budget.
Admission decodes once, checks shared retry/login/capacity policy, verifies the
complete proof and publishes affected records without a fallible tail or heap clone.
Failed proof work retains its charge without reserving or displacing a login.

Exact retry preserves the original receipt after proof expiry or replay pruning.
Proof-free reads check current authority and retained login/device association.
Renewal and higher-generation replacement preserve physical replay quotas and
immutable login deadlines. Known-login revocation removes the session and retains
denial/fences/replay. Bounded pruning removes expired sessions/replays only.
Logical byte accounting retains reference record representations through optional
Wire/Serde views; no persisted format or old-format reader was introduced.

Memory starts closed with empty credit. Fresh activation is a host assertion for
a genuinely new receiver, not recovery from lost state; close is terminal.
Boot authentication, lost-memory recovery, unknown-login denial and complete Wire
logout integration remain open. The host owns authenticated caller/time/enrollment,
transaction isolation and abort/rollback. Native failure injection and installed IC
lifecycle evidence are still required before claiming those integration guarantees.

`make test-memory` exercises nine public-API Ed25519 scenarios using six independent
Node/OpenSSL credentials and one certified-only receiver scenario with real BLS.
It covers retry, failure charging, renewal/replacement, retained fencing after replay
pruning, byte boundaries, revocation, bounds and lifecycle refusal. Existing verifier,
shared-budget and Wire/domain cases remain regression coverage. Focused lint, docs,
formatting and Wasm library compilation pass; compilation is not host qualification.

Shared [receiver policy](../development/receiver-policy-boundary.md) remains pure
and reusable by the forthcoming stable backend. Login/device and count/byte
preflight refuse before expensive work where appropriate. One protected verifier
profile selects complete cryptography; submitted proof tags never install authority.
Successful standalone verification still creates no session on its own.

Wire retains 37 reference frames and 296 resource thresholds against IC Wire 0.24.1.
Generated Node codec checks remain separate from SDK/browser lifecycle qualification.
Rust 1.100.0 and authenticated Toko Tooling 0.44.4 remain selected. Workspaces have
separate locks; protocol-only, certified-only and Ed25519-only capability boundaries
remain explicit. All tooling/source versions are unchanged in this source checkpoint.

Next are unknown-login denial and host readiness/recovery integration, followed by
the stable backend and cross-backend acceptance. Issuer state and browser lifecycle
remain unimplemented. Published `ic-storage` is interim; `ic-storage-new` orthogonal
persistence is the durable destination subject to its own lifecycle/rollback gates.
The original repository remains production owner. No registry publication,
deployment or live storage migration is authorized by this work.
