# Minimum Rust compiler qualification

The public libraries, native tooling and Testkit runner inherit
`rust-version = "1.91.0"` from the virtual root, as does the unpublished qualification
canister. Rust 1.99.0 remains the development compiler for formatting, Clippy and
functional tests; it is independent of the supported minimum.

The workspace minimum is deliberately aligned at Rust 1.91.0. Earlier checks
qualified Rust 1.88.0 for an earlier dependency selection; that historical result
does not qualify the current graph. A future proposed lower floor needs a
maintained dependency selection and actual compiler qualification.

## Commands and coverage

`make install-tools` explicitly prepares the workspace minimum compiler and its Wasm target,
alongside the separately selected developer tools. `make install-msrv` prepares
that compiler and host-tool prerequisites. `make msrv-tools-check` prints
the actual rustc/Cargo versions and requires the installed Wasm target. It never
installs a missing compiler implicitly.

`make check-msrv` uses explicit `cargo +1.91.0` selection read from the root floor.
It prints the compiler identities, works offline and refuses lock changes. Cargo
on the separately selected development compiler normalizes all three public package
archives without uploading. Their bounded digests and a unique unpacked fixture
are retained under `target/portable-fixtures/msrv.*`.

An independent consumer checks the normalized packages outside the production
workspace, so private test infrastructure cannot supply dependency features.
Only its manifest patches crates.io to select the three exact unpacked packages;
packaged manifests and locks remain untouched. Offline metadata projects the
fixture lock from the selected workspace graph and adds this consumer root.
Before compilation, admission requires all three exact package names, versions and
manifest paths, enforces the library's forbidden-dependency boundary, and requires every
external version, source and checksum to remain a subset of the original lock.
The native utility delegates external-lock comparison to published IC Build,
using existing no-follow reads with an explicit 8 MiB per-document allowance.
Malformed and duplicate lock rows are refused. Auth retains package identity,
source/feature isolation and release policy; Build stays outside public libraries.
Compiler checks are locked/offline, without a dependency downgrade, network
fallback or `--ignore-rust-version`.

The independent consumer constructs and hashes the public canonical session
request on native and Wasm without optional features. Each default selection's
normal/build graph is recorded and rejects Candid, K256, certificate verification, CDK
and stable storage. The optional session engine remains a separate feature check.

Admission refusal cases run as part of `check-msrv`. Hashes seal the archives and
every unpacked file before metadata resolution and again after all compilation;
original and consumer manifests/locks must also remain unchanged. Failed hashes,
comparisons or compiler commands fail the check and retain their evidence. Sealing
uses one bounded `hash-files0` batch per round, preserving sorted NUL-delimited
package paths and explicit manifest/archive inputs. Batching does not cache
identities across rounds or replace the before/after comparisons. This
qualifies local archive interoperability, not crates.io publication readiness;
`publish-dry-run` retains ordinary Cargo registry admission.
See [Auth #21](https://github.com/dragginzgame/ic-auth/issues/21) and the adopted
[coordinated-archive guidance](../rules/cargo-dependencies.md#qualifying-coordinated-package-archives-before-a-version-bump).

| Package path | Minimum-compiler coverage |
| --- | --- |
| Protocol types | Native and `wasm32-unknown-unknown`, independent normalized package |
| Wire transport | Native/Wasm independent normalized package; TypeScript generation included in the all-feature profile |
| Auth library | Each target with default features, isolated portable Ed25519 verification, isolated chain-key verification, signature verification, signature preparation, certified portable verification, sessions, session restoration, and all features together |
| Native tooling, fixture transport and testkit runner | Separate workspace checks of all targets/features |
| Qualification canister | Separate Rust 1.91.0 `wasm32-unknown-unknown` check |

Build output stays under `target/msrv/`, with package inputs under
`target/msrv-package/`; it does not clean existing output. Source/packages and
minimum-compiler logs are retained for inspection after success or failure.
The real server, lifecycle and signed-ingress tests remain owned by
[`make test-qualification`](ic-testkit-qualification.md).

The configured CI/release gate includes `check-msrv` on Linux and both supported
macOS architectures. Setup installs the declared compiler; the check selects it
explicitly despite `rust-toolchain.toml` and records its actual Bash version.
At 0.1.11, Linux's complete CI passed, but both macOS jobs stopped at an empty
native-target array under Bash's strict unset-variable checks. The current fix
preserves zero native arguments and the unchanged Wasm/feature matrix using
Bash 3.2-safe expansion. That full matrix passes on Linux using actual Bash
3.2.57; it does not establish native macOS qualification before matching CI runs.
