# Development

The virtual root owns the protocol/types, Auth engine and Wire transport libraries,
unpublished native tooling in `apps/tooling/` and the reference host/runner in
`apps/qualification/`. The root catalog selects published Types and Platform, the
Host FS/Process, Build and Testkit packages from
[IC Tooling](https://github.com/dragginzgame/ic-tooling), allocation/collections from
[IC Storage](https://github.com/dragginzgame/ic-storage) and the Wire 0.15
codec/TypeScript generator. The lock and [handoff](status/current.md) record exact selections and evidence;
[session contracts](sessions.md) distinguish supported behavior from pending hosts.
Rust `1.99.1` is both the pinned development/release compiler and the common
package minimum, inherited from the root catalog by all members including the
internal qualification canister. Explicit setup prepares it once. See [MSRV coverage](msrv.md).
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
Current complete native acceptance is recorded in the
[handoff](status/current.md); that historical incomplete run is not the current gate.

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

### Normal checkout workflow

Make routine implementation and dependency changes directly in the normal Auth
checkout. Leave reviewable local changes and run the relevant focused checks.
Create a PR when requested or required by repository workflow. When a PR is
needed, use a topic branch in this checkout. Do not create contribution
directories, additional clones or worktrees for ordinary work or delivery.
Authorized releases start from clean `main` after the selected source has been
integrated through its authorized workflow. Shared's qualified frozen controller
captures that source under this repository's `target/release-frozen/BATCH`, with
Git objects/journals under `.git/release-state/frozen/BATCH`. This is a managed
delivery path, not an additional implementation checkout.

A branch does not isolate simultaneous sessions sharing a checkout. Coordinate
with active agents, editors, dependency updaters and builds before switching
branches or changing inputs. The captured source stays stable during release and
publication; later normal-checkout updates belong to the next batch. Source/index
checks, the complete admitted release plan and the input-drift guard remain
required inside the candidate. Do not clean `target/` during an active batch.

The existing `make release-patch`, `make release-minor` and `make release-major`
commands still require release authority. Select one group with `RELEASE_GROUP`,
or the complete supplier-first roster through Shared's `RELEASE_GROUPS`.
Git-only delivery remains the default; registry publication is explicit:

```sh
make release-patch RELEASE_GROUP=engine
make release-patch RELEASE_GROUPS=protocol,engine RELEASE_PUBLISH_TARGET=publish
make release-batch-status BATCH=batch-XXXXXXXX
make release-batch-resume BATCH=batch-XXXXXXXX
```

Run the controller from an ordinary shell without `CARGO_TARGET_DIR` or other
Git/Cargo context overrides. Candidate hooks select their own target and prepare
their own formatting tools and browser dependencies; recovery checks saved-source
tool selections offline. The standard runner retains direct commit/tag/push and
the product's exact publication receipts. PR delivery remains unqualified.

The normal branch does not advance automatically. Preserve deferred work durably
before `make release-batch-reconcile BATCH=…`, which requires a clean branch and
fast-forwards only. Reapply deferred dependency edits deliberately afterward.
`make release-batch-retire BATCH=…` explicitly removes the selected completed
batch's managed checkout and recovery state; retain required evidence elsewhere
first. No local lifecycle wrappers or automatic stash/reset/merge exist.

For legacy ordinary journals, keep the original source, adapter and attempt state, then run
`make release-resume RELEASE_GROUP=protocol|engine VERSION=X.Y.Z`. Preserve
`.git/release-state`, publication intent, prepared tools and `target` evidence.
Do not reset, re-clone or clean away recovery state to retry. Normal later
releases start from current clean, committed metadata and run fresh admission;
historical plans do not supply their validation.

The former sibling capacity and 0.33 release checkouts are retired. Their HEADs
are preserved by local branches `archive/ic-auth-capacity` and
`archive/ic-auth-release-033`; both are already in `main`. Required historical
release/publication records, capacity evidence and selected final artifacts are
retained with checksums under `target/checkout-retirement/20261104-siblings/`.
These are historical observations, not new recovery admission. Reproducible
source/build trees were removed; no full checkout archive belongs in `.git`.
Explicit `cargo clean` may remove the entire target, including retained evidence.

### Editor preparation and recovery

Saved-source release editor admission uses a 60-second read-only contention wait.
Busy or timeout retains status 75 and the saved release; it never installs tools.
Missing/corrupt selections remain failures. Ordinary `release-tools-check` stays
immediate; `scripts/dev/release-tools.sh check-wait` selects bounded waiting.

Shared 0.25.1 delegates Cargo manifest and lock rendering to the released IC Build
editor. The selected Build 0.63 binary requires `cli`; Auth records its executable
features in `workspace.metadata.ic-auth.release-editor-features`, separately from
its feature-free library dependency. Auth's normal release preflight prepares and
admits the selected editor after source and changelog admission, alongside the other required tools. The
adapter selects the absolute path internally; no shell export is required.

For offline checks outside a release, explicitly prepare the current selection:

```sh
make install-release-tools
make release-tools-check
```

The check prints the admitted path and never installs anything. Auth uses an
8 MiB per-input allowance. Recovery reads both member identities and the editor
selection, including executable features, from the validated Git source, so a
newer or partially prepared live manifest/lock cannot change either. Recovery
and checks only admit the already prepared saved-source editor offline; they never install or rebuild it. The
manual setup target uses today's lockfile; `scripts/dev/release-tools.sh` also
accepts an explicit saved `LOCKFILE` and `SOURCE_MANIFEST` for deliberate historical
tool preparation. Sources before CLI feature gating have no feature declaration;
their existing feature-free editor selection stays valid.
CI prepares and checks the editor before release fixtures. No release/publication
is performed by either setup/check command. The old Perl editors and Auth's
manifest rewrite wrapper are retired; application APIs and package versions are
unchanged.

## Crypto dependency boundary

Auth selects Platform's independent `ecdsa` capability for chain-key
verification and `ed25519` for portable signatures. Original K256/Dalek types and
cryptography stay upstream. Auth owns prehash construction, high-S refusal,
explicit producer normalization, canonical-scalar checks and verification policy.
There are no direct K256 imports or declarations; a test-only Dalek feature
trigger remains in the unpublished adversarial harness. SHA-256, CBOR and LEB128
remain direct general-purpose dependencies with current Auth-owned consumers.
Platform 0.17 retains K256 0.14 and Dalek 3/signature 3. Auth retains strict
verification, canonical-scalar admission and explicit high-S refusal, and the
adversarial harness enables permissive parsing on that same Dalek generation.
The Wasm graph admits two support generations for BLS/Principal hashing and the
new RustCrypto stack; Wasm curve/signature providers still have one identity. The
boundary fixtures refuse a third support generation or duplicate K256.
The repository's Wasm target flags select upstream RustCrypto's qualified
`cpubits="32"` backend. Other canister build owners must select it themselves;
dependency Cargo configuration is not inherited, and explicit `RUSTFLAGS` or
`CARGO_ENCODED_RUSTFLAGS` overrides must preserve this selection.
Testkit 0.86 owns native clients and DTOs in `ic_testkit::pocket_ic`, together with
bounded server startup, lifecycle extensions and typed Wire calls;
only the native runner selects these capabilities. Its client requires an
explicit server endpoint; Auth's existing managed `build_instance` supplies it
and retains the admitted server handle.

Native file and process callers select Host FS/Process 0.74.2 with defaults
disabled. Library use does not enable Host's optional CLI, certificate decoder
or Wasm text/encoding backends. Host stays outside the production library graph.
The locked Build 0.74.3 and Testkit 0.86.0 share Auth's Host 0.74 identities. The independently
pinned file CLI is selected at 0.74.2.
Build and Testkit libraries keep `cli` disabled; only the prepared executable
selections request it. Testkit owns its benchmark-average errors and has no
Metrics dependency; Auth's diagnostics select Metrics 0.29.1.
No caller enables optional native
metadata; the old Host 0.73 and Metrics 0.25 providers are removed.

Auth and Testkit 0.89 share Platform 0.23 and canonical Types 0.18.
Native Testkit and its Candid/management DTOs remain outside the public library
and Wasm host graphs.
The host selects Runtime 0.19 directly, matching Storage 0.68's Wasm dependency.
Wire 0.17 provides the matching canonical Principal schema implementation.
Native calls retain explicit external Principal conversions; native SDK packages
remain outside public Auth and the reference Wasm graph.

The unpublished qualification host selects IC Metrics 0.28.2 for fixed
admission outcome counters, with defaults disabled: arithmetic and validated Serde
do not select its optional native Wire metadata. Public Auth graphs forbid that
dependency. See the
[host-owned diagnostics reference](sessions.md#host-owned-admission-diagnostics)
for event admission, commit/rollback and diagnostic reset policy; this adds no RPC.

The native Auth CLI uses Clap for its client-contract generation and external-lock
commands. Generic file operations now use the separately prepared published
`ic-host-file`, pinned independently in `ci/host-file.env`. There is no local
implementation or fallback. See [release and file-tool mechanics](#releases-and-publication)
for the existing byte/path limits and release-recovery boundary.

## Dependency roles and deployment admission

Shared Tooling 0.31.0 at `74313941af2fce4399bec0e6909ca6229673c3f8` is adopted
through the canonical snapshot. This identifies reviewed committed source, not a
claim of upstream publication or a matching release tag. Auth retains Cargo-owned versions
and the default release mode; Shared's pending canonical `VERSION` workflow does
not add a version file, prepare-work command or package-version edit here.
The root catalog uses `Production dependencies` and `Tests and tooling only`;
member annotations distinguish the three production libraries from internal
qualification and native tooling packages. These roles describe graph use, not
registry publication. Types is the passive identity owner; Platform is an optional
production entry selected by crypto capabilities and admitted per graph.
Certified libraries select BLS and admit upstream Candid/raw IC0 dependencies
under the approved Platform 0.22 certificate exception. Default/session-only,
wallet-signature and passive contracts retain their stricter graph boundaries.
Only the runtime host selects Runtime IC0/executor.
Native Host, Build and Testkit selections remain
tests/tooling entries.

`make fmt`, `make fmt-check` and the pre-commit hook use Shared's Rust formatting
driver: grouped Cargo sorting followed by Rustfmt, with progress and timing per
pass. Auth has no paired formatter override or extra workspace roster. The driver
can reuse unchanged inputs when its source discovery supports them; Auth's
explicit module paths disable that shortcut, and staged hooks always run the
formatters. `FORMAT_RUST_ARGS=--no-cache` explicitly runs every pass.
Release preparation writes that same ordering; current, prepared and
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

Its `check-candid-boundary` prerequisite also selects Shared's `--cbor` source
mode across `crates/` and `apps/`, with no allowances. Cargo's `--external-ic`
check uses the canonical CBOR dependency inventory; the browser SDK guard checks
the client and generated sources. The root is this repository's only maintained
Cargo workspace. These checks are lexical/dependency ownership evidence, not
macro expansion, copied-parser detection or proof of external packet identity.

There is no production service or deployment command here. The Storage/runtime Wasm
host is qualification-only and legitimately uses test/tooling packages. A Canic or
future wallet-service deployment must check its own exact target/features and
bind source, lockfile and successful build evidence to the final artifact digest.
This repository's graph checks do not enforce that external deployment binding,
validate role comments mechanically, or qualify unlisted feature combinations.

## IC Platform boundary

The protocol package selects published `ic-platform-types` 0.18.0 directly for
canonical Principal and parse-error identities, without a facade, external
Principal, SHA-2, Candid or runtime edge. The Auth engine selects Types'
`principal-self-authenticating` for browser-key hashing. Its Platform dependency
is optional: only crypto capabilities select it, while default and session-only
graphs exclude the facade. Consumers must align on Types 0.18 and Wire 0.16.
Serialized bytes are unchanged.
Direct dependency and resolved graph policies enforce the owner boundaries. Convert
explicitly at native Agent/Testkit boundaries using Platform's
`principal::ExternalPrincipal` with `principal-external` selected. Platform 0.18
canister-signature keys instead use the canonical owned Principal directly.
Significant bytes, ordering and binary serialization remain compatible; text
longer than 63 bytes is now refused before normalization. See the
[consumer migration](canic-adoption.md#platform-015-migration). No storage migration
or replacement cryptography is introduced.

Private controls and diagnostics now use bounded Wire contracts under
`apps/qualification/contracts/`. Shared checked configuration and the admission
recorder have one unpublished library owner in `apps/qualification/fixtures/`;
both host and native runner import it through Cargo. The shared fixture's unit
test runs under `test-qualification`, Clippy and MSRV alongside those consumers.
Application methods remain in `ic-auth-wire`.

Native qualification retains state and logs under `target/portable-fixtures/`.
PocketIC's Unix socket path limit requires short transport scratch under
`/tmp/ic-auth-pic.*`; the launcher records that path with the retained state,
removes its own scratch after success and preserves it on failure. This is a
transport-only exception; build output and canister state stay in the repository. After preparing
the canister with `make build-qualification-canister`, select a native case with
`bash scripts/dev/test-qualification.sh <test-name> --exact`; omitting the filter
retains the complete native signature target used by CI.

The runner uses Testkit's opt-in typed Wire calls for ordinary
requests/replies, with raw calls retained for malformed inputs and explicit byte
evidence. Init and post-upgrade arguments also use explicit Wire method identities.
Platform entrypoints with explicit codec hooks require no direct Candid declaration,
derive or service export. The host selects Platform
`cdk`/`certification`; the native runner selects `agent`/`canister-sig-creation`.
The host parser stays disabled.

The maintainer authorized retiring the unpublished fixture's Candid checkpoint.
Auth 0.22 introduced the bounded native-Wire checkpoint. Auth 0.25 uses
`AUTHNAT3` and identity
`0x41555449f101`, containing the new Wire session/replay/login records. Previous
`AUTHNAT2` (text-ID records), `AUTHNAT1` (CBOR records), `AUTHCELL`/Candid and finite-Wire `AUTHWIR2` payloads
are refused without migration or reset. The maintainer confirmed no retained Auth
records require preservation. The later Storage consolidation also changes
backing formats; start with a coherent new fixture checkpoint for this hard cut.
Checkpoint restores remain closed until matching controller authority is confirmed.
The unpublished qualification host selects Storage 0.68.0 for both allocation
and collections; `ic-memory` and `ic-stable-structures` are absent from the lock.
Storage owns `StableKey`, `StorageConfig`, namespace grants, backing and exclusive
leases. Its `StableCell::create` requires empty memory; `load(memory, maximum)`
admits Auth's 128 KiB payload plus magic before allocation and validates cell
framing. Storage 0.68 uses copied ICSS cell images without collection checksums;
Auth still checks its payload identity and bounded Wire records. Fallible
creation/writes are checked; host traps retain IC rollback.
No combined initialization or reset path is selected.

The historical Storage consolidation changed allocator and collection formats.
Old manager/ledger images are refused without fallback, initialization or clearing.
The native fixture freezes a published Memory 0.63 manager header and preserves
separate application sentinel bytes on refusal. Current Storage-format reopen,
occupied creation, detached-lease custody, size bounds and framing corruption
have explicit tests. Existing installations require a separately reviewed whole
backing transition; this adoption performs no reset or data retirement.

Storage adoption alone keeps the Auth payload and signed bytes unchanged.
The pending #127 hard cut separately replaces `AUTHNAT3` with `AUTHCLK1`,
retires the login-record/logout identities and refuses their old bytes. The
maintainer confirmed no retained affected records; no implicit reset is added. Storage 0.68 retires the previous collection images: use fresh allocation
for the private fixture; old cell framing is explicitly refused without clearing.
No production Auth store is migrated and no retained backing is reset here. Its
internal codec, Auth's contracts and Testkit now share Wire 0.17. No Wire values
cross Auth's Storage adapter boundary. The host retains one Runtime, generator
and runtime guard.
Storage's direct Wasm Runtime dependency resolves to that same maintained owner;
public library graphs still forbid runtime capabilities and Storage. The host
selects Metrics 0.28.2; Testkit's independent benchmark counters need no Metrics dependency.
No Metrics/Logging storage adapter is selected. See
[session restoration](sessions.md#storage-and-reset).

CDK runtime/controller behavior is retained ([Platform #1](https://github.com/dragginzgame/ic-platform/issues/1)).
No controller wrapper, executor, callback or crypto implementation is recreated.
Native Agent/delegated identities come from Platform, with native PEM support;
caller/root-key selection, request policy and authenticated SDK behavior remain
fixture-owned. Auth no longer needs Testkit's distinct Candid limits identity.

Certified Auth capabilities now select Platform's `canister-sig-creation`,
`certification` and `signature-verification` features explicitly. The `bls`
feature is used only by native test signing; it is not forwarded by Auth's public
features. All direct registry creation/verifier declarations and local crypto
implementations remain absent ([#16](https://github.com/dragginzgame/ic-platform/issues/16),
[#17](https://github.com/dragginzgame/ic-platform/issues/17),
[#18](https://github.com/dragginzgame/ic-platform/issues/18)).
Platform 0.22 delegates certification/BLS to published upstream implementations;
certificate/tree and signature/error types have upstream identities. Its key
adapter retains canonical Types 0.18.0 issuer identities. DER/CBOR bytes, Auth
resource admission and protected authority remain. Platform's adapters reject
identity trust-root and delegated keys, covered through Auth's verifier; native
Agent's separate external verifier does not inherit that correction. The
maintainer approved Candid and raw IC0 as transitive dependencies of these
certificate helpers. Platform exposes no upstream signature-store API; Auth's
store still takes host-supplied time/certificates and the host publishes roots.
Public libraries still exclude Runtime/executor capabilities.

The independent `ic-platform-runtime` owner's `cdk` feature supplies runtime adapters and explicit
codec entrypoints without depending on the full CDK or selecting Candid. The
host's runtime calls and lifecycle/query/update macros use `ic_platform_runtime::cdk`;
no local runtime wrapper is introduced. Explicit Wire hooks, including unit RPCs,
retain byte admission, payloads, errors and traps. Controller Boolean/panic
behavior remains. Runtime 0.19 owns its maintained executor and IC0 bindings,
with their upstream provenance; no separate registry executor is selected.
The retired no-op `hidden = true` option is removed from all fixture entrypoints;
explicit Wire decoding/encoding and endpoint exports remain unchanged.

Auth keeps runtime-specific call/error types internal and uses none of the
retired storage or `change_timeout` APIs. Its public Principal has canonical
Types 0.18 identity, reexported by Platform. Select `principal` when using the
facade alone; self-authentication and external conversions are separate capabilities.
Published Storage selects Runtime 0.19 directly on Wasm. Native Testkit shares
canonical Types 0.18 with Auth; it does not define Auth's public Principal.
Storage owns its backing/collection types; native Agent/Testkit boundaries retain
the upstream external Principal. No dependency override or new conversion is needed.
Counter-zero measurements continue using the exact raw API, equivalent to CDK's
conversion of the same constant.

`make check-boundaries` runs the adopted Shared direct-dependency checker with
`ci/cargo-dependency-boundaries.json`. It checks canonical package identities,
including aliases and normal/build/dev/target/optional declarations, independently
of enabled features. Transitive library checks admit upstream Candid/raw IC0
and registry signature packages only for the three certificate capabilities and
their all-feature composition. Default/session, wallet-signature, passive and
unknown future capabilities retain strict exclusions. The all-feature archive
consumer admits the same certificate dependency family. Refusal fixtures cover
Candid/raw IC0 leakage into non-certificate capabilities, explicit Platform
Candid/runtime features and executor leakage into public libraries.
Private Wire schemas reject Candid/CDK/tooling on native and Wasm. The reference
host admits the upstream certificate dependencies while rejecting parsers,
native tooling, full `ic-cdk`/macros and Platform's Candid/runtime features. It
retains one Runtime executor/guard/generator identity. Negative and positive
fixtures qualify this scoped exception. Graph changes alone establish no Wasm
size or performance result.

The root catalog was reviewed separately: the Shared checker does not inspect
unused catalog entries or infer exclusions from role comments. There is one
workspace/lockfile and no independent maintained fixture workspace. These checks
still do not bind another application's deployed artifact to its source/lockfile;
consumer deployment admission and native macOS qualification remain separate.

### Why Candid appears in Cargo.lock

The one workspace lockfile includes native qualification dependencies as well as
the deployed packages. The runner in `apps/qualification/runner/` selects
Platform's `agent` feature for real signed ingress, and Testkit's `wire` feature
for application calls. Candid arrives through three paths:

- Platform `agent` selects IC Agent, which depends on Candid and IC transport types.
- Testkit owns the native PocketIC client and its external transport/management
  DTOs, which depend on Candid.
- Platform 0.22's certificate capabilities select upstream canister-signature
  helpers that depend on Candid and raw IC0, including on Wasm. This is the
  approved certificate-only exception; default/session and passive graphs
  remain independent of these packages.

Auth uses Testkit's Wire call helpers. Testkit's native management helpers select
Platform's external protocol adapters; those native Agent/PocketIC paths stay
outside deployed graphs. Testkit 0.86 and Auth share Platform 0.22;
canonical Types and the actual Wasm Runtime owner stay
aligned. All Auth application requests remain Wire frames.

A workspace-wide reverse tree unifies Platform features across selected members,
so it can print Candid beneath shared Auth dependencies. Inspect isolated package
and target graphs to determine deployment dependencies. These checks are offline:

```sh
cargo tree --locked --offline --workspace -i candid
make check-boundaries
```

The first command explains the dependency paths; the second checks every supported
public library capability on native/Wasm and the qualification canister's Wasm
normal/build graph, rejecting Candid and its Platform feature where prohibited.

## Setup

Complete the [native/system bootstrap](local-setup.md#native-bootstrap-before-cargo-tools)
first, including the platform compiler/linker/SDK and required system utilities.
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
The ordered aggregate prepares common host/Rust tools and Testkit using the already
prepared compiler. Auth has no product caller for the separate IC executable bundle; it is
not installed, checked or placed on PATH by the aggregate or CI. Its explicit
`install-ic-tools` / `ic-tools-check` commands remain available, with earlier
installations retained.

The complete gate's preparation inventory is below. Native/compiler setup must
finish before any Cargo executable is prepared, including the release editor.
If that environment changes later, explicitly re-prepare affected tools; checks
must retain fingerprint refusal rather than installing or repairing implicitly.

| Prerequisite | Explicit preparation and offline admission |
| --- | --- |
| Native shell/system utilities and selected Rust compiler, Rustfmt, Clippy, Wasm target | Platform setup above and `rustup show active-toolchain`; shared supported-host requirements apply |
| Common host/Cargo tools | `make install-tools` / `make tools-check` (includes minimum-compiler, Testkit and file-tool extensions) |
| Lock-selected Testkit executable and PocketIC server | Included above; separately `make install-testkit-tools` / `make testkit-tools-check` |
| Selected file-operation executable | `make install-host-file` / `make host-file-check` |
| Lock-selected release editor | `make install-release-tools` / `make release-tools-check` |
| Locked Node/npm and private client dependencies | [Browser client setup](#browser-client-setup), `make install-client-dependencies` / `make client-tools-check` |

Client preparation disables npm lifecycle scripts and does not change compiler,
SDK or system-package inputs, so CI may prepare it before Cargo tools. Real
Chromium is an additional explicit prerequisite for `test-client-browser` and
`test-client-package-browser`;
the ordinary CI/release gate compiles that harness and runs Node SDK fixtures.
No native package installer or browser download runs inside an ordinary check.

The Makefile includes reviewed [`make/consumer.mk`](make-interface.md), which owns
common setup, formatting, hooks, snapshot verification and release routing.
`check-snapshot` retains its public name and depends on `shared-tooling-check`.
The recorded snapshot pins Shared Tooling 0.31.0 at
`74313941af2fce4399bec0e6909ca6229673c3f8`. Adopt its committed source and
recorded entrypoint closure through the canonical exporter; see
[snapshot adoption](consuming-snapshots.md#entrypoint-selection-in-070).
Adoption also compares every exported payload with an independently reviewed
commit using the trusted verifier's paired `--source` / `--revision` options;
ordinary builds retain offline local integrity checks. See
[snapshot verification](consuming-snapshots.md#drift-verification).
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
without executing unauthenticated payloads. IC Auth admits the selected compiler/Wasm
target and prepares the locked
Testkit CLI/server through `LOCAL_TOOL_INSTALL_TARGETS` and
`LOCAL_TOOL_CHECK_TARGETS`. These ordered extensions stop on failure even under
parallel Make. Direct host-installer callers remove `--with-ripgrep`/`--with-cloc`;
all four host tools are required. Explicit setup reuses matching installations
and preserves selected artifacts and required receipts. `make install-msrv` explicitly
prepares the one selected compiler, Rustfmt,
Clippy and Wasm target (and host tool prerequisites); `make msrv-tools-check`
checks the pin, actual compiler identities and Wasm target.
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
tools to pass. Earlier in-place installations and selected artifacts remain
protected under the [tool retention policy](tool-store-maintenance.md).
This contributor setup change preserves IC Auth's published
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
path. Selected local slots and complete sets retain their admission evidence.
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
interactive direct commands. Workspace builds stay under this checkout's `target/`.
Shared 0.27.26 fingerprints managed native build tools, headers and libraries,
so unrelated Linux application updates do not invalidate Cargo tools. Explicitly
rerun `make install-tools` and `make install-release-tools`, then their checks,
after adopting this changed build identity. Setup registers each checkout's current
requirements and discards successful tool compilation scratch after admission;
selected binaries, source and receipts remain available. Failed scratch has bounded
retention and recovery pins. Legacy generations remain protected until explicit
migration; executable reclamation requires quiescence and is separate from setup.
See [tool retention](tool-store-maintenance.md). This adoption performs no legacy
reclamation or workspace/release-artifact cleanup.

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

After changing the locked IC Wire selection, run `make generate-client-contracts`
and `make test-client`. Generation refreshes both the Auth DTO bindings and Wire's
supplied TypeScript runtime; unchanged DTOs do not imply an unchanged runtime.
The check compares all five files and refuses drift rather than rewriting source.
Generation and `make check-client-contracts` use Rust and the admitted file tool;
they do not require Node, npm or installed browser dependencies. `make test-client`
and live qualification admit those browser tools explicitly before using them.

`make test-qualification` also requires these prepared client dependencies. It
compiles current TypeScript before invoking the SDK logout adapter through the
live Testkit gateway. Missing or mismatched tools refuse; checks never install.
The Node journal uses fake IndexedDB, so this does not replace native browser
storage qualification. See the [transport evidence](ic-testkit-qualification.md#sdk-logout-transport-against-live-receivers).

For real Chromium qualification, prepare a Chromium executable separately and
refresh the locked client and Testkit tools with `make install-client-dependencies`
and `make install-testkit-tools`. Then run:

```sh
IC_AUTH_BROWSER_EXECUTABLE=/absolute/path/to/chromium make test-client-browser
```

For archive/public-API composition with real IndexedDB, run
`IC_AUTH_BROWSER_EXECUTABLE=/absolute/path/to/chromium make test-client-package-browser`.
This builds, installs and qualifies the actual `.tgz`, then checks admission,
logout invalidation and journal reopen/cleanup with injected transports. It
retains a `packed-consumer-chromium` receipt and browser executable/bundle hashes.
It does not need Testkit or a Wasm build. Both Chromium commands use isolated
profiles and never download a browser.

This focused, opt-in command uses locked Playwright Core and esbuild, builds the
reference Wasm, and runs one explicitly ignored native test. It downloads no
browser and uses an isolated profile, never a personal profile. The local HTTP
origin proxies only the two fixture receivers; the SDK pins the fixture IC root.
Logs record the actual browser version, executable/bundle hashes and retained
state path. Ordinary CI compiles this harness but does not launch a browser.
See [browser qualification](ic-testkit-qualification.md#real-browser-qualification)
for the injected issuer and clock boundaries.

## Validation

The complete CI/release gate admits the snapshot and prepared tools, then runs
`make clippy` before other validation targets. It keeps the same native all-target/
all-feature and Wasm lint scope. The sequential runner stops on either lint
failure even under parallel Make, before tests, package builds or publication
qualification. Standard releases call this same gate; publication requires its
source-bound release receipt. This library repository has no deployment target.

For ordinary development, select checks for the completed behavior change and
its callers. For example, a change to portable revocation can use
`cargo test --locked --offline -p ic-auth --features sessions,portable-signature-verification --test portable_revocation`;
a native generator change can use
`cargo clippy --locked --offline -p ic-auth-tooling --all-targets -- -D warnings`
and `make check-client-contracts`. These are selections, not a required sequence.
Run broader capability, native/Wasm or minimum-compiler matrices only when the
changed contract requires that coverage. The table lists available commands;
multi-package, multi-feature and multi-target entries are aggregate suites.

| Command or aggregate suite | Purpose |
| --- | --- |
| `make test-client-tools` | Actual CI pipeline and Make callers: selected Node/npm path, later-step admission and first-failure controls with substituted npm effects |
| `make test-client` | Packed-consumer proof, portable admission/logout, strict journal transactions and Rust/TypeScript Wire roundtrips |
| `make pack-client` | Build a local private npm archive with source-input and integrity receipts; no upload |
| `make test-client-package` | Offline archive installation, strict public declarations, browser bundling and injected admission/logout |
| `make test-client-package-browser` | Opt-in: installed archive/public exports with real Chromium IndexedDB and journal reopen; injected transports and explicit executable |
| `make test-client-browser` | Opt-in: actual Chromium storage, cross-tab/crash recovery and signed SDK receiver calls; requires an explicitly prepared executable |
| `make check-client-contracts` | Compare fresh locked bindings with the Rust-owned checked-in contracts offline |
| `make check-client-boundary` | Reject direct/aliased SDK dependencies and literal imports in the browser package; also runs in `make test-client` through npm |
| `make generate-client-contracts` | Explicitly regenerate Auth DTO bindings and the locked Wire TypeScript runtime |
| `make test-types` | Public types, bounded Wire codecs and private fixture projections |
| `make test-protocol` | Canonical encoding, signed vectors, ordering and size rejection |
| `make test-portable-signatures` | Portable Ed25519 issuer proofs with protected audience/issuer/key binding, independent signature vectors and strict malformed/weak-key refusal, including an unpublished consumer enabling permissive Dalek parsing through feature unification; this does not admit sessions |
| `make test-chain-key-signatures` | Isolated SEC1 admission, explicit producer normalization and exact prehash verification, including malformed material and strict high-S refusal |
| `make test-signatures` | Real BLS-signed IC fixtures, signer/seed/root rejection, certificate freshness and bounded decoding |
| `make test-signature-store` | Bounded preparation/retrieval, unchanged retry deadlines, pruning and real BLS verification with host-composed branches |
| `make test-certified-proofs` | Portable certified secp256k1/BLS proofs, protected delegation bindings, budgets and rejection |
| `make test-sessions` | Atomic admission/replay failure, exact retry, live authority invalidation, logout and quotas using certified and Ed25519 portable proofs |
| `make test-backend-conformance` | Shared native backend scenarios against reference callbacks and an independent encoded store; deliberate partial-commit rejection; no simulator/private fixture dependency |
| `make pack-backend-conformance` | Cargo-normalized unpublished source archive for a consumer's dev dependency; selected Engine/Protocol dependencies must already be published; no upload |
| `make test-backend-package` | Opt-in: archive, excluded dev-dependency consumer, exact registry Auth identities, packaged scenarios and graph refusals; receipt and inputs retained; registry cohort must be explicitly cached |
| `make profile-canonical-allocations` | Opt-in: optimized harness-free Cargo benchmark with native DHAT observations of unchanged canonical hash functions on synthetic small/wide inputs; outside CI; see [measurement limits and results](canonical-allocation-observations.md) |
| `bash scripts/dev/profile-retry-allocations.sh` | Opt-in: native DHAT observations of existing exact retries and claims decoding, excluding setup/signing/request construction; see [measurement scope and results](retry-allocation-observations.md) |
| `make test-host-tooling` | Native file-operation CLI: bounded input, exact digest output, no-follow reads and preserved create-only evidence |
| `make check-msrv` | Aggregate: isolated public package payloads using the declared compiler, with native/Wasm and feature coverage; internal applications are covered by Clippy |
| `make test-compiler-policy` | Compiler pin/version/active-selection and Wasm refusal cases; explicit preparation with substituted compiler effects |
| `make test-validation-inputs` | Discover Cargo workspace manifests for CI sealing; qualify added members, paths with spaces, input drift and discovery refusals |
| `make tasks` | Read the adopted maintenance task catalog; no task, agent or schedule starts |
| `make test-qualification` | Build the internal Wasm host; verify real certificates, portable proof/session admission, live authority rejection, root composition, upgrades, authenticated ingress and SDK logout through `ic-testkit` with the checked server and prepared client dependencies |
| `make build-qualification-canister` | Build the internal host's release Wasm without starting a server |
| `make check-wasm` | Aggregate: compile-check all supported public library capabilities for `wasm32-unknown-unknown` |
| `make clippy` | Aggregate: lint all libraries, native tooling/qualification and the Wasm host |
| `make metadata` | Validate the selected locked graph offline |
| `make check-candid-boundary` | Lightweight lexical check of owned Candid declarations across `crates/` and `apps/`, with no allowances; macro expansion and cross-module aliases need separate review |
| `make check-boundaries` | Run the Candid source check and inspect every capability graph, admitting upstream certificate dependencies only in certificate-enabled graphs; preserve host/runtime and passive-contract boundaries |
| `make fmt` / `make fmt-check` | Shared manifest sorting followed by Rust formatting |
| `make check-snapshot` | Verify the adopted shared files and modes |
| `make check-dependency-pins` | Check declarations, lockfile tracking and root inheritance |
| `make check-doc-links` | Check local Markdown targets |
| `make publish-dry-run RELEASE_GROUP=protocol\|engine` | Build the selected group's distributable libraries without uploading; requires registry metadata access |
| `make check-package-builds` | Package and verify all three local library archives together, included in the complete gate |
| `make install-release-tools` / `make release-tools-check` | Explicitly prepare the lock-selected IC Build editor, or admit and print its absolute path offline |
| `make test-release-tools` | Exercise release/recovery and publication rejection/retry using local and mocked effects |
| `make test-release-runner` | Opt-in producer runner matrix for changed-contract adoption or a named missing consumer proof |
| `make test-evidence-archive` | Opt-in producer archive matrix for changed-contract adoption or a named missing consumer proof |
| `make test-tools-evidence` | Actual setup/check caller failure status and adopted collector archive bytes, with Cargo installation substituted |
| `make test-testkit-tools` | Locked CLI selection, setup/check failures and selection changes, prepared-input drift refusal, qualification path and actual CI evidence archive with substituted effects |
| `make test-make-jobserver` | Actual owned Make recipes preserve pipe jobserver descriptors through Cargo/wrappers; unsafe modes refuse before substituted effects |

`make ci` is the complete configured CI/release gate. Ordinary development and
handoff select the relevant checks above; local full-gate execution follows
[Shared Tooling validation policy](../DRAGGINZGAME.md#validation-and-evidence).

Auth's ordered public package inventory is `scripts/release/packages.sh`.
Publication/retry evidence and publication dry runs select its explicit group
subset. MSRV package creation/extraction and license checks inspect the whole
inventory; archive admission reads each group's own versions. Capability-specific consumer harnesses
retain their explicit package roles and feature selections.
The generic runner/archive matrices stay at Shared Tooling and remain explicitly
callable here. Auth retains `test-release-tools` for its actual metadata,
release/recovery and publication rejection/retry; `test-tools-evidence` and
`test-testkit-tools` exercise its setup and CI archive callers, including failure
and retained evidence. Snapshot integrity still qualifies adopted bytes. No
product, native-host or release-adapter coverage is removed by this roster change.
For validation-receipt edits, `bash scripts/release/test-tools.sh --validation`
selects real metadata preparation, interrupted recovery and fresh-clone publication
with local Git and mocked upload effects. It skips unchanged setup and registry
response matrices; the complete CI target still runs the full fixture.

It uses the Shared Tooling 0.6.6 logger and retains failures. CI declares the
root Cargo manifest/lock, all member manifests, group policy, Rust compiler selection, common host/Rust and IC pins,
and browser package manifest/lock/Node selection as prepared inputs. Their byte
identities are checked before and after each stage. A change, deletion or read
failure stops later dispatch, preserves an original stage failure and reports
which inputs require fresh preparation and the full gate. Nested gates in the
same checkout retain the checkpoint; disposable checkouts establish their own.
Member manifests come from Shared's admitted Cargo group plans, including when
Protocol and Engine versions differ; Auth seals every returned member.
This detects drift at stage boundaries, without isolating arbitrary source edits
inside a running stage. Checks never fetch, reinstall or silently restart.
The workflow prepares tools and caches first, then calls that same gate. The public remote is
[dragginzgame/ic-auth](https://github.com/dragginzgame/ic-auth), and `main` contains
the current release. Local and hosted evidence for the current batch is recorded
in the [handoff](status/current.md). Dependency pinning requires each lockfile and
the npm root manifest to be tracked.
Cargo tests, metadata and Wasm checks run offline after `fetch`. The complete
gate's package check deliberately accesses registry metadata: the selected Cargo's
offline multi-package check fails when a staged dependency has no resolved checksum.
`make check-package-builds` packages and verifies all three current source archives
together, so a pending public Protocol identity change does not select the older
published Protocol when verifying Engine. It never uploads. Each invocation retains
a fresh staging/build directory under `target/package-qualification/`; Cargo's
temporary registry has a distinct identity so earlier same-version source is not
reused. Normalized package locks and build logs identify the selected suppliers.
Existing staging and caches are preserved.
`publish-dry-run RELEASE_GROUP=…` separately checks the selected group's normal
registry admission under `target/publication-dry-runs/`. That readiness check can
refuse until a changed Protocol supplier is published; the local archive check
does not relax final publication admission. New release gate logs remain under
`target/release-validation/`; saved receipts still admit historical log paths.

Once this workflow is delivered to `main`, an explicitly authorized qualification
of another branch or a tag-only release can dispatch the same workflow at that ref:

```sh
gh workflow run ci.yml --repo dragginzgame/ic-auth --ref engine-vX.Y.Z
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
adoption. Certified-proof tests additionally verify root batch ECDSA signatures
and issuer BLS certification together with caller, narrowing and current authority inputs;
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
make release-patch RELEASE_GROUP=protocol                # compatible Protocol change
make release-minor RELEASE_GROUP=engine                  # breaking Engine change
make release-major RELEASE_GROUP=engine                  # explicit major decision
make release-resume RELEASE_GROUP=engine VERSION=X.Y.Z   # exact saved group attempt
make publish-dry-run RELEASE_GROUP=protocol              # selected builds, no upload
make check-package-builds                                # both groups, no upload
make publish RELEASE_GROUP=engine                        # selected delivered release
```

Release execution requires release authority; a scoped ship/deploy request
delegates group selection and the established release/publication workflow.
Release-by-release adoption history belongs in the [changelog](../CHANGELOG.md).

`ci/cargo-release-groups.json` owns two groups. Protocol contains
`ic-auth-protocol-types` and `ic-auth-wire`; Engine contains `ic-auth` and every
unpublished qualification/tooling member. All member versions are explicit.
Each group owns `releases/<group>/CHANGELOG.md`; the root changelog indexes these
ledgers and the retained coordinated history through 0.29.2. The split changes
no package version. Missing, empty and unknown group selections refuse before
setup, validation or publication. There is no implicit all-groups release.
Engine releases preserve Protocol identities and notes. Protocol releases update
their root path requirements without publishing or changing Engine's version.
Publish a changed Protocol supplier before an Engine release that requires it.
The 0.30.0 public identity cut shipped Protocol first, then Engine against that
published supplier. Future Engine-only minor releases need no
Protocol bump.

`make test-release-tools` exercises current working sources in a disposable
export, including nonignored additions and deletions. Git-ignored build output
and prepared dependencies stay out. The shared fixture helper admits and copies
files; local regression tests check selection and preserve source/index bytes.
This test export does not relax actual release preflight's clean-source rule.

Pending numbered changelog headings may use `## [X.Y.Z]` or
`## [X.Y.Z] - Unreleased`. Standard release preparation replaces the marker with
the saved UTC date before commit/tag/push. Other suffixes and conflicting dates
still refuse; interrupted recovery retains its exact saved notes and date.

The [shared restart contract](releases.md#source-changes-after-saved-preparation-intent)
applies to IC Auth's direct delivery. Normal patch/minor/major targets always
select from current clean, committed metadata and run fresh preflight, setup and
the complete gate. They do not scan older plans, reuse their validation or replay
their commit/tag/push effects. The new candidate's stale local plan/staging/PR
receipts may be superseded; older build and validation evidence keeps its source
identity. Missing older tags or malformed historical plans do not block a new
release. Current source/index, lock, candidate ref/payload, destination and branch
protection conflicts still refuse.

Use `make release-resume RELEASE_GROUP=engine VERSION=X.Y.Z` to continue one specific interrupted
attempt against its saved source, version, payload and destination. Auth's
preparation adapter owns manifest/lock, changelog and validation receipt writes;
Git staging and delivery remain runner-owned. Neither path installs a saved
editor implicitly during recovery or weakens source/receipt admission.
See [Shared #309](https://github.com/dragginzgame/shared-tooling/issues/309).

The locked Testkit package selects its versioned setup/check CLI, even when
its server version is unchanged. After a selection change, explicitly run
`make install-testkit-tools`; `make testkit-tools-check` returns the admitted
absolute server path offline. The Cargo installer owns lockfile selection and
CLI admission; Auth re-admits the same CLI after its server operation. Missing
or changed selections refuse with the preparation command and original failure
status. Checks never install or fall back to an older CLI.

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
durable host qualification remain consumer-owned. Commit the implementation
and its pending notes before executing a release; ordinary preflight requires clean source.

All three release targets use the adopted runner, the same `make ci` gate and
direct delivery to `origin/main` by default. `RELEASE_REMOTE` and `RELEASE_BRANCH`
select another already-authorized destination. Ordinary implementation remains
local; use a PR when requested or required. The runner's separate PR delivery
mode is not qualified here.
`ci/release-delivery.txt` declares the supported direct delivery once; Make and
standalone metadata use the shared read-only policy checker. CI seals this file
with its other preparation inputs and refuses policy drift before later stages.
Preflight reports all refused staged, unstaged and untracked paths, quotes
unusual names, and distinguishes Git observation errors from dirty source.
Initial refusal identifies that this attempt has not started validation or
version preparation. Interrupted preparation retains its exact group file
metadata allowance and receipt checks. Admission preserves files and index bytes.
Normal preflight fetches the selected lockfile, then runs `make install-tools`
and `make tools-check`. The canonical ordered aggregate prepares host and common
Rust tools and lock-selected Testkit CLI/server using the already prepared compiler;
the second aggregate admits that selection offline. Shared tool
payloads alone do not activate a checkout's missing local references.
Explicit Cargo offline settings still apply; a failed preparation stops before
validation or version changes. Saved-receipt recovery admits interrupted metadata
without provisioning a potentially partial graph. Ordinary checks never install
tools. Shared group preparation changes only selected member versions, their root
dependency requirements, selected local lock identities and the selected ledger.
Auth renders from an exported validated source using the shared helper, then
installs the exact file plan and receipt through the shared metadata transaction.
`release-validation.json` binds the selected exhaustive plan to the group, policy checksum,
validated source/lock, original notes and release identity. Saved receipts include
the group in their names. External selections and unselected versions/notes are
preserved. The primary member's version is installed last; source-only version
observation lets recovery reach saved-source reconciliation with a partial payload.
Interrupted base/prepared files are reconciled against that same source.

New validated-source extracts use `target/release-metadata/`. The shared runner
and direct Auth metadata/publication adapters apply the
[retained-state guard](releases.md#artifact-retention-and-exact-push-scope) before
work and afterward, preserving the original failure if a step also violates
retention. Git-side state has a 512 MiB aggregate budget, 32 MiB per file and a
30-day age limit for bulk files of at least 1 MiB; compact receipts do not expire.
Refusal preserves evidence for review rather than deleting recovery state.
The focused caller check is `bash scripts/release/test-tools.sh --retention`;
it also runs in the normal release-tools fixture target.

The [Auth group-validation contract](auth-release-validation.md) defines required
and excluded stages, full-scope expansion and schema-4 receipts binding Shared's
canonical run evidence. `make ci` always runs the entire canonical roster.
Historical schema-2/3 receipts retain their original meaning and require their
original release checkout.

The runner stages only the owned release files, commits and creates an annotated
`<group>-vX.Y.Z` tag, then atomically pushes that branch and tag. Group journals
are `<group>-X.Y.Z.plan`. It retains plans, logs
and build artifacts on failure and success. Rerun the same normal target for
recovery; explicit resume selects one saved version. Late checks read the selected
release commit even if HEAD contains later work. Releases do not publish crates.
Historical `vX.Y.Z` tags and pre-group journals/receipts retain their original
meaning. Resume or publish those releases using their original adapter/source in
the dedicated release checkout; the new adapter refuses to reinterpret them.

`make publish RELEASE_GROUP=protocol` selects `ic-auth-protocol-types` then
`ic-auth-wire`; `RELEASE_GROUP=engine` selects only `ic-auth`. Publication always
targets crates.io. It requires a clean worktree, the exact annotated group
tag at HEAD, a validated release receipt and the same tag object on the selected
remote. Cargo builds the distributable archives before dispatch. Each package
includes its MIT notice; the package-local copies must match the root license.
Authenticate using Cargo's normal credential provider/login before an authorized
publication; the script never prints or reads credential files itself.

Publication intent under `.git/publication-state/` fixes source, tag, registry,
group/version and archive checksums before upload, in group-qualified intent and
dispatch paths. The publication and shared-attempt locks remain repository-wide.
An overlapping coordinated publication intent or dispatch marker refuses group
publication; changing namespaces cannot authorize retry of an uncertain old upload.
Successful matching registry
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
effect independently before reconciling the retained intent.

Actual upload dispatch uses the adopted Shared Tooling
`scripts/ci/run-cargo-publication.sh` with one exact `NAME@VERSION` selector after
Auth writes its private dispatched marker and rechecks source/tag admission.
Shared owns Cargo arguments, output, exit statuses and interruption evidence;
Auth still owns archive checksums and registry acceptance. Each invocation has a
fresh attempt beneath `.git/publication-state/`, named from the retained aggregate
log directory and package. Its stable parent retains `.cargo-publication.lock`
after an incomplete attempt, independently of Auth's outer `lock`.

A matching registry observation can reconcile Auth's intent while that shared
lock remains retained. It cannot release the lock or authorize the next package
upload. Inspect the stopped owner's attempt and exact registry state and obtain an
explicit operator disposition before any new dispatch; never steal a stale lock,
erase its evidence or replay an attempt. A successful read-only reconciliation
reports any remaining shared lock. An absent unresolved upload still refuses
before dispatch, even when the helper lock has been separately resolved.

Aggregate logs and archives remain in `target/`, and private attempt records remain
in publication state; no cleanup, GitHub Release object or deployment is
implicit. See [Cargo publication](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
for credentials, dry-run and index-propagation semantics.

File mechanics use the published `ic-host-file` executable. Auth pins 0.74.2 in
`ci/host-file.env`, independently of the library versions selected by Cargo.lock.
`make install-host-file` prepares it through Shared Tooling with
`--features cli`; `make host-file-check` admits that exact selection offline. The
common aggregates include both. CI cache keys include the executable pin.

`scripts/dev/run-host-file.sh` only admits and executes that binary, preserving
the invocation directory and raw arguments. It never builds the workspace or
falls back to a local command. Checks, publication and release recovery require
prior setup; admitted fresh release preflight prepares the complete toolset.
The independent pin is not rewritten during workspace release preparation.

Callers retain byte limits: 8 MiB for lockfiles, 64 MiB for package archives,
1 MiB for validation receipts, 64 KiB for publication intent and 1 KiB for
dispatch markers. NUL batches explicitly pass 4194304 path bytes and 4096 files,
preserving order, duplicates and raw paths with no successful output prefix on
refusal. Host owns hashing, no-follow admission, mode 0600 and durable
create/replace. Auth owns source custody, input seals, publication intent and
uncertain-effect reconciliation.

The unpublished Rust utility retains Auth client-contract generation and
Build-owned external-lock admission. Client generation and MSRV archive sealing
use the admitted Host file tool for their before/after identities. Actual binary
file tests and the local/mocked release fixture qualify this boundary; native
macOS remains part of hosted qualification.

The qualification runner reads Wasm through Host and obtains the simulator
client and bounded typed Wire calls through Testkit. Its SDK qualification
subprocess uses Host process admission directly. Testkit owns server selection, startup and
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

The native `ic-auth-tooling` CLI uses Clap's standard help/usage handling. Help
and version requests exit 0, invalid command syntax exits 2, and operational
failures after argument admission exit 1. Scripts must distinguish usage from
operation failures without parsing diagnostic text.

Client-contract generation records every published source/vector/policy file in
the candidate's `artifacts.tsv`. The shell driver validates this producer-owned
inventory before comparing or publishing files; it carries no separate filename
roster. `make test-client-tools` qualifies future outputs and refuses malformed
inventories or changed generator inputs before publication. Native tooling and
cross-language tests retain independent codec expectations.

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
