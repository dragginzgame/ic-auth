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
cp scripts/ci/{check-auth-boundaries,auth-dependency-policy}.sh "$fixture/repo/scripts/ci/"
cp scripts/ci/check-cargo-graph.pl "$fixture/repo/scripts/ci/"
printf '[workspace]\n' > "$fixture/repo/Cargo.toml"
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
    printf 'ic-auth-qualification-canister v0.9.1\nic-cdk-executor v2.0.0\nic-cdk-executor v2.0.0 (*)\n'
    if [[ "$BOUNDARY_MODE" == parser ]]; then printf 'candid_parser v0.3.0\n'; fi
    if [[ "$BOUNDARY_MODE" == executor ]]; then printf 'ic-cdk-executor v3.0.0\n'; fi
    if [[ "$BOUNDARY_MODE" == executor-source ]]; then printf 'ic-cdk-executor v2.0.0 (git+https://example.invalid/executor#12345678)\n'; fi
    # Certified verification still imports Candid, while the Wire runtime must
    # not select Platform Candid or the retired full CDK packages.
    printf 'candid v0.10.38\n'
    if [[ "$BOUNDARY_MODE" == host-cdk ]]; then printf 'ic-cdk v0.20.3\n'; fi
    if [[ "$BOUNDARY_MODE" == host-cdk-macros ]]; then printf 'ic-cdk-macros v0.20.3\n'; fi
    if [[ "$BOUNDARY_MODE" == host-platform-feature ]]; then
        printf 'ic-platform feature "candid"\n'
    fi
    exit 0
fi
printf '%s v0.9.1\n' "$package"
printf 'ic-platform v0.1.12\nic_principal v0.1.5\n'
if [[ "$BOUNDARY_MODE" == principal-candid && "$package" == ic-auth-protocol-types ]]; then
    printf 'candid v0.10.38\n'
fi
if [[ "$BOUNDARY_MODE" == principal-executor && "$package" == ic-auth-protocol-types ]]; then
    printf 'ic-cdk-executor v2.0.0\n'
fi
if [[ "$BOUNDARY_MODE" == principal-bls && "$package" == ic-auth-protocol-types ]]; then
    printf 'ic-verify-bls-signature v0.6.0\n'
fi
if [[ "$BOUNDARY_MODE" == control-candid && "$package" == ic-auth-qualification-contracts ]]; then
    printf 'candid v0.10.38\n'
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
    fi
fi
STUB
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$root/.tools/host/bin:$PATH"
export BOUNDARY_EVENTS="$fixture/events"
for mode in accepted principal-candid principal-executor principal-bls control-candid control-platform-feature wasm-default wasm-combined wasm-capability wasm-portable-bls parser executor executor-source host-cdk host-cdk-macros host-platform-feature; do
    export BOUNDARY_MODE="$mode"
    : > "$BOUNDARY_EVENTS"
    status=0
    bash "$fixture/repo/scripts/ci/check-auth-boundaries.sh" > "$fixture/$mode.log" 2>&1 || status=$?
    if [[ "$mode" == accepted ]]; then
        [[ "$status" == 0 ]] || { cat "$fixture/$mode.log"; exit 1; }
        # Three public defaults, nine feature selections and the private Wire
        # schema on both targets, plus crypto-generation and reference host.
        [[ "$(wc -l < "$BOUNDARY_EVENTS")" == 28 ]]
        rg -- '--target x86_64-unknown-linux-gnu' "$BOUNDARY_EVENTS" > /dev/null
        rg -- '--target wasm32-unknown-unknown' "$BOUNDARY_EVENTS" > /dev/null
    else
        [[ "$status" != 0 ]] || { echo "missing exclusion: $mode" >&2; exit 1; }
        rg '^Cargo graph policy failed:' "$fixture/$mode.log" > /dev/null
        rg 'ic-agent|tokio|ic-testkit-candid|candid|ic-cdk|ic-verify-bls-signature' "$fixture/$mode.log" > /dev/null
    fi
done
echo 'Native/Wasm graph selections and transitive tooling refusals passed'
