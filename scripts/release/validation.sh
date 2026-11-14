#!/usr/bin/env bash
# Auth owns group obligations; Shared owns exhaustive dispatch and checkpoints.
set -euo pipefail
[[ $# == 1 ]]
directory=$1
[[ ! -e $directory && ! -L $directory ]]
mkdir -p "$directory/logs"
perl scripts/release/validation-plan.pl "${RELEASE_SOURCE:?}" "${RELEASE_GROUP:?}" "${RELEASE_PREVIOUS:?}" "$directory"
export AUTH_RELEASE_VALIDATION_DIRECTORY="$directory"
VALIDATION_LOG_DIR="$directory/logs" bash scripts/ci/run-auth-validation.sh --plan "$directory/plan.json"
perl scripts/release/validation-plan.pl "$RELEASE_SOURCE" "$RELEASE_GROUP" "$RELEASE_PREVIOUS" "$directory" --check > /dev/null
source vendor/toko-tooling/scripts/ci/release-receipts.sh
release_receipt_write "$directory/evidence.json" perl scripts/release/validation-record.pl collect "$directory"
