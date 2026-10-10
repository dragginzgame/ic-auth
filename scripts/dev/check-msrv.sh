#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
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
CARGO_TARGET_DIR="$root/target/msrv-package" cargo +"$development" package \
    --locked --offline --allow-dirty --no-verify -p ic-auth-protocol-types -p ic-auth
CARGO_TARGET_DIR="$root/target" cargo +"$development" build --locked --offline -p ic-auth-tooling
for package in ic-auth-protocol-types ic-auth; do
    archive="$root/target/msrv-package/package/tmp-crate/$package-$version.crate"
    digest="$("$root/target/debug/ic-auth-tooling" hash-file "$archive" 8388608)"
    printf '%s  %s\n' "$digest" "$archive" >> "$fixture/package-identities.sha256"
    tar -xzf "$archive" -C "$fixture"
done
auth="$fixture/ic-auth-$version"
types="$fixture/ic-auth-protocol-types-$version"
package_inputs() {
    find "$auth" "$types" ! -type d -print0 > "$fixture/input-paths"
    perl -0e 'print sort <STDIN>' < "$fixture/input-paths" > "$fixture/sorted-input-paths"
    while IFS= read -r -d '' file; do
        digest="$("$root/target/debug/ic-auth-tooling" hash-file "$file" 8388608)" || return 1
        printf '%s\t%s\0' "$digest" "$file"
    done < "$fixture/sorted-input-paths"
    for file in "$root/Cargo.toml" "$root/Cargo.lock" \
        "$root/crates/ic-auth/Cargo.toml" "$root/crates/ic-auth-protocol-types/Cargo.toml" \
        "$root/target/msrv-package/package/tmp-crate/ic-auth-$version.crate" \
        "$root/target/msrv-package/package/tmp-crate/ic-auth-protocol-types-$version.crate" "$@"; do
        digest="$("$root/target/debug/ic-auth-tooling" hash-file "$file" 8388608)" || return 1
        printf '%s\t%s\0' "$digest" "$file"
    done
}
package_inputs > "$fixture/archive-inputs"
# Keep normalized manifests and their locks byte-for-byte intact. Only this
# independent consumer selects the exact unpacked archives via registry patches.
all_features="$(yq -p toml -o json '.' "$auth/Cargo.toml" | jq -c '[.features | keys[] | "ic-auth/" + .]')"
cat > "$fixture/Cargo.toml" <<TOML
[workspace]
exclude = ["ic-auth-$version", "ic-auth-protocol-types-$version"]

[package]
name = "ic-auth-msrv-consumer"
version = "0.0.0"
edition = "2024"
rust-version = "$floor"
publish = false

[dependencies]
ic-auth-protocol-types = "=$version"
ic-auth = { version = "=$version", optional = true, default-features = false }

[features]
all = $all_features

[patch.crates-io]
ic-auth = { path = "ic-auth-$version" }
ic-auth-protocol-types = { path = "ic-auth-protocol-types-$version" }
TOML
mkdir "$fixture/src"
cat > "$fixture/src/lib.rs" <<'RUST'
pub use ic_auth_protocol_types;
#[cfg(feature = "ic-auth")]
pub use ic_auth;
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
for target in native wasm32-unknown-unknown; do
    target_args=()
    [[ "$target" == native ]] || target_args=(--target "$target")
    # Bash 3.2 treats an empty array as unset under nounset. Keep zero native
    # target arguments rather than passing an empty string or weakening -u.
    cargo +"$floor" check --locked --offline --lib --no-default-features \
        --manifest-path "$fixture/Cargo.toml" ${target_args[@]+"${target_args[@]}"}
    for feature in default canister-signature-verification canister-signature-preparation token-verification sessions all; do
        feature_args=(--no-default-features --features ic-auth)
        case "$feature" in
            default) ;;
            all) feature_args=(--all-features) ;;
            *) feature_args=(--no-default-features --features "ic-auth/$feature") ;;
        esac
        printf 'MSRV %s: ic-auth %s on %s\n' "$floor" "$feature" "$target"
        cargo +"$floor" check --locked --offline --lib --manifest-path "$fixture/Cargo.toml" \
            ${target_args[@]+"${target_args[@]}"} "${feature_args[@]}"
    done
done
# Internal applications have the same qualified floor, checked separately from
# the public dependency path. Runtime qualification remains test-qualification.
cargo +"$floor" check --locked --offline -p ic-auth-tooling -p ic-auth-qualification --all-targets --all-features
cargo +"$floor" check --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown
package_inputs "$fixture/Cargo.toml" "$fixture/Cargo.lock" "$fixture/src/lib.rs" > "$fixture/inputs-after"
cmp "$fixture/inputs-before" "$fixture/inputs-after"
printf 'Package archives, unpacked sources, manifests and locks remained unchanged\n'
