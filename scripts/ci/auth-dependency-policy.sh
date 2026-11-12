#!/usr/bin/env bash
# Auth-owned graph rules; Shared supplies the boundary-script observation/matching.
auth_dependency_pattern() {
    # Types owns passive identities; Platform is selected only for crypto.
    local profiling='dhat'
    local product="candid_parser|canic|toko|solana|ic-siws|ic-host|ic-build|ic-testkit|ic-entropy|ic-metrics|ic-agent|tokio|pocket-ic|ic-platform-pocket-ic|ic-auth-qualification|ic-auth-feature-unification|ic-auth-tooling|$profiling"
    local storage='ic-platform-runtime|ic-platform-macros|ic-cdk|ic-stable-structures|ic-memory|ic-timers'
    local allocation='ic-platform-runtime-guard|ic-platform-macros|ic-cdk|ic-stable-structures|ic-memory|ic-timers'
    local certificates='ic-certification|ic-signature-verification|ic-canister-sig-creation'
    local pattern
    case "$1" in
        profiling) pattern="$profiling" ;;
        backend-conformance) pattern="$product|$storage|ic-storage|$certificates|candid|candid_derive|ic0|k256|ic-verify-bls-signature" ;;
        default) pattern="$product|ic-storage|candid|candid_derive" ;;
        passive) pattern="ic_principal|sha2|ic-storage|$storage|ic0|$certificates|ic-verify-bls-signature" ;;
        pure) pattern="ic-storage|candid|candid_derive|curve25519-dalek|ed25519-dalek|k256|$certificates|ic-verify-bls-signature|ic-cdk|ic-stable-structures" ;;
        capability|deployment|packaged) pattern="$product|$storage|ic-storage|candid|candid_derive|ic0|ic-canister-sig-creation|ic-signature-verification" ;;
        persistent) pattern="$product|$allocation|candid|candid_derive|ic-canister-sig-creation|ic-signature-verification" ;;
        # Approved Platform 0.22 exception: upstream certificate dependencies,
        # with native tooling and runtime/executor ownership still forbidden.
        certified) pattern="$product|$storage|ic-storage" ;;
        persistent-certified|packaged-certified) pattern="$product|$allocation" ;;
        certificate-free) pattern="$certificates|ic-verify-bls-signature|candid|candid_derive" ;;
        session) pattern="serde_cbor|candid|candid_derive|k256|$certificates|ic-verify-bls-signature" ;;
        *) echo 'unknown Auth dependency policy' >&2; return 1 ;;
    esac
    # The facade itself is forbidden here; its Types owner remains permitted.
    case "$1" in passive|pure) printf '^ic-platform( |$)|' ;; esac
    printf '^(%s)([-_ ]|$)\n' "$pattern"
}
