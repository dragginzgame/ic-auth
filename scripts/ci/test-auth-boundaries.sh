#!/usr/bin/env bash
set -euo pipefail
# Run the real graph admission against substituted Cargo observations. This proves
# target/feature dispatch and transitive exclusions without building a dependency.
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
root="$PWD"
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/auth-boundaries.XXXXXX")"
printf 'Dependency boundary evidence: %s\n' "$fixture"
mkdir -p "$fixture/repo/scripts/ci" "$fixture/bin"
cp scripts/ci/{check-auth-boundaries,auth-dependency-policy,auth-feature-selections}.sh "$fixture/repo/scripts/ci/"
mkdir -p "$fixture/repo/vendor/toko-tooling/scripts/ci"
cp vendor/toko-tooling/scripts/ci/check-cargo-graph.pl "$fixture/repo/vendor/toko-tooling/scripts/ci/"
mkdir -p "$fixture/repo/crates/ic-auth"
mkdir -p "$fixture/repo/qualification"
printf '[workspace]\n' > "$fixture/repo/qualification/Cargo.toml"
# A new capability must enter both target checks without another curated list.
awk '{ print } /^\[features\]$/ { print "future-capability = []" }' \
    crates/ic-auth/Cargo.toml > "$fixture/repo/crates/ic-auth/Cargo.toml"
printf '[workspace]\n' > "$fixture/repo/Cargo.toml"
selection_count="$(bash scripts/ci/auth-feature-selections.sh "$fixture/repo/crates/ic-auth/Cargo.toml" | wc -l | tr -d '[:space:]')"
cat > "$fixture/bin/rustc" <<'STUB'
#!/usr/bin/env bash
printf 'host: x86_64-unknown-linux-gnu\n'
STUB
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == *'--locked --offline'* && "$*" == *'--edges normal,build'* ]] || exit 1
printf '%s\n' "$*" >> "$BOUNDARY_EVENTS"
package=""
target=""
features=""
all_features=no
while [[ $# -gt 0 ]]; do
    case "$1" in
        --package) package="$2"; shift ;;
        --target) target="$2"; shift ;;
        --features) features="$2"; shift ;;
        --all-features) all_features=yes ;;
    esac
    shift
done
[[ -n "$package" ]]
if [[ "$package" == ic-auth-qualification-canister ]]; then
    printf 'ic-auth-qualification-canister v0.9.1\n'
    printf 'ic-wire v0.14.1\n'
    if [[ "$BOUNDARY_MODE" != host-storage-missing ]]; then printf 'ic-storage v0.65.0\n'; fi
    case "$BOUNDARY_MODE" in
        host-storage-duplicate) printf 'ic-storage v0.64.0\n' ;;
        host-retired-memory) printf 'ic-memory v0.63.0\n' ;;
        host-legacy-stable-structures) printf 'ic-stable-structures v0.7.2\n' ;;
        host-native-client) printf 'ic-platform-pocket-ic v0.18.0\n' ;;
    esac
    if [[ "$BOUNDARY_MODE" == host-wire-duplicate ]]; then printf 'ic-wire v0.13.5\n'; fi
    if [[ "$BOUNDARY_MODE" != host-runtime-missing ]]; then printf 'ic-platform-runtime v0.17.1\n'; fi
    if [[ "$BOUNDARY_MODE" == host-runtime-duplicate ]]; then printf 'ic-platform-runtime v0.18.0\n'; fi
    case "$BOUNDARY_MODE" in
        host-facade-ic0) printf 'ic-platform feature "ic0"\n' ;;
        host-facade-executor) printf 'ic-platform feature "executor"\n' ;;
        host-facade-runtime) printf 'ic-platform feature "runtime"\n' ;;
    esac
    if [[ "$BOUNDARY_MODE" != generator ]]; then
        printf 'ic-platform-macros v0.9.1\nic-platform-macros v0.9.1 (*)\n'
    fi
    if [[ "$BOUNDARY_MODE" != runtime-guard-missing ]]; then printf 'ic-platform-runtime-guard v0.9.1\n'; fi
    if [[ "$BOUNDARY_MODE" == runtime-guard-source ]]; then printf 'ic-platform-runtime-guard v0.9.1 (git+https://example.invalid/guard#12345678)\n'; fi
    if [[ "$BOUNDARY_MODE" == parser ]]; then printf 'candid_parser v0.3.0\n'; fi
    if [[ "$BOUNDARY_MODE" == executor ]]; then printf 'ic-cdk-executor v3.0.0\n'; fi
    if [[ "$BOUNDARY_MODE" == executor-source ]]; then printf 'ic-cdk-executor v2.0.0 (git+https://example.invalid/executor#12345678)\n'; fi
    if [[ "$BOUNDARY_MODE" == generator-source ]]; then printf 'ic-platform-macros v0.9.1 (git+https://example.invalid/macros#12345678)\n'; fi
    # The certified host admits upstream Candid/IC0, but no facade runtime.
    if [[ "$BOUNDARY_MODE" == host-candid ]]; then printf 'candid v0.10.38\n'; fi
    if [[ "$BOUNDARY_MODE" == host-candid-derive ]]; then printf 'candid_derive v0.10.38\n'; fi
    if [[ "$BOUNDARY_MODE" == host-registry-ic0 ]]; then printf 'ic0 v1.2.0\n'; fi
    if [[ "$BOUNDARY_MODE" == host-cdk ]]; then printf 'ic-cdk v0.20.3\n'; fi
    if [[ "$BOUNDARY_MODE" == host-cdk-macros ]]; then printf 'ic-cdk-macros v0.20.3\n'; fi
    if [[ "$BOUNDARY_MODE" == host-profiler ]]; then printf 'dhat v0.3.3\n'; fi
    if [[ "$BOUNDARY_MODE" == host-entropy ]]; then printf 'ic-entropy v0.1.0\n'; fi
    if [[ "$BOUNDARY_MODE" == host-platform-feature ]]; then
        printf 'ic-platform feature "candid"\n'
    fi
    exit 0
fi
printf '%s v0.9.1\n' "$package"
if [[ "$package" == ic-auth-backend-conformance ||
      ( "$package" == ic-auth && ( "$all_features" == yes || "$features" == *signature* || "$features" == certified-proof-verification ) ) ]]; then
    printf 'ic-platform v0.16.0\n'
fi
case "$BOUNDARY_MODE:$package:$features" in
    principal-facade:ic-auth-protocol-types:*|wire-facade:ic-auth-wire:*|control-facade:ic-auth-qualification-contracts:*|engine-facade:ic-auth:|session-facade:ic-auth:sessions)
        printf 'ic-platform v0.16.0\n' ;;
    session-runtime:ic-auth:sessions|principal-runtime-owner:ic-auth-protocol-types:*)
        printf 'ic-platform-runtime v0.17.1\n' ;;
esac
if [[ "$package" == ic-auth-backend-conformance ]]; then
    case "$BOUNDARY_MODE" in
        conformance-metrics) printf 'ic-metrics v0.24.1\n' ;;
        conformance-private) printf 'ic-auth-qualification-contracts v0.28.0\n' ;;
        conformance-certified) printf 'ic-verify-bls-signature v0.6.0\n' ;;
        conformance-profiler) printf 'dhat v0.3.3\n' ;;
        conformance-entropy) printf 'ic-entropy v0.1.0\n' ;;
    esac
fi
if [[ "$package" == ic-auth ]]; then printf 'ic_principal v0.1.5\n'; fi
if [[ "$package" == ic-auth && "$target" == wasm32-unknown-unknown && "$all_features" == yes ]]; then
    case "$BOUNDARY_MODE" in
        crypto-generations|crypto-generations-third)
            for dependency in block-buffer crypto-common digest ff group rand_core sha2; do
                printf '%s v0.1.0\n%s v0.2.0\n' "$dependency" "$dependency"
            done
            if [[ "$BOUNDARY_MODE" == crypto-generations-third ]]; then printf 'digest v0.3.0\n'; fi
            ;;
        crypto-algorithm-duplicate) printf 'k256 v0.13.4\nk256 v0.14.0\n' ;;
    esac
fi
if [[ "$BOUNDARY_MODE" == library-profiler && "$package" == ic-auth && "$target" == wasm32-unknown-unknown && "$features" == sessions ]]; then
    printf 'dhat v0.3.3\n'
fi
if [[ "$BOUNDARY_MODE" == library-entropy && "$package" == ic-auth && "$target" == wasm32-unknown-unknown && "$features" == sessions ]]; then
    printf 'ic-entropy v0.1.0\n'
fi
if [[ "$BOUNDARY_MODE" == principal-external && "$package" == ic-auth-protocol-types ]]; then
    printf 'ic_principal v0.1.5\n'
fi
if [[ "$BOUNDARY_MODE" == wire-hashing && "$package" == ic-auth-wire ]]; then
    printf 'sha2 v0.10.9\n'
fi
if [[ "$BOUNDARY_MODE" == control-external && "$package" == ic-auth-qualification-contracts ]]; then
    printf 'ic_principal v0.1.5\n'
fi
if [[ "$BOUNDARY_MODE" == principal-candid && "$package" == ic-auth-protocol-types ]]; then
    printf 'candid v0.10.38\n'
fi
if [[ "$BOUNDARY_MODE" == principal-executor && "$package" == ic-auth-protocol-types ]]; then
    printf 'ic-cdk-executor v2.0.0\n'
fi
if [[ "$BOUNDARY_MODE" == principal-owned-executor && "$package" == ic-auth-protocol-types ]]; then
    printf 'ic-platform feature "executor"\n'
fi
if [[ "$BOUNDARY_MODE" == principal-runtime-guard && "$package" == ic-auth-protocol-types ]]; then
    printf 'ic-platform-runtime-guard v0.9.1\n'
fi
if [[ "$BOUNDARY_MODE" == principal-bls && "$package" == ic-auth-protocol-types ]]; then
    printf 'ic-verify-bls-signature v0.6.0\n'
fi
if [[ "$BOUNDARY_MODE" == control-candid && "$package" == ic-auth-qualification-contracts ]]; then
    printf 'candid v0.10.38\n'
fi
if [[ "$BOUNDARY_MODE" == control-storage && "$package" == ic-auth-qualification-contracts ]]; then
    printf 'ic-storage v0.53.0\n'
fi
if [[ "$package" == ic-auth ]]; then
    if [[ "$BOUNDARY_MODE" == persistent-storage && ( "$features" == persistent || "$all_features" == yes ) ]]; then printf 'ic-storage v0.73.4\n'; fi
    if [[ "$BOUNDARY_MODE" == wallet-storage && "$features" == portable-signature-verification ]]; then printf 'ic-storage v0.73.4\n'; fi
fi
if [[ "$BOUNDARY_MODE" == principal-storage && "$package" == ic-auth-protocol-types ]]; then
    printf 'ic-storage v0.53.0\n'
fi
if [[ "$BOUNDARY_MODE" == control-platform-feature && "$package" == ic-auth-qualification-contracts ]]; then
    printf 'ic-platform feature "candid"\n'
fi
# A native near-name must not be mistaken for the actual tooling package.
printf 'ic-agentic v1.0.0\ntokiology v1.0.0\n'
if [[ "$target" == wasm32-unknown-unknown ]]; then
    if [[ "$BOUNDARY_MODE" == wasm-default && "$package" == ic-auth-wire ]]; then
        printf 'ic-agent v0.49.2\n'
    elif [[ "$BOUNDARY_MODE" == wasm-combined && "$all_features" == yes ]]; then
        printf 'tokio v1.0.0\n'
    elif [[ "$BOUNDARY_MODE" == wasm-capability && "$features" == sessions ]]; then
        printf 'ic-testkit-candid v0.53.0\n'
    elif [[ "$BOUNDARY_MODE" == wasm-portable-bls && "$features" == portable-signature-verification ]]; then
        printf 'ic-verify-bls-signature v0.6.0\n'
    elif [[ "$BOUNDARY_MODE" == wasm-session-candid && "$features" == sessions ]]; then
        printf 'candid v0.10.38\n'
    elif [[ "$BOUNDARY_MODE" == wasm-session-ic0 && "$features" == sessions ]]; then
        printf 'ic0 v1.2.0\n'
    elif [[ "$BOUNDARY_MODE" == wasm-portable-ic0 && "$features" == portable-signature-verification ]]; then
        printf 'ic0 v1.2.0\n'
    elif [[ "$BOUNDARY_MODE" == wasm-unknown-candid && "$features" == future-capability ]]; then
        printf 'candid v0.10.38\n'
    elif [[ "$features" == certified-proof-verification ]]; then
        case "$BOUNDARY_MODE" in
            wasm-certified-candid) printf 'candid v0.10.38\n' ;;
            wasm-certified-ic0) printf 'ic0 v1.2.0\n' ;;
            wasm-certified-owned-ic0) printf 'ic-platform feature "ic0"\n' ;;
            wasm-certified-runtime) printf 'ic-platform feature "runtime"\n' ;;
            wasm-certified-creation) printf 'ic-canister-sig-creation v1.3.1\n' ;;
            wasm-certified-verifier) printf 'ic-signature-verification v0.3.0\n' ;;
        esac
    fi
fi
STUB
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$root/.tools/host/bin:$PATH"
export BOUNDARY_EVENTS="$fixture/events"
for mode in accepted persistent-storage wallet-storage crypto-generations crypto-generations-third crypto-algorithm-duplicate principal-facade wire-facade control-facade engine-facade session-facade session-runtime principal-runtime-owner conformance-metrics conformance-private conformance-certified conformance-profiler library-profiler host-profiler library-entropy conformance-entropy host-entropy principal-external wire-hashing control-external principal-candid principal-executor principal-owned-executor principal-runtime-guard principal-bls principal-storage control-candid control-storage control-platform-feature wasm-default wasm-combined wasm-capability wasm-portable-bls wasm-portable-ic0 wasm-session-candid wasm-session-ic0 wasm-unknown-candid wasm-certified-owned-ic0 wasm-certified-candid wasm-certified-ic0 wasm-certified-runtime wasm-certified-creation wasm-certified-verifier host-candid host-candid-derive host-registry-ic0 parser executor executor-source generator generator-source runtime-guard-missing runtime-guard-source host-cdk host-cdk-macros host-platform-feature host-runtime-missing host-runtime-duplicate host-facade-ic0 host-facade-executor host-facade-runtime host-wire-duplicate host-storage-missing host-storage-duplicate host-retired-memory host-legacy-stable-structures host-native-client; do
    export BOUNDARY_MODE="$mode"
    : > "$BOUNDARY_EVENTS"
    status=0
    bash "$fixture/repo/scripts/ci/check-auth-boundaries.sh" > "$fixture/$mode.log" 2>&1 || status=$?
    if [[ "$mode" == accepted || "$mode" == persistent-storage || "$mode" == crypto-generations ||
          "$mode" == wasm-certified-candid || "$mode" == wasm-certified-ic0 ||
          "$mode" == wasm-certified-creation || "$mode" == wasm-certified-verifier ||
          "$mode" == host-candid || "$mode" == host-candid-derive || "$mode" == host-registry-ic0 ]]; then
        [[ "$status" == 0 ]] || { cat "$fixture/$mode.log"; exit 1; }
        # Replace Auth's default selection with all three public defaults, then
        # add private Wire schemas per target, two final host checks and the
        # isolated native conformance dependency check.
        [[ "$(wc -l < "$BOUNDARY_EVENTS")" == "$((2 * (selection_count + 3) + 3))" ]]
        [[ "$(rg -c -- '--package ic-auth-backend-conformance' "$BOUNDARY_EVENTS")" == 1 ]]
        [[ "$(rg -c -- '--features future-capability' "$BOUNDARY_EVENTS")" == 2 ]]
        rg -- '--target x86_64-unknown-linux-gnu' "$BOUNDARY_EVENTS" > /dev/null
        rg -- '--target wasm32-unknown-unknown' "$BOUNDARY_EVENTS" > /dev/null
    else
        [[ "$status" != 0 ]] || { echo "missing exclusion: $mode" >&2; exit 1; }
        rg '^Cargo graph policy failed:' "$fixture/$mode.log" > /dev/null
        rg 'ic-platform|ic-wire|ic-memory|ic-storage|ic-stable-structures|ic_principal|sha2|digest|k256|ic-agent|tokio|ic-testkit-candid|ic-entropy|candid|ic-cdk|ic0|ic-canister-sig-creation|ic-signature-verification|ic-verify-bls-signature|dhat' "$fixture/$mode.log" > /dev/null
    fi
done
echo 'Native/Wasm graph selections and transitive tooling refusals passed'
