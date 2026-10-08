#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export PATH="$PWD/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
[[ $# == 1 ]] || { echo 'usage: msrv-tools.sh floor|install|check' >&2; exit 2; }
floor="$(yq -p toml -o json '.' Cargo.toml | jq -er '.workspace.package["rust-version"]')"
[[ "$floor" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'invalid workspace MSRV' >&2; exit 1; }
case "$1" in
    floor) printf '%s\n' "$floor" ;;
    install) rustup toolchain install "$floor" --profile minimal --target wasm32-unknown-unknown --no-self-update ;;
    check)
        rustc +"$floor" --version
        cargo +"$floor" --version
        installed="$(rustup target list --installed --toolchain "$floor")"
        [[ $'\n'"$installed"$'\n' == *$'\nwasm32-unknown-unknown\n'* ]] || {
            echo 'MSRV Wasm target missing; run make install-msrv' >&2; exit 1;
        }
        ;;
    *) echo 'usage: msrv-tools.sh floor|install|check' >&2; exit 2 ;;
esac
