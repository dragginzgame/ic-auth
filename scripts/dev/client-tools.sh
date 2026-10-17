#!/usr/bin/env bash
# Consumer-owned Node selection and explicit dependency preparation.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../../packages/client"
# shellcheck source=scripts/dev/client-toolchain.sh
source "${script_path%/*}/client-toolchain.sh"
client_toolchain_select "$PWD"
observed_node="$(node --version)"
observed_npm="$(npm --version)"
if ! client_node_matches "$observed_node" || ! client_npm_matches "$observed_npm"; then
    printf 'Observed Node=%s path=%s; npm=%s path=%s\n' \
        "$observed_node" "$(command -v node)" "$observed_npm" "$(command -v npm)" >&2
    echo "Select Node $selected_node and npm $selected_npm before client setup/checks" >&2; exit 1;
fi
printf 'Client toolchain: Node %s; npm %s\n' "$selected_node" "$selected_npm"
case "${1:-}" in
    check) [[ -x node_modules/.bin/tsc ]] || {
        echo 'Run make install-client-dependencies explicitly first' >&2; exit 1;
    } ;;
    install) npm ci --ignore-scripts --no-audit --no-fund ;;
    *) echo 'usage: client-tools.sh check|install' >&2; exit 2 ;;
esac
