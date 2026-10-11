#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/tools-evidence.XXXXXX")"
printf 'Tool failure evidence retained: %s\n' "$fixture"
consumer="$fixture/consumer with spaces"
mkdir -p "$consumer/scripts/dev" "$consumer/scripts/ci" "$consumer/make" "$consumer/ci" "$fixture/bin" "$fixture/runner"
# Copy every include loaded by the actual Makefile so injected failures reach
# the selected tool targets rather than stopping during Makefile parsing.
for input in Makefile make/tools.mk make/release.mk make/rust-format.mk make/execution.mk \
    scripts/ci/check-make-execution.sh scripts/ci/run-formatting.sh ci/tool-versions.env \
    scripts/dev/ci-tools.sh scripts/dev/install-rust-tools.sh; do
    cp -p "$input" "$consumer/$input"
done
# The common aggregate now admits IC and Rust prerequisites before installing.
# Substitute IC admission, retaining the actual Rust preflight and installer.
cat > "$consumer/scripts/dev/install-ic-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 5 && "$1" == --consumer && "$3" == --pins && "$5" == --preflight ]]
SCRIPT
# Admit the earlier common sets without provisioning them in this failure
# fixture. Keep the actual aggregate, Rust installer and collector, so the
# injected later failure cannot dispatch product setup/check or lose evidence.
cat >> "$consumer/Makefile" <<'MAKE'

install-host-tools install-ic-tools host-tools-check ic-tools-check:
	@:
install-msrv install-testkit-tools msrv-tools-check testkit-tools-check:
	@printf 'unexpected product dispatch\n' >> target/product-dispatch.log
	@exit 99
MAKE
cat > "$fixture/bin/cargo" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ $# == 1 && "$1" == --version ]]; then
    [[ "$RUSTUP_AUTO_INSTALL" == 0 ]]
    printf 'cargo 1.99.0\n'
    exit 0
fi
[[ $# == 13 && "$1" == install && "$2" == cargo-sort && "$3" == --version &&
    "$5" == --locked && "$6" == --root && "$8" == --target-dir && "$9" == "$7/build" &&
    "${10}" == --bin && "${11}" == cargo-sort && "${12}" == --registry && "${13}" == crates-io ]] || exit 1
mkdir -p "$9"
printf 'retained failed Rust compilation\n' > "$9/failed-build.txt"
printf 'dispatch\n' >> "$9/dispatches.txt"
echo 'injected Cargo installation failure (23)' >&2
exit 23
SCRIPT
chmod +x "$fixture/bin/cargo"
ln -s "$BASH" "$fixture/bin/bash"
for phase in install check; do
    status=0
    PATH="$fixture/bin:$PATH" "$BASH" "$consumer/scripts/dev/ci-tools.sh" "$phase" \
        > "$fixture/$phase.stdout" 2>&1 || status=$?
    [[ "$status" == 2 ]]
    cmp "$fixture/$phase.stdout" "$consumer/target/rust-tools-$phase.log"
done
grep -F 'injected Cargo installation failure (23)' "$consumer/target/rust-tools-install.log" > /dev/null
grep -F 'missing or mismatched common Rust tool set' "$consumer/target/rust-tools-check.log" > /dev/null
set -- "$consumer"/.tools/rust/build/cargo-attempt.*
[[ $# == 1 && -f "$1/build/failed-build.txt" ]] || exit 1
attempt="${1##*/}"
[[ "$(cat "$1/build/dispatches.txt")" == dispatch ]] || exit 1
[[ ! -e "$consumer/target/product-dispatch.log" ]]
# Logging failure cannot mask a rejected aggregate, or turn a successful one
# into an unrecorded success. Keep these extra runs outside the retained oracle.
cp -R "$consumer" "$fixture/log-errors"
tee_binary="$(command -v tee)"
cat > "$fixture/bin/tee" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
"$TEST_TEE_BINARY" "$@"
exit 41
SCRIPT
chmod +x "$fixture/bin/tee"
status=0
TEST_TEE_BINARY="$tee_binary" PATH="$fixture/bin:$PATH" "$BASH" "$fixture/log-errors/scripts/dev/ci-tools.sh" check \
    > "$fixture/logging-and-check-failed.log" 2>&1 || status=$?
[[ "$status" == 2 ]]
cat > "$fixture/bin/make" <<'SCRIPT'
#!/usr/bin/env bash
printf 'substitute successful aggregate\n'
SCRIPT
chmod +x "$fixture/bin/make"
status=0
TEST_TEE_BINARY="$tee_binary" PATH="$fixture/bin:$PATH" "$BASH" "$fixture/log-errors/scripts/dev/ci-tools.sh" check \
    > "$fixture/logging-failed.log" 2>&1 || status=$?
[[ "$status" == 41 ]]
# The collector retains the complete physical Rust tree, including earlier
# slot build evidence. A redirected child is archived as a link, never followed.
slot_evidence='.tools/rust/fixture-1.0.0-bin-fixture-release/installed/build/evidence.txt'
mkdir -p "$consumer/${slot_evidence%/*}" "$fixture/outside"
printf 'retained prior slot build evidence\n' > "$consumer/$slot_evidence"
printf 'must not upload\n' > "$fixture/outside/private"
ln -s "$fixture/outside" "$consumer/.tools/rust/redirected-slot"

# Execute the adopted collector block; never maintain another selection.
yq -p yaml -o json '.' .github/actions/retain-failure-evidence/action.yml | \
    jq -er '.runs.steps[] | select(.id == "archive") | .run' > "$fixture/collector.sh"
EVIDENCE_TEMP_ROOT="$consumer/target" EVIDENCE_REPOSITORY_ROOT="$consumer" \
EVIDENCE_ACTION_ROOT="$root/.github/actions/retain-failure-evidence" EVIDENCE_COMPACT=false \
EVIDENCE_HOST_VERSIONS='' EVIDENCE_IC_PINS='' RUNNER_TEMP="$fixture/runner" \
GITHUB_OUTPUT="$fixture/collector.outputs" "$BASH" "$fixture/collector.sh" > "$fixture/collector.log" 2>&1
archive="$(sed -n 's/^path=//p' "$fixture/collector.outputs")"
[[ -f "$archive" ]]
{
    for path in rust-tools-install.log rust-tools-check.log; do
        digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$consumer/target/$path")"
        printf '%s  %s\n' "$digest" "$path"
    done
    for path in failed-build.txt dispatches.txt; do
        relative=".tools/rust/build/$attempt/build/$path"
        digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$consumer/$relative")"
        printf '%s  %s\n' "$digest" "$relative"
    done
    digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$consumer/$slot_evidence")"
    printf '%s  %s\n' "$digest" "$slot_evidence"
} > "$fixture/expected.sha256"
mkdir "$fixture/unpacked"
tar -xzf "$archive" -C "$fixture/unpacked"
(cd "$fixture/unpacked"; bash "$root/scripts/ci/verify-evidence-checksums.sh" "$fixture/expected.sha256")
[[ -L "$fixture/unpacked/.tools/rust/redirected-slot" &&
    "$(readlink "$fixture/unpacked/.tools/rust/redirected-slot")" == "$fixture/outside" ]] || exit 1
[[ ! -e "$fixture/unpacked/outside" ]] || exit 1
# Native CI collects through the ordinary action and verifies the exact-ID
# downloaded artifact against this oracle outside the upload selection.
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'consumer=%s\ntemp=%s\nexpected=%s\n' "$consumer" "$consumer/target" "$fixture/expected.sha256" >> "$GITHUB_OUTPUT"
fi
printf 'Rust setup/check failure status and collected bytes verified: %s\n' "$fixture"
