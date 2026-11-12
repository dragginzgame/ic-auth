# Current handoff — 2026-11-12

## Delivery

Protocol and Engine **0.39.1** remain published. All three registry checksums
match retained archives/schema-2 receipts; tags and release commits are pushed.
Batch `batch-BiswIN7d` passed both complete 30-stage gates and reconciled into
the normal checkout. Readback: `target/auth-release-0391/registry/verified-receipts.json`.

Protocol/Engine **0.39.2** publication is authorized for Auth only. Its source
adds optional indexed persistence and adopts Toko Tooling 0.39.17. The failed
and interrupted candidates below published nothing. A fresh captured candidate
must qualify the corrected source; never mutate captured inputs. Hosted CI is
asynchronous. Actual Chromium is unprepared/ignored. No Auth service, Canic
source change or sibling delivery is included.

## Indexed persistence — #132 remains open

The optional `persistent` capability owns receiver session/replay/fence records
and issuer intent/login/renewal/result bookkeeping. Hosts retain checked memory
leases, trusted caller/time/authority, atomic transactions and certification
composition. Caller sessions have a separate point table; maintenance uses
incremental quotas, indexes and bounded reference-counted expiry cleanup. No
routine operation scans or serializes unrelated login history. See
[persistent integration](../integration/persistent-auth.md).

The receiver schema is `0x41555448525302`; earlier local prototypes refuse.
Signed claims and existing public session/replay/login payloads are unchanged.
Canic's reinstall approval does not retire other consumers' records. Storage
0.73's backing/cell formats refuse older images without reset or a reader; no
production backing is implicitly retired. Library persistence selects Storage's
Wasm Runtime IC0, never an executor/guard/CDK; the host owns its one runtime.

The maintainer accepts roughly **1.223M** instructions for cold ordinary
receiver authentication at 100,000 records for now. The regression ceiling is
**1.3M**, including after same-release upgrade. Published Storage **0.73.4**
is sufficient; neither its locality experiment nor a sub-1M result blocks Auth.
This changes no verification checks or trust assumptions. No verification cache,
inter-canister call or cryptographic replacement was added.

The earlier matched accepted-cost run passed both installed 100,000-record cases:
receiver 1,223,407; after upgrade 1,222,307; issuer prepare 3,704,176, renewal
2,570,094 and maximum individual fill prepare 6,779,989 instructions. Evidence:
`target/auth-132-installed/accepted-cost/`. These timings predate the incoming
supplier patches below. The newest captured run passed issuer scaling and
receiver through 10,000 records before deliberate interruption for native test
preparation performance. Bookkeeping probes exclude initial proof verification,
Root approval, signing and endpoint overhead. Earlier warm samples in
`target/auth-132-installed/scaling.log` are not cold-call acceptance.

Full issuer key/enrollment/effect custody convergence and installed
certified/Ed25519 production-adapter lifecycle, corruption/interruption and
handoff acceptance remain in [Auth #132](https://github.com/dragginzgame/ic-auth/issues/132).
[Canic #764](https://github.com/dragginzgame/canic/issues/764) owns later adoption
and deletion. This release must not claim either is complete. Current Canic
adoption has not been inspected for this delivery; older observations are history.

## Tooling and suppliers

Both workspaces select published Toko Tooling **0.39.17**. Its authenticated
110-file snapshot includes scoped sequential `deps-update` forwarding and the
module-placement rule. Sixteen child-bearing module roots moved to `mod.rs`
without changing public paths; an empty integration-test directory was removed
while its standalone test remained. Snapshot, metadata and actual caller checks
passed. Evidence: `target/auth-release-0392/tooling/` and
`target/tooling-adoption-20261112/`.

Pre-freeze registry review confirmed Build 0.83.0, Entropy 0.1.9, Host FS/Process
0.76.5, Metrics 0.29.11, Platform 0.31.3, Runtime/macros/guard 0.21.3, Types
0.19.5, Storage 0.73.4, Testkit 0.101.0, Wire 0.21.4 and Tooling 0.39.17.
The incoming locks already contained the newer Platform/Host/Wire selections;
our Cargo updates changed only Tooling. Both locks were fetched/admitted.
Wire regenerates all five browser files: four Auth bindings unchanged; shared
runtime improves u32 lengths and packed booleans. The independent file CLI pin
stays 0.76.4; its optional update is deferred to Host/tool maintenance. Evidence:
`target/auth-release-0392/tooling/freshness.json` and retained indexes.

Testkit retains managed startup and its native PocketIC 16.1.0 re-export; one
Wire identity is required. Principal boundaries copy validated bytes. Awaited
instance deletion remains best-effort, with teardown logs retained. Protocol
and default/session graphs exclude native tooling and persistent Storage.

## Qualification and recovery custody

- `batch-XoEWGnOX` stopped at an obsolete all-feature package fixture refusal
  of optional Storage. Corrected fixtures retain passive/runtime-guard refusals;
  focused checks passed. Keep `target/auth-release-0392/delivery.log`.
- `batch-5x63tbsF` stopped at a partial Make fixture missing the new dependency
  include. Auth now declares the companion and uses Shared's closure copier.
  Evidence and command callers pass; keep `delivery-retry.log` in that directory.
- `batch-H6iT55MB` passed the first 27 stages plus installed issuer scaling.
  Receiver checks passed through 10,000 records; native debug generation of
  100,000 signed records was too slow. The owned test/server were stopped before
  publication. Keep `delivery-final.log` and all candidate/recovery state.
  Qualification now optimizes only curve arithmetic in the test profile,
  retaining debug/overflow checks, all records/assertions and the canister's
  release profile. The focused proof uses that candidate's exact Wasm;
  it passed in 202.31 seconds at 100,000 records: 1,222,906 instructions,
  then 1,222,307 after upgrade. See `target/auth-release-0392/optimized-seeding.log`.

Preserve every candidate under `target/release-frozen/` and durable
`.git/release-state/frozen/`, including publication-state. Preserve prior
`batch-BiswIN7d`, `batch-t5q4t2DE`, `batch-1JDi3jNF`, `batch-sat4DSEt`,
`batch-4hebGSFP`, `batch-yiX0dEoV` and `target/publication-recovery/`.
Do not resume old attempts against versions already published.

Earlier evidence inventories, device-contract decisions and supplier qualification
remain in Git at `0bcf7ac667f693e874a00c6550b3b965359d7ffc` and
[history](history.md). Keep their referenced `target/` evidence; a prior external
cleanup removed some older raw logs, which narrative measurements cannot replace.
The [session contract](../protocol/sessions.md) and
[Canic adoption guide](../integration/canic-adoption.md) own bounded device,
ninety-day renewal, retained fences and lost-state refusal behavior.
