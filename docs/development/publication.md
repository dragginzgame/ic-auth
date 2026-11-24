# Crates.io publication scope

This release publishes Auth 0.49.3 and Auth Wire 0.49.0 to crates.io. Both depend on the published protocol-types 0.44.0. All three product crates use `publish = true`.
Publication covers the volatile and stable-checkpoint libraries.

| Product crate | Published responsibility | In-repository prerequisite |
| --- | --- | --- |
| `ic-auth-protocol-types` | Passive validated authentication protocol values | None |
| `ic-auth` | Authentication verification and bounded volatile receiver/issuer decisions | `ic-auth-protocol-types` |
| `ic-auth-wire` | Credential, admission, read and logout Wire contracts; optional TypeScript generation | `ic-auth-protocol-types` |

Publish the selected protocol-types version first. Once that exact required
version is available from crates.io, Auth and Auth Wire can publish independently;
neither depends on the other. All product-to-product Cargo dependencies must retain
explicit registry version requirements alongside their local development paths.
Changed supplier versions must be available before dependent uploads. Do not bump
or republish an unchanged product solely to make all three version numbers match.

The independent `qualification/` workspace is permanently outside this publication
scope: `ic-auth-certified-conformance`, `ic-auth-host-conformance`,
`ic-auth-host-fixture` and `ic-auth-wire-conformance` remain `publish = false`.
Product packages must not acquire dependencies on these packages. The private
`@dragginzgame/ic-auth-client` browser package is not a Rust crate and is not a
crates.io publication candidate; npm delivery requires its own future decision.
Do not introduce empty crates, endpoint installers or storage adapters to expand
this inventory.

## Upload authorization boundary

At the authorized publication milestone, review each affected product package's
actual archive and public feature/dependency surface, confirm the intended
crates.io destination, and qualify registry resolution in dependency order.
Verify package contents, repository/documentation metadata, required source and
fixtures, and consumer compilation against uploaded supplier versions. Recheck
crates.io name ownership and existing versions then; this plan does not claim
that any name or version is currently available. Confirm each upload before
publishing its dependents.

This release uses Cargo directly with exact package selectors, locked dependencies
and all features. No local publication automation is installed. Future shared
entrypoints must be adopted through the authenticated Toko exporter. Production deployment, consumer migration and live-storage changes
remain separate from crates.io delivery. The stable features use published `ic-storage` as an optional dependency.

## Validation and recovery sequence

Prepare the final intended dependency/version selection and required tools before
expensive validation. For a release affecting the current complete Auth surface,
run the existing product checks once through the installed shared runner:

```sh
VALIDATION_REPOSITORY_ROOT="$PWD" \
VALIDATION_LOG_DIR="$PWD/target/release-validation" \
bash vendor/toko-tooling/scripts/ci/run-validation-targets.sh --fail-fast \
  test-product test-qualification verify-fixtures fmt-check lint-protocol \
  lint-certified-qualification lint-wire-qualification lint-host doc-protocol \
  check-protocol-wasm check-client-generated test-client qualify-wire-browser
```

The same runner and leaf selections are exercised by local CI qualification.
Installed behavior has a separate guarantee: with the documented PocketIC server
and Chromium executable prepared, run `qualify-host` and `qualify-client-http`
through that runner with the same retained log root. Those commands and environment
inputs are documented in [installed qualification](tooling.md#installed-auth-qualification).
Retain supported-host CI evidence separately; a local Linux pass does not qualify
macOS. Select narrower release scope only when all affected consumers and required
guarantees are accounted for.

After validation, perform the final-source and archive checks described above for
each changed publishable package, then upload in dependency order and verify the
registry artifacts. Archive tests establish the shipped package's behavior; they
do not require another full workspace/installed pass solely because an upload is
next. CI success, timing logs and compiler-cache hits are not transferable release
proofs. No automatic evidence reuse or resume is installed.

On failure, retain the original logs and inspect the failed phase. A setup failure
needs setup recovery; a validation failure needs the affected checks after repair.
Changed source, locks, compiler or fixtures invalidate their affected validation.
If an upload's outcome is uncertain, inspect the registry version and checksum
before retrying; do not bump versions or repeat the whole release to discover
whether the upload succeeded. Shared release orchestration remains a future
qualified adoption, not a second repository-local release engine.
