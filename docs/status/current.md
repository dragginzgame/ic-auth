# Current handoff — 2026-11-08

## Published state

**Engine 0.35.8** (`ic-auth`) and **Protocol 0.35.1**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
Their release commits and annotated tags are pushed to `origin/main`.

- Engine: `9ed4afbd74c27f56e00aed2dea18fcb195ffb674`, tag `engine-v0.35.8`
  (`1a4e6e888e9465e8f4ae5f1ac5f081d0af540b46`). Registry checksum:
  `e2cd57a28f4e190efef2d80a54bad323b4b47ac4d353e8e2d3f566da81e1defa`.
- Protocol: `39b6e2605ad336c3aadff94bfd8af0dcb07cf8b3`, tag `protocol-v0.35.1`
  (`e13957c26a45f90cecfa88c88f884342c6be04e5`). Registry checksums:
  types `bb4db5f8c7da4dbfd2095cf3b9eff1b722fa4a6fd0e8bca04df05ed680183c99`,
  Wire `9d147f39648df63d46c779ac7c44f666a6dcdacb6824da9b602f5ec164c9757c`.

Custody: `.git/publication-state/engine-0.35.8.json` and
`.git/publication-state/protocol-0.35.1.json`. Upload/readback directories:
`target/release-frozen/batch-YyKgHPDM/target/publication-protocol-0.35.1.xj7Apl/` and
`target/release-frozen/batch-YyKgHPDM/target/publication-engine-0.35.8.1GVwin/`.
The batch is complete and fast-forward reconciled; its durable recovery remains
at `.git/release-state/frozen/batch-YyKgHPDM/`. No upload is unresolved.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) remains closed.

## Completed Protocol 0.35.1 / Engine 0.35.8 delivery — 2026-11-08

The authorized supplier-first frozen batch `batch-YyKgHPDM` completed both
registry publications and checksum readbacks. Source commit
`f65f22700e05e15ade75a082c69e26a0fe9387ed` was captured from normal `main`;
the Protocol release then supplied Engine's source. Both release commits and
annotated tags are pushed to `https://github.com/dragginzgame/ic-auth.git`.
The normal checkout was clean and fast-forward reconciled. Older batches,
receipts, archives and raw evidence remain retained.

Each group passed a fresh, complete **30-stage Linux release gate**:

| Group | Gate source | Retained logs |
| --- | --- | --- |
| Protocol | `f65f22700e05e15ade75a082c69e26a0fe9387ed` | `target/release-frozen/batch-YyKgHPDM/target/release-validation/protocol-gate.vJVFGU/logs/validation.9wobJA` |
| Engine | `39b6e2605ad336c3aadff94bfd8af0dcb07cf8b3` | `target/release-frozen/batch-YyKgHPDM/target/release-validation/engine-gate.vaIxEo/logs/validation.XXhUc6` |

Both include native/Wasm Clippy, isolated archive/MSRV checks, boundaries,
all 93 client tests and Wire interoperability, 20 backend-conformance tests,
all 11 ordinary installed certificate/session/upgrade/ingress and SDK cases,
local archives and release/recovery callers. Chromium remains opt-in/ignored.
No gate or stage was reused. The final release-tools fixtures took 1,034 seconds
for Protocol and 1,039 seconds for Engine. Hosted CI remains asynchronous;
secret scanning stays outside release preparation, group gates and publication.

The released selection is Platform 0.26.0, Types 0.18.2, Wire/derive 0.17.13,
Storage 0.68.10, Runtime/macros/guard 0.20.1, Testkit 0.93.1, Build 0.79.0,
Host 0.75.0, Metrics 0.29.6 and Shared Tooling 0.31.10. Testkit alone retains
Platform 0.25.1 in native qualification; the Wasm host has one facade and one
Runtime executor/generator/guard. Rust remains 1.99.1. No Auth field selects
the new packed-ID codec; method identities, signed bytes and stored formats
are unchanged.

The Protocol-gate reference Wasm exactly reproduced the earlier 1,898,602-byte
adoption artifact. After Protocol 0.35.1, the Engine-gate reference is
**1,898,687 total / 1,461,817 code-section bytes**,
SHA-256 `1ebded2e43a28e3d559387263f768f6e0ca3028dd1eb24180cd44565a7593eea`.
The latter adds 85 non-code bytes with the same code-section size. Against the
previous published reference it is +259 total / -37 code bytes. Compiler and
release profile are unchanged; no post-optimizer or throughput claim is involved.
These are reference qualification artifacts before each group's version
preparation, not a deployed Canic binary. Receipts, delivery and reconciliation
logs are retained under `target/delivery-0358/`.

## Shared Tooling 0.31.10 adoption — 2026-11-08

The normal checkout adopts committed Shared Tooling 0.31.10 at
`c5285791991c731d266fb5c85dc3ee876d29f6b9` through the canonical exporter.
The 165-file closure explicitly selects the shared download-cache action and
pinned CI-tool installer. Auth's CI restores authenticated downloads before
setup and saves after admission, before tests. Its existing Cargo build cache
and product setup remain intact. A separate, unconditional Linux repository
secret job owns the full-history Gitleaks scan outside the host matrix; no
scanner step enters Make CI, release-group gates, setup or publication.

The first pinned scanner download exhausted bounded retries with HTTP 504.
The identical official asset was obtained through GitHub's asset API, verified
against the catalog SHA-256, then re-admitted through the canonical installer's
authenticated cache. The initial history scan found 21 source SHA-256 rows,
two generated diagram edges and one historical prose comment. The default
rules remain enabled: only the generic heuristic receives path-and-full-line
exceptions for those non-secret formats, and the comment has one exact finding
fingerprint. The final full-history scan passes. Six controls confirm allowed
forms, credential rejection in both exception files, Rust credential rejection
and refusal to apply the source-hash exception outside its metadata file.

Workflow lint, dependency pins, snapshot integrity, host-tool admission and
Auth's actual client preparation/tool-evidence caller fixtures pass. New crate
and `crate::module` LOC selections preserve classified counts; unknown modules
refuse without a partial report. Evidence and previews are retained under
`target/shared-tooling-adoption-03110/`. Hosted cache service execution and
macOS remain untested locally; no hosted CI run was launched or awaited.

Existing Build/Platform/Wire/Storage selections, Cargo files and all five
generated browser files remain byte-identical to the incoming selection.
That earlier adoption's focused evidence remains scoped as recorded below.
This Shared update alone was private tooling maintenance. Publication above was
selected by the public dependency changes and the maintainer's deploy request.

## Build / Platform / Wire / Storage adoption — 2026-11-08

The unpublished workspace selects Build 0.79.0, Platform 0.26.0, Wire/derive
0.17.13 and Storage 0.68.10. Incoming Runtime/macros/guard 0.20.1 are preserved.
Auth uses neither Build's removed batch APIs nor Platform's retired BLAKE3
facade, so no Rust source migration is needed. The compatible Wire minimum is
now 0.17.13; no Auth field selects its new opt-in packed 16-byte ID encoding.
The four regenerated method files are unchanged. The reviewed fifth file, the
shared browser runtime, adds `hash16` support without changing Auth schemas,
method identities, bounds or signed/stored bytes.

Focused checks pass: 216 Auth cases including doctests, 17 Wire/contract/fixture
cases, 13 private-host cases, 4 native tooling cases and 37 adversarial
feature-unification cases. All 93 client tests, packed-consumer checks and
Rust/TypeScript interoperability pass. Native/Wasm Clippy and actual dependency
boundaries pass. The host has one Platform 0.26 and one Runtime executor,
generator and guard; Testkit alone retains Platform 0.25 in native qualification.
Build 0.79's release editor is explicitly prepared and admitted offline.

Cargo.lock changed externally during qualification only to Testkit 0.93.1.
The offline native lint and installed check refused its unprepared source/tool;
that incoming patch was retained and explicitly prepared. Native/Wasm lint,
final generator comparison and all 11 ordinary installed certificate, upgrade,
ingress and SDK cases then passed with the final lock. PocketIC remains 16.1.
The earlier client and pure-library results retain their original Testkit 0.93.0
lock; their selected graphs do not include Testkit, and no relevant source or
dependency changed. Chromium remains opt-in/ignored. These focused checks preceded the complete
release gates recorded above.

The reference Wasm is **1,898,602 total / 1,461,817 code-section bytes**,
SHA-256 `a335cb0822568d9afb9d269ab043459f859da0f41400fe30f50f80737edcb9a2`.
Against the published reference it is **+174 total / -37 code bytes**, using the
same release profile and no post-optimizer. This is a combined dependency/version
comparison, not a matched single-dependency benchmark or throughput claim.
Testkit's later native-only patch does not change that Wasm graph or artifact.
Logs, the initial/final lock selections and the size receipt are under
`target/upstream-adoption-20261108/`; installed state is retained at
`target/portable-fixtures/qualification.0uu5P6/`.

The existing Shared Tooling adoption was preserved. Public dependency requirements
changed, selecting the Engine 0.35.8 and Protocol 0.35.1 release notes. That later
authorized delivery is recorded above; these earlier observations retain their
original source, package versions and evidence scope.

## Shared Tooling 0.31.9 adoption — 2026-11-08

The normal checkout adopts committed Shared Tooling 0.31.9 at
`d95f271bc7b49951ccf286c3655cef7446b8cf56`, using the canonical exporter's
explicit revision selection. Unrelated 0.31.10 drafts in the source checkout
were excluded without creating or switching checkouts. The unchanged entrypoint
selection now closes over 163 payloads, including the new shared hashing policy
and inventory. Preview, impact and source verification are retained under
`target/shared-tooling-adoption-0319/`.

Auth's existing formatting entrypoint now selects only its owned Cargo members;
actual formatting/hook preservation checks pass. The actual direct-dependency
checker passes with shared hash/crypto admission enabled; Auth already uses the
approved direct SHA-256 implementation. `make cloc CLOC_DEPTH=0` and
`CLOC_DEPTH=2` expand crates/modules with equal category totals and no
unclassified Rust. Snapshot integrity, documentation links and the diff pass.
These are focused consumer checks, not a new full release or macOS result.

That adoption alone was private tooling/governance maintenance: public library
sources, manifests, readmes, dependency declarations and the lockfile remained
unchanged from the published release. It did not select publication. Subsequent
dependency-adoption work has its own scope and evidence above.

## Completed Engine 0.35.7 delivery — 2026-11-08

Engine publication completed through Shared's managed frozen batch
`batch-FqmjNnSq`. Protocol remains 0.35.0. The selected lock includes Platform
0.25.1, Types 0.18.2, Testkit 0.93.0, Runtime/macros/guard 0.20.0, Storage 0.68.8,
Wire/derive 0.17.12, Metrics 0.29.6, Host 0.75.0, Build 0.78.0 and arc-swap 1.9.4.
Auth and native Testkit now share one Platform 0.25 identity; the Wasm host retains
one Runtime executor/generator/guard. Rust remains 1.99.1.

A fresh complete **30-stage Linux release gate** passed against captured source
`1c32d5aecee88cfed75826161368e23d001b0515`:
`target/release-frozen/batch-FqmjNnSq/target/release-validation/engine-gate.24sIwL/logs/validation.gCQLRL`.
This includes all 93 client tests and Wire interoperability, 20 backend-conformance
tests, all 11 ordinary installed certificate/session/upgrade/ingress and SDK cases,
native/Wasm Clippy, isolated package/MSRV checks, package archives and release/recovery
callers. Chromium remains opt-in. The final release-tools stage took 1,047 seconds.
No earlier gate or stage was reused. Hosted CI remains asynchronous.

The gate's reference Wasm is **1,898,428 total / 1,461,854 code-section bytes**,
SHA-256 `363611d69dbd622270141e6401207a1476eb83448d5f04bbfbb8b049de15b196`.
It exactly matches the final measured normal-checkout artifact. On matched
compiler, lock and flags, the retained cleanups save **6,716 total / 6,896 code
bytes** against the initial cleanup baseline. There is no 200 kB regression or
production-throughput claim. Receipt: `target/delivery-0357/release-wasm.json`.

Initial setup stopped on HTTP 504 while fetching PocketIC 16.1.0. Testkit 0.93
subsequently authenticated the exact pinned archive and executable retained in
the previous completed frozen batch, before and after copying only those assets
into this batch. The canonical setup/check then passed. Hash evidence is at
`target/delivery-0357/pocketic-assets.sha256`. A subsequent resume refused the
not-yet-prepared IC Build editor; explicit saved-source setup prepared Build
0.78.0, after which the same batch ran the fresh full gate. Failed attempts and
recovery evidence remain retained; no input-drift or tool admission was bypassed.
Delivery, setup, registry readback and reconciliation logs are under
`target/delivery-0357/`. The normal checkout was clean and fast-forward reconciled.

## Complexity cleanup and matched Wasm measurement

Four cleanup candidates are retained: pending/completed browser issuance types
with the same stored profile; one private receiver-configuration projection
roster; one fixture login lifetime/ledger policy for installation and restoration;
and canonical field encoding shared by byte output and streaming SHA-256.
There is no public Rust API, signed-byte, verification-rule or stored-layout cut.
The existing upstream cryptographic primitives remain unchanged. Length-framed
hashes count the same encoder output before hashing; an overflow test refuses
before the hashing pass. Standalone encoders retain their structural checks.

Matched normal-checkout release Wasm builds use Rust 1.99.1, identical Cargo
settings/target flags, no post-processing, and the same saved lock throughout.
That incoming lock already selects Platform 0.25.1, Storage 0.68.8, Testkit 0.93,
Runtime 0.20 and Wire/derive 0.17.12; it was preserved, not changed for the experiment.

| Stage | Total bytes | Code-section bytes |
| --- | ---: | ---: |
| Before cleanup | 1,905,144 | 1,468,750 |
| Fixture cleanup | 1,905,029 | 1,468,740 |
| Retained streaming hashes | 1,899,691 | 1,462,917 |

That stage's final artifact is byte-identical to its retained hashing stage: **5,453 fewer
total bytes and 5,833 fewer code bytes** than baseline. Generic callback splitting
was evaluated separately and discarded: the combined split added 3,944 total /
3,108 code bytes over the hashing stage; the isolated revocation split also grew.
No callback-plan abstraction or compiler inlining override is retained.

Native DHAT observations use unchanged synthetic small and 16-role/32-scope
inputs, 100 iterations and equal digests. Certificate, complete-proof, header,
issuer-leaf and derivation-path hashing now allocate zero heap blocks; the wide
complete-proof case previously allocated 32,640 cumulative bytes per operation.
Portable-claims hashing was unchanged in that stage. These are allocation and Wasm-size results,
not runtime-instruction or production-throughput claims.

Focused Rust canonical/proof/session/refusal suites, fixture conversion tests,
93 client tests plus packed-consumer/Wire interoperability, native/Wasm Clippy,
and all 11 ordinary installed certificate/session/upgrade/ingress and SDK cases
pass. Chromium remains opt-in/ignored. Evidence, exact source/input seals, native
reports and all measured artifacts are in `target/evidence/complexity-20261108/`.
These focused checks preceded the complete Engine 0.35.7 gate and delivery
recorded above; their evidence retains its original, narrower scope.

The portable-claims follow-up also shares one bounded encoder between byte output
and SHA-256. Fresh matched builds on the same lock/compiler/settings move from
1,899,691 to **1,898,461 total bytes**, and 1,462,917 to **1,461,887 code bytes**:
another **1,230 total / 1,030 code bytes saved**. Combined with the earlier batch,
the reductions are 6,683 total and 6,863 code bytes. That artifact's SHA-256:
`6322118651dd380c3a5f54abedda6aaef7517714a0b986935e0e4940cc795fed`.

At 100 native DHAT iterations per profile, portable-claims hashing moves from
5 blocks / 868 cumulative bytes (small) and 9 blocks / 14,308 bytes (wide) per
operation to **zero**. All benchmark digests match. Frozen and wide/mechanism
vectors, hash/encoding structural-refusal ordering, 115 focused Rust cases,
native/Wasm Clippy and all 11 ordinary installed cases pass. Chromium remains
ignored. The earlier client results retain their original stage; no frontend or
protocol changes required another client suite. Evidence and exact input/source
seals: `target/evidence/portable-claims-20261108/`; installed state:
`target/portable-fixtures/qualification.ZK5rAO/`. No public API, stored format,
signed bytes, crypto primitive, verification policy or dependency selection changed.

The fixed certified-message follow-up replaces its temporary `Vec` with a 60-byte
stack array; a compile-time check retains the one-byte domain-length bound.
Matched builds on the unchanged lock/compiler/settings save another **33 total
and 33 code bytes**: latest artifact 1,898,428 total / 1,461,854 code bytes,
SHA-256 `363611d69dbd622270141e6401207a1476eb83448d5f04bbfbb8b049de15b196`.
Cumulative savings across these cleanups are 6,716 total and 6,896 code bytes.
Fifty focused signature/certified-proof/refusal cases and native/Wasm Clippy
pass. All 11 ordinary installed cases pass against that exact artifact;
Chromium remains ignored. Proofs produced with the existing general message
helper confirm unchanged framing; no whole-verifier allocation or performance
improvement is claimed. Evidence: `target/evidence/certified-message-20261108/`;
installed state: `target/portable-fixtures/qualification.Yimzgb/`.

## Testkit 0.93 adoption

The workspace and lock select published Testkit 0.93.0
(`be8a613d17037c20953015fc85bdd46c4291799d97e93ab924c9e89ddd185e6b`).
Native CBOR errors now use Testkit's `CborErrorKind` with their original typed
source. Auth does not match the retired error variant, so no Rust source migration
is needed. The final qualified lock shares Platform 0.25.1 between Testkit and Auth; the older native Platform 0.24
identity is removed. Explicit 30-second request/startup budgets and managed async
instance/server teardown are retained. PocketIC remains 16.1.0.

The incoming arc-swap 1.9.4 selection is preserved and included in the focused
checks. Wire/derive remain 0.17.12: inspection of the captured release lock and
gate logs corrected stale 0.17.11 wording in the release/handoff documentation.
The generator check matches all five current browser files; there is no new
Wire selection or generated source change in this batch.

Focused checks pass: warning-denied native runner all-target Clippy, actual Auth
dependency boundaries and all 11 ordinary installed certificate/session/upgrade/
signed-ingress and SDK admission/logout cases. The opt-in Chromium case is
ignored. The lock-selected CLI was explicitly prepared and admitted offline.
Evidence and the stable selected manifest/lock are under
`target/evidence/testkit-0930/`; installed state is at
`target/portable-fixtures/qualification.csrYfU/`.
That earlier focused adoption retained one Runtime executor/generator/guard;
its receipt does not claim a byte-identical artifact or performance result across
checkout paths. The later complete Engine 0.35.7 gate above qualifies the final
Platform 0.25.1 / Storage 0.68.8 selection and all retained cleanup together.

## Completed Engine 0.35.6 delivery

The maintainer authorized and completed Engine publication. Protocol
remains 0.35.0: its source, four generated method bindings and declared compatible
dependency requirements are unchanged. The selected full lock includes Platform
0.25.0, Runtime/macros/guard 0.20.0, Storage 0.68.7, Host 0.75.0, Build 0.78.0,
Testkit 0.92.0, Wire/derive 0.17.12, crypto-common 0.1.6 and generic-array 0.14.9.
The Wire runtime was regenerated and all five files reviewed; only 19 packed-bool
runtime lines changed, with no Auth field using the opt-in adapter.

The fresh complete **30-stage Linux release gate** passed against captured source
`5e0cd37121c3d98fbce96565a1c1fbd1de203e95`, qualifying the combined graph:
`target/release-frozen/batch-ZU5ln9dY/target/release-validation/engine-gate.qxTd0M/logs/validation.a5OKs0`.
This includes all 93 client tests and Wire interoperability, 20 backend-conformance
tests, all 11 ordinary installed certificate/session/upgrade/ingress cases,
native/Wasm Clippy, MSRV, package archives and release/recovery callers. Chromium
remains opt-in. The final release-tools stage took 999 seconds; no timing saving
is claimed. Earlier focused receipts below retain their narrower selections.

The full-release reference Wasm is 1,905,656 bytes, with a 1,468,826-byte code
section; SHA-256 `a254de16e54abfe683902662c17398d0b52e43077d10e3468637e353edcf41e5`.
This is +270 total bytes and -1 code byte from the earlier aligned-cohort artifact.
Its Wire/support selections and checkout path differ, so this observation is not
a matched single-dependency benchmark. No 200 kB regression occurred. Receipt:
`target/delivery-0356/release-wasm.json`.

Delivery uses Shared's existing managed `target/release-frozen/BATCH` workflow,
with main/tag delivery to `origin` and crates.io publication. Later normal-tree
updates stay outside the captured batch. Hosted CI remains asynchronous.
Preparation and delivery logs are under `target/delivery-0356/`.

## Aligned Platform 0.25 / Runtime 0.20 / Storage 0.68.7 qualification

Workspace requirements and the qualified lock select Platform 0.25.0, Runtime,
macros and guard 0.20.0, and Storage 0.68.7. Storage's newly published Runtime
0.20 requirement resolves the earlier single-owner blocker below. The private
host retains exactly one Platform, Runtime, macro generator and singleton guard.
Auth already uses ordinary `ic_platform_runtime::cdk` entrypoints, so no Rust
source migration is needed. The graph policy drops the removed facade `cdk`
feature and its two synthetic cases; current facade/runtime, registry-executor
and public-library isolation checks remain enforced.

Focused checks pass: strict Wasm-host and native signature-target Clippy, all 51
certified/portable/chain-key cases, all 11 ordinary installed certificate,
upgrade, session and signed-ingress cases, ShellCheck/syntax, current boundary
refusal fixtures and actual native/Wasm capability graphs. Chromium remains
opt-in/ignored. Crypto algorithms, signed bytes, public contracts, trust inputs
and stored layouts are unchanged. Host 0.75, Build 0.78, Testkit 0.92 and Types
0.18.2 remain selected. Testkit retains Platform 0.24 only in its native helpers;
that older facade is absent from the canister graph. No error-identity bridge or
dependency override is introduced.

The matched reference Wasm compares Platform 0.24 / Runtime 0.19.5 / Storage
0.68.3 with this aligned cohort. Auth Rust source, compiler 1.99.1, flags and
release profile are unchanged. The baseline artifact is reused after verifying
its source, build context and dependency identities/features. Total bytes move
from 1,905,153 to 1,905,386 (**+233**); code-section bytes move from 1,468,844 to
1,468,827 (**-17**). This measures the three-dependency cohort, not Platform alone,
and makes no runtime-performance claim. Evidence, artifacts and the exact
qualification lock are in `target/evidence/platform-0250-runtime-0200/`.

This focused qualification held Wire/derive 0.17.6, crypto-common 0.1.7 and generic-array
0.14.7 fixed. Incoming Wire/derive 0.17.11, crypto-common 0.1.6 and generic-array
0.14.9 were preserved under `.git/release-state/deferred-platform-0250/` and
restored afterward for the combined release gate above. The captured incoming
lock had already
partially advanced Storage to 0.68.7 while retaining both Runtime generations;
the aligned single-owner selection supersedes that partial transition. Its
original bytes remain in that custody.
The combined released lock is not the exact graph of this focused size comparison;
its complete qualification is recorded above.

## Earlier Platform 0.24 qualification

The initial Platform-only qualification selected published Platform 0.24.0
(`6f2bafc5da7b457cb0efdb622151e74c3367740d26eb8892f72be2c9b4683c53`).
The release removes native offline public-key derivation and unconfigured Candid
decoder shortcuts; Auth uses neither. No Auth Rust source, public protocol,
signed bytes, verification policy or retained-record format changed. The aligned
cohort above supersedes this earlier development selection.

Focused qualification passes: warning-denied native signature-runner Clippy,
51 certified/portable/chain-key tests, all 11 ordinary installed certificate,
upgrade, session and ingress cases, and actual native/Wasm dependency boundaries.
The opt-in Chromium case remains ignored. This initial Platform-only check used
Testkit 0.90.0, which retained Platform 0.23.6 for native Candid/protocol helpers.
The Wasm host selected one Platform 0.24.0 and one Runtime/macros/guard 0.19.5.
The following native-tooling batch removes that remaining older native facade;
Types and Runtime selections are unchanged.

The matched reference Wasm builds use the same Auth 0.35.5 source, Rust 1.99.1,
target flags and Cargo release profile without post-processing. Platform 0.23.6
produces 1,904,961 total bytes and 1,468,843 code-section bytes; Platform 0.24.0
produces 1,905,153 and 1,468,844 respectively: **+192 total bytes, +1 code-section
byte**. No runtime-performance claim is made. Evidence, both artifacts, source
seals, build context and exact locks are under `target/evidence/platform-0240/`.

These checks hold Storage 0.68.3 and Wire/derive 0.17.6 constant. The separately
preserved incoming Storage 0.68.5 / Wire 0.17.9 delta is restored afterward and
remains unqualified, as recorded below. Its browser generator adoption is outside
this Platform-only batch; do not treat the combined working lock as released or
as the exact graph measured above.

## Earlier Host 0.75, Build 0.78 and Testkit 0.92 qualification

Workspace requirements and the selected lock now use Build 0.78.0,
Host Artifacts/FS/Process 0.75.0 and Testkit 0.92.0. The independently pinned
Host file CLI is also 0.75.0. Auth uses none of Build's retired compiler-wrapper,
prepared-input or progress APIs, or Host's retired provenance/RSS modules.
No Auth Rust source change was required. At this stage the native client shared
one Platform 0.24.0 and Types 0.18.2 identity; Platform 0.23.6 was removed from the lock.
Public Auth contracts, explicit request/startup budgets and managed teardown are
unchanged. PocketIC stays 16.1.0 and Rust stays 1.99.1.

Focused qualification passes: warning-denied tooling-binary and native
signature-target Clippy; all four actual file/lock/argument/contract adapter tests;
real Engine manifest/lock rendering with the selected Build editor; preservation
of Protocol versions and all external selections; the 8 MiB input-limit refusal;
and all 11 ordinary installed certificate, upgrade, session and signed-ingress
cases. The opt-in Chromium case remains ignored. All three selected executables
are explicitly prepared and their offline admission is exercised by these checks.
The Wasm dependency identities/features and retained qualification binary match
the earlier Platform 0.24 evidence exactly, so no additional Wasm rebuild or size
claim is needed for this native-only batch.

Evidence and the exact qualification lock are under
`target/evidence/host-build-testkit-20261107/`. Qualification holds Storage 0.68.3,
Wire/derive 0.17.6, crypto-common 0.1.7 and generic-array 0.14.7 constant. Incoming
updater edits instead select Storage 0.68.5, Wire/derive 0.17.9, crypto-common
0.1.6 and generic-array 0.14.9. That complete incoming lock and reapplication
patch are preserved under
`.git/release-state/deferred-host-build-testkit-20261107/` and restored after
qualification. They remain unqualified; the combined working lock is not the
exact qualification graph or a published release.

## Earlier Runtime 0.20 blocker — resolved

Published Runtime, macros and guard 0.20.0 are available. Auth already uses the
ordinary entrypoints through `ic_platform_runtime::cdk`; it uses none of the
removed prefixed macro exports. The remaining blocker is the host composition:
then-latest published Storage 0.68.6 required Runtime `^0.19.4` on Wasm with only
`ic0` enabled. It cannot resolve to the requested Runtime 0.20 owner.

A minimal dependency-resolution probe using Runtime 0.20.0/`cdk` and Storage
0.68.6/`test-support` resolves Runtime 0.19.5 (`ic0`) alongside Runtime 0.20.0
(`cdk`, `executor`, `ic0`, `runtime`), with macros/guard 0.20.0. The one-Runtime
package check correctly refuses two identities. This is not a claim of two
executors: the older package supplies raw IC0 only, but it still violates Auth's
single-owner composition requirement. No policy or dependency override was added.

That metadata-only probe left Runtime 0.19.5 and root manifest/lock bytes unchanged;
evidence remains under `target/evidence/runtime-0200/`. Storage 0.68.7 subsequently
published the compatible Runtime 0.20 requirement. The aligned cohort and fresh
host qualification above resolve this blocker without weakening the policy.

## Completed Shared Tooling 0.31.4 adoption and Engine 0.35.5 delivery

The normal checkout adopts reviewed Shared Tooling 0.31.4 at
`dad6d15750db66726850710f5fb423921b652c8d` through its canonical exporter.
All 161 payloads independently match that commit; existing entrypoints, catalogs,
package versions and the incoming Platform 0.23.6 lock were preserved during
adoption. AGENTS.md selects the new baseline; Engine 0.35.5 captures it.
The earlier 0.35.4 release retains its captured 0.31.0.

This repairs the checksum helper's Bash 3.2 parser issue seen in both macOS jobs
of [run 42359982273](https://github.com/dragginzgame/ic-auth/actions/runs/42359982273),
which stopped before Auth validation; Ubuntu passed. The snapshot also corrects
grouped-release command checks and LOC scratch handling after cleanup. No Auth
Rust, crypto, transport or stored-record implementation changed.

Focused Linux checks pass: actual host-tool admission; GNU/Perl checksum output,
NUL refusal and backend failure propagation; ShellCheck/syntax; both actual Auth
release-group adapters with inherited release settings and substituted runners;
and workspace LOC reporting from a previously absent scratch path with zero
unclassified lines. Fresh evidence: `target/evidence/shared-tooling-0314/`.
The local interpreter is Bash 5.2. The Shared-only commit `a6e2370` is pushed;
[native CI](https://github.com/dragginzgame/ic-auth/actions/runs/42363303068) used
its original released dependency selection. Both macOS architectures passed the
previously failing bootstrap step. Ubuntu passed all product/package stages but
failed the final release-command fixture: the updated checker correctly rejects
the CI Make target inherited through `CARGO_TARGET_DIR`.

The Auth caller now clears that variable at its ordinary-shell fixture boundary,
matching the existing batch fixture policy. The delivery guard is unchanged.
`bash scripts/release/test-tools.sh --commands` executes this actual caller without
the unrelated metadata/publication matrices. The original failure is reproduced;
the corrected caller passes with no selected group, Protocol and Engine while
inheriting CI Cargo and release settings. The replacement
[native run](https://github.com/dragginzgame/ic-auth/actions/runs/42364852620)
qualifies source `272cb99fe475afc116a0e38d0b331979f9e45201`, including the
separately committed Platform selection. All three complete suites pass: Linux,
Apple Silicon macOS and Intel macOS. Intel's final release-tools stage took 2295
seconds; its long active status was not a failure. No native host was substituted.
Evidence is under `target/delivery-0355/`. Subsequent hosted CI runs asynchronously;
the earlier synchronous Intel wait added delay and is not the delivery policy.

A fresh complete **30-stage Linux release gate** passed against captured source
`247e172758bbc3b1eb61211910f3ff2b233098da`, including all 93 client tests,
20 backend-conformance tests, 11 ordinary installed cases, MSRV/archives and the
actual release/recovery/publication callers. Gate:
`target/release-frozen/batch-IjY8zEHe/target/release-validation/engine-gate.RkMc4u/logs/validation.haMLoj`.
The final release-tools stage took 1063 seconds. Delivery, reconciliation and
registry-source logs are under `target/delivery-0355/`. No earlier gate was reused.

## Platform 0.23.6 qualification for Engine 0.35.5

The selected update advances Platform 0.23.5 to 0.23.6, crypto-common 0.1.6 to
0.1.7 and generic-array 0.14.9 to 0.14.7. Other selections remain unchanged,
including Storage 0.68.3, Testkit 0.90.0 and Runtime/macros/guard 0.19.5.
Warning-denied native consumer Clippy, 51 focused certificate/portable/chain-key
tests and all 11 ordinary installed certificate/session/ingress cases pass on Linux.
The opt-in Chromium case stays ignored. This adds no Auth crypto or protocol code.

Matched Wasm builds use the same Auth source, Rust 1.99.1, target flags and Cargo
release profile without post-processing. Total bytes change from 1,905,088 to
1,904,978 (-110); code-section payload size stays 1,468,844 bytes. This is no code
size or runtime-performance saving claim. Source/lock seals, build commands,
compiler context and both artifacts are retained at `target/delivery-0355/`.
Do not compare these totals to artifacts with different source or optimizers.

Storage 0.68.4 also arrived before qualification and is held separately on
`deferred/storage-0684-before-engine-0355` (`67d2359`). Its complete incoming lock
and Storage-only reapplication patch are under
`.git/release-state/deferred-engine-0355/`. It is not part of the selected release.

Subsequent updater writes are also preserved separately. The latest selects
Storage 0.68.5 and Wire/derive 0.17.9 on
`deferred/upstream-after-engine-0355` (`3f68f95`), with the complete incoming lock
and `after-publication.patch` under the same durable custody directory. Earlier
selections remain on their local custody branches. Only this dependency delta is
reapplied after clean release reconciliation, preserving Engine 0.35.5's local
versions. It remains unqualified and outside the published 0.35.5 graph; a Wire
adoption must regenerate and qualify browser contracts in its own batch.

## Completed Engine 0.35.4 delivery

Release `0b431149bbcbcd9a9f1c445ac382bff38ab9b3a6` and its verified receipt
`.git/publication-state/engine-0.35.4.json` remain retained alongside that batch.

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
release. It was preserved on `deferred/upstream-20261107-engine-0354`
(`2a6723a`) and under
`.git/release-state/deferred-engine-0354/incoming-after-capture/`.
Its later qualification and publication are now recorded in the 0.35.5 sections
above; this does not change either older captured graph.

## Clean backend helper delivery

The Engine 0.35.8 / Protocol 0.35.1 helper archive passed registry-consumer identity
checks and all 20 packaged tests from clean release source
`9ed4afbd74c27f56e00aed2dea18fcb195ffb674` (`dirty: false`). Receipt:
`target/release-frozen/batch-YyKgHPDM/target/portable-fixtures/backend-package.lRTaUR/delivery.json`.
Archive: that evidence directory's
`target/package/ic-auth-backend-conformance-0.35.8.crate`.
SHA-256: `2aa7dab3184ddd597d69fe5d6e211616f3fed4ac0e363214e25d0bb526ebb0be`.
The ten scenarios retain the existing `Backend` hooks, including 90-day renewal
and expanded authority checks. Canic must retain the archive and receipt in its
own custody and qualify its actual adapter; this supplies evidence only.
Earlier helper archives and receipts remain at
`target/release-frozen/batch-FqmjNnSq/target/portable-fixtures/backend-package.hc5VlO/`
(0.35.7),
`target/release-frozen/batch-ZU5ln9dY/target/portable-fixtures/backend-package.xsCAV0/`
(0.35.6),
`target/release-frozen/batch-IjY8zEHe/target/portable-fixtures/backend-package.FItPBs/`
(0.35.5) and `target/portable-fixtures/backend-package.mEpEDB/` (0.35.4).

The private browser source archive is also captured from clean release source
`9ed4afbd74c27f56e00aed2dea18fcb195ffb674`. Receipt and archive:
`target/release-frozen/batch-YyKgHPDM/target/browser-client-packages/package.qkhp1l/`.
The receipt records npm integrity
`sha512-0ZmJ9URapYXYMYkXScORyJkPuv1HFAponFn72K6s/o6QIV87L83mZOAKtfZFebJGLSPhYkvGIXT+l0pM+bLGRg==`.
This final archive is archive-only; client lifecycle and packaged-consumer checks
passed in both captured release gates before version delivery. No npm publication,
Chromium run or Canic adoption is claimed. Earlier archives and receipts remain at
`target/release-frozen/batch-FqmjNnSq/target/browser-client-packages/package.USHlpS/`
(0.35.7) and
`target/release-frozen/batch-ZU5ln9dY/target/browser-client-packages/package.rpLzwL/`
(0.35.6).

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

The refreshed [read-only Canic review](../canic-adoption.md#read-only-canic-review--2026-11-07)
inspects registry Engine 0.35.5 / Protocol 0.35.0, 179 passing native Auth tests,
51 actual stable-adapter tests, installed authority/capacity acceptance and the
real-II browser/cleanup records. Their Wire 0.17.9 and 0.17.10 phases keep their
own saved graphs. This supports local functional completion of the existing
composition, not a fresh full release gate or mainnet deployment.

Canic has since begun a separately accepted extraction into a real `canic-auth`
adapter with explicit mounting. Its current handoff records partial authorization
and restoration separation; package/mount, storage/memory/lifecycle composition
and installed qualification of that arrangement remain incomplete. The earlier
proofs do not qualify the new composition. No Auth implementation gap was found;
Canic's [integration tracker](https://github.com/dragginzgame/canic/issues/491)
still owns that work and final delivery. Canic remains read-only here.

The 0.35.5 backend archive and receipt above were rechecked against the retained
checksum and clean release source. Its original 20 packaged tests pass; they were
not rerun for this documentation update. Canic still consumes the 0.35.1 helper.
The refreshed helper README supplies the existing-driver adoption steps for all
ten scenarios, including 90-day renewal/reopen, without a new hook or production
format change. Small integrity evidence: `target/evidence/canic-handoff-20261107/`.

## Evidence retention

Before the cleanup incident, raw evidence and archives existed under earlier
`target/` paths recorded in Git history. Those files were deleted and cannot be
independently verified locally now. Committed source, published releases, Git-owned
publication receipts and earlier deferred branches remain retained. Newly recreated
paths above are current. Do not clean `target/` during delivery or erase journals.
Keep bulk artifacts out of `.git`; only source, small custody records and refs
belong there. No new Wasm performance or production-adoption claim is made.
