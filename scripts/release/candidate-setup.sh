#!/usr/bin/env bash
set -euo pipefail

# Shared owns batch orchestration. Auth prepares the candidate's own formatting
# tools and browser dependencies; preflight retains product tools and the gate.
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
[[ "${RELEASE_BATCH_ACTIVE:-}" == 1 &&
   "${RELEASE_SOURCE:-}" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || {
    echo 'candidate setup requires the saved frozen release context' >&2; exit 2;
}
case "${RELEASE_SETUP_MODE:-}" in
    prepare) make --no-print-directory install-host-tools ;;
    check) make --no-print-directory host-tools-check ;;
    *) echo 'candidate setup requires prepare or check mode' >&2; exit 2 ;;
esac
export PATH="$PWD/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
source scripts/release/group.sh
auth_release_group - < <(git show "$RELEASE_SOURCE:ci/cargo-release-groups.json")
if [[ "$RELEASE_SETUP_MODE" == prepare ]]; then
    make --no-print-directory install-rust-tools install-client-dependencies
else
    make --no-print-directory rust-tools-check client-tools-check
fi
if [[ "$RELEASE_SETUP_MODE" == check ]]; then
    # A partial version install must never select an editor from the live lock.
    mkdir -p target/release-candidate-setup
    scratch="$(mktemp -d "$PWD/target/release-candidate-setup/saved.XXXXXX")"
    finish() {
        local status=$?
        if [[ $status == 0 ]]; then rm -rf "$scratch"
        else printf 'Candidate setup evidence retained: %s\n' "$scratch" >&2; fi
        exit "$status"
    }
    trap finish EXIT
    git show "$RELEASE_SOURCE:Cargo.lock" > "$scratch/base.lock"
    git show "$RELEASE_SOURCE:Cargo.toml" > "$scratch/base.toml"
    bash scripts/dev/release-tools.sh check-wait "$scratch/base.lock" "$scratch/base.toml" > /dev/null
fi
