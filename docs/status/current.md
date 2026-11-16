# Current status

The current source checkpoint is Auth **0.46.1**; protocol-types/Wire remain
0.42.0. This adds the volatile issuer, browser renewal, bounded retention and
synchronous certified publication described below.
All packages remain unpublished. Version 0.2.0 remains the last tagged milestone.
The current milestone is volatile Auth. Stable storage is explicitly deferred;
no interim `ic-storage` adapter is planned.

[Hosted CI passed](https://github.com/dragginzgame/ic-auth-new/actions/runs/43770028168)
for Linux, Apple Silicon and Intel macOS, including Linux strict checks/Wasm
compilation and one redacted history scan. [Issue #1](https://github.com/dragginzgame/ic-auth-new/issues/1)
is closed. This hosted evidence covers 0.45.0; 0.46.0 local evidence is recorded below.

Product crates implement passive claims/labels, canonical schema-2 claims and
request identity/TTL, all four Wire contracts with generated codecs, and optional
Ed25519 and complete certified verification. Legacy cryptographic schemes and
canonical signed bytes remain unchanged; Platform 0.33.7 owns the primitives.
Canic owns endpoint wiring. Wire retains 37 reference frames and 296 resource
thresholds against IC Wire 0.24.1. Rust 1.100.0 and Toko Tooling 0.44.4 are unchanged.

The optional [memory receiver](../development/memory-receiver-boundary.md) owns
bounded caller/device/replay/account/expiry state and separate retained proof and
unknown-logout budgets. Admission verifies once and publishes affected records
without a fallible tail or heap clone. Exact retry, renewal/replacement, proof-free
reads, known/unknown logout and indexed pruning preserve deadlines and fences.
Synchronous `authorize` projections run only after current session checks; copied
receipts do not replace reauthorization after an await.
Unknown logout remains unconfirmed, including after a committed denial.
Passive inspection caps results and visited rows, advances cursors across empty
pages, and observes retained facts without changing clocks, budgets or storage.

Auth 0.45.0 replaces bare activation with bound enrollment request/response
metadata. It checks installed response source, receiver/domain/challenge, local
receipt deadline, bounded freshness, monotonic revision/generation and conflicting
content. Admission/read require current accepted enrollment; its deadline caps
stored leases. Refresh cannot extend old leases, generation changes invalidate
sessions without clearing safety state, and stale enrollment still permits safe
logout. Closed/poisoned instances cannot reopen through enrollment.

The host still authenticates the actual IC response, validates complete authority
entries and terminal key revocation, computes the metadata digests and installs
entries atomically with receiver metadata. Plain projections do not prove their
provenance. `new_fresh` is only for genuinely new state; it cannot justify replacing
lost replay/revocation evidence with an empty receiver. Installed boot, complete
recovery and host rollback remain unqualified.

Local evidence: 31 focused memory Ed25519 scenarios, an instrumented inspection iterator
and five real certified receiver cases pass, including seven readiness and four
inspection cases. New projection and two-issuer/multi-device cases check callback
refusal, quota isolation, unaffected devices and receiver-local logout. Two
compile-fail cases fence borrowed projection receipts; two more fence the private
policy API. An independent model adds 353 traces/7,307 operations across
two receivers; it also detects a deliberately broken write-then-error adapter.
Two publication-fault tests interrupt eight mutation paths and require terminal
closure after every injected partial publication. Hooks exist only in test builds.
Actual host rollback remains unqualified; stable-adapter evidence is deferred.
Focused memory/fixture checks, strict product/qualification lint, Rustdoc and
Wasm compilation pass for 0.46.0; compilation does not establish installed-host
lifecycle behavior.
The signing fixtures remain independently constructed with Node/OpenSSL and BLS.

Auth 0.46.0 adds certified multi-issuer/device/receiver lifecycle qualification:
seven independently packed claims and a second signed root authorization exercise
quota isolation, failed proofs, replacement, retry, renewal and logout. Certified
refresh/generation cases preserve captured leases and retained fences; unknown
logout before admission installs no identity. Receiver behavior is unchanged.
The unpublished Rust API is narrower: policy snapshots/decisions/helpers are
private; host types and inspection results are exposed through `receiver_memory`.
All nineteen policy tests moved intact beside their private owner. Two synthetic
proof-to-facts drivers were replaced with mapped real receiver coverage, including
certified replacement of a denied slot. No public testing facade was added.

`issuer-memory` implements bounded Ed25519/certified issuance and browser renewal:
exact intent, nonce/approval, one Ed25519 dispatch or certified publication,
reconciliation and retained completion.
One account owner holds immutable logins and private operation progress; the module
requires no receiver, verifier, storage or runtime feature. It reserves completed
claims and applicable signature bytes before effects and starts credit full.
Account authentication, authority, effects and transactions stay host-owned.
Renewal uses the authenticated browser, immutable login window and sequential
completion watermark. One lookup index points to the shared operation records;
missing evidence cannot recreate consumed sequences. Read-only preflight does
not reserve work; preparation rechecks the binding after awaits. Independent
Wire-frame/hash vectors preserve the existing renewal operation identity.
Indexed pruning removes at most 32 operations/logins per call after captured
reconciliation retention. It releases exact result-byte reservations, preserves
referenced logins and sequence watermarks, and closes consumed-ID epochs without
invalidating unrelated live callbacks. Slot reuse advances generations. Explicit
restart has its own callback fence and does not refill credit.
Certified completion now invokes one synchronous host leaf/root publication
callback before retaining completion and its shared renewal watermark. Protected
authority captures the mechanism and seed. Refusals precede the callback; host
error/unwind or partial local publication closes the instance. Completed retries
never recreate a leaf, even after expiry/revocation/restart or host leaf loss.
Thirty-six public scenarios, nine private tests and a full certified issuance
fixture cover the slice. Independent Node/OpenSSL and Platform BLS proofs check
unchanged claims and refreshed witnesses with the same credential identity.
This proves neither installed-host rollback nor recovery. Expired rows remain
charged until explicit pruning; the host owns leaf retention and witness assembly.
Focused issuer and issuer-signature checks, strict Clippy, Rustdoc, formatting and
combined/issuer-only Wasm compilation pass. Hosted CI for 0.46.1 is pending.

Remaining current work includes host enrollment/loss handling, browser lifecycle
and installed issuer/receiver host qualification. Stable storage and
cross-backend acceptance are a future milestone. `ic-storage-new` orthogonal
persistence remains the intended durable destination once its contract is ready,
subject to its own lifecycle, corruption-admission and rollback qualification.
The [receiver complexity audit](../reports/audits/2026/11/15/receiver-complexity/01/report.md)
found real state/index consolidation but no proved equivalent-feature LOC saving.
Its public-policy finding is addressed by the API narrowing. The design records
the issuer's single operation owner, private progress and preserved/deferred
capabilities; native issuance/renewal do not complete issuer acceptance.
The original repository remains production owner. No registry publication,
deployment or live-storage migration is authorized by this work.
