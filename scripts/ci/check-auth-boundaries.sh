#!/usr/bin/env bash
set -euo pipefail

# Inspect the actual selected transitive graph, not only direct declarations.
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source scripts/ci/auth-dependency-policy.sh
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
for package in ic-auth-protocol-types ic-auth; do
    graph="$(cargo tree --locked --offline -p "$package" --no-default-features --edges normal,build --prefix none --format '{p}')"
    if printf '%s\n' "$graph" | rg "$(auth_dependency_pattern default)"; then
        echo "$package pulls product, wallet or native host dependencies into the default graph" >&2
        exit 1
    fi
    if [[ "$package" == ic-auth-protocol-types ]] && printf '%s\n' "$graph" | rg "$(auth_dependency_pattern passive)"; then
        echo 'passive types pull runtime, storage or certification state' >&2
        exit 1
    fi
    if [[ "$package" == ic-auth ]] && printf '%s\n' "$graph" | rg "$(auth_dependency_pattern pure)"; then
        echo 'pure encoding/request graph pulls cryptographic or session storage dependencies' >&2
        exit 1
    fi
done
# The selected authentication capabilities must remain free of product, wallet,
# host tooling and service-owned storage dependencies too. Upstream IC key
# parsing includes ic0 transitively; verification performs no runtime calls.
for feature in portable-signature-verification chain-key-verification canister-signature-verification canister-signature-preparation token-verification sessions session-restore sessions,portable-signature-verification; do
    graph="$(cargo tree --locked --offline -p ic-auth --no-default-features --features "$feature" --edges normal,build --prefix none --format '{p}')"
    if printf '%s\n' "$graph" | rg "$(auth_dependency_pattern capability)"; then
        echo "$feature pulls product, wallet, native host or runtime/storage dependencies" >&2
        exit 1
    fi
    if [[ "$feature" == chain-key-verification || "$feature" == portable-signature-verification ]] && printf '%s\n' "$graph" | rg "$(auth_dependency_pattern certificate-free)"; then
        echo "$feature pulls certificate machinery" >&2
        exit 1
    fi
    if [[ "$feature" == sessions* || "$feature" == session-restore ]] && printf '%s\n' "$graph" | rg "$(auth_dependency_pattern session)"; then
        echo "$feature pulls certificate or chain-key machinery" >&2
        exit 1
    fi
done
echo 'Auth dependency boundaries verified'
