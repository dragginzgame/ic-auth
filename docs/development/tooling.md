# Developer tooling

Use Rust 1.100.0, pinned in `rust-toolchain.toml`. The root virtual workspace
contains three product packages under `crates/`. Each inherits
shared package fields, lints and the exact Toko source-tooling dev-dependency.
`qualification/` is an independent workspace containing current Wire fixture
checks and certified issuer fixtures. Ordinary integration tests
stay in the product crates.

## Installed selection

Toko Tooling **0.46.9** is selected by Cargo and installed through its authenticated
exporter. Cargo.lock and `.toko-tooling.snapshot` identify archive SHA-256
`42cc531ac13934abd2fff578d828bc36166edda0612dc9c6e5fdcbf8af1e6257`.
The release source is
[`de56db7686ad00e0f3b853071eef14d8c6a912b5`](https://github.com/dragginzgame/toko-tooling/tree/de56db7686ad00e0f3b853071eef14d8c6a912b5).
Do not hand-edit the receipt or installed vendor files.

The executable selections are `make/consumer.mk`, `make/cargo-dependencies.mk`,
the formatting hook, the common-tools and failure-evidence composite actions,
the pinned CI-tool installer and the setup diagnostic redactor, with their required companions. No release or
publication interface is selected. Product checks are owned by the root Makefile.
[Publication scope](publication.md) includes all three product crates and
keeps qualification packages excluded. Product manifests now allow publication;
the maintainer authorized the current crates.io release.

## Setup and focused checks

Validation scope follows the changed inputs. The entrypoints below have distinct
owners and guarantees; dependency adoption does not trigger publication.

| Flow | Entrypoint and owner | Input scope and trigger |
| --- | --- | --- |
| Development | Auth's focused `make test-*`, lint and fixture targets below | Changed product behavior and its affected callers |
| Tooling adoption | Shared `make toko-tooling-preview`, `toko-tooling-update`, `snapshot-check`, `tools-check`; Auth caller checks | Requested snapshot refresh; authenticate installed bytes and qualify changed local wiring |
| Dependency adoption | Cargo locked graphs; Auth product/qualification, Wasm and installed targets below | Changed supplier capabilities; qualify each affected workspace and execution boundary |
| Repository CI | `.github/workflows/ci.yml` | Push/PR inputs; required native hosts and redacted security checks |
| Release validation | Auth's selected product, qualification and installed checks | Authorized release; one planned validation pass for the affected product and consumers |
| Publication | Exact-package Cargo commands in [publication scope](publication.md) | Authorized upload; fresh final-source, archive and registry admission |
| Recovery | Diagnose retained failing-phase logs and rerun its affected command | Failed phase; changed inputs invalidate affected prior results |

No shared release adapter is installed yet. The documented direct Cargo release
path remains the fallback; it has no cross-command proof admission or automatic
resume. Repeated archive tests establish packaged-source behavior separately from
workspace tests. A future adapter must qualify dispatch, invalidation and failure
recovery before replacing that path.

From the repository root on a fresh checkout:

```sh
make install-tools
make tools-check
cargo fetch --locked
cargo fetch --locked --manifest-path qualification/Cargo.toml
make install-hooks
```

Setup may download pinned tools/crates; hook installation changes only the local
Git hook selection and refuses to replace other hooks. Auth’s Cargo commands inherit the caller’s network mode. The installed Wasm fixture
has an [IC Build limitation](../../qualification/ic-auth-host-conformance/README.md).

```sh
make toko-tooling-check
make test-request
make test-session-policy
make test-receiver-policy
make test-admission-capacity
make test-credential-profile
make test-memory
make test-issuer
make test-claims
make test-delegation-encoding
make test-wire
make test-wire-projection
make qualify-wire-admission
make test-verification
make qualify-certified-issuer
make qualify-root-delegation
make qualify-certified
make qualify-shared-verification
make lint-certified-qualification
make qualify-wire
make lint-wire-qualification
make lint-protocol
make doc-protocol
make check-protocol-wasm
make fmt-check
```

These are focused on the implemented protocol boundary. `test-request` includes
the scope/request integration files and the immutable-scope compile-fail doctest;
`test-session-policy` selects role narrowing and captured lease cases, including
both proof mechanisms using independent qualification signatures;
`test-receiver-policy` runs nineteen private policy tests, including thirteen
read/retry/login scenarios and six count/byte cases. For login-only changes select
`cargo test --locked -p ic-auth --lib receiver_policy::tests::login_`;
real proof-to-receiver integration is owned by `test-memory`, without a public
fact-assembly API;
`test-admission-capacity` covers six count/byte accounting scenarios;
`test-credential-profile` checks single-mechanism protected dispatch, while
`qualify-wire-admission` covers the combined graph through Wire/domain projection;
`test-memory` checks independent signed memory admission sequences and certified-only
retention, including failed proofs, proof-expired retries, unknown logout,
exact logout Wire result projections and bound enrollment readiness. It also
measures inspection iterator consumption and checks passive pagination through
retained sessions and fences, plus independent generated operation sequences and
a write-then-error negative control. Test-only publication interruptions require
terminal closure throughout admission, logout and pruning mutations;
the same target qualifies synchronous authorization projections, borrowed-receipt
compile failures, and two-issuer/multi-device isolation across two receivers for
both proof mechanisms. Five certified receiver cases use independently packed
claims and two OpenSSL-signed root-authorized issuers; Node checks the new frozen
records here and through the hosted `verify-fixtures` aggregate;
`test-issuer` runs thirty-six issuance/renewal/retention/publication API scenarios,
nine private fault/identity tests and one complete certified issuance fixture.
Independent Node/OpenSSL claims, renewal identity frames and Platform BLS proofs
check both proof mechanisms without a production signing service.
For renewal-only runtime checks select
`cargo test --locked -p ic-auth --features issuer-stable,ed25519-verification --test issuer_memory renewal`.
For pruning-only API checks select
`cargo test --locked -p ic-auth --features issuer-stable,ed25519-verification --test issuer_memory retention`.
For issuer-only feature compilation use
`cargo check --locked -p ic-auth --features issuer-stable`; it does not
enable the receiver or either verifier. It selects Wire/Serde for the preserved
renewal operation hash and Platform Types for browser principal derivation.
Native pruning checks cover the 32-record bound, reference counts, consumed-ID
epochs, exact quota release and interruption containment. Native certified
publication checks refusal ordering, failure containment and changing witnesses
with fixed claims; installed host transactions and recovery remain unqualified;
`test-claims` covers roles and canonical signed-claims bytes/refusals; `test-wire`
covers all four contracts, nested proofs and bounded refusal. `qualify-wire`
checks current Wire against independent frames and retained resource expectations; `lint-wire-qualification` checks that private
package. `test-verification` covers optional Ed25519/context/budget behavior and
immutable-result compile-fail cases. `test-delegation-encoding` checks certificate,
root-batch and issuer-leaf signing commitments and their bounded inputs.
`test-wire-projection` checks consuming Wire/domain conversion against retained
reference fields; `qualify-wire-admission` selects three real Wire-to-verifier
host-driver cases, including unsigned request policy and failure charging.
`qualify-certified-issuer` constructs real BLS
certificates outside Auth to check protected signer policy and trust-chain
refusals; its paired lint command is `lint-certified-qualification`.
`qualify-root-delegation` checks independent OpenSSL-signed root authorization,
revocation and proof bounds plus immutable-result API cases in that same package.
`qualify-certified` checks complete root/issuer proofs, common claims policy and
charging order. `qualify-shared-verification` selects the one real Ed25519/certified
shared-budget case using the qualification package's optional comparison feature.
The lint/docs/Wasm targets aggregate the three implemented protocol
packages with all implemented verification features and the memory receiver enabled; lint includes the optional native generator and Wasm proves its metadata
remains excluded even with the feature forwarded. For a narrower check,
use `cargo clippy --locked -p ic-auth-wire --all-targets --features typescript -- -D warnings`,
`RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-auth-wire --no-deps`,
or `cargo check --locked -p ic-auth-wire --lib --target wasm32-unknown-unknown`.
`check-protocol-wasm` requires the Rust toolchain's `wasm32-unknown-unknown` target
and proves compilation only. No full replacement, installed lifecycle or release
gate is implied. `make fmt` writes the same Rust/manifest selection checked by
`fmt-check`. The configured hook was exercised by the 0.1.0 source milestone.

`toko-tooling-check` admits product members/lock and the installed snapshot.
Its local qualification prerequisite uses the same shared declaration/lock
checker and compares its result with the installed receipt. `snapshot-check` is the narrower byte-integrity check retained
from bootstrap, without Cargo selection admission.

## Hosted CI

[Auth CI](../../.github/workflows/ci.yml) runs on main pushes, pull requests and
manual dispatch. Native contract tests run on Ubuntu 24.04 and macOS 15, including
both Apple Silicon and Intel. Rustup prepares the tracked toolchain and components;
the authenticated common-tools action restores caches, installs and admits the
pinned tools, then saves prepared caches before product checks. The Linux job runs the independent signing-fixture checks once with Node 24.21.0.

The CI aggregates `make test-product test-qualification verify-fixtures` select
each implemented product and qualification package once, enabling both complete
verifiers, the memory receiver and memory issuer. They do not replace
focused development checks.
Linux additionally checks formatting, product and qualification Clippy (including
the host harness and Wasm fixture), Rustdoc and Wasm library compilation. It
prepares the locked browser graph with `npm ci --ignore-scripts`, then runs
`check-client-generated`, `test-client` and `qualify-wire-browser`.
Both workspaces fetch their locked dependencies before validation. Snapshot and
Cargo pin admission precede the test aggregates; strict compiler warnings apply
throughout the native matrix.

The installed `run-validation-targets.sh --fail-fast` owns target dispatch,
per-target logs, timings and GitHub job summaries. CI explicitly sets
`VALIDATION_REPOSITORY_ROOT` to the checkout because the runner lives under
`vendor/toko-tooling`. Each invocation retains its own log directory beneath
`$RUNNER_TEMP/auth-validation`. The failure action selects that directory through
the input introduced by
[Toko Tooling #392](https://github.com/dragginzgame/toko-tooling/issues/392).

A separate Linux job installs pinned Actionlint, ShellCheck and Gitleaks, validates
the workflow, and scans complete Git history once with secrets fully redacted.
Actions are commit-pinned, repository permissions are read-only, and newer work
cancels the same workflow/ref. Shared failure evidence retains native logs and
security-tool setup/check logs for 14 days. Rustup, Cargo-fetch and npm setup logs
also use collector-selected names. For those commands and the security-tool
installer, `scripts/ci/setup.pl` passes a bounded, redacted failure excerpt to
`tee` and preserves the command’s exit status. Gitleaks
output stays fully redacted. Installed host/SDK lifecycle qualification remains a
separate gate; Wasm coverage here proves compilation only.

The CI wiring was exercised locally through the shared runner and a disposable
failure probe: a failing first target withheld later targets, a failed Cargo
fetch retained exit 42 through `tee`, and the action's collection shell archived
the selected logs, timings and summary. This does not claim hosted upload or
macOS execution of the changed workflow; those remain checks of the next CI run.

## Private browser client

`npm ci --prefix packages/client --ignore-scripts` prepares the separate locked
npm graph. `make test-client` runs strict TypeScript checking, ten SDK signing
cases through the internal attempt owner, six credential/admission/logout codec
cases, eight HTTP adapter cases and two public API boundary cases. Compile-time
negative checks keep operation stages out of consumer handles. For the HTTP-only check, run
`node --test --test-isolation=none --test-reporter=tap tests/http.test.ts` from `packages/client/`.
Node 24.21.0 runs erasable TypeScript directly;
no product build or registry delivery is introduced.
The local check uses Node's test runner with isolation disabled so individual
cases are reported. `IC_AUTH_BROWSER_EXECUTABLE=/absolute/path/to/chromium make
test-client-journal` runs fifty-eight native IndexedDB cases in an ephemeral two-tab
profile. Set `IC_AUTH_CLIENT_SUITE=journal`, `issuance`, `transport`, `admission`,
`cleanup`, `logout`, `http` or `readiness` for a focused group; the default runs all eight.
The twelve readiness cases cover coalescing, independent receivers, shared renewal,
64-operation admission, expiry retry bounds, reopening, explicit clock mapping,
logout fencing, timeout capacity release and captured identity/destination bindings.
Qualification-only esbuild 0.28.2 bundles the fixture in memory;
playwright-core 1.65.1 controls the supplied executable. Neither tool downloads a
browser. Chromium 141.0.7390.37 was exercised locally. These explicit gates are not
currently part of hosted CI. `make generate-client` generates the adopted
credential/admission/logout codecs and runtime from Rust schemas as JavaScript plus declarations;
`make check-client-generated` compares it with fresh generator/compiler output.
Generated scratch stays under `target/client-wire/`. See
[scope and setup](../../packages/client/README.md).

## Installed Auth qualification

`POCKET_IC_SERVER_URL=http://127.0.0.1:PORT make qualify-host` supplies prepared
tool paths and the fixture's Cargo identity to IC Build. The harness builds release Wasm once per process and installs
the owned bytes for issuer, receiver, enrollment, upgrade and cost tests on one simulated subnet with the provided
PocketIC 16.x server through Testkit. `make lint-host` checks its native harness
and Wasm fixture. These are explicit local gates; the hosted aggregate does not
run them or provision a server. `HOST_TEST_FILTER=receiver::` selects only the
receiver cases, including an intact-receiver control for valid proofs;
`HOST_TEST_FILTER=enrollment::` selects the authority-call/callback case. Node
24.21.0 regenerates independent host-time vectors before the tests. Host FS and
Host Artifacts 0.80.2 own held-file access and bounded vector reads. See [evidence and prerequisites](../../qualification/ic-auth-host-conformance/README.md).

`POCKET_IC_SERVER_URL=http://127.0.0.1:PORT IC_AUTH_BROWSER_EXECUTABLE=/absolute/path/to/chromium make qualify-client-http`
selects the separate installed HTTP harness. Five Chromium cases check real SDK
certificate acceptance, wrong-root/tamper refusal and old-request replay. Four
additional cases connect issuer reconciliation, durable journal, real admission,
protected application writes and logout on a single-subnet combined fixture. It uses the prepared client npm graph;
Host Process owns the bounded Node child, Playwright closes Chromium, and the caller retains server custody.
The `installed-http` and `installed-flow` groups are excluded from ordinary browser
tests and hosted aggregates. `lint-host` checks both native harnesses and the Wasm
fixture. Shared setup uses ordinary modules beneath explicit test entrypoints.

The `external-signer` selector covers ten account/browser signing recovery cases
using delayed/rejected real SDK signatures and native IndexedDB. It includes two
actual five-second timeout cases. Run with `IC_AUTH_CLIENT_SUITE=external-signer`
and the prepared-browser command. It qualifies Auth's adapter boundary, not a real
wallet/device protocol or persistent external key custody.

The `scheduler` browser selector checks cleanup scheduling, cancellation,
cross-tab leases and resumption; `cleanup` and `logout` retain the journal and
one-step effect cases.

The `multi-logout` selector checks three mixed-outcome cleanup cases across an
issuer and three receivers, with public admission creating the inventory. It covers
two-tab handoff, stale reply refusal, exactly eight attempts per unconfirmed/
unavailable destination, and retained uncertainty at the original deadline.
It uses an injected host clock and transport test doubles with real SDK signing.
Run with `IC_AUTH_CLIENT_SUITE=multi-logout` and the prepared browser.

The `disposal` selector covers seven terminal key-disposal cases with native
IndexedDB and host conditional-deletion test doubles. The `interruption` selector
also covers deletion before a lost callback reply, then retry without a signer.
Neither qualifies a production key store.

The `interruption` selector uses a dedicated disposable Chromium browser per case.
It obtains that browser's renderer process IDs through CDP, sends SIGKILL only to
those processes, observes the crash event and reopens the same context storage.
Its eight cases cover key-disposal retry, logout and issuance/renewal recovery, retained evidence and
logout fencing after reopening. Run with `IC_AUTH_CLIENT_SUITE=interruption` and the existing prepared-browser
command. This qualifies renderer loss, not whole-browser or operating-system loss.

The `admission-interruption` selector reuses that dedicated-browser crash helper
for four receiver admission cases: suspended signing, submitted/lost reply,
completed reply and logout after reopening. It checks exact request reuse, no
implicit submission, durable cleanup inventory and no post-logout admission.
Its host transport recorder is a test double; it does not establish remote effects.
Run with `IC_AUTH_CLIENT_SUITE=admission-interruption` and the prepared browser.

The focused installed receiver measurement lane uses
`HOST_TEST_FILTER='cost_observations --nocapture' make qualify-host`
with the prepared `POCKET_IC_SERVER_URL`. It reports six receiver and six issuer
synchronous operation instruction deltas, heap pages and fixture frame sizes.
The `receiver_cost_observations` and `issuer_cost_observations` filters select each
scenario independently. The `limit_cost_observations` filter adds exhausted
verification credit/refill and revoked pending issuer completion, with retained
state assertions. It is also selected by the broader `cost_observations` filter.
The `capacity_cost_observations` filter compares full and empty installed receivers
with the same valid second-device credential, proving no eviction or verification
charge at the known-login limit and retained capacity after logout. The
`replay_capacity_cost_observations` filter fills both replay slots for one login,
checks valid-proof refusal against an empty control, retained exact retry after
proof expiry, and replay fences after logout. See
[installed costs](installed-costs.md) for the exercised command, artifact identity,
observations and exclusions; these are not accepted production budgets.

## Dependency maintenance and LOC

```sh
make deps-list
make deps-update
make deps-update DEPENDENCY_PACKAGE=toko-tooling
toko-cloc -d 1
```

`deps-list` shows the independent `auth`, `wire` and `protocol-types` release groups.
Their package versions and changelogs are mapped in [the release group configuration](../../ci/cargo-release-groups.json).
The four private qualification packages appear outside those groups. `deps-update` runs Cargo in the
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
assembles all 37 fixtures independently and executes only the current generated
codecs with Node 24.21.0. No legacy checkout or Git objects are required. See the
[Wire evidence](wire-boundary.md) for exact scope.

The independent qualification workspace owns its catalog, lock and lints, uses
`license-file = "../LICENSE.md"`, inherits tooling and remains unpublished. The
root formatter includes it through `FORMAT_RUST_ARGS`. Product packages never
depend on qualification packages. The selected published Wire supplies the codec;
historical codec captures survive only as frame/threshold provenance.
Strict Ed25519 cases run through Platform under `test-verification`. Neither
workspace deliberately enables a legacy scalar mode or directly depends on Dalek.
Certified-only consumers use `certified-issuer-verification` without Ed25519;
certificate signing/encoding fixtures and range builders stay qualification-only.
Root-only consumers use `root-delegation-verification` for Platform ECDSA without
selecting BLS or Ed25519. Complete `certified-verification` composes root/issuer
checks with shared context and charging, without selecting the Ed25519 verifier.

See the reviewed upstream
[adoption guide](https://github.com/dragginzgame/toko-tooling/blob/08d63df7661910554ef72980037f55edd914aba0/docs/adoption/consuming-snapshots.md)
and [LOC setup](https://github.com/dragginzgame/toko-tooling/blob/08d63df7661910554ef72980037f55edd914aba0/docs/fleet/fleet-tools.md#cargo-workspace-loc-report).
