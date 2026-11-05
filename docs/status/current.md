# Current handoff — 2026-11-05

## Deployment review: Platform 0.22.1 and latest committed LOC fix

Shared Tooling revision `b37a882314d22bf7ab393b3fe561809377ffd22d` is adopted
through the exporter, including the duplicate TOTAL-row fix and portable exported
documentation links. Newer uncommitted Shared work is excluded. Auth's incoming
lock selects Platform 0.22.1, Wire/derive 0.17.1 and Metrics 0.29.2; those selections
are preserved. The Platform minimum now requires its 0.22.1 compatibility fix.
Wire generation changes only the browser runtime's common error translation;
the four application contracts remain byte-identical.

Platform's published correction restores packed-map/sequence envelope admission
and removes redundant BLS key serialization/parsing without dropping checks.
Its matched Canic report finds approximately 29 KB residual overhead relative to
0.21 after the correction; it does not establish size parity or erase Auth's
earlier fixture growth. On 2026-11-05 the maintainer explicitly accepted the
residual increase and authorized deployment, lifting the earlier publication
hold. Release Protocol 0.34.0 first, then Engine 0.34.0 through the standard
gates and publication adapters. Evidence: `target/auth034-delivery/`.
All 204 native Engine tests/doctests, 70 browser tests and Rust/TypeScript
interoperability pass, as do the release Wasm build and all 11 ordinary live
qualification cases. The opt-in Chromium case is not run. These focused checks
do not replace the required release gate after the size decision.

## Shared Tooling 0.30 rules audit

The subsequent LOC display fix is adopted at committed revision
`08cd17240fd60dfd309e35eb131e73323e883e6f`: single-workspace tables display
`ic-auth` instead of the full manifest path; JSON retains its absolute manifest.
Evidence: `target/cloc-fix-adoption/`. Classification and counts are unchanged.

Adopted committed revision `07dce12cbe4a798fd4a6d263095518678c4f6b45` through
the canonical exporter from the existing Shared checkout. Its VERSION is 0.30.0;
the source ledger still labels it unreleased, so this is committed-snapshot
adoption, not a claim of upstream publication. Added the linked LOC/stack reference
documents through the exporter as well; no snapshot-owned payload was edited.

| Rule area | Auth review and action |
| --- | --- |
| Rust layout and LOC | All six private qualification packages declare `tests`; tooling declares `build_tooling`. Allocation observations moved from `examples/` to harness-free `benches/`, with updated scripts and Cargo targets. Rust test paths already use `tests/` or `tests.rs`; the TypeScript `test/` directory is outside this Rust rule. |
| Lint expectations | No Rust `allow(...)`, Cargo allow-level settings or command-line Rust allow flags found in maintained callers. The two existing `expect` attributes remain narrowly scoped to Wire control names and corruption-fixture discriminants. Unfulfilled expectations remain enabled. |
| Module/source ownership | No `#[path]`, Rust `include!` wiring, embedded executable Rust patches or std-only `core::` imports found. Byte/hex includes are data; generated MSRV/archive consumers are compiler qualification inputs. |
| Dependency/API boundaries | Root-owned dependencies, virtual workspace, independent release groups and explicit capabilities retained; direct-dependency ownership check passes. Shared metadata changes do not alter public Auth APIs, crypto, records or TEE trust. |
| Validation dispatch | Fixed an incorrect Engine exclusion: `test-protocol` actually runs Engine default tests. It now remains required, with refusal coverage in the actual selector/runner fixture. Full CI and opt-in cross-group qualification remain distinct. |
| Process and tooling owners | Production tooling uses Host FS/Build; live Node capture uses Host Process with output bounds and a timeout. Existing controller/runtime/storage ownership stays unchanged. No local replacement for Shared's classifier or release evidence machinery was added. |
| Retention and delivery | Read-only retention admission passes for Git-side release/publication custody. No evidence or checkout was deleted. Final-candidate stage reuse remains unqualified here: preserve ordinary execution rather than introduce a skip/cache path. |
| Formatting and documentation | Canonical hook/formatter wiring retained; manifest roles sorted with cargo-sort. New documentation dependencies are snapshot-owned. Historical evidence and pending dependency work preserved. |

The schema-2 inventory reports 5,454 library/app lines (4,652 Engine, 419 Types,
383 Wire), 16,217 tests, 340 benchmarks and 285 tooling lines; zero unclassified.
The total is 22,296. Category changes are not code savings. Fifteen conservative
macro/conditional-attribute diagnostics were reviewed: they describe ordinary
macros and feature-controlled Serde/Wire derives, not hidden module paths.
Evidence and focused qualification logs: `target/shared-tooling030-audit/`.
All 156 snapshot payloads verify. Tooling Clippy and both benchmark script runs
pass; the actual release-selector fixture passes both groups, shared expansion,
required/excluded failures (including Engine default tests), drift and retained
evidence checks using substituted stage bodies. Cargo ownership, Git-side
retention, manifest sorting, Rust formatting, changed-shell ShellCheck and all
740 local documentation references pass. The incoming Wire/derive 0.17.1 lock
selection was preserved; these tooling checks are not a new full qualification
of that dependency patch or a release gate.
This is repository practice/caller review, not a new cryptographic or production
security audit. Publication remains held for the Platform Wasm fix.

## Wire 0.17 / Testkit 0.86 adoption; publication held

Published Testkit 0.86.0 resolves the native `Limits` identity blocker. Auth,
Storage and Testkit now resolve one Wire 0.17.0; schema generation explicitly
selects its metadata derive through Auth's existing feature forwarding. No
caller bridge or production Rust source change is needed. Protocol and Engine
pending notes now select 0.34.0 for the exposed Wire/Protocol identity cut;
package versions remain unchanged pending release preparation.

Qualification passes: 12 transport tests, 21 session tests, 70 browser tests
plus Rust/TypeScript interoperability, native runner compilation/Clippy, the
release Wasm build and passive Wasm dependency checks. All five generated browser
files are unchanged. The prepared Testkit 0.86 CLI admits PocketIC 16.1.0;
all 11 ordinary live cases pass, including certification, upgrades and signed
ingress. The real Chromium case remains opt-in and was not run. Logs and the
qualified lock checksum are under `target/wire017-testkit086/`.

The maintainer explicitly chose to hold publication until the Platform 0.22
Wasm regression is resolved. Testkit/Wire adoption is local qualification only;
do not interpret the earlier deploy request as permission to release this
unresolved size increase.

## Earlier Storage 0.67 qualification

The private checkpoint host now selects published Storage 0.67.0, which uses
Wire 0.17 internally without the metadata derive. Auth and Testkit still select
Wire 0.16.2: latest published Testkit 0.85.1 exposes that version's `Limits`.
No Wire values cross the Storage adapter boundary, so Storage can advance
independently. Public Auth library graphs still exclude Storage; no public API,
backing format or reset policy changes. Engine's compatible 0.33.3 draft records
this adoption, without changing package versions or authorizing publication.

All ten native checkpoint tests, the release Wasm build and Wasm Clippy pass.
All five browser files regenerate unchanged; 70 browser tests and Rust/TypeScript
interoperability pass. The lock-selected Testkit 0.85.1 CLI is now prepared and
the focused installed checkpoint upgrade passes, preserving sessions/denials
and requiring fresh authority after restore. Evidence: `target/storage067-adoption/`.
Platform's unresolved Wasm increase remains a separate delivery hold.

## Canic certified-issuance handoff

The [adoption guide](../canic-adoption.md#read-only-canic-source-review--2026-11-04)
now reflects read-only Canic HEAD `e6a061999` plus its existing working changes,
separating recorded local acceptance from production obligations. The
[certified issuance handoff](../signature-preparation.md#certified-portable-issuance-handoff)
maps the existing preparation, root publication, query retrieval and receiver
verification APIs to the consumer's durable journal. It highlights that queries
cannot commit completion, query certificate bytes can change, lookup-only
reconciliation must preserve exact committed results, and volatile leaf recovery
is a signing decision rather than journal restoration. Existing installed source
examples are linked; no new API, consumer edit or runtime qualification is claimed.

## Earlier direct Wire 0.17 attempt

The requested published Wire 0.17.0 candidate passes 12 transport tests,
21 session tests and all 70 browser tests plus Rust/TypeScript interoperability.
All five regenerated browser files are byte-identical. Existing schema feature
forwarding already enables Wire's now-optional metadata derive.

The native qualification runner cannot compile: published Testkit 0.85.1's
`WireCallSpec` takes Wire 0.16 `Limits`, while Auth's candidate contracts expose
Wire 0.17 `Limits`. Testkit 0.85.1 is the latest published selection checked for
this attempt. Storage then retained an internal Wire 0.16 dependency. No legacy
adapter or sibling source change was introduced. The pre-attempt Wire 0.16.2
lock and 0.16 minimum are restored, preserving the incoming Testkit 0.85.1 and
Types 0.18.1 selections. Candidate lock and focused logs are retained under
`target/wire017-adoption/`. Completing adoption needs a published matching
Testkit; Protocol's exposed Wire identity will require a minor release.

Platform 0.22 remains local and held from delivery over the unresolved Wasm
increase. The decoder experiment below does not measure Platform 0.21 versus
0.22 through Canic's release pipeline. No release is authorized by this update.

## Decoder proposal: defer after Canic artifact comparison

The shared-decoder API is not recommended for the measured Canic consumers.
Frozen Canic working sources at HEAD `0509ec870` plus existing dirty work were
captured read-only with matching before/after/export hashes. Its actual
`delegation_issuer_stub` and `local_authorization_probe` roles were built
separately through Canic's role validator and complete release artifact builder:
opt-level z, Full LTO, one codegen unit, symbols stripped, ic-wasm 0.11.1 and
checksum-admitted Binaryen 133. Both arms use the same recipe and test release ID.

| Role | Total bytes before → prototype | Code payload bytes before → prototype |
| --- | ---: | ---: |
| Issuer | 3,395,544 → 3,395,663 | 3,160,993 → 3,161,118 |
| Receiver | 3,069,213 → 3,069,345 | 2,851,587 → 2,851,712 |

The prototype adds 119/132 total bytes and 125 code bytes respectively; it does
not improve final Wasm size. Import/export sets and exact Candid/Wire contracts
match. The compiler-only Auth-fixture savings below do not justify a new public
Platform API for these Canic roles. No optimizer or maintained implementation
was changed. This is build/contract evidence, not installed Canic acceptance or
production instruction/latency qualification.

The controls select Auth 0.33.2, Protocol 0.33.1, Platform 0.22 and the required
Host Tools 0.82 pairing; Canic's former Host Tools 0.81 exposes incompatible
Platform 0.21 ledger identities to its host builder. Query's independent native
Platform 0.19/Host Tools 0.79 selection is preserved. These selections and all
prototype edits exist only in Auth-owned target scratch; Canic's live checkout
was not edited. The normal ambient Binaryen was rejected, so both arms explicitly
use Canic's already prepared, hash-admitted tools through its public toolchain API.
Evidence: `target/canic-decoder-probe/measurements.json`, final artifacts/contracts,
`artifacts.sha256`, source capture and logs under that directory.

## Earlier Platform/Testkit dependency qualification

The requested Wire update selects published Wire/derive 0.16.2 in the incoming
lock; the compatible root minimum remains 0.16. Regeneration reviewed all five
browser source files with no byte changes. All 70 browser tests and Rust/TypeScript
interoperability pass, as do 16 transport contract tests, 35 session/restore tests
and nine request/configuration tests. Native and Wasm Wire/session dependency
graphs pass the existing Auth rules. Evidence: `target/wire-platform-adoption/`.
Protocol's 0.33.2 draft records the adoption. No release or publication is requested
for this batch; existing package versions are unchanged.

Platform 0.22.0 is adopted locally after the maintainer approved the scoped
upstream certificate exception in [AGENTS.md](../../AGENTS.md). Certificate
capabilities and the reference Wasm admit transitive Candid/raw IC0 through the
upstream helpers. Default/session, wallet-signature, passive and unknown future
capabilities retain strict exclusions. Explicit Platform Candid/runtime features,
native tooling and library executors remain forbidden. Auth-owned models and
RPCs remain Wire-only; clocks, persistence and root publication remain host-owned.
The Engine 0.33.3 draft records the adoption; Auth's public APIs and Types 0.18
identity are unchanged. Testkit 0.85.0 retains a separate native Platform 0.21
facade; actual Wasm resolves one Platform 0.22 and one Runtime/guard 0.19.1.

Testkit 0.85.0 is adopted and its exact CLI/server pairing is explicitly prepared.
Auth's existing managed startup supplies the newly required server endpoint;
native caller Clippy, the CLI/server caller fixture and offline admission pass.
The ordinary reference Wasm passes all 11 live cases under Testkit 0.85, as does
the measured Full-LTO candidate below. The separate Chromium case remains opt-in.
Testkit stays outside the Wasm graph. Evidence: `target/testkit085-wasm/`.

All 204 Auth tests/doctests, strict-signature feature-unification tests, native
and Wasm Clippy, native/Wasm capability graphs, scoped graph refusal/acceptance
fixtures and isolated package/MSRV consumers pass. Live qualification passes
11 cases, including certification, upgrades, protected authority, signed ingress
and SDK admission/logout; the existing opt-in Chromium case remains ignored.
The final combined lock again regenerates all five browser files unchanged and
passes all 70 browser tests plus Rust/TypeScript interoperability. ShellCheck
passes for changed scripts. Evidence remains under `target/wire-platform-adoption/`.

The matched reference-canister comparison separates Platform and LTO changes:

| Platform | LTO | Complete module (bytes) | Code-section payload (bytes) |
| --- | --- | ---: | ---: |
| 0.21 | None | 2,068,236 | 1,579,582 |
| 0.22 | None | 2,243,989 | 1,709,668 |
| 0.21 | Full | 1,870,060 | 1,473,539 |
| 0.22 | Full | 2,081,586 | 1,625,165 |

All four builds use one exported source path, identical Rust source, Wire 0.16.2,
Testkit 0.85.0, Rust 1.99.1, opt-level 3, 16 codegen units, symbols retained and
Wasm `cpubits="32"`, without a post-link optimizer. Only Platform and LTO differ.
Section-10 payloads measure code; all import/export sets match. Full LTO helps
both generations, but the Platform upgrade still adds 211,526 total and 151,626
code bytes at matched Full LTO. It is not a fix for the dependency regression.
The initial Thin-LTO candidate was larger still: 2,584,708 total / 1,930,043 code.

Original symbol attribution assigns +103,953 code bytes to CBOR-generated
functions; several large decoder instantiations grow from three copies to four.
Function-name metadata adds 43,549 bytes; BLS-attributed code falls 4,943 bytes.
These symbol buckets are attribution observations, not isolated source changes.
Full-LTO's candidate passes the complete 11-case live suite. Compiler/linker
experiments remain evidence only: no maintained profile or optimizer was changed.
The test fixture is not Canic's production artifact or a production workload.
Matching source, artifacts, metadata, locks and section measurements remain under
`target/testkit085-wasm/apples/`; earlier attribution remains under
`target/wire-platform-adoption/attribution/`. Previous publication receipts below
retain their original inputs.

Decoder-boundary investigation found four compilation owners: upstream verifier,
Platform's key/time evidence adapter, Auth's signature store and private profiling.
A target-only prototype shares a non-inlined upstream
`decode_certificate(&[u8]) -> Result<Certificate, serde_cbor::Error>` through
Platform. Its body remains `serde_cbor::from_slice(bytes)`; existing checks and
decode operations remain. A real API should select CBOR explicitly rather than
expanding passive certification graphs. No maintained implementation was changed.

With private profiling unchanged, the matched Platform/Auth reuse experiment is:

| LTO | Complete module before → after (bytes) | Code payload before → after (bytes) |
| --- | ---: | ---: |
| None | 2,243,901 → 2,100,272 | 1,709,668 → 1,602,895 |
| Full | 2,083,618 → 1,921,078 | 1,625,732 → 1,510,799 |

These are path-patched published-Platform-copy controls; compare within each
row, not against registry-source artifacts with different package identities.
All import/export sets match. The common decoder's library changes pass 34
focused native tests, Clippy and preparation-only compilation. The exact narrow
no-LTO artifact passes all 11 live cases; Chromium remains opt-in. Sharing the
private profiling decoder as well saves an additional 71,567 total / 49,689 code
bytes in the no-LTO fixture, separately qualified with 11 live cases. A helper
kept private to Platform alone gives no meaningful saving. No production-size,
latency or parse-count reduction is inferred. Evidence, source variants, diffs,
locks and checksums: `target/decoder-boundary-probe/`. The later Canic comparison
above supersedes the proposal to publish this helper for these consumers.

## Published cohort

Protocol 0.33.1 (`ic-auth-protocol-types`, `ic-auth-wire`) and Engine 0.33.2
(`ic-auth`) are published on crates.io. Both annotated tags and the release
commits are pushed to `origin/main` from the normal checkout. Registry identity
and archive checksums were observed by Auth's publication adapter:

| Package | Version | Registry SHA-256 |
| --- | --- | --- |
| `ic-auth-protocol-types` | 0.33.1 | `48da6380603cea3b451c43a9fc5fb580f487940f57571969ebae2266cb9cf000` |
| `ic-auth-wire` | 0.33.1 | `c57063776628aa608060aedeb95ac9246763652d053212e21eac1e2b6faa6cf8` |
| `ic-auth` | 0.33.2 | `8aca377ec4e9abeefbf03a363cf6007203f3a3a703614fb8ad898d02af1892fc` |

Protocol release commit: `4dec02e083181f37706df8046c6dd9cdbe2bfcb8`, tag
`protocol-v0.33.1` (`7597795af1e0ec38dca235af37677c9816151ef0`). Engine release
commit: `f9dc978505832ca3a32a068feee7cdaa317a272e`, tag `engine-v0.33.2`
(`5ee62f2f9c5d950b14ed825d458d0ff1a86ee644`). Completed plans and checksum-bound
publication custody remain under `.git/release-state/` and `.git/publication-state/`.

All 30 stages passed separately for both standard releases, including native/
Wasm Clippy, isolated package/MSRV consumers, browser lifecycle/interoperability,
real canister certification/upgrade/signed ingress, archive builds and Auth's
release/recovery/publication fixtures:

- Protocol source `9168a91`:
  `target/release-validation/protocol-gate.H39gaN/logs/validation.mqtPDn`.
- Engine source `f61f4ea`:
  `target/release-validation/engine-gate.fiypz8/logs/validation.VQ5F8b`.
- Delivery/upload logs: `target/auth-033-delivery/`.

The released Engine lock selects Metrics 0.29.1, Storage 0.66.2, Host/Build 0.74.3,
Runtime/macros/guard 0.19.1, Platform 0.21, Types 0.18, Testkit 0.84.1 and
Wire/derive 0.16.1. Rust 1.99.1 remains the compiler/minimum. Shared Tooling
0.29.4 is adopted at `628ccc75d813bfd9bae777bfbaef55838d25b1b2`; all 150
snapshot payloads match the canonical pinned-revision export.

## Delivered behavior and adoption support

Signature-output serialization now refuses oversized writes before appending,
with unchanged CBOR envelope bytes and exact-boundary admission. Cryptographic
checks, signed material and the TEE/non-TEE trust model are unchanged. No Wasm
saving or latency improvement is claimed.

The backend bundle now has eight scenarios, adding bounded cleanup rollback and
retained clock/generation checks. Drivers supply prune/clock hooks and fallible
generation updates. Its 15 tests include deliberately broken bounds, partial
writes, generation updates and lost clock floors. The clean release archive was
qualified after publication against the exact registry cohort, with trait/error
identity admission, graph refusals and all 15 packaged tests passing:

- Receipt: `target/portable-fixtures/backend-package.WoSpAu/delivery.json`.
- Archive: `target/portable-fixtures/backend-package.WoSpAu/target/package/ic-auth-backend-conformance-0.33.2.crate`.
- Archive SHA-256: `aa0cb3efb9e23089610d420bc00ab79915bc464952f404f11dd911d9c9bbcbba`.
- Source `f9dc978`, dirty false. This unpublished source bundle is dev support;
  actual Canic backing, clock provenance and installed acceptance remain Canic-owned.

The unchanged exact-retry path was profiled natively: 100 calls each showed five
allocations/116 bytes for small claims and 545 allocations/16,736 bytes for wide
claims, matching standalone decode. Setup/signing/request construction were
excluded. No retry optimization was implemented. Evidence:
`target/portable-fixtures/retry-allocations.x76x4g/`;
[measurement scope](../retry-allocation-observations.md).

Rust unit tests/private helpers use ordinary adjacent `tests.rs` discovery.
All 263 Rust test attributes are under `tests.rs` or `tests/`; API doctests stay
with their declarations. Metrics recorders no longer own report serialization;
Canic's shared adapter owns reporting. Auth's release/publication callers apply
Shared's retained-state guard and put new source extracts/gate logs under
`target/`, preserving original failure status and historical recovery paths.

## Interrupted attempts and retained history

Incoming dependency updates invalidated three earlier attempts. Their passing
stages are historical observations, never substituted for the final receipts:

- Metrics/Storage drift: `target/release-validation/protocol-gate.z5fVNr/logs/validation.pPPrEH`.
- Host/Runtime drift: `target/release-validation/engine-gate.GYD3n3/logs/validation.ZdiDU0`.
- Build drift: `target/release-validation/engine-gate.rEEpuX/logs/validation.GxoZGh`.

The incoming selections were preserved and committed, then completely qualified
in the successful attempts above. No interrupted attempt created an Engine tag
or upload. The maintainer paused all Auth updaters for the final delivery.

The redundant `ic-auth-capacity` and `ic-auth-release-033` sibling checkouts were
removed with maintainer approval after confirming their work was integrated.
Their refs remain `archive/ic-auth-capacity` and `archive/ic-auth-release-033`.
Required compact historical evidence is at
`target/checkout-retirement/20261104-siblings/manifest.json`. Two verified,
reproducible old Git-side metadata extracts were also removed; receipt:
`target/shared-tooling-0294/recovery-review/removal.json`. No bulk checkout/build
archive was retained in `.git`. Other recovery/publication state is preserved.
Explicit `cargo clean` can remove all target-owned evidence.

## Remaining ownership

[Canic adoption](../canic-adoption.md) describes consumer contracts. Canic
[#491](https://github.com/dragginzgame/canic/issues/491) owns production acceptance;
Auth [#125](https://github.com/dragginzgame/ic-auth/issues/125)'s previously pending
native acceptance is now verified: [run 41680139546](https://github.com/dragginzgame/ic-auth/actions/runs/41680139546)
completed successfully at Engine 0.33.0 commit
`fa6823a25cedb04f55f91301efcf261ab4109b1f`. Ubuntu 24.04, Intel macOS 15 and
Apple Silicon macOS 15 each passed the explicit full-versus-selected release-group
comparison and retained its evidence. The issue was closed at the maintainer's
request on 2026-11-05; this historical release proof does not qualify the current dirty
dependency batch or production Canic adoption. The previous browser/backend handoff
at `target/platform-021-delivery/delivery/manifest.json` remains separate Engine
0.33.1/Protocol 0.33.0 evidence; it does not qualify the new backend hooks.
[Earlier delivery history](history-0.33-delivery-and-qualification.md) remains
historical. Continued implementation uses the normal checkout, without new
sibling directories.
