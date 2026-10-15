#!/usr/bin/env bash
# Sourced selection/admission helpers; installation and PATH effects stay callers'.
client_toolchain_select() {
    selected_node="$(cat "$1/.nvmrc")" || return
    selected_npm="$(node -p 'require(process.argv[1]).packageManager.slice(4)' "$1/package.json")" || return
}
client_node_matches() {
    [[ "$1" == "v$selected_node" ]]
}
client_npm_matches() {
    [[ "$1" == "$selected_npm" ]]
}
