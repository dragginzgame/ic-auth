#!/usr/bin/env bash
set -euo pipefail

# Offline qualification only. Setup prepares the complete pinned IC bundle;
# this wrapper authenticates it and never downloads a server during tests.
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export CARGO_TARGET_DIR="$PWD/target" RUSTUP_AUTO_INSTALL=0
verified_bin="$(bash scripts/dev/install-ic-tools.sh --consumer "$PWD" --pins ci/ic-auth-tools.tsv --check)"
bash scripts/ci/check-pocketic-alignment.sh --manifest Cargo.toml --pins ci/ic-auth-tools.tsv > /dev/null
export POCKET_IC_BIN="$verified_bin/pocket-ic"
export IC_AUTH_QUALIFICATION_WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/ic_auth_qualification_canister.wasm"
mkdir -p "$CARGO_TARGET_DIR/portable-fixtures"
IC_AUTH_QUALIFICATION_STATE_ROOT="$(mktemp -d "$CARGO_TARGET_DIR/portable-fixtures/qualification.XXXXXX")"
export IC_AUTH_QUALIFICATION_STATE_ROOT TMPDIR="$IC_AUTH_QUALIFICATION_STATE_ROOT"
printf 'ic-testkit qualification state retained: %s\n' "$IC_AUTH_QUALIFICATION_STATE_ROOT"
cargo test --locked --offline -p ic-auth-qualification --test signatures -- --nocapture
