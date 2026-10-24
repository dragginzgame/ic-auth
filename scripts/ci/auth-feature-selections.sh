#!/usr/bin/env bash
# Every declared Auth capability, plus the supported session/portable pairing.
set -euo pipefail
manifest="${1:-crates/ic-auth/Cargo.toml}"
features="$(yq -p toml -o json '.' "$manifest" | jq -er '.features | keys[] | select(. != "default")')"
printf 'default\n%s\nsessions,portable-signature-verification\nall\n' "$features"
