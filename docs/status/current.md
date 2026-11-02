# Current handoff — 2026-11-02

## Current local upstream adoption

The all-upstream retry selects published Platform **0.18.0**, Types **0.17.0**,
Runtime **0.18.0**, Storage **0.65.0**, Wire/derive **0.15.0**, Metrics **0.27.0**
and Testkit **0.80.0**. Build **0.74.0**, Host FS/Process/Artifacts **0.74.1**,
canonical PocketIC client **0.18.0** and browser SDK **0.1.1** remain current.
The browser lock is unchanged. Storage now imports Runtime directly on Wasm;
Wire's schema bridge now implements the canonical Types 0.17 identity. The two
previous publication blockers are resolved without aliases or overrides. Public
and native graphs share one Platform/Types generation; the host retains one
Runtime/generator/guard. Native Testkit/Host retain arithmetic-only Metrics 0.26.1
internally, separate from Auth's 0.27.0 counters; no counter crosses that boundary.

Shared Tooling is canonically refreshed to **0.27.28** at
`8c7fae6605c69a930e3b1f14e1ff5c4c9dafbcb7`, independently verified against its
frozen committed source (145 payloads). This fixes the release-group guide's
companion closure; optional group plans remain unselected and Auth retains the
complete existing CI/release gate. No sibling working tree was changed. The
maintainer has authorized deployment: prepare Protocol 0.31.0 first, publish it,
then release/publish Engine 0.31.0 through a dedicated release checkout. The
source package versions remain 0.30.0 until the release runner prepares them.

Current retry evidence is under `target/upstream-retry/`. Locked metadata, native
and canister Wasm Clippy, 221 library/crypto checks, adversarial feature-unification
tests, seven backend conformance scenarios, 11 Storage tests and five metrics
tests pass. All five generated browser files remain unchanged; 70 browser checks,
archive builds for all three libraries, package/declaration/graph refusals and
11 live PocketIC tests pass (one profiler intentionally ignored). Live evidence
covers certified proofs, composed roots, protected-authority upgrades, signed
ingress, SDK admission and durable logout. The host again forbids all Platform
facade runtime-forwarding features; Storage selects the direct Runtime owner.
Testkit 0.80 setup/caller fixtures and offline file/editor/server admission pass.
The initial sandbox registry refusal and successful network retry are retained.
Formatting, local links and 145-payload snapshot/source verification pass.
Earlier partial-adoption proof remains under `target/all-upstream-adoption/`,
including the prior blockers and passing Build 0.74 release/recovery fixture.
That fixture's editor/runner code is unchanged by this retry; no full CI or
hosted matrix is claimed for the current dependency set.

The maintainer requested the consolidation from retired Memory into Storage.
The uncommitted Storage migration now selects published **ic-storage 0.65.0**,
archive source `84a7ecf6ed6517b989848c8e953d93fb5f561505`. It removes `ic-memory` and
`ic-stable-structures` from the workspace/lock and migrates the private checkpoint
host to Storage-owned backing, keys, namespace admission and checked leases.
The authorized release includes this migration; it performs no backing reset or
sibling change. Pending Protocol and Engine notes are independently at 0.31.0.

The release preserves the reviewed incoming README corrections and qualified
cc 1.11.0/crc32fast 1.5.4 lock selections.
Incoming Metrics 0.26.1 is deliberately advanced to 0.27.0 by this adoption.
Before inputs and qualified locks are retained under `target/upstream-retry/` and
`target/storage-owner-adoption/`; no unrelated dependency selection was refreshed.
The previous Git delivery intentionally excluded those README/cc/crc32fast edits.

The Storage 0.64 consolidation and 0.65 B-tree change alter backing formats. Use fresh
backing; previous manager/ledger images are refused without a reader or automatic
reset. No retained backing was cleared. A production installation requires a
separately reviewed whole-image transition. The fixture is unpublished and has
no production deployment here. Auth still owns its `AUTHNAT3` application payload,
128 KiB ceiling and Wire session/replay/login formats; those bytes are unchanged.

The host uses `StorageConfig`, `testing::fixture_runtime`, `StableKey`, checked
`RuntimeMemory` and `adapter::StableCell`. Loading passes the explicit byte ceiling
and verifies checksums; create/set errors are checked. Host message rollback owns
trap atomicity. Public Auth packages, passive contracts and backend conformance
remain independent of Storage/runtime. The direct dependency policy permits
Storage only in the private host and forbids retired Memory everywhere.
Storage's direct Wasm Runtime dependency resolves to the host's one maintained
Runtime, generator and singleton guard. Registry CDK/executor and Candid
remain excluded from Wasm; public graphs still forbid runtime capabilities.
The installed Shared snapshot's older Memory wording is reconciled by the
explicit consolidation note in AGENTS; snapshot-owned payloads were not edited.

## Earlier Storage 0.64 qualification

Evidence is under `target/storage-owner-adoption/`:

- `candidate-metadata.json`, `candidate-graph.log`, `metadata.json`: locked graph
  preparation, one Storage/Wire/Runtime owner, no retired storage crates.
- `native-clippy.log`, `wasm-clippy.log`: affected host/fixture/runner lint passes
  on actual Rust 1.99.1 native and Wasm.
- `storage-tests.log`: 11 tests pass, including bounded restore, occupied creation,
  detached custody, retired payloads, checksum corruption and read-only refusal.
- A frozen published Memory/Storage 0.63 manager header replaces the old 0.55
  ledger-vector setup; no legacy SDK capability remains in dev dependencies.
  Generation and prior source fixtures are retained locally. The vector is a
  header rejection test with a separate application sentinel, not a full image.
- `boundaries-retry.log`, `direct-boundaries-retry.log` and
  `package-boundary-tests.log`: public/capability/declaration isolation passes,
  including explicit refusal of Storage in public archive consumers.
- `wasm-build.log`: actual release Wasm rebuilt with Storage 0.64.
- The first live attempt refused a changed native tool fingerprint. Explicit
  common/Testkit/file/editor setup was rerun; original refusal/setup logs remain.
  `tool-admission.log` passes offline. `live-qualification-retry.log` passes all
  11 PocketIC lifecycle/upgrade/authority/ingress/SDK tests; one profiler stays
  intentionally ignored. Formatting, local links and snapshot integrity pass.

No current hosted CI, macOS or production-backend acceptance is claimed for this
new local batch. The live tests use fresh Storage backing and current-format
upgrades; they cannot establish an upgrade from Memory/Storage 0.63 images.

## Prior delivered work and acceptance

Commit **28ee29d12122cfbfe05b19a49460bd5ee79bf7c8** is pushed to main. It contains
the ten accepted complexity cuts, Rust 1.99.1 support/qualification consolidation,
Shared 0.27.26, Memory/Storage 0.63 and Metrics 0.26 adoption. All three jobs in
[CI 41434941116](https://github.com/dragginzgame/ic-auth/actions/runs/41434941116)
passed (Linux, Intel macOS, Apple Silicon macOS), including their full Validate
steps. [Auth #124](https://github.com/dragginzgame/ic-auth/issues/124) is closed
with that exact source/run evidence. This does not qualify the newer local
Storage 0.64 batch or publish pending package versions.

The prior delivery used Shared Tooling **0.27.26**,
`e84252f5740bacd708359580455ea2b6fd67df38` (142 payloads); the current refresh is
qualified above. Keep `target/shared-tooling-0.27.26/`,
`target/issue124-delivery/`, `target/memory-metrics-storage-adoption/` and
`target/complexity-hard-cut/` evidence. The historical matched raw qualification
Wasm comparison was 2,065,826 → 2,042,064 total and 1,543,468 → 1,530,443 code
bytes, using the earlier 0.62.1/0.25.1 lock. No new size or speed claim is made.

## Safety, recovery custody and consumer ownership

Preserve signed bytes, cryptographic verification, protected authority and the
TEE/non-TEE trust model. Keep session/login IDs `0x415554485311`/`0x415554485313`
and replay `0x415554485302`. Hosts own clock, allocation, protected generation,
atomicity and certification-root publication. Do not use a storage-format cut to
weaken restoration, erase unknown issuance or clear only allocation metadata.

Protocol 0.30.0 is published at `ec0233f3c4d746bb9bf7009f7432683a57c0b616`,
tag `protocol-v0.30.0`; Engine 0.30.0 at
`80459e300cdb6fb67a94ff45d33f0774ebf0bd9f`, tag `engine-v0.30.0`.
Keep `/tmp/ic-auth-release.EqLgnm`, its original `validation.OQxMG0`,
`validation.P7atRk`, failed `validation.YInWyN`, journals, receipts, intents and
archives. Both attempts completed; no resume is pending. Historical recovery
must use its original adapter and saved source/group/version, never the new
Storage formats or a bypassed custody lock. Older evidence remains under
`target/persistence-wire-delivery/`, `target/cohort-adoption/`,
`target/memory-storage-adoption/`, `target/release-groups-work/` and
`target/types-boundary.BvpfC1/`.

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns installed
adoption, issuer/renewal, guards, durable backend and browser acceptance; #720 owns
storage deduplication. No Canic source, production issuer/backend, npm package or
live-IC deployment is changed here. No verifier rewrite or result cache is implied.
