#!/usr/bin/env bash
set -euo pipefail
# Prepare the same substituted Cargo effect boundary for portable and native
# retention qualification. This creates no installation and contacts no registry.
[[ $# == 1 && -n "$1" && ! -e "$1" && ! -L "$1" ]] || exit 2
fixture="$1"
case "$fixture" in *$'\r'*|*$'\n'*) echo 'fixture directory contains CR/LF' >&2; exit 2 ;; esac
[[ "$fixture" == /* ]] || fixture="$PWD/$fixture"
mkdir -p "$fixture/bin"
fixture="$(CDPATH='' cd -P -- "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"
case "$fixture" in
    *$'\r'*|*$'\n'*) printf 'Invalid physical fixture directory retained: %q\n' "$fixture" >&2; exit 2 ;;
esac
real_rustc="$(command -v rustc)"
printf '#!%s\n' "$BASH" > "$fixture/bin/rustc"
printf 'fixture=%q\nreal_rustc=%q\n' "$fixture" "$real_rustc" >> "$fixture/bin/rustc"
cat >> "$fixture/bin/rustc" <<'STUB'
if [[ "$*" == '--print sysroot' ]]; then echo "$fixture"; else exec "$real_rustc" "$@"; fi
STUB
printf '#!%s\nset -euo pipefail\n' "$BASH" > "$fixture/bin/cargo"
cat >> "$fixture/bin/cargo" <<'STUB'
case "$1" in
    --version|-vV) echo 'cargo retention fixture'; exit 0 ;;
    info)
        package="${2%@*}"; version="${2#*@}"
        mkdir -p "$CARGO_HOME/registry/cache/index.crates.io-fixture" "$CARGO_HOME/fixture/$package-$version"
        printf 'version=3\n[[package]]\nname="%s"\nversion="%s"\n' "$package" "$version" > "$CARGO_HOME/fixture/$package-$version/Cargo.lock"
        tar -czf "$CARGO_HOME/registry/cache/index.crates.io-fixture/$package-$version.crate" -C "$CARGO_HOME/fixture" "$package-$version"
        exit 0 ;;
    install) ;;
    *) exit 1 ;;
esac
build=''
while [[ $# -gt 0 ]]; do
    if [[ "$1" == --target-dir ]]; then build="$2"; break; fi
    shift
done
[[ -n "$build" && "$build" == "${HOME%/home}/build" ]] || exit 1
mkdir -p "$build"
printf 'retained Rust build output\n' > "$build/failed-build.txt"
echo 'injected Cargo installation failure' >&2
exit 23
STUB
chmod +x "$fixture/bin/rustc" "$fixture/bin/cargo"
