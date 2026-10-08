#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true
floor="$(bash scripts/dev/msrv-tools.sh floor)"
development="$(yq -p toml -o json '.' rust-toolchain.toml | jq -er '.toolchain.channel')"
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
bash scripts/dev/msrv-tools.sh check
mkdir -p "$root/target/portable-fixtures"
fixture="$(mktemp -d "$root/target/portable-fixtures/msrv.XXXXXX")"
printf 'MSRV package-consumer evidence retained: %s\n' "$fixture"

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
    printf '\n[workspace]\n' >> "$fixture/$package-$version/Cargo.toml"
done
auth="$fixture/ic-auth-$version"
types="$fixture/ic-auth-protocol-types-$version"
# Stage the matching package, rather than testing the registry's previous payload.
# Only local fixture paths/source identity change; all external lock selections stay.
IC_AUTH_MSRV_TYPES_PATH="../ic-auth-protocol-types-$version" perl -0pi -e '
    my $n = s{(\[dependencies\.ic-auth-protocol-types\]\n)}{$1 . "path = \"$ENV{IC_AUTH_MSRV_TYPES_PATH}\"\n"}e;
    die "expected one protocol dependency section\n" unless $n == 1;
' "$auth/Cargo.toml"
perl -0pi -e '
    my $n = s{(\[\[package\]\]\nname = "ic-auth-protocol-types"\nversion = "[^"]+"\n)source = "[^"]+"\nchecksum = "[a-f0-9]+"\n}{$1}g;
    die "expected one protocol registry lock entry\n" unless $n == 1;
' "$auth/Cargo.lock"
# Packaging may change the local source identity, never registry selections.
yq -p toml -o json '.' "$root/Cargo.lock" > "$fixture/workspace-lock.json"
for package_root in "$types" "$auth"; do
    yq -p toml -o json '.' "$package_root/Cargo.lock" > "$fixture/package-lock.json"
    jq -se '
        .[0].package as $selected
        | all(.[1].package[] | select(.source != null);
            . as $entry | any($selected[];
                .name == $entry.name and .version == $entry.version
                and .source == $entry.source and .checksum == $entry.checksum))
    ' "$fixture/workspace-lock.json" "$fixture/package-lock.json" > /dev/null || {
        echo 'package consumer changed an external lock selection' >&2; exit 1;
    }
done
export CARGO_TARGET_DIR="$root/target/msrv"
for target in native wasm32-unknown-unknown; do
    target_args=()
    [[ "$target" == native ]] || target_args=(--target "$target")
    cargo +"$floor" check --locked --offline --lib --manifest-path "$types/Cargo.toml" "${target_args[@]}"
    for feature in default canister-signature-verification canister-signature-preparation token-verification sessions all; do
        feature_args=(--no-default-features)
        case "$feature" in
            default) ;;
            all) feature_args=(--all-features) ;;
            *) feature_args+=(--features "$feature") ;;
        esac
        printf 'MSRV %s: ic-auth %s on %s\n' "$floor" "$feature" "$target"
        cargo +"$floor" check --locked --offline --lib --manifest-path "$auth/Cargo.toml" \
            "${target_args[@]}" "${feature_args[@]}"
    done
done
# Internal applications have the same qualified floor, checked separately from
# the public dependency path. Runtime qualification remains test-qualification.
cargo +"$floor" check --locked --offline -p ic-auth-tooling -p ic-auth-qualification --all-targets --all-features
cargo +"$floor" check --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown
