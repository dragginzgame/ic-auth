# Current handoff — 2026-10-08

The maintainer accepted the larger Canic authentication extraction, selected the
name `ic-auth`, and confirmed `/home/adam/projects/ic-auth` as the local repository.

The current committed release is `61b113961e9ab0719a9fee417d9be484c6f2e3fa`
(`0.1.1`) on `main`, with annotated tag `v0.1.1`. The maintainer executed that
release after committing the command implementation. The original bootstrap was
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

The virtual Rust workspace now contains `ic-auth-protocol-types` and `ic-auth`, with one
lockfile, root-owned dependency selections and current manifest version `0.1.1`.
The passive token/proof contracts and canonical encoding are adapted from the
clean Canic source at `e286b3fd98460c98670336853f80658a920966e0`; see the
[source review](../design/canic-source-review.md) for exact ownership and API
adaptations. Both packages now allow crates.io publication of their implemented
contracts/encoding, with inherited repository/README metadata and matching MIT
notices in the package payloads. They remain unpublished. There is no token verifier,
session engine, wallet endpoint or TypeScript client yet. Canic adoption has not
occurred; no sibling repository was modified.

The engineering snapshot contains 58 committed Shared Tooling files from
`2687f26317952c43c685f7f799ed09288dc10a67`. It was exported from a clean temporary
clone, preserving the source checkout's unrelated uncommitted work. The snapshot
verifier passed during installation. The expanded snapshot supplies shared
Make/tool setup, formatter hook and focused governance helpers. Pinned host,
Rust and IC tools are installed in this checkout and `make tools-check` passed
on Linux x86_64. Tool installation output is retained at
`/tmp/ic-auth-install-tools.log`; installation build output remains under
`.tools/rust/build/`. The repository-local formatting hook is enabled.
The added upstream helpers provide exact release-tag checks and byte-preserving
local lock-version transformation from the same adopted revision.

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

Current focused checks cover real metadata transforms and local bare-remote
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

The current local repair renames our package and directory to
`ic-auth-protocol-types`, with Rust import `ic_auth_protocol_types`, throughout
the workspace, dependency graph, publication intent, release transforms and
fixtures. `ic-auth` and the repository name are unchanged. Candid layouts,
signed domains and hash contracts are unchanged. Neither spelling of the new
name appeared in the registry index on 2026-10-08; that does not reserve it.
The old `0.1.1` tag, release receipt, publication intent and artifacts remain
intact. Dependency/import names are a breaking public Rust contract; the
maintainer explicitly selected pending version `0.1.2` as an exception scoped
to this rename, instead of the normally required `0.2.0` minor increment.
The rename remains marked breaking and requires consumer dependency/import
updates. The canonical manifest version stays
`0.1.1` until an authorized release. This repair is uncommitted and does not
execute a release or upload. See [development](../development.md) for command
effects and the fresh-release requirement.

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
passed with the move admitted into an isolated temporary index; the real index
is untouched. Stage both sides of the rename when committing. Failed attempts
and the previous publication evidence remain retained.

Documentation closeout identified the wallet-service contract decisions explicitly
in the design: application identity scoping, credential linking/recovery, distinct
revocation semantics and exact public protocol limits. They do not block initial
Canic library extraction, but must be settled before supported wallet login.
