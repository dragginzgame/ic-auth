#!/usr/bin/env bash
set -euo pipefail
[[ $# == 1 ]]
jq -e --arg stage "release-stage-$1" '.stages | index($stage) != null' ci/release-validation.json >/dev/null
[[ $(git rev-parse HEAD) == "${RELEASE_SOURCE:?}" ]]
ledger=$(jq -er --arg group "${RELEASE_GROUP:?}" '.groups[] | select(.name == $group) | .changelog' ci/cargo-release-groups.json)
baseline=$(jq -er .baseline_ref "${AUTH_RELEASE_VALIDATION_DIRECTORY:?}/context.json")
bash vendor/toko-tooling/scripts/ci/check-release-source.sh --allow "$ledger"
perl scripts/release/validation-plan.pl "$RELEASE_SOURCE" "$RELEASE_GROUP" "${RELEASE_PREVIOUS:?}" \
    "$AUTH_RELEASE_VALIDATION_DIRECTORY" --baseline "$baseline" --check > /dev/null
make --no-print-directory -- "$1"
[[ $(git rev-parse HEAD) == "$RELEASE_SOURCE" ]]
bash vendor/toko-tooling/scripts/ci/check-release-source.sh --allow "$ledger"
perl scripts/release/validation-plan.pl "$RELEASE_SOURCE" "$RELEASE_GROUP" "$RELEASE_PREVIOUS" \
    "$AUTH_RELEASE_VALIDATION_DIRECTORY" --baseline "$baseline" --check > /dev/null
