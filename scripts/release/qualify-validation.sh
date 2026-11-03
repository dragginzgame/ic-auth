#!/usr/bin/env bash
# Actual same-source mapping comparison after the full CI gate. No release receipt.
set -euo pipefail
# Match the root Makefile before recording the context used by its stage recipes.
export CARGO_TARGET_DIR="$PWD/target" RUSTUP_AUTO_INSTALL=0
bash scripts/ci/check-release-source.sh
source=$(git rev-parse HEAD)
mkdir -p target/release-validation
directory=$(mktemp -d target/release-validation/qualification.XXXXXX)
printf 'Auth group qualification retained: %s\n' "$PWD/$directory"
jq -r '.stages[] | ltrimstr("release-stage-")' ci/release-validation.json > "$directory/full-targets"
full=()
for run in "${VALIDATION_LOG_DIR:?full CI logs required}"/validation.*; do
    [[ -f "$run/timings.tsv" && ! -L "$run/timings.tsv" ]] || continue
    if awk -F '\t' 'NR>1 {if ($2 != "PASS") exit 1; print $1}' "$run/timings.tsv" > "$directory/observed-targets" &&
        cmp -s "$directory/full-targets" "$directory/observed-targets"; then
        full+=("$run")
    fi
done
[[ ${#full[@]} == 1 ]] || { echo 'exact complete passing CI run required' >&2; exit 1; }
mkdir "$directory/full-ci"
marker=$(jq -er 'keys | map(select(test("^target/validation-inputs/run\\.[^/]+/ci-source\\.json$"))) |
    if length == 1 then .[0] else error("one sealed CI source record required") end' "${full[0]}/inputs.json")
jq -e --arg source "$source" '.schema == 1 and .clean == true and .source == $source' "$marker" > /dev/null
digest=$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$marker")
jq -e --arg marker "$marker" --arg digest "$digest" '.[$marker] == $digest' "${full[0]}/inputs.json" > /dev/null
cp "$marker" "$directory/full-ci/source.json"
cp "${full[0]}/timings.tsv" "$directory/full-ci/timings.tsv"
cp "${full[0]}/inputs.json" "$directory/full-ci/inputs.json"
while IFS=$'\t' read -r target _result _seconds log; do
    [[ "$target" != target ]] || continue
    cp "$log" "$directory/full-ci/$target.log"
    digest=$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$directory/full-ci/$target.log")
    printf '%s\t%s.log\n' "$digest" "$target" >> "$directory/full-ci/logs.sha256"
done < "${full[0]}/timings.tsv"
export RELEASE_SOURCE=$source
for group in protocol engine; do
    export RELEASE_GROUP=$group
    RELEASE_PREVIOUS=$(bash scripts/ci/cargo-release-group.sh version "$group")
    export RELEASE_PREVIOUS
    gate="$directory/$group"
    mkdir -p "$gate/logs"
    # Zero-delta, fixed-source comparison exercises the candidate mapping; this
    # explicit qualification baseline never enters the release adapter.
    perl scripts/release/validation-plan.pl "$source" "$group" "$RELEASE_PREVIOUS" "$gate" --baseline "$source" > "$gate/selection.log"
    export AUTH_RELEASE_VALIDATION_DIRECTORY=$gate
    VALIDATION_LOG_DIR="$gate/logs" bash scripts/ci/run-auth-validation.sh --plan "$gate/plan.json"
    perl scripts/release/validation-plan.pl "$source" "$group" "$RELEASE_PREVIOUS" "$gate" --baseline "$source" --check > /dev/null
    source scripts/ci/release-receipts.sh
    release_receipt_write "$gate/evidence.json" perl scripts/release/validation-record.pl collect "$gate"
done
[[ $(git rev-parse HEAD) == "$source" ]]
bash scripts/ci/check-release-source.sh
jq -cnS --arg source "$source" --arg full "$directory/full-ci" --arg directory "$directory" \
    '{schema:1,kind:"auth-group-qualification",source:$source,full_ci:$full,protocol:($directory+"/protocol/evidence.json"),engine:($directory+"/engine/evidence.json")}' > "$directory/comparison.json"
