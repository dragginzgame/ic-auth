#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/testkit-tools.XXXXXX")"
printf 'Testkit caller evidence retained: %s\n' "$fixture"
consumer="$fixture/consumer with spaces"
mkdir -p "$consumer/scripts/dev" "$consumer/.tools/ic/bin" "$fixture/bin"
cp scripts/dev/testkit-tools.sh scripts/dev/test-qualification.sh "$consumer/scripts/dev/"
cat > "$consumer/Cargo.lock" <<'LOCK'
version = 4
[[package]]
name = "ic-testkit"
version = "0.25.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
LOCK
cp "$consumer/Cargo.lock" "$fixture/selected.lock"
printf 'retained former bundle\n' > "$consumer/.tools/ic/bin/pocket-ic"
export TESTKIT_FIXTURE_LOG="$fixture/dispatch.log"
export TESTKIT_FIXTURE_CLI="$fixture/ic-testkit-server"
export TESTKIT_FIXTURE_SERVER="$fixture/admitted server"
export TESTKIT_FIXTURE_READY="$fixture/cli-ready"
export TESTKIT_FIXTURE_CONSUMER="$consumer"
export TESTKIT_FIXTURE_LOCK="$fixture/selected.lock"
export PATH="$root/.tools/host/bin:$fixture/bin:$PATH"
printf 'synthetic server bytes\n' > "$TESTKIT_FIXTURE_SERVER"
cat > "$consumer/scripts/dev/install-rust-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ $# == 4 || ( $# == 5 && "${5:-}" == --check ) ]]; then
    if [[ $# == 5 ]]; then target=rust-tools-check; else target=install-rust-tools; fi
    printf '%s\n' "$target" >> "$TESTKIT_FIXTURE_LOG"
    exit 0
fi
if [[ $# == 5 && "$1" == --consumer && "$2" == "$TESTKIT_FIXTURE_CONSUMER" &&
      "$3" == --versions && "$4" == "$TESTKIT_FIXTURE_CONSUMER/ci/tool-versions.env" && "$5" == --preflight ]]; then
    printf 'rust preflight\n' >> "$TESTKIT_FIXTURE_LOG"
    exit "${TESTKIT_FIXTURE_RUST_PREFLIGHT_STATUS:-0}"
fi
[[ $# == 12 || $# == 13 ]] || exit 1
[[ "$1" == --consumer && "$2" == "$TESTKIT_FIXTURE_CONSUMER" &&
   "$3" == --package && "$4" == ic-testkit && "$5" == --lockfile &&
   "$6" == Cargo.lock && "$7" == --bin && "$8" == ic-testkit-server &&
   "$9" == --profile && "${10}" == debug &&
   "${11}" == --features && "${12}" == cli ]] || exit 1
if [[ $# == 13 ]]; then
    [[ "${13}" == --check ]] || exit 1
    printf 'cli check\n' >> "$TESTKIT_FIXTURE_LOG"
    if ! cmp -s "$TESTKIT_FIXTURE_LOCK" "$TESTKIT_FIXTURE_CONSUMER/Cargo.lock"; then
        if [[ "${TESTKIT_FIXTURE_CHANGED_CLI_READY:-0}" == 1 ]]; then
            printf '%s-next\n' "$TESTKIT_FIXTURE_CLI"
            exit 0
        fi
        echo 'missing selected Cargo tool: package=ic-testkit version=0.26.0 target=bin:ic-testkit-server profile=debug' >&2
        exit 27
    fi
    [[ -f "$TESTKIT_FIXTURE_READY" ]] || {
        echo 'missing selected Cargo tool: package=ic-testkit version=0.25.5 target=bin:ic-testkit-server profile=debug' >&2
        exit 27
    }
else
    printf 'cli install\n' >> "$TESTKIT_FIXTURE_LOG"
    [[ "${TESTKIT_FIXTURE_INSTALL_STATUS:-0}" == 0 ]] || exit "$TESTKIT_FIXTURE_INSTALL_STATUS"
    touch "$TESTKIT_FIXTURE_READY"
fi
printf '%s\n' "$TESTKIT_FIXTURE_CLI"
SCRIPT
cat > "$consumer/scripts/dev/install-ic-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
printf 'unexpected unselected IC tool dispatch\n' >> "$TESTKIT_FIXTURE_LOG"
exit 99
SCRIPT
cat > "$TESTKIT_FIXTURE_CLI" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 3 && "$2" == --directory && "$3" == "$TESTKIT_FIXTURE_CONSUMER/.tools/testkit-server" ]] || exit 1
printf 'server %s\n' "$1" >> "$TESTKIT_FIXTURE_LOG"
case "$1" in
    setup)
        [[ "${TESTKIT_FIXTURE_SETUP_STATUS:-0}" == 0 ]] || exit "$TESTKIT_FIXTURE_SETUP_STATUS"
        mkdir -p "$3"
        touch "$3/admitted"
        ;;
    check)
        [[ "${TESTKIT_FIXTURE_CHECK_STATUS:-0}" == 0 ]] || exit "$TESTKIT_FIXTURE_CHECK_STATUS"
        [[ -f "$3/admitted" ]] || exit 29
        ;;
    *) exit 2 ;;
esac
if [[ "${TESTKIT_FIXTURE_CHANGE_SELECTION:-0}" == 1 ]]; then
    sed 's/0.25.5/0.26.0/' "$TESTKIT_FIXTURE_CONSUMER/Cargo.lock" > "$3/changed.lock"
    cp "$3/changed.lock" "$TESTKIT_FIXTURE_CONSUMER/Cargo.lock"
fi
printf '%s\n' "$TESTKIT_FIXTURE_SERVER"
SCRIPT
cat > "$fixture/bin/cargo" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo dispatch %s\n' "$*" >> "$TESTKIT_FIXTURE_CONSUMER/cargo-dispatch.log"
[[ "$POCKET_IC_BIN" == "$TESTKIT_FIXTURE_SERVER" ]] || exit 1
[[ "$IC_AUTH_QUALIFICATION_WASM" == "$TESTKIT_FIXTURE_CONSUMER/target/wasm32-unknown-unknown/release/ic_auth_qualification_canister.wasm" ]] || exit 1
[[ -d "$IC_AUTH_QUALIFICATION_STATE_ROOT" && "$TMPDIR" == "$IC_AUTH_QUALIFICATION_STATE_ROOT" ]] || exit 1
[[ "$*" == 'test --locked --offline -p ic-auth-qualification --test signatures -- --nocapture' ]] || exit 1
printf 'qualified admitted server\n' >> "$TESTKIT_FIXTURE_LOG"
SCRIPT
cat > "$consumer/scripts/dev/client-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == check ]] || exit 1
printf 'client tools check\n' >> "$TESTKIT_FIXTURE_LOG"
exit "${TESTKIT_FIXTURE_CLIENT_STATUS:-0}"
SCRIPT
# Model contract generation/comparison effects while keeping the real runner's
# ordering and failure propagation. The runner admits client tools separately.
cat > "$consumer/scripts/dev/client-contracts.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == check ]] || exit 1
printf 'client contracts check\n' >> "$TESTKIT_FIXTURE_LOG"
exit "${TESTKIT_FIXTURE_CONTRACTS_STATUS:-0}"
SCRIPT
cat > "$fixture/bin/npm" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == '--prefix packages/client run build' ]] || exit 1
printf 'client build\n' >> "$TESTKIT_FIXTURE_LOG"
exit "${TESTKIT_FIXTURE_CLIENT_BUILD_STATUS:-0}"
SCRIPT
chmod +x "$TESTKIT_FIXTURE_CLI" "$fixture/bin/cargo" "$fixture/bin/npm"
cp "$consumer/scripts/dev/install-rust-tools.sh" "$fixture/mock-installer.sh"
ln -s "$BASH" "$fixture/bin/bash"

# Missing selected CLI refuses even when an old shared executable is retained.
status=0
"$BASH" "$consumer/scripts/dev/testkit-tools.sh" check > "$fixture/missing.stdout" 2> "$fixture/missing.stderr" || status=$?
[[ "$status" == 27 && ! -s "$fixture/missing.stdout" ]] || exit 1
[[ "$(cat "$TESTKIT_FIXTURE_LOG")" == 'cli check' ]] || exit 1
rg -q 'package=ic-testkit version=0[.]25[.]5 target=bin:ic-testkit-server' "$fixture/missing.stderr"
rg -F 'make install-testkit-tools' "$fixture/missing.stderr" > /dev/null

# Actual Make entry points refuse before Wasm compilation or any later CI lane,
# including parallel Make. Substitute the already-prepared common bundle only;
# Testkit admission and the validation runner remain their actual callers.
mkdir -p "$consumer/make" "$consumer/ci" "$consumer/scripts/ci"
for input in Makefile make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk \
    scripts/ci/check-release-delivery.sh ci/release-delivery.txt \
    ci/tool-versions.env ci/ic-tools.tsv ci/cargo-dependency-boundaries.json ci/external-ic-dependencies.txt ci/host-file.env \
    scripts/dev/run-host-file.sh Cargo.toml rust-toolchain.toml \
    packages/client/package.json packages/client/package-lock.json packages/client/.nvmrc \
    scripts/ci/check-make-execution.sh scripts/ci/verify-file-checksum.sh \
    scripts/ci/run-validation-targets.sh; do
    mkdir -p "$consumer/$(dirname "$input")"
    cp -p "$input" "$consumer/$input"
done
cat > "$consumer/scripts/dev/install-host-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${*: -1}" == --check ]]; then target=host-tools-check; else target=install-host-tools; fi
printf '%s\n' "$target" >> "$TESTKIT_FIXTURE_LOG"
SCRIPT
cat > "$consumer/scripts/dev/msrv-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    install) printf 'install-msrv\n' >> "$TESTKIT_FIXTURE_LOG" ;;
    check) printf 'msrv-tools-check\n' >> "$TESTKIT_FIXTURE_LOG"; exit "${TESTKIT_FIXTURE_MSRV_STATUS:-0}" ;;
    *) exit 2 ;;
esac
SCRIPT
cat > "$consumer/scripts/dev/host-file-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    install|check) printf 'file tools %s\n' "$1" >> "$TESTKIT_FIXTURE_LOG" ;;
    *) exit 2 ;;
esac
SCRIPT
cat > "$consumer/scripts/ci/verify-shared-tooling-snapshot.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
printf 'snapshot check\n' >> "$TESTKIT_FIXTURE_LOG"
if [[ -n "${TESTKIT_FIXTURE_INPUT_EDIT:-}" ]]; then
    case "${TESTKIT_FIXTURE_INPUT_ACTION:-append}" in
        append) printf '\n' >> "$TESTKIT_FIXTURE_INPUT_EDIT" ;;
        delete) rm "$TESTKIT_FIXTURE_INPUT_EDIT" ;;
        *) exit 2 ;;
    esac
fi
exit "${TESTKIT_FIXTURE_SNAPSHOT_STATUS:-0}"
SCRIPT
for target in test-qualification ci; do
    : > "$TESTKIT_FIXTURE_LOG"
    status=0
    (cd "$consumer"; make --no-print-directory -j4 "$target") \
        > "$fixture/early-$target.log" 2>&1 || status=$?
    if [[ "$status" != 2 || -e "$consumer/cargo-dispatch.log" ]] ||
        ! rg -Fx 'cli check' "$TESTKIT_FIXTURE_LOG" > /dev/null ||
        ! rg -F 'make install-testkit-tools' "$fixture/early-$target.log" > /dev/null; then
        printf 'Unexpected early %s refusal (status %s); log: %s\n' \
            "$target" "$status" "$fixture/early-$target.log" >&2
        cat "$fixture/early-$target.log" >&2
        exit 1
    fi
    if [[ "$target" == ci ]]; then
        printf '%s\n' 'snapshot check' host-tools-check rust-tools-check \
            msrv-tools-check 'cli check' > "$fixture/expected-early.log"
        cmp "$fixture/expected-early.log" "$TESTKIT_FIXTURE_LOG"
    fi
done

# The actual CI roster seals both Rust and frontend preparation before stages.
# Each independent export mutates one declared input in the first stage; no
# common/product admission or Cargo dispatch may follow, even under parallel Make.
for scenario in rust frontend delivery boundaries external-boundaries file-pin file-selector file-runner deleted failed-stage; do
    mutant="$fixture/drift-$scenario"
    cp -R "$consumer" "$mutant"
    : > "$TESTKIT_FIXTURE_LOG"
    input=Cargo.lock; action=append; snapshot_status=0
    case "$scenario" in
        frontend) input=packages/client/package-lock.json ;;
        delivery) input=ci/release-delivery.txt ;;
        boundaries) input=ci/cargo-dependency-boundaries.json ;;
        external-boundaries) input=ci/external-ic-dependencies.txt ;;
        file-pin) input=ci/host-file.env ;;
        file-selector) input=scripts/dev/host-file-tools.sh ;;
        file-runner) input=scripts/dev/run-host-file.sh ;;
        deleted) input=rust-toolchain.toml; action=delete ;;
        failed-stage) snapshot_status=37 ;;
    esac
    status=0
    (cd "$mutant"; TESTKIT_FIXTURE_INPUT_EDIT="$input" TESTKIT_FIXTURE_INPUT_ACTION="$action" \
        TESTKIT_FIXTURE_SNAPSHOT_STATUS="$snapshot_status" make --no-print-directory -j4 ci) \
        > "$fixture/drift-$scenario.log" 2>&1 || status=$?
    [[ "$status" == 2 && "$(cat "$TESTKIT_FIXTURE_LOG")" == 'snapshot check' && \
       ! -e "$mutant/cargo-dispatch.log" ]] || exit 1
    rg -F "validation input $input" "$fixture/drift-$scenario.log" > /dev/null || \
        rg -F "validation input changed: $input" "$fixture/drift-$scenario.log" > /dev/null || exit 1
    rg -F 'Restart preparation and the full gate against the current inputs' "$fixture/drift-$scenario.log" > /dev/null || exit 1
    [[ "$scenario" != failed-stage ]] || rg -F 'Error 37' "$fixture/drift-$scenario.log" > /dev/null || exit 1
done

# Real consumer aggregates retain common/product ordering under parallel Make.
# Only explicit setup dispatches the selected CLI installer and server setup;
# individual product setup targets still admit their host prerequisite.
: > "$TESTKIT_FIXTURE_LOG"
status=0
(cd "$consumer"; TESTKIT_FIXTURE_RUST_PREFLIGHT_STATUS=39 make --no-print-directory -j4 install-tools) \
    > "$fixture/preflight-rust.log" 2>&1 || status=$?
printf 'rust preflight\n' > "$fixture/expected-preflight.log"
[[ "$status" == 2 && ! -e "$TESTKIT_FIXTURE_READY" && ! -e "$consumer/cargo-dispatch.log" ]] || exit 1
cmp "$fixture/expected-preflight.log" "$TESTKIT_FIXTURE_LOG"
: > "$TESTKIT_FIXTURE_LOG"
(cd "$consumer"; make --no-print-directory -j4 install-tools) > "$fixture/setup.log" 2>&1
printf '%s\n' 'rust preflight' install-host-tools install-rust-tools \
    install-host-tools install-msrv install-host-tools 'cli install' 'server setup' 'cli check' \
    install-host-tools 'file tools install' > "$fixture/expected-setup.log"
cmp "$fixture/expected-setup.log" "$TESTKIT_FIXTURE_LOG"
: > "$TESTKIT_FIXTURE_LOG"
(cd "$consumer"; make --no-print-directory -j4 tools-check) > "$fixture/check.log" 2>&1
printf '%s\n' host-tools-check rust-tools-check msrv-tools-check \
    'cli check' 'server check' 'cli check' 'file tools check' > "$fixture/expected-check.log"
cmp "$fixture/expected-check.log" "$TESTKIT_FIXTURE_LOG"

# Exercise the actual sequential CI recipe under parallel Make. Substitute only
# compiler effects and the first post-lint check: both native and Wasm lint must
# succeed before anything later runs. Keep the real tool-admission ordering.
lint_consumer="$fixture/lint consumer"
cp -R "$consumer" "$lint_consumer"
mkdir "$fixture/lint-bin"
cat > "$fixture/lint-bin/cargo" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "${1:-}" == clippy ]] || exit 99
case " $* " in
    *' --target wasm32-unknown-unknown '*) phase=wasm ;;
    *) phase=native ;;
esac
printf 'clippy %s\n' "$phase" >> "$TESTKIT_FIXTURE_LOG"
if [[ "$phase" == native ]]; then exit "${TESTKIT_FIXTURE_NATIVE_LINT_STATUS:-0}"; fi
exit "${TESTKIT_FIXTURE_WASM_LINT_STATUS:-0}"
SCRIPT
chmod +x "$fixture/lint-bin/cargo"
cat > "$lint_consumer/scripts/ci/check-dependency-pins.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
printf 'post-lint gate\n' >> "$TESTKIT_FIXTURE_LOG"
exit 41
SCRIPT
for scenario in native wasm success; do
    : > "$TESTKIT_FIXTURE_LOG"
    native_status=0; wasm_status=0
    case "$scenario" in
        native) native_status=37 ;;
        wasm) wasm_status=38 ;;
    esac
    status=0
    (cd "$lint_consumer"; PATH="$fixture/lint-bin:$PATH" \
        TESTKIT_FIXTURE_CONSUMER="$lint_consumer" \
        TESTKIT_FIXTURE_NATIVE_LINT_STATUS="$native_status" \
        TESTKIT_FIXTURE_WASM_LINT_STATUS="$wasm_status" \
        make --no-print-directory -j4 ci) > "$fixture/lint-$scenario.log" 2>&1 || status=$?
    [[ "$status" == 2 ]]
    printf 'snapshot check\n' > "$fixture/expected-lint-$scenario.log"
    cat "$fixture/expected-check.log" >> "$fixture/expected-lint-$scenario.log"
    printf 'clippy native\n' >> "$fixture/expected-lint-$scenario.log"
    if [[ "$scenario" != native ]]; then
        printf 'clippy wasm\n' >> "$fixture/expected-lint-$scenario.log"
    fi
    if [[ "$scenario" == success ]]; then
        printf 'post-lint gate\n' >> "$fixture/expected-lint-$scenario.log"
        rg -F 'Error 41' "$fixture/lint-$scenario.log" > /dev/null
    else
        if [[ "$scenario" == native ]]; then expected_status=37; else expected_status=38; fi
        rg -F "Error $expected_status" "$fixture/lint-$scenario.log" > /dev/null
        rg -F 'VALIDATION FAILED: clippy' "$fixture/lint-$scenario.log" > /dev/null
    fi
    cmp "$fixture/expected-lint-$scenario.log" "$TESTKIT_FIXTURE_LOG"
done

# Missing minimum compiler refuses before Testkit, without installation.
: > "$TESTKIT_FIXTURE_LOG"
status=0
(cd "$consumer"; TESTKIT_FIXTURE_MSRV_STATUS=31 make --no-print-directory -j4 tools-check) \
    > "$fixture/missing-msrv.log" 2>&1 || status=$?
[[ "$status" == 2 ]] || exit 1
printf '%s\n' host-tools-check rust-tools-check msrv-tools-check > "$fixture/expected-msrv.log"
cmp "$fixture/expected-msrv.log" "$TESTKIT_FIXTURE_LOG"

# Original setup/installation/admission failures cannot report an identity.
for phase in install check; do
    status=0
    TESTKIT_FIXTURE_SETUP_STATUS=39 TESTKIT_FIXTURE_CHECK_STATUS=33 \
        "$BASH" "$consumer/scripts/dev/testkit-tools.sh" "$phase" \
        > "$fixture/failed-$phase.stdout" 2> "$fixture/failed-$phase.stderr" || status=$?
    if [[ "$phase" == install ]]; then expected=39; else expected=33; fi
    [[ "$status" == "$expected" && ! -s "$fixture/failed-$phase.stdout" ]] || exit 1
done
status=0
TESTKIT_FIXTURE_INSTALL_STATUS=23 "$BASH" "$consumer/scripts/dev/testkit-tools.sh" install \
    > "$fixture/failed-cli.stdout" 2> "$fixture/failed-cli.stderr" || status=$?
[[ "$status" == 23 && ! -s "$fixture/failed-cli.stdout" ]] || exit 1
[[ "$(tail -n 1 "$TESTKIT_FIXTURE_LOG")" == 'cli install' ]] || exit 1

# A changed locked selection cannot report the former selection as success.
for phase in install check; do
    for ready in 0 1; do
        status=0
        TESTKIT_FIXTURE_CHANGE_SELECTION=1 TESTKIT_FIXTURE_CHANGED_CLI_READY="$ready" \
            "$BASH" "$consumer/scripts/dev/testkit-tools.sh" "$phase" \
            > "$fixture/changed-$phase-$ready.stdout" 2> "$fixture/changed-$phase-$ready.stderr" || status=$?
        [[ "$status" != 0 && ! -s "$fixture/changed-$phase-$ready.stdout" ]] || exit 1
        if [[ "$ready" == 0 ]]; then
            [[ "$status" == 27 ]] || exit 1
            rg -q 'locked Testkit CLI could not be re-admitted' "$fixture/changed-$phase-$ready.stderr"
        else
            [[ "$status" == 1 ]] || exit 1
            rg -q 'locked Testkit selection changed' "$fixture/changed-$phase-$ready.stderr"
        fi
        [[ -f "$consumer/.tools/testkit-server/admitted" ]] || exit 1
        cp "$fixture/selected.lock" "$consumer/Cargo.lock"
    done
done

# Ambiguous or non-registry CLI identity refuses before any tool effect.
# Exercise the canonical reader, rather than teaching the substitute to parse.
cp scripts/dev/install-rust-tools.sh "$consumer/scripts/dev/install-rust-tools.sh"
for input in scripts/dev/cargo-tool-build.sh scripts/dev/tool-store.sh scripts/ci/verify-file-checksum.sh; do
    cp -p "$input" "$consumer/$input"
done
# Prepare substituted registry source without installing any executable. The
# offline check must reach missing selection admission, not an absent cache.
bash scripts/ci/prepare-cargo-failure-fixture.sh "$fixture/source-compiler"
mkdir -m 700 "$fixture/canonical-store" "$fixture/canonical-store/cargo-cache"
(umask 077; CARGO_HOME="$fixture/canonical-store/cargo-cache" "$fixture/source-compiler/bin/cargo" \
    info ic-testkit@0.25.5 --registry crates-io)
# A valid lock reaches canonical offline admission and names the exact missing
# CLI, without invoking any server or Cargo installation effect.
cp "$TESTKIT_FIXTURE_LOG" "$fixture/before-canonical-check.log"
status=0
SHARED_TOOLING_STORE="$fixture/canonical-store" "$BASH" "$consumer/scripts/dev/testkit-tools.sh" check \
    > "$fixture/canonical-missing.stdout" 2> "$fixture/canonical-missing.stderr" || status=$?
[[ "$status" == 1 && ! -s "$fixture/canonical-missing.stdout" ]] || exit 1
rg -F 'missing or invalid shared Cargo tool: ic-testkit 0.25.5 (bin:ic-testkit-server, debug)' "$fixture/canonical-missing.stderr" > /dev/null
rg -F 'make install-testkit-tools' "$fixture/canonical-missing.stderr" > /dev/null
cmp "$TESTKIT_FIXTURE_LOG" "$fixture/before-canonical-check.log"
for kind in duplicate nonregistry; do
    cp "$fixture/selected.lock" "$consumer/Cargo.lock"
    if [[ "$kind" == duplicate ]]; then
        sed '1d' "$fixture/selected.lock" >> "$consumer/Cargo.lock"
    else
        sed 's|registry+https://github.com/rust-lang/crates.io-index|git+https://example.invalid/testkit|' \
            "$fixture/selected.lock" > "$consumer/Cargo.lock"
    fi
    cp "$TESTKIT_FIXTURE_LOG" "$fixture/before-$kind.log"
    status=0
    "$BASH" "$consumer/scripts/dev/testkit-tools.sh" check \
        > "$fixture/$kind.stdout" 2> "$fixture/$kind.stderr" || status=$?
    [[ "$status" != 0 && ! -s "$fixture/$kind.stdout" ]] || exit 1
    cmp "$TESTKIT_FIXTURE_LOG" "$fixture/before-$kind.log"
done
cp "$fixture/selected.lock" "$consumer/Cargo.lock"
cp "$fixture/mock-installer.sh" "$consumer/scripts/dev/install-rust-tools.sh"

# The real qualification caller consumes only Testkit's check-returned path.
POCKET_IC_BIN=/unadmitted/override "$BASH" "$consumer/scripts/dev/test-qualification.sh" \
    > "$fixture/qualification.stdout" 2> "$fixture/qualification.stderr" || {
    status=$?
    printf 'Testkit qualification caller failed (exit %s); stderr: %s\n' \
        "$status" "$fixture/qualification.stderr" >&2
    cat "$fixture/qualification.stderr" >&2
    exit "$status"
}
[[ "$(tail -n 7 "$TESTKIT_FIXTURE_LOG")" == $'cli check\nserver check\ncli check\nclient tools check\nclient contracts check\nclient build\nqualified admitted server' ]] || exit 1
[[ "$(cat "$consumer/.tools/ic/bin/pocket-ic")" == 'retained former bundle' ]] || exit 1

# Client admission, contracts and build failures stop before native dispatch,
# without setup; contract refusal must also stop before the client build.
for phase in check contracts build; do
    before="$(wc -l < "$consumer/cargo-dispatch.log" | tr -d ' ')"
    : > "$TESTKIT_FIXTURE_LOG"
    status=0
    if [[ "$phase" == check ]]; then
        TESTKIT_FIXTURE_CLIENT_STATUS=31 "$BASH" "$consumer/scripts/dev/test-qualification.sh" \
            > "$fixture/client-$phase.stdout" 2> "$fixture/client-$phase.stderr" || status=$?
        [[ "$status" == 31 ]] || exit 1
    elif [[ "$phase" == contracts ]]; then
        TESTKIT_FIXTURE_CONTRACTS_STATUS=34 "$BASH" "$consumer/scripts/dev/test-qualification.sh" \
            > "$fixture/client-$phase.stdout" 2> "$fixture/client-$phase.stderr" || status=$?
        [[ "$status" == 34 ]] || exit 1
    else
        TESTKIT_FIXTURE_CLIENT_BUILD_STATUS=32 "$BASH" "$consumer/scripts/dev/test-qualification.sh" \
            > "$fixture/client-$phase.stdout" 2> "$fixture/client-$phase.stderr" || status=$?
        [[ "$status" == 32 ]] || exit 1
    fi
    [[ "$(wc -l < "$consumer/cargo-dispatch.log" | tr -d ' ')" == "$before" ]] || exit 1
    printf '%s\n' 'cli check' 'server check' 'cli check' 'client tools check' \
        > "$fixture/expected-client-$phase.log"
    if [[ "$phase" != check ]]; then
        printf 'client contracts check\n' >> "$fixture/expected-client-$phase.log"
    fi
    if [[ "$phase" == build ]]; then
        printf 'client build\n' >> "$fixture/expected-client-$phase.log"
    fi
    cmp "$fixture/expected-client-$phase.log" "$TESTKIT_FIXTURE_LOG"
done

# Execute the actual CI extension of the shared collector, with opaque owner
# failure bytes. The shared archiver preserves that evidence without new policy.
mkdir -p "$consumer/scripts/ci"
cp scripts/ci/archive-evidence.sh "$consumer/scripts/ci/"
printf 'retained Testkit failure bytes\000\377\n' > "$consumer/.tools/testkit-server/setup.log"
yq -p yaml -o json '.' .github/workflows/ci.yml | jq -er '
    .jobs.rust.steps[] | select(.name == "Archive retained Testkit setup evidence") | .run
' > "$fixture/archive-caller.sh"
(cd "$consumer"; "$BASH" "$fixture/archive-caller.sh") > "$fixture/archive.stdout"
archive="$(rg --files "$consumer/target/portable-fixtures" | rg '/server-evidence\.tar\.gz$')"
mkdir "$fixture/unpacked"
tar -xzf "$archive" -C "$fixture/unpacked"
cmp "$consumer/.tools/testkit-server/setup.log" "$fixture/unpacked/.tools/testkit-server/setup.log"
if rg 'warning: (overriding recipe|ignoring old recipe)' "$fixture"/*.log; then exit 1; fi
printf 'Testkit admission, fail-fast native/Wasm CI lint, input seals and qualification transport verified\n'
