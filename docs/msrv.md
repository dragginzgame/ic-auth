# Minimum Rust compiler qualification

Rust **1.99.1** is both the supported minimum and the pinned development/release
compiler, as approved in [Auth #124](https://github.com/dragginzgame/ic-auth/issues/124).
Every workspace package inherits the root `rust-version`; the independent archive
and backend consumers declare that same floor. `rust-toolchain.toml` selects the
exact version, Rustfmt, Clippy and `wasm32-unknown-unknown`. Moving `stable` and a
separate older-compiler promise are not supported. Historical 1.88/1.91 evidence
and caches remain retained; they do not qualify the current graph.

## Preparation and admission

Explicitly prepare the toolchain with `rustup show active-toolchain` before
`make install-tools`. Alternatively, `make install-msrv` prepares the compiler,
Rustfmt, Clippy and Wasm target after host-tool setup. The common setup aggregate
then prepares Cargo tools and the selected Testkit/file extensions; it no longer
installs a second compiler. `make tools-check` includes `make msrv-tools-check`.

Compiler admission reads the workspace floor and requires the toolchain pin to
match it. It prints actual Cargo/rustc identities, verifies both versions, checks
that active invocations report those same identities, and requires the installed
Wasm target. Ordinary checks disable Rustup auto-installation and never provision
missing tools. `make test-compiler-policy` tests pin/version/active-selection/Wasm
refusals and explicit setup with substituted compiler effects.

## Retained compilation coverage

`make check-msrv` selects `cargo +1.99.1` from the admitted floor. Cargo normalizes
all three public packages without uploading, retains their bounded digests and
unpacks them into a unique `target/portable-fixtures/msrv.*` fixture. An independent
consumer compiles those exact normalized packages outside the workspace, so
private test dependencies cannot provide missing library features.

Only the consumer patches crates.io; packaged manifests and locks remain intact.
Offline metadata projects its lock from the workspace graph. Before compilation,
admission checks package identities, independently selected Engine/Protocol
versions, source paths and forbidden dependencies. Published IC Build admits each
external version/source/checksum as a subset of the original lock, with bounded
no-follow input reads. Malformed/duplicate rows and identity or graph drift fail.
Checks use locked dependencies offline, without downgrades or `--ignore-rust-version`.

| Obligation | Retained owner in complete CI/release gate |
| --- | --- |
| Protocol types and Wire normalized packages | `check-msrv`: native/Wasm independent consumer; canonical Types identity witness |
| Every public Auth capability | `check-msrv`: native/Wasm isolated feature selections plus session/portable combination and all features; Wire TypeScript generation included |
| Pure request dependency isolation | `check-msrv`: constructed/hashed public request and recorded normal/build graph, refusing crypto, Platform facade, Candid, CDK and stable storage |
| Native internal applications, test targets and feature-unification harness | Native all-target/all-feature `clippy`, including proc-macro expansion |
| Internal runtime host and generator | Canister Wasm `clippy`, plus actual release Wasm build in `test-qualification` |
| Runtime, upgrade, certificates and signed ingress | `test-qualification`, using admitted Testkit/server and browser dependencies |
| Supported native hosts | Same gate on Linux x86_64 and macOS Intel/Apple Silicon |

The repeated internal `cargo check` stages and the gate's additional workspace
`check-wasm` pass are removed because the same compiler now covers their graphs
above. `make check-wasm` remains available as a focused development selection.
No hosted jobs or required checks are renamed or removed.

The library feature roster comes from `scripts/ci/auth-feature-selections.sh`.
Archive admission refusal cases remain part of `check-msrv`. Hashes seal all
archives/unpacked files before metadata and after compilation; manifests and locks
must also remain unchanged. Failures retain evidence. Package normalization stays
under `target/msrv-package/`; compilation reuses the ordinary `target/` cache.
Existing `target/msrv/` evidence is neither deleted nor repurposed.

Archive interoperability does not establish registry publication readiness or
installed consumer adoption. `make publish-dry-run RELEASE_GROUP=protocol|engine`
retains registry admission. See [Auth #21](https://github.com/dragginzgame/ic-auth/issues/21)
and the [archive guidance](../rules/cargo-dependencies.md#qualifying-coordinated-package-archives-before-a-version-bump).
