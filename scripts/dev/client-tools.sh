#!/usr/bin/env bash
# Consumer-owned Node selection and explicit dependency preparation.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../../packages/client"
selected_node="$(cat .nvmrc)"
selected_npm="$(node --input-type=module -e 'import manifest from "./package.json" with {type:"json"}; console.log(manifest.packageManager.slice(4))')"
[[ "$(node --version)" == "v$selected_node" && "$(npm --version)" == "$selected_npm" ]] || {
    echo "Select Node $selected_node and npm $selected_npm before client setup/checks" >&2; exit 1;
}
printf 'Client toolchain: Node %s; npm %s\n' "$selected_node" "$selected_npm"
case "${1:-}" in
    check) [[ -x node_modules/.bin/tsc && -x node_modules/.bin/icp-bindgen ]] || {
        echo 'Run make install-client-dependencies explicitly first' >&2; exit 1;
    } ;;
    install) npm ci --ignore-scripts --no-audit --no-fund ;;
    *) echo 'usage: client-tools.sh check|install' >&2; exit 2 ;;
esac
