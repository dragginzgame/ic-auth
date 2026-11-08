#!/usr/bin/env bash
# Add every Cargo-admitted member manifest to Auth's validation input seal.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
bash scripts/ci/check-make-execution.sh
mkdir -p target/validation-inputs
evidence="$(mktemp -d "$PWD/target/validation-inputs/run.XXXXXX")"
checkpoint=scripts/ci/validation-input-checkpoint.pl
checksum=scripts/ci/verify-file-checksum.sh
ci_mode=false
if [[ "${1:-}" == --ci ]]; then ci_mode=true; shift; fi
discovery=(Cargo.toml Cargo.lock toko-tooling.lock ci/cargo-release-groups.json scripts/ci/run-auth-validation.sh
    scripts/ci/cargo-release-group.sh scripts/ci/cargo-release-groups.jq)
if [[ "$ci_mode" == true ]]; then
    discovery+=(ci/release-validation.json ci/cargo-release-groups.json scripts/ci/run-auth-ci.sh
        scripts/ci/validation-evidence.pl scripts/ci/ValidationPlan.pm)
fi
perl "$checkpoint" "$PWD" '' "$evidence/before.json" "$checksum" "${discovery[@]}"
# Shared's admitted plans own the complete Cargo inventory and permit unequal
# release-group versions. Auth only projects their manifest inputs.
jq -L scripts/ci -er 'include "cargo-release-groups"; release_policy |
    if all(.groups[]; .version == "package") then .groups[].name
    else error("Auth requires explicit package-version groups") end' \
    ci/cargo-release-groups.json > "$evidence/group-names"
: > "$evidence/manifests"
while IFS= read -r group; do
    (unset RELEASE_COMMIT; CDPATH='' bash scripts/ci/cargo-release-group.sh plan "$group") > "$evidence/$group-plan.json"
    jq -er '.manifests[].path' "$evidence/$group-plan.json" >> "$evidence/manifests"
done < "$evidence/group-names"
inputs=(--input Cargo.toml --input Cargo.lock --input toko-tooling.lock --input ci/cargo-release-groups.json
    --input scripts/ci/run-auth-validation.sh --input scripts/ci/cargo-release-group.sh
    --input scripts/ci/cargo-release-groups.jq)
while IFS= read -r manifest; do inputs+=(--input "$manifest"); done < "$evidence/manifests"
if [[ "$ci_mode" == true ]]; then
    ci_source=$(git rev-parse HEAD)
    git status --porcelain=v1 --untracked-files=all > "$evidence/ci-status"
    ci_clean=false
    [[ -s "$evidence/ci-status" ]] || ci_clean=true
    jq -cnS --arg source "$ci_source" --argjson clean "$ci_clean" \
        '{schema:1,source:$source,clean:$clean}' > "$evidence/ci-source.json"
    inputs+=(--input "${evidence#"$PWD/"}/ci-source.json")
    jq -cS '[.groups[].name]' ci/cargo-release-groups.json > "$evidence/groups.json"
    perl scripts/ci/validation-evidence.pl roster "$PWD" ci/release-validation.json \
        "${evidence#"$PWD/"}/groups.json" > "$evidence/stages"
    stages=()
    while IFS= read -r stage; do stages+=("${stage#release-stage-}"); done < "$evidence/stages"
    set -- --fail-fast --input ci/release-validation.json --input ci/cargo-release-groups.json \
        --input scripts/ci/run-auth-ci.sh --input scripts/ci/validation-evidence.pl \
        --input scripts/ci/ValidationPlan.pm "$@" "${stages[@]}"
fi
perl "$checkpoint" "$PWD" "$evidence/before.json" "$evidence/discovered.json" "$checksum"
status=0
bash scripts/ci/run-validation-targets.sh "${inputs[@]}" "$@" || status=$?
# Discovery and execution must see the same root selection, including the gap
# before the runner takes over its expanded seal. Preserve the first failure.
perl "$checkpoint" "$PWD" "$evidence/before.json" "$evidence/after.json" "$checksum" || {
    [[ "$status" != 0 ]] || status=1
}
if [[ "$ci_mode" == true ]]; then
    if [[ $(git rev-parse HEAD) != "$ci_source" ]] ||
        { [[ "$ci_clean" == true ]] && ! bash scripts/ci/check-release-source.sh; }; then
        echo 'CI source changed during validation' >&2
        [[ "$status" != 0 ]] || status=1
    fi
fi
exit "$status"
