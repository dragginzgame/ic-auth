#!/usr/bin/env bash
# Auth-owned graph rules; callers retain independent graph collection/admission.
auth_dependency_pattern() {
    local product='canic|toko|solana|ic-siws|ic-host|ic-build'
    local storage='ic-cdk|ic-stable-structures|ic-memory|ic-timers'
    local certificates='ic-certification|ic-signature-verification|ic-canister-sig-creation'
    local pattern
    case "$1" in
        default) pattern="$product" ;;
        passive) pattern="$storage|ic0|ic-certification" ;;
        pure) pattern="curve25519-dalek|ed25519-dalek|k256|$certificates|ic-verify-bls-signature|ic-cdk|ic-stable-structures" ;;
        capability) pattern="$product|$storage" ;;
        certificate-free) pattern="$certificates" ;;
        session) pattern="k256|$certificates|ic-verify-bls-signature" ;;
        packaged) pattern="$product|ic-testkit|pocket-ic|$storage" ;;
        *) echo 'unknown Auth dependency policy' >&2; return 1 ;;
    esac
    printf '^(%s)([- ]|$)\n' "$pattern"
}
