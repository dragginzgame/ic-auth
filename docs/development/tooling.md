# Developer tooling

Use Rust 1.100.0, pinned in `rust-toolchain.toml`. The root virtual workspace
contains three unpublished product packages under `crates/`. Each inherits
shared package fields, lints and the exact Toko source-tooling dev-dependency.
`qualification/` is an independent workspace containing the reference/current
Wire comparison and adversarial Ed25519 feature check. Ordinary integration tests
stay in the product crates.

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
cargo fetch --locked --manifest-path qualification/Cargo.toml
make install-hooks
```

Setup may download pinned tools/crates; hook installation changes only the local
Git hook selection and refuses to replace other hooks. Routine checks are offline:

```sh
make toko-tooling-check
make test-request
make test-claims
make test-wire
make test-verification
make qualify-ed25519
make lint-ed25519-qualification
make qualify-wire
make lint-wire-qualification
make lint-protocol
make doc-protocol
make check-protocol-wasm
make fmt-check
```

These are focused on the implemented protocol boundary. `test-request` includes
the scope/request integration files and the immutable-scope compile-fail doctest;
`test-claims` covers roles and canonical signed-claims bytes/refusals; `test-wire`
covers all four contracts, nested proofs and bounded refusal. `qualify-wire`
compares both native suppliers; `lint-wire-qualification` checks that private
package. `test-verification` covers optional Ed25519/context/budget behavior and
immutable-result compile-fail cases. `qualify-ed25519` deliberately enables the
legacy scalar feature in its separate workspace; its paired lint command is
`lint-ed25519-qualification`. The lint/docs/Wasm targets aggregate the three implemented protocol
packages with the verifier enabled; lint includes the optional native generator and Wasm proves its metadata
remains excluded even with the feature forwarded. For a narrower check,
use `cargo clippy --offline --locked -p ic-auth-wire --all-targets --features typescript -- -D warnings`,
`RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-auth-wire --no-deps`,
or `cargo check --offline --locked -p ic-auth-wire --lib --target wasm32-unknown-unknown`.
`check-protocol-wasm` requires the Rust toolchain's `wasm32-unknown-unknown` target
and proves compilation only. No full replacement, installed lifecycle or release
gate is implied. `make fmt` writes the same Rust/manifest selection checked by
`fmt-check`. The configured hook was exercised by the 0.1.0 source milestone.

`toko-tooling-check` admits product members/lock and the installed snapshot.
Its local qualification prerequisite uses the same shared declaration/lock
checker and compares its result with the installed receipt. `snapshot-check` is the narrower byte-integrity check retained
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

## Browser codecs and qualification

`make generate-wire` emits codecs and the matching runtime into
`qualification/target/wire-browser`. `make qualify-wire-browser` additionally
executes the pinned reference and new generated codecs with Node 24.21.0.
Set `AUTH_REFERENCE_REPO` if the reference Git objects are somewhere other than
`../ic-auth`; the script only reads that checkout. See the
[Wire evidence](wire-boundary.md) for exact scope.

The independent qualification workspace owns its catalog, lock and lints, uses
`license-file = "../LICENSE.md"`, inherits tooling and remains unpublished. The
root formatter includes it through `FORMAT_RUST_ARGS`. Product packages never
depend on qualification packages; the old codec exists only in that workspace.
The legacy-scalar activator is likewise qualification-only. Normal Auth consumers
use Platform through the optional `ed25519-verification` feature, with no direct
Dalek dependency and no legacy mode selected by the product workspace.

See the reviewed upstream
[adoption guide](https://github.com/dragginzgame/toko-tooling/blob/8bcece573511a6ce6ee8879443a8d19b6e43e7ad/docs/adoption/consuming-snapshots.md)
and [LOC setup](https://github.com/dragginzgame/toko-tooling/blob/8bcece573511a6ce6ee8879443a8d19b6e43e7ad/docs/fleet/fleet-tools.md#cargo-workspace-loc-report).
