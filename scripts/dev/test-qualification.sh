#!/usr/bin/env bash
set -euo pipefail

# Offline qualification only. Testkit admits its prepared server; tests never
# download it or select a retained shared-bundle executable.
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export CARGO_TARGET_DIR="$PWD/target" RUSTUP_AUTO_INSTALL=0
POCKET_IC_BIN="$(bash scripts/dev/testkit-tools.sh check)"
export POCKET_IC_BIN
# This case executes the actual SDK logout adapter against the live fixture.
# Admit already prepared client tools and rebuild source; never install here.
bash scripts/dev/client-tools.sh check
bash scripts/dev/client-contracts.sh check
npm --prefix packages/client run build
IC_AUTH_QUALIFICATION_NODE="$(command -v node)"
export IC_AUTH_QUALIFICATION_NODE
export IC_AUTH_QUALIFICATION_WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/ic_auth_qualification_canister.wasm"
mkdir -p "$CARGO_TARGET_DIR/portable-fixtures"
IC_AUTH_QUALIFICATION_STATE_ROOT="$(mktemp -d "$CARGO_TARGET_DIR/portable-fixtures/qualification.XXXXXX")"
export IC_AUTH_QUALIFICATION_STATE_ROOT
# PocketIC uses Unix sockets with a small absolute-path limit. Keep durable
# state/logs above; only transport scratch needs this short system path.
socket_scratch=$(mktemp -d /tmp/ic-auth-pic.XXXXXX)
export TMPDIR="$socket_scratch"
printf '%s\n' "$socket_scratch" > "$IC_AUTH_QUALIFICATION_STATE_ROOT/socket-scratch"
cleanup_socket_scratch() {
    local status=$?
    if [[ $status == 0 ]]; then rm -rf "$socket_scratch"
    else printf 'PocketIC socket scratch retained: %s\n' "$socket_scratch" >&2; fi
}
trap cleanup_socket_scratch EXIT
printf 'ic-testkit qualification state retained: %s\n' "$IC_AUTH_QUALIFICATION_STATE_ROOT"
cargo test --manifest-path qualification/Cargo.toml --locked --offline -p ic-auth-qualification --test signatures -- --nocapture "$@"
