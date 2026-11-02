# Current handoff — 2026-11-02

## Accepted batch and delivery boundary

The maintainer approved all ten local complexity reductions as a hard cut, then
[Auth #124](https://github.com/dragginzgame/ic-auth/issues/124): Rust 1.99.1 as both
minimum and pinned development/release compiler. The maintainer subsequently
authorized committing/pushing the completed Auth changes, hosted CI qualification
and closing #124 after acceptance. Package versions remain 0.30.0; pending notes
live independently in both group ledgers at 0.31.0. No registry publication, tag
or sibling delivery is part of this Git delivery. #124 owns current hosted CI and
closure evidence; the local qualification below does not claim a hosted pass.

The original locks/patches remain under `target/complexity-hard-cut/` and
`target/shared-tooling-0.27.26/`. Git delivery excludes the pre-existing README
edits and cc 1.11.0/crc32fast 1.5.4 lock refresh, preserving them as working changes.
The selected delivery lock retains cc 1.10.0/crc32fast 1.5.3. Earlier local tests
used the incoming lock; hosted CI must qualify the selected delivery lock.
`target/issue124-delivery/` retains both locks and the exact restoration patch.
Shared Tooling is refreshed through the canonical exporter to 0.27.26, commit
`e84252f5740bacd708359580455ea2b6fd67df38` (142 payloads), independently verified
against that frozen committed source. Its compiler policy now matches #124.
Adoption evidence is under `target/shared-tooling-0.27.26/`. Common tools, locked
Testkit/server, file CLI and release editor are prepared and admitted offline.
Native build classification and managed tool retention are active; no legacy
reclamation or product/release cleanup ran.

## Completed source changes

1. Issuance carries the shared `Lifetime`/`Grants` causes directly; retire flattened
   variants and handwritten maps. Delegation keeps its existing refusal projection.
2. Fresh sessions construct from the already verified immutable admission snapshot.
   Exact retries, later reads, restoration and store commit checks stay independent.
3. Browser logout chooses wait/dispatch/completion and commits its lease in one
   strict transaction. Signing checks, failed-commit isolation and stored profile remain.
4. Upgrade decoding borrows the checked stable payload instead of copying it whole.
5. Private qualification controls use bounded typed refusals; native callers and
   codecs change together. The textual refusal adapter is retired.
6. Session/revocation tests share mechanical valid signing/context construction;
   adversarial mutations and independent crypto vectors retain their own owners.
7. Release fixtures retain both groups' actual delivery, partial preparation/resume,
   committed-disposition restart, custody and publication refusals; generic major
   arithmetic and repeated delivery permutations stay with Shared Tooling.
8. Digest-only roots use maintained Platform operations; witnesses still build trees.
9. Encoded conformance occupancy counts decoded replay records without a temporary Vec.
10. Native logout diagnostics reuse the existing Wire conversion.

All members now inherit Rust 1.99.1. Admission checks floor/pin agreement, actual
Cargo/rustc versions, active selection and installed Wasm target; CI seals that
selector between stages. One prepared compiler/cache replaces old duplicate setup
and internal checks. The [coverage map](../msrv.md) retains isolated public archive
features on native/Wasm, native and Wasm Clippy, proc macros and real runtime proof.
No hosted job/required-check identity is removed. Old caches/evidence remain intact.

## Qualification and measurement

Passing focused Linux evidence in `target/complexity-hard-cut/`:

- Core issuance/session/revocation/signature-store tests: `core-tests-retry.log`.
- Certified proof/admission refusals: `certified-admission-tests.log` (35 tests).
- Private contracts, checkpoint and encoded backend tests: `private-tests.log`.
- Browser lifecycle, durable leases and failed cleanup commit: `client-final.log`
  (70 tests); all five generated public browser files remain unchanged.
- Actual PocketIC lifecycle, composed certificates, upgrade, protected refresh,
  signed ingress and SDK transport: `live-qualification.log` (11 passed, one
  intentionally ignored profiling test).
- Rust 1.99.1 archive native/Wasm feature matrix and sealed inputs:
  `compiler-archives.log`; native/Wasm lint: `compiler-*-clippy.log`.
- Compiler refusals: `target/portable-fixtures/compiler-policy.OLNVLE`;
  setup ordering/input seals: `compiler-callers-seal.log`; Make: `compiler-jobserver.log`.
- Independent backend archive/registry identities and packaged scenarios:
  `compiler-backend-consumer.log`, fixture `backend-package.7SER8B/delivery.json`.
- Formatting, local documentation links and shared snapshot integrity pass.

Release/recovery fixtures pass, including the consolidated compiler setup:
`release-fixtures.log` and `release-compiler-fixtures.log`. Retain their isolated
source/effect evidence in `release-tools.64kMrZ` and `release-tools.tz1oCv`.
No complete new `make ci`, hosted macOS matrix or real registry upload ran here.

Shared 0.27.26 adoption passes independent source/snapshot verification,
`tools-evidence.log`, `testkit-callers.log`, `release-adapters.log`, the actual
CI cache-key command, formatting and local links. `offline-admission.log` records
Rust 1.99.1 and every selected executable. The failure-evidence fixture now copies
the installer's new native classifier, retention helper and owned-tree module.
These Shared checks preceded the dependency update below. Compiler and public
library source/graphs remain unchanged; affected private runtime proof is refreshed.

The requested published Memory/Storage **0.63.0** and Metrics **0.26.0** are now
selected by private qualification. Their published `src/` trees match the prior
versions byte-for-byte; the support floor is the already selected Rust 1.99.1.
The host Wasm has one Memory, Storage, Metrics and Wire provider. Native Testkit
0.78 independently retains Metrics 0.25.1; no Metrics values cross that boundary.
Combined graph preparation and focused native/Wasm Clippy pass. Checkpoint/storage
tests pass (10 host tests plus the fixture test), all five admission-metrics tests
pass, and live PocketIC lifecycle/upgrade/authority/SDK qualification passes
(11 tests; one intentionally ignored profiler). No storage migration or reset ran.
Evidence and pre-adoption inputs: `target/memory-metrics-storage-adoption/`.

Historical ten-change raw reference Wasm comparison, same Rust 1.99.1/lock/flags:
2,065,826 → 2,042,064 total bytes; 1,543,468 → 1,530,443 code-section bytes.
Savings: **23,762 total / 13,025 code bytes**. Published Host artifact inspection,
hashes and build logs are retained beside `wasm-facts.tsv`. This is the private
qualification canister before Binaryen, not production Canic size or latency.
`final-wasm-build.log` confirms the final support-floor edit produces the identical
measured Wasm bytes.
That comparison used the prior Memory/Storage 0.62.1 and Metrics 0.25.1 lock;
it is not a new size/performance measurement of the dependency adoption above.
No new instruction/performance claim follows from this size comparison.

## Retained safety, delivery and ownership

Signed bytes, public Wire schemas, crypto checks, authority rules and TEE/non-TEE
trust are unchanged. Preserve `AUTHNAT3`, session/login IDs
`0x415554485311`/`0x415554485313` and replay `0x415554485302`. No retained storage
format or old client profile is erased. Keep checked Memory handles, bounded
loading, occupied-backing refusals and host-owned clock, generation, atomicity,
allocation and certification-root publication.

Protocol 0.30.0 is published at `ec0233f3c4d746bb9bf7009f7432683a57c0b616`,
tag `protocol-v0.30.0`; Engine 0.30.0 at
`80459e300cdb6fb67a94ff45d33f0774ebf0bd9f`, tag `engine-v0.30.0`.
All three crates were checksum-verified and both release gates passed in
`/tmp/ic-auth-release.EqLgnm` (`validation.OQxMG0`, `validation.P7atRk`).
Keep that checkout, original failed `validation.YInWyN`, journals, receipts,
publication intents and archives. Both attempts completed; no resume is pending.
Future fresh releases select a group explicitly; recover historical attempts only
with their original adapter, saved source/group/version and custody locks.
Prior evidence remains under `target/persistence-wire-delivery/`,
`target/cohort-adoption/`, `target/memory-storage-adoption/`,
`target/release-groups-work/` and `target/types-boundary.BvpfC1/`.

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns adoption,
issuer/renewal, guards, durable backend and browser acceptance; #720 owns storage
deduplication. This batch establishes no installed Canic adoption, production
issuer/backend, npm publication or live-IC deployment. The allocation/Wasm
observations justify no verifier rewrite or verification-result cache.
