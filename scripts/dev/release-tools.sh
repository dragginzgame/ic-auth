#!/usr/bin/env bash
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export RUSTUP_AUTO_INSTALL=0
[[ $# == 1 || $# == 2 ]] || { echo 'usage: release-tools.sh install|check [LOCKFILE]' >&2; exit 2; }
selection=(--consumer "$PWD" --package ic-build --lockfile "${2:-Cargo.lock}"
    --bin ic-build-release-metadata --profile debug)
case "$1" in
    install) ;;
    check) selection+=(--check) ;;
    *) echo 'usage: release-tools.sh install|check [LOCKFILE]' >&2; exit 2 ;;
esac
exec bash scripts/dev/install-rust-tools.sh "${selection[@]}"
