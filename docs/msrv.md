# Minimum Rust compiler qualification

All five workspace packages inherit `rust-version = "1.88.0"` from the virtual
root. Rust 1.99.0 remains the development compiler for formatting, Clippy and
functional tests; it is independent of the supported minimum.

The selected public native dependency path includes `psm` 0.1.32 and
`ar_archive_writer` 0.5.3, both requiring Rust 1.88.0. A real Rust 1.85.0 check
rejects that graph before compilation. No dependencies were downgraded to change
the result. Rust 1.88.0 passes the selected paths below; internal CDK, IC agent,
testkit and host packages also require that floor. A future proposed lower floor
needs a maintained dependency selection and actual compiler qualification.

## Commands and coverage

`make install-tools` explicitly prepares the minimum compiler and its Wasm target,
alongside the separately selected developer tools. `make install-msrv` prepares
only that compiler and host-tool prerequisites. `make msrv-tools-check` prints
the actual rustc/Cargo versions and requires the installed Wasm target. It never
installs a missing compiler implicitly.

`make check-msrv` uses explicit `cargo +1.88.0` selection read from the root floor.
It prints the compiler identities, works offline and refuses lock changes. Cargo
on the separately selected development compiler normalizes both public package
archives without uploading. Their bounded digests and a unique unpacked fixture
are retained under `target/portable-fixtures/msrv.*`.

The normalized packages are checked outside the production workspace, so private
test infrastructure cannot supply dependency features to a library consumer.
The auth package uses the matching local protocol payload; only that fixture's
path/source identity changes. Every external package version, source and checksum
must remain a subset of the real workspace lock. There is no unlocked resolution,
dependency downgrade, network fallback or `--ignore-rust-version` check.

| Package path | Minimum-compiler coverage |
| --- | --- |
| Protocol types | Native and `wasm32-unknown-unknown`, independent normalized package |
| Auth library | Each target with default features, signature verification, signature preparation, token verification, sessions, and all features together |
| Native tooling and testkit runner | Separate workspace checks of all targets/features |
| Qualification canister | Separate `wasm32-unknown-unknown` check |

Build output stays under `target/msrv/`, with package inputs under
`target/msrv-package/`; it does not clean existing output. Source/packages and
minimum-compiler logs are retained for inspection after success or failure.
The real server, lifecycle and signed-ingress tests remain owned by
[`make test-qualification`](ic-testkit-qualification.md).

The configured CI/release gate includes `check-msrv` on Linux and both supported
macOS architectures. Setup installs the declared compiler; the check selects it
explicitly despite `rust-toolchain.toml`. Local Linux checks pass. Hosted native
macOS evidence for these changed sources awaits their own CI jobs; the 0.1.7
release jobs do not qualify this uncommitted batch.
