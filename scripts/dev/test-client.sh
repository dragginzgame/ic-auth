#!/usr/bin/env bash
# Offline source tests and both directions of the canonical IC Wire boundary.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export CARGO_TARGET_DIR="$PWD/target" RUSTUP_AUTO_INSTALL=0
bash scripts/dev/package-client.sh check
npm --prefix packages/client test
cargo run --quiet --locked --offline -p ic-auth-tooling -- client-contracts check target/browser-client
