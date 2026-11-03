# Current handoff — 2026-11-03

## Platform 0.21 / Engine 0.33.1 delivery in progress

The maintainer authorized Platform adoption and delivery in the normal Auth
checkout. Published Platform 0.21 removes its retired PocketIC client feature;
its Types, Runtime, Agent and crypto implementations are unchanged. Auth selects
0.21 and already uses Testkit 0.84's owned client. Testkit's native-only
Candid/management DTOs retain the 0.20 facade; Types 0.18 and Runtime 0.19
remain canonical. No facade alias or verifier relaxation is introduced.

The selected release is Engine 0.33.1, with Protocol left at 0.33.0. The pending
Shared 0.29.2, Testkit and conformance/browser work is included. Delivery uses
this checkout and `origin/main`, an annotated `engine-v0.33.1` tag and crates.io
publication. Historical recovery state and the completed 0.33 release checkout
remain intact. Earlier candidate receipts below keep their original meaning;
they do not qualify the newly selected dependency until checked afresh.

## Maximum-shape and packed-browser qualification

Auth's backend conformance entrypoint now includes `assert_maximum_shape` as its
sixth scenario. It admits all 16 role grants with 32 scopes per grant and
64-byte labels, requests 16 scopes from the final role, reopens session/login
backing and authorizes every requested scope with unchanged records. Real
Platform signatures and the public engine perform admission. Both the reference
callbacks and independent encoded-record adapter pass; the native suite has
nine passing cases. Focused backend Clippy also passes. This tests shapes and
restoration, not actual Canic host performance.

`make test-client-package-browser` now extends the existing packed consumer with
an explicit prepared Chromium executable and a bounded, isolated-profile run.
The bundle imports only installed archive/public exports. The shared consumer
smoke now closes/reopens the logout journal before cleanup, in both fake and
real IndexedDB. The actual check passed with Chromium 156.0.8078.4: one issuance,
one admission, local logout invalidation and two revocation acknowledgements.
Issuer/receiver calls remain injected. Browser version and executable/bundle
hashes are retained; no browser was downloaded and no user profile was used.

Updated candidates are assembled at
`target/adoption-boundaries/delivery/manifest.json` with verified checksums.
Backend archive evidence is `target/portable-fixtures/backend-package.qnK9he/`:
registry identity/consumer admission, graph refusals and all nine packaged cases
pass. Browser archive evidence is `target/browser-client-packages/package.qndU6C/`,
with `packed-consumer-chromium` receipt and `browser-proof.json`. Missing/relative
browser selection refuses before setup. Shell lint, JavaScript syntax and Rust
format checks pass. All changes remain local dirty candidates; library/runtime
and generated-contract sources still match the published 0.33 tags. No crypto,
verification behavior, TEE policy, consumer deployment or publication changed.
Evidence summaries are under `target/adoption-boundaries/`.

## Earlier matched Canic adoption candidates

The matching backend and private browser bundles are assembled at
`target/canic-adoption-033/delivery/manifest.json`, with archive/receipt copies,
input records and a verified `SHA256SUMS`. Both receipts identify local dirty
source `a35db7b6c5f462bf8bc81441f665a23dac00a97d`; these are development
candidates, not newly delivered releases. Library and generated-contract source
still match the published Protocol/Engine 0.33.0 tags. Rebuild and requalify from
the intended delivered commit before production adoption.

`make test-backend-package` passed the normalized archive's registry consumer,
dependency-graph refusals and eight packaged scenario tests, including detection
of admission/logout mutations before an error. Exact evidence is
`target/portable-fixtures/backend-package.H4R974/`. After simplifying the browser
README to use one build-and-qualify command, `make test-client-package` passed
strict declarations, injected-transport/IndexedDB smoke, private-import refusal
and browser bundling of the actual archive. Final browser evidence is
`target/browser-client-packages/package.wumMjR/`; the earlier archive remains
separate. The source/dependency cohort seal is unchanged. Setup-only release
metadata attempts remain recorded separately from the passing admitted group
callers and failure-status test below.

The [adoption guide](../canic-adoption.md#matched-qualification-handoff) now gives
exact archive installation and integrity instructions. Run the backend contract
against Canic's actual adapter and qualify its real production browser transports
there; Auth's reference/test adapters do not establish that acceptance. Read-only
Canic review confirms Auth 0.33 and Timers 0.39 selections. The former runtime
blocker is resolved, with incoming-family qualification still in progress and
production clock/restore, certified issuance, maximum-shape cost and browser
delivery remaining Canic-owned. No sibling changes or new publication occurred.

## Testkit 0.84 adoption

The current local batch selects published Testkit 0.84.0. The native runner now
imports clients/DTOs from `ic_testkit::pocket_ic`; the separate
`ic-platform-pocket-ic` declaration and locked package are removed, and the local
direct-dependency policy forbids its return. The incoming Build 0.74.2 lock
selection was preserved. PocketIC server 16.1.0, Platform 0.20, Types 0.18,
Runtime 0.19 and Wire/derive 0.16.1 remain selected. Public package versions remain
0.33.0, with Engine 0.33.1 notes pending. No new commit, PR or release was made.

Focused checks pass under `target/testkit-084/`: native runner Clippy, actual
direct-dependency admission, Wasm library/host isolation and singleton guards,
fixture Wasm build, four file-tooling tests, and explicit Testkit CLI/server
setup plus offline admission. Three exact real cases pass independently:
`certified-upgrade.log`, `signed-ingress.log` and `sdk-sessions.log`. They cover
certification/upgrade, ingress after upgrade and SDK admission/logout across two
receivers/devices, including lost replies, reopened journal and retained-key
refusal. `selection-check.log` confirms unchanged manifest/lock/runner/policy
bytes throughout. Full CI, hosted acceptance and production Canic integration
are not inferred from these selected checks.

Auth's direct-boundary checker admits this consumer because the native external
dependencies belong to registry Testkit, not an Auth workspace member. Shared
0.29.2 also records Testkit's ownership; its resolved refresh is described below.
No supplier-owner exception was patched locally or weakened.

## Normal-checkout workflow correction

The maintainer clarified that routine coding and adoption happen directly in the
normal checkout; PRs are used when requested or required. No additional clone or
automatic PR is needed. Existing work is preserved on
`chore/shared-tooling-029-branch-workflow` in `/home/adam/projects/ic-auth`.
The Auth overlay and development guide no longer require a separate release
clone. Coordinate shared-checkout sessions before branch switches and keep
release inputs stable. The existing qualified direct release commands retain
their separate authority; no unqualified PR release adapter is advertised.
The completed 0.33 release checkout and its original evidence remain intact.
[Auth PR #126](https://github.com/dragginzgame/ic-auth/pull/126) was closed at the
maintainer's request; its branch/commit remain, and this clarification is local.

The former exporter refusal is resolved by Shared 0.29.2 at
`04a87b366eae3e63b9f032c0b891c17bdc59e69b`. Auth now adopts that exact commit
using `--revision` against the existing local supplier repository; no source
checkout, source branch switch or new clone is needed. All 149 payloads match
the independently selected commit. Both actual group-order callers and
direct-dependency admission pass, and a substituted sort failure preserves
status 37 through Auth's caller. Evidence is under `target/canic-adoption-033/`.
[Shared #353](https://github.com/dragginzgame/shared-tooling/issues/353) is closed;
its earlier refusal remains at `target/shared-tooling-029/impact.log`.

## Protocol/Engine 0.33 delivered

Protocol 0.33.0 (`ic-auth-protocol-types`, `ic-auth-wire`) and Engine 0.33.0
(`ic-auth`) are published on crates.io. Source candidate `27373a9` was delivered
Protocol first at `b5d8756596976d05060e6ec35e3dd87b88273f01`, then Engine at
`fa6823a25cedb04f55f91301efcf261ab4109b1f`. Both annotated tags
`protocol-v0.33.0` / `engine-v0.33.0` and `main` were pushed to
`dragginzgame/ic-auth`. Registry observations match all three archive checksums;
both publication adapters completed with no unresolved upload.

Each group passed its fresh complete 30-stage local release plan, including
native/Wasm Clippy, archive/MSRV consumers, browser tests, cryptographic/session
checks, live PocketIC and the full release/recovery/publication fixture.
The approved dedicated checkout `/home/adam/projects/ic-auth-release-033`
preserves `.git/release-state`, `.git/publication-state` and `target/delivery/`
logs. Protocol proof is `protocol-gate.agsMcV/logs/validation.5ADxWq`; Engine
proof is `engine-gate.A89Q1Y/logs/validation.BPefXH`, both under release state.
The normal checkout was fast-forwarded to the Engine release before this handoff
update. No application or Canic deployment was performed.

The explicit [native group-comparison run](https://github.com/dragginzgame/ic-auth/actions/runs/41680139546)
uses the exact Engine release commit with `qualify_release_groups=true` and retains
Ubuntu, macOS Intel and Apple Silicon. It was in progress at handoff; hosted
acceptance and #125 closure are not claimed from local release success.

## Auth validation ownership — [#125](https://github.com/dragginzgame/ic-auth/issues/125)

Auth now delegates canonical run-evidence admission, roster validation and
baseline observation to Shared Tooling 0.28.3. The local generic evidence parser
is deleted. `validation-plan.pl` retains product exclusions and complete source
custody; `validation-record.pl` binds Shared's opaque proof to Auth's receipt.
Schema-4 receipts preserve original-source/notes admission and retained proof
bytes during interrupted preparation. Exact committed publication admits that
self-contained record in a fresh clone. Historical schema-2/3 attempts require
their original adapters; the earlier frozen qualification below keeps its own
meaning and is not reinterpreted.

CI discovers manifests from Shared's admitted Cargo group plans, retaining the
before/after discovery seal and every returned member even with unequal group
versions. No new Cargo inventory or event/timing parser is maintained here.
Ordinary CI remains the full 30-stage gate. Both mappings run additionally only
under explicit `qualify_release_groups` workflow dispatch. No whole-gate reuse,
version projection or product/crypto/runtime change is introduced.

Focused consumer qualification is retained under `target/issue-125/`.
Group dispatch/Shared evidence (`group-rerun.log`, `release-validation.ZuGdwf`),
the separate actual-CI roster refusal (`roster-caller-final.log`), unequal-version
manifest discovery (`discovery-final.log`, `auth-validation.p8w3v8`) and Testkit
caller checks (`testkit-caller.log`, `testkit-tools.shK9Is`) pass. The final focused
release fixture passes (`recovery-focused.log`, `release-tools.S8whci`): actual
metadata, changed-retained-log refusal, exact resume without gate replay,
fresh-clone publication, both groups and unselected identity/ledger preservation.
It uses local Git/bare remotes and mocked stage/upload effects; stopped and failed
attempts are retained separately, not counted as passes. ShellCheck, Perl syntax,
documentation links and diff whitespace checks pass. The complete CI target
retains its full setup/registry/recovery matrix. Full CI, Rust/MSRV,
new hosted/macOS mapping measurements and publication are outside this batch.
These focused checks completed in the requested checkout before source preparation.

## Upstream update and Canic review before delivery

The current maintainer request adopts published upstream dependencies and reviews
Canic adoption. Existing uncommitted group-CI and conformance work is preserved.
The selected family is Platform/PocketIC **0.20.0**, Types **0.18.0**, independent
Runtime/macros/guard **0.19.0**, Testkit **0.83.0**, Metrics **0.28.2**, Storage
**0.66.1**, Wire/derive **0.16.1**, Build **0.74.1**, Host libraries and the
independent file CLI **0.74.2**. Browser SDK **0.1.1** is still latest. All current
published selections were checked against the supplier registries. Shared Tooling
**0.28.3** is adopted at merged `07f41c2723850f8fd86c61f2f50e65b0c6cde886`;
all 149 payloads match that independently selected source. Its VERSION is stable;
this does not assert a matching release tag.

This upstream selection supplies the Shared helpers used by the #125 consumer
migration above. Routine CI still runs one full gate and group comparison stays opt-in.

Native runner/fixture and Wasm canister Clippy, five Metrics cases, one fixture
unit case, four real file-CLI caller cases, selected normal/Wasm graphs,
manifest-input caller tests, dependency pins and actual group-dispatch/refusal
tests pass. Testkit/server, Host file and release-editor
setup is explicit. Three selected real pairing cases pass: certified proof with
upgrade, signed ingress after upgrade, and SDK receiver/device admission plus logout. Each exact
filter runs one named test; the SDK case exercises lost-reply/reopened-journal
recovery without reissuance, two receivers/devices, independent durable logout
and retained-key refusal. Their logs are `certified-upgrade.log`,
`signed-ingress.log` and `sdk-sessions.log` in the evidence directory. Full CI,
archive/MSRV matrices and hosted/native macOS acceptance are not inherited from the previous cohort. Evidence is under
`target/upstream-20261103/`. Delivery explicitly regenerated the locked Wire
0.16.1 browser output: all five files were unchanged, and all 70 client tests
passed (`target/delivery-033/wire-generation.log` and
`target/delivery-033/wire-client-qualified.log`). Native entrypoints were compiled without
advertising production wallet or Canic installation acceptance.

Canic source review is read-only at HEAD `0509ec870891515923f79d3932ece127b351cd15`
plus its owned dirty integration. Controlled-local receiver/guards and issuer-key
registration now exist. Both #720 helpers are already in its uncommitted adapter;
qualification/closure remains with Canic. #491 retains production clock bounds,
snapshot fencing, issuance/reconciliation, browser delivery and installed proof.
The published Timers 0.38.1 still selects Runtime 0.18.1 and blocks a complete
Runtime 0.19 singleton family. Host Tools 0.80 is now published with Types 0.18;
Query 0.89 uses that Types identity too. Auth's matching 0.33 Protocol/Engine
cohort was still unpublished at that review and is now delivered above. No new missing Auth API was found. See
`docs/canic-adoption.md` for exact ownership and evidence limits. No sibling edits,
issue mutations, real commits, pushes, releases or deployments were performed.

## Previous group-CI qualification (earlier dependency cohort)

The maintainer requested Shared's group CI rules. Auth now owns one ordered
30-stage roster in `ci/release-validation.json`; ordinary CI always runs it all.
Source-only Protocol plans retain 29 stages, Engine 27. Coverage reasons are in
`ci/release-validation-coverage.json`; shared/unknown changes select all 30.
Both retain complete Clippy/MSRV/graph/browser/crypto/session/canister/archive
obligations. Scripts, locks, manifests, generators and cross-owner changes are full.
The adapter uses Shared's plan runner and Auth's complete source inventory checks
before/after every stage. No stage reuse or Cargo version projection is enabled.

That frozen source's schema-3 receipts carry plans, original context and stage log hashes. Unfinished
releases re-admit retained proof bytes; exact committed releases can publish from
fresh clones using their self-contained record. Schema-2 recovery uses its original
checkout. See `docs/auth-release-validation.md`. Explicit native workflow dispatch
with `qualify_release_groups=true` compares both mappings with its same-source
full gate and retains successful proof artifacts; ordinary CI runs one full gate.
Linux full/group qualification passed. No macOS result is claimed for that frozen
source; the current native comparison is dispatched separately above.
The full gate seals its starting clean-source attestation; comparisons reject
evidence from a different HEAD or dirty source.
Dispatch tests with substituted stage bodies cover both groups, full expansion,
required/excluded failures, drift, stale plans, corruption and ordinary CI routing.
Release/resume/publication fixture qualification passes, including fresh-clone
publication (`release-tools.00q6Jn` in the full gate and focused
`release-tools.Q9viuF`, local Git/mocked upload effects). Keep failed
and interrupted attempts in target. Native tooling tests consume Wire contracts
and are retained in both groups; only unchanged release fixtures are omitted in
Protocol, with passive Protocol unit tests additionally omitted in Engine.

Linux evidence uses frozen source `e2bb7024cab4e6bf40e691925ea9ae241b0aca3e`
in `target/group-validation-adoption/source.ZklNDJ/`. Full gate
`target/ci-validation-sequential/validation.OjiP20/` passed all 30 stages.
`target/release-validation/qualification.O0jl30/comparison.json` binds that full
proof to Protocol's 29 passes/1 exclusion and Engine's 27 passes/3 exclusions;
both raw evidence sets were checked again after collection. Summed observed stage
times: full **1245 s**, Protocol **382 s**, Engine **388 s** (same source/Linux host).
These are qualification observations, not production/Wasm measurements.
The standalone helper's Make target environment fix followed that frozen cohort;
`target/release-group-make-context.log` passes the focused regression with the
actual Make export and an initially unset target. The unchanged cohort's real
comparison used the equivalent explicit target. The `.git` evidence-path/Testkit
caller probe passed (`path-probe.9eDS4x`).

Before delivery, pending notes selected **Protocol 0.33.0** and **Engine 0.33.0**
for public dependency identity alignment and the new release receipt contract;
package versions then remained 0.32.0. Those observations preceded delivery above.

## Previous dependency qualification

The previous qualification used published Platform/PocketIC/
Runtime/macros/guard **0.19.0**, Types **0.18.0**, Testkit **0.82.0**, Wire/derive
**0.16.0** and Storage **0.66.1** (incoming patch after the 0.66.0 cohort). This resolves the prior duplicate Runtime,
Principal schema and Testkit Limits blockers. Platform 0.20/PocketIC 0.20 are
separate optional adapter cuts; this batch retains Testkit 0.82's matching 0.19
family. Metrics 0.27.2, Build 0.74.1, Host libraries 0.74.2 and file CLI 0.74.1
remain selected. Rust 1.99.1, PocketIC server 16.1.0 and browser SDK 0.1.1 stay.

Native fixtures explicitly handle PocketIC's fallible calls without adding retries.
Native runner Clippy passes. All five browser files were regenerated and reviewed
unchanged; all 70 browser tests, Rust/TS interoperability and packed consumer checks
pass. Testkit 0.82 and the release editor are explicitly prepared. New-graph
native/Wasm Clippy, dependency boundaries and isolated MSRV checks pass. Actual
canister qualification passes (11 tests, 1 separate Chromium case ignored).
PocketIC transport scratch uses a short recorded `/tmp` path to avoid Unix socket
limits; canister state/logs remain under target. The caller fixture verifies
cleanup on success and retention on failure. All three local package archives
also build together on the new identity cohort.
The late incoming lock changed only Storage 0.66.0 to 0.66.1. Both registry
archives match their locked checksums; every library source file is identical,
and the normalized manifest changes only its version. Changes are confined to
release examples/tooling outside Auth's selected graph. Focused Wasm Clippy for
the qualification canister passes on 0.66.1. Evidence is
`target/storage-0661-source-review.json` and `target/storage-0661-clippy.log`.
Keep the earlier full/group measurements explicitly on 0.66.0.

Retain failed/interrupted attempts. `validation.WlxbFr` failed its last fixture
on disk exhaustion (`release-tools.pFmFMz`); `validation.Mfsk9G` lost active `/tmp`
scratch (`release-tools.p5e3yk`). The maintainer confirmed cleanup paused.
`validation.s1JBEn` lost Cargo's unpacked temporary-registry source during archive
verification; the isolated retry passed (`target/package-builds.retry.log`). The
successful final comparison ran sequentially with normal scratch under the
checkout's `target/portable-fixtures/`; only socket scratch uses `/tmp`.
The first group attempt (`qualification.JCa5Xi`) correctly refused the changed
Make target environment. Keep it alongside the successful attempt; never turn
failed full attempts into passing evidence. Older 0.30–0.32 custody is corrected
below after cleanup removed those locations.
Evidence: `target/group-validation-adoption/` and `target/release-group-*.log`.
Earlier failed registry trials remain in `target/platform-019-adoption/`,
`target/platform-latest-adoption/`, `target/upstream-033-adoption/` and
`target/platform-wire-adoption/`. The former two-Runtime prerequisite failure is
retained at `target/release-group-runtime-prerequisite.log`.

That earlier cohort used Shared Tooling **0.28.0**, commit
`70939d3b6ca335cb718363b47f15cc616c3bed1c`, with 145 source-verified payloads.
No private-package restructuring is activated. Auth's delivery authority rules
remain binding; adopting a plan does not authorize remote or release effects.

## Canic support and archive boundary

The shared backend contract adds scoped callbacks, unadmitted caller/changed issuer
refusals and before/exact/after expiry, before and after reopen. Eight native tests
and focused Clippy passed on the earlier 0.32 graph. The candidate archives and
receipts in `target/canic-adoption-032/delivery/manifest.json` also passed their
registry/backend and browser-consumer checks on that earlier cohort. They retain
base `bbcd4cc` plus dirty inputs, Platform 0.18 and Types 0.17. They are not delivery
proof for current source. New backend archives require published matching 0.33
libraries; do not mix registry 0.32 with current Types 0.18 source.

Original archive evidence is `target/portable-fixtures/backend-package.9VuiS0/`
and `target/browser-client-packages/package.YRrwfx/`; the earlier backend
`backend-package.4hpO9J` predates a README correction. Reference backends and fake
IndexedDB/injected transports do not qualify Canic production storage, installed
endpoints or Chromium. Canic #491 owns production issuer/renewal, guards, durable
backend and browser acceptance; #720 owns storage deduplication.

## Delivered releases and recovery custody

Published/checksum-verified Protocol **0.32.0** is commit
`d31dc297e5026c2e6d2b1b97af99a761b50ae686`, tag `protocol-v0.32.0`; Engine
**0.32.0** is `bbcd4cc694e91f3b133d9c5bf0dc25c012b6d481`, tag `engine-v0.32.0`.
Both tags/main were pushed; no service/npm/sibling/reset effects occurred.
The former `/tmp/ic-auth-release.qoEtNy` checkout is now absent after cleanup,
as is `target/complexity-032-delivery/`. Its local raw proof/archive custody must
not be claimed available. No uncertain upload or release/helper lock remained
at delivery. Development is based on `bbcd4cc` and
preserves its incoming dependency work outside that frozen checkout.

Original complete gates are `validation.xgxTtJ` (Protocol source `119ea015`) and
`validation.6Sm2XS` (Engine source `e93794c`) under the release clone's
`.git/release-state/validation-logs/`. Publication evidence there is
`target/publication-protocol-0.32.0.ZMVFZG/` and
`target/publication-engine-0.32.0.S8p3im/`. Development copies receipts/checksums,
remote refs, interrupted/failed attempts and source/lock preservation facts under
the now-missing `target/complexity-032-delivery/`. These paths describe original
delivery evidence, not currently available copies. The Engine source includes the controlled-clock
repair for the old hosted browser timeout race; no fresh native-host CI status is
inferred for this new batch. Audit items 1–9 shipped in 0.32; retain their ledgers.

Earlier 0.31 identities are Protocol `ae8ae1dc`, Engine `38ba68f6` and their tags;
the former `/tmp/ic-auth-release.x4YbcZ` and 0.30 `/tmp/ic-auth-release.EqLgnm`
checkouts are also now absent. Earlier 0.30 identities/evidence remain described in the handoff
at `d08ae8916dca83b6583e1550173328b877ddd41f` and `/tmp/ic-auth-release.EqLgnm`.
Historical recovery requires its original source, adapter and formats; do not
manufacture missing raw validation evidence from release tags or passing summaries.

## Storage and trust

Keep `AUTHNAT3`, session/login IDs `0x415554485311` / `0x415554485313`, replay
`0x415554485302`, valid signed/Wire/storage bytes and every crypto/delegation/
certificate check. Auth has no Account DTO and no verification-result cache.
The TEE/non-TEE trust model is unchanged. No production wallet service/backend
is deployed. The maintainer confirmed no retained roles above 64 bytes.
Storage's consolidated backing still requires fresh state; old manager/ledger
images refuse without fallback/reset. No retained backing was cleared. Production
needs a reviewed whole-image transition. The host owns clock, checked allocation,
atomicity, protected generation and certification-root publication. No matched
Wasm or latency improvement is claimed from this batch.
