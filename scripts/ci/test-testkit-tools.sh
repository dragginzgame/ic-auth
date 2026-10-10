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
export PATH="$root/.tools/host/bin:$fixture/bin:$PATH"
printf 'synthetic server bytes\n' > "$TESTKIT_FIXTURE_SERVER"
cat > "$consumer/scripts/dev/install-rust-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ $# == 5 && "$1" == --consumer && "$2" == "$TESTKIT_FIXTURE_CONSUMER" &&
      "$3" == --versions && "$4" == "$TESTKIT_FIXTURE_CONSUMER/ci/tool-versions.env" && "$5" == --preflight ]]; then
    printf 'rust preflight\n' >> "$TESTKIT_FIXTURE_LOG"
    exit "${TESTKIT_FIXTURE_RUST_PREFLIGHT_STATUS:-0}"
fi
[[ $# == 10 || $# == 11 ]]
[[ "$1" == --consumer && "$2" == "$TESTKIT_FIXTURE_CONSUMER" &&
   "$3" == --package && "$4" == ic-testkit && "$5" == --version &&
   "$6" == 0.25.5 && "$7" == --bin && "$8" == ic-testkit-server &&
   "$9" == --profile && "${10}" == debug ]]
if [[ $# == 11 ]]; then
    [[ "${11}" == --check ]]
    printf 'cli check\n' >> "$TESTKIT_FIXTURE_LOG"
    [[ -f "$TESTKIT_FIXTURE_READY" ]] || exit 27
else
    printf 'cli install\n' >> "$TESTKIT_FIXTURE_LOG"
    [[ "${TESTKIT_FIXTURE_INSTALL_STATUS:-0}" == 0 ]] || exit "$TESTKIT_FIXTURE_INSTALL_STATUS"
    touch "$TESTKIT_FIXTURE_READY"
fi
printf '%s\n' "$TESTKIT_FIXTURE_CLI"
SCRIPT
cat > "$consumer/scripts/dev/install-ic-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 5 && "$1" == --consumer && "$2" == "$TESTKIT_FIXTURE_CONSUMER" &&
   "$3" == --pins && "$4" == "$TESTKIT_FIXTURE_CONSUMER/ci/ic-tools.tsv" && "$5" == --preflight ]]
printf 'ic preflight\n' >> "$TESTKIT_FIXTURE_LOG"
exit "${TESTKIT_FIXTURE_IC_PREFLIGHT_STATUS:-0}"
SCRIPT
cat > "$TESTKIT_FIXTURE_CLI" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 3 && "$2" == --directory && "$3" == "$TESTKIT_FIXTURE_CONSUMER/.tools/testkit-server" ]]
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
[[ "$POCKET_IC_BIN" == "$TESTKIT_FIXTURE_SERVER" ]]
[[ "$IC_AUTH_QUALIFICATION_WASM" == "$TESTKIT_FIXTURE_CONSUMER/target/wasm32-unknown-unknown/release/ic_auth_qualification_canister.wasm" ]]
[[ -d "$IC_AUTH_QUALIFICATION_STATE_ROOT" && "$TMPDIR" == "$IC_AUTH_QUALIFICATION_STATE_ROOT" ]]
[[ "$*" == 'test --locked --offline -p ic-auth-qualification --test signatures -- --nocapture' ]]
printf 'qualified admitted server\n' >> "$TESTKIT_FIXTURE_LOG"
SCRIPT
chmod +x "$TESTKIT_FIXTURE_CLI" "$fixture/bin/cargo"
ln -s "$BASH" "$fixture/bin/bash"

# Missing selected CLI refuses even when an old shared executable is retained.
status=0
"$BASH" "$consumer/scripts/dev/testkit-tools.sh" check > "$fixture/missing.stdout" 2> "$fixture/missing.stderr" || status=$?
[[ "$status" == 27 && ! -s "$fixture/missing.stdout" ]]
[[ "$(cat "$TESTKIT_FIXTURE_LOG")" == 'cli check' ]]
rg -q 'ic-testkit-server.*0[.]25[.]5' "$fixture/missing.stderr"
rg -F 'make install-testkit-tools' "$fixture/missing.stderr" > /dev/null

# Actual Make entry points refuse before Wasm compilation or any later CI lane,
# including parallel Make. Substitute the already-prepared common bundle only;
# Testkit admission and the validation runner remain their actual callers.
mkdir -p "$consumer/make" "$consumer/ci" "$consumer/scripts/ci"
for input in Makefile make/tools.mk make/release.mk make/rust-format.mk make/execution.mk \
    ci/tool-versions.env scripts/ci/check-make-execution.sh scripts/ci/run-validation-targets.sh; do
    cp -p "$input" "$consumer/$input"
done
cat >> "$consumer/Makefile" <<'MAKE'

install-host-tools install-ic-tools install-rust-tools install-msrv host-tools-check ic-tools-check rust-tools-check:
	@printf '%s\n' '$@' >> "$(TESTKIT_FIXTURE_LOG)"
msrv-tools-check:
	@printf '%s\n' '$@' >> "$(TESTKIT_FIXTURE_LOG)"
	@exit "$${TESTKIT_FIXTURE_MSRV_STATUS:-0}"
check-snapshot:
	@printf 'snapshot check\n' >> "$(TESTKIT_FIXTURE_LOG)"
MAKE
for target in test-qualification ci; do
    : > "$TESTKIT_FIXTURE_LOG"
    status=0
    (cd "$consumer"; make --no-print-directory -j4 "$target") \
        > "$fixture/early-$target.log" 2>&1 || status=$?
    [[ "$status" == 2 && ! -e "$consumer/cargo-dispatch.log" ]]
    [[ "$(tail -n 1 "$TESTKIT_FIXTURE_LOG")" == 'cli check' ]]
    rg -F 'make install-testkit-tools' "$fixture/early-$target.log" > /dev/null
    if [[ "$target" == ci ]]; then
        printf '%s\n' 'snapshot check' host-tools-check ic-tools-check rust-tools-check \
            msrv-tools-check 'cli check' > "$fixture/expected-early.log"
        cmp "$fixture/expected-early.log" "$TESTKIT_FIXTURE_LOG"
    fi
done

# Real consumer aggregates retain common/product ordering under parallel Make.
# Only explicit setup dispatches the selected CLI installer and server setup;
# individual product setup targets still admit their host prerequisite.
for failed in ic rust; do
    : > "$TESTKIT_FIXTURE_LOG"
    status=0
    if [[ "$failed" == ic ]]; then
        (cd "$consumer"; TESTKIT_FIXTURE_IC_PREFLIGHT_STATUS=37 make --no-print-directory -j4 install-tools) \
            > "$fixture/preflight-$failed.log" 2>&1 || status=$?
        printf 'ic preflight\n' > "$fixture/expected-preflight.log"
    else
        (cd "$consumer"; TESTKIT_FIXTURE_RUST_PREFLIGHT_STATUS=39 make --no-print-directory -j4 install-tools) \
            > "$fixture/preflight-$failed.log" 2>&1 || status=$?
        printf 'ic preflight\nrust preflight\n' > "$fixture/expected-preflight.log"
    fi
    [[ "$status" == 2 && ! -e "$TESTKIT_FIXTURE_READY" && ! -e "$consumer/cargo-dispatch.log" ]]
    cmp "$fixture/expected-preflight.log" "$TESTKIT_FIXTURE_LOG"
done
: > "$TESTKIT_FIXTURE_LOG"
(cd "$consumer"; make --no-print-directory -j4 install-tools) > "$fixture/setup.log" 2>&1
printf '%s\n' 'ic preflight' 'rust preflight' install-host-tools install-ic-tools install-rust-tools \
    install-host-tools install-msrv install-host-tools 'cli install' 'server setup' > "$fixture/expected-setup.log"
cmp "$fixture/expected-setup.log" "$TESTKIT_FIXTURE_LOG"
: > "$TESTKIT_FIXTURE_LOG"
(cd "$consumer"; make --no-print-directory -j4 tools-check) > "$fixture/check.log" 2>&1
printf '%s\n' host-tools-check ic-tools-check rust-tools-check msrv-tools-check \
    'cli check' 'server check' > "$fixture/expected-check.log"
cmp "$fixture/expected-check.log" "$TESTKIT_FIXTURE_LOG"

# Missing minimum compiler refuses before Testkit, without installation.
: > "$TESTKIT_FIXTURE_LOG"
status=0
(cd "$consumer"; TESTKIT_FIXTURE_MSRV_STATUS=31 make --no-print-directory -j4 tools-check) \
    > "$fixture/missing-msrv.log" 2>&1 || status=$?
[[ "$status" == 2 ]]
printf '%s\n' host-tools-check ic-tools-check rust-tools-check msrv-tools-check > "$fixture/expected-msrv.log"
cmp "$fixture/expected-msrv.log" "$TESTKIT_FIXTURE_LOG"

# Original setup/installation/admission failures cannot report an identity.
for phase in install check; do
    status=0
    TESTKIT_FIXTURE_SETUP_STATUS=39 TESTKIT_FIXTURE_CHECK_STATUS=33 \
        "$BASH" "$consumer/scripts/dev/testkit-tools.sh" "$phase" \
        > "$fixture/failed-$phase.stdout" 2> "$fixture/failed-$phase.stderr" || status=$?
    if [[ "$phase" == install ]]; then expected=39; else expected=33; fi
    [[ "$status" == "$expected" && ! -s "$fixture/failed-$phase.stdout" ]]
done
status=0
TESTKIT_FIXTURE_INSTALL_STATUS=23 "$BASH" "$consumer/scripts/dev/testkit-tools.sh" install \
    > "$fixture/failed-cli.stdout" 2> "$fixture/failed-cli.stderr" || status=$?
[[ "$status" == 23 && ! -s "$fixture/failed-cli.stdout" ]]
[[ "$(tail -n 1 "$TESTKIT_FIXTURE_LOG")" == 'cli install' ]]

# A changed locked selection cannot report the former selection as success.
for phase in install check; do
    status=0
    TESTKIT_FIXTURE_CHANGE_SELECTION=1 "$BASH" "$consumer/scripts/dev/testkit-tools.sh" "$phase" \
        > "$fixture/changed-$phase.stdout" 2> "$fixture/changed-$phase.stderr" || status=$?
    [[ "$status" != 0 && ! -s "$fixture/changed-$phase.stdout" ]]
    rg -q 'locked Testkit selection changed' "$fixture/changed-$phase.stderr"
    [[ -f "$consumer/.tools/testkit-server/admitted" ]]
    cp "$fixture/selected.lock" "$consumer/Cargo.lock"
done

# Ambiguous or non-registry CLI identity refuses before any tool effect.
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
    [[ "$status" != 0 && ! -s "$fixture/$kind.stdout" ]]
    cmp "$TESTKIT_FIXTURE_LOG" "$fixture/before-$kind.log"
done
cp "$fixture/selected.lock" "$consumer/Cargo.lock"

# The real qualification caller consumes only Testkit's check-returned path.
POCKET_IC_BIN=/unadmitted/override "$BASH" "$consumer/scripts/dev/test-qualification.sh" \
    > "$fixture/qualification.stdout" 2> "$fixture/qualification.stderr"
[[ "$(tail -n 3 "$TESTKIT_FIXTURE_LOG")" == $'cli check\nserver check\nqualified admitted server' ]]
[[ "$(cat "$consumer/.tools/ic/bin/pocket-ic")" == 'retained former bundle' ]]

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
printf 'Testkit explicit setup, offline admission, failure propagation, qualification path and retained CI bytes verified\n'
