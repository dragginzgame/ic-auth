#!/usr/bin/env bash
# Fixture-only bindings from the actual compiled canister, never public protocol DTOs.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
case "${1:-}" in check|generate) ;; *) echo 'usage: qualification-contracts.sh check|generate' >&2; exit 2 ;; esac
export CARGO_TARGET_DIR="$root/target" RUSTUP_AUTO_INSTALL=0 PATH="$root/.tools/host/bin:$PATH"
bash scripts/dev/client-tools.sh check
# shellcheck source=ci/tool-versions.env
source ci/tool-versions.env
extractor="$(bash scripts/dev/install-rust-tools.sh --consumer "$root" --package candid-extractor \
    --version "$SHARED_TOOLING_CANDID_EXTRACTOR_VERSION" --bin candid-extractor --profile release --check)"
cargo build --quiet --locked --offline -p ic-auth-tooling
tool="$CARGO_TARGET_DIR/debug/ic-auth-tooling"
wasm="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/ic_auth_qualification_canister.wasm"
mkdir -p "$root/target/browser-client"
candidate="$(mktemp -d "$root/target/browser-client/qualification-contracts.XXXXXX")"
"$tool" hash-file "$wasm" 16777216 > "$candidate/wasm-before.sha256"
"$extractor" "$wasm" > "$candidate/canister.did"
"$tool" hash-file "$candidate/canister.did" 1048576 > "$candidate/candid.sha256"
packages/client/node_modules/.bin/icp-bindgen --did-file "$candidate/canister.did" --out-dir "$candidate" \
    --actor-disabled --declarations-root-exports --declarations-typescript --declarations-flat
"$tool" hash-file "$wasm" 16777216 > "$candidate/wasm-after.sha256"
cmp "$candidate/wasm-before.sha256" "$candidate/wasm-after.sha256"
destination=packages/client/test/qualification/generated
case "$1" in
    check) for name in canister.did canister.did.ts; do cmp "$candidate/$name" "$destination/$name"; done ;;
    generate) mkdir -p "$destination"; cp "$candidate/canister.did" "$candidate/canister.did.ts" "$destination/" ;;
esac
