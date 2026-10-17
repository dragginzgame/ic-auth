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
bash scripts/ci/test-artifact-discovery.sh
consumer="$fixture/consumer with spaces"
mkdir -p "$consumer/scripts/dev" "$consumer/scripts/ci" "$consumer/make" "$consumer/ci" "$fixture/bin" "$fixture/runner"
# Copy every include loaded by the actual Makefile so injected failures reach
# the selected tool targets rather than stopping during Makefile parsing.
for input in Makefile make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk \
    scripts/ci/check-release-delivery.sh ci/release-delivery.txt \
    scripts/ci/check-make-execution.sh scripts/ci/run-formatting.sh ci/tool-versions.env \
    scripts/dev/ci-tools.sh scripts/dev/install-rust-tools.sh \
    scripts/dev/cargo-tool-build.sh scripts/dev/tool-store.sh \
    scripts/ci/verify-file-checksum.sh scripts/ci/tool-catalog-records.pl; do
    cp -p "$input" "$consumer/$input"
done
# Retain the actual ordered aggregate and Rust preflight/installer. Earlier
# common sets are inert; a later Rust refusal must never dispatch product tools.
for script in install-host-tools install-ic-tools; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "$consumer/scripts/dev/$script.sh"
done
for script in msrv-tools testkit-tools; do
    cat > "$consumer/scripts/dev/$script.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
printf 'unexpected product dispatch\n' >> target/product-dispatch.log
exit 99
SCRIPT
done
# The canonical substitute provides a compiler-local Cargo boundary compatible
# with the standard cleared build environment. It never contacts a registry.
bash scripts/ci/prepare-cargo-failure-fixture.sh "$fixture/compiler"
export SHARED_TOOLING_STORE="$fixture/store"
ln -s "$BASH" "$fixture/bin/bash"
for phase in install check; do
    status=0
    PATH="$fixture/compiler/bin:$fixture/bin:$PATH" "$BASH" "$consumer/scripts/dev/ci-tools.sh" "$phase" \
        > "$fixture/$phase.stdout" 2>&1 || status=$?
    [[ "$status" == 2 ]]
    cmp "$fixture/$phase.stdout" "$consumer/target/rust-tools-$phase.log"
done
grep -F 'injected Cargo installation failure' "$consumer/target/rust-tools-install.log" > /dev/null
grep -F 'missing or invalid shared Cargo tool:' "$consumer/target/rust-tools-check.log" > /dev/null
set -- "$SHARED_TOOLING_STORE"/rust-*/set.*
[[ $# == 1 && -f "$1/build/failed-build.txt" ]] || exit 1
attempt="${1#"$SHARED_TOOLING_STORE/"}"
[[ -s "$1/install.log" && -s "$1/inputs.json" ]] || exit 1
[[ ! -e "$consumer/target/product-dispatch.log" ]]
if rg 'warning: (overriding recipe|ignoring old recipe)' "$fixture"/*.stdout; then exit 1; fi
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
# The selector retains the local Rust tree and the referenced failed shared
# attempt. An unrelated generation or redirected child must not be followed.
slot_evidence='.tools/rust/fixture-1.0.0-bin-fixture-release/installed/build/evidence.txt'
mkdir -p "$consumer/${slot_evidence%/*}" "$fixture/outside"
printf 'retained prior slot build evidence\n' > "$consumer/$slot_evidence"
printf 'must not upload\n' > "$fixture/outside/private"
ln -s "$fixture/outside" "$consumer/.tools/rust/redirected-slot"
mkdir -p "$SHARED_TOOLING_STORE/rust-unrelated/set.unrelated"
printf 'must not upload\n' > "$SHARED_TOOLING_STORE/rust-unrelated/set.unrelated/private"

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
    for path in build/failed-build.txt install.log inputs.json Cargo.lock source.crate native-environment.txt; do
        relative="$attempt/$path"
        digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$SHARED_TOOLING_STORE/$relative")"
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
[[ ! -e "$fixture/unpacked/rust-unrelated" ]] || exit 1
grep -Fx 'collection-complete=true' "$fixture/collector.outputs" > /dev/null
# Native CI collects through the ordinary action and verifies the exact-ID
# downloaded artifact against this oracle outside the upload selection.
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'consumer=%s\ntemp=%s\nexpected=%s\nstore=%s\n' "$consumer" "$consumer/target" "$fixture/expected.sha256" "$SHARED_TOOLING_STORE" >> "$GITHUB_OUTPUT"
fi
printf 'Rust setup/check failure status and collected bytes verified: %s\n' "$fixture"
