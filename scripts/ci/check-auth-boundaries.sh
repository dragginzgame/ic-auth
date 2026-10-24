#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source scripts/ci/auth-dependency-policy.sh
native_target="$(rustc -vV | sed -n 's/^host: //p')"
[[ -n "$native_target" ]] || { echo 'missing compiler host target' >&2; exit 1; }
selections="$(bash scripts/ci/auth-feature-selections.sh)"
checker=(perl scripts/ci/check-cargo-graph.pl --consumer "$PWD")
for target in "$native_target" wasm32-unknown-unknown; do
    for package in ic-auth-protocol-types ic-auth ic-auth-wire; do
        rules=(--forbid-pattern "$(auth_dependency_pattern deployment)"
               --forbid-pattern "$(auth_dependency_pattern default)")
        if [[ "$package" == ic-auth-protocol-types ]]; then
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
        rules=(--forbid-pattern "$(auth_dependency_pattern capability)")
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
        --forbid-pattern '^(candid|candid_derive|candid_parser|ic-cdk|ic0|ic-memory|ic-agent|ic-host|ic-build|ic-testkit|pocket-ic|tokio)([- ]|$)' \
        --forbid-feature ic-platform/candid
done
# Optional crypto packages may be absent; every observed package has at most one
# rendered version/source identity. The set and this policy remain Auth-owned.
crypto_singletons=(
    base16ct block-buffer const-oid crypto-bigint crypto-common curve25519-dalek
    der digest ecdsa ed25519 ed25519-dalek elliptic-curve ff group hmac
    ic-certification ic-representation-independent-hash ic-verify-bls-signature ic_bls12_381 k256 pairing
    pkcs8 primeorder rfc6979 sec1 sha2 signature spki subtle zeroize
)
rules=()
for package in "${crypto_singletons[@]}"; do rules+=(--max-count "$package=1"); done
"${checker[@]}" --package ic-auth --target wasm32-unknown-unknown --all-features "${rules[@]}"
"${checker[@]}" --package ic-auth-qualification-canister --target wasm32-unknown-unknown \
    --forbid-pattern '^(candid|candid_derive|candid_parser|ic-canister-sig-creation|ic-signature-verification|ic-host|ic-build|ic-agent|ic-testkit|pocket-ic|tokio)([- ]|$)' \
    --forbid ic-cdk --forbid ic-cdk-macros \
    --forbid-feature ic-platform/candid \
    --require ic-cdk-executor=1
echo 'Auth dependency boundaries verified'
