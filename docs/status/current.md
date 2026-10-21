# Current handoff — 2026-10-21

## Source and selections

Delivered baseline is Auth **0.14.1** at
`5d7da698d3e4d297f32445fb88adbc66a79c5520`; package versions remain 0.14.1.
Pending compatible **0.14.2** notes cover private-client fixes and stronger
integration qualification. Public Rust APIs, signed bytes, Wire and storage
formats are unchanged. Incoming implementations, tests and lock edits were
preserved; this pass adds qualification code and updates notes only.

The pending lock selects published Platform/macros **0.6.6**, Memory **0.50.10**,
Wire/derive **0.5.19** and IC Tooling/Testkit **0.63.2**. It is byte-for-byte unchanged
from this pass's incoming selection. Registry source trees for Platform, its
macros and Memory are identical to their preceding selected patches; manifest
changes select release identities only. No sibling/path overrides or storage
migration are involved. Shared Tooling remains the 118-file **0.25.9** snapshot at
`85aded191d85ac4a8c0e5deab207f6be18895862`. Development uses Rust 1.99.1; the
package minimum remains 1.91.

## Pending client fixes

[Auth #83 — Expire browser logout signing capabilities when each transport attempt completes](https://github.com/dragginzgame/ic-auth/issues/83):
cleanup signers now belong to their individual bounded transport attempt. Abort
checks before/after durable admission and asynchronous signing reject retained
capabilities and late results even when consecutive claims share a coarse-clock
timestamp. The timeout regression waits for dispatch before ticking a mocked
timer, avoiding its former IndexedDB/one-millisecond race.

[Auth #84 — Bind browser admission to an immutable receiver destination](https://github.com/dragginzgame/ic-auth/issues/84):
the coordinator captures canonical receiver text before any await and uses it for
journaling, in-flight identity and signing confinement. Journal and transport get
separate SDK Principal objects; mutable backing bytes cannot retarget authority.
Retries preserve the original credential and deadline. Keep both issues open
until default-main delivery. No journal field, codec or new auth flow was added.

## Cross-subnet reference acceptance

The existing Ed25519 SDK case now places its two receiver canisters on explicitly
selected, distinct Testkit application subnets. Setup checks actual placement;
the case requires different subnet IDs. Other fixtures retain one application
subnet, including the certified-proof cost comparison. The actual SDK scenario
still covers two devices for one account, distinct account/browser identities,
lost-reply/reopen recovery, one original credential per device across receivers,
exact retry, original leases, proof-free reads and independent durable logout.
Retained raw keys and newly signed credentials cannot revive revoked logins.
Receiver admission remains local; no receiver-to-issuer admission call was added.

These are controlled clocks and a local reference host. Issuer operations remain
injected/read-only probes, and IndexedDB is faked in Node. This does not establish
production enrollment, issuer receipts, browser persistence, subnet latency/cost,
a production skew bound, native macOS, installed IC or TEE qualification. The
cross-subnet SDK scenario uses Ed25519; certified portable proofs retain their
separate existing qualification rather than gaining a new topology claim.

## Validation

Current evidence: `target/platform-066-0142/`. Focused protocol/Wire, both verifier
and session/restoration tests pass, including adversarial crypto feature
unification. Public normal/build dependency graphs pass on native and Wasm;
public libraries and reference Wasm retain their Candid exclusion. Strict Clippy
passes for the public packages and updated qualification runner. Focused Rust 1.91
public all-feature native/Wasm library checks, runner checking and fixture Wasm
checking pass. These are workspace checks, not a rerun of isolated package-payload
MSRV matrices.

`make test-client` passes all **54** tests, five exact generated-file comparisons
and three bidirectional Wire vectors. The final `make test-qualification` passes
five native fixture cases and all **11** live cases, including the two-subnet SDK
case, with the already prepared Testkit 0.63.2/server 16.1.0 pairing. Full CI was
not run. No executable installation, commit, release, publication or deployment ran.
Earlier evidence remains in `target/admission-target-0142/`,
`target/logout-attempt-0142/` and `target/client-fixes-final-check/`.

## Canic ownership and remaining acceptance

Canic's owner has adopted published Auth 0.14.1 locally and reports independent
protected-authority, restoration and maintenance-clock qualification. Production
Root enrollment, retained receiver runtime/budgets and successful scoped exports
remain unfinished in [Canic #491](https://github.com/dragginzgame/canic/issues/491).
Portable issuance, lookup-only reconciliation and browser-key issuer logout remain
[Canic #507](https://github.com/dragginzgame/canic/issues/507); future-start endpoint
acceptance remains [Canic #418](https://github.com/dragginzgame/canic/issues/418).
No new producer proof/session API blocker was established; sibling source is read-only.

[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58) and
[#59](https://github.com/dragginzgame/ic-auth/issues/59) stay open for production
issuer/browser, persistence/reset and abuse/workload acceptance. The private
client is unpublished; upgrading Rust crates alone cannot install these browser
fixes. Production skew activation remains closed, with TEE-required evidence a
separate gate. No test value supplies production approval.
