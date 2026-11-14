#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
mkdir -p target/portable-fixtures
evidence="$(mktemp -d "$root/target/portable-fixtures/public-docs.XXXXXX")"
printf 'Public documentation evidence retained: %s\n' "$evidence"
# Product and private fixture APIs have no missing-docs exception. Keep actual
# Rustdoc warning refusal, including links and macro-emitted public items.
export RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -D warnings"
cargo doc --locked --offline --workspace --all-features --no-deps 2>&1 | tee "$evidence/product.log"
(
    cd qualification
    cargo doc --locked --offline -p ic-auth-qualification -p ic-auth-qualification-contracts -p ic-auth-qualification-fixtures -p ic-auth-backend-conformance -p ic-auth-feature-unification --all-features --no-deps 2>&1 | tee "$evidence/qualification.log"
    cargo doc --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown --no-deps 2>&1 | tee "$evidence/canister.log"
)
