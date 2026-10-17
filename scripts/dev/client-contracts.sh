#!/usr/bin/env bash
# Offline generation/comparison from Rust DTOs and locked SDK bindgen.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
case "${1:-}" in check|generate) ;; *) echo 'usage: client-contracts.sh check|generate' >&2; exit 2 ;; esac
export CARGO_TARGET_DIR="$root/target" RUSTUP_AUTO_INSTALL=0
bash scripts/dev/client-tools.sh check
mkdir -p target/browser-client
candidate="$(mktemp -d "$root/target/browser-client/contracts.XXXXXX")"
# Build once, then hash the same ordered input list in one native batch per seal.
cargo build --quiet --locked --offline -p ic-auth-tooling
tool="$CARGO_TARGET_DIR/debug/ic-auth-tooling"
printf '%s\0' Cargo.toml Cargo.lock crates/ic-auth-protocol-types/Cargo.toml \
    apps/tooling/Cargo.toml apps/tooling/src/*.rs crates/ic-auth-protocol-types/src/*.rs \
    apps/qualification/contracts/Cargo.toml apps/qualification/contracts/src/*.rs \
    packages/client/package.json packages/client/package-lock.json packages/client/.nvmrc packages/client/.npmrc \
    scripts/dev/client-contracts.sh > "$candidate/input-paths0"
"$tool" hash-files0 4194304 < "$candidate/input-paths0" > "$candidate/inputs.tsv0"
{
    node --version
    npm --version
    packages/client/node_modules/.bin/tsc --version
    packages/client/node_modules/.bin/icp-bindgen --version
} > "$candidate/tools.txt"
"$tool" client-contracts generate "$candidate"
packages/client/node_modules/.bin/icp-bindgen --did-file "$candidate/protocol.did" --out-dir "$candidate" \
    --actor-disabled --declarations-root-exports --declarations-typescript --declarations-flat
"$tool" hash-files0 4194304 < "$candidate/input-paths0" > "$candidate/inputs-after.tsv0"
cmp "$candidate/inputs.tsv0" "$candidate/inputs-after.tsv0"
case "${1:-}" in
    check) for name in protocol.did protocol.did.ts; do cmp "$candidate/$name" "packages/client/src/generated/$name"; done ;;
    generate) cp "$candidate/protocol.did" "$candidate/protocol.did.ts" packages/client/src/generated/ ;;
esac
for name in request credential logout; do
    cp "$candidate/$name-vector.hex" "target/browser-client/$name-vector.hex"
done
