# Current status

Source package versions remain Auth **0.46.1** and protocol-types/Wire **0.42.0**.
The **0.46.2 unreleased** development checkpoint adds single-subnet installed-host
qualification and the private browser client, including durable issuance/recovery,
receiver admission and cleanup attempt tracking. Rust product sources and package versions are
unchanged. All packages remain unpublished;
0.2.0 is the last tagged milestone. Volatile Auth is the current milestone.
Stable storage is deferred; no interim `ic-storage` adapter is planned.

[Latest hosted native checks passed](https://github.com/dragginzgame/ic-auth-new/actions/runs/43854575633)
on Linux and both macOS architectures, including Linux strict checks/Wasm. The
history scan failed on six generated TypeScript type names; exact historical
exceptions are included in this checkpoint and pass locally. [Issue #1](https://github.com/dragginzgame/ic-auth-new/issues/1)
is closed. Installed and browser checks remain explicit local gates.

Product crates implement passive claims/labels, canonical schema-2 claims,
request identity/TTL, all four Wire contracts with generated codecs, and optional
Ed25519 and complete certified verification. Legacy cryptographic schemes and
canonical bytes remain unchanged; Platform 0.33.7 owns the primitives. Canic owns
application endpoint wiring. Wire retains 37 reference frames and 296 resource
thresholds against IC Wire 0.24.1. Rust remains 1.100.0; Toko Tooling 0.44.5 is now selected and authenticated.

The [memory receiver](../development/memory-receiver-boundary.md) owns bounded
caller/device/replay/account/expiry state and separate proof/unknown-logout budgets.
Admission verifies once and publishes affected records without a fallible tail or
heap clone. Exact retry, renewal, replacement, logout and indexed pruning retain
deadlines and fences. Synchronous application projections follow current session
checks; copied receipts cannot authorize after an await. Passive inspection bounds
both results and visited rows without changing clocks, budgets or storage.
Policy snapshots and decision helpers are private to their owner.

Enrollment admission binds response source, domain, receiver and challenge; checks
freshness and monotonic revision/generation; and rejects conflicting content.
Refresh cannot extend old leases, generation changes invalidate sessions while
retaining safety state, and stale enrollment still permits safe logout. The host
must authenticate the actual response, validate complete entries/terminal key
revocation and install entries atomically. Projection fields establish no provenance.
`new_fresh` never justifies replacing lost state with an empty receiver.

Native receiver evidence includes 31 Ed25519 scenarios, five complete certified
cases, two issuers/devices/receivers, bounded inspection and compile-fail API guards.
An independent model checks 353 traces/7,307 operations and rejects a deliberately
broken write-then-error adapter. Native fault injection interrupts eight mutation
paths and requires terminal closure. Signed fixtures remain independently built
with Node/OpenSSL and Platform BLS. See [compatibility](../design/compatibility.md)
for the evidence boundaries.

`issuer-memory` retains bounded Ed25519/certified issuance and browser renewal in
one private operation model. It reserves completion bytes before effects, keeps
original login windows and sequential renewal watermarks, and makes uncertain
external outcomes lookup-only. Pruning visits at most 32 indexed rows, preserves
referenced logins and closes consumed-ID epochs independently of restart fences.
Certified completion synchronously commits through host leaf/root publication;
exact retries never republish. Native issuer and signature checks, strict lint,
Rustdoc and combined/issuer-only Wasm compilation pass for the pushed checkpoint.

[Installed Auth evidence](../../qualification/ic-auth-host-conformance/README.md)
now covers five single-subnet Wasm cases with PocketIC 16.0.0. Issuer traps restore
leaf/system-root/retained completion together; actual certificates verify unchanged
claims. Receiver admission/logout and protected application writes roll back with
the enclosing trap, while ordinary bad-proof refusal retains its work charge.
Proof-expired retry preserves its original lease; logout retains replay/login fences.
Actual upgrades leave lost issuer and receiver state closed, including after a
new enrollment attempt. An intact companion admits the same valid proof and shows
why expired unknown denial is not conclusive logout. Strict native/Wasm lint and
the unchanged independent native vectors pass. No Rust product implementation was
added to obtain this installed evidence.
The harness now adopts IC Build 0.87.3 and Host FS/Artifacts 0.80.1 for one bounded
fixture build returning owned Wasm bytes and held-file vector reads.

The receiver now fetches enrollment through a real IC call to its installed
authority. A separate canister supplies caller-bound replies; mismatched/expired
responses, substituted keys and revision regression refuse. Callback traps restore
metadata and host key entries together, and terminal revocation cannot be undone
by a higher revision. Refresh preserves captured session leases; revoked authority
still permits safe logout. Request submission remains a separate committed message. Installed
late/overlapping-reply checks remain outstanding for the supported single-subnet
Canic flow; the corresponding native semantic requirements/checks remain intact.

The private [browser client](../../packages/client/README.md) now owns a single
bounded signing attempt using published Platform SDK identities. It captures
principal/destination, rechecks the injected journal guard before/after signing
and completion, and closes on success, failure, cancellation or timeout. Ten Node
cases with real SDK Ed25519 signing and strict TypeScript checking pass. The strict
IndexedDB journal now commits local invalidation and credential deletion together,
retaining pending sequences and unknown issuance. Issuance reservation, lookup-only
recovery, sequential renewal and exact credential completion now share that owner.
The retrieval operation now dispatches one bounded call through an injected issuer
adapter, binding signer role, original issuer and sequence before committing evidence.
Explicit receiver admission now commits inventory before dispatch and checks active
proof expiry/sequence around browser signing and receipt acceptance. Thirty native
Chromium cases cover two tabs, uncertain replies, aborted writes and suspended work.
Six Node codec cases check adopted credential/admission/logout output against independent
reference frames; generation drift checks pass. Actual HTTP reply authentication,
readiness coordination, remote cleanup and renderer interruption remain unqualified.

Seven additional browser cases qualify journal-owned cleanup attempts: one durable
lease across tabs, eight attempts per destination, crash/reload retention, aborted
settlement and permanent ticket closure during SDK signing. Deadline expiry leaves
unconfirmed destinations and historical issuance uncertainty visible. Confirmation
inputs remain injected. `cleanupOnce` now dispatches one reserved Wire request and
commits only a positive Revoked reply as confirmation; errors retain the counted
lease. Seven adapter browser cases, strict TypeScript and generation drift checks
pass. Actual HTTP authentication, cleanup scheduling and key disposal remain deferred.

These remain standalone Platform/Wire fixtures with one fixed test key/domain.
Complete Canic registry integration, production challenge/refresh scheduling,
browser lifecycle and accepted installed costs remain open. Stable storage/cross-backend acceptance
are a future milestone targeting `ic-storage-new` orthogonal persistence once its
contract and lifecycle/corruption/rollback qualification are ready.
The [complexity audit](../reports/audits/2026/11/15/receiver-complexity/01/report.md)
found state/index consolidation but no proved equivalent-feature LOC saving;
its public-policy finding is addressed by the narrower API.
The original repository remains production owner. This work authorizes no registry
publication, production deployment or live-storage migration.
