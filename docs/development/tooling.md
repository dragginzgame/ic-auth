# Developer tooling

Use Rust 1.100.0, pinned in `rust-toolchain.toml`. The root virtual workspace
contains the two unpublished product packages under `crates/`. Each inherits
shared package fields, lints and the exact Toko source-tooling dev-dependency.
`qualification/` is reserved for an independent workspace when its first real
package arrives; ordinary integration tests stay in the product crates.

## Installed selection

Toko Tooling **0.44.4** is selected by Cargo and installed through its authenticated
exporter. Cargo.lock and `.toko-tooling.snapshot` identify archive SHA-256
`b52bb793b386309b8c4a9e88f32711bb5d1fd11a47bf337c11c63ca2aa22fc97`.
The release source is
[`8bcece573511a6ce6ee8879443a8d19b6e43e7ad`](https://github.com/dragginzgame/toko-tooling/tree/8bcece573511a6ce6ee8879443a8d19b6e43e7ad).
Do not hand-edit the receipt or installed vendor files.

The executable selections are `make/consumer.mk`, `make/cargo-dependencies.mk`
and the formatting hook, with their required companions. No release or
publication interface is selected. Local checks are owned by the root Makefile.

## Setup and focused checks

From the repository root on a fresh checkout:

```sh
make install-tools
make tools-check
cargo fetch --locked
make install-hooks
```

Setup may download pinned tools/crates; hook installation changes only the local
Git hook selection and refuses to replace other hooks. Routine checks are offline:

```sh
make toko-tooling-check
make test-request
make test-claims
make lint-protocol
make doc-protocol
make check-protocol-wasm
make fmt-check
```

These are focused on the implemented protocol boundary. `test-request` includes
the scope/request integration files and the immutable-scope compile-fail doctest;
`test-claims` covers roles and canonical signed-claims bytes/refusals.
`check-protocol-wasm` requires the Rust toolchain's `wasm32-unknown-unknown` target
and proves compilation only. No full replacement, installed lifecycle or release
gate is implied. `make fmt` writes the same Rust/manifest selection checked by
`fmt-check`. Hook activation and an actual staged-commit exercise are separate;
no commit or staged payload has been created by this implementation task.

`toko-tooling-check` admits every member's declaration, Cargo.lock and installed
snapshot together. `snapshot-check` is the narrower byte-integrity check retained
from bootstrap, without Cargo selection admission.

## Dependency maintenance and LOC

```sh
make deps-list
make deps-update
make deps-update DEPENDENCY_PACKAGE=toko-tooling
toko-cloc -d 1
```

`deps-list` is an offline local package inventory. `deps-update` runs Cargo in the
root and any maintained qualification workspace sequentially; it does not edit
manifest requirements or roll back earlier lock changes. Review the locks.
The exact Toko pin changes only by deliberate selection; after updating that pin
and lock, use `make toko-tooling-preview` and `make toko-tooling-update`.

`toko-cloc` selects the current checkout's reporter and reports both workspaces
when present. There is no `make cloc` alias. The repository-local launcher is
`bash vendor/toko-tooling/scripts/dev/toko-cloc`. For direct prepared-tool use:

```sh
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
```

## Adding a qualification workspace

With the first real qualification package, create its independent virtual
workspace, catalog, lock and common lint table; use
`license-file = "../LICENSE.md"`, inherited tooling selection and
`publish = false`. Add `FORMAT_RUST_ARGS := --extra-workspace qualification`
before the shared Make include and admit tooling in each workspace.
Product packages must never depend on qualification packages.

See the reviewed upstream
[adoption guide](https://github.com/dragginzgame/toko-tooling/blob/8bcece573511a6ce6ee8879443a8d19b6e43e7ad/docs/adoption/consuming-snapshots.md)
and [LOC setup](https://github.com/dragginzgame/toko-tooling/blob/8bcece573511a6ce6ee8879443a8d19b6e43e7ad/docs/fleet/fleet-tools.md#cargo-workspace-loc-report).
