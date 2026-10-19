# Development

The virtual root owns the protocol/types, Auth engine and Wire transport libraries,
unpublished native tooling in `apps/tooling/` and the reference host/runner in
`apps/qualification/`. The root catalog and lock select published Platform 0.1.18,
Host FS/Process 0.29.0, Testkit, Memory, Build and the shared Wire codec/TypeScript
generator. The [handoff](status/current.md) records exact evidence;
[session contracts](sessions.md) distinguish supported behavior from pending hosts.
Rust `1.99.1` remains the development compiler selected by `rust-toolchain.toml`.
The common package minimum is Rust `1.91.0`, inherited from the root catalog and
qualified by explicit older-compiler checks. The internal qualification canister
inherits the same minimum; explicit setup prepares it once. See [MSRV coverage](msrv.md).
Linux x86_64 is the initially exercised host.

The supported host matrix is Linux x86_64 and macOS 15 on Intel and Apple Silicon.
CI runs the same complete gate on `ubuntu-24.04`, `macos-15-intel` and `macos-15`.
Automatic runs cover `main` pushes and pull requests. Release tags do not enqueue
a second equivalent matrix. Newer revisions cancel older queued and running
checks for the same workflow and branch or PR ref; unfinished hosts on cancelled
revisions remain unqualified.
Source-bound host acceptance belongs in the [current handoff](status/current.md);
a local Linux pass does not establish macOS qualification.
Shared
Tooling's [host prerequisites](supported-hosts.md) apply, including Bash 3.2+,
GNU Make, rustup, Git and standard Unix setup utilities.

CI prepares Node through its pinned setup action, then
[scripts/dev/prepare-client-ci.sh](../scripts/dev/prepare-client-ci.sh) installs
exact npm into that Node installation with an explicit prefix. It records actual
versions/paths/prefixes and publishes the admitted bin directory for later steps
only after dependency preparation and offline admission succeed. The workflow's
strict pipeline preserves the first setup failure in retained logs. This is
explicit CI setup; ordinary checks do not install. The retained Intel macOS
failure under [#38](https://github.com/dragginzgame/ic-auth/issues/38) has no actual
version/path observations. The repaired route passes Intel macOS at 0.5.5.
Complete native acceptance remains source-bound and still requires the unfinished
ARM case.

CI waits for its exact uploaded Rust-tool artifact through the shared read-only
helper before the pinned download action. Successful listings retry an absent
ID at most six times with 60 seconds of backoff, within a two-minute step;
API/schema, expired or wrong-run identities fail immediately. Download requires
discovery success and retains strict digest admission and independent retained
byte checks. `test-tools-evidence` exercises the actual workflow command with
substituted API/sleep effects; it does not upload or retrieve live artifacts.

Shared 0.9.6 also bounds transient host/IC asset setup retries. Permanent download
and checksum errors remain terminal, with failed candidates and previous tool
selections retained. Ordinary checks stay offline. Snapshot and release recovery
admit only complete 40- or 64-character Git identities; PR identity strings reject
trailing newlines before state writes. Auth retains direct release delivery.

## Release editor setup

### Release checkout isolation

Keep a dedicated clean checkout for release/publication and a separate checkout
for development and dependency updates. Finish and commit the dependency batch
before bringing it into the release checkout. For initial setup:

```sh
git clone https://github.com/dragginzgame/ic-auth.git /path/to/ic-auth-release
cd /path/to/ic-auth-release
git status --short
```

Run the existing `make release-patch`, `make release-minor` or
`make release-major` there only with release authority. Keep agents, editors,
dependency updaters and other validation out of that checkout for the entire
attempt. The release still performs source admission and the complete gate;
the input hash guard remains necessary. A separate checkout prevents ordinary
development edits from changing its files; it does not serialize competing
remote releases or make a failed gate reusable.

Before later releases, fast-forward the clean release checkout with
`git pull --ff-only` only when there is no unfinished release. Resume an
interrupted attempt in its original checkout through the normal release command,
preserving `.git/release-state`, publication intent, prepared tools and `target`
evidence. Do not reset, re-clone or clean away that recovery state. After delivery,
fast-forward the development checkout when its own pending work permits it.

### Editor preparation and recovery

Shared 0.25.1 delegates Cargo manifest and lock rendering to the released IC Build
editor. Auth's normal release preflight prepares and admits the selected editor
after source and changelog admission, alongside the other required tools. The
adapter selects the absolute path internally; no shell export is required.

For offline checks outside a release, explicitly prepare the current selection:

```sh
make install-release-tools
make release-tools-check
```

The check prints the admitted path and never installs anything. Auth uses an
8 MiB per-input allowance. Recovery reads both member identities and the editor
selection from the validated Git source, so a newer or partially prepared live
manifest/lock cannot change either. Recovery/checks only admit the already
prepared saved-source editor offline; they never install or rebuild it. The
manual setup target uses today's lockfile; `scripts/dev/release-tools.sh` also
accepts an explicit saved `LOCKFILE` for deliberate historical tool preparation.
CI prepares and checks the editor before release fixtures. No release/publication
is performed by either setup/check command. The old Perl editors and Auth's
manifest rewrite wrapper are retired; application APIs and package versions are
unchanged.

## Crypto dependency boundary

Auth selects Platform 0.1.18's independent `ecdsa` capability for chain-key
verification and `ed25519` for portable signatures. Original K256/Dalek types and
cryptography stay upstream. Auth owns prehash construction, high-S refusal,
explicit producer normalization, canonical-scalar checks and verification policy.
There are no direct K256 imports or declarations; a test-only Dalek feature
trigger remains in the unpublished adversarial harness. SHA-256, CBOR and LEB128
remain direct general-purpose dependencies with current Auth-owned consumers.
Testkit retains the native simulator and server-lifecycle interface; routing its
already shared runtime re-export again would not remove a direct dependency.

Native file and process callers select Host FS/Process 0.29.0 with defaults
disabled. Auth uses library APIs, not the optional `ic-host-file` CLI, certificate
decoder or Wasm text/encoding backends. Those new Host capabilities do not replace
an existing direct dependency here. Keep them out of the production library graph.

The native Auth CLI uses Clap for argument grammar, values and help. Its generic
file-operation adapter is retained until Shared's verified selected-tool installer
can select Host's required `cli` feature
([Shared #289](https://github.com/dragginzgame/shared-tooling/issues/289));
[Auth #78](https://github.com/dragginzgame/ic-auth/issues/78) owns consumer cutover. Do not
add a bespoke store, implicit Cargo flags or an older-binary fallback. Cut over
only after explicit setup/offline admission and Auth's real file/release fixtures
qualify the published CLI; preserve Auth's byte/path limits and recovery policy.

## Dependency roles and deployment admission

Shared Tooling 0.25.1 source is adopted at
`d57c631206e3618755e45bebf2c88c673564ddc1` (reviewed committed source, not a
claim of registry publication or a matching release tag).
The root catalog uses `Production dependencies` and `Tests and tooling only`;
member annotations distinguish the three production libraries from internal
qualification and native tooling packages. These roles describe graph use, not
registry publication. Platform is a production entry; its optional capabilities
are admitted per graph. Candid, ic0 and BLS verification can remain transitive in
certified/runtime selections. Native Host, Build and Testkit selections remain
tests/tooling entries.

`make fmt`, `make fmt-check` and the pre-commit hook use Shared's grouped Cargo
sorting default; Auth no longer repeats paired formatter overrides. Release preparation writes that same ordering; current, prepared and
selected committed-source checks refuse unsorted groups without rewriting them.
`make test-formatting-hooks` derives an unsorted manifest from current inputs and
qualifies sorting, lock preservation and staged-file isolation.

`make check-boundaries` inspects locked normal/build transitive graphs on the
compiler's native host target and `wasm32-unknown-unknown`: all three library
defaults, each Auth capability, the session/portable combination and all Auth
capabilities together. Negative fixtures include Wasm-only tooling leaks. The
check runs in CI/release validation, independently of comment labels. Public
libraries exclude native Host/Build/Testkit/Agent/Tokio, fixture packages and
service-owned CDK/storage. Certified capabilities intentionally retain their
upstream certificate stack; portable/session-only graphs remain certificate-free.

There is no production service or deployment command here. The Memory/CDK Wasm
host is qualification-only and legitimately uses test/tooling packages. A Canic or
future wallet-service deployment must check its own exact target/features and
bind source, lockfile and successful build evidence to the final artifact digest.
This repository's graph checks do not enforce that external deployment binding,
validate role comments mechanically, or qualify unlisted feature combinations.

## IC Platform boundary

The protocol package uses published `ic-platform` 0.1.12 without features for its
unconditional Principal re-export. This retains the same upstream identity and
serde/self-authenticating/conversion capabilities, with no Candid or runtime edge.
The direct `ic_principal` declaration is removed; Platform owns that selection.

Private controls and diagnostics now use bounded Wire contracts under
`apps/qualification/contracts/`; public application contracts remain in
`ic-auth-wire`. The runner uses Testkit raw calls with typed Wire requests/replies,
not its Candid adapter. Init and post-upgrade arguments also use explicit Wire
method identities. Hidden CDK exports with custom codec hooks require no direct
Candid declaration, derive or service export. The host selects Platform
`cdk`/`certification`; the native runner selects `agent`/`canister-sig-creation`.
The host parser stays disabled.

The maintainer authorized retiring the unpublished fixture's Candid checkpoint.
Its replacement is a separately identified bounded Wire checkpoint with
`AUTHWIR2` framing; the old `AUTHCELL`/Candid payload is refused, not migrated or
reset. IC Memory allocation and public/production storage contracts are unchanged.
Checkpoint restores remain closed until matching controller authority is confirmed.

CDK runtime/controller behavior is retained ([Platform #1](https://github.com/dragginzgame/ic-platform/issues/1)).
No controller wrapper, executor, callback or crypto implementation is recreated.
Native Agent/delegated identities come from Platform, with native PEM support;
caller/root-key selection, request policy and authenticated SDK behavior remain
fixture-owned. Auth no longer needs Testkit's distinct Candid limits identity.

Certified Auth capabilities now select Platform's `canister-sig-creation`,
`certification` and `signature-verification` features explicitly. The `bls`
feature is used only by native test signing; it is not forwarded by Auth's public
features. All imports and direct upstream declarations for these four owners
are removed ([#16](https://github.com/dragginzgame/ic-platform/issues/16),
[#17](https://github.com/dragginzgame/ic-platform/issues/17),
[#18](https://github.com/dragginzgame/ic-platform/issues/18)). Upstream types,
DER/CBOR bytes, errors, resource admission and protected authority remain intact.
Published creation still imports Candid for Principal; verification still reaches
it transitively. No unpublished Principal patch is selected.

Platform 0.1.12's `cdk` feature exports `certified_data_set` and
`data_certificate`, resolving the host's final missing APIs
([Platform #21](https://github.com/dragginzgame/ic-platform/issues/21)). The host's
runtime calls and lifecycle/query/update macros now all use `ic_platform::cdk`;
both direct CDK declarations are removed and the direct policy refuses them.
The upstream runtime, controller Boolean/panic behavior and executor identity
remain unchanged. Hidden/custom Wire codecs preserve byte admission, payloads,
errors and traps. No certificate, codec or runtime wrapper is introduced.

The runtime bundle includes Platform Candid; this is permitted only on the host
graph. It does not add Candid protocol methods or relax schema/library exclusions.
Counter-zero measurements continue using the exact raw API, equivalent to CDK's
conversion of the same constant.

`make check-boundaries` runs the adopted Shared direct-dependency checker with
`ci/cargo-dependency-boundaries.json`. It checks canonical package identities,
including aliases and normal/build/dev/target/optional declarations, independently
of enabled features. Transitive library checks permit principal-only Platform
while excluding Candid and runtime dependencies from pure/portable/session
selections as appropriate. Refusal fixtures cover Candid and executor leakage
into the passive protocol graph. The reference-Wasm graph
also rejects host parsers/native tooling and requires one executor identity.
Private Wire schemas reject Candid/CDK/tooling on native and Wasm. The reference
host permits Platform's Candid capability as part of its explicit runtime bundle;
private Wire schemas reject that feature as well as Candid package leakage.
Certified verification still reaches upstream Candid transitively; CDK macros and
executor remain in the qualified host. This is ownership control, not Wasm removal.

The root catalog was reviewed separately: the Shared checker does not inspect
unused catalog entries or infer exclusions from role comments. There is one
workspace/lockfile and no independent maintained fixture workspace. These checks
still do not bind another application's deployed artifact to its source/lockfile;
consumer deployment admission and native macOS qualification remain separate.

## Setup

Install rustup through your normal host setup, then from this checkout:

```sh
rustup show active-toolchain
make install-tools
make tools-check
make install-hooks
make fetch
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
```

The first command prepares the toolchain, rustfmt, Clippy and Wasm target selected
by the toolchain file. Common system prerequisites, exact executable pins and
installation behavior are described in the shared [local setup](local-setup.md).
The snapshot owns the common host/Cargo and IC catalogs. Make, standalone setup,
formatting, CI cache identity and failure evidence use those same reviewed pins.
There are no local catalog exceptions; host, IC and Rust override selections are
independent. Compiler/MSRV and locked Testkit CLI/server selection stay local.
After catalog adoption, explicitly run `make install-tools` and `make tools-check`.
The ordered aggregate prepares common host/Rust tools, the minimum compiler and
Testkit. Auth has no product caller for the separate IC executable bundle; it is
not installed, checked or placed on PATH by the aggregate or CI. Its explicit
`install-ic-tools` / `ic-tools-check` commands remain available, with earlier
installations retained.

The Makefile includes reviewed [`make/consumer.mk`](make-interface.md), which owns
common setup, formatting, hooks, snapshot verification and release routing.
`check-snapshot` retains its public name and depends on `shared-tooling-check`.
The recorded snapshot pins Shared Tooling 0.24.0 at
`cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`. Adopt its committed source and
recorded entrypoint closure through the canonical exporter; see
[snapshot adoption](consuming-snapshots.md#entrypoint-selection-in-070).
The canonical verifier also supports `--read-manifest FILE`: it validates the
complete captured syntax before emitting records, rejects NUL/extra fields, and
performs no payload/ownership admission in this mode. Normal snapshot checks
still verify bytes/modes. See [reader semantics](consuming-snapshots.md).
Shared 0.9.10 rejects non-boolean Cargo `net.offline` values before tool dispatch.
Its changed build policy selects fresh tool receipts: explicitly run
`make install-rust-tools` and `make install-testkit-tools`, then `make tools-check`.
Existing installations and evidence are retained. CI uses the canonical Cargo
build cache identity, restores shared builds before setup and saves admitted
builds on main pushes before tests. Pull requests only restore; cache availability
never bypasses full setup, offline admission or the three-host validation matrix.
The [change-reporting rules](../rules/change-reporting.md) govern final maintainer
handoffs, including scoped zero counts and separate symbol deletion/move/rename
classification.

The new [Cargo-tool freshness task](../tasks/cargo-tool-versions.md) is included
with the catalog; its online checker runs from Shared Tooling and is separate
from Auth's offline admission and CI. No schedule is enabled by adoption.
The shared LOC helper also supports machine-readable workspace counts:
`bash scripts/dev/cloc.sh --json "$PWD"`. Its package rows and totals use numeric
fields; `make cloc` retains the human table. Fleet aggregation remains Shared-owned.
Auth owns its tool extensions, application checks and release metadata adapters.
The shared aggregate admits the selected Rust/Cargo toolchain before setup
downloads, then prepares the common host and Rust toolsets in order. Its
read-only preflight disables
automatic Rustup installation; prepare an unavailable compiler explicitly.
Host-tool failures identify the selected tool, version, path and repair command
without executing unauthenticated payloads. IC Auth adds its minimum compiler/Wasm target, then locked
Testkit CLI/server through `LOCAL_TOOL_INSTALL_TARGETS` and
`LOCAL_TOOL_CHECK_TARGETS`. These ordered extensions stop on failure even under
parallel Make. Direct host-installer callers remove `--with-ripgrep`/`--with-cloc`;
all four host tools are required. Explicit setup reuses matching installations
and retains earlier bundles and build evidence. `make install-msrv` prepares the
two declared minimum compilers (and host tool prerequisites);
`make msrv-tools-check` inspects its prepared versions.
Installation is explicit; checks run
offline. `fetch` prepares the selected lockfile without changing it.
Shared includes own standard release dispatch and single-workspace formatting;
IC Auth retains its direct delivery policy, cache preparation, metadata adapters
and complete validation roster. Formatting checks use the prepared checkout-local
tools without installing them or activating Git hooks.
Shared Tooling 0.6.6 selects cargo-sort-derives 0.14.1 for its literal `$crate`
macro-path fix. Run `make install-rust-tools`, then `make rust-tools-check` when
adopting that pin. Auth's formatter uses grouped Cargo manifest sorting and
rustfmt; the common tool selection does not enable derive sorting locally.
The release/format includes also load the selected `make/execution.mk` companion,
which rejects ignore-errors, dry-run, touch and question modes before recipes.
IC Auth-owned Cargo recipes and wrappers preserve Make's inherited jobserver
descriptors with recursive recipe marking. The same execution guard refuses
unsafe modes before installations, builds, publication or metadata effects;
callers do not replace jobserver flags or select a separate parallelism budget.
Shared Tooling 0.3.2 supplies the corresponding formatter, Rust installer/check
and LOC recipe handoff under
[Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99). Its tool
include also admits Make execution independently, so standalone tool consumers
retain refusal of unsafe modes. Shared fixtures require explicit completion
before reporting success or deleting their evidence, including on Bash 3.2.
The validation runner also rejects malformed nesting depth before dispatch and
requires explicit completion before success. Shared Tooling 0.3.7 also enforces
mandatory portable assertions explicitly on Bash 3.2, including selected Cargo
tool activation. The formatting-adoption checker refuses premature completion
under [Shared #103](https://github.com/dragginzgame/shared-tooling/issues/103).
Routine CI cancels older queued and running checks for the same workflow/ref;
unfinished hosts on cancelled revisions remain unqualified. The retained run
still requires the complete Linux and macOS matrix.
The adopted Shared snapshot resolves Cargo tool setup and offline checks from
the selected consumer directory, using its prepared compiler. Tool builds use
the standard environment; application Cargo configuration remains local.
Live diagnostics and retained error summaries now share one classification,
including early fatal, LLVM and missing-file failures.
Shared Tooling installs common Rust tools through its existing versioned
Cargo slot/receipt engine. Run `make install-rust-tools` explicitly after adoption,
then `make rust-tools-check`; common PATH activation requires all three pinned
tools to pass. Earlier in-place installations, completed slots and failed attempts
remain retained. This contributor setup change preserves IC Auth's published
contract and uses patch notes. Shared Tooling 0.5.0 now shares verified host and IC
bundles across checkouts through its per-user store. Common host admission uses
`make install-host-tools` and `make host-tools-check`; the separate IC commands
apply only when explicitly selecting those optional executables. Existing PATH entries remain valid; earlier physical bundles stay
retained but are no longer admitted as active installations. The default store is
`${XDG_DATA_HOME:-$HOME/.local/share}/shared-tooling/tools`; use the same absolute
`SHARED_TOOLING_STORE` override for setup, checks and failure collection if needed.
Shared Tooling 0.6.0 also shares Cargo tools under exact compiler, native environment,
package and packaged-lock identities. Run `make install-rust-tools` and
`make rust-tools-check`, then `make install-testkit-tools` and
`make testkit-tools-check` after adoption. Common PATH stays
`.tools/rust/common/bin`; selected CLI callers use the installer's checked return
path. Earlier local slots, complete sets and failed attempts remain retained.
Tool builds exclude application flags/wrappers/configuration and use two jobs;
explicit offline settings still apply. The managed source cache is needed for
offline checks. Failure collection keeps local Rust evidence and only its
referenced shared generations, including failed build output. Testkit's server
storage remains local and Testkit-owned. See
[Shared storage](local-setup.md#shared-executable-storage).
Shared Tooling 0.6.1 corrects failure propagation through common Cargo setup.
Run explicit Rust/Testkit setup and their separate offline checks after adopting
this revision: the changed build-policy identity selects new shared generations.
Lock, receipt and activation failures stop setup with failure evidence retained;
checks never install missing tools or bypass a foreign lock.
The optional IC toolset includes Binaryen 133; `make install-ic-tools`
explicitly prepares that unselected bundle. This repository builds its qualification Wasm directly
with Cargo and has no optimized product bundle to qualify.
The release and tool-evidence fixtures copy that companion and its execution
probe explicitly; qualification uses disposable checkouts and substituted effects.
Admission resolves that probe relative to the selected include, independently of
ambient tooling roots, and uses the running GNU Make executable when recursive
Make commands carry extra arguments.

Make selects checkout-local tools automatically. The shell export is needed for
interactive direct commands. Builds stay under this checkout's `target/`; Cargo
tool installation retains failed build attempts under `.tools/rust/build/` and
successful Testkit CLI builds in their versioned `.tools/rust/` selection slot.

Copied-workspace experiments must use their own `CARGO_TARGET_DIR`, for example
`target/<experiment>/cargo-target`, rather than this workspace's `target/`.
Cargo can reuse a copied package's executable when package identity and relative
source paths match; that executable may contain experimental fixture inputs.
If outputs were shared accidentally, retain the executable and dependency metadata
as evidence, run `cargo clean --offline -p ic-auth-qualification` in this workspace,
then rerun `make test-qualification`. This clears only the native runner package;
it does not reset canister storage or repair an incompatible retained ledger.

The reviewed pre-commit hook formats selected files without building or testing,
rejects partial staging and preserves unrelated edits. Hook activation does not
establish independent formatting qualification.

## Browser client setup

The private [browser client](browser-client.md) has its own locked npm build root.
Select exact Node from `packages/client/.nvmrc` and npm from that package's
`packageManager`, then run `make install-client-dependencies` and
`make client-tools-check`. Preparation uses `npm ci` with lifecycle scripts
turned off. Validation does not install dependencies. CI prepares these selections
explicitly on all three hosts before its complete gate. Client source and example
outputs remain under ignored `packages/client/dist/`; generated-contract evidence
is retained under `target/browser-client/`.

`make test-qualification` also requires these prepared client dependencies. It
compiles current TypeScript before invoking the SDK logout adapter through the
live Testkit gateway. Missing or mismatched tools refuse; checks never install.
The Node journal uses fake IndexedDB, so this does not replace native browser
storage qualification. See the [transport evidence](ic-testkit-qualification.md#sdk-logout-transport-against-live-receivers).

## Validation

| Focused command | Purpose |
| --- | --- |
| `make test-client-tools` | Actual CI pipeline and Make callers: selected Node/npm path, later-step admission and first-failure controls with substituted npm effects |
| `make test-client` | Portable admission/logout, strict journal transactions and Rust/TypeScript Wire roundtrips |
| `make check-client-contracts` | Compare fresh locked bindings with the Rust-owned checked-in contracts offline |
| `make generate-client-contracts` | Explicitly regenerate the checked-in client contracts from Rust DTOs |
| `make test-types` | Public types, bounded Wire codecs and private fixture projections |
| `make test-protocol` | Canonical encoding, signed vectors, ordering and size rejection |
| `make test-portable-signatures` | Portable Ed25519 issuer proofs with protected audience/issuer/key binding, independent signature vectors and strict malformed/weak-key refusal, including an unpublished consumer enabling permissive Dalek parsing through feature unification; this does not admit sessions |
| `make test-chain-key-signatures` | Isolated SEC1 admission, explicit producer normalization and exact prehash verification, including malformed material and strict high-S refusal |
| `make test-signatures` | Real BLS-signed IC fixtures, signer/seed/root rejection, certificate freshness and bounded decoding |
| `make test-signature-store` | Bounded preparation/retrieval, unchanged retry deadlines, pruning and real BLS verification with host-composed branches |
| `make test-certified-proofs` | Portable certified secp256k1/BLS proofs, protected delegation bindings, budgets and rejection |
| `make test-sessions` | Atomic admission/replay failure, exact retry, live authority invalidation, logout and quotas using certified and Ed25519 portable proofs |
| `make test-host-tooling` | Native file-operation CLI: bounded input, exact digest output, no-follow reads and preserved create-only evidence |
| `make check-msrv` | Isolated public package payloads and internal applications using the declared minimum compiler, with native/Wasm and feature coverage |
| `make tasks` | Read the adopted maintenance task catalog; no task, agent or schedule starts |
| `make test-qualification` | Build the internal Wasm host; verify real certificates, portable proof/session admission, live authority rejection, root composition, upgrades, authenticated ingress and SDK logout through `ic-testkit` with the checked server and prepared client dependencies |
| `make build-qualification-canister` | Build the internal host's release Wasm without starting a server |
| `make check-wasm` | Compile-check the public libraries and chain-key/signature verification/preparation, token and session capabilities for `wasm32-unknown-unknown` |
| `make clippy` | Lint the libraries, native tooling/qualification and Wasm host with their selected targets |
| `make metadata` | Validate the selected locked graph offline |
| `make check-boundaries` | Inspect capability graphs; reject Candid in default, Ed25519 and local-session selections |
| `make fmt` / `make fmt-check` | Shared manifest sorting followed by Rust formatting |
| `make check-snapshot` | Verify the adopted shared files and modes |
| `make check-dependency-pins` | Check declarations, lockfile tracking and root inheritance |
| `make check-doc-links` | Check local Markdown targets |
| `make publish-dry-run` | Build all three distributable libraries without uploading; requires registry metadata access |
| `make install-release-tools` / `make release-tools-check` | Explicitly prepare the lock-selected IC Build editor, or admit and print its absolute path offline |
| `make test-release-tools` | Exercise release/recovery and publication rejection/retry using local and mocked effects |
| `make test-release-runner` | Adopted Shared Tooling runner regression fixtures with substituted release effects |
| `make test-evidence-archive` | Adopted evidence archive fixtures: retained files, symlinks and literal paths |
| `make test-tools-evidence` | Actual setup/check caller failure status and adopted collector archive bytes, with Cargo installation substituted |
| `make test-testkit-tools` | Locked CLI selection, setup/check failures and selection changes, prepared-input drift refusal, qualification path and actual CI evidence archive with substituted effects |
| `make test-make-jobserver` | Actual owned Make recipes preserve pipe jobserver descriptors through Cargo/wrappers; unsafe modes refuse before substituted effects |

`make ci` is the complete configured CI/release gate. Ordinary development and
handoff use the relevant focused checks above; local full-gate execution follows
[Shared Tooling validation policy](../DRAGGINZGAME.md#validation-and-evidence).
It uses the Shared Tooling 0.6.6 logger and retains failures. CI declares the
root Cargo manifest/lock, Rust compiler selection, common host/Rust and IC pins,
and browser package manifest/lock/Node selection as prepared inputs. Their byte
identities are checked before and after each stage. A change, deletion or read
failure stops later dispatch, preserves an original stage failure and reports
which inputs require fresh preparation and the full gate. Nested gates in the
same checkout retain the checkpoint; disposable checkouts establish their own.
This detects drift at stage boundaries, without isolating arbitrary source edits
inside a running stage. Checks never fetch, reinstall or silently restart.
The workflow prepares tools and caches first, then calls that same gate. The public remote is
[dragginzgame/ic-auth](https://github.com/dragginzgame/ic-auth), and `main` contains
the current release. Local and hosted evidence for the current batch is recorded
in the [handoff](status/current.md). Dependency pinning requires each lockfile and
the npm root manifest to be tracked.
Cargo tests, metadata and Wasm checks run offline after `fetch`. The publication
dry run deliberately accesses registry metadata: the selected Cargo's offline
multi-package check fails when a staged dependency has no resolved checksum.
It never uploads packages. It is part of the complete CI/release gate; release
logs remain under `.git/release-state/validation-logs/`.
Each invocation retains a fresh staging/build directory under
`target/publication-dry-runs/`. This gives Cargo's temporary registry a distinct
identity, preventing earlier same-version source from being reused after edits.
Existing package staging and Cargo caches are preserved.

Once this workflow is delivered to `main`, an explicitly authorized qualification
of another branch or a tag-only release can dispatch the same workflow at that ref:

```sh
gh workflow run ci.yml --repo dragginzgame/ic-auth --ref vX.Y.Z
```

Dispatch requires the workflow on the default branch and its `workflow_dispatch`
trigger at the selected ref; historical tags without that trigger cannot acquire
it from later source. Check the resulting run's `headSha` against the intended
commit, and require all three native jobs. A PR run qualifies its merge candidate,
not a different release commit. Existing 0.2.0 branch/tag runs retain their original
source and qualification status. Tag identity, validation receipts
and exact pushed-tag admission remain in the release/publication scripts.
Manual dispatch runs validation only; it does not release or publish packages.
See [GitHub's event rules](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)
and [workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax).

CI stores validation logs and local fixture evidence under
`target/portable-fixtures/`. Failed jobs use the adopted retention action and
archiver; downloaded artifacts contain `evidence.tar.gz`. Extract that archive
to inspect the retained files. This layout does not erase prior evidence under
`target/release-tools.*/`.

CI's `scripts/dev/ci-tools.sh install|check` invokes the real aggregate targets,
preserves pipeline failure status and saves their output in
`target/rust-tools-install.log` and `target/rust-tools-check.log`. Both logs and
the complete physical `.tools/rust` tree are selected by the adopted failure
collector without following child links, including failed attempts and earlier
slot receipts/builds. The native
matrix also injects a failed Cargo installation into an isolated consumer, checks
offline refusal, uploads through the same action and downloads that exact artifact
ID to verify retained bytes. Readback uses the current repository/run and the
job's read-only Actions token, refuses digest mismatch and independently verifies
the extracted payload. Local archive checks do not establish hosted upload
or macOS qualification before those jobs run.

The snapshot deliberately omits fleet reporters. `make cloc` still reports this
workspace; the optional shared `cloc-tooling` target explains that fleet reports
run in Shared Tooling and does not execute another checkout.

Signature tests exercise real cryptography with deterministic fixture trust keys,
including root/subnet chains, and preserve Canic's signature-domain bytes. They
do not establish live network trust,
session admission, PocketIC behavior, service upgrades or independent consumer
adoption. Token tests additionally verify root batch ECDSA signatures and issuer
BLS certification together with caller, narrowing and current authority inputs;
see the [signature](signatures.md) and [delegation](delegation.md) contracts. Session tests
cover admission and authorization with a volatile backend and a failing staged
transaction; they do not qualify stable canister storage or lifecycle. The
[session contract](sessions.md) specifies the host's durability obligations.
Preparation tests additionally cover host-owned certification composition and
bounded retrieval retention; see [the preparation contract](signature-preparation.md).
They do not qualify an actual canister's root publication or query/upgrade lifecycle.
The separate [IC Testkit command](ic-testkit-qualification.md) exercises those host
paths, portable certified credentials and real signed ingress. It consumes the locked Testkit CLI's offline
admission path and retains state beneath `target/`.
It does not qualify browser/wallet login or a durable session backend.

## Releases and publication

Fresh release preflight includes [release editor setup](#release-editor-setup).
Exact recovery uses the already prepared saved-source selection offline.

```sh
make release-patch                 # compatible patch increment: 0.4.4 -> 0.4.5
make release-minor                 # breaking pre-1.0 contract: 0.4.4 -> 0.5.0
make release-major                 # explicit major decision: 0.4.4 -> 1.0.0
make release-resume VERSION=X.Y.Z   # reconcile the exact saved attempt
make publish-dry-run               # validate packages without upload
make publish                       # upload a clean, delivered tagged release
```

Released 0.4.0 carries the diagnostic hard cut at the required minor boundary.
Released 0.4.1 adopts Shared Tooling 0.5.0 without changing the library contract.
Released 0.4.2 adds pure [token preparation](delegation.md#retired-delegated-token-lifecycle), adopts Shared
Tooling 0.6.0 and qualifies native dependency updates. Released 0.4.3 fixes browser
flight result validation. Released 0.4.4 adopts Shared Tooling 0.6.1's setup failure
propagation and qualifies the incoming native selections. Released 0.4.5 adopts
Shared Tooling 0.6.3's restart of committed preparation. Existing Rust APIs,
signed bytes, stored formats and consumer build requirements remain unchanged.
Release execution still requires an explicit request for the selected command.

The [shared restart contract](releases.md#source-changes-after-saved-preparation-intent)
applies to IC Auth's direct delivery. Its preparation adapter writes only local
Cargo manifest/lock, changelog and validation receipt, with the manifest last;
Git staging and delivery remain runner-owned. At unchanged HEAD, interruption
still resumes the exact saved version. Once the maintainer commits the disposition
of that metadata in a clean descendant, a standard target archives the old
`prepare` plan unchanged and selects its requested increment from current metadata.
It repeats setup/admission and the complete gate before preparing again. Old
source/candidate-bound validation receipts remain in `.git/release-state/` and
cannot qualify the new source. No separate preparation journal or retirement
helper is added. Explicit `release-resume`, dirty/divergent source, changed
destinations and plans with staging or later effects keep exact-source refusal.
See [Auth #28](https://github.com/dragginzgame/ic-auth/issues/28) and
[Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126).

Released `0.1.13` adopts Shared Tooling 0.1.36's selected Cargo installer and
receipt/path/failure-status checks, dependency-preparation guidance and Host
Tooling 0.8.9. Product CLI selections and profiles remain consumer-owned;
that release preserved its then-current PocketIC setup. Released `0.1.14` documents
the [Canic library adoption contract](canic-adoption.md) and qualifies its token
issuance Candid envelopes in both directions. Released `0.2.0`
extends that qualification to complete tokens and root/issuer proof transport.
It also adopts Shared Tooling 0.2.0's breaking PocketIC ownership handoff. Run
explicit `make install-tools` after updating: the shared IC bundle now holds
five tools, and the locked Testkit CLI owns server setup and offline admission.
`make install-testkit-tools` prepares only that CLI/server route;
`make testkit-tools-check` returns the admitted absolute server path without
downloading. A lockfile update that changes Testkit selects a new versioned CLI,
even when its PocketIC server is unchanged. Explicitly rerun
`make install-testkit-tools` before qualification; a missing CLI check reports
the selected package/version and preparation command, preserving its failure
status without installing or falling back to an older CLI.
Shared 0.3.7 owns the lockfile selection directly through the Cargo installer's
`--lockfile Cargo.lock` argument. The wrapper delegates parsing and CLI admission
to that owner, then re-admits the current CLI offline after Testkit's server
operation. A changed selection refuses, including when the newly selected CLI
is already installed. Checks never prepare a missing selection.

`make test-formatting-hooks` selects current Rust files/manifests and tracked
deletions through Shared Tooling, then checks actual staged formatting, partial-stage and
formatter-failure preservation, lockfiles, unrelated edits and hook installation
in disposable exports. It is included in the complete gate on all three native
hosts. Failed checker scratch stays under `target/portable-fixtures/`; successful
scratch is removed. The fixture exports the current qualification sources too.

Complete validation checks the prepared tools immediately after snapshot
integrity. Standalone qualification admits Testkit before dispatching the Wasm
build, with explicit sequencing that also applies to parallel Make.
`make test-testkit-tools` checks caller selection, failures and
offline qualification with substituted effects. Canic's actual adapter and
durable host qualification remain consumer-owned. Released
`0.1.12` added portable MSRV checking, retained setup evidence
and Shared Tooling 0.1.34 optional fleet selection/path handling, plus Host 0.8.8
streaming file identities and Testkit 0.25.4 runtime qualification. Released `0.1.11` added
Host Tooling 0.8.5 and opt-in transactional browser storage. See
[the handoff](ic-tools.md#pocketic-ownership-handoff) for the new server owner and
retained-bundle disposition.
Released `0.1.8` added qualified minimum-compiler
coverage and Shared Tooling 0.1.28 adoption. Existing encoding, signature, token
and session APIs and signed bytes are unchanged. Released `0.1.7` added IC Testkit
qualification; signature preparation was introduced in `0.1.6`. Native SDK/CDK
dependencies belong only to the qualification apps.
The earlier `0.1.2` rename used the maintainer's explicitly selected exception
to the usual pre-1.0 minor requirement; later changes follow the normal
[shared release contract](releases.md). Commit the implementation
before executing a release; only pending notes may be dirty at ordinary preflight.

All three release targets use the adopted runner, the same `make ci` gate and
direct delivery to `origin/main` by default. `RELEASE_REMOTE` and `RELEASE_BRANCH`
select another already-authorized destination. PR delivery is not qualified here.
`ci/release-delivery.txt` declares the supported direct delivery once; Make and
standalone metadata use the shared read-only policy checker. CI seals this file
with its other preparation inputs and refuses policy drift before later stages.
Preflight reports all refused staged, unstaged and untracked paths, quotes
unusual names, and distinguishes Git observation errors from dirty source.
Initial refusal identifies that this attempt has not started validation or
version preparation. Interrupted preparation retains its exact four-file
metadata allowance and receipt checks. Admission preserves files and index bytes.
Normal preflight fetches the selected lockfile, then runs `make install-tools`
and `make tools-check`. The canonical ordered aggregate prepares host and common
Rust tools before the minimum compiler and lock-selected Testkit CLI/server;
the second aggregate admits that selection offline. Shared tool
payloads alone do not activate a checkout's missing local references.
Explicit Cargo offline settings still apply; a failed preparation stops before
validation or version changes. Saved-receipt recovery admits interrupted metadata
without provisioning a potentially partial graph. Ordinary checks never install
tools. Preparation changes the root
version, the inherited internal
dependency requirement, all local workspace lock entries and the candidate notes.
It adds `release-validation.json`, binding the complete gate to the validated
source, lockfile, original notes and selected release identity. External dependency
selections remain byte-for-byte unchanged. The canonical root version is replaced
last so interrupted preparation can be repeated from the retained intent.

The runner stages only the owned release files, commits and creates an annotated
`vX.Y.Z` tag, then atomically pushes that branch and tag. It retains plans, logs
and build artifacts on failure and success. Rerun the same normal target for
recovery; explicit resume selects one saved version. Late checks read the selected
release commit even if HEAD contains later work. Releases do not publish crates.

`make publish` selects only `ic-auth-protocol-types` and `ic-auth`, in that dependency order,
and always targets crates.io. It requires a clean worktree, the exact annotated
tag at HEAD, a validated release receipt and the same tag object on the selected
remote. Cargo builds the distributable archives before dispatch. Each package
includes its MIT notice; the package-local copies must match the root license.
Authenticate using Cargo's normal credential provider/login before an authorized
publication; the script never prints or reads credential files itself.

Publication intent under `.git/publication-state/` fixes source, tag, registry,
version and archive checksums before upload. Successful matching registry
observations skip an identical upload. Registry errors and checksum conflicts
stop the run. Exact-version transport and metadata admission use the selected
Shared Tooling helper, requiring curl 8.4.0+ and the prepared jq. It bounds each
response to 1 MiB, rejects incorrect identity/schema and retains each observation
in a separate private directory beneath the publication log. HTTP 404 means
absent; all unavailable/invalid observations refuse dispatch. Auth retains the
archive checksum decision; matching yanked bytes reconcile prior publication
without another upload. An attempted upload without confirmation remains unresolved:
rerunning observes the exact version/checksum and does not redispatch an absent
version after an uncertain response. If it never appears, establish the failed
effect independently before reconciling the retained intent. Publication logs and
archives remain in `target/`; no cleanup, GitHub Release object or deployment is
implicit. See [Cargo publication](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
for credentials, dry-run and index-propagation semantics.

`scripts/dev/run-host-tooling.sh` runs the unpublished utility through locked,
offline Cargo with output in this checkout. The wrapper and release adapters
resolve their physical checkout without emitting inherited `CDPATH` banners;
digest stdout and NUL-delimited release paths retain their exact contracts.
File identities use
`ic-host-artifacts::artifact::Sha256Digest`; no-follow regular reads and durable
private-file publication use `ic-host-fs`. The caller supplies finite byte limits:
8 MiB for lockfiles, 64 MiB for package archives, 1 MiB for validation receipts,
64 KiB for publication intent and 1 KiB for dispatch markers. These are local
tooling limits, not authentication protocol limits. The publication lock and
registry uncertainty policy remain owned by the release/publication scripts.

The Unix-only `ic-auth-tooling hash-files0 MAX_BYTES` operation reads ordered
NUL-terminated paths from stdin and emits `hex_sha256<TAB>path<NUL>` records in
the same order, retaining duplicates and raw pathname bytes. It accepts at most
4 MiB of path input and 4,096 nonempty paths; an empty batch is valid. Each file
uses the caller's byte ceiling and Host's no-follow regular-file hash. Malformed
lists and failed files refuse before any identity output. This bounds the batch,
not concurrent source mutation: callers still own source custody and input seals.
Client contract generation builds the utility once and uses one batch per
before/after seal, retained as `inputs.tsv0` and `inputs-after.tsv0` beside the
path list. MSRV checks likewise keep their existing archive/source comparisons,
using one native invocation per sealing round rather than one process per file.

The native utility and qualification runner consume the root-selected published
`ic-host-fs` and `ic-host-process` packages. The utility uses
`durable::write_with` with explicit create/replace modes and owner-only
permissions; publication errors retain their typed before/after state and
secondary cleanup diagnostics. Host owns path admission, bounded file reads and
process execution. Auth owns release identity, publication policy and which
qualification commands are permitted.

The qualification runner reads Wasm through Host and obtains canister runtime
and bounded Candid-call helpers through Testkit. Its SDK qualification subprocess
uses Host process admission directly. Testkit owns server selection, startup and
its process integration; no compatibility shim is retained here. These dependencies
remain outside all three public library graphs. Run `make install-testkit-tools`
for explicit preparation, then `make testkit-tools-check` for offline admission.
Dependency selections live in the root manifest/lock; actual certification,
upgrade and ingress evidence belongs in [the current handoff](status/current.md).
Earlier dependency adoption observations remain in Git history and the changelog.

Receipt reconciliation uses the adopted portable checksum helper, because an
interrupted preparation can temporarily leave incompatible manifest/lock versions
and cannot rebuild the workspace utility. Normal validation and publication use
the Rust utility. Shared Bash tooling remains canonical for process dispatch and
Git release orchestration; no current Candid-extraction or process-capture call
justifies adding `ic-host-tools` or `ic-host-process` to this application.

The attempted `0.1.1` publication stopped before dispatch because the existing
`ic_auth_types 0.1.1` archive has a different checksum. That crate belongs to a
different project. Retain the old intent and logs; the rename requires a fresh
release and its own publication intent, not a replacement of the `v0.1.1` tag.
Registry-index absence for `ic-auth-protocol-types` was checked during the rename;
it is an availability observation, not a reservation of the name.

The focused fixture uses real version/lock transforms, a substitute complete gate,
local commits/tags/bare remotes and mocked Cargo uploads/registry observations.
It covers patch/minor/major, gate failure, staged source rejection, candidate
conflicts, interrupted preparation, exact resume, dirty/untagged publication,
registry errors, checksum conflicts and lost-reply reconciliation. This is
consumer adapter evidence; it is not a live release or registry upload.

## Public types and fixture transport

Released 0.8.0 removed Candid traits from public domain types. Released 0.9.0 added
the separate `ic-auth-wire` codec adapter and generates its TypeScript directly
through published IC Wire. `make test-types` checks its bounds, canonical frames,
checked identifier projections and local request/result contracts.

The private Wire controls in `apps/qualification/contracts/` are limited to
controller/certificate qualification. Bounded projections preserve checked
identifiers; there is no Candid selector or intermediate codec. This is not a
production consumer dependency. The generated
application codec has one owner in `crates/ic-auth-wire/`, with no Candid fallback.
`make generate-client-contracts` is the only artifact-update command; checks
compare generated files and retain input seals. See the
[consumer boundary](canic-adoption.md#wire-transport).
