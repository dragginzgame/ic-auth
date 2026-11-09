#!/usr/bin/env bash
# Auth selects the shared owner for both workspace and release admission.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export PATH="$PWD/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
product=$(perl scripts/dev/tooling-lock.pl "$PWD")
qualification=$(perl scripts/dev/tooling-lock.pl "$PWD/qualification")
jq -en --argjson product "$product" --argjson qualification "$qualification" '
    $product.version == $qualification.version and
    $product.checksum == $qualification.checksum' > /dev/null || {
    echo 'Product and qualification must select the installed Toko Tooling release together.' >&2
    exit 1
}
# Includes private publication, path/catalog, reverse-edge and both-lock checks.
bash scripts/ci/cargo-release-group.sh check-graph engine > /dev/null
echo 'Product and independent qualification workspace admission passed.'
