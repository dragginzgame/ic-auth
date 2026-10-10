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
selection=(--consumer "$PWD" --package ic-testkit --lockfile Cargo.lock
    --bin ic-testkit-server --profile debug)
[[ "$phase" != check ]] || selection+=(--check)
cli="$(bash scripts/dev/install-rust-tools.sh "${selection[@]}")" || {
    status=$?
    printf 'Testkit CLI ic-testkit-server selected by Cargo.lock %s failed (exit %s).\n' "$phase" "$status" >&2
    if [[ "$phase" == check ]]; then
        echo 'Prepare the selected CLI with make install-testkit-tools; checks never install tools.' >&2
    fi
    exit "$status"
}
case "$phase" in
    install) server="$("$cli" setup --directory "$PWD/.tools/testkit-server")" ;;
    check) server="$("$cli" check --directory "$PWD/.tools/testkit-server")" ;;
esac
# The server operation is a separate effect boundary. Re-admit through the
# canonical owner offline, then compare identities before reporting its path.
[[ "$phase" != install ]] || selection+=(--check)
observed_cli="$(bash scripts/dev/install-rust-tools.sh "${selection[@]}")" || {
    status=$?
    echo 'locked Testkit CLI could not be re-admitted after server setup/check; rerun explicit setup/check' >&2
    exit "$status"
}
[[ "$observed_cli" == "$cli" ]] || {
    echo 'locked Testkit selection changed; rerun explicit setup/check' >&2
    exit 1
}
printf '%s\n' "$server"
