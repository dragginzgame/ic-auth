# Current handoff — 2026-11-09

## Published state

**Engine 0.36.1** (`ic-auth`) and **Protocol 0.36.0**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
Both release commits are on `origin/main` and both annotated tags are pushed.
Standard batch reconciliation fast-forwarded the normal checkout to that source.

- Protocol commit: `3c30f425bbc8864526514db0dcfa9d810591a2cc`;
  tag `protocol-v0.36.0`, object `4d187d269d247d3bb9be0caccc361cab2089e5a4`.
- Engine commit: `f5c380f72119627fb33250fc449215a6615a5a5a`;
  tag `engine-v0.36.1`, object `49e9b243e2f20a9b27a2ed2d9dd411903adfed37`.

| Published package | Registry SHA-256 |
| --- | --- |
| `ic-auth-protocol-types` | `12ae95d99ec8d6efef08819986166914bbe201758da98fec84239fb0e3f5d8be` |
| `ic-auth-wire` | `45dbcd7c83b877806c7e64b8cf49033248fe738bb66c6c169679e65f3e7d6340` |
| `ic-auth` | `09f6a947a0ea8ddfadd1b95949dde176e1906865b278e0923eac3fcb875547a3` |

## 0.36.1 delivery and qualification

Engine-only batch **batch-QTi7wJc_** captured source
`842194ba28741fa05740ed9a4bb187529a16876e`. Its source-bound workspace plan
passed all **30 required stages**, including native/Wasm Clippy, isolated MSRV
and package payloads, 97 client tests, installed certification/upgrade/ingress,
input sealing and release/recovery/archive fixtures. No prior stage was reused.
The final release-tool fixture took 1,028 seconds. Chromium remains opt-in and
was not run; hosted CI completion is not claimed.

Gate: `target/release-frozen/batch-QTi7wJc_/target/release-validation/engine-gate.6ZxtbE/`,
logs `logs/validation.YffOH4/`. Publication artifacts remain in the candidate's
`target/publication-engine-0.36.1.0uRoeN/`. The completed schema-2 intent, Git
objects and original receipt remain under
`.git/release-state/frozen/batch-QTi7wJc_/`; an identical receipt is retained at
`.git/publication-state/engine-0.36.1.json`. Independent registry checksum and
remote-ref readback, delivery and standard reconciliation logs are under
`target/delivery-0361/`. The normal checkout fast-forwarded cleanly. Keep this
custody and the managed checkout; no batch retirement was performed.

The first attempt, **batch-8jjqjTO1**, passed 29 stages, then stopped in the
release fixture before version preparation, push or publication. Its pin-drift
case assumed Tooling 0.36.0. Commit `842194b` makes that test increment the actual
selected patch version; its focused `--tooling` check passes. A new captured
batch reran the complete gate. The original source `cc55f72`, failed gate
`engine-gate.shGw7v` and delivery log remain retained; do not resume that attempt
against the delivered version.

The delivered lock retains the 0.36.0 cohort below, updating only Metrics to
0.29.8 and Toko Tooling to 0.36.1. Protocol stays at 0.36.0. Cryptographic checks,
signed bytes, trust rules and stored profiles remain unchanged. No sibling source,
release or deployment changed; no standalone Auth service is deployed here.

## Tooling 0.36.1 adoption evidence

The working checkout now selects **Toko Tooling 0.36.1**, SHA-256
`56ab4b48eb5264ad9a98426a685804fd6cbd441bb370ae9d0046aa43c4aee6a3`.
The canonical updater installed authenticated crates.io bytes and retained all
recorded entrypoints. Only the owned LOC classifier and its documentation change;
product release hooks and the other snapshot payloads are unchanged. The root
exact pin, dependency-pin justification and active setup instructions agree.
Every real member retains its inherited dev-dependency and Machete exemption.

Snapshot/member admission, dependency-pin policy, Auth's `make cloc` caller,
local links and the focused published `test-cloc.sh --scopes-only` case pass.
The classifier bytes exercised by that case match the installed script. Its first
fixture was intentionally excluded beneath `target/`; the standard temporary
source location passes. That failed setup and the successful run are retained
under `target/tooling-next/`. Unrelated Cargo selections, including the incoming
Metrics 0.29.8, are preserved. The completed 0.36.0 delivery below keeps its
original Tooling 0.36.0 receipt. These focused adoption checks preceded the
complete 0.36.1 release gate above.

## Metrics and client/tooling cleanup evidence

Metrics **0.29.8** is now qualified for private admission diagnostics alongside
the delivered Tooling 0.36.1 adoption. The stored journal profile, unknown issuance,
fixed counter meanings and public Auth dependency boundaries are preserved.

Metadata-only journal reads validate the unchanged loaded record once. Credential
reads still validate stored metadata before attaching its separate frame, then
validate the complete view. Both modes retain post-callback validation and strict
transaction completion. A raw-storage corruption case verifies neither callback
runs and both retained records survive refusal.

Contract generation admits its Host file CLI once per bounded operation and uses
that path for both source seals, matching the existing backend/package callers.
Shared keys the selector by admitted build inputs; no cross-run cache or fallback
is added. The caller fixture checks one admission, two hashes, missing-tool refusal
before Cargo and unchanged inventory/source-drift refusals.

Five Metrics outcome/reset/overflow cases, 97 client tests and local client
archive checks pass. Native fixture compilation, Wasm Clippy and the rebuilt host
pass; the installed SDK session/logout/upgrade/signed-ingress case passes against
that host. Public all-feature Wasm graph admission excludes Metrics/native tools.
All five generated browser files and Cargo.lock remain unchanged through checks.
Evidence: `target/cleanup-0361/`. These changes reduce client/tooling work; no
runtime speedup or Wasm saving is inferred. These focused checks preceded the
complete 0.36.1 release gate above; the changes are now committed and delivered.

## 0.36.0 delivery and qualification

The previous Engine release remains recorded at commit
`9e13045373c1f33be71d31752dbc4614183a5d27`, tag `engine-v0.36.0`, object
`af44292ccd9b91afca7163ef2eeb4a7663fbf4a4`, with registry SHA-256
`964ad6751e9a95afa604e999cf0b6578c48a1c7ee23603182f1501511ac55858`.

Batch **batch-i8_wg3AC** captured `6c51fa0e29b5501eaee2db44cf08a8dd81830bf8`
and completed both groups in supplier-first order. Each source-bound workspace
plan passed all **30 required stages**, including native/Wasm Clippy, isolated
MSRV/package payloads, 96 client tests, installed certification/upgrade/ingress,
input sealing and release/recovery/archive fixtures. No prior stage was reused.
Protocol gate: `target/release-frozen/batch-i8_wg3AC/target/release-validation/protocol-gate.7ZAy0H/`;
Engine gate: the sibling `engine-gate.towfR2/`. Their release-tool fixture stages
took 1,052 and 1,037 seconds respectively. Chromium remains opt-in and was not
run. Hosted CI completion is not claimed.

Independent registry checksum and remote-ref readback, plus reconciliation logs,
are under `target/delivery-0360/`. Publication artifacts remain in the candidate's
`target/publication-protocol-0.36.0.BWH6n0/` and
`target/publication-engine-0.36.0.a46v3y/`. Durable batch/Git custody is under
`.git/release-state/frozen/batch-i8_wg3AC/`; verified receipt copies also remain at
`.git/publication-state/protocol-0.36.0.json` and `engine-0.36.0.json`.
Keep those records and the managed checkout; no retirement was performed.

The first attempt, **batch-TXwHoUcY**, stopped at dependency-pin admission before
version preparation, push or publication. Commit `6c51fa0` records the required
exact Tooling pin's compatibility justification in
`ci/dependency-pinning-exceptions.json`. Its failed gate and original source
`6c04007` remain retained; do not resume that attempt against the delivered version.

The delivered lock selects Build 0.81.0, Host 0.76.1, Testkit 0.95.0, Wire/derive
0.18.0, Storage 0.70.0, Platform 0.27.1, Types 0.18.3, Runtime/macros/guard 0.20.3,
Metrics 0.29.7 and Toko Tooling 0.36.0, with Rust 1.99.1. The temporary native
Wire 0.17 dependency is removed. Browser generated bytes, cryptographic checks,
signed bytes and stored profiles remain unchanged. The Rust Wire/Protocol type
identity change is documented in both release ledgers.

An external Metrics 0.29.8 lock update arrived during delivery. It is preserved
outside this release on local ref `deferred/metrics-0298-before-auth-0360`, commit
`69de328401956cb1778adadb05becba30a034692`. The completed release uses its tested
0.29.7 selection; incoming work is not evidence for that captured gate.
No sibling source, release or deployment changed. No standalone Auth service is
deployed here. Earlier qualification and publication records follow below.

## Previous 0.35.10 publication

**Engine 0.35.10** (`ic-auth`) and **Protocol 0.35.1**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
The Engine branch/tag push, registry checksum readback and standard batch
reconciliation completed. Their original receipts remain retained.

- Engine release commit: `d447e1ff45a7a79725119d8b3d917217daacc7e6`.
- Engine tag: `engine-v0.35.10`, object `e743f49b3a725b7d67943ef80a04cbd2b8d238b2`.
- Registry checksum: `ea1a19077449d4110e1f3d0309c5e5578e89ca1c3ed333366e69663466fbf54d`.
- Protocol is unchanged: commit `39b6e2605ad336c3aadff94bfd8af0dcb07cf8b3`,
  tag `protocol-v0.35.1`.

The [Engine changelog](../../releases/engine/CHANGELOG.md) owns shipped changes:
narrower internal scope checks, the session-only getter, root-owned Tooling,
Platform 0.27.0 and Storage 0.69.0. Public APIs, signed bytes, cryptographic policy
and current checkpoint formats are preserved. No reset or migration is introduced.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) remains closed.

## Previous delivery and qualification

Batch **batch-tnVpLRob** captured source
`8e1a0a8037870c6a153c1834ca0026a5da41b658` for Engine alone. Its schema-2 intent
is complete at index 1; reconciliation fast-forwarded the clean normal checkout.
No upload remains unresolved. This delivery was explicitly scoped to Auth.

- Durable intent, Git and recovery: `.git/release-state/frozen/batch-tnVpLRob/`.
- Registry receipt: `.git/publication-state/engine-0.35.10.json`; identical bytes
  remain in the batch's `repository.git/publication-state/`.
- Managed delivery checkout: `target/release-frozen/batch-tnVpLRob/`.
- Publication evidence: its `target/publication-engine-0.35.10.G9nKGy/`.
- Delivery, independent registry/ref readback and reconciliation: `target/delivery-03510/`.

All **30 required Linux stages** passed. Gate:
`target/release-frozen/batch-tnVpLRob/target/release-validation/engine-gate.UhakLb/`,
logs `logs/validation.oazCDy/`. It covers native/Wasm Clippy, package/MSRV and
boundary checks, 95 client cases, proof/session suites, 11 installed lifecycle
and ingress cases, archives and full release/recovery fixtures. Chromium remains
opt-in and ignored. The final fixture stage took 1,097 seconds. Hosted CI remains
asynchronous; no hosted completion is claimed.

Earlier attempts are retained: `batch-8Pq1jYH0` was interrupted after a focused
check found the missing jobserver-fixture substitutions; `batch-VzhWRpEn` passed
15 stages before the Testkit caller fixture refused at its old admission stub.
Both fixture gaps were corrected and qualified before the successful capture.
Neither attempt prepared a version, tag, push or upload. Keep their journals,
checkouts and logs; do not resume them against the delivered 0.35.10 identity.
The completed 0.35.9 batch `batch-rovUq5Bd`, its two contention interruptions and
receipt remain retained. Earlier detail is preserved in Git and `target/delivery-0359/`.

## Selected tooling, dependencies and footprint

These adoption notes record pre-delivery qualification. The reviewed changes
are now included in 0.36.0; their earlier local/pending descriptions are historical.

The latest local private-tooling adoption selects **Build 0.81.0**, **Host
FS/Process/Artifacts 0.76.1**, the separately pinned **Host file CLI 0.76.1** and
**Testkit 0.95.0** with PocketIC 16.1.0. Wire/derive remains at the latest
published 0.18.0. Testkit now shares that identity: the temporary
`ic-wire-testkit` dependency and budget projection are removed, and the runner
passes canonical method budgets directly. Its managed instance uses Testkit's
concrete `PocketIcInstance`; awaited deletion and server custody are retained.
Host HTTP 0.76.1 is added only through native Testkit. The application Wasm
normal/build graph is unchanged and excludes Build, Host and Testkit. Other
locked package versions are preserved; no public Auth API, signed bytes or
stored profile changes. Evidence is under `target/upstream-20261109/`.

Platform 0.27.1, crypto-bigint 0.7.11, syn 3.0.10, itoa 1.0.19 and Host 0.76.1
were already in the incoming lock before this batch. They are retained separately
from the targeted Build/Testkit changes in `incoming-selections.json`. The host
is rebuilt against this current cohort rather than reusing the earlier Wasm.
Its reference module is **1,870,456 uncompressed bytes**, with a 1,438,017-byte
code-section payload; SHA-256
`53abb17ffa5ebdcacf2c76705f82033e198fa93d837e7e0eb72668a40961aaea`.
No size change is attributed to native Build, Host or Testkit from that different
cohort. The previous matched Wire/Storage measurements remain separate below.

Native/Wasm Clippy, five tooling tests, ten frozen ECDSA cases and 96 client
tests pass. All five regenerated browser files are unchanged. The three selected
CLIs admit offline; real metadata rendering/admission passes for both release
groups, as does the Testkit setup/CI caller fixture. Installed SDK session/logout,
upgrade/signed ingress and live-authority certification pass against the rebuilt
module (`matched-*` logs); ingress deadline/cancellation also passes. The first
tooling run stopped at the unprepared newly pinned Host CLI; that case passes
after explicit setup. Earlier installed runs used the prior reference module
and are retained separately. Cargo.lock stayed unchanged through final checks.
Snapshot admission, local links, formatting and the diff check pass. No full CI,
Chromium or release gate ran. Changes remain local, uncommitted and unpublished.

The earlier private-tooling adoption selected **Build 0.80.0** and direct **Host
FS/Process 0.76.0**, including the separately pinned file CLI, plus **Testkit
0.94.0**. Native Platform 0.27 and Host 0.76 providers now have single identities;
the older Testkit-only copies are removed. Other package identities and root
Tooling selection are preserved; public normal/build graphs exclude native tools.
Affected native Clippy, five tooling tests, generated-contract freshness and
both groups' real metadata rendering/admission pass. Both new CLIs were explicitly
prepared and admitted. Evidence: `target/build-host-adoption/`. Changes remain
uncommitted; package versions remain 0.35.10 and 0.35.1. The subsequent Wire
adoption below advances the pending combined-delivery notes to 0.36.0.
Testkit's locked CLI/server setup and offline admission pass with PocketIC 16.1.0.
Native runner Clippy, its setup/CI caller fixture and three installed cases pass:
certification/upgrade, real ingress delegation across upgrade and deadline/abort.
No Rust API migration was needed. Evidence: `target/testkit-adoption/`; it retains
an initial evidence-directory collision and the successful isolated ingress rerun.
No full gate or publication ran for this private adoption.

The working checkout selects **toko-tooling 0.36.0** through one exact root
workspace dependency, inherited as a dev-dependency by all ten real packages.
Each package exempts that source-bundle dependency from cargo-machete. Cargo.lock
and the installed 173-file snapshot agree on SHA-256
`de0dd05b6437061622a76942baa25d40d2181497e7ba66d9fd809132a3082936`.
The old metadata selection and dedicated tooling lock are removed. The producer
installer authenticated registry payloads and preserved all recorded entrypoints;
no selection-only package or artificial Rust import is introduced. Real Auth
generators, benchmarks and qualification packages are retained. Cargo introduced only the tooling
package during this migration; incoming dependency updates are preserved separately. Evidence: `target/tooling-036-adoption/`.
The published 0.35.10 receipt and historical changelogs retain their truthful old
selection. No product publication is included in this adoption.

The initial Wire adoption selected published **Wire/derive 0.18.0** and matching
**Storage 0.70.0**. Platform 0.27.0, Types 0.18.3,
Metrics 0.29.7 and the private-tool versions above are preserved. Published
Testkit 0.94.0 then used Wire 0.17: the native runner projected every canonical
request/response budget through a private `ic-wire-testkit` dev-dependency,
now removed by the Testkit 0.95 adoption above.
The Wasm host has one Wire 0.18 identity and one Runtime executor/generator/guard;
public normal/build graphs exclude Testkit and that older Wire boundary.
All five regenerated browser contract/runtime files are byte-for-byte unchanged.
Method IDs, budgets, signed bytes, cryptographic policy and stored profiles are
preserved. The exposed Wire Rust type identity changes, so the next combined
Protocol/Engine delivery has pending 0.36.0 notes; package versions are unchanged.

Locked fetch, affected native/Wasm Clippy, protocol/tooling/codec/session/proof
tests, all three public package archive builds and 96 client tests pass.
Three installed cases pass for typed refusals/retry, SDK admission/logout across
receivers/devices and upgrade/signed ingress, and live-authority certification
after upgrade. Snapshot admission and the single-owner Wasm graph check pass.
Cargo.lock remained unchanged through those checks. Evidence, registry selections
and source-bound artifacts are under `target/wire-adoption/`.
The matched Wire/Storage update adds **997 total / 298 code bytes** to the
reference Wasm under unchanged build settings; [the measurement](../canonical-allocation-observations.md#wire-018-and-storage-070-adoption--2026-11-08)
records both artifacts and limits. No runtime speedup or production Canic size is
inferred. This adoption is local, uncommitted and unpublished; full CI and
Chromium were not run.

An external lock update then selected Runtime/macros/guard **0.20.3**. Its
upstream change adopts Tooling 0.36 in dev-dependencies while preserving normal
dependencies, generated code and executor behavior. The incoming lock is retained
separately as `incoming-runtime.Cargo.lock`. Native/Wasm Clippy and all three
installed cases pass again with that lock unchanged; artifacts and logs remain
under the same evidence directory. At that selection the reference module was
1,869,903 bytes, with the same 1,437,987-byte code section as the measured Wire/
Storage adoption. The linked matched table uses Runtime 0.20.2 on both sides;
the later Runtime selection is not attributed to Wire or mixed into that pair.

The published release selects Platform 0.27.0, Types 0.18.2, Runtime/macros/guard 0.20.2,
Testkit 0.93.2, Storage 0.69.0, Wire/derive 0.17.14, Build 0.79.1, Host 0.75.1
and Metrics 0.29.6. Testkit retains Platform 0.26.1 only in native helpers; the
Wasm host keeps one facade and one Runtime executor/generator/guard. Rust remains
1.99.1. All five generated browser files retain their original bytes.
Focused adoption evidence: `target/platform-027-adoption/` and
`target/storage-069-adoption/`. No packed-ID codec is selected.

The gate's reference qualification Wasm, before Engine version preparation, is
**1,869,560 uncompressed bytes**, SHA-256
`f6e1584d38c86dbe8fdf4677b561f6ce89770c175201185c3d63bcb5ad9b8a8a`.
Its artifact identity is retained in `target/delivery-03510/gate-wasm.sha256`.
This fixture measurement belongs to the captured dependency cohort; no throughput
or matched implementation saving is inferred from it.

Earlier matched scope cleanup used Platform 0.26.1 and its own fixed lock/compiler:
1,798 total / 992 code bytes saved, evidence `target/evidence/scope-surface-20261108/`.
[Other allocation observations](../canonical-allocation-observations.md#signature-retention-and-shared-grants--2026-11-08)
retain their original source/dependency limits and evidence. No speedup is claimed.

## Tooling and package layout — #129

[#129](https://github.com/dragginzgame/ic-auth/issues/129) is closed as locally
implemented and qualified; its closure note records the delivery state.

Published Toko Tooling 0.36.0 was adopted and checked before relocating packages.
All seven private packages now live under `crates/<package-name>/`. Names,
versions, feature declarations, dependencies, targets and publication roles match
the pre-move Cargo inventory. The move itself left Cargo.lock bytes unchanged.
Workspace paths, generator/profile inputs, native browser paths, archive callers,
release ownership and active guides follow the new locations. The obsolete
non-fleet `apps/` reservation is removed; no compatibility directory, symlink,
selection-only package or sibling change was introduced.

Incoming Storage 0.69.1 was already selected at the start. Metrics 0.29.7 and
Types 0.18.3 arrived externally during qualification and are preserved. The final
lock stayed unchanged through the current Types 0.18.3 checks. Focused protocol/
proof tests, native/Wasm Clippy and the rebuilt host pass, as do three installed
SDK/session/certification cases covering the relocated Node callers, logout,
upgrade and real ingress. Backend archive/consumer/scenario qualification also
passes with the same registry identities. Its scenarios use a separate generated
workspace so test-only dependencies resolve without modifying the extracted
package or adding it to the real consumer's workspace.

Metadata/membership comparison, formatting, source boundaries, generated-contract
freshness, client/jobserver callers, release-plan ownership (including relocated
host/runner cases), CI input sealing and publication admission pass. Common CI/
release entrypoints and the direct publisher retain tooling admission before
effects. Evidence is under `target/tooling-036-adoption/` and `target/layout-129/`;
`incoming-types-*` records the final cohort. Earlier failures remain retained with
their fixes/reruns. Full CI, the complete frozen archive matrix and Chromium were
not run. Changes remain uncommitted, unpushed and unpublished.

Remaining old selection/path mentions are deliberate retired-input rejection
fixtures, the dated Canic source review, shipped changelogs/publication evidence
and immutable producer-owned fleet snapshots. They are not active Auth callers;
no snapshot-owned file or historical receipt was hand-edited.

## Local complexity cleanup

The current batch keeps grant-validation evidence tied to immutable input,
separates metadata/credential journal transactions, shares deterministic test
construction, narrows permissive-feature qualification and gives release fixtures
explicit scenario dispatch with one admission substitute. The browser unknown
state is typed while preserving uncertainty without a recoverable sequence.
Public Rust APIs, signed bytes, crypto/trust policy and stored profiles remain
unchanged. The original session/replay expiry indexes are retained: their proposed
replacement increased the reference module by 11,909 total / 10,001 code bytes.
State-specific login retention was not introduced because preserving its codec
would add a parallel representation.

The retained grant change saves **568 total / 454 code bytes** in matched builds;
[the comparison](../canonical-allocation-observations.md#grant-evidence-and-rejected-expiry-queue--2026-11-08)
records absolute sizes, source/dependency limits and the rejected experiment.
Evidence is under `target/complexity-cleanup/`. Focused native Clippy, 147 regular
Rust cases, 12 permissive-feature cases, 96 client cases, the reference Wasm build,
jobserver/Testkit callers and release `--commands`, `--tooling` and `--validation`
cases pass. The final release/recovery run is
`release-validation-stable.log`, using an unchanged driver checksum; the earlier
run's final shell-read failure is retained separately. Snapshot admission, local
links and the diff check pass. No complete CI gate or native frozen-batch archive
matrix was run for this cleanup. Changes remain local and pending; no product
release/version preparation or publication is part of this batch.

## Storage and consumer boundaries

The private client uses `ic-auth-login-cleanup-journal-v1`. The maintainer confirmed
no retained journals on 2026-11-08; retired `ic-auth-local-logout-journal-v1` records
are refused without clearing. Credential expiry, renewal sequences and unknown
issuance remain durable. See [storage and retirement](../browser-client.md#durable-logout-and-guarded-identities).
That approval does not authorize erasing Canic or other protected safety state.

The [Canic contract](../canic-adoption.md) and
[clock-independent design](../design/portable-clock-audit.md) retain 90-day login
windows, short proofs, 30-minute receiver leases, bounded unknown denial and
lookup-only uncertain issuance. Renewal never extends the original login window;
ordinary authorization stays local. Lost protected safety state stays closed
pending trusted recovery. Complete durable restore plus fresh authority is supported.

The last [read-only Canic review](../canic-adoption.md#read-only-canic-review--2026-11-07)
found no Auth implementation gap. Canic composition and installed adoption belong
to [Canic #491](https://github.com/dragginzgame/canic/issues/491). No sibling source,
release or deployment changed here; no standalone Auth service is deployed.
Preserve delivery/recovery and profiling evidence. Earlier raw files lost in the
cleanup incident cannot be independently verified. Keep build trees out of `.git`.
