# Current handoff — 2026-10-08

The maintainer accepted the larger Canic authentication extraction, selected the
name `ic-auth`, and confirmed `/home/adam/projects/ic-auth` as the local repository.

The current committed release is `b2bdbbe9a5723eb9c93f7d7a274e5e2280278e9b`
(`0.1.3`) on `main`, with annotated tag `v0.1.3`; both remote identities were
verified during this batch. Both libraries' `0.1.3` registry checksums match
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
dependency selections and current manifest version `0.1.3`.
The passive token/proof contracts and canonical encoding are adapted from the
clean Canic source at `e286b3fd98460c98670336853f80658a920966e0`; see the
[source review](../design/canic-source-review.md) for exact ownership and API
adaptations. Both packages now allow crates.io publication of their implemented
contracts/encoding, with inherited repository/README metadata and matching MIT
notices in the package payloads. The released `0.1.3` also includes optional IC
canister-signature verification. Complete application-token verification is now
implemented locally for the pending `0.1.4`; there is no session engine, wallet
endpoint or TypeScript client yet. Canic adoption has not
occurred; no sibling repository was modified.

## Current application-token verification batch

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
