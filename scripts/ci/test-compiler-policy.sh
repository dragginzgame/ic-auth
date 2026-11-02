#!/usr/bin/env bash
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH"
floor="$(bash scripts/dev/msrv-tools.sh floor)"
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/compiler-policy.XXXXXX")"
printf 'Compiler policy evidence retained: %s\n' "$fixture"
mkdir -p "$fixture/scripts/dev" "$fixture/bin"
cp scripts/dev/msrv-tools.sh "$fixture/scripts/dev/"
printf '[workspace.package]\nrust-version = "%s"\n' "$floor" > "$fixture/Cargo.toml"
printf '[toolchain]\nchannel = "%s"\n' "$floor" > "$fixture/rust-toolchain.toml"
cat > "$fixture/bin/rustc" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
name="${0##*/}"
version="$COMPILER_FIXTURE_FLOOR"
[[ "$name" != rustc ]] || version="${COMPILER_FIXTURE_RUSTC:-$version}"
[[ "$name" != cargo ]] || version="${COMPILER_FIXTURE_CARGO:-$version}"
[[ "$1" != --version ]] || version="${COMPILER_FIXTURE_ACTIVE:-$version}"
printf '%s %s (fixture 2026-11-02)\n' "$name" "$version"
STUB
cp "$fixture/bin/rustc" "$fixture/bin/cargo"
cat > "$fixture/bin/rustup" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == toolchain ]]; then
    printf '%s\n' "$@" > "$COMPILER_FIXTURE_INSTALL_LOG"
else
    [[ "$*" == "target list --installed --toolchain $COMPILER_FIXTURE_FLOOR" ]] || exit 1
    [[ "${COMPILER_FIXTURE_WASM:-yes}" != yes ]] || echo wasm32-unknown-unknown
fi
STUB
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$PATH" COMPILER_FIXTURE_FLOOR="$floor" COMPILER_FIXTURE_INSTALL_LOG="$fixture/install-args"
run() { bash "$fixture/scripts/dev/msrv-tools.sh" "$@"; }
refuse() {
    local name="$1"; shift
    if "$@" > "$fixture/$name.log" 2>&1; then
        echo "compiler policy unexpectedly admitted $name" >&2; exit 1
    fi
    [[ ! -e "$COMPILER_FIXTURE_INSTALL_LOG" ]] || exit 1
}
run check > "$fixture/admitted.log"
refuse wrong-rustc env COMPILER_FIXTURE_RUSTC=0.0.0 bash "$fixture/scripts/dev/msrv-tools.sh" check
refuse wrong-cargo env COMPILER_FIXTURE_CARGO=0.0.0 bash "$fixture/scripts/dev/msrv-tools.sh" check
refuse active-override env COMPILER_FIXTURE_ACTIVE=0.0.0 bash "$fixture/scripts/dev/msrv-tools.sh" check
refuse missing-wasm env COMPILER_FIXTURE_WASM=no bash "$fixture/scripts/dev/msrv-tools.sh" check
printf '[toolchain]\nchannel = "stable"\n' > "$fixture/rust-toolchain.toml"
refuse moving-pin run floor
refuse mismatched-install run install
printf '[toolchain]\nchannel = "%s"\n' "$floor" > "$fixture/rust-toolchain.toml"
run install
printf '%s\n' toolchain install "$floor" --profile minimal --component rustfmt --component clippy \
    --target wasm32-unknown-unknown --no-self-update > "$fixture/expected-install-args"
cmp "$fixture/expected-install-args" "$COMPILER_FIXTURE_INSTALL_LOG"
echo 'Compiler version, active selection, Wasm admission and explicit preparation passed (substituted compiler effects)'
