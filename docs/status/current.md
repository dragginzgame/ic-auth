# Current handoff — 2026-11-07

## Published state

**Engine 0.35.4** (`ic-auth`) and **Protocol 0.35.0**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
Their release commits and annotated tags are pushed to `origin/main`.

- Engine: `0b431149bbcbcd9a9f1c445ac382bff38ab9b3a6`, tag `engine-v0.35.4`
  (`c65dd425ff46b2f803525716ce04c653d6413a88`). Registry checksum:
  `771bb2b46a608190710df260f12448bb50621910f42b0230e45126a40700ca09`.
- Protocol: `83028f07c75bb80d38867b69ae31cd9e0a2651c4`, tag `protocol-v0.35.0`
  (`fc8755ef3572af17d806aa6dfc8203f93350bc73`). Registry checksums:
  types `6204b7d777028c617275f1f63b15bf7d542d89a7b8735c952edaf6770f5421c2`,
  Wire `b1a16f6109fe1736d81512fb43e3e4e03efcb8e821e023f62d97d18f354136c3`.

Custody: `.git/publication-state/engine-0.35.4.json` and
`.git/publication-state/protocol-0.35.0.json`. The Engine upload/readback is at
`target/release-frozen/batch-mhsYrIQc/target/publication-engine-0.35.4.MxWRc3/`.
The batch is complete and fast-forward reconciled; its durable recovery remains
at `.git/release-state/frozen/batch-mhsYrIQc/`. No upload is unresolved.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) remains closed.

## Completed Engine 0.35.4 delivery

[#128](https://github.com/dragginzgame/ic-auth/issues/128) is closed as completed
after verified publication. Its fix and incoming
updates were preserved in `deferred/auth-128-20261107` (`47c776f`) before
reconciling 0.35.3, then reapplied without reverting local package versions.
The released source fixes its only PocketIC constructor with
`PocketIcBuilder::new(Duration::from_secs(30))`. Startup retains its separate
30-second budget; managed instance/server teardown remains explicit and async.
No public Auth protocol, crypto, browser or stored-record contract changes.

The released graph is Build 0.77.0, Testkit 0.90.0, Platform 0.23.5,
Types 0.18.2, Runtime/macros/guard 0.19.5, Storage 0.68.3, Metrics 0.29.6,
Wire/derive 0.17.6 and Host 0.74.4 (file CLI 0.74.2). Platform 0.23.5 was the
latest published facade at admission. Testkit's registry checksum is
`aeeb9dd0920b5a4343aed61f2cb7c0266488823dab5f7b19e0b632b54b7d8f5a`.
The public package versions are Engine 0.35.4 / Protocol 0.35.0.

Focused Linux evidence recreated after cleanup:

- `target/evidence/platform-testkit-20261107/`: warning-denied runner Clippy,
  reference Wasm build, all 11 ordinary installed cases, formatting, ownership,
  dependency pins and documentation links pass. The opt-in Chromium case is
  ignored; native macOS was not run. One Runtime executor/guard remains in Wasm.
- `target/evidence/ic-build-077/`: bounded-lock admission, strict tooling-binary
  Clippy, real Engine manifest/lock rendering and input-limit refusal pass.
  Auth does not use the removed cache-report getters. Protocol and all external
  selections survive rendering unchanged.

The fresh complete **30-stage Linux release gate** passed against captured source
`dd4004ba64c4477f3bbcd30cac8a5e0b0eb37774`, including all 93 client tests,
20 backend-conformance tests, 11 ordinary installed cases and complete archives,
MSRV and release/recovery/publication callers. Gate:
`target/release-frozen/batch-mhsYrIQc/target/release-validation/engine-gate.W4Va0C/logs/validation.Qmv1B0`.
The final release-tools stage took 1056 seconds; no timing saving is claimed.
Both completed batch journals and their evidence remain retained.

Platform 0.23.6 arrived in the normal lock during this gate, together with
crypto-common 0.1.7 and generic-array 0.14.7. It did not enter either captured
release. Preserve this pending update on `deferred/upstream-20261107-engine-0354`
(`2a6723a`) and under
`.git/release-state/deferred-engine-0354/incoming-after-capture/`.
It is reapplied after clean delivery qualification; the working lock's later
selection is not the released or qualified graph. A future batch must qualify it.

## Clean backend helper delivery

The Engine 0.35.4 / Protocol 0.35.0 helper archive passed registry-consumer identity
checks and all 20 packaged tests from clean release source
`0b431149bbcbcd9a9f1c445ac382bff38ab9b3a6` (`dirty: false`). Receipt:
`target/portable-fixtures/backend-package.mEpEDB/delivery.json`. Archive:
that evidence directory's `target/package/ic-auth-backend-conformance-0.35.4.crate`.
SHA-256: `a768350ba06a5eb6c73049d5ee4603b8b6139122e2d4c71a40d6098e351233b0`.
The ten scenarios retain the existing `Backend` hooks, including 90-day renewal
and expanded authority checks. Canic must retain the archive and receipt in its
own custody and qualify its actual adapter; this is supplying evidence only.

## Completed 0.35.3 qualification and recovery

External cleanup deleted the entire normal `target/` during the first final
release-tools stage. Durable Git state survived. The exact clean candidate was
reconstructed from its verified index; tools were explicitly prepared and the
same batch resumed. No deleted proof was reused. A fresh complete **30-stage**
Linux gate passed against captured source `d1f7b8c`:
`target/release-frozen/batch-vgrJaHrk/target/release-validation/engine-gate.Z9veMt/logs/validation.w5XTFO`.
This includes native/Wasm Clippy, MSRV/archives, boundaries, all 93 client tests,
Wire interoperability, 20 backend-conformance tests, all 11 ordinary installed
cases and real release/recovery/publication callers. Chromium stayed opt-in.
The final release-tools stage took 1067 seconds; no timing saving is claimed.

The captured graph used Build 0.76.0, Testkit 0.89.4, Runtime/guard 0.19.4,
Storage 0.68.2, Platform 0.23.5, Types 0.18.2, Metrics 0.29.6 and Wire 0.17.6.
Rust 1.99.1 remains compiler/minimum; PocketIC is 16.1.0. Shared Tooling 0.31.0
at `74313941af2fce4399bec0e6909ca6229673c3f8` has 161 verified payloads.
Delivery/reconciliation logs: `target/delivery-0354/`.

Engine 0.35.3 delivers the private journal's explicit
`saved`/`issue`/`renew`/`reconcile` credential actions, unchanged stored journal
bytes, shared allocation-profile capture/DHAT helpers and regenerated Wire 0.17.6
browser runtime. Its unpublished backend helper adds a tenth scenario: 90-day
login restore, late renewal under fresh authority, immutable deadlines and a
second restore. The expanded authority matrix covers key/mechanism changes,
withdrawn grants and authority/enrollment expiry before/after reopen.
Earlier delivered source archives keep their original scenario sets; the new
qualified archive above carries the expanded set.

## Auth contract and Canic adoption

The [Canic adoption contract](../canic-adoption.md) and
[clock-independent contract](../design/portable-clock-audit.md) separate reusable
Auth machinery from consumer authority, issuer policy, persistence and recovery.
Protected login windows can last up to 90 days; proofs stay short-lived and
session leases retain their 30-minute ceiling. Renewal cannot extend the original
login deadline. Known logout denies through that deadline. Unknown logout commits
bounded temporary denial and returns `Unconfirmed`, including retries. Ordinary
authorization stays local; no per-operation inter-canister call was added.

The [private client](../browser-client.md#ninety-day-remembered-login) retains exact
issuance evidence and renewal sequences, keeps unknown issuance lookup-only and
fences late results after logout. No production issuer or npm publication is
implied. Actual loss of protected safety state stays closed pending trusted
recovery; complete durable restore plus fresh authority remains supported.
Old Rust/browser formats refuse without clearing, under the maintainer's prior
no-retained-record approvals. Those approvals do not authorize erasing Canic state.

The earlier Nov 7 read-only Canic review found root Engine 0.35.2/Testkit 0.89.4,
private consumers and retained helper on 0.35.1, and snapshot policy allowing
captured-state rollback subject to original deadlines. Its new installed
snapshot-expiry attempt stopped for unprepared Testkit; its handoff described a
different proposed policy. Final policy/docs/installed acceptance remains owned
by [Canic #491](https://github.com/dragginzgame/canic/issues/491). Canic stays read-only here.

## Evidence retention

Before the cleanup incident, raw evidence and archives existed under earlier
`target/` paths recorded in Git history. Those files were deleted and cannot be
independently verified locally now. Committed source, published releases, Git-owned
publication receipts and earlier deferred branches remain retained. Newly recreated
paths above are current. Do not clean `target/` during delivery or erase journals.
Keep bulk artifacts out of `.git`; only source, small custody records and refs
belong there. No new Wasm performance or production-adoption claim is made.
