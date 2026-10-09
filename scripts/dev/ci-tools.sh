#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
[[ $# == 1 ]] || { echo 'usage: ci-tools.sh install|check' >&2; exit 2; }
case "$1" in
    install) target=install-tools ;;
    check) target=tools-check ;;
    *) echo 'usage: ci-tools.sh install|check' >&2; exit 2 ;;
esac
mkdir -p target
# Use the real consumer aggregate and retain its original failure status/output.
if make --no-print-directory "$target" 2>&1 | tee "target/rust-tools-$1.log"; then
    exit 0
else
    statuses=("${PIPESTATUS[@]}")
    # A failing logger must not replace the failed aggregate's status. If the
    # aggregate succeeded, still refuse to report success without its log.
    [[ "${statuses[0]}" == 0 ]] || exit "${statuses[0]}"
    exit "${statuses[1]}"
fi
