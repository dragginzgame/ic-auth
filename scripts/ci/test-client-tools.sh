#!/usr/bin/env bash
# Exercise the actual CI preparation pipeline and Make callers with owned effects.
set -euo pipefail
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && pwd -P)"
mkdir -p "$root/target/portable-fixtures"
fixture="$(mktemp -d "$root/target/portable-fixtures/client-tools.XXXXXX")"
complete=false
finish() {
    local status=$?
    [[ "$complete" == true || "$status" != 0 ]] || status=1
    printf 'Client setup evidence retained: %s\n' "$fixture"
    exit "$status"
}
trap finish EXIT
REAL_CLIENT_MAKE="$(command -v make)"
CLIENT_FIXTURE_NODE_PIN="$(cat "$root/packages/client/.nvmrc")"
CLIENT_FIXTURE_NPM_PIN="$(node -p 'require(process.argv[1]).packageManager.slice(4)' "$root/packages/client/package.json")"
export REAL_CLIENT_MAKE CLIENT_FIXTURE_NODE_PIN CLIENT_FIXTURE_NPM_PIN
mkdir -p "$fixture/templates"
yq -r '.jobs.rust.steps[] | select(.name == "Prepare selected client package manager and locked dependencies") | .run' \
    "$root/.github/workflows/ci.yml" > "$fixture/workflow.sh"
cat > "$fixture/templates/node" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
case "$*" in
    --version) printf 'v%s\n' "${CLIENT_FIXTURE_NODE_VERSION:-$CLIENT_FIXTURE_NODE_PIN}" ;;
    '-p process.execPath') printf '%s/bin/node\n' "$CLIENT_FIXTURE_NODE_PREFIX" ;;
    *packageManager.slice*)
        [[ "${CLIENT_FIXTURE_PIN_EXIT:-0}" == 0 ]] || exit "$CLIENT_FIXTURE_PIN_EXIT"
        printf '%s\n' "$CLIENT_FIXTURE_NPM_PIN" ;;
    *) exit 79 ;;
esac
SCRIPT
cat > "$fixture/templates/npm-installed" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
case "$*" in
    --version) printf '%s\n' "${CLIENT_FIXTURE_INSTALLED_VERSION:-$CLIENT_FIXTURE_NPM_PIN}" ;;
    'prefix --global') printf '%s\n' "$CLIENT_FIXTURE_OLD_PREFIX" ;;
    'ci --ignore-scripts --no-audit --no-fund')
        echo dependencies >> "$CLIENT_FIXTURE_EVENTS"
        [[ "${CLIENT_FIXTURE_DEPENDENCY_EXIT:-0}" == 0 ]] || exit "$CLIENT_FIXTURE_DEPENDENCY_EXIT"
        mkdir -p node_modules/.bin
        for tool in tsc; do
            printf '#!/bin/sh\nexit 0\n' > "node_modules/.bin/$tool"
            chmod +x "node_modules/.bin/$tool"
        done ;;
    *) exit 79 ;;
esac
SCRIPT
cat > "$fixture/templates/npm-bootstrap" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
case "$*" in
    --version) echo 11.0.0; exit 0 ;;
    'prefix --global') printf '%s\n' "$CLIENT_FIXTURE_OLD_PREFIX"; exit 0 ;;
esac
[[ "$1" == install ]] || exit 79
prefix=''; package=''; scripts=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix) prefix="$2"; shift ;;
        npm@*) package="$1" ;;
        --ignore-scripts) scripts=true ;;
    esac
    shift
done
printf 'bootstrap-prefix:%s\n' "$prefix" >> "$CLIENT_FIXTURE_EVENTS"
[[ "$prefix" == "$CLIENT_FIXTURE_NODE_PREFIX" && "$package" == "npm@$CLIENT_FIXTURE_NPM_PIN" && "$scripts" == true ]] || exit 79
[[ "${CLIENT_FIXTURE_SETUP_EXIT:-0}" == 0 ]] || exit "$CLIENT_FIXTURE_SETUP_EXIT"
cp "$CLIENT_FIXTURE_TEMPLATE/npm-installed" "$prefix/bin/npm"
chmod +x "$prefix/bin/npm"
SCRIPT
cat > "$fixture/templates/make" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
printf 'make:%s\n' "$*" >> "$CLIENT_FIXTURE_EVENTS"
exec "$REAL_CLIENT_MAKE" "$@"
SCRIPT
chmod +x "$fixture/templates/"*
export CLIENT_FIXTURE_TEMPLATE="$fixture/templates"
for scenario in separate-prefix wrong-node setup-failure wrong-prepared-npm dependency-failure missing-path selection-failure; do
    consumer="$fixture/$scenario/consumer with spaces"
    export CLIENT_FIXTURE_NODE_PREFIX="$fixture/$scenario/node install"
    export CLIENT_FIXTURE_OLD_PREFIX="$fixture/$scenario/unrelated global"
    export CLIENT_FIXTURE_EVENTS="$fixture/$scenario/events"
    export GITHUB_PATH="$fixture/$scenario/github-path"
    mkdir -p "$consumer/scripts/dev" "$consumer/scripts/ci" "$consumer/packages/client" "$consumer/ci" \
        "$consumer/target/browser-client" "$CLIENT_FIXTURE_NODE_PREFIX/bin" "$CLIENT_FIXTURE_OLD_PREFIX" \
        "$fixture/$scenario/foreign" "$fixture/$scenario/commands"
    : > "$CLIENT_FIXTURE_EVENTS"
    printf 'prior-path\n' > "$GITHUB_PATH"
    cp "$GITHUB_PATH" "$fixture/$scenario/path-before"
    printf 'unrelated installation\n' > "$CLIENT_FIXTURE_OLD_PREFIX/sentinel"
    cp "$root/Makefile" "$consumer/"
    cp -Rp "$root/make" "$consumer/"
    cp "$root/ci/tool-versions.env" "$consumer/ci/"
    cp "$root/scripts/dev/client-tools.sh" "$root/scripts/dev/prepare-client-ci.sh" "$root/scripts/dev/client-toolchain.sh" "$consumer/scripts/dev/"
    cp "$root/scripts/ci/check-make-execution.sh" "$consumer/scripts/ci/"
    cp "$root/packages/client/package.json" "$root/packages/client/.nvmrc" "$consumer/packages/client/"
    cp "$fixture/templates/node" "$CLIENT_FIXTURE_NODE_PREFIX/bin/node"
    ln -s "$CLIENT_FIXTURE_NODE_PREFIX/bin/node" "$fixture/$scenario/foreign/node"
    cp "$fixture/templates/npm-bootstrap" "$fixture/$scenario/foreign/npm"
    cp "$fixture/templates/make" "$fixture/$scenario/commands/make"
    unset CLIENT_FIXTURE_PIN_EXIT CLIENT_FIXTURE_NODE_VERSION CLIENT_FIXTURE_SETUP_EXIT CLIENT_FIXTURE_INSTALLED_VERSION CLIENT_FIXTURE_DEPENDENCY_EXIT
    expected=0
    case "$scenario" in
        wrong-node) export CLIENT_FIXTURE_NODE_VERSION=18.0.0; expected=1 ;;
        setup-failure) export CLIENT_FIXTURE_SETUP_EXIT=17; expected=17 ;;
        wrong-prepared-npm) export CLIENT_FIXTURE_INSTALLED_VERSION=11.0.0; expected=1 ;;
        dependency-failure) export CLIENT_FIXTURE_DEPENDENCY_EXIT=23; expected=2 ;;
        missing-path) unset GITHUB_PATH; expected=2 ;;
        selection-failure) export CLIENT_FIXTURE_PIN_EXIT=29; expected=29 ;;
    esac
    status=0
    (cd "$consumer"; PATH="$fixture/$scenario/commands:$fixture/$scenario/foreign:$PATH" \
        "$BASH" "$fixture/workflow.sh") > "$fixture/$scenario/output.log" 2>&1 || status=$?
    [[ "$status" == "$expected" ]]
    [[ "$(cat "$CLIENT_FIXTURE_OLD_PREFIX/sentinel")" == 'unrelated installation' ]]
    if [[ "$scenario" == separate-prefix ]]; then
        grep -F 'make:install-client-dependencies' "$CLIENT_FIXTURE_EVENTS" >/dev/null
        grep -F 'make:client-tools-check' "$CLIENT_FIXTURE_EVENTS" >/dev/null
        [[ "$(tail -1 "$GITHUB_PATH")" == "$CLIENT_FIXTURE_NODE_PREFIX/bin" ]]
        # A later step must select the admitted npm through the published path.
        (cd "$consumer"; PATH="$CLIENT_FIXTURE_NODE_PREFIX/bin:$fixture/$scenario/foreign:$PATH" \
            "$REAL_CLIENT_MAKE" client-tools-check) > "$fixture/$scenario/later-step.log" 2>&1
    else
        cmp "$fixture/$scenario/path-before" "$fixture/$scenario/github-path"
        if grep -F 'make:client-tools-check' "$CLIENT_FIXTURE_EVENTS" >/dev/null; then exit 1; fi
        if [[ "$scenario" != dependency-failure ]]; then
            if grep -F 'make:install-client-dependencies' "$CLIENT_FIXTURE_EVENTS" >/dev/null; then exit 1; fi
        fi
    fi
    if [[ "$scenario" == wrong-node || "$scenario" == wrong-prepared-npm ]]; then
        grep -F 'client setup refused' "$fixture/$scenario/output.log" >/dev/null
    fi
    if [[ "$scenario" == wrong-node ]]; then
        status=0
        (cd "$consumer"; PATH="$fixture/$scenario/foreign:$PATH" \
            "$BASH" scripts/dev/client-tools.sh check) > "$fixture/$scenario/offline-refusal.log" 2>&1 || status=$?
        [[ "$status" == 1 ]]
        grep -F 'Observed Node=v18.0.0 path=' "$fixture/$scenario/offline-refusal.log" >/dev/null
        grep -F 'npm=11.0.0 path=' "$fixture/$scenario/offline-refusal.log" >/dev/null
    fi
    if [[ "$scenario" == wrong-prepared-npm || "$scenario" == selection-failure ]]; then
        status=0
        (cd "$consumer"; PATH="$CLIENT_FIXTURE_NODE_PREFIX/bin:$fixture/$scenario/foreign:$PATH"             "$BASH" scripts/dev/client-tools.sh check) > "$fixture/$scenario/offline-refusal.log" 2>&1 || status=$?
        if [[ "$scenario" == selection-failure ]]; then
            [[ "$status" == 29 ]]
        else
            [[ "$status" == 1 ]]
            grep -F 'npm=11.0.0 path=' "$fixture/$scenario/offline-refusal.log" >/dev/null
        fi
    fi

done
echo 'Actual client CI pipeline: selected-prefix activation, later-step admission and first-failure preservation passed (substituted npm effects)'
# Contract generation uses Rust alone; deliberately unavailable browser tools
# must not prevent generation/checking, or get consulted by either operation.
consumer="$fixture/rust-contracts"
mkdir -p "$consumer/scripts/dev" "$consumer/packages/client/src/generated" "$consumer/bin"
cp "$root/scripts/dev/client-contracts.sh" "$consumer/scripts/dev/"
cat > "$consumer/scripts/dev/client-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
exit 89
SCRIPT
for tool in node npm tsc; do
    cp "$consumer/scripts/dev/client-tools.sh" "$consumer/bin/$tool"
    chmod +x "$consumer/bin/$tool"
done
# Substitute Rust and file-tool effects; execute the actual generation driver.
cat > "$consumer/scripts/dev/run-host-file.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
[[ "$1" == hash-files0 ]]
cat
SCRIPT
cat > "$consumer/bin/cargo" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
[[ "$*" == 'build --quiet --locked --offline -p ic-auth-tooling' ]]
mkdir -p "$CARGO_TARGET_DIR/debug"
cp "$PWD/generator" "$CARGO_TARGET_DIR/debug/ic-auth-tooling"
SCRIPT
cat > "$consumer/generator" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
[[ "$1 $2" == 'client-contracts generate' ]]
for name in credential.ts establish.ts read.ts logout.ts wire-runtime.ts credential-vector.hex logout-vector.hex admission-vector.hex; do
    printf 'rust-owned %s\n' "$name" > "$3/$name"
done
SCRIPT
chmod +x "$consumer/bin/cargo" "$consumer/generator"
for mode in generate check; do
    (cd "$consumer"; PATH="$consumer/bin:$PATH" bash scripts/dev/client-contracts.sh "$mode") \
        > "$fixture/contracts-$mode.log" 2>&1
done
printf 'stale\n' > "$consumer/packages/client/src/generated/credential.ts"
if (cd "$consumer"; PATH="$consumer/bin:$PATH" bash scripts/dev/client-contracts.sh check) \
    > "$fixture/contracts-stale.log" 2>&1; then
    echo 'stale generated contract unexpectedly accepted' >&2; exit 1
fi
rg -F 'differs from the locked generator' "$fixture/contracts-stale.log" > /dev/null
[[ "$(cat "$consumer/packages/client/src/generated/credential.ts")" == stale ]]
echo 'Rust-only contract generation, comparison and effect-free stale refusal passed (substituted Rust effects)'
complete=true
