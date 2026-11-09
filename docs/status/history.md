# Historical Auth handoff notes

Preserved from the startup handoff before the Tooling 0.37.4 / SHA-2 update.
These dated observations retain their original qualification and delivery scope;
local/pending descriptions may be superseded. Resume from [the current handoff](current.md).

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

## Local Toko Tooling 0.37.2 follow-up

The current root pin and Cargo.lock select published **0.37.2**, registry SHA-256
`7660f9041b01073ba4038044983f18f3ac5a73aeebb3a9d054c70810357e20e3`.
Canonical `make toko-tooling-update` preserves the same 84 entrypoints and 174
owned files. Relative to 0.37.1, only the installed stack guide and catalog change:
dependency reports distinguish conditional, dev/build and qualification edges,
retain declaration evidence and identify unobserved generated fixture coverage.
No installed executable, Make guard, tool version or product source changed.

Cargo updated and fetched only Tooling relative to the incoming lock; pending
Storage and code cleanups remain intact. Setup/offline tools admission, snapshot/
member/pin checks, documentation links, inventory JSON and diff checks pass.
The unchanged caller checks below remain applicable; no Rust suite, product CI,
release or sibling change was needed. Evidence is in `target/tooling-0372/`.
Changes remain local, uncommitted and unpublished.

## Prior local Toko Tooling 0.37.1 adoption

Auth now selects published `toko-tooling = "=0.37.1"`, registry SHA-256
`47d50f4e65b643cd42c88f0fafe91f071044a94d1cfd0aa0998469b97b68f68b`.
The released installer authenticated crates.io payloads and the previous receipt,
preserved all 84 mapped entrypoints and installed 174 owned files. It retired
unchanged old owned filenames; snapshot-owned bytes were not hand-edited.
Cargo changed only Tooling relative to the incoming lock. All ten real packages
retain inherited dev-dependencies and cargo-machete exemptions; no carrier
package, selection metadata or independent tooling lock was introduced.

This is the 0.37 naming cut: use `TOKO-TOOLING.md`, `.toko-tooling.snapshot`,
`make toko-tooling-{update,install,check,status,preview}` and `TOKO_TOOLING_*`
overrides. The seven private LOC roles use `package.metadata.toko-tooling.loc`.
Auth's workflow evidence, caller fixtures and direct publication admission use
the new names. Its effectful Make recipes explicitly select the scoped execution
guard; help/catalog dry runs remain available. The jobserver regression records
every attempted Cargo dispatch before checking inherited descriptors, and checks
that unsafe Make modes refuse before effects.

Canonical setup/offline admission, snapshot/member/pin checks, LOC admission,
release-command dispatch and real publication admission, Make jobserver/routing,
Testkit/client callers, evidence retention and formatting-hook fixtures pass.
The Auth release-plan fixture also passes both groups, shared expansion,
required/excluded failures, drift rejection, retained evidence and CI dispatch
using substituted stage bodies.
Documentation links, shell syntax and diff checks pass. Test effects are
substituted at their documented boundaries; no product CI or release occurred.
Evidence, including the initial jobserver failures and successful retry, is under
`target/tooling-0371/`. Pending product source and earlier measurements are
unchanged; those Wasm measurements used Tooling 0.36.1, not this new selection.

Remaining old names are historical release/changelog/handoff evidence and issue
URLs, the preserved old scratch exclusion, intentional rejection fixtures, or
filenames retained by the canonical producer payload. In particular,
`release-validation.json` remains the exact published Engine 0.36.1 receipt;
it is not rewritten for local adoption. No compatibility aliases were added.
Changes remain local, uncommitted and unpublished; sibling repositories and
retained release custody were not changed.

## Local Storage 0.71 adoption

The root now requires published **Storage 0.71.0**, registry SHA-256
`f278c72c95e2819c20e6c6e59931b3a8891ccf38d1f258af28e599c828d38417`.
Cargo updated only that package and fetched the locked graph; Storage's normal
Wire edge is removed. Other dependency selections and the preceding local Canic
documentation edits are preserved. Product versions remain unchanged.

Storage replaces snapshot-directory backing with owned bucket allocation.
Auth's private qualification host already uses the supported default config;
no production adapter or public Auth library selects Storage. Use fresh private
fixture backing for this cut. Old snapshot headers are refused without rewriting
retained bytes, initialization or reset. Auth's `AUTHCLK1` payload, record codecs,
signed bytes and trust rules remain unchanged; same-version upgrades reopen the
checkpoint normally. No existing backing was retired, reset or migrated.

All 12 native storage cases pass, including occupied creation, bounds, corruption,
detached-lease custody, reopen and both retired Memory/Storage allocation headers.
Native/Wasm host Clippy, the release Wasm and the single installed certificate/
composition/checkpoint-upgrade case pass. The installed run initially could not
bind its local server under the sandbox; the successful retry used loopback
permission. Both logs and retained test state remain under `target/storage-071/`
and `target/portable-fixtures/qualification.{lM6uaA,WDmAPe}/`.
Dependency ownership, host Storage/Runtime singletons, public-library exclusions,
pin policy and changed-file formatting pass. No complete CI, broad installed
suite or Wasm size/performance comparison was run. Changes remain local,
uncommitted and unpublished; the 0.36.1 receipt below retains Storage 0.70.0.

## Local reference-store cleanup

Session and login replacement use `BTreeMap::insert`'s returned old value rather
than removing and reinserting the map entry. Login deletion/replacement share
expiry and pool cleanup; session deletion no longer needs a one-caller forwarding
helper. Public APIs, record codecs, verification checks, signed bytes, replay
retention, byte totals and commit/error ordering remain unchanged. Failed host
commits still publish no map/index changes.

The extended pool-transfer regression checks that an expired unknown record's
old expiry cannot prune its replacement, then reclaims the replacement exactly
at its new deadline while preserving another caller's denial. All 66 selected
session/revocation cases and 20 backend conformance cases pass. Focused native
and Wasm-host Clippy, changed-file formatting, the reference release Wasm and
one installed SDK session/logout/upgrade/signed-ingress case also pass.

Matched reference modules decrease from 1,831,907 total / 1,411,604 code bytes to
1,822,844 total / 1,404,357 code bytes: **9,063 total / 7,247 code bytes saved**.
Both sides use Storage 0.71 and the same sealed lock, Rust 1.99.1 and build recipe;
the baseline store source matches `e22cebd`. The
[observation](../canonical-allocation-observations.md#reference-store-replacement--2026-11-09)
records identities, measurement method and limits. Evidence is retained under
`target/store-cleanup/`. No runtime speedup, Canic footprint or complete CI pass
is inferred. The earlier Storage/adoption documentation work remains intact;
all pending changes are local, uncommitted and unpublished.

## Local signature-store lookup cleanup

Signature preparation reuses seed presence from its deadline lookup when choosing
the existing/new message tree. It no longer searches the outer tree again before
insertion. All validations, capacity refusals, retention deadlines, proof bytes,
certified roots and host publication ownership remain unchanged.

The capacity regression now covers both a new message under an existing seed and
a new seed, preserving the root and physical count on refusal. The 12 signature
integration cases and one retained-expiry unit case pass, as do focused native/
Wasm Clippy, formatting, the release Wasm and the single installed certificate/
composition/upgrade case. No complete CI or broad installed suite was run.

This saves another **360 total / 221 code bytes** against the retained store
cleanup above: 1,822,844 total / 1,404,357 code bytes becomes 1,822,484 total /
1,404,136 code bytes with the identical lock and build recipe. The
[measurement](../canonical-allocation-observations.md#signature-store-seed-lookup--2026-11-09)
records absolute sizes, identities and limits. A simultaneous borrowed-leaf trial
grew Wasm and was rejected; the qualification host's original source bytes are
restored. Evidence is under `target/signature-cleanup/`. No runtime speedup or
Canic size saving is claimed. Earlier pending work remains intact; all changes
are local, uncommitted and unpublished.

## Local borrowed-record cleanup

Private login predicates now borrow records; pool membership checks the existing
enum directly instead of extracting a binding. Session deletion no longer returns
a discarded boolean. Public APIs, serialized records, verification, denial
retention and commit/error ordering remain unchanged.

The extended saturation case checks that known-login denial retains its admitted
slot across logout retries, while unknown denial retains separate capacity;
expiry reclaims only the known slot at its exact deadline. All 66 session/
revocation cases and 20 backend conformance cases pass. Focused native and Wasm
Clippy, changed-file formatting, the release Wasm and one installed SDK session/
logout/upgrade/signed-ingress case also pass. No complete CI was run.

The matched reference module decreases from 1,822,484 total / 1,404,136 code bytes
to 1,820,985 total / 1,402,758 code bytes: **1,499 total / 1,378 code bytes saved**.
Both sides use the same sealed Storage 0.71 lock and build recipe. The
[measurement](../canonical-allocation-observations.md#borrowed-login-predicates--2026-11-09)
records identities and limits; evidence remains under `target/record-cleanup/`.
No runtime speedup or Canic footprint is inferred. Earlier pending changes remain
intact; all changes are local, uncommitted and unpublished.

The follow-up capacity helper now accepts only the requested pool, and logout
borrows the retained record. A new regression checks expired records attempting
both pool-transfer directions when the destination is full, preserving physical
counts and the other caller's denial. All 67 session/revocation cases, 20 backend
conformance cases, native/Wasm Clippy and one installed SDK session/logout/
upgrade/signed-ingress case pass. The matched module is 1,821,026 total /
1,402,675 code bytes: **+41 total / -83 code bytes** against the prior step.
This is a clarity cleanup with a small mixed size effect. The
[measurement](../canonical-allocation-observations.md#narrow-pool-capacity-inputs--2026-11-09)
and `target/pool-cleanup/` retain its exact inputs and evidence. The lock remains
unchanged. No full CI or publication was performed.

## Local pruning-candidate cleanup

Bounded session/replay pruning keeps expiry candidates borrowed until selecting
the staged deletion. Its two indexes, session-first deadline ties, work cap,
byte/count accounting and atomic callback ordering remain unchanged. A new
regression exercises equal deadlines with a one-record budget, failed commit,
identical retry and complete reclamation.

All 41 session cases and 20 backend conformance cases pass, as do focused native/
Wasm Clippy, formatting and the release Wasm. Every non-custom Wasm section is
byte-identical to the prior narrow-capacity module; only symbol names differ.
Totals are 1,821,027 module / 1,402,675 code bytes (+1 total / 0 code bytes).
The [observation](../canonical-allocation-observations.md#borrowed-pruning-candidates--2026-11-09)
records exact identities and `target/prune-cleanup/` retains evidence. A replay-
counter entry trial grew Wasm and was reverted before the final rebuild.
No new installed run, runtime speedup, full CI or publication is claimed.
The lock and earlier pending work remain unchanged; all edits are local.

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

## Canic adoption documentation and review — 2026-11-09

The [adoption guide](../canic-adoption.md#current-integration-cohort) now names
released Engine 0.36.1 / Protocol 0.36.0 and the qualified Wire 0.18 family.
It distinguishes the public Rust identity cut from unchanged method budgets,
encoded bytes, generated browser files, signed bytes and stored profiles.

Read-only review of Canic HEAD `3700c6e1bf78cb04a60dcfaf0f789d4dd992c9e0` plus
its dirty normal checkout found no new Auth API gap in the inspected paths.
Its root and both private consumer locks already select Engine/Protocol 0.36.0
and Wire 0.18. Its real adapter owns mounts, lifecycle and stable storage, and
the retained 123-test Auth run includes the complete actual-store conformance
entrypoint. The guide records the run's original lock and the newer observed
lock separately; this review reran no consumer tests.

Canic's 0.36.0 helper preserves its dirty archive receipt and actual-adapter
qualification. Its source/test bytes match Auth's Engine 0.36.1 release source;
this is not a new clean producer-archive qualification. Canic deployment remains
on hold, and no installed result on its current graph is claimed. Engine 0.36.1
adoption and final acceptance remain consumer-owned under open Canic #491.
Only Auth documentation changed; no sibling source, lock, issue or release was
modified. These documentation changes remain local and unpublished.

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

The last [read-only Canic review](../canic-adoption.md#read-only-canic-review--2026-11-09)
found no Auth implementation gap. Canic composition and installed adoption belong
to [Canic #491](https://github.com/dragginzgame/canic/issues/491). No sibling source,
release or deployment changed here; no standalone Auth service is deployed.
Preserve delivery/recovery and profiling evidence. Earlier raw files lost in the
cleanup incident cannot be independently verified. Keep build trees out of `.git`.


## Pre-0.37 delivery handoff — 2026-11-09

Historical snapshot from source `7aec8a1`. The later cleanup removed the old
`target/` trees referenced here; their raw evidence is no longer independently
readable. Current 0.37 delivery has fresh gate evidence.


### Published state

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

### Pending local work

The 0.37.0 drafts include Platform/Wire adoption, the Storage 0.71 update,
reference-store/pruning and signature-store cleanups, current Tooling/SHA-2
adoption and #130 host nonce qualification. Product versions remain unchanged.
Implementation is committed locally for release capture; no new publication
has occurred. Prior source/test edits and delivery custody are preserved.

Publication is authorized for Protocol 0.37.0 followed by Engine 0.37.0, only in
Auth. Testkit 0.97 is now published with matching Wire 0.19 / Platform 0.29;
its required Host 0.76.2 cohort also qualifies the earlier incoming product lock.
The native limits blocker is cleared. Batch `batch-Z6H_KgCz` passed 29/30
Protocol stages, including installed certification, upgrade, ingress and SDK
logout. Its last release-caller fixture failed on an obsolete dependency-pair
assumption. The fixture now accounts for the current catalog and shared
qualification metadata; a fresh source capture will rerun the required gates.
The failed batch/evidence are preserved. No new product version or upload occurred.

### Current Tooling and qualification placement

Both workspace catalogs select published `toko-tooling = "=0.38.2"`, SHA-256
`45c12cc2aa598d14e137dd7db50f12e943470b4ce351dfcaffeccde6821c1876`.
Canonical installation preserves 87 entrypoints and installs 176 owned files.
The four product packages remain under `crates/`; six private packages now live
in the independent `qualification/` workspace, with their own catalog/lock and
unchanged effective versions. Product release groups exclude those private
packages. Shared owns atomic two-lock preparation through the release policy
opt-in; Auth retains its source-bound receipt and delivery/recovery safeguards.

Focused checks pass for both group file plans and preparation, private-version
preservation, rollback after failed qualification, product resolution with an
incompatible qualification supplier, and manifest/lock input drift. Native fixture,
adversarial, metrics, Wasm lint/build and an installed SDK/session/logout/upgrade/
ingress case pass from the new workspace. Formatting-hook, jobserver and Testkit
caller checks exercise both roots. All three public archives build; the private
backend archive passes registry-consumer and packaged-scenario checks. CI/direct
publication refuse missing or drifted qualification tooling before effects.
Both-workspace formatting, pin, ownership, LOC-role and local-link checks pass.
The two-lock union preserves every incoming registry identity except the requested
Tooling update. All 106 original Rust files match their pre-move bytes. Evidence
is under `target/tooling-0382/`. The complete CI/release gate was not run.
No product API, signed bytes, Rust implementation or stored format changed in
this relocation. No release or sibling source changed.

At the tooling split, incoming Storage 0.71.1 introduced Runtime 0.21 beside
0.20.3. The maintainer preserved it and the failed singleton evidence. The
Platform/Wire batch below now resolves that conflict with one Runtime 0.21.

The preceding `sha2 = "0.11.1"` adoption retains its 105 Auth / 12 adversarial
cases and two installed SDK/session/certificate/upgrade/ingress passes under
Tooling 0.37.4. Its checksum, source and limits remain in
`target/tooling-0374-sha2/` and [the retained history](history.md).

The matched SHA-2 update grew reference Wasm from 1,821,027 total / 1,402,675
code-section bytes to 1,823,778 / 1,405,535 (+2,751 / +2,860 bytes) under unchanged
Rust 1.99.1 release settings and `cpubits="32"`, without an external optimizer.
See [the measured scope](../canonical-allocation-observations.md#direct-sha-2-011-adoption--2026-11-09).
No new Wasm measurement, runtime speedup or Canic footprint claim is made.

### Host entropy contract — #130

Published `ic-entropy` 0.1.0 (SHA-256
`d5a7386a4f8a63fc339bd18b773c49c078427a0273f66b708466852336045297`)
is selected only as a private backend-conformance dev-dependency. The
[host contract](../host-entropy.md) distinguishes portable nonces, fresh Canic
challenges, seeds and proofs; it preserves existing signed/replay/lifetime rules
and documents fail-closed acquisition, restart/restore and TEE/non-TEE limits.
Five conformance cases exercise real signing/session admission, exact retry,
replay, unavailable/short/exhausted providers, seeded resets and zero-nonce policy.
All five plus the 20 existing backend cases, focused Clippy, injected boundary
refusals, ownership/pin/snapshot/link/format checks pass. Selected library and host
Wasm normal/build graphs match the pre-change graphs exactly. Evidence is under
`target/issue-130/`; no new Wasm size or production entropy-source claim is made.
Canic #757 remains the adapter owner; no composed Canic acceptance is claimed.
Auth [#130](https://github.com/dragginzgame/ic-auth/issues/130) is closed as locally
implemented and qualified; the changes remain uncommitted and unpublished.

### Platform and Wire adoption

Both catalogs retain the compatible Platform 0.29 minimum. Both locks select
the incoming published **0.29.1**; its extra native Candid service exporter is
unselected by Auth. The frozen release gate will qualify this incoming patch.
Types 0.19.0, Wire 0.19.0, Runtime/macros/guard 0.21.0, Storage 0.71.1 and all
unrelated selections are preserved. Native Testkit 0.97 shares Platform 0.29; the actual Wasm host has exactly one Platform 0.29 /
Runtime 0.21 cohort. The changed upstream files are solely the owned Candid
decoder and its tests. Public Auth and the Wasm host do not select that codec;
Testkit selects it only in the native runner.

Fresh full locked metadata, declaration and selected graph checks, affected
product/private-core/Wasm Clippy, 68 crypto tests and 25 backend tests pass.
The release Wasm builds at 1,831,434 total / 1,411,686 code-section bytes
(+133 / +9 against the retained pre-update artifact, unchanged release settings).
Generated browser sources still match. Evidence and preserved inputs are under
`target/platform-adoption/`. No Auth implementation or product version changed.

The preceding Platform 0.28 / Wire 0.19 adoption retains 127 Auth, 15 Protocol,
26 private, 12 adversarial and 97 client test passes, browser interoperability
and three archive builds under `target/platform-wire-adoption/`; those broader
checks were not replayed for this facade-only update. Four Auth browser contracts
were unchanged; only Wire's shared runtime changed. The 0.37.0 draft ledgers
retain the prior breaking Rust identity cut, with no signed-byte/stored-format
migration. Agent boundaries copy validated Principal bytes explicitly.
The full CI/release gate was not run.

Testkit 0.97 removes the old Wire 0.18 / Platform 0.28 edges and requires Host
0.76.2. Metadata and graph checks admit the aligned cohort. Native Principal
identity conversions were removed where the supplier now takes the owned type;
Agent/delegation byte conversions remain. The CLI/server is explicitly prepared.
Earlier failure logs remain under `target/publication-readiness/`; the required
complete release gate will provide fresh installed and publication evidence.
Registry backend-bundle delivery requires the new Auth product cohort.

### Persistence and consumer boundaries

Storage 0.71 selects bucket backing only in the private qualification host.
Use fresh fixture backing; retired allocation headers refuse without clearing.
No production adapter, retained backing or `AUTHCLK1` checkpoint was migrated.
The [prior Storage qualification](history.md#local-storage-071-adoption) applies
to 0.71.0. The 0.71.1 host now passes the aligned Runtime graph check. Native
installed qualification will be recorded by the new complete release gate.

The browser journal is `ic-auth-login-cleanup-journal-v1`; the maintainer approved
retiring old journals on 2026-11-08. Unknown issuance, renewal sequence and expiry
remain durable. That approval never permits erasing Canic's protected safety state.
The [browser contract](../browser-client.md#durable-logout-and-guarded-identities)
and [clock-independent design](../design/portable-clock-audit.md) retain 90-day
login windows, short proofs and 30-minute receiver leases. Renewal cannot extend
the login window; lost protected state remains closed pending trusted recovery.

The last [read-only Canic review](../canic-adoption.md#read-only-canic-review--2026-11-09)
covered HEAD `3700c6e1bf78cb04a60dcfaf0f789d4dd992c9e0` plus dirty inputs and found
no Auth API gap. Canic's deployment hold, Engine 0.36.1 adoption and final installed
qualification remain consumer-owned under [Canic #491](https://github.com/dragginzgame/canic/issues/491).
This dependency batch did not re-review or change Canic. No standalone Auth service
is deployed here. GitHub issues remain the task tracker; the release preparation is recorded above.

### Retained delivery custody

Keep completed Engine `batch-QTi7wJc_`, its managed checkout, original receipts and
`.git/release-state/frozen/batch-QTi7wJc_/` plus
`.git/publication-state/engine-0.36.1.json`. Its 30-stage release proof and publication
artifacts remain under `target/release-frozen/batch-QTi7wJc_/`; independent readback
is under `target/delivery-0361/`. Do not resume failed `batch-8jjqjTO1` against the
delivered version. Protocol 0.36.0 and older delivery custody remain protected.
[Historical notes](history.md) preserve earlier sources, locations, checksums,
qualification limits and rejected trials. No artifacts were deleted while trimming
this startup summary. Earlier raw files lost in the cleanup incident still cannot
be independently verified. Keep build trees and bulk archives out of `.git`.
