# Current handoff — 2026-11-10

## Delivered state

Protocol and Engine **0.38.0** are published on crates.io: `ic-auth-protocol-types`,
`ic-auth-wire` and `ic-auth` are non-yanked, with registry checksums matching the
exact locally retained Cargo archives and schema-2 publication receipts.
Both annotated tags and release commits are on `origin/main`; Shared's batch
reconciliation fast-forwarded the normal checkout without discarding other work.

- Protocol: `1bd5ff4396d48ded6ae8367de7c806d6438e397f`, `protocol-v0.38.0`.
- Engine: `2b8fa629a7c52d064dd9e617b8502efcb6036f2b`, `engine-v0.38.0`.
- Completed batch: `batch-sat4DSEt`, source `94301322d03b861d0bde9f2071ba83680e04c9fb`.
- Registry readback: `target/auth-0380-registry-verification.json`.

Both exhaustive 30-stage source-bound gates passed, including native/Wasm lint,
compiler/package admission, crypto/session/browser checks, installed canister
certificates/upgrades/ingress/logout, all three local archives and release/recovery
caller fixtures. Actual Chromium remains unprepared/ignored; do not claim it ran.
Hosted CI runs asynchronously and is separate from these completed local gates.
No standalone Auth service or Canic deployment occurred.

## Bounded multi-device contract

Schema 2 signs `device_slot` (0–7) and nonzero `device_generation`; schema 1
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

The pre-release fixture grew 27,528 bytes (1.48%) to 1,886,577; ordinary installed
session authorization measured 4,676 instructions. These retained observations
under `target/device-contract-*` are not production fees or a matched published
release comparison. Both memory and independent encoded backends, plus the
Storage-backed fixture upgrade, were qualified.

## Canic boundary and next work

Canic was inspected read-only and remains unchanged. The observed checkout selected
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

Both catalogs/locks select Toko Tooling **0.39.7**; the authenticated snapshot has
108 managed files and no managed reference-doc/rule/task copies. Required
release helpers remain installed; `TOKO_TOOLING_RELEASE := 1` retains the release
surface. Consumer `make tasks` forwarding is retired. Canonical root license and
contribution files match the selected package; required source notices remain in
packaged READMEs and private client archives. [Developer setup](../development/setup.md)
and [the docs index](../README.md) own current command and documentation entrypoints.

Delivered product Platform is 0.29.2; qualification retains 0.29.1 with Runtime
0.21.0, Testkit 0.97.0, Storage 0.71.1 and Metrics 0.29.9. Both use Types/Wire 0.19.0,
Host 0.76.2 and the explicit SHA-2 0.11.1 selection; product Build is 0.81.1.
The previously deferred product Platform/CRC selections are now included and
qualified. Six private qualification packages retain independent versions/lock;
four product packages remain under `crates/`. No pending source update is deferred
from this delivery. No sibling source or dependency selection was changed.

## Evidence and recovery custody

Keep `target/release-frozen/batch-sat4DSEt/` and durable
`.git/release-state/frozen/batch-sat4DSEt/`, including `repository.git/publication-state/`.
Gate roots beneath that checkout are `target/release-validation/protocol-gate.fhKDzR/`
and `target/release-validation/engine-gate.zXGgOr/`. Delivery log:
`target/auth-0380-release-corrected.log`. Initial `batch-4hebGSFP` failed before
publication on stale tooling pin exceptions; the corrected source was recaptured,
not substituted into its saved custody. Preserve that failed evidence too.

No batch or recovery evidence was retired. Keep older `batch-yiX0dEoV` custody and
`target/publication-recovery/`. An earlier external cleanup deleted old raw evidence;
historical measurements cannot replace those bytes. Do not resume earlier failed
batches against already delivered versions. The previous detailed handoff and
cleanup account are retained in Git at `2b8fa629a7c52d064dd9e617b8502efcb6036f2b`;
[history](history.md) retains older observations with their original scope.
