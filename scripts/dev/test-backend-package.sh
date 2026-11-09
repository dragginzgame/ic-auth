#!/usr/bin/env bash
# Local source delivery qualification; no registry upload or implicit preparation.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
mkdir -p target/portable-fixtures
evidence="$(mktemp -d "$root/target/portable-fixtures/backend-package.XXXXXX")"
printf 'Backend package evidence retained: %s\n' "$evidence"
tool="$(bash scripts/dev/host-file-tools.sh check)"
version="$(yq -p toml -o json '.' crates/ic-auth/Cargo.toml | jq -er '.package.version')"
protocol_version="$(yq -p toml -o json '.' crates/ic-auth-protocol-types/Cargo.toml | jq -er '.package.version')"
floor="$(bash scripts/dev/msrv-tools.sh floor)"
name="ic-auth-backend-conformance-$version"
consumer="$evidence/consumer"
bundle="$consumer/test-support/$name"
scenarios="$consumer/test-support"
export CARGO_TARGET_DIR="$evidence/target"
mkdir -p "$consumer/src" "$consumer/test-support"
{
    rg --files crates/ic-auth-backend-conformance -0
    printf '%s\0' Cargo.toml Cargo.lock crates/ic-auth/Cargo.toml crates/ic-auth-protocol-types/Cargo.toml rust-toolchain.toml ci/host-file.env \
        scripts/dev/test-backend-package.sh scripts/ci/check-backend-package-graph.sh \
        scripts/ci/test-backend-package-graph.sh scripts/ci/auth-dependency-policy.sh
} > "$evidence/inputs.nul"
inputs() { "$tool" hash-files0 8388608 4194304 4096 < "$evidence/inputs.nul"; }
inputs > "$evidence/before.hashes"
source_revision="$(git rev-parse HEAD)"
git status --porcelain > "$evidence/source-status.txt"
run() {
    local stage="$1"; shift
    "$@" > "$evidence/$stage.stdout.log" 2> "$evidence/$stage.stderr.log" || {
        local status=$?
        cat "$evidence/$stage.stderr.log" >&2
        printf 'Backend package stage %s failed (%s); evidence: %s\n' "$stage" "$status" "$evidence" >&2
        exit "$status"
    }
}
run package cargo package --locked --offline --allow-dirty --no-verify -p ic-auth-backend-conformance
# Use Cargo's final delivery archive, rather than its staging copy. Only this
# freshly produced local archive is extracted into the private evidence directory.
archive="$CARGO_TARGET_DIR/package/$name.crate"
digest="$("$tool" hash-file "$archive" 8388608)"
run extract tar -xmzf "$archive" -C "$consumer/test-support"
cat > "$consumer/Cargo.toml" <<TOML
[workspace]
exclude = ["test-support/$name"]
[package]
name = "auth-backend-package-consumer"
version = "0.0.0"
edition = "2024"
rust-version = "$floor"
publish = false
[dependencies]
ic-auth = { version = "=$version", default-features = false, features = ["session-restore", "portable-signature-verification"] }
ic-auth-protocol-types = "=$protocol_version"
[dev-dependencies]
ic-auth-backend-conformance = { path = "test-support/$name" }
TOML
# Keep the delivered package excluded from the real consumer. A separate
# generated workspace can exercise its own dev-dependencies without changing
# any extracted source or inheriting Auth's surrounding development workspace.
cat > "$scenarios/Cargo.toml" <<TOML
[workspace]
members = ["$name"]
resolver = "3"
TOML
cat > "$consumer/src/lib.rs" <<'RUST'
#[cfg(test)]
pub mod identity {
    use ic_auth::session::{SessionError, SessionStore};
    use ic_auth_backend_conformance::Backend;
    // This type-checks only when the source bundle implements the same registry
    // Auth trait and error identities as the consumer's direct dependency.
    pub fn read<B: Backend>(backend: &mut B) -> Result<(), SessionError> {
        SessionStore::read(backend, |_| Ok(()))
    }
}
RUST
cp Cargo.lock "$consumer/Cargo.lock"
run metadata cargo metadata --offline --format-version 1 --manifest-path "$consumer/Cargo.toml"
cp "$evidence/metadata.stdout.log" "$evidence/metadata.json"
bash scripts/ci/check-backend-package-graph.sh "$evidence/metadata.json" "$bundle/Cargo.toml" "$version" "$protocol_version"
run graph-refusals bash scripts/ci/test-backend-package-graph.sh "$evidence/metadata.json" \
    "$bundle/Cargo.toml" "$version" "$protocol_version" "$evidence/graph-refusals"
cp "$consumer/Cargo.lock" "$scenarios/Cargo.lock"
run scenario-metadata cargo metadata --offline --format-version 1 --manifest-path "$scenarios/Cargo.toml"
jq -e --slurpfile consumer "$evidence/metadata.json" '
    . as $scenario
    | ([.packages[] | select(.name == "ic-auth-backend-conformance") | .id] == .workspace_members)
    and all(["ic-auth", "ic-auth-protocol-types", "ic-platform", "ic-platform-types"][];
        . as $name
        | [$scenario.packages[] | select(.name == $name) | {name,version,source}]
          == [$consumer[0].packages[] | select(.name == $name) | {name,version,source}])
' "$evidence/scenario-metadata.stdout.log" > /dev/null

{
    find "$bundle" ! -type d -print0
    printf '%s\0' "$consumer/Cargo.toml" "$consumer/Cargo.lock" "$consumer/src/lib.rs" "$evidence/metadata.json" \
        "$scenarios/Cargo.toml" "$scenarios/Cargo.lock" "$evidence/scenario-metadata.stdout.log"
} > "$evidence/consumer-inputs.nul"
"$tool" hash-files0 8388608 4194304 4096 < "$evidence/consumer-inputs.nul" > "$evidence/consumer-before.hashes"
run consumer cargo test --locked --offline --manifest-path "$consumer/Cargo.toml"
# The package's own dev-dependencies belong only to its isolated test graph.
run scenarios cargo test --locked --offline --manifest-path "$scenarios/Cargo.toml"
"$tool" hash-files0 8388608 4194304 4096 < "$evidence/consumer-inputs.nul" > "$evidence/consumer-after.hashes"
cmp "$evidence/consumer-before.hashes" "$evidence/consumer-after.hashes"
inputs > "$evidence/after.hashes"
cmp "$evidence/before.hashes" "$evidence/after.hashes"
[[ "$("$tool" hash-file "$archive" 8388608)" == "$digest" ]]
jq -n --arg source "$source_revision" --arg version "$version" --arg protocol "$protocol_version" --arg archive "$archive" \
    --arg sha256 "$digest" --argjson dirty "$(test ! -s "$evidence/source-status.txt" && echo false || echo true)" \
    '{source_revision: $source, dirty: $dirty, engine_version: $version, protocol_version: $protocol, archive: $archive,
      sha256: $sha256, qualification: "registry-consumer-and-packaged-scenarios"}' > "$evidence/delivery.json"
printf 'Backend archive, registry identities, consumer and packaged scenarios qualified: %s\n' "$evidence/delivery.json"
