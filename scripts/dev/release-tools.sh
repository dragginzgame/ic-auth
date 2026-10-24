#!/usr/bin/env bash
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export PATH="$PWD/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
[[ $# -ge 1 && $# -le 3 ]] || {
    echo 'usage: release-tools.sh install|check|check-wait [LOCKFILE [SOURCE_MANIFEST]]' >&2; exit 2;
}
selection=(--consumer "$PWD" --package ic-build --lockfile "${2:-Cargo.lock}"
    --bin ic-build-release-metadata --profile debug)
# Older validated sources predate CLI feature gating. Read their declared
# selection, never infer it from the version or a partially prepared live graph.
features="$(yq -p toml -o json '.' "${3:-Cargo.toml}" | jq -er '
    (.workspace.metadata."ic-auth" // {})
    | if has("release-editor-features") then ."release-editor-features" else [] end
    | if type == "array" and all(.[]; type == "string")
      then join(",") else error("invalid release editor features") end')"
[[ -z "$features" ]] || selection+=(--features "$features")
case "$1" in
    install) ;;
    check) selection+=(--check) ;;
    check-wait) selection+=(--check --wait-seconds 60) ;;
    *) echo 'usage: release-tools.sh install|check|check-wait [LOCKFILE [SOURCE_MANIFEST]]' >&2; exit 2 ;;
esac
exec bash scripts/dev/install-rust-tools.sh "${selection[@]}"
