#!/usr/bin/env bash
# Explicit real-browser preparation is separate from ordinary offline checks.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
: "${IC_AUTH_BROWSER_EXECUTABLE:?Select an explicitly prepared Chromium executable}"
[[ "$IC_AUTH_BROWSER_EXECUTABLE" == /* && -x "$IC_AUTH_BROWSER_EXECUTABLE" ]] || {
    echo 'IC_AUTH_BROWSER_EXECUTABLE must be an absolute executable path' >&2; exit 1;
}
export IC_AUTH_BROWSER_EXECUTABLE CARGO_TARGET_DIR="$PWD/target" RUSTUP_AUTO_INSTALL=0
bash scripts/dev/client-tools.sh check
POCKET_IC_BIN="$(bash scripts/dev/testkit-tools.sh check)"
export POCKET_IC_BIN
npm --prefix packages/client run check:sdk
npm --prefix packages/client run build
cargo build --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown --release
IC_AUTH_QUALIFICATION_NODE="$(command -v node)"
export IC_AUTH_QUALIFICATION_NODE
export IC_AUTH_QUALIFICATION_WASM="$PWD/target/wasm32-unknown-unknown/release/ic_auth_qualification_canister.wasm"
mkdir -p target/portable-fixtures
IC_AUTH_QUALIFICATION_STATE_ROOT="$(mktemp -d "$PWD/target/portable-fixtures/browser-qualification.XXXXXX")"
export IC_AUTH_QUALIFICATION_STATE_ROOT TMPDIR="$IC_AUTH_QUALIFICATION_STATE_ROOT"
printf 'Browser qualification evidence retained: %s\n' "$IC_AUTH_QUALIFICATION_STATE_ROOT"
cargo test --locked --offline -p ic-auth-qualification --test signatures actual_browser_storage_signing_and_receiver_transport -- --ignored --nocapture
