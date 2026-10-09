# Current handoff — 2026-10-09

The maintainer accepted the larger Canic authentication extraction, selected the
name `ic-auth`, and confirmed `/home/adam/projects/ic-auth` as the local repository.

The current committed release is `01f765b2b0b1c403feb3df3e0a7210bcaa6c26c9`
(`0.1.10`) on `main`, with annotated tag `v0.1.10`; both remote identities were
verified during this batch. Both libraries' `0.1.10` registry checksums match
the retained publication intent for that exact source/tag. The maintainer
executed the release and publication. The original bootstrap was
`7a03102e52b7a400588530997a9f7fb91c83de9d` (`0.1.0`).
The public remote [dragginzgame/ic-auth](https://github.com/dragginzgame/ic-auth)
was created at the maintainer's request and is configured as `origin` using
`https://github.com/dragginzgame/ic-auth.git`.
The accepted [design](../design/extraction.md) covers signatures, application
tokens and local sessions, plus an independent Solana wallet-auth service.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) remains the existing
extraction and Canic adoption tracker. New IC Auth-specific issues belong in
the public repository's issue tracker.
The [bootstrap evidence comment](https://github.com/dragginzgame/canic/issues/491#issuecomment-6055817375)
records this batch without closing the larger extraction.

The virtual Rust workspace contains `ic-auth-protocol-types`, `ic-auth` and the
unpublished native application in `apps/tooling/`, with one lockfile, root-owned
dependency selections and current manifest version `0.1.10`.
The passive token/proof contracts and canonical encoding are adapted from the
clean Canic source at `e286b3fd98460c98670336853f80658a920966e0`; see the
[source review](../design/canic-source-review.md) for exact ownership and API
adaptations. Both packages now allow crates.io publication of their implemented
contracts/encoding, with inherited repository/README metadata and matching MIT
notices in the package payloads. Released `0.1.6` includes optional complete token
and IC signature verification, session/replay machinery and bounded signature
preparation/retrieval. Released `0.1.7` includes the internal canister/runner
packages under `apps/qualification/` and their IC Testkit qualification.
There is no stable canister adapter or wallet endpoint yet. The private browser
client now implements injected token lifecycle machinery; real issuer adapter
qualification, npm publication and consumer adoption remain pending.
Canic adoption has not
occurred; no sibling repository was modified.

## Current 0.1.11 qualification

The maintainer requested review of new Host Tooling and continuation of 0.1.11.
The incoming lock already selected all four Host packages at 0.8.5, replacing
the committed 0.8.4 selections. That change is preserved. Live registry metadata,
cached package archive digests and selected lock checksums agree for every package;
all are unyanked and declare Rust 1.88.0. Published package source and remote
main/tag resolve to `1cad3253096b6eb67be5187209e7fb606593c501`.
The clean Host sibling was read-only. The 0.8.4-to-0.8.5 crate sources are identical;
the release adopts the IC installer reuse fix already selected here. No further
native API substitution is needed: the consumer already delegates bounded
no-follow reads, SHA-256 and durable private publication to Host.

Shared Tooling subsequently advanced to committed
`635a39a9dd5f8d021fa9c9196b591e00521a7e02` (0.1.32), verified against remote main.
A new clean isolated clone supplied the canonical refresh of the same 80 files;
the sibling's further dirty dashboard/setup work was preserved and excluded.
The selected additions document PocketIC's intended Testkit ownership and correct
Cargo-install qualification guidance. No new workflow or installer was selected.
The consumer workflow has no concurrency cancellation, so it already preserves
each push's native jobs. The complete IC installer, consumer pins, receipts and
Testkit 0.25.3/PocketIC 16.1.0 pairing remain in place until the published replacement
setup/check contract is qualified. No direct PocketIC dependency was added.

Two additional storage rejection tests prove that unavailable strict durability
does not admit a relaxed fallback or alter retained intent, and that exhausting
nat64 revisions cannot wrap, clear an existing intent or allocate another key.
All 35 Node tests and both Rust/TypeScript Candid directions pass on Host 0.8.5.
The previous nine native Chromium observations remain evidence for the unchanged
storage implementation; the new injected cases are Node qualification.
Four native file tests, strict tooling/runner Clippy and actual Rust 1.88.0
tooling/runner checks pass. Both real IC Testkit scenarios pass with Host 0.8.5,
covering certification/composed roots, protected upgrades and delegated ingress.
Snapshot integrity, dependency declarations/inheritance, local links, locked
metadata, library graph boundaries and formatting pass. The release/recovery and
publication reconciliation fixture passes with the final selected snapshot and
Host lock, using local/mocked effects only. Full CI and native macOS remain pending.

Current evidence is retained under `target/continuation-0.1.11/host-0.8.5/`.
Rust manifests remain 0.1.10 and the compatible draft remains 0.1.11. No existing
function, method or type was removed. No commit, push, release, publication or
sibling source change was performed. Real issuer adoption, wallet login and native
macOS qualification remain separate from this local batch.

## Earlier preparation of the current 0.1.11 batch

The following records the earlier 0.1.31 refresh and initial storage qualification;
the current source selection and additional tests are recorded above.

The maintainer pushed/released 0.1.10 and authorized continued local development
and current Shared Tooling adoption. Remote main/tag resolve to the exact release
above. Both unyanked 0.1.10 registry checksums match retained publication intent:
`51162f12aeb7cf52a2249475d8c423e1dc9e33e18e4727ce53687f3696975c04`
(protocol types) and
`f962f1ad928c5aabff78891a09c883847503661d6a41c4b29c520b2e16b3cc98`
(auth). Rust manifest/lock versions remain 0.1.10. The next compatible draft is
0.1.11: opt-in private storage and installer reuse are additive, with no existing
Rust API or signed-byte change. No commit, push, release or publication was made.

Shared Tooling remote main and committed sibling HEAD are
`9af82393c620e486578febed74a648523725c234` (0.1.31). The sibling has further dirty
Cargo-install qualification changes; they were preserved and excluded. A clean
isolated clone at that exact commit supplied the canonical 80-file refresh.
The selected baseline adds fleet/report ownership guidance and the IC installer
compares complete validated records across all hosts instead of raw pin bytes.
No fleet dashboard, new Cargo installer or central qualification workflow was
added to consumer CI. The existing tool/version/checksum selections are unchanged.

The old installer rejects the same selected records with reversed rows and new
comments. The adopted installer passes offline against the existing real bundle,
with exact stdout, unchanged active link and unchanged installed-pin digest. The
clean upstream IC installer fixture also passes with mocked downloads/archives;
that evidence is distinct from real bundle reuse and native macOS qualification.
The current GitHub description still matches the independent authentication
library scope; no description write was necessary.

The private browser package now offers opt-in `IndexedDbTokenStore`, with an
explicit factory, dedicated owner-selected database and frozen bounded profile.
It atomically reserves capacity and revisions with the record CAS, waits for
strict transaction commit, retains tombstones/uncertain intent, validates exact
schema and refuses malformed state without deleting it. Closing/version-change
only closes the connection. No storage migration, implicit reset or alternate
reader was introduced. `fake-indexeddb 6.2.5` is test-only, prepared explicitly with
lifecycle scripts disabled; no existing npm selections or Rust dependencies moved.
See [browser storage](../browser-client.md) for the contract and owner obligations.

Native Linux Chromium 153.0.8010.12 passes the actual IndexedDB fixture, including
independent-connection CAS, detached bytes, reopening/exact bigint intent,
tombstone/capacity admission, profile/corrupt-state refusal and late transaction
abort. Node tests additionally exercise the client recovering a lost prepare
reply after reopening and transaction timeout/rollback. These are storage/lifecycle
checks, not a production issuer adapter, browser proof verifier, live consumer
adoption, device-crash recovery or native macOS/browser-engine qualification.
The standard DOM-dump attempt stalled, and a virtual-time attempt observed an
incomplete page; retained artifacts distinguish them from the passing real-time
CDP observation. The isolated test browser and local server were closed.

Focused checks pass: 33 Node lifecycle/codec/storage/example tests, both Candid
directions, and nine native Chromium storage observations against the final
compiled source. Snapshot integrity, tracked npm declarations/Cargo inheritance,
local links, formatting, locked metadata, auth graph boundaries and ShellCheck
pass. The local/mocked release/publication fixture also passes with the adopted
snapshot and complete working client tree. Source/input digests and the observed
local headless-browser binary digest use the existing Host utility. Existing
Rust sources, dependency selections and declared compiler floors are unchanged;
no broad local CI or unrelated Rust feature suites were run. Remote CI for the
uncommitted batch and native macOS/browser-engine qualification remain pending.

Evidence is retained under `target/continuation-0.1.11/`, with native fixture source,
results, selected tool/source identities and failed attempts. No existing function,
method or type was removed. Neither Shared Tooling nor Canic/Toko source was edited.

## Earlier browser client batch (released in 0.1.10)

The following records preparation before the maintainer's 0.1.10 release. Its
pending-version and untracked-file wording is historical.

The maintainer pushed/released 0.1.9, then authorized continuation on the browser
client in [IC Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
Live 0.1.9 registry checksums match retained publication intent:
`de44ab5b877f3df89674d273b3e38b2e398e25cc60ab5bf3caa61d8cc436b7ff`
(protocol types) and
`fa9b48d98aa8232eb264eb94f2ff0238a922a786e314884fc3e52e1cd978f101`
(auth). The Rust manifest remains 0.1.9. The next compatible draft is 0.1.10:
the private client/dev tooling is additive and existing Rust APIs and signed
bytes remain unchanged. No release, push or publication was performed here.

The private npm root uses Node 24.21.0/npm 12.2.0 and locked ICP SDK core 6.1.0,
bindgen 0.4.1 and TypeScript 7.0.2. Runtime contracts come from the Rust DTO owner;
no client service endpoints or hand-written Candid DTO copies were introduced.
A Rust development-only printer feature exports the contracts without changing
normal protocol dependency selections. Native Host packages remain outside the
browser and public protocol graphs. See [browser client](../browser-client.md)
for uncertainty, generation, cache, storage and consumer boundaries.

Explicit npm preparation used reviewed registry routing with lifecycle scripts
disabled. Subsequent checks were offline and preserved the lock. Local source
fixtures exercise login/renewal races, captured-generation invalidation,
unknown prepare/retrieval replies, typed TTL exhaustion, expiry/poll bounds,
cache isolation, material binding, storage admission and CAS behavior. The
Internet Identity example compiles with the actual SDK and checks authenticated
agent identity plus logout during pending login. These are transport/lifecycle
fixtures, not a live issuer adapter or browser cryptographic qualification.

The read-only Toko source review selected
`44d4e2c6d41e3575b2503e79ffa369129ff3ecf2` and preserved its dirty Cargo lock.
The adoption recipe names replacement boundaries but leaves user/shard/project
semantics in Toko and issuer authority in Canic. A real adapter must expose
qualified exact-ID reconciliation before old token helpers are removed.
Issue #3 remains open for that consumer work. Neither sibling was edited.

Focused validation passed: 23 Node lifecycle/codec/example tests and both Candid
directions; protocol type tests and the exporter on development Rust 1.99.0 and
minimum Rust 1.88.0; strict protocol Clippy; auth dependency boundaries, locked
metadata, formatting, ShellCheck, Actionlint, snapshot integrity and local links.
The release/publication fixture includes the client source/lock without build
outputs and passes its local/mocked release and uncertainty checks. These fixture
Make override warnings are deliberate substitute gate recipes, not production
Makefile duplicates. Evidence is retained under `target/browser-client/`, with
failed first attempts alongside the final logs. Generated-contract directories
record exact input hashes using the existing Host utility and tool versions.

The canonical dependency declaration check passes in an isolated source fixture
with the new npm manifest/lock tracked; the real index was unchanged. The real
checkout's npm check correctly requires those currently new files to be tracked
at commit. The Rust lock is unchanged. No broad local CI or native macOS run is
claimed; configured CI now prepares and checks the client on all three hosts.
No existing function, method or type was removed in this additive batch.

## Earlier Host 0.8.3 review and wrapper cleanup (released in 0.1.9)

The following records preparation against 0.1.8, before the maintainer's 0.1.9
release. Its pending-version wording is historical.

All four Host packages were already selected at 0.8.3 in the incoming dirty
lockfile. That update and the prior Shared Tooling batch were preserved.
Live registry metadata identifies 0.8.3 as the latest unyanked version of each;
all four checksums match the selected lock. Published package source and observed
remote main are `67d031222073f23ad437b45053156e229f86a016`. The clean sibling was
read-only. Evidence is retained in `target/host-0.8.3-review/`.

The committed 0.8.2-to-0.8.3 changes add live output observation to owned-child
communication and expose regular lock-file admission without acquiring a lock.
This consumer has no private pipe-reader loop or descriptor-opening lock
implementation to retire. IC Testkit owns server startup and retained output;
Shared Tooling owns release process orchestration. Directory locks and registry
uncertainty policy remain with the shell release/publication owner. The current
native CLI already delegates bounded no-follow reads, digest computation and
durable private publication to Host. Its explicit permissions and create-only
semantics are retained; the link-following `hash_file` API is not an equivalent
substitute. No new dependency, adapter, function or type was needed or removed.

A concrete wrapper defect was reproduced: inherited `CDPATH=.` emitted the
checkout path before the digest (`cdpath-cached-before.stdout`) and before the
NUL-delimited release metadata (`cdpath-metadata-before.nul`). Consumer wrapper,
release metadata, publication and fixture entrypoints now resolve an absolute
script path and use physical `cd`, preserving exact output. The release fixture
checks exact metadata bytes and keeps `CDPATH` set throughout its local release
and mocked publication flows. No authentication API or signed bytes changed.

An initial offline attempt lacked the selected Host cache entries and failed
without resolution changes (`cdpath-before.stderr`). Explicit `cargo fetch
--locked` prepared those four entries (`cache-preparation.log`); subsequent Rust
checks stayed offline. Four native file tests pass (`native-file-tests.log`),
including bounded hashing, no-follow admission, private replacement and saved
intent preservation. Exact wrapper stdout and release metadata pass after the
fix (`cdpath-after.stdout`, `cdpath-metadata-after.nul`). ShellCheck passes.

Both real IC Testkit scenarios pass with the selected 0.25.2 client and 16.1.0
server on Host 0.8.3 (`final-testkit-qualification.log`, retained state
`target/portable-fixtures/qualification.3kaA17`). The first run was refused by
the sandbox at loopback binding before either scenario could execute; its log
and `qualification.RK3eWS` state are retained separately. The successful run used
the same verified server outside that restriction. This exercises certification,
composed roots, protected upgrades and actual delegated ingress, not wallet login.

Focused native tooling/runner checks pass on actual Rust 1.88.0
(`native-msrv.log`, compiler identities in `compiler-identities.log`), as does
strict native Clippy (`native-clippy.log`). This checks the changed Host graph;
the earlier isolated public-package/native/Wasm minimum qualification remains
its own evidence. Consumer release/recovery/publication fixtures pass with
inherited `CDPATH` (`consumer-release-fixtures.log`); their local/mocked effects
are not live release or upload evidence. Snapshot, pinning/inheritance, links,
metadata, library boundaries and the diff pass (`final-governance.log`).
Final source/lock/snapshot/Wasm identities and the working diff are retained here.

The compatible pending batch remains 0.1.9; manifests remain 0.1.8.
No commit, push, release, publication or sibling change was performed.

## Earlier Shared Tooling 0.1.29 batch (released in 0.1.9)

The following records preparation before the maintainer's 0.1.9 release; its
pending-version wording is historical.

Observed remote Shared Tooling main is
`1a54fb625d6e47efa64c4384808ecbc87be84e7e` (0.1.29). A clean isolated checkout
exported its committed bytes; the sibling's untracked Cargo-install helper was
excluded and the sibling remained read-only. The canonical refresh passed for
80 files, explicitly adding `scripts/ci/check-release-source.sh` to the existing
selection. Evidence is retained in `target/baseline-0.1.29/`.

The local release adapter now uses that helper. Ordinary initial admission
allows only pending `CHANGELOG.md` notes; interrupted preparation retains the
four exact metadata paths and receipt-bound byte checks. Refusals name all
staged, unstaged and untracked paths, quote unusual names and distinguish failed
Git observations from successfully observed dirty source. The runner identifies
an initial preflight refusal before this attempt starts validation/preparation.
No files or index bytes are repaired merely to pass admission.

The refreshed baseline restricts standing issue-writing permission to verified
`dragginzgame/*` destinations, including the maintenance prompt. Its optional npm
checks remain unselected: this repository has no frontend publication root.
The optional sibling issue dashboard was not added to the snapshot. No task,
maintenance agent or schedule was activated.

The shared server defaults now select the same reviewed 16.1.0 archives as
`ci/ic-auth-tools.tsv`. The consumer-owned selection remains explicit because
its client/server pairing is qualified with IC Testkit. No Cargo dependency,
compiler floor, authentication API, signed bytes or runtime pairing changed.

Both unyanked 0.1.8 registry checksums match the retained publication receipt:
protocol `ef8d1f627a45eefbf5b3e831a61c4ee16b283976ae9bf155686fba8b64722300`,
auth `6e239af75a1eca605a0b9b0da602466de2f1f7685637213ad236d68de1bd1d2c`.
The remote annotated tag object is `4245aeec57a212e31d1c27e1077a401defd47d8b`
and dereferences to the release commit above. The repository description remains
consistent with implemented scope. Current status prose now reflects that release;
earlier batch records below retain their original evidence and limits.

Focused checks pass: canonical source-admission fixtures cover lock-only,
hidden staged, unstaged, untracked, unusual-name and rename cases, Git errors
and preservation (`shared-source-fixtures.log`). The adopted simulation-only
runner passes (`shared-release-runner.log`). Consumer fixtures additionally
confirm lock/staged diagnostics, refusal before the substitute gate, unchanged
source/index bytes, exact local tags/push, interrupted preparation and publication
reconciliation (`final-consumer-release-fixtures.log`, retained fixture
`target/portable-fixtures/release-tools.ZaZcZM`). Their mocked gate/transport and
local bare remote do not establish live delivery. Duplicate-recipe warnings are
deliberate fixture overrides; production `make help` has empty stderr.

The source-owned optional npm checker fixtures pass without executing npm, Node,
Cargo or network commands (`shared-npm-fixtures.log`); no npm root was selected
for this Rust repository. Snapshot integrity (80 files), Cargo pinning/inheritance,
documentation links and the diff pass (`final-governance.log`). ShellCheck passes
with the sourced PR companion included (`final-shellcheck.log`); the earlier
invocation omitted that companion and retained its SC1091 diagnostic separately.
Source identities and the working diff are retained alongside these logs.
No authentication or dependency graph changed, so no Rust library, full CI or
live canister qualification gate was run for this tooling batch.

The compatible pending changelog is 0.1.9; manifest versions remain 0.1.8.
No commit, push, release or publication was run.

## Earlier Shared Tooling/MSRV batch (released in 0.1.8)

The following records preparation against 0.1.7, before the maintainer's 0.1.8
release above. Its pending-version and qualification wording is historical.

Shared Tooling `1872ed2c20f6c70689bb2249050b1d673c60bfa0` (0.1.28) matched remote
main at observation. A clean isolated checkout exported committed bytes; an
initial attempt used an origin URL with a different `.git` spelling and was
refused before writing (`snapshot-refresh.log`). The corrected canonical URL
refresh passed (`final-snapshot-refresh.log`). The 79-file selection includes the
new complete governance/task catalog, malformed-link installer admission and
simulation-only release-runner fixture. `make tasks` reads that catalog;
no maintenance agent, timer or schedule was activated.

Rust 1.88.0 is now the common qualified package minimum, inherited by all five
members. Development Rust remains 1.99.0. The actual Rust 1.85.0 native probe
failed because selected `psm` 0.1.32 and `ar_archive_writer` 0.5.3 require 1.88.0
(`probe-1.85-default.log`); no dependency downgrade or ignore-version escape was
used. Native/Wasm 1.88 probes passed before declaring the floor. See
[the minimum compiler contract](../msrv.md).

`make check-msrv` prints real old-compiler identities and checks normalized
public package payloads outside the production workspace. Each library feature
and all features together pass on native and Wasm; internal tooling/runner
all-targets/all-features and the Wasm canister pass separately. Local protocol
payload staging preserves all external versions/sources/checksums; the script
checks that invariant against the workspace lock. The final successful run retained
`target/portable-fixtures/msrv.n5KtOq` (`qualified-msrv.log`); earlier successful
inputs remain separately retained. Setup explicitly
prepares 1.88.0 and its Wasm target without changing development selection or
updating rustup (`msrv-install.log`). The configured complete CI/release gate
now includes the minimum lane; local execution remained focused.

The initial Host 0.8.1 inspection was followed by a concurrent lock update to
all four Host 0.8.2 packages. That update was preserved. Live unyanked registry
checksums match the final lock; packaged source and observed remote main are
`92bd2fecc71124b562e227a32a67644e1e5e34b7`. The committed 0.8.1-to-0.8.2 diff
changes a portable child fixture from `/bin/true` to `/bin/sh -c 'exit 0'` and
release metadata, with no production API/runtime/dependency change. Dirty sibling
files were read-only and never imported. Auth graphs remain free of testkit,
native Host, Canic and wallet dependencies.

Released protocol/auth 0.1.7 checksums match the retained publication receipt:
`ccdbbc0217ee785e69d9c4d64f5e00a096e240ad4893a310f0cceecc4c9c0b5b` and
`130c5ef49bf8c9288acdf9ed9a78950a5daafd130234db4006ad897581b1e420`.
Both initially observed release CI runs were queued; the final observation has
one running and one queued (`final-release-ci.json`). No completed hosted
qualification is claimed:
[run 37800480770](https://github.com/dragginzgame/ic-auth/actions/runs/37800480770),
[run 37800481361](https://github.com/dragginzgame/ic-auth/actions/runs/37800481361).

Real testkit scenarios and strict native/Wasm Clippy pass on Host 0.8.2
(`host-0.8.2-qualification.log`, retained state
`target/portable-fixtures/qualification.u6K2B1`). Four native file tests, the
local/mocked consumer release fixture and the adopted simulation-only runner pass
(`focused-tooling.log`). The source-owned host/IC installer fixtures pass,
including malformed active links and actual collector cases
(`upstream-host-fixtures.log`, `upstream-ic-fixtures.log`); those synthetic native
host substitutions do not establish real macOS execution or footprint savings.
Both public package dry-runs build without uploading (`package-dry-run.log`).
The GitHub description remains consistent with implemented library scope.

Current evidence is retained separately under `target/baseline-0.1.28/`.
Final formatting, tool checks, snapshot, pinning/inheritance, links, metadata and
boundaries are recorded in `final-governance.log`; ShellCheck passes for the
consumer-owned new scripts. Final source/lock/snapshot/Wasm and normalized package
identities are recorded in `source-identities.sha256` in that same directory.
The selected next version is 0.1.8: support for a lower compiler, additive local
commands and shared tooling fixes preserve public auth APIs and signed bytes.
Manifest versions remain 0.1.7. No commit, push, release, publication, production
deployment or sibling adoption was performed. Wallet service policy and durable
session host adoption remain separate; older sections retain their original
pre-release selection/evidence wording.

## Earlier IC Testkit adoption (released in 0.1.7)

The following records preparation against 0.1.6, before the maintainer's 0.1.7
release above. Its pending-version and qualification wording is historical.

The maintainer required `ic-testkit` as the canister test infrastructure. The
working qualification package now selects published `ic-testkit` 0.25 and uses
its runtime re-export, typed Candid calls, IC Host file reader and bounded startup
with retained server stdout/stderr. The direct `pocket-ic` declaration and exact
pin exception were removed; no authentication library API or signed bytes changed.
Both complete certificate/lifecycle and signed-ingress scenarios are retained.

The command is now `make test-qualification`; its wrapper and documentation are
`scripts/dev/test-qualification.sh` and
[the IC Testkit contract](../ic-testkit-qualification.md). There is no alias for
the earlier uncommitted command. `ci/ic-auth-tools.tsv` is the consumer-owned
matrix: PocketIC 16.1.0 matches testkit's transitive client, while other tools keep
the shared defaults. Official release metadata verified all three archive hashes;
the immutable shared matrix/snapshot remains unchanged. Make setup, checks and
the offline qualification wrapper all select the consumer matrix. No test-time
download or alternative server discovery is enabled.

Published testkit 0.25.0 is unyanked, checksum
`1757127736e1d3bd4e6abe4d49017cfa75e2cdd7f9b07cb6a634a789c87e8c8f`.
Its package source revision and observed remote main are
`6b6d2cfe7f4c3e7a204a0c18beb6edb8895007b4`. The sibling's dirty files were
read-only and excluded. The concurrent Host fs/artifact update to 0.8.1 is preserved;
all four selected Host packages now use 0.8.1 from source
`973f00a029dd873c242a4d06e9f8d2d5172ff0df`. Qualification
dependencies remain confined to the unpublished native runner.

Both full scenarios pass through testkit with the checked 16.1.0 server
(`final-qualification.log`, retained state
`target/portable-fixtures/qualification.VRqUsc`), including CDPATH-sensitive wrapper
invocation. Strict native/Wasm Clippy passes (`qualification.log`). Local tool,
formatting, snapshot, pinning/inheritance, link, metadata and library-boundary
checks pass, as do the four native file-operation tests (`final-checks.log`).
ShellCheck and local/mocked release fixtures pass (`shellcheck.log`,
`final-release-fixtures.log`); the fixture copies the consumer matrix and new
wrapper along with the complete local packages. Both public package dry-runs
build successfully without uploads (`package-dry-run.log`). All four Host 0.8.1
registry checksums match the final lock and are unyanked (`ic-host-*-registry.json`).
Final source, lock, both matrices, snapshot, Wasm and package archive identities
are recorded in `target/testkit-adoption/source-identities.sha256`.
An initial formatting check
found the longer renamed command string, and a later link check found the old
document path; both were corrected before final checks. Server stdout/stderr
are retained separately per instance. Native macOS execution of these changed
sources awaits the configured CI jobs; no complete local CI gate was run.

Evidence from the earlier direct-PocketIC implementation below is historical;
its source identity list does not attest these changed files. Current adoption
evidence is retained separately under `target/testkit-adoption/`. Versions remain
0.1.6 with pending 0.1.7 notes; no commit, release or publication is authorized by
this correction.

## Earlier direct PocketIC qualification batch

`make test-pocketic` builds the unpublished qualification host and tests real
certification, root composition, expiry cleanup and protected metadata upgrades.
The host uses actual runtime caller/time/certificate values, controller-only
signing and one certification root containing assets, signatures and status.
The native runner uses the PocketIC instance's network key and the real library
verifier. Its independent resource canister checks an installation-protected
owner after ingress authentication. See the
[qualification contract](../ic-testkit-qualification.md).

The HTTP gateway scenario uses the IC agent's canonical delegation encoding and
real signed requests. Fresh Ed25519 session keys retain the same fixed fixture
principal across signer/resource upgrades. The replica rejects wrong targets,
changed signed expiration, wrong session keys and expired delegations. Valid
non-owner and anonymous requests reach the application guard and are denied,
separately from replica credential rejection. Previously issued, unexpired
delegations still authenticate after pending leaves are reset; clearing the
store is not revocation. Stable owner/branch metadata survives the fixture's
upgrade; this is not a stable session or signature-restoration adapter.

Both integration scenarios pass on Linux (`final-strict-qualification.log`), with retained
instance state under `target/portable-fixtures/pocketic.*`. The wrapper checks
the complete pinned IC bundle and locked client/server alignment before any
server execution; both PocketIC selections are 16.0.0. Negative ingress assertions
require HTTP 400/403 and the corresponding target/signature/expiration refusal;
transport failures cannot satisfy them. An initial strict assertion expected 403
for target rejection; the server correctly returns 400 for that case
(`strict-rejection-qualification.log`). The final run checks both supported
authentication rejection codes and specific reasons, and retains state in
`target/portable-fixtures/pocketic.kJbtc7`.
The first attempt lacked
an NNS subnet and correctly had no network root (`initial-pocketic.log`). Adding
that required trust-root subnet fixed the fixture, without changing library
verification. Intermediate and final successful logs remain separate.

The IC agent 0.49.2/CDK 0.20.3/PocketIC dependencies are confined to unpublished
applications. Bounded no-follow Wasm reading uses `ic-host-fs`. PocketIC owns its
server protocol/process behavior. Its 16.0.0 package pins `thiserror` 2.0.18, so
workspace resolution necessarily selects that patch instead of 2.0.21. Existing
type/encoding/signature/token/session and native tooling focused tests pass with
the resulting lock, as do every selected Wasm graph, metadata, dependency guards
and strict native/Wasm Clippy (`focused-rust.log`, `final-clippy.log`). No full
local CI/workspace gate was run. Final fixture-only source cleanup removed an
unused direct upstream signing dependency; metadata, both PocketIC scenarios and
strict native/Wasm Clippy passed again afterward (`final-strict-qualification.log`).

The initial user-changed Host 0.7.2 lock entries were preserved, followed by a
concurrent root/lock selection of Host 0.8.0. Live registry records match those
final fs/artifact entries; their packaged source revision is
`fc74f679c7503ef9dd2db8fbd893c5c5907c72c4`. Focused Clippy/native file tests and
the final PocketIC execution use 0.8.0. Its process cleanup API break is outside
this repository's selected fs/artifact usage; no local public contract changed.
The initial qualification executions used 0.7.2 and are not relabelled.

Shared Tooling source `db039347d2372b877c1c46dcdd2b5c3aa9412009` (VERSION 0.1.27)
matches remote main. An isolated clean copy exported committed bytes, excluding
the sibling's uncommitted edits. The canonical 65-file snapshot includes the
failure-evidence selector and PocketIC alignment/binary helpers. Default tool
retention remains full; no CI compaction option was selected. The source-owned
host evidence fixture passed against an isolated copied tool bundle, including
the actual collector (`selector-fixtures.log`). An earlier invocation omitted
required arguments and was refused (`upstream-evidence-selector.log`). This is
producer/helper evidence, not hosted GitHub artifact-roundtrip qualification.

Release fixtures now copy the actual adopted snapshot and discover every local
workspace package before synthetic version rewriting. Local/mocked publication,
runner and archive fixtures pass (`portable-fixtures.log`); warnings there come
from deliberate substitute Make recipes, not duplicate production recipes.
They perform no real repository release or registry upload.

Formatting, the 65-file snapshot, dependency pinning/root inheritance and local
links pass (`governance.log`). The exact SDK pin was initially refused by the
dependency guard; its client/server compatibility requirement is now documented
in `AGENTS.md` and scoped in `ci/dependency-pinning-exceptions.json`.
ShellCheck passes for the new wrapper and changed consumer release fixture
(`shellcheck.log`); that fixture passes again with the exception overlay
(`final-release-fixtures.log`). Both public package dry-runs build successfully
using live registry metadata, without upload (`package-dry-run.log`). The GitHub
description remains consistent with the independent authentication libraries
(`repository-description.json`). Final source, lock, snapshot and Wasm identities
are retained in `target/pocketic-qualification/source-identities.sha256`.

Live 0.1.6 registry checksums match its receipt: protocol types
`fad377ac63ecda737ec1636d79690729fc07b6b5f2d749262ed294b412645bec`, auth
`7eb9c82e1657c68bc9127ba02142a9d04c0c9e2e691a53fd837ef4a701521ddf`.
Both maintainer-triggered 0.1.6
[CI](https://github.com/dragginzgame/ic-auth/actions/runs/37782819316)
[runs](https://github.com/dragginzgame/ic-auth/actions/runs/37782819222)
completed successfully. This qualifies that released source on the configured
hosts, not the new uncommitted fixture on macOS.

The selected next version is 0.1.7: new commands/internal qualification and
shared tooling adoption are compatible; public auth APIs, signed bytes and
default dependency boundaries are unchanged. Manifest versions stay 0.1.6.
No commit, push, release, publication, production deployment or sibling adoption
was performed. Wallet service policies, wallet proofs/browser login and durable
session adoption remain separate. Older evidence below retains its original
source/selection wording and absent target files are historical references.

## Earlier signature preparation/retrieval batch (released in 0.1.6)

The following records preparation against 0.1.5 before the maintainer's 0.1.6
release/publication above; pending-version and qualification statements below
describe that earlier evidence.

Optional `canister-signature-preparation` exposes a bounded volatile
`SignatureStore`, independently of verification/token/session features. It
retains the existing Canic domains, empty signature paths, upstream DER keys and
CBOR proof DTO. The host authorizes preparation and supplies actual signer,
protected limits, clock, query certificate and ordered composition siblings.
There are no IC runtime calls, implicit root writes or native Host dependencies.
See the [contract](../signature-preparation.md) for input, lifecycle and
certificate-source obligations.

Physical capacity includes expired records. Each leaf has one expiry index
entry; live retries preserve their deadline, while expired-key repreparation
replaces that entry. Explicit prune removes at most 128 earliest expired records;
remove/prune preserve other leaves and never clone or scan the complete store.
All fallible preparation checks precede mutation. Retrieval enforces the
exclusive deadline and requires the supplied host certificate's canister root
to match the current composed witness digest. Its structural certificate check
does not substitute for consumer BLS/freshness verification. Deleting leaves
cannot revoke proofs already delivered.

Read-only extraction input remains Canic
`4c51a87c6a32397196bb3f65d064641194df10a5`; its dirty sibling changes were excluded.
The selected upstream signing-map source was reviewed rather than wrapped:
its internal clock and query-certificate calls are incompatible with this pure
boundary. DFINITY's certified tree, key and hash primitives are reused. Endpoint
eligibility, operation/caller records, renewal and other certified state remain
host-owned. Stable root publication/restore and real IC ingress delegation are
not qualified by this pure mechanism batch.

Eleven new tests pass, including 1999 live retries without expiry-index growth,
expiry/repreparation, failure atomicity, overflow/clock bounds, multi-key removal,
bounded pruning, certificate/output/depth limits and real BLS verification of
both Canic domains composed with two other certified branches. All 70 workspace
tests pass, including existing token/session and native file operations.
Strict all-feature Clippy, default and each independently selected Wasm graph,
locked metadata and dependency guards pass. Logs are retained under
`target/signature-preparation/`: `store-tests-fixed.log`, `workspace-tests.log`
and `compile-and-boundaries.log`. The first test compile rejected a borrowed
temporary (`initial-tests.log`); the following fixture run correctly rejected
an obsolete certificate and an undersized output limit (`store-tests.log`). The
fixtures were corrected without weakening production checks; failures remain.

The Cargo publication dry run built both archives and explicitly aborted uploads
(`package-dry-run.log`). The unpacked auth archive passed the preparation unit
and ten integration tests against checksum-verified published protocol types
0.1.5, and its preparation-only feature compiled for Wasm. Evidence is
`packaged-tests.log` and `packaged-wasm.log`. The isolated fixture adds an empty
workspace table and replaces Cargo's staged protocol checksum with the observed
published checksum; archive/source bytes are unchanged. This checks the package
boundary, not a live upload or a durable host.

Formatting, the 62-file snapshot, dependency pins and local links pass
(`governance.log`), along with ShellCheck and the final diff review.
`source-identities.sha256`, produced through the native `ic-host-*` utility,
binds source/tests, metadata, selected tooling/guards and the original archives.

The concurrently changed lockfile selection of `ic-host-fs` and
`ic-host-artifacts` 0.7.1 was preserved. Live registry checksums match the selected
entries and packaged source revision `410fee7c309e781edf6a361f0e480d71b7c11e5a`,
also observed as remote Host main. Native tests and Clippy use that selection.
Shared Tooling remote main still matches the adopted 0.1.26 revision
`75a8a60f49cec11d3f6aecab5c977029c42cc549`; no snapshot refresh is needed.

Live 0.1.5 registry checksums match the retained publication receipt: protocol
types `07832762f8124cfdc79d3c897e1ee74671b60e993cd8aa027be03c28643751bf`, auth
`40e6ed17becd4bd06fef97ef9d2d1ec9da09f06fd2a800a4613b96500487570c`.
The maintainer's 0.1.5 CI runs were still queued at the latest observation;
the preceding [0.1.4 run](https://github.com/dragginzgame/ic-auth/actions/runs/37772047548)
completed successfully on all configured hosts. Neither result qualifies this
uncommitted preparation batch on macOS.

The pending version is `0.1.6`: preparation is a compatible optional addition;
existing APIs, signed bytes and default graphs are unchanged. Manifest versions
remain 0.1.5. No commit, push, release, upload, full local CI, sibling adoption
or stable/PocketIC lifecycle was performed. Earlier target logs named below were
not present when this turn started and remain historical references.

## Earlier session/replay batch (released in 0.1.5)

The following records preparation against 0.1.4 before the maintainer's 0.1.5
release/publication above; its observations and pending-version wording are
historical evidence.

`ic-auth` now has optional `sessions`, with complete token verification before
new admission, canonical scope narrowing and exact Canic request hashes. The
engine resolves retry and consumption inside the host's synchronous transaction,
then stages only the touched session/replay pair. Exact retry after proof expiry
returns the unchanged still-authorized session. Replacement and logout retain
replay history; authority generation changes do not erase it. Live checks include
actual caller/subject, audience, role, current scopes, generation, enrolled root
key identity, epoch/version floors and narrowed root acceptance deadlines.

`MemorySessionStore` supplies a bounded volatile backend with explicit physical
record/encoded-byte quotas, indexed expiry cleanup capped at 128 records, checked
generation advancement and no whole-store clone/encoding. Per-record CBOR decode
is bounded and validates structural invariants; it is for trusted host storage,
not an ingress credential. The [session contract](../sessions.md) specifies
transaction rollback, monotonic cleanup time, durable restore and protected
authority-generation obligations. It distinguishes local session lifetime from
the original proof, IC ingress delegation and resource ownership. No CDK, stable
memory allocation, globals, certification publication or wallet graph was added.

The read-only extraction input remains Canic
`4c51a87c6a32397196bb3f65d064641194df10a5`. Its workflow, pure application policy,
scope model and storage operations were reviewed from committed source; the
sibling's staged/dirty work was excluded. Canic still owns its protected fleet
context, endpoint guards and durable backend. Consumer adoption must settle its
retained representation and remove the superseded owner in a separately
authorized change. This batch does not claim that ownership convergence.

Fourteen session tests pass with real ECDSA/BLS chains, including forged-proof
rejection, strict proof starts, TTL overflow at the maximum clock, exact retry,
generation/root-key/scope invalidation, logout/replacement, byte/capacity failures,
rollback after staging, corrupt record rejection and indexed bounded cleanup.
The request hash has an independently computed byte vector. Existing protocol,
token, IC signature and type tests pass too, along with strict all-feature
Clippy, default/signature/token/session Wasm compilation, locked metadata,
dependency boundaries and native tooling tests. Focused logs are retained under
`target/session-engine/`: `focused-checks.log`, `final-focused-checks.log` and
`session-final-checks.log` distinguish successive source/host selections.

The final Cargo package dry run built both archives and aborted uploads
(`package-dry-run.log`). The unpacked auth payload passed all 14 session tests
offline against checksum-verified, published protocol-types 0.1.4 and compiled
for Wasm with `sessions`. Logs are `packaged-native.log` and `packaged-wasm.log`.
The isolated fixture added an empty workspace table and replaced Cargo's staged
protocol checksum with the observed published checksum; library source and
original archives were untouched. This verifies the package boundary, not
downstream application or stable host adoption. Formatting, snapshot integrity,
dependency pins, local links, ShellCheck with sourced scripts and the diff pass;
evidence includes `governance-checks.log` and `shellcheck.log`.
`source-identities.sha256`, produced with the native `ic-host-*` utility, binds
the final source/tests, manifests/lock, Makefile, toolchain, shared snapshot and
selected guards/runner. Original validation logs and failed attempts are preserved.

The initial session build rejected the fixture's module path before execution;
`initial-tests.log` records it. The next run failed one expected strict-start
case because its IC certificate was signed ahead of the test clock. The fixture
time was corrected without weakening production freshness checks; the failed
run remains in `session-tests.log`. Later checks stopped correctly when the root
manifest concurrently selected Host 0.7 while the lock still selected 0.6;
`locked-host-selection-failure.log` records that refusal.

The concurrent root selection was preserved. Published `ic-host-fs` and
`ic-host-artifacts` 0.7.0 availability and checksums were verified, and their
packaged source revision `491fc0e231b9650526f5f57b9ab7b1f62f02218c` matched remote
main. Explicit offline workspace resolution changed only those two lock entries;
explicit locked fetch prepared them before offline validation. Native bounded
hash/no-follow/durable-create/replacement tests and Clippy pass on 0.7.0.
The earlier focused checks used 0.6.0 and are not relabelled. Both native owners
remain outside every auth graph; no Candid/process dependency was needed.

Shared Tooling 0.1.26 at `75a8a60f49cec11d3f6aecab5c977029c42cc549` was verified
as remote main, reviewed and adopted through its canonical exporter from the
clean sibling source. Its 62-file snapshot preserves the source-owned runner's
locked Git-ref transaction and concurrent symbolic-ref handling. Rules/tool pins
are unchanged. The adopted runner regression tests and local bare-remote release
adapters with mocked registry/upload transport pass; they do not run a live
release or the full local CI gate.

Live sparse registry records match the retained 0.1.4 intent: protocol types
`f3b8439d22f13f7bfd5fb382d5563f213c308a5951dbddb2a6014421fe926e56`, auth
`baad1f7e93287cfc77ccb863b2fad5760b650749ef9597fd0d347b65ca2fa4ee`.
The public repository description was corrected to the released token-verifier
scope and read back. The maintainer's 0.1.4
[CI run](https://github.com/dragginzgame/ic-auth/actions/runs/37772047548)
passed Linux and ARM macOS; Intel macOS was still running at the final observed
read. This qualifies the pushed 0.1.4 source on those completed hosts, including
the fixture-isolation fix. Native macOS qualification is not inferred for this
new session/Host 0.7/Shared Tooling 0.1.26 batch.

The pending version is 0.1.5: session APIs are optional compatible additions,
existing token/signature APIs, signed bytes and default graphs are unchanged,
and the native Host/shared-runner updates preserve maintained command contracts.
Manifest versions remain 0.1.4. No commit, source push, release, upload, full local
CI, stable backend/PocketIC lifecycle or downstream adoption was performed.
Current evidence is under `target/session-engine/`; the earlier target logs listed
below were not present when this turn started and remain historical references.

## Earlier application-token verification batch (released in 0.1.4)

The following evidence records preparation against 0.1.3 before the maintainer's
subsequent 0.1.4 release/publication recorded above.

The optional `token-verification` feature verifies the complete existing root
ECDSA chain-key/Merkle proof and issuer IC BLS proof. Actual authenticated caller,
clock, root-key enrollment, audience, local role, allowed/required scopes, finite
input limits and live epoch/version/acceptance policy are explicit host inputs.
Both cryptographic verifiers are concrete; no accepting callbacks or retained
identity cache can bypass a proof. Verified claims are borrowed, cannot be
deserialized into verification evidence, and expire no later than the host's
current root-key acceptance deadline. The [API contract](../tokens.md) explains
the remaining host enrollment, decoding, replay, session and resource-policy
obligations.

The read-only source remains committed Canic
`4c51a87c6a32397196bb3f65d064641194df10a5`. Its root chain-key header, delegation
leaf, Merkle directions, key/path bindings and issuer domain are preserved.
The canonical derivation-path hash now has an independent fixed byte vector.
`k256` owns SEC1 parsing, ECDSA scalar validation, high-s detection and prehash
verification. No custom cryptography, Canic authority registry, network-key-name
policy, storage, runtime clock, certification-root publication or Solana graph
was imported. Native file/artifact operations continue to use `ic-host-fs` and
`ic-host-artifacts` `0.5.2`; they remain excluded from both auth libraries.

Focused Linux checks passed: 41 auth tests across the selected APIs, strict
all-feature Clippy, default/signature/token Wasm builds, locked metadata and
library dependency boundaries. The 12 token tests use real ECDSA and BLS proofs
and cover identity, grant narrowing, authority invalidation, exact time bounds,
forged signed bytes, malformed/high-s signatures, witness tampering and input
budgets. Logs are under `target/token-verification/`, including
`focused-checks.log` and `initial-tests.log`. `source-identities.sha256`, produced
by the native `ic-host-*` utility, binds the final library/tooling source and
tests, manifests, lockfile, Makefile, toolchain, snapshot and boundary guard;
documentation-only source comments were completed before final packaging.
No existing locked external version
changed; 12 selections were added for optional `k256` verification.

The pushed release's native CI exposed an intermittent parallel fixture
collision: in [run 37764724605](https://github.com/dragginzgame/ic-auth/actions/runs/37764724605),
both macOS jobs failed the tooling intent test after reading the hash test's
`abc` input. Linux passed. The old directory name used PID and clock precision,
and `create_dir_all` could share a directory between simultaneous tests.
The local helper now adds a process-wide atomic sequence and exclusively creates
each leaf directory. Four host-tooling tests and strict Clippy pass locally;
native macOS qualification of this fix awaits CI for the new source. The failed
job log is retained in `previous-ci-failure.log`, and local checks in
`fixture-checks.log`. This changes test isolation, not production intent writes.

The final real Cargo package dry run built both distributable archives and
aborted uploads (`package-dry-run-final.log`); the earlier dry run remains in
`package-dry-run.log`. The unpacked auth payload also compiled offline for native
and Wasm with `token-verification` against the checksum-verified, published
protocol-types `0.1.3`. This isolated fixture copied the normalized manifest,
added an empty workspace table, and replaced Cargo's staged-dependency checksum
with the observed published checksum; it did not change the source or archives.
Logs are `packaged-native.log` and `packaged-wasm.log`. The first attempt refused
the enclosing workspace before compilation; that result remains in
`packaged-native-workspace-failure.log`. This is package-boundary compilation,
not downstream application adoption.

Formatting, dependency pins, snapshot integrity, local links, boundary-script
ShellCheck and the diff passed. Local bare-remote release adapters and mocked
publication/recovery passed with a substituted complete gate, not live effects;
logs are `governance-checks.log` and `release-adapter-checks.log`. Recipe override
warnings occur only in that intentionally substituted fixture Makefile. The
maintained Makefile has one recipe per target. Existing-version dry-run warnings
are expected because manifests remain `0.1.3`.
Sparse registry records independently matched the
retained `0.1.3` publication checksums: protocol types
`b667ca6a00543bc8a3e36d2d6d8a8e3cee2528e986a894a28025c86311aec273`, auth
`56f767225901ae8e618ddddb266d5391cb755bd13e7caba9dafed24da2c52e25`.
The crates.io API returned HTTP 403 during read-only checking; the public sparse
index supplied the live registry observation instead.

Shared Tooling `0.1.25` at `eeb72e741199bd8574280eacb3542d8379b912f6`
was reverified as remote `main`; the reviewed 62-file snapshot is unchanged.
The repository description was read and still accurately describes the released
scope. The next undated changelog entry is `0.1.4`: optional token verification
and fixture isolation are compatible additions/fixes, with existing encoding,
wire contracts and default graphs preserved. No manifest bump, commit, push,
release, publication, full local CI, native macOS run, live IC/PocketIC execution
or consumer adoption was performed by this batch. Session/replay atomicity,
signature preparation/retention and wallet service contracts remain separate
unfinished extraction work in the existing tracker.

## Earlier signature verification batch (released in 0.1.3)

The following evidence records preparation against `0.1.2`, before the
maintainer's subsequent `0.1.3` release and publication recorded above.

The maintainer requested continuation of the accepted extraction. The working
tree now adds the optional `canister-signature-verification` feature to `ic-auth`.
It uses DFINITY's public-key parser and IC verifier with protected signer, seed
hash, raw network root key, clock and finite byte/freshness limits supplied by
the host. It guards the upstream short-input DER slice, requires exact short-form
DER re-encoding, and checks authenticated signing-certificate time after signature
verification. The [API contract](../signatures.md) distinguishes message
authentication from application-token/session admission.

The read-only Canic input was committed
`4c51a87c6a32397196bb3f65d064641194df10a5`; sibling dirty changes were excluded.
Issuer and root-role signature domains are preserved byte-for-byte. No Canic
authority, global signature maps, metrics, clock calls or certification-root writes
were imported. Preparation, retrieval, bounded signature retention, host-owned
certification composition, the complete token/session engine and wallet service
remain unimplemented. The existing `0.1.2` registry packages still contain only
the earlier contracts/encoding; this addition is local and unpublished.

Focused Linux checks passed: signature tests using real BLS-signed fixture
certificates, the existing 17 protocol/type tests, all-feature strict Clippy,
default and verification-selected Wasm compilation, locked metadata and both
selected dependency graphs. Evidence is retained in
`target/signature-verification/validation.log`. Coverage includes exact Canic
domain bytes, protected authority changes, message/certificate/tree tampering,
subnet delegation range and nesting rejection, DER truncation/integrity, input
limits and missing/truncated/overflow/trailing time with exact age/skew boundaries.
The initial run failed one fixture expectation: changing the encoded principal
length can form a structurally valid key with the wrong signer. The test now
separates malformed lengths from the maintained `CanisterMismatch` contract;
production verification was unchanged. The failed run remains in the session
transcript and its build artifacts were preserved.

No existing locked registry package version changed during this addition; the
23 added selections support optional verification and its fixture cryptography.
`ic-host-fs`/`ic-host-artifacts` were already selected at `0.5.2` when this turn
started and remain in the native app only. The earlier host-tooling evidence below
describes its original `0.5.1` adoption. The published Host `0.5.2` source at
`c7014995bf0890c1df9cd9b9a6ec14ea70f98c6f` was verified as remote `main` and
inspected read-only; no unrelated package update was performed.

Shared Tooling advanced to `0.1.25` at
`eeb72e741199bd8574280eacb3542d8379b912f6`, verified as remote `main`. The reviewed
delta hardens archive paths, including inherited `CDPATH`, option-like names,
newlines and occupied pipes. The canonical exporter initially refused to replace
the earlier uncommitted snapshot files. After verifying the old snapshot, its
exact source files were preserved under `target/shared-tooling-evidence/0.1.24/`;
a clean source and isolated canonical export supplied the replacement bytes.
The 62-file snapshot includes the upstream archive regression fixture. No shared
script was patched locally. Baseline rules and tool pins are unchanged.

The adopted archive regression fixture passed; formatting, dependency pins,
snapshot integrity, local links, ShellCheck and substituted release Make routing
passed. Logs are under `target/signature-verification/`. The live registry-aware
package dry run built both distributable archives and aborted upload; its log is
`target/signature-verification/package-dry-run.log`. Existing-version warnings
reflect unchanged manifest version `0.1.2`, not a new publication attempt. The
public description was read and remains accurate for the released encoding APIs
and complete token verification still in development.

The pending release remains `0.1.3`: the optional verifier is a compatible
addition, existing encoding APIs/wire bytes are unchanged, and no manifest version
was bumped. No full local CI, native macOS, live network signature, PocketIC,
commit, source push, release or upload is claimed by this batch.

The earlier host-tooling snapshot contained 61 committed Shared Tooling files from
`e9bfdc54c0daefc3dbbdfe091e5665dca5468eb3` (0.1.24), verified as remote `main`.
It was exported by the canonical refresh helper from a clean temporary clone,
preserving the sibling's dirty archive/evidence work. The snapshot verifier
passed. The snapshot supplies shared
Make/tool setup, formatter hook and focused governance helpers. Pinned host,
Rust and IC tools are installed in this checkout and `make tools-check` passed
on Linux x86_64. Tool installation output is retained at
`/tmp/ic-auth-install-tools.log`; installation build output remains under
`.tools/rust/build/`. The repository-local formatting hook is enabled.
The refresh adds the direct-runner regression fixture, evidence archiver and
failure-retention action. Latest final-hook payload/index/tag checks, completed
resume reconciliation and conditional remote-tracking refresh are adopted.
Tool pins and the toolchain selection are unchanged; no tool upgrade was needed.

## Earlier host-tooling adoption

The maintainer requested continued work using `ic-host-*` where applicable and
the latest committed Shared Tooling. That batch originally selected
published `ic-host-fs`/`ic-host-artifacts` 0.5.1 only in `apps/tooling/`. Registry
availability/digests were checked; the sibling's dirty process changes were not
adopted. Existing locked package selections were preserved; only the host app,
its two owners and their required dependencies were added. The dependency guard
now rejects native host dependencies in either auth library graph.

The native CLI delegates bounded SHA-256 identities, no-follow reads and durable
private-file publication to those owners. Release validation receipts use durable
replacement; publication intent and dispatch markers use durable create-only
writes before upload dispatch. Limits and caller ownership are documented in
[development](../development.md). Registry policy, uncertain-effect recovery,
locking and Git orchestration remain here or in their adopted canonical scripts.
No current Candid/process adapter calls justify adding the other two Host crates.

Focused Linux validation passed: four native CLI tests, strict Clippy, locked
metadata, Wasm compilation, auth graph boundaries, release Make routing,
the latest upstream runner fixture and local/mocked consumer release/publication
fixtures. The exact already-published `0.1.2` commit also passed the updated late
metadata check: local package transformation is derived from its saved source,
so a release predating the host app remains verifiable. ShellCheck, Actionlint,
formatting, installed tool checks, pins, snapshot integrity and local links pass.
The evidence archiver was exercised against real retained host fixtures. The live
Cargo publication dry run built both distributable libraries and aborted uploads;
its log is `/tmp/ic-auth-host-publish-dry-run.log`. Existing registry selections
were not upgraded and native host packages are absent from the library graphs.

The first recovery fixture failed because locked Cargo could not rebuild the
utility while the interrupted manifest/lock pair was inconsistent. Receipt
reconciliation now uses the already-adopted portable checksum helper at that
boundary; normal validation/publication use the Rust utility. The corrected
fixtures pass. The failure remains in `/tmp/ic-auth-host-release-tools.log` and
`target/portable-fixtures/release-tools.HjHcgk/`; final evidence is in
`/tmp/ic-auth-host-release-tools-final3.log` and its reported fixture directory.
The publication fixture also changes source during dispatch-marker creation and
proves refusal before either upload; source is rechecked after the durable write.
The shared runner log is `/tmp/ic-auth-latest-release-runner.log`.

CI now runs the complete gate on Linux and macOS 15 Intel/Apple Silicon and
archives failed validation/fixture evidence from `target/portable-fixtures/`.
No native macOS, full local CI, PocketIC, new release, source push or upload was
performed by that batch. Its pending `0.1.3` notes selected a compatible patch:
library APIs, signed bytes and registry upload/retry semantics were unchanged.
The repository description was reviewed for that batch. At that time, the local
signature capability was not yet included in the published package contract.

## Earlier extraction and release evidence

This starts A1 and the encoding portion of A3 without claiming either complete.
Protocol `Fleet` labels and signed domains remain unchanged. Auth role parsing
now validates at construction/deserialization; the host still owns trusted
identity projection and all authority. Remaining Canic consumer adaptation,
generated-Candid/feature coverage and canonical ownership convergence stay in
the tracker. Preserve Canic's current release boundary and keep its reinstall-only
policy out of wallet identity storage.

Focused validation passed: 17 tests across both packages, the three imported
Canic hash vectors, Candid identity/role byte compatibility, Wasm compile checks,
Clippy with warnings denied, locked metadata, transitive dependency boundaries,
formatting, snapshot integrity and local links. Initial Clippy validation found a
constant-size iterator lint; it was corrected and the check passed. No full CI,
PocketIC, live verification, service or downstream-adoption evidence is claimed.

The implemented release/publication commands are tracked by
[IC Auth #1](https://github.com/dragginzgame/ic-auth/issues/1).
The shared runner now has working direct-delivery metadata and validation adapters
for patch/minor/major/resume. It retains complete-gate receipts and checks exact
selected commits, narrowly owned metadata, annotated tags and atomic branch/tag
delivery. Publication requires a clean, tagged, pushed release and verifies
crates.io archive checksums with retained intent before each upload.

Earlier focused checks cover real metadata transforms and local bare-remote
releases with a substitute complete gate, plus mocked registry/upload transport.
They exercise gate failure, staged source rejection, candidate conflicts, all
three increments, interrupted preparation, exact resume, dirty/untagged publish,
unknown registry state, checksum conflicts and lost-upload-reply reconciliation.
Fixtures and failed attempts remain under `target/release-tools.*/`. Package dry
runs use real Cargo and registry metadata, build both distributable crates and
abort upload. No live release or registry write is claimed by those checks.
Dependency pinning/root inheritance now pass against the tracked real lockfile;
formatting, locked metadata, shell/Perl checks, license copies, snapshot integrity
and local links are checked for this batch. The previous 17 auth tests remain
bootstrap evidence; no full CI or PocketIC run was added to this local tooling task.

Strictly offline attempts failed for unavailable registry metadata and Cargo's
staged-dependency checksum error; the first metadata fixture exposed yq's inability
to emit complete TOML. The implementation uses an explicit registry-aware dry
run and narrowly checked byte-preserving manifest edits. The interrupted-write
fixture also exposed an expected-receipt path error, which was corrected before
the recovery checks passed. Raw attempts are retained under `target/` and in
`/tmp/ic-auth-*-dry-run*.log` and `/tmp/ic-auth-release-tools.log`.

The maintainer's subsequent `make publish` attempt packaged `0.1.1` and stopped
at a registry checksum conflict for `ic-auth-types`, before either upload was
dispatched. The retained `.git/publication-state/0.1.1.json` binds that attempted
source, tag and archives; there are no dispatch markers for that attempt.
The existing package is actually `ic_auth_types`, from
[ldclabs/ic-auth](https://docs.rs/crate/ic_auth_types/0.1.1); crates.io's
[name restrictions](https://doc.rust-lang.org/cargo/reference/registry-index.html#name-restrictions)
prevent distinguishing it by hyphens versus underscores. Its delegation/CBOR
types do not replace our Canic-compatible application-token/proof contracts.

The delivered `0.1.2` repair renamed our package and directory to
`ic-auth-protocol-types`, with Rust import `ic_auth_protocol_types`, throughout
the workspace, dependency graph, publication intent, release transforms and
fixtures. `ic-auth` and the repository name are unchanged. Candid layouts,
signed domains and hash contracts are unchanged. Neither spelling of the new
name appeared in the registry index on 2026-10-08; that does not reserve it.
The old `0.1.1` tag, release receipt, publication intent and artifacts remain
intact. Dependency/import names are a breaking public Rust contract; the
maintainer explicitly selected version `0.1.2` as an exception scoped
to this rename, instead of the normally required `0.2.0` minor increment.
The rename remains marked breaking and requires consumer dependency/import
updates. The original repair did not execute a release or upload; subsequent
maintainer delivery/publication is recorded above. See
[development](../development.md) for current command effects.

At the maintainer's request, the changelog now separates bootstrap/extraction
notes under the original `0.1.0` entry from the release/publication tooling added
in `0.1.1`. The original undated `0.1.0` heading is retained, with its bootstrap
commit/date and explicit untagged/unpublished status; the finalized `0.1.1` date
is unchanged. These historical corrections are based on commits `7a03102`,
`15a7fe4` and `61b1139`, rather than a fabricated earlier tag or publication.
Local links, snapshot integrity and the documentation diff were checked; no
Rust builds or portable test suites were run for this documentation correction.

Rename validation passed: all 17 existing Rust tests, locked offline metadata,
dependency boundaries, Wasm compilation, formatting, license checks, snapshot
integrity, local links and shell/Perl checks. The release/publication fixture
passed with the renamed package, real transforms/local bare remotes and mocked
uploads; its evidence is retained in `/tmp/ic-auth-rename-release-tools.log` and
the named `target/release-tools.*` directory. The live registry-aware Cargo dry
run built both package payloads and aborted both uploads; its log is
`/tmp/ic-auth-rename-publish-dry-run.log`. External locked dependency selections
are unchanged. No full CI or macOS execution is claimed for this rename.

The dependency-pinning checker enumerates tracked paths as well as untracked
ones, so it rejected the unstaged directory move's missing old manifest. It
passed with the move admitted into an isolated temporary index, leaving the real
index untouched during that repair. The move was subsequently committed by the
maintainer. Failed attempts and previous publication evidence remain retained.

Documentation closeout identified the wallet-service contract decisions explicitly
in the design: application identity scoping, credential linking/recovery, distinct
revocation semantics and exact public protocol limits. They do not block initial
Canic library extraction, but must be settled before supported wallet login.
