#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
source scripts/ci/auth-dependency-policy.sh
printf 'MSRV wrapper Bash %s\n' "$BASH_VERSION"
export PATH="$root/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true
floor="$(bash scripts/dev/msrv-tools.sh floor)"
development="$(yq -p toml -o json '.' rust-toolchain.toml | jq -er '.toolchain.channel')"
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
bash scripts/dev/msrv-tools.sh check
bash scripts/ci/test-package-consumer.sh
mkdir -p "$root/target/portable-fixtures"
fixture="$(mktemp -d "$root/target/portable-fixtures/msrv.XXXXXX")"
printf 'MSRV package-consumer evidence retained: %s\n' "$fixture"
cp Cargo.toml "$fixture/original-workspace.toml"
cp Cargo.lock "$fixture/original-workspace.lock"

# Cargo owns package normalization. Check its real payloads outside the workspace
# so private test dependencies cannot supply features to a public consumer.
source scripts/release/packages.sh
CARGO_TARGET_DIR="$root/target/msrv-package" cargo +"$development" package \
    --locked --offline --allow-dirty --no-verify "${package_args[@]}"
CARGO_TARGET_DIR="$root/target" cargo +"$development" build --locked --offline -p ic-auth-tooling
for package in "${packages[@]}"; do
    archive="$root/target/msrv-package/package/tmp-crate/$package-$version.crate"
    digest="$(bash scripts/dev/run-host-file.sh hash-file "$archive" 8388608)"
    printf '%s  %s\n' "$digest" "$archive" >> "$fixture/package-identities.sha256"
    # Cargo archives normalize mtimes. Fresh extraction times prevent reuse of
    # stale local-package metadata in the shared target directory after edits.
    tar -xmzf "$archive" -C "$fixture"
done
auth="$fixture/ic-auth-$version"
types="$fixture/ic-auth-protocol-types-$version"
wire="$fixture/ic-auth-wire-$version"
package_inputs() {
    find "$auth" "$types" "$wire" ! -type d -print0 > "$fixture/input-paths"
    perl -0e 'print sort <STDIN>' < "$fixture/input-paths" > "$fixture/sorted-input-paths"
    printf '%s\0' "$root/Cargo.toml" "$root/Cargo.lock" \
        "$root/ci/host-file.env" "$root/rust-toolchain.toml" \
        "$root/scripts/dev/host-file-tools.sh" "$root/scripts/dev/run-host-file.sh" \
        "$root/crates/ic-auth/Cargo.toml" "$root/crates/ic-auth-protocol-types/Cargo.toml" "$root/crates/ic-auth-wire/Cargo.toml" \
        "$root/target/msrv-package/package/tmp-crate/ic-auth-$version.crate" \
        "$root/target/msrv-package/package/tmp-crate/ic-auth-protocol-types-$version.crate" \
        "$root/target/msrv-package/package/tmp-crate/ic-auth-wire-$version.crate" "$@" \
        >> "$fixture/sorted-input-paths"
    bash scripts/dev/run-host-file.sh hash-files0 8388608 4194304 4096 < "$fixture/sorted-input-paths"
}
package_inputs > "$fixture/archive-inputs"
# Keep normalized manifests and their locks byte-for-byte intact. Only this
# independent consumer selects the exact unpacked archives via registry patches.
all_features="$(yq -p toml -o json '.' "$auth/Cargo.toml" | jq -c '[.features | keys[] | "ic-auth/" + .] + ["ic-auth-wire/typescript"]')"
platform_version="$(yq -p toml -o json '.' "$types/Cargo.toml" | jq -er '.dependencies."ic-platform".version')"
cat > "$fixture/Cargo.toml" <<TOML
[workspace]
exclude = ["ic-auth-$version", "ic-auth-protocol-types-$version", "ic-auth-wire-$version"]

[package]
name = "ic-auth-msrv-consumer"
version = "0.0.0"
edition = "2024"
rust-version = "$floor"
publish = false

[dependencies]
ic-auth-protocol-types = "=$version"
ic-auth-wire = "=$version"
ic-auth = { version = "=$version", optional = true, default-features = false }
ic-platform = { version = "$platform_version", default-features = false, features = ["principal"] }

[features]
all = $all_features

[patch.crates-io]
ic-auth = { path = "ic-auth-$version" }
ic-auth-protocol-types = { path = "ic-auth-protocol-types-$version" }
ic-auth-wire = { path = "ic-auth-wire-$version" }
TOML
mkdir "$fixture/src"
cat > "$fixture/src/lib.rs" <<'RUST'
pub use ic_auth_protocol_types;
pub use ic_auth_wire;
pub fn platform_principal_identity(
    principal: ic_platform::Principal,
) -> ic_auth_protocol_types::Principal {
    principal
}
#[cfg(feature = "ic-auth")]
pub use ic_auth;
#[cfg(feature = "ic-auth")]
pub fn canonical_session_request(
    scopes: Vec<String>, ttl_secs: Option<u64>,
) -> Result<
    (Vec<ic_auth_protocol_types::ScopeLabel>, [u8; 32]),
    ic_auth::session_request::SessionRequestError,
> {
    let request = ic_auth::session_request::SessionRequest::new(scopes, ttl_secs)?;
    Ok((request.scopes().to_vec(), request.hash()))
}
RUST
cp "$fixture/original-workspace.lock" "$fixture/Cargo.lock"
cargo +"$development" metadata --offline --format-version 1 --all-features \
    --manifest-path "$fixture/Cargo.toml" > "$fixture/metadata.json"
bash scripts/ci/check-package-consumer.sh "$fixture/original-workspace.lock" "$fixture" "$version"
package_inputs > "$fixture/resolved-archive-inputs"
cmp "$fixture/archive-inputs" "$fixture/resolved-archive-inputs"
package_inputs "$fixture/Cargo.toml" "$fixture/Cargo.lock" "$fixture/src/lib.rs" > "$fixture/inputs-before"
cmp Cargo.toml "$fixture/original-workspace.toml"
cmp Cargo.lock "$fixture/original-workspace.lock"
export CARGO_TARGET_DIR="$root/target/msrv"
selections="$(bash scripts/ci/auth-feature-selections.sh "$auth/Cargo.toml")"
for target in native wasm32-unknown-unknown; do
    target_args=()
    [[ "$target" == native ]] || target_args=(--target "$target")
    # Bash 3.2 treats an empty array as unset under nounset. Keep zero native
    # target arguments rather than passing an empty string or weakening -u.
    cargo +"$floor" check --locked --offline --lib --no-default-features \
        --manifest-path "$fixture/Cargo.toml" ${target_args[@]+"${target_args[@]}"}
    while IFS= read -r feature; do
        feature_args=(--no-default-features --features ic-auth)
        case "$feature" in
            default) ;;
            all) feature_args=(--all-features) ;;
            *) feature_args=(--no-default-features --features "ic-auth/${feature//,/,ic-auth/}") ;;
        esac
        printf 'MSRV %s: ic-auth %s on %s\n' "$floor" "$feature" "$target"
        cargo +"$floor" check --locked --offline --lib --manifest-path "$fixture/Cargo.toml" \
            ${target_args[@]+"${target_args[@]}"} "${feature_args[@]}"
        if [[ "$feature" == default ]]; then
            cargo +"$floor" tree --locked --offline --manifest-path "$fixture/Cargo.toml" \
                ${target_args[@]+"${target_args[@]}"} "${feature_args[@]}" \
                --edges normal,build --prefix none --format '{p}' > "$fixture/pure-request-$target.graph"
            if rg "$(auth_dependency_pattern pure)" "$fixture/pure-request-$target.graph"; then
                echo 'isolated pure request consumer pulls Candid, cryptographic or session storage dependencies' >&2
                exit 1
            fi
        fi
    done <<< "$selections"
done
# Internal applications use the same floor with separate native/Wasm checks.
# Runtime qualification remains test-qualification.
cargo +"$floor" check --locked --offline -p ic-auth-tooling -p ic-auth-qualification -p ic-auth-qualification-contracts -p ic-auth-qualification-fixtures -p ic-auth-feature-unification --all-targets --all-features
cargo +"$floor" check --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown
package_inputs "$fixture/Cargo.toml" "$fixture/Cargo.lock" "$fixture/src/lib.rs" > "$fixture/inputs-after"
cmp "$fixture/inputs-before" "$fixture/inputs-after"
printf 'Package archives, unpacked sources, manifests and locks remained unchanged\n'
