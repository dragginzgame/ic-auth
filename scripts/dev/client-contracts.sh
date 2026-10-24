#!/usr/bin/env bash
# Offline generation/comparison from Rust-owned IC Wire contracts.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
case "${1:-}" in check|generate) ;; *) echo 'usage: client-contracts.sh check|generate' >&2; exit 2 ;; esac
export CARGO_TARGET_DIR="$root/target" RUSTUP_AUTO_INSTALL=0
mkdir -p target/browser-client
candidate="$(mktemp -d "$root/target/browser-client/contracts.XXXXXX")"
# Build once, then hash the same ordered input list in one native batch per seal.
cargo build --quiet --locked --offline -p ic-auth-tooling
tool="$CARGO_TARGET_DIR/debug/ic-auth-tooling"
printf '%s\0' Cargo.toml Cargo.lock crates/ic-auth-protocol-types/Cargo.toml \
    apps/tooling/Cargo.toml apps/tooling/src/*.rs crates/ic-auth-protocol-types/src/*.rs \
    crates/ic-auth-wire/Cargo.toml crates/ic-auth-wire/src/*.rs \
    ci/host-file.env rust-toolchain.toml scripts/dev/host-file-tools.sh scripts/dev/run-host-file.sh \
    scripts/dev/client-contracts.sh > "$candidate/input-paths0"
bash scripts/dev/run-host-file.sh hash-files0 4194304 4194304 4096 < "$candidate/input-paths0" > "$candidate/inputs.tsv0"
"$tool" client-contracts generate "$candidate"
bash scripts/dev/run-host-file.sh hash-files0 4194304 4194304 4096 < "$candidate/input-paths0" > "$candidate/inputs-after.tsv0"
cmp "$candidate/inputs.tsv0" "$candidate/inputs-after.tsv0"
case "${1:-}" in
    check)
        for name in credential.ts establish.ts read.ts logout.ts wire-runtime.ts; do
            cmp "$candidate/$name" "packages/client/src/generated/$name" || {
                status=$?
                printf 'Client contract check refused: packages/client/src/generated/%s is missing, unreadable or differs from the locked generator.\n' "$name" >&2
                echo 'Run make generate-client-contracts, review the generated files with the dependency/DTO change, then rerun make test-client.' >&2
                printf 'Generated candidate retained: %s\n' "$candidate" >&2
                exit "$status"
            }
        done ;;
    generate) for name in credential.ts establish.ts read.ts logout.ts wire-runtime.ts; do cp "$candidate/$name" packages/client/src/generated/; done ;;
esac
for name in credential logout admission; do
    cp "$candidate/$name-vector.hex" "target/browser-client/$name-vector.hex"
done
