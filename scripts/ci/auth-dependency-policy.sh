#!/usr/bin/env bash
# Auth-owned graph rules; callers retain independent graph collection/admission.
auth_dependency_pattern() {
    local product='canic|toko|solana|ic-siws|ic-host|ic-auth-qualification|ic-auth-tooling'
    local storage='ic-cdk|ic-stable-structures|ic-memory|ic-timers'
    local certificates='ic-certification|ic-signature-verification|ic-canister-sig-creation'
    local pattern
    case "$1" in
        default) pattern="$product|candid|candid_derive" ;;
        passive) pattern="$storage|ic0|ic-certification" ;;
        pure) pattern="candid|candid_derive|curve25519-dalek|ed25519-dalek|k256|$certificates|ic-verify-bls-signature|ic-cdk|ic-stable-structures" ;;
        capability) pattern="$product|$storage" ;;
        certificate-free) pattern="$certificates|candid|candid_derive" ;;
        session) pattern="candid|candid_derive|k256|$certificates|ic-verify-bls-signature" ;;
        packaged) pattern="$product|ic-testkit|pocket-ic|$storage" ;;
        *) echo 'unknown Auth dependency policy' >&2; return 1 ;;
    esac
    printf '^(%s)([- ]|$)\n' "$pattern"
}
