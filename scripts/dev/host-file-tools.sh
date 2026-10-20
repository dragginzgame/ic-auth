#!/usr/bin/env bash
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export PATH="$PWD/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
# Keep auxiliary tool identity stable inside cargo +MSRV test processes too.
RUSTUP_TOOLCHAIN="$(yq -p toml -o json '.' rust-toolchain.toml | jq -er '.toolchain.channel')"
export RUSTUP_TOOLCHAIN
[[ $# == 1 && ( "$1" == install || "$1" == check ) ]] || {
    echo 'usage: host-file-tools.sh install|check' >&2; exit 2;
}
# shellcheck source=ci/host-file.env
source ci/host-file.env
selection=(--consumer "$PWD" --package ic-host-fs --version "$HOST_FILE_VERSION"
    --bin ic-host-file --profile debug --features cli)
[[ "$1" != check ]] || selection+=(--check)
exec bash scripts/dev/install-rust-tools.sh "${selection[@]}"
