#!/usr/bin/env bash
# Build local archives only. npm publication is a separately authorized operation.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export CARGO_TARGET_DIR="$PWD/target" RUSTUP_AUTO_INSTALL=0
case "${1:-}" in pack|check|check-browser) ;; *) echo 'usage: package-client.sh pack|check|check-browser' >&2; exit 2 ;; esac
if [[ "$1" == check-browser ]]; then
    : "${IC_AUTH_BROWSER_EXECUTABLE:?Select an explicitly prepared Chromium executable}"
    [[ "$IC_AUTH_BROWSER_EXECUTABLE" == /* && -x "$IC_AUTH_BROWSER_EXECUTABLE" ]] || {
        echo 'IC_AUTH_BROWSER_EXECUTABLE must be an absolute executable path' >&2; exit 1;
    }
    export IC_AUTH_BROWSER_EXECUTABLE
fi
bash scripts/dev/client-tools.sh check
directory="$(node scripts/dev/package-client.mjs prepare)"
printf 'Client package evidence retained: %s\n' "$directory"
# Keep Cargo below the recursive Make recipe, with its inherited jobserver.
# Node's subprocess wrappers intentionally do not receive arbitrary parent FDs.
bash scripts/dev/client-contracts.sh check > "$directory/contracts.log" 2>&1 || {
    status=$?
    cat "$directory/contracts.log" >&2
    exit "$status"
}
node scripts/dev/package-client.mjs "$1" "$directory"
