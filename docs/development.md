# Development

The virtual root owns two libraries, unpublished native tooling in `apps/tooling/`
and the internal host/runner in `apps/qualification/`, plus dependency selections,
lints and one lockfile. The current manifest version is `0.1.14`, tagged as
`v0.1.14`; both libraries are published on crates.io.
Rust `1.99.0` remains the development compiler selected by `rust-toolchain.toml`.
The common package minimum is Rust `1.88.0`, inherited from the root catalog and
qualified by explicit older-compiler checks. See [MSRV coverage](msrv.md).
Linux x86_64 is the initially exercised host.

The supported host matrix is Linux x86_64 and macOS 15 on Intel and Apple Silicon.
CI runs the same complete gate on `ubuntu-24.04`, `macos-15-intel` and `macos-15`.
Native macOS qualification for this batch remains pending those jobs. Shared
Tooling's [host prerequisites](supported-hosts.md) apply, including Bash 3.2+,
GNU Make, rustup, Git and standard Unix setup utilities.

## Setup

Install rustup through your normal host setup, then from this checkout:

```sh
rustup show active-toolchain
make install-tools
make tools-check
make install-hooks
make fetch
export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PWD/.tools/rust/bin:$PATH"
```

The first command prepares the toolchain, rustfmt, Clippy and Wasm target selected
by the toolchain file. Common system prerequisites, exact executable pins and
installation behavior are described in the shared [local setup](local-setup.md).
The Makefile includes the reviewed `make/tools.mk` and adds the shared Rust tool
set, the locked Testkit CLI/server and the declared minimum compiler/Wasm target
to aggregate setup/check commands. `make install-msrv` prepares the minimum compiler alone (and its host
tool prerequisites); `make msrv-tools-check` inspects its prepared versions.
Installation is explicit; checks run
offline. `fetch` prepares the selected lockfile without changing it.

Make selects checkout-local tools automatically. The shell export is needed for
interactive direct commands. Builds stay under this checkout's `target/`; Cargo
tool installation retains failed build attempts under `.tools/rust/build/` and
successful Testkit CLI builds in their versioned `.tools/rust/` selection slot.

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

## Validation

| Focused command | Purpose |
| --- | --- |
| `make test-client` | Private client lifecycle, IndexedDB transactions, compiled SDK consumer example and Rust/TypeScript Candid roundtrip |
| `make check-client-contracts` | Compare fresh locked bindings with the Rust-owned checked-in contracts offline |
| `make generate-client-contracts` | Explicitly regenerate the checked-in client contracts from Rust DTOs |
| `make test-types` | Identifier rejection and Candid wire compatibility |
| `make test-protocol` | Canonical encoding, signed vectors, ordering and size rejection |
| `make test-signatures` | Real BLS-signed IC fixtures, signer/seed/root rejection, certificate freshness and bounded decoding |
| `make test-signature-store` | Bounded preparation/retrieval, unchanged retry deadlines, pruning and real BLS verification with host-composed branches |
| `make test-tokens` | Complete real secp256k1/BLS proof chains, signed binding and protected authority/window/scope rejection |
| `make test-sessions` | Atomic admission/replay failure, exact retry, live authority invalidation, bounded pruning and quotas using real token proofs |
| `make test-host-tooling` | Native file-operation CLI: bounded input, exact digest output, no-follow reads and preserved create-only evidence |
| `make check-msrv` | Isolated public package payloads and internal applications using the declared minimum compiler, with native/Wasm and feature coverage |
| `make tasks` | Read the adopted maintenance task catalog; no task, agent or schedule starts |
| `make test-qualification` | Build the internal Wasm host; verify real certificates, root composition, upgrades and authenticated ingress through `ic-testkit` with the checked server |
| `make build-qualification-canister` | Build the internal host's release Wasm without starting a server |
| `make check-wasm` | Compile-check both libraries and signature verification/preparation, token and session capabilities for `wasm32-unknown-unknown` |
| `make clippy` | Lint the libraries, native tooling/qualification and Wasm host with their selected targets |
| `make metadata` | Validate the selected locked graph offline |
| `make check-boundaries` | Inspect default, signature verification/preparation, token and session transitive dependency graphs |
| `make fmt` / `make fmt-check` | Shared manifest sorting followed by Rust formatting |
| `make check-snapshot` | Verify the adopted shared files and modes |
| `make check-dependency-pins` | Check declarations, lockfile tracking and root inheritance |
| `make check-doc-links` | Check local Markdown targets |
| `make publish-dry-run` | Build both distributable libraries without uploading; requires registry metadata access |
| `make test-release-tools` | Exercise release/recovery and publication rejection/retry using local and mocked effects |
| `make test-release-runner` | Adopted Shared Tooling runner regression fixtures with substituted release effects |
| `make test-evidence-archive` | Adopted evidence archive fixtures: retained files, symlinks and literal paths |
| `make test-tools-evidence` | Actual setup/check caller failure status and adopted collector archive bytes, with Cargo installation substituted |
| `make test-testkit-tools` | Locked CLI selection, setup/check failures and selection changes, qualification path and actual CI evidence archive with substituted effects |

`make ci` is the complete configured gate, reserved for explicit requests and CI.
It uses the shared validation logger and retains failures. The workflow prepares
tools and caches first, then calls that same gate. The public remote is
[dragginzgame/ic-auth](https://github.com/dragginzgame/ic-auth), and `main` contains
the current release. This batch has focused local evidence only; remote CI is
not inferred from it. Dependency pinning requires each lockfile and the npm root manifest to be tracked.
Cargo tests, metadata and Wasm checks run offline after `fetch`. The publication
dry run deliberately accesses registry metadata: the selected Cargo's offline
multi-package check fails when a staged dependency has no resolved checksum.
It never uploads packages. It is part of the complete CI/release gate; release
logs remain under `.git/release-state/validation-logs/`.

CI stores validation logs and local fixture evidence under
`target/portable-fixtures/`. Failed jobs use the adopted retention action and
archiver; downloaded artifacts contain `evidence.tar.gz`. Extract that archive
to inspect the retained files. This layout does not erase prior evidence under
`target/release-tools.*/`.

CI's `scripts/dev/ci-tools.sh install|check` invokes the real aggregate targets,
preserves pipeline failure status and saves their output in
`target/rust-tools-install.log` and `target/rust-tools-check.log`. Both logs and
`.tools/rust/build` are selected by the adopted failure collector. The native
matrix also injects a failed Cargo installation into an isolated consumer, checks
offline refusal, uploads through the same action and downloads that exact artifact
ID to verify retained bytes. Local archive checks do not establish hosted upload
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
see the [signature](signatures.md) and [token](tokens.md) contracts. Session tests
cover admission and authorization with a volatile backend and a failing staged
transaction; they do not qualify stable canister storage or lifecycle. The
[session contract](sessions.md) specifies the host's durability obligations.
Preparation tests additionally cover host-owned certification composition and
bounded retrieval retention; see [the preparation contract](signature-preparation.md).
They do not qualify an actual canister's root publication or query/upgrade lifecycle.
The separate [IC Testkit command](ic-testkit-qualification.md) exercises those host
paths and real signed ingress. It obtains the verified server from the local IC
bundle, checks exact client/server alignment and retains state beneath `target/`.
It does not qualify browser/wallet login or a durable session backend.

## Releases and publication

```sh
make release-patch                 # only for compatible work; current hard cut requires minor
make release-minor                 # current pending developer-tooling hard cut: 0.1.14 -> 0.2.0
make release-major                 # explicit major decision: 0.1.14 -> 1.0.0
make release-resume VERSION=X.Y.Z   # reconcile the exact saved attempt
make publish-dry-run               # validate packages without upload
make publish                       # upload a clean, delivered tagged release
```

Released `0.1.13` adopts Shared Tooling 0.1.36's selected Cargo installer and
receipt/path/failure-status checks, dependency-preparation guidance and Host
Tooling 0.8.9. Product CLI selections and profiles remain consumer-owned;
that release preserved its then-current PocketIC setup. Released `0.1.14` documents
the [Canic library adoption contract](canic-adoption.md) and qualifies its token
issuance Candid envelopes in both directions. The pending `0.2.0` draft
extends that qualification to complete tokens and root/issuer proof transport.
It also adopts Shared Tooling 0.2.0's breaking PocketIC ownership handoff. Run
explicit `make install-tools` after updating: the shared IC bundle now holds
five tools, and the locked Testkit CLI owns server setup and offline admission.
`make install-testkit-tools` prepares only that CLI/server route;
`make testkit-tools-check` returns the admitted absolute server path without
downloading. `make test-testkit-tools` checks caller selection, failures and
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
Preflight reports all refused staged, unstaged and untracked paths, quotes
unusual names, and distinguishes Git observation errors from dirty source.
Initial refusal identifies that this attempt has not started validation or
version preparation. Interrupted preparation retains its exact four-file
metadata allowance and receipt checks. Admission preserves files and index bytes.
Preflight fetches only the selected lockfile. Preparation changes the root
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
stop the run. An attempted upload without confirmation remains unresolved:
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
