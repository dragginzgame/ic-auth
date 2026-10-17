#!/usr/bin/env bash
set -euo pipefail

# Inspect the actual selected transitive graph, not only direct declarations.
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source scripts/ci/auth-dependency-policy.sh
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
for package in ic-auth-protocol-types ic-auth ic-auth-wire; do
    graph="$(cargo tree --locked --offline -p "$package" --no-default-features --edges normal,build --prefix none --format '{p}')"
    if printf '%s\n' "$graph" | rg "$(auth_dependency_pattern default)"; then
        echo "$package pulls Candid, product, wallet or native host dependencies into the default graph" >&2
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
for feature in portable-signature-verification chain-key-verification canister-signature-verification canister-signature-preparation certified-proof-verification sessions session-restore sessions,portable-signature-verification; do
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
# The complete deployed verifier must share its crypto support generation with
# the IC stack. Inspect normal Wasm edges, excluding native qualification tools.
graph="$(cargo tree --locked --offline -p ic-auth --all-features --target wasm32-unknown-unknown --edges normal --prefix none --format '{p}')"
duplicates="$(printf '%s\n' "$graph" | awk '
    $1 ~ /^(base16ct|block-buffer|const-oid|crypto-bigint|crypto-common|curve25519-dalek|der|digest|ecdsa|ed25519|ed25519-dalek|elliptic-curve|ff|group|hmac|ic-canister-sig-creation|ic-certification|ic-representation-independent-hash|ic-signature-verification|ic-verify-bls-signature|ic_bls12_381|k256|pairing|pkcs8|primeorder|rfc6979|sec1|sha2|signature|spki|subtle|zeroize)$/ {
        identity = $1 " " $2
        if (!seen[identity]++) count[$1]++
    }
    END { for (name in count) if (count[name] > 1) print name }
' | LC_ALL=C sort)"
if [[ -n "$duplicates" ]]; then
    printf 'Auth deployed Wasm resolves duplicate crypto packages:\n%s\n' "$duplicates" >&2
    exit 1
fi
echo 'Auth dependency boundaries verified'
