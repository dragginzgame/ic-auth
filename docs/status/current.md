# Current handoff — 2026-10-28

## Delivered Auth base

Auth **0.19.4**, `27c83de897bcfa29b308994bd565eaf9626b16ad`, is pushed;
the published package was observed in the registry during Canic adoption.
The preceding availability/tooling evidence is preserved in the
[release's handoff](https://github.com/dragginzgame/ic-auth/blob/27c83de897bcfa29b308994bd565eaf9626b16ad/docs/status/current.md).
Auth's integration documentation and dependency-adoption edits remain uncommitted
for the user; Auth package versions are unchanged.

Delivered 0.19.4 selections were Platform/macros **0.7.1**, Host/Build/Testkit
**0.69.4**, Memory **0.51.13**, Wire/derive **0.7.18**, browser Platform npm **0.1.1** over Core
**6.1.1**, and PocketIC **16.1.0**. Shared source is exact
`cc81a58a5b2dd287c577cbffd2c576c178b6785c`, VERSION **0.27.3**, **127 files**.
Rust **1.91** remains the package minimum; development uses **1.99.1**.

## Pending dependency adoption

Current selection: Memory/Storage **0.53.0**, Host/Build/Testkit **0.69.6**,
Wire/derive **0.7.19**, Platform/macros **0.7.3**. The Memory/Storage root
requirements are `0.53`; the other compatible ranges remain. Exact Testkit and
release-editor executables are explicitly prepared; PocketIC stays **16.1.0**.
[#103](https://github.com/dragginzgame/ic-auth/issues/103) owns the latest batch.

The unpublished host uses Storage's governed `StableCell` over its one admitted
Memory allocation. Auth retains its pre-load 128 KiB ceiling, occupied-fresh-state
refusal and exact checkpoint bytes. Memory 0.53 trusts exclusive backing custody
between construction/lease-open/growth checks; live cell/detached handles retain
the backing claim. No raw alias is used during collection access. Public graphs
and passive Wire controls explicitly refuse Storage. Auth verification, public
APIs, signed bytes and TEE trust remain unchanged.

Latest Linux evidence: **six** checkpoint cases, **four** native tooling cases,
external-lock/package refusal fixtures, fixture Clippy with warnings denied,
release Wasm build, Rust **1.91** host/Wasm check, dependency boundaries/pins and
fresh browser-contract comparison pass. **Four** installed PocketIC cases also
pass: certificates/composed roots, certified proof authority after upgrade,
retained session/denial checkpoints and real signed ingress after upgrade.
Logs and the prior Wasm remain in `target/persistence-adoption-20261028/`.

[#102](https://github.com/dragginzgame/ic-auth/issues/102) retains the preceding
Memory 0.52.1/Tooling 0.69.5 evidence: **11** transport/control, **13** signature,
**68** browser cases and Rust/TypeScript roundtrips, plus four installed upgrade/
ingress cases. Its five generated browser files were reviewed; only the Wire
runtime changed. Earlier logs, including the corrected evidence-directory refusal,
remain in `target/dependency-adoption-20261028/`. Neither batch establishes Canic
production adoption or native macOS qualification; no full local CI ran.

## Persistent Canic receiver integration

The maintainer selected **persistent receiver integration first**, independently
of issuer custody. [Canic #491](https://github.com/dragginzgame/canic/issues/491)
remains the owning integration/acceptance tracker; issuer issuance and lookup-only
reconciliation remain in [#507](https://github.com/dragginzgame/canic/issues/507).

[Canic PR #701](https://github.com/dragginzgame/canic/pull/701), source
`3c559fd6e37f246e0dc8e432139416077e234ddc`, is open from released Canic
`5c236ab130c0d63892ef281b45bdc3fa353e012f`. Its internal persistent receiver
component restores the full logout ledger against checkpoint clock/counts,
retains independent proof/unknown-logout budgets and fences stale owners.
Bounded owner-mediated maintenance preserves the ledger; returned errors retain
clock advancement and work charges. Its lock selects published Auth/protocol types
**0.19.4**. No stored format, signed bytes, verifier or TEE/non-TEE policy changes.

Evidence in `target/canic-integration-20261028/`: **40** focused native
application-session storage tests, normal native and Wasm receiver graphs without
dev dependencies, warning-denied package/test Clippy, formatting and local links
pass. Six new cases use Auth's real Ed25519 verifier. These are native restoration
and compile proofs, not installed upgrade or certified-proof receiver acceptance.
The first hosted run exposed missing metadata-editor preparation, a macOS FIFO
fixture API and a stale embedded peer. The PR repairs all three without bypassing
admission: the Linux FIFO case, all **17** release-flow cases, CLI Clippy,
workflow lint and the normal regenerated-peer verifier/corruption case pass.
Fresh hosted checks remain asynchronous, including both macOS architectures.

[Canic PR #703](https://github.com/dragginzgame/canic/pull/703), source
`0ab0d17398d1896dfed63a79fc83b4d2f695929f`, is stacked on #701. It adds bounded
Root enrollment snapshots, a persistent monotonic receiver checkpoint and an
opaque direct-Root prepare/fetch/finish pipeline. Installation binds authority,
receiver, Fleet and boot challenge; rejects rollback/equivocation and forgotten
retirements; and atomically advances local session generation with authority
changes. Restore closes readiness. Snapshot reads write no stable state, and the
asynchronous pull holds no mutable receiver borrow.

Passing local evidence: **15** selected enrollment cases (nine new), **two**
Wire/schema cases, **40** storage cases, **11** state-descriptor cases and **27**
role-catalog cases; normal base/receiver/Root-only native/Wasm graphs and
warning-denied Core/Contracts Clippy. Its own embedded peer was regenerated and
normally verified. These native ownership tests do not poll the IC-call future
and do not establish IC caller authentication or installed lifecycle acceptance.
The Root method identity is reserved, not mounted. Both PRs remain open; #491
records the evidence and remaining acceptance.

The isolated clone is `target/canic-integration-20261028/canic` on
`integration/receiver-enrollment-20261028`. The normal Canic checkout and its
unrelated work were preserved. Standing cross-repository authority covered its
scoped commit, topic push, PR and owning-issue evidence; no integration-branch
push, merge, package-version change, release, publication or deployment ran.

## Remaining boundary

The production facade still denies. Root RPC mounting, approved issuer
registration/key custody, runtime refresh scheduling, lifecycle/Wire endpoint
wiring and actual persistent upgrade/rollback/outage qualification remain.
The host must obtain caller/time, supply fresh boot randomness and qualified
clock-skew/budget policy, and validate fresh enrollment for admission/authorization. Logout
must remain independent of enrollment expiry. No production policy values or new
configuration stanza were invented. The private browser client remains unpublished.
See the [adoption contract](../canic-adoption.md).

Keep the accepted certified verifier and its admission cost. No verification
cache or crypto/trust optimization follows; the
[operational profile](../design/0.6-portable-authentication.md#certified-proof-operational-profile--2026-10-27)
retains that decision. Preserve build and release/recovery evidence. Integration
status stays in the owning GitHub issues, not a second task queue here.
