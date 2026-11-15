# Current status

[0.2.0](https://github.com/dragginzgame/ic-auth-new/releases/tag/v0.2.0) remains the
last tagged milestone. The completed Auth 0.43.0 memory receiver was pushed in
[285ac17](https://github.com/dragginzgame/ic-auth-new/commit/285ac178cf8df9f10467368d51c117aa7853d5a3).
Post-push review found no actionable issue in the delivered admission, accounting,
verifier integration or affected checks. Local development is now Auth 0.44.0;
unchanged protocol-types/Wire remain 0.42.0. All crates remain unpublished.

Product crates supply passive claims/labels, canonical schema-2 claims and request
identity/TTL, all four Wire contracts with generated browser codecs, and optional
Ed25519 and complete certified verification. Canic owns endpoint wiring. See the
[Wire](../development/wire-boundary.md), [Ed25519](../development/verification-boundary.md)
and [certified](../development/certified-verification-boundary.md) boundaries.
Legacy cryptographic schemes, signing domains and canonical bytes are retained;
Platform 0.33.7 owns primitives and IC certificate verification.

The [memory receiver](../development/memory-receiver-boundary.md) is implemented
behind `receiver-memory` plus at least one complete verifier feature. It owns bounded
caller/device/replay/account/expiry state and retained work budgets. Admission decodes
once, checks shared retry/login/capacity policy, verifies the complete proof and
publishes affected records without a fallible tail or heap clone. Failed proof work
retains its charge without reserving or displacing a login.

Exact retry preserves the original receipt after proof expiry or replay pruning.
Proof-free reads check current authority and retained login/device association.
Renewal and higher-generation replacement preserve physical replay quotas and
immutable login deadlines. Known-login logout removes the session and retains
denial/fences/replay. Unknown logout now uses a separate bounded denial pool and
initially empty operation bucket. Live retries retain their deadline and remain
unconfirmed. Expired unknown rows transfer only after verified admission or explicit
pruning; flood traffic cannot consume known capacity or proof credit. The two
budgets share refill arithmetic while retaining independent state.

Memory starts closed with empty credit. Fresh activation is a host assertion for
a genuinely new receiver, not recovery from lost state; close is terminal.
Boot authentication, lost-memory recovery and actual host/endpoint integration
remain open. Qualification projects logout outcomes to the unchanged Wire frames.
The host owns authenticated caller/time/enrollment, transaction isolation and
abort/rollback. Native failure injection and installed IC lifecycle evidence are
still required before claiming those integration guarantees.

`make test-memory` exercises seventeen public-API Ed25519 scenarios using seven
independent Node/OpenSSL credentials and one certified-only receiver scenario with
real BLS. It covers retry, failure charging, replacement, replay/device fencing,
known/unknown logout, pool and budget isolation, future credentials, byte bounds,
pruning and lifecycle refusal. Existing verifier/shared-budget cases remain
regression coverage. Focused lint, docs, formatting and Wasm library compilation
pass; compilation is not installed-host qualification.

Shared [receiver policy](../development/receiver-policy-boundary.md) remains reusable
by the forthcoming stable backend. One protected verifier profile selects complete
cryptography; submitted proof tags never install authority. Successful standalone
verification creates no session on its own.

Wire retains 37 reference frames and 296 resource thresholds against IC Wire 0.24.1.
Generated Node codec checks remain separate from SDK/browser lifecycle qualification.
Rust 1.100.0 and authenticated Toko Tooling 0.44.4 remain selected. Workspaces have
separate locks; capability boundaries remain explicit. Tooling and supplier versions
are unchanged. Only Auth advances to 0.44.0.

Hosted [Auth CI](../../.github/workflows/ci.yml) now selects Linux and both macOS
architectures for native contracts and independent fixture verification, with
Linux formatting/lint/Rustdoc/Wasm compilation and one redacted history scan.
Local CI aggregates, workflow validation and secret scan pass. Hosted evidence
is pending; [issue #1](https://github.com/dragginzgame/ic-auth-new/issues/1) remains
open until the selected hosted checks pass.

Next are host readiness/recovery integration, then the stable backend and
cross-backend acceptance. Issuer state and browser lifecycle remain unimplemented.
Published `ic-storage` is interim; `ic-storage-new` orthogonal persistence is the
durable destination subject to its own lifecycle/rollback gates. The original
repository remains production owner. No registry publication, deployment or live
storage migration is authorized by this work.
