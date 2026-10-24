#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
selections="$(bash scripts/ci/auth-feature-selections.sh)"
cargo check --locked --offline -p ic-auth-protocol-types -p ic-auth-wire --target wasm32-unknown-unknown
while IFS= read -r feature; do
    selected=(--no-default-features)
    case "$feature" in
        default) ;;
        all) selected=(--all-features) ;;
        *) selected+=(--features "$feature") ;;
    esac
    cargo check --locked --offline -p ic-auth --target wasm32-unknown-unknown "${selected[@]}"
done <<< "$selections"
