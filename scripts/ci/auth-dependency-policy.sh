#!/usr/bin/env bash
# Auth-owned graph rules; Shared supplies the boundary-script observation/matching.
auth_dependency_pattern() {
    # Principal-only Platform is production; its optional capabilities are
    # constrained by the observed Candid/runtime/host dependencies below.
    local product='candid_parser|canic|toko|solana|ic-siws|ic-host|ic-build|ic-testkit|ic-metrics|ic-agent|tokio|pocket-ic|ic-auth-qualification|ic-auth-feature-unification|ic-auth-tooling'
    local storage='ic-platform-runtime-guard|ic-cdk|ic-stable-structures|ic-memory|ic-storage|ic-timers'
    local certificates='ic-certification|ic-signature-verification|ic-canister-sig-creation'
    local pattern
    case "$1" in
        default) pattern="$product|candid|candid_derive" ;;
        passive) pattern="ic_principal|sha2|$storage|ic0|$certificates|ic-verify-bls-signature" ;;
        pure) pattern="candid|candid_derive|curve25519-dalek|ed25519-dalek|k256|$certificates|ic-verify-bls-signature|ic-cdk|ic-stable-structures" ;;
        capability|deployment|packaged) pattern="$product|$storage|candid|candid_derive|ic0|ic-canister-sig-creation|ic-signature-verification" ;;
        certificate-free) pattern="$certificates|ic-verify-bls-signature|candid|candid_derive" ;;
        session) pattern="serde_cbor|candid|candid_derive|k256|$certificates|ic-verify-bls-signature" ;;
        *) echo 'unknown Auth dependency policy' >&2; return 1 ;;
    esac
    printf '^(%s)([- ]|$)\n' "$pattern"
}
