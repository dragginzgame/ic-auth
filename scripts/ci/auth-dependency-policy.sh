#!/usr/bin/env bash
# Auth-owned graph rules; Shared supplies the boundary-script observation/matching.
auth_dependency_pattern() {
    # Principal-only Platform is production; its optional capabilities are
    # constrained by the observed Candid/runtime/host dependencies below.
    local product='candid_parser|canic|toko|solana|ic-siws|ic-host|ic-build|ic-testkit|ic-agent|tokio|pocket-ic|ic-auth-qualification|ic-auth-feature-unification|ic-auth-tooling'
    local storage='ic-cdk|ic-stable-structures|ic-memory|ic-timers'
    local certificates='ic-certification|ic-signature-verification|ic-canister-sig-creation'
    local pattern
    case "$1" in
        default) pattern="$product|candid|candid_derive" ;;
        passive) pattern="$storage|ic0|$certificates|ic-verify-bls-signature" ;;
        pure) pattern="candid|candid_derive|curve25519-dalek|ed25519-dalek|k256|$certificates|ic-verify-bls-signature|ic-cdk|ic-stable-structures" ;;
        capability|deployment) pattern="$product|$storage" ;;
        certificate-free) pattern="$certificates|ic-verify-bls-signature|candid|candid_derive" ;;
        session) pattern="candid|candid_derive|k256|$certificates|ic-verify-bls-signature" ;;
        packaged) pattern="$product|$storage" ;;
        *) echo 'unknown Auth dependency policy' >&2; return 1 ;;
    esac
    printf '^(%s)([- ]|$)\n' "$pattern"
}
