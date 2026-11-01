#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source scripts/ci/auth-dependency-policy.sh
native_target="$(rustc -vV | sed -n 's/^host: //p')"
[[ -n "$native_target" ]] || { echo 'missing compiler host target' >&2; exit 1; }
perl scripts/ci/check-cargo-graph.pl --consumer "$PWD" \
    --package ic-auth-backend-conformance --target "$native_target" \
    --forbid-pattern "$(auth_dependency_pattern backend-conformance)"
selections="$(bash scripts/ci/auth-feature-selections.sh)"
checker=(perl scripts/ci/check-cargo-graph.pl --consumer "$PWD")
for target in "$native_target" wasm32-unknown-unknown; do
    for package in ic-auth-protocol-types ic-auth ic-auth-wire; do
        rules=(--forbid-pattern "$(auth_dependency_pattern deployment)"
               --forbid-pattern "$(auth_dependency_pattern default)"
               --forbid-feature ic-platform/ic0 --forbid-feature ic-platform/executor
               --forbid-feature ic-platform/runtime --forbid-feature ic-platform/cdk)
        if [[ "$package" == ic-auth-protocol-types || "$package" == ic-auth-wire ]]; then
            rules+=(--forbid-pattern "$(auth_dependency_pattern passive)")
        fi
        if [[ "$package" == ic-auth ]]; then
            rules+=(--forbid-pattern "$(auth_dependency_pattern pure)")
        fi
        "${checker[@]}" --package "$package" --target "$target" --no-default-features "${rules[@]}"
    done
    # Platform owns imported signature code; library graphs need no Candid/IC0.
    while IFS= read -r feature; do
        [[ "$feature" != default ]] || continue
        selected=(--no-default-features --features "$feature")
        [[ "$feature" != all ]] || selected=(--all-features)
        rules=(--forbid-pattern "$(auth_dependency_pattern capability)"
               --forbid-feature ic-platform/ic0 --forbid-feature ic-platform/executor
               --forbid-feature ic-platform/runtime --forbid-feature ic-platform/cdk)
        if [[ "$feature" == chain-key-verification || "$feature" == portable-signature-verification ]]; then
            rules+=(--forbid-pattern "$(auth_dependency_pattern certificate-free)")
        fi
        if [[ "$feature" == sessions* || "$feature" == session-restore ]]; then
            rules+=(--forbid-pattern "$(auth_dependency_pattern session)")
        fi
        "${checker[@]}" --package ic-auth --target "$target" "${selected[@]}" "${rules[@]}"
    done <<< "$selections"
    # Private fixture schemas must stay Wire-only even though the actual host's
    # runtime selects an executor, but neither path requires Candid.
    "${checker[@]}" --package ic-auth-qualification-contracts --target "$target" \
        --forbid-pattern "$(auth_dependency_pattern passive)" \
        --forbid-pattern '^(candid|candid_derive|candid_parser|ic-cdk|ic0|ic-memory|ic-storage|ic-agent|ic-host|ic-build|ic-testkit|pocket-ic|tokio)([- ]|$)' \
        --forbid-feature ic-platform/candid \
        --forbid-feature ic-platform/ic0 --forbid-feature ic-platform/executor \
        --forbid-feature ic-platform/runtime --forbid-feature ic-platform/cdk
done
# Curve/signature providers retain one identity. Platform 0.16's RustCrypto
# generation and maintained BLS/Principal hashing require two support generations.
crypto_singletons=(
    base16ct const-oid crypto-bigint curve25519-dalek
    der ecdsa ed25519 ed25519-dalek elliptic-curve hmac
    ic-certification ic-representation-independent-hash ic-verify-bls-signature ic_bls12_381 k256 pairing
    pkcs8 primeorder rfc6979 sec1 signature spki subtle zeroize
)
rules=()
for package in "${crypto_singletons[@]}"; do rules+=(--max-count "$package=1"); done
for package in block-buffer crypto-common digest ff group rand_core sha2; do
    rules+=(--max-count "$package=2")
done
"${checker[@]}" --package ic-auth --target wasm32-unknown-unknown --all-features "${rules[@]}"
"${checker[@]}" --package ic-auth-qualification-canister --target wasm32-unknown-unknown \
    --forbid-pattern "$(auth_dependency_pattern profiling)" \
    --forbid-pattern '^(candid|candid_derive|candid_parser|ic-canister-sig-creation|ic-signature-verification|ic-host|ic-build|ic-agent|ic-testkit|pocket-ic|tokio)([- ]|$)' \
    --forbid ic-cdk --forbid ic-cdk-macros --forbid ic-cdk-executor --forbid ic0 \
    --forbid-feature ic-platform/candid \
    --require ic-platform-macros=1 --require ic-platform-runtime-guard=1
echo 'Auth dependency boundaries verified'
