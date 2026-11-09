#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
    canonical) label=Canonical ;;
    retry) label=Retry ;;
    *) echo 'usage: run-allocation-profile.sh canonical|retry [benchmark arguments]' >&2; exit 2 ;;
esac
profile="$1"
shift
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
export PATH="$PWD/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR="$PWD/target"
mkdir -p target/portable-fixtures
evidence="$(mktemp -d "$PWD/target/portable-fixtures/$profile-allocations.XXXXXX")"
printf '%s allocation evidence retained: %s\n' "$label" "$evidence"
tool="$(bash scripts/dev/host-file-tools.sh check)"
{
    rg --files crates/ic-auth/src crates/ic-auth-protocol-types/src -0
    printf '%s\0' Cargo.toml Cargo.lock rust-toolchain.toml crates/ic-auth-tooling/Cargo.toml \
        "crates/ic-auth-tooling/benches/${profile}_allocations.rs" crates/ic-auth-tooling/benches/allocation_profile/mod.rs \
        "scripts/dev/profile-$profile-allocations.sh" "${BASH_SOURCE[0]}"
} > "$evidence/inputs.nul"
"$tool" hash-files0 8388608 4194304 4096 < "$evidence/inputs.nul" > "$evidence/before.hashes"
git rev-parse HEAD > "$evidence/source.txt"
git status --porcelain > "$evidence/source-status.txt"
rustc --version > "$evidence/compiler.txt"
(
    # DHAT failure profiles and all build output remain in this repository.
    cd "$evidence"
    cargo bench --manifest-path "$OLDPWD/Cargo.toml" --locked --offline \
        -p ic-auth-tooling --bench "${profile}_allocations" -- "$@"
) > "$evidence/report.json" 2> "$evidence/command.log"
"$tool" hash-files0 8388608 4194304 4096 < "$evidence/inputs.nul" > "$evidence/after.hashes"
cmp "$evidence/before.hashes" "$evidence/after.hashes"
cat "$evidence/report.json"
