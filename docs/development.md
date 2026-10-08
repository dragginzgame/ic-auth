# Rust development

The virtual root owns both packages, dependency selections, lints and the single
lockfile. The initial local package version is `0.1.0`; there is no released base
or published history. Publication is disabled. Rust `1.99.0` matches the inspected
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

`make ci` is the complete configured gate, reserved for explicit requests and CI.
It uses the shared validation logger and retains failures. The workflow prepares
tools and caches first, then calls that same gate. The public remote is
[dragginzgame/ic-auth](https://github.com/dragginzgame/ic-auth), but no source has
been pushed, so the workflow has no remote execution evidence. Dependency pinning requires
the lockfile to be in Git's index; the initial uncommitted bootstrap can be checked
in an isolated staged fixture without staging the user's checkout.

These checks do not establish cryptographic verification, authenticated session
admission, PocketIC behavior, service upgrades or independent consumer adoption.
Those checks belong with the corresponding implementation.

## Release boundary

The three standard release targets and resume entry point dispatch to the
reviewed shared runner; their routing is checked with a substitute runner that
has no release effects. Release preflight deliberately rejects this bootstrap.
There is no finalized version base or qualified release metadata and delivery
adapter. The public `origin` remote is configured. Before the first authorized release, settle that identity
and implement/qualify the remaining adapters under the [release contract](releases.md).
The current `0.1.0` changelog describes the initial local implementation; it is not
evidence of a release. No release, registry publication or deployment is enabled
or authorized by this setup.
