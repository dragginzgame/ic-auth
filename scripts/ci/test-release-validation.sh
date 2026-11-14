#!/usr/bin/env bash
# Auth's actual selector/runner/receipt path, with substituted stage bodies.
set -euo pipefail
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
root="$(pwd -P)"
export PATH="$root/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
mkdir -p target/portable-fixtures
evidence="$(mktemp -d "$root/target/portable-fixtures/release-validation.XXXXXX")"
printf 'Auth release-plan evidence retained: %s\n' "$evidence"
consumer="$evidence/consumer with spaces"
mkdir "$consumer"
bash scripts/release/export-fixture-source.sh "$root" "$consumer"
cd "$consumer"
export CARGO_TARGET_DIR="$PWD/target"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
git init -q
git config user.name 'Auth qualification'
git config user.email 'auth-fixture@example.invalid'
cat > Makefile <<'MAKE'
export CARGO_TARGET_DIR := $(CURDIR)/target
.PHONY: ci release-stage-force
ci:
	+bash scripts/ci/run-auth-ci.sh
release-stage-force:
release-stage-%: release-stage-force
	+bash scripts/release/validation-stage.sh "$*"
MAKE
while IFS= read -r stage; do
    printf '\n.PHONY: %s\n%s:\n\t@echo %s >> target/executions\n' "$stage" "$stage" "$stage" >> Makefile
    cat >> Makefile <<'MAKE'
	@test "$@" != "$${FAIL_TARGET:-}"
MAKE
    if [[ "$stage" == check-snapshot ]]; then
        cat >> Makefile <<'MAKE'
	@if test "$${DRIFT_INPUT:-}" = true; then echo drift >> Cargo.toml; fi
MAKE
    fi
done < <(jq -r '.stages[] | ltrimstr("release-stage-")' ci/release-validation.json)
git add .
git commit -qm 'Fixed Auth qualification baseline'
baseline=$(git rev-parse HEAD)
git tag -am 'Protocol baseline' protocol-v0.32.0
git tag -am 'Engine baseline' engine-v0.32.0
export RELEASE_PREVIOUS=0.32.0
mkdir -p target
run() {
    local name=$1; shift
    bash scripts/release/validation.sh "target/$name" > "$evidence/$name.log" 2>&1 || {
        local status=$?
        tail -35 "$evidence/$name.log" >&2
        return "$status"
    }
}
for group in protocol engine; do
    git reset --hard -q "$baseline"
    if [[ $group == protocol ]]; then path=crates/ic-auth-protocol-types/src/lib.rs
    else path=crates/ic-auth/src/lib.rs; fi
    printf '\n// Owner-only qualification change.\n' >> "$path"
    git add "$path"; git commit -qm "$group source change"
    RELEASE_SOURCE=$(git rev-parse HEAD)
    export RELEASE_SOURCE RELEASE_GROUP=$group
    run "$group"
    jq -e '.scope == "group" and any(.stages[]; .required == false)' "target/$group/plan.json" >/dev/null
    perl scripts/release/validation-record.pl check "target/$group" > "$evidence/$group-receipt.json"
    perl scripts/release/validation-plan.pl "$RELEASE_SOURCE" "$group" "$RELEASE_PREVIOUS" "target/$group" --check > /dev/null
    jq -cnS --slurpfile evidence "target/$group/evidence.json" \
        '{schema:4,gate:"release-plan",validation:$evidence[0]}' > "target/$group-record.json"
    perl scripts/release/validation-record.pl check-record "target/$group-record.json" > /dev/null
    jq '.validation.proof.scope = "foreign"' "target/$group-record.json" > "target/$group-corrupt.json"
    if perl scripts/release/validation-record.pl check-record "target/$group-corrupt.json" > "$evidence/$group-corrupt-record.log" 2>&1; then exit 1; fi
    jq '.schema = 3' "target/$group-record.json" > "target/$group-old.json"
    if perl scripts/release/validation-record.pl check-record "target/$group-old.json" > "$evidence/$group-old-schema.log" 2>&1; then exit 1; fi
    perl scripts/release/validation-plan.pl "$RELEASE_SOURCE" "$group" "$RELEASE_PREVIOUS" "target/$group" \
        --check --source-root "$PWD" --record "target/$group-record.json" > /dev/null
done
# A baseline's annotation identity is part of the retained selection.
saved_tag=$(git rev-parse refs/tags/engine-v0.32.0)
git tag -f -am 'Changed annotation' engine-v0.32.0 "$baseline" > /dev/null
if perl scripts/release/validation-plan.pl "$RELEASE_SOURCE" engine "$RELEASE_PREVIOUS" target/engine --check > "$evidence/changed-baseline.log" 2>&1; then exit 1; fi
git update-ref refs/tags/engine-v0.32.0 "$saved_tag"
# Required failure blocks, but an unrelated release-fixture failure is excluded.
export RELEASE_GROUP=engine
# test-protocol executes the Engine's default-feature tests despite its name.
jq -e 'any(.stages[]; .target == "release-stage-test-protocol" and .required)' target/engine/plan.json >/dev/null
if FAIL_TARGET=test-protocol run engine-default-failure; then exit 1; fi
[[ ! -e target/engine-default-failure/evidence.json ]]
if FAIL_TARGET=test-sessions run required-failure; then exit 1; fi
[[ ! -e target/required-failure/evidence.json ]]
FAIL_TARGET=test-release-tools run excluded-failure
FAIL_TARGET=test-release-validation run excluded-selector-failure
if DRIFT_INPUT=true run input-drift; then exit 1; fi
[[ ! -e target/input-drift/evidence.json ]]
git checkout -- Cargo.toml
# Original successful proof bytes are re-admitted, never synthesized from PASS alone.
run_path=(target/engine/logs/validation.*)
cp "${run_path[0]}/timings.tsv" target/timings.saved
printf 'foreign\tPASS\t0\tmissing.log\n' >> "${run_path[0]}/timings.tsv"
if perl scripts/release/validation-record.pl check target/engine > "$evidence/corrupt-receipt.log" 2>&1; then exit 1; fi
cp target/timings.saved "${run_path[0]}/timings.tsv"
log=$(awk -F '\t' 'NR==2 {print $4}' "${run_path[0]}/timings.tsv")
cp "$log" target/log.saved; echo drift >> "$log"
if perl scripts/release/validation-record.pl check target/engine > "$evidence/corrupt-log.log" 2>&1; then exit 1; fi
if perl scripts/release/validation-record.pl check-retained target/engine > "$evidence/corrupt-retained.log" 2>&1; then exit 1; fi
cp target/engine/evidence.json target/record.saved
source vendor/toko-tooling/scripts/ci/release-receipts.sh
# A changed log is a different valid observation when freshly collected. An
# invalid execution must fail collection and preserve the previous receipt.
printf 'foreign\tPASS\t0\tmissing.log\n' >> "${run_path[0]}/timings.tsv"
if release_receipt_write target/engine/evidence.json perl scripts/release/validation-record.pl collect target/engine > "$evidence/failed-write.log" 2>&1; then exit 1; fi
cmp target/record.saved target/engine/evidence.json
cp target/timings.saved "${run_path[0]}/timings.tsv"
cp target/log.saved "$log"
perl scripts/release/validation-record.pl check target/engine > /dev/null
printf '# shared lock change\n' >> Cargo.lock
git add Cargo.lock; git commit -qm 'Shared dependency input change'
RELEASE_SOURCE=$(git rev-parse HEAD)
export RELEASE_SOURCE
run shared
# Recovery admits the saved source, then retained bytes; a changed current lock
# must not be treated as a new execution or make old proof impossible to inspect.
original=$(jq -er .validation.source target/engine-record.json)
mkdir target/original-engine
git archive "$original" | tar -xf - -C target/original-engine
perl scripts/release/validation-plan.pl "$original" engine "$RELEASE_PREVIOUS" target/engine \
    --check --source-root "$PWD/target/original-engine" > /dev/null
perl scripts/release/validation-record.pl check-retained target/engine > /dev/null
if perl scripts/release/validation-record.pl check target/engine > "$evidence/changed-live-inputs.log" 2>&1; then exit 1; fi
jq -e '.scope == "group" and all(.stages[]; .required == (.target != "release-stage-test-release-validation"))' target/shared/plan.json >/dev/null
if perl scripts/release/validation-plan.pl "$RELEASE_SOURCE" engine "$RELEASE_PREVIOUS" target/engine --check > "$evidence/stale.log" 2>&1; then exit 1; fi
if RELEASE_GROUP=unknown bash scripts/release/validation.sh target/unknown > "$evidence/unknown.log" 2>&1; then exit 1; fi
# Every ordinary CI obligation executes; no release exclusions leak into CI.
rm -f target/executions
VALIDATION_LOG_DIR=target/full-ci make --no-print-directory ci > "$evidence/ci.log" 2>&1
jq -r '.stages[] | ltrimstr("release-stage-")' ci/release-validation.json > target/expected
cmp target/expected target/executions
git commit --allow-empty -qm 'Different HEAD after full CI'
if env -u CARGO_TARGET_DIR VALIDATION_LOG_DIR=target/full-ci bash scripts/release/qualify-validation.sh > "$evidence/foreign-ci-source.log" 2>&1; then exit 1; fi
git reset --hard -q "$RELEASE_SOURCE"
env -u CARGO_TARGET_DIR VALIDATION_LOG_DIR=target/full-ci bash scripts/release/qualify-validation.sh > "$evidence/comparison.log" 2>&1
# Roster drift must refuse at Auth's actual ordinary-CI caller boundary.
cp ci/release-validation.json target/roster.saved
jq -cS '.groups = ["engine"]' target/roster.saved > ci/release-validation.json
if VALIDATION_LOG_DIR=target/roster-ci make --no-print-directory ci > "$evidence/roster-drift.log" 2>&1; then exit 1; fi
rg -F 'Cargo and validation groups differ' "$evidence/roster-drift.log" > /dev/null
cp target/roster.saved ci/release-validation.json
[[ -z $(git status --porcelain=v1 --untracked-files=all) ]]
# End roster drift case.
# Relocated private host/runner source retains Engine ownership. Selecting a
# plan here exercises the real classifier without replaying stage bodies.
for path in qualification/ic-auth-qualification-canister/src/lib.rs qualification/ic-auth-qualification/tests/session_metrics.rs; do
    git reset --hard -q "$baseline"
    printf '\n// Relocated Engine-owned source.\n' >> "$path"
    git add "$path"; git commit -qm 'Relocated Engine owner change'
    selected=$(git rev-parse HEAD)
    directory="target/relocated-${path#crates/}"
    mkdir -p "$directory"
    perl scripts/release/validation-plan.pl "$selected" engine "$RELEASE_PREVIOUS" "$directory"
    jq -e '.group == "engine" and .scope == "group" and any(.stages[]; .required == false)' "$directory/plan.json" >/dev/null
done
# Data-only changes retain the complete product/tool/native/archive roster;
# only the substituted selector regression is unrelated. Actual infrastructure,
# manifests, unknown paths and explicit full qualification retain every stage.
for path in Cargo.lock qualification/Cargo.lock README.md docs/integration/persistent-auth.md packages/client/src/generated/wire-runtime.ts \
    Cargo.toml qualification/Cargo.toml scripts/release/validation-stage.sh ci/release-validation-coverage.json unknown-input; do
    git reset --hard -q "$baseline"
    case "$path" in
        *.json) printf '\n' >> "$path" ;;
        *) printf '\n# Selector applicability fixture\n' >> "$path" ;;
    esac
    git add "$path"; git commit -qm 'Selector applicability change'
    selected=$(git rev-parse HEAD)
    directory="target/applicability-$path"
    mkdir -p "$directory"
    # Coverage JSON is itself canonical input; invalid bytes must refuse.
    if [[ "$path" == ci/release-validation-coverage.json ]]; then
        if perl scripts/release/validation-plan.pl "$selected" engine "$RELEASE_PREVIOUS" "$directory" > "$evidence/invalid-coverage.log" 2>&1; then exit 1; fi
        continue
    fi
    perl scripts/release/validation-plan.pl "$selected" engine "$RELEASE_PREVIOUS" "$directory"
    case "$path" in
        Cargo.lock|qualification/Cargo.lock|README.md|docs/*|packages/*)
            jq -e '.scope == "group" and all(.stages[]; .required == (.target != "release-stage-test-release-validation"))' "$directory/plan.json" >/dev/null ;;
        *) jq -e '.scope == "workspace" and all(.stages[]; .required)' "$directory/plan.json" >/dev/null ;;
    esac
done
git reset --hard -q "$baseline"
mkdir target/forced-full target/missing-baseline
perl scripts/release/validation-plan.pl "$baseline" engine "$RELEASE_PREVIOUS" target/forced-full --workspace
perl scripts/release/validation-plan.pl "$baseline" engine 99.99.99 target/missing-baseline
for directory in target/forced-full target/missing-baseline; do
    jq -e '.scope == "workspace" and all(.stages[]; .required)' "$directory/plan.json" >/dev/null
done
printf 'Both groups, shared expansion, required/excluded failures, stale selection, retained evidence and complete CI dispatch passed (substituted stage bodies)\n'
