#!/usr/bin/env bash
# One exhaustive roster owns ordinary CI and release-plan admission.
set -euo pipefail
exec bash scripts/ci/run-auth-validation.sh --ci "$@"
