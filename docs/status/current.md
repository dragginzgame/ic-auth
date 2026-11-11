# Current handoff — 2026-11-11

## Delivered state

Protocol and Engine **0.39.0** are published on crates.io: `ic-auth-protocol-types`,
`ic-auth-wire` and `ic-auth` are non-yanked, with registry checksums matching the
exact locally retained Cargo archives and schema-2 publication receipts.
Both annotated tags and release commits are on `origin/main`; Shared's batch
reconciliation fast-forwarded the normal checkout without discarding other work.

- Protocol: `44fcdd86f3db07c0499349e83e14f6b8d4dc9b15`, `protocol-v0.39.0`.
- Engine: `c701eb6864306c9f8a4d9d5cab33bbebca19ece8`, `engine-v0.39.0`.
- Completed batch: `batch-t5q4t2DE`, source `ceb3b96ff00535eb9e8bfb0a91945db5a56883b8`.
- Registry readback: `target/auth-release-0390/registry/verified-receipts.json`.

Both exhaustive 30-stage source-bound gates passed, including native/Wasm lint,
compiler/package admission, crypto/session/browser checks, installed canister
certificates/upgrades/ingress/logout, all three local archives and release/recovery
caller fixtures. Actual Chromium remains unprepared/ignored; do not claim it ran.
Hosted CI runs asynchronously and is separate from these completed local gates.
No standalone Auth service or Canic deployment occurred.

## Pending Testkit qualification

Published Testkit **0.100.2** is selected in the qualification catalog/lock.
The native runner uses its `ic_testkit::pocket_ic` re-export; Auth's direct client
declarations are removed. This implements
[#131 — Use Testkit's PocketIC re-export](https://github.com/dragginzgame/ic-auth/issues/131)
locally, without changing upstream client identity or transport/teardown behavior.
Both locked metadata graphs, dependency boundaries, runner Clippy, formatting and
installed qualification passed (11 tests; optional Chromium ignored). Evidence:
`target/testkit-0101-adoption/`. The lock-selected CLI/server is prepared.
Protocol and Engine 0.39.1 delivery is authorized. The incoming product-lock
Platform 0.31.1 selection is preserved for full release qualification; the
independent qualification lock retains Platform 0.31.0. The new prepared-root API
is not selected by Auth.

## Bounded multi-device contract

Schema 2, introduced in 0.38, signs `device_slot` (0–7) and nonzero `device_generation`; schema 1
refuses. Up to eight devices can coexist per issuer-qualified `(issuer, subject)`.
Hosts can select a tighter limit before admission/restore and must persist it.
Renewal preserves the binding and original window. Verified higher generations
with fresh browser keys replace a slot atomically; stale rebinding and wrap refuse.
See [the session contract](../protocol/sessions.md) and
[Canic adoption](../integration/canic-adoption.md) for the complete requirements.

- Known slot fences survive logout and expiry. Host-owned pruning removes expired
  sessions/replays and unknown denials, never known slot fences. Retained capacity
  bounds accounts; safe account retirement remains an explicit host decision.
- Durable admission deletes `login.displaced_caller()` from old session/login rows
  in the same transaction as new records/indexes/totals, retaining replay history.
  The accessor is transaction-only, absent from encoded checkpoints. Failed
  commits preserve the original device; old-key logout cannot remove its replacement.
- Replay rows retain `(issuer, subject)` independently of sessions. Account-wide
  quotas survive logout, key changes and restore; durable adapters must implement
  the account-based occupancy contract. Corrupt associations refuse restoration.
- The maintainer authorized retiring old Auth records. Session/replay/login IDs are
  `0x415554485341`, `0x415554485342`, `0x415554485343`; fixture checkpoint identity
  is `AUTHSLOT` / `0x41555449f102`. Old profiles refuse; no implicit deletion or
  fallback reader. This never authorizes deleting Canic's protected safety state.
- Cryptographic algorithms and verification checks remain upstream-owned. Signed
  claims change; public Wire envelopes remain unchanged. No hidden inter-canister
  call, timer or certification-root publication was introduced.

## Canic boundary and next work

This session changed no Canic files. The last read-only inspection observed
Engine 0.36.1 / Protocol 0.36, with caller-based replay accounting and global
issuer ceilings rather than account slot allocation. Adoption requires coordinated
supplier identities, protected issuer slot/generation custody and retry/callback
fences, durable receiver adapter updates, an explicit retained-state decision and
installed consumer qualification. See the linked adoption guide;
[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns integration.
Auth #130's entropy contract was delivered previously; Canic #757 owns its adapter.
90-day login windows still use short proofs and 30-minute receiver leases;
renewal cannot extend the original window and lost protected state stays closed.

## Tooling and dependencies

Delivered Protocol/Engine **0.39.0** selects Tooling 0.39.10, Wire 0.21.1,
Build 0.83.0, Host libraries/file CLI 0.76.4 and Platform 0.31.0. Both locks share
Types 0.19.3; the private host retains one Runtime/macros/guard 0.21.1. Required
Testkit 0.100 clears the prior Wire pairing blocker. Storage 0.71.1 and Metrics 0.29.9
are unchanged. The dependency batch and earlier changelog cleanup are delivered.
Six qualification packages retain their independent workspace/versions; four
product packages remain under `crates/`. No sibling files or releases changed.
The incoming product-lock `ar_archive_writer` 0.5.5 selection passed both complete
release gates. No source or lock updates remain deferred from this delivery.

The native runner now uses upstream PocketIC 16.1 through Testkit's native-only
re-export. Testkit retains managed startup,
lifecycle and typed Wire helpers. Principal boundaries copy validated bytes.
Upstream transport panics propagate; its 30,000 ms polling setting is not an
absolute HTTP/body deadline, and awaited deletion is best-effort with retained logs.
Wire retains Postcard redundant-varint acceptance and canonical method budgets.
All five generated browser files were reviewed: Auth's four explicitly named
bindings are unchanged; the shared runtime updates alias-cycle detection.
Platform's BLS wrapper/errors now have Platform identity; Auth retains its signed
bytes, verification policy and trust inputs. Tooling admits the supplier's scoped
Digest adapter; Auth adds no direct Digest dependency or cryptographic code.

Focused adoption evidence remains under `target/upstream-followup-20261111/`;
the complete release evidence below supersedes it for delivery. Each installed
gate passed 11 cases, with optional Chromium ignored. The pre-release normal-checkout
fixture was 1,760,067 bytes (code payload 1,361,483), versus retained
1,766,564 / 1,368,974 under the same fixture/build profile. This aggregate
dependency observation is not an isolated supplier gain or published artifact size.
Keep prior observations under `target/device-contract-*`, `target/upstream-20261111/`,
`target/platform-0300-adoption/`, `target/wire-0210-adoption/` and
`target/tooling-adoption-0398/`. [Developer setup](../development/setup.md) and
[the docs index](../README.md) own the current commands and topic entrypoints.

## Evidence and recovery custody

Keep `target/release-frozen/batch-t5q4t2DE/` and durable
`.git/release-state/frozen/batch-t5q4t2DE/`, including `repository.git/publication-state/`.
Gate roots beneath that checkout are `target/release-validation/protocol-gate.1PIBBY/`
and `target/release-validation/engine-gate.HlyTG7/`. Delivery log:
`target/auth-release-0390/delivery.log`. Both gates passed all 30 stages.

No batch or recovery evidence was retired. Keep older `batch-sat4DSEt`, failed
`batch-4hebGSFP` and `batch-yiX0dEoV` custody and
`target/publication-recovery/`. An earlier external cleanup deleted old raw evidence;
historical measurements cannot replace those bytes. Do not resume earlier failed
batches against already delivered versions. The previous detailed handoff and
cleanup account are retained in Git at `c701eb6864306c9f8a4d9d5cab33bbebca19ece8`;
[history](history.md) retains older observations with their original scope.
