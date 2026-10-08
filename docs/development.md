# Rust development

The virtual root owns both packages, dependency selections, lints and the single
lockfile. The committed bootstrap package version is `0.1.0`; there is no tagged
or published history. Both libraries select crates.io. Rust `1.99.0` matches the inspected
Canic toolchain; `rust-toolchain.toml` is the sole toolchain selection. No lower
MSRV is claimed. Linux x86_64 is the initially exercised host.

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
set to its aggregate setup/check commands. Installation is explicit; checks run
offline. `fetch` prepares the selected lockfile without changing it.

Make selects checkout-local tools automatically. The shell export is needed for
interactive direct commands. Builds stay under this checkout's `target/`; Cargo
tool installation retains its build output under `.tools/rust/build/`.

The reviewed pre-commit hook formats selected files without building or testing,
rejects partial staging and preserves unrelated edits. Hook activation does not
establish independent formatting qualification.

## Validation

| Focused command | Purpose |
| --- | --- |
| `make test-types` | Identifier rejection and Candid wire compatibility |
| `make test-protocol` | Canonical encoding, signed vectors, ordering and size rejection |
| `make check-wasm` | Compile-check both libraries for `wasm32-unknown-unknown` |
| `make clippy` | Lint these two packages and their test targets |
| `make metadata` | Validate the selected locked graph offline |
| `make check-boundaries` | Inspect actual transitive default dependency graphs |
| `make fmt` / `make fmt-check` | Shared manifest sorting followed by Rust formatting |
| `make check-snapshot` | Verify the adopted shared files and modes |
| `make check-dependency-pins` | Check declarations, lockfile tracking and root inheritance |
| `make check-doc-links` | Check local Markdown targets |
| `make publish-dry-run` | Build both distributable libraries without uploading; requires registry metadata access |
| `make test-release-tools` | Exercise release/recovery and publication rejection/retry using local and mocked effects |

`make ci` is the complete configured gate, reserved for explicit requests and CI.
It uses the shared validation logger and retains failures. The workflow prepares
tools and caches first, then calls that same gate. The public remote is
[dragginzgame/ic-auth](https://github.com/dragginzgame/ic-auth), and `main` contains
the bootstrap commit. This batch has focused local evidence only; remote CI is
not inferred from it. Dependency pinning requires the lockfile to be tracked.
Cargo tests, metadata and Wasm checks run offline after `fetch`. The publication
dry run deliberately accesses registry metadata: the selected Cargo's offline
multi-package check fails when a staged dependency has no resolved checksum.
It never uploads packages. It is part of the complete CI/release gate; release
logs remain under `.git/release-state/validation-logs/`.

These checks do not establish cryptographic verification, authenticated session
admission, PocketIC behavior, service upgrades or independent consumer adoption.
Those checks belong with the corresponding implementation.

## Releases and publication

```sh
make release-patch                 # 0.1.0 -> 0.1.1
make release-minor                 # 0.1.0 -> 0.2.0
make release-major                 # 0.1.0 -> 1.0.0
make release-resume VERSION=X.Y.Z   # reconcile the exact saved attempt
make publish-dry-run               # validate packages without upload
make publish                       # upload a clean, delivered tagged release
```

The undated `0.1.1` draft contains the entire initial pending batch. This selects
the first patch increment from the committed `0.1.0` manifest; it does not invent
a finalized `0.1.0` release. A minor/major request needs a matching pending heading,
as required by the [shared release contract](releases.md). Commit the implementation
before executing a release; only pending notes may be dirty at ordinary preflight.

All three release targets use the adopted runner, the same `make ci` gate and
direct delivery to `origin/main` by default. `RELEASE_REMOTE` and `RELEASE_BRANCH`
select another already-authorized destination. PR delivery is not qualified here.
Preflight admits staged and unstaged paths independently and fetches only the
selected lockfile. Preparation changes the root version, the inherited internal
dependency requirement, the two local lock entries and the candidate notes.
It adds `release-validation.json`, binding the complete gate to the validated
source, lockfile, original notes and selected release identity. External dependency
selections remain byte-for-byte unchanged. The canonical root version is replaced
last so interrupted preparation can be repeated from the retained intent.

The runner stages only the owned release files, commits and creates an annotated
`vX.Y.Z` tag, then atomically pushes that branch and tag. It retains plans, logs
and build artifacts on failure and success. Rerun the same normal target for
recovery; explicit resume selects one saved version. Late checks read the selected
release commit even if HEAD contains later work. Releases do not publish crates.

`make publish` selects only `ic-auth-types` and `ic-auth`, in that dependency order,
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

The focused fixture uses real version/lock transforms, a substitute complete gate,
local commits/tags/bare remotes and mocked Cargo uploads/registry observations.
It covers patch/minor/major, gate failure, staged source rejection, candidate
conflicts, interrupted preparation, exact resume, dirty/untagged publication,
registry errors, checksum conflicts and lost-reply reconciliation. This is
consumer adapter evidence; it is not a live release or registry upload.
