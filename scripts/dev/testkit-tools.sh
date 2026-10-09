#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export PATH="$PWD/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
[[ $# == 1 ]] || { echo 'usage: testkit-tools.sh install|check' >&2; exit 2; }
case "$1" in
    install|check) phase="$1" ;;
    *) echo 'usage: testkit-tools.sh install|check' >&2; exit 2 ;;
esac

# The consumer lock selects the CLI package, not a second server catalog.
# Shared Tooling owns executable installation/receipts; Testkit owns server
# assets, compatibility and offline admission. No cache glob or fallback route.
locked_testkit_version() {
    yq -p toml -o json '.' Cargo.lock | jq -er '
    [.package[] | select(.name == "ic-testkit")]
    | if length == 1 and .[0].source == "registry+https://github.com/rust-lang/crates.io-index"
      then .[0].version else error("expected one registry Testkit selection") end
'
}
version="$(locked_testkit_version)"
selection=(--consumer "$PWD" --package ic-testkit --version "$version"
    --bin ic-testkit-server --profile debug)
[[ "$phase" != check ]] || selection+=(--check)
cli="$(bash scripts/dev/install-rust-tools.sh "${selection[@]}")"
case "$phase" in
    install) server="$("$cli" setup --directory "$PWD/.tools/testkit-server")" ;;
    check) server="$("$cli" check --directory "$PWD/.tools/testkit-server")" ;;
esac
[[ "$(locked_testkit_version)" == "$version" ]] || {
    echo 'locked Testkit selection changed; rerun explicit setup/check' >&2
    exit 1
}
printf '%s\n' "$server"
