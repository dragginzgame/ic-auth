# Current handoff — 2026-11-12

## Delivery

Protocol and Engine **0.39.2** are published. All three registry checksums
match retained archives/schema-2 receipts; tags and release commits are pushed.
Batch `batch-6Eztyvd6` passed both complete 30-stage gates and reconciled into
the normal checkout. Readback: `target/auth-release-0392/registry/verified-receipts.json`.
Protocol commit: `c083d8b6bfb11c250494317de38d2cb2351e5171`;
Engine commit: `ba07b723e50284645e058257b7aa9e2c3527eb76`.
Source: `905b2995ca54735efa28f6f533ce05cd58e41cfc`.

This Auth-only delivery adds optional indexed persistence and Toko Tooling
0.39.17. Hosted CI remains asynchronous. Both installed suites passed 13 cases;
actual Chromium is unprepared/ignored. No Auth service, Canic source change,
production-storage mutation or sibling deployment occurred. Canic remains blocked
on the APIs and acceptance below; this is a foundation release.

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

Both delivered gates passed installed scaling at 100,000 records. Protocol's
receiver measured 1,223,207 instructions; Engine's measured 1,223,407. Both
measured 1,222,307 after upgrade. Issuer prepare was 3,704,202, renewal 2,570,120
and maximum individual fill prepare 6,780,015. Evidence:
`target/auth-release-0392/delivery-optimized.log`, with gate roots under the
captured checkout's `target/release-validation/`. Bookkeeping probes exclude
initial proof verification, Root approval, signing and endpoint overhead.
Keep the earlier `target/auth-132-installed/accepted-cost/` evidence. Warm samples in
`target/auth-132-installed/scaling.log` are not cold-call acceptance.

### Local #132 completion, not yet released

`IssuerStore::retrieve_renewal` now provides browser-authenticated lookup-only
recovery, including expired/interrupted originals. `preflight_renewal` projects
the authenticated account, exact request and original policy before application
authorization. Both share the private renewal index/binding checks with `renew`;
neither reserves work or advances time. `ReceiverStore::inspect_sessions` returns
passive expired/stale metadata in caller order, with an exclusive cursor and a
bounded scan across fences. Empty pages can have continuations. See the
[consumer contract](../integration/persistent-auth.md). Stored formats, signed
bytes and verification/trust rules are unchanged. Canic retains host keys,
Root enrollment/approval and application policy; no additional Auth key store
or consumer renewal index/session mirror is required.

Focused evidence: `target/auth-132-apis/`. Eighteen native tests pass, including
100,000 logins, corrupt index/record refusal and zero-write read APIs. Seven
installed session cases pass; Chromium remains unprepared/ignored. Both real
certified and Ed25519 durable lifecycles cover device capacity/overflow, retry,
renewal, upgrade, unexpired late-callback refusal, logout, authority invalidation
and bounded cleanup. The fixture's existing certification owner commits certified
completion/leaf publication in one transaction. Strict native/Wasm Clippy,
dependency boundaries, snapshot/membership, formatting and doc links pass.

Installed scaling also passes through 100,000 records: cold authentication
**1,224,068**, post-upgrade **1,223,195**, renewal preflight **508,966**, recovery
**645,001**, and two-row inspection **2,106,025** instructions. Keep
`installed-scaling.log`, `installed-sessions-final.log`, native logs and
`inputs.sha256`. `installed-issuer-final.log` also proves refusal of the 100,001st
login with unchanged occupancy. These are focused checks, not a release gate.

[Auth #132](https://github.com/dragginzgame/ic-auth/issues/132) stays open across
delivery and consumer acceptance. [Canic #764](https://github.com/dragginzgame/canic/issues/764)
owns adoption, duplicate-engine deletion and actual endpoint qualification.
No Canic source, production backing or deployment changed. Engine 0.39.3 notes
are pending; product versions remain 0.39.2. The new APIs are not published yet.

## Tooling and suppliers

Both workspaces select Toko Tooling **0.39.17**, with a verified 110-file snapshot,
scoped `deps-update` forwarding and child-bearing `mod.rs` placement. Adoption
checks passed; retain `target/auth-release-0392/tooling/` and
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

External updates selected Storage **0.73.5**, qualification Metrics **0.29.12**
and Entropy **0.1.10**; the #132 focused checks use these inputs. The authorized
Engine 0.39.3 release additionally captures incoming Wire **0.21.5** in both
locks, with all five browser artifacts regenerated. Its new complete gate is
pending. Published 0.39.2 used Storage **0.73.4** and Wire **0.21.4**.

Testkit retains managed startup and its native PocketIC 16.1.0 re-export; one
Wire identity is required. Principal boundaries copy validated bytes. Awaited
instance deletion remains best-effort, with teardown logs retained. Protocol
and default/session graphs exclude native tooling and persistent Storage.

## Qualification and recovery custody

Successful delivery log: `target/auth-release-0392/delivery-optimized.log`;
reconciliation: `target/auth-release-0392/reconcile.log`. Keep the completed
`batch-6Eztyvd6` checkout and durable state, including publication receipts.

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
