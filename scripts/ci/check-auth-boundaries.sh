#!/usr/bin/env bash
set -euo pipefail

# Inspect the actual selected transitive graph, not only direct declarations.
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
for package in ic-auth-protocol-types ic-auth; do
    graph="$(cargo tree --locked --offline -p "$package" --no-default-features --edges normal,build --prefix none --format '{p}')"
    if printf '%s\n' "$graph" | rg '^(canic|toko|solana|ic-siws|ic-host)([- ]|$)'; then
        echo "$package pulls product, wallet or native host dependencies into the default graph" >&2
        exit 1
    fi
    if [[ "$package" == ic-auth-protocol-types ]] && printf '%s\n' "$graph" | rg '^(ic-cdk|ic0|ic-stable-structures|ic-memory|ic-timers|ic-certification)([- ]|$)'; then
        echo 'passive types pull runtime, storage or certification state' >&2
        exit 1
    fi
done
# The selected verification capability must remain free of product, wallet,
# host tooling and service-owned storage dependencies too. Upstream IC key
# parsing includes ic0 transitively; verification performs no runtime calls.
graph="$(cargo tree --locked --offline -p ic-auth --no-default-features --features canister-signature-verification --edges normal,build --prefix none --format '{p}')"
if printf '%s\n' "$graph" | rg '^(canic|toko|solana|ic-siws|ic-host|ic-cdk|ic-stable-structures|ic-memory|ic-timers)([- ]|$)'; then
    echo 'signature verification pulls product, wallet, native host or runtime/storage dependencies' >&2
    exit 1
fi
echo 'Auth dependency boundaries verified'
