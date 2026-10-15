#!/usr/bin/env bash
# Explicit CI preparation in the Node installation selected by setup-node.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
[[ -n "${GITHUB_PATH:-}" ]] || { echo 'CI preparation requires GITHUB_PATH' >&2; exit 2; }
source scripts/dev/client-toolchain.sh
client_toolchain_select "$PWD/packages/client"
observed_node="$(node --version)"
node_bin="$(dirname "$(node -p 'process.execPath')")"
node_prefix="$(dirname "$node_bin")"
bootstrap_npm="$(command -v npm)"
printf 'Client setup: selected Node=%s npm=%s; observed Node=%s path=%s\n' \
    "$selected_node" "$selected_npm" "$observed_node" "$(command -v node)"
printf 'Client setup: bootstrap npm=%s path=%s prefix=%s; install prefix=%s\n' \
    "$("$bootstrap_npm" --version)" "$bootstrap_npm" \
    "$("$bootstrap_npm" prefix --global)" "$node_prefix"
client_node_matches "$observed_node" || {
    echo 'Selected Node is not active; client setup refused' >&2; exit 1;
}
export PATH="$node_bin:$PATH"
"$bootstrap_npm" install --global --prefix "$node_prefix" "npm@$selected_npm" \
    --registry=https://registry.npmjs.org --ignore-scripts --no-audit --no-fund
hash -r
observed_npm="$(npm --version)"
printf 'Client setup: prepared npm=%s path=%s prefix=%s\n' \
    "$observed_npm" "$(command -v npm)" "$(npm prefix --global)"
client_npm_matches "$observed_npm" || {
    echo 'Selected npm is not active; client setup refused' >&2; exit 1;
}
make install-client-dependencies
make client-tools-check
# Publish the admitted path only after complete preparation succeeds.
printf '%s\n' "$node_bin" >> "$GITHUB_PATH"
