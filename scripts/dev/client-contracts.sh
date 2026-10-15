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
# Retain exact generation inputs through the existing Host artifact utility.
for path in Cargo.toml Cargo.lock crates/ic-auth-protocol-types/Cargo.toml \
    apps/tooling/Cargo.toml apps/tooling/src/*.rs crates/ic-auth-protocol-types/src/*.rs \
    packages/client/package.json packages/client/package-lock.json packages/client/.nvmrc packages/client/.npmrc \
    scripts/dev/client-contracts.sh; do
    digest="$(bash scripts/dev/run-host-tooling.sh hash-file "$path" 4194304)"
    printf '%s\t%s\n' "$digest" "$path"
done > "$candidate/inputs.tsv"
{
    node --version
    npm --version
    packages/client/node_modules/.bin/tsc --version
    packages/client/node_modules/.bin/icp-bindgen --version
} > "$candidate/tools.txt"
cargo run --quiet --locked --offline -p ic-auth-tooling -- client-contracts generate "$candidate"
packages/client/node_modules/.bin/icp-bindgen --did-file "$candidate/protocol.did" --out-dir "$candidate" \
    --actor-disabled --declarations-root-exports --declarations-typescript --declarations-flat
case "${1:-}" in
    check) for name in protocol.did protocol.did.ts; do cmp "$candidate/$name" "packages/client/src/generated/$name"; done ;;
    generate) cp "$candidate/protocol.did" "$candidate/protocol.did.ts" packages/client/src/generated/ ;;
esac
for name in request credential logout; do
    cp "$candidate/$name-vector.hex" "target/browser-client/$name-vector.hex"
done
