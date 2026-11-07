# Current handoff — 2026-11-07

## Published state

**Engine 0.35.2** (`ic-auth`) and **Protocol 0.35.0**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
Release commits and annotated tags are pushed to `origin/main`:

- Engine: `f6f55dd623ab8cb907de34a4ed413b40db119def`, tag `engine-v0.35.2`
  (`7099d193c4d2abde2f81c1b48cf8a19517d116ea`). Registry checksum:
  `7f7e6ee51ee11ebf20b86db0dd8f564fe5e2d9a3ea351a1b3f3368257def272b`.
- Protocol: `83028f07c75bb80d38867b69ae31cd9e0a2651c4`, tag `protocol-v0.35.0`
  (`fc8755ef3572af17d806aa6dfc8203f93350bc73`). Registry checksums:
  types `6204b7d777028c617275f1f63b15bf7d542d89a7b8735c952edaf6770f5421c2`,
  Wire `b1a16f6109fe1736d81512fb43e3e4e03efcb8e821e023f62d97d18f354136c3`.

Custody: `.git/publication-state/engine-0.35.2.json` and
`.git/publication-state/protocol-0.35.0.json`. Upload/readback evidence:
`target/release-frozen/batch-aSZPSweq/target/publication-engine-0.35.2.dClEoR/` and
`target/publication-protocol-0.35.0.P5GKEs/`. There is no unresolved upload.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) is closed as completed.
Canic production adoption remains owned by #491; Canic stayed read-only.

## Released contract and consumer action

Engine 0.35.2 adopts Shared Tooling 0.31.0 and qualifies its captured batch flow;
it does not change production Auth behavior, record formats or crypto.
The [Canic adoption contract](../canic-adoption.md) describes the released cohort.

The 0.35 contract supports protected login windows up to 90 days, retaining
60-second proofs and 30-minute session ceilings. Hosts select their protected
maximum login TTL explicitly; shorter policies remain supported. Known logout
retains denial through its signed immutable deadline. Unseen logout commits
bounded temporary denial and returns `Unconfirmed`, including retries. Keep work
budgets separate. No per-operation inter-canister calls or crypto relaxations
were added; ordinary authorization stays local.

The [clock-independent contract](../design/portable-clock-audit.md) retires
unqualified timestamp-based reopening. Complete durable restore plus fresh
authority remains supported; actual safety-state loss stays closed pending
trusted recovery. The fixture checkpoints never-configured, closed and retained
states. Profiling resets reinstall disposable canisters; they are not a receiver
recovery bypass. The maintainer confirmed no retained affected Rust/private
browser records. Logout Wire identity is `0x415554480014`, login storage identity
`0x415554485323`, fixture payload `AUTHCLK1`, and private journal
`ic-auth-local-logout-journal-v1`. Old bytes refuse without clearing; this does not
authorize resetting Canic's stores.

The [remembered-login client](../browser-client.md#ninety-day-remembered-login)
retains exact issuance evidence and renewal sequences separately from guard
metadata. Unresolved issuance is lookup-only; durable completion permits
browser-key renewal. Initial issuance requires account authentication. Readiness
prepares before new signing, bounds expiry recovery and never replays application
effects or renews while polling. Logout fences late results and retains uncertainty.
Issuer endpoints, durable key custody, clock mapping and Canic/Toko wiring remain
consumer-owned.

## Qualified delivery and dependency selection

Engine 0.35.2 passed all **30 release-gate stages**, including native/Wasm Clippy,
package/MSRV and boundary checks, 92 client tests/interoperability, all 11 ordinary
installed cases, local archives and release/recovery/publication fixtures.
Chromium remains opt-in/skipped. Gate:
`target/release-frozen/batch-aSZPSweq/target/release-validation/engine-gate.WFyGP0/logs/validation.SZgrt6`.
Its source is `df396bb`; `target/delivery-0352/` retains delivery/reconciliation logs.
The release-control fixture took 1110 seconds; no timing savings are claimed.
Protocol's prior complete gate remains at
`target/release-validation/protocol-gate.CAofib/logs/validation.JeBDEm`.

The release uses Platform 0.23.2, Testkit 0.89.0, Metrics 0.29.3, Build 0.75.0,
Storage 0.68.0, Wire/derive 0.17.4, Types 0.18.2, Runtime/guard 0.19.3 and Host 0.74.4.
Rust 1.99.1 remains compiler/minimum; PocketIC is 16.1.0. Shared Tooling 0.31.0
at `74313941af2fce4399bec0e6909ca6229673c3f8` has 161 verified payloads.
The release used the shared batch controller, with explicit registry publication.
Earlier release/adoption evidence remains under its original paths.

The earlier clean private browser archive at
`target/browser-client-packages/package.5x3Jeg/` retains its original 0.35 source
and receipt. It is source/archive delivery, not npm publication or production
issuer acceptance. Storage's old collection framing refuses without clearing;
fresh private fixture allocation is required. No production migration occurred.

## Adopted Shared Tooling and completed batch

Shared Tooling 0.31.0/PR #361 has 161 independently verified payloads. Evidence:
`target/evidence/shared-tooling-0.31.0-pr361/`. The release gate includes actual
candidate setup/archives, legacy recovery and schema-2 two-group batch fixtures.
Earlier focused logs remain at `target/portable-fixtures/release-tools.8R2OHK/`
and `release-tools.gF3NGy/`. Shared owns capture, explicit publication and lifecycle;
retired settings refuse without aliases. Auth's completed `batch-aSZPSweq` was
fast-forward reconciled, preserving deferred dependency hunks. Recovery remains at
`.git/release-state/frozen/batch-aSZPSweq/`; candidate and evidence are not retired.

## Clean backend-conformance source delivery

The delivered nine scenarios use the existing `Backend` hooks. Unknown logout
covers atomicity, spent work credit, fixed `Unconfirmed` deadlines, restoration
and known-login isolation; independent backing and broken-backend controls pass.

The matching Engine 0.35.2 / Protocol 0.35.0 archive passed registry-consumer
qualification and all 18 packaged tests. Receipt:
`target/portable-fixtures/backend-package.1Zq4ye/delivery.json`, clean source
`36b3f27e6d0ed3c65a1027376540eb7808744d36`, `dirty: false`, archive SHA-256
`cc09eadea778769ab0be8ce19ead1eb4e523d02a778538f493b547af07a1077c`.
Archive: that evidence directory's `target/package/ic-auth-backend-conformance-0.35.2.crate`. Retain archive and receipt
at the consumer before producer cleanup. Canic's working tree now passes the clean
0.35.1 helper on its actual stable adapter; the adoption guide records its local
browser/logout/restore progress and separate production obligations. Earlier receipts remain intact.

## Pending consumer qualification support

The working helper adds a tenth scenario using the existing `Backend` hooks:
90-day login restore, late renewal under fresh authority, unchanged login window,
second restore and before/exact/after deadline authorization without cleanup.
Both local adapters and a missing-login control pass in the 20-test package suite;
warning-denied package/all-target Clippy and local documentation links pass.
The authority matrix now adds key/mechanism changes, withdrawn scopes, expired or
shortened enrollment and a valid refresh, before/after reopen without record changes.
The adoption guide now separates confirmed/unconfirmed logout, complete restore,
missing state and older-valid-snapshot acceptance. Evidence:
`target/evidence/auth-canic-handoff-035/`. The scenario is uncommitted, outside the
delivered archive; Canic remains read-only. Production behavior is unchanged.

New native profiling adds portable claims: small/wide allocation totals are
868/14,308 bytes per call, including growth; older delegation rows are unchanged.
The fresh reference Wasm has one retained admission and authorization instantiation
each. No hashing or generic-code refactor is justified by this evidence alone;
the earlier streaming experiment was rejected. See the existing allocation guide
and `target/evidence/auth-next-035/`; no production performance saving is claimed.

The private browser journal now returns `saved`/`issue`/`renew`/`reconcile` actions
with the required signer, replacing caller reconstruction from flags. Stored
journal records are unchanged. All 93 client tests, archive checks and Wire
interop pass; the five browser contracts were regenerated/reviewed for incoming
Wire 0.17.6 (only the shared runtime changed). Allocation runners now share capture
and DHAT helpers, with complete before/after reports unchanged and the shared
sources sealed. Bench Clippy, ShellCheck and syntax checks pass. Evidence:
`target/evidence/auth-dedup-035/`. These source changes remain unpublished.

IC Build 0.76.0 is selected in the root catalog/lock and its CLI editor is prepared.
Auth tooling tests, strict package Clippy and real Engine manifest/lock rendering
pass; Protocol/external selections and original inputs remain intact. Evidence:
`target/evidence/ic-build-076/`. No release ran; prior publication custody is unchanged.

The Nov 7 read-only Canic check finds root Engine 0.35.2/Testkit 0.89.4, with private
consumers/helper still on 0.35.1. Its working snapshot guide allows captured-state
rollback with original deadlines; only the clock activation gate remains in code.
The new installed snapshot/expiry attempt stopped for unprepared Testkit 0.89.4.
Its handoff still describes a different proposed policy; the adoption guide records
this discrepancy. Final policy/docs/installed acceptance remain Canic-owned.

## Deferred updates and recovery

External updates now select Metrics 0.29.6, Platform 0.23.5, Runtime/guard 0.19.4,
Testkit 0.89.3, Storage 0.68.2, Wire/derive 0.17.6 and TOML updates. The focused
backend checks used this graph without changing its lock; it is not release-qualified.
The earlier excluded updates and released/tested locks remain under
`.git/release-state/deferred-engine-0352/`; durable branch
`deferred/upstream-20261106-engine-0352` is `1b1aca9`. The working lock retains
Engine 0.35.2. Full adoption needs its own qualification.
Earlier deferred branches remain historical; do not merge their old package-version rows.

Preserve `.git/release-state/`, `.git/publication-state/` and required target custody.
Older release gates, publication attempts and archive receipts remain historical
under their original identities; no cleanup or recovery bypass was performed.
Retired checkout refs are `archive/ic-auth-capacity` and `archive/ic-auth-release-033`,
with evidence under `target/checkout-retirement/20261104-siblings/`.
The maintainer accepted Platform's residual Wasm increase on 2026-11-05: about
29 KB over 0.21 in the matched Canic report. Preserve the
[historical scope](https://github.com/dragginzgame/ic-auth/blob/ed785d7/docs/status/current.md).
No new Wasm size comparison is claimed.
