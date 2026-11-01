#!/usr/bin/env bash
set -euo pipefail

# Auth owns complete-gate receipts and saved-source recovery. Shared Tooling owns
# group admission, file plans and transactional Cargo/changelog rendering.
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
auth_release_root="$PWD"
operation="${1:?operation required}"
fail() { echo "release metadata refused: $*" >&2; exit 1; }
bash scripts/ci/check-release-delivery.sh ci/release-delivery.txt "${RELEASE_DELIVERY:-direct}" || fail 'this repository qualifies direct delivery only'
source scripts/release/group.sh
if [[ "$operation" == files ]]; then
    auth_release_group
    exec bash "$PWD/scripts/ci/cargo-release-group.sh" files "$RELEASE_GROUP"
fi
[[ -n "${RELEASE_GROUP:-}" ]] || fail 'explicit RELEASE_GROUP required'
for value in "${RELEASE_PREVIOUS:?}" "${RELEASE_VERSION:?}"; do
    [[ "$value" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || fail 'invalid selected version'
done
[[ "${RELEASE_SOURCE:?}" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ && "${RELEASE_DATE:?}" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || fail 'invalid source/date'
auth_release_group - < <(git show "$RELEASE_SOURCE:ci/cargo-release-groups.json") ||
    fail 'saved source has no matching release group; use its original release checkout for legacy recovery'
limit=8388608
state="$(git rev-parse --absolute-git-dir)/release-state"
[[ ! -L "$state" ]] || fail 'symlinked release state'
mkdir -p "$state"
auth_scratch="$(mktemp -d "$state/metadata.XXXXXX")"
complete=false
finish() {
    local status=$?
    if [[ "$complete" == true && "$status" == 0 ]]; then rm -rf "$auth_scratch"
    else printf 'Release metadata evidence retained: %s\n' "$auth_scratch" >&2; fi
    exit "$status"
}
trap finish EXIT
mkdir "$auth_scratch/source"
git archive --format=tar "$RELEASE_SOURCE" | tar -xf - -C "$auth_scratch/source"
# Pre-group attempts must resume with their original source and adapter in their
# dedicated release checkout. Do not reinterpret or discard historical custody.
auth_release_group "$auth_scratch/source/ci/cargo-release-groups.json" || fail 'saved source has no matching release group; use its original release checkout for legacy recovery'
policy_digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$auth_scratch/source/ci/cargo-release-groups.json")"
receipt="$state/validation-$RELEASE_GROUP-$RELEASE_SOURCE-$RELEASE_VERSION.json"
cp "$auth_scratch/source/Cargo.lock" "$auth_scratch/base.lock"
cp "$auth_scratch/source/Cargo.toml" "$auth_scratch/base.toml"
(
    unset RELEASE_COMMIT
    cd "$auth_scratch/source"
    bash "$auth_release_root/scripts/ci/cargo-release-group.sh" plan "$RELEASE_GROUP"
) > "$auth_scratch/plan.json"
[[ "$(jq -r .previous "$auth_scratch/plan.json")" == "$RELEASE_PREVIOUS" ]] || fail 'source version mismatch'
files=() allow_changes=() metadata_files=()
while IFS= read -r path; do
    files+=("$path")
    allow_changes+=(--allow "$path")
    [[ "$path" == "$changelog" ]] || metadata_files+=("$path")
done < <(jq -r '.files[]' "$auth_scratch/plan.json")
# Keep the bootstrap version old until the complete payload is installed. The
# runner can then re-enter saved-source reconciliation after a partial write.
primary_manifest="$(yq -p toml -o json '.' "$auth_scratch/base.toml" | jq -er --arg primary "$primary" '.workspace.dependencies[$primary].path + "/Cargo.toml"')"
jq -e --arg path "$primary_manifest" '.manifests | any(.path == $path and .version == "package")' "$auth_scratch/plan.json" > /dev/null
prepare_files=()
for path in "${files[@]}"; do [[ "$path" == "$primary_manifest" ]] || prepare_files+=("$path"); done
prepare_files+=("$primary_manifest")
admit_changes() { bash scripts/ci/check-release-source.sh "${allow_changes[@]}"; }
read_receipt() {
    [[ -f "$1" && ! -L "$1" ]] || fail 'missing validation receipt; run the release through the shared runner'
    jq -e --arg source "$RELEASE_SOURCE" --arg previous "$RELEASE_PREVIOUS" --arg candidate "$RELEASE_VERSION" \
        --arg kind "${RELEASE_KIND:?}" --arg date "$RELEASE_DATE" --arg group "$RELEASE_GROUP" --arg policy "$policy_digest" '
        .schema == 2 and .gate == "ci" and .group == $group and .policy_sha256 == $policy
        and .source == $source and .previous == $previous and .candidate == $candidate
        and .kind == $kind and .date == $date and (.changelog | type == "string")
        and (.lock_sha256 | test("^[a-f0-9]{64}$"))
    ' "$1" > /dev/null || fail 'validation receipt does not match the selected group/source/release'
    [[ "$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$auth_scratch/base.lock")" == "$(jq -r .lock_sha256 "$1")" ]] || fail 'validated lockfile mismatch'
}
render() {
    local editor status
    if editor="$(bash scripts/dev/release-tools.sh check-wait "$auth_scratch/base.lock" "$auth_scratch/base.toml")"; then :
    else
        status=$?
        if [[ "$status" == 75 ]]; then
            echo 'release metadata busy: saved-source editor admission timed out; retry the saved release after the installation owner finishes' >&2
            exit 75
        fi
        fail 'saved-source release editor admission failed; inspect the diagnostic above before preparing tools'
    fi
    jq -j .changelog "$auth_scratch/receipt.json" > "$auth_scratch/notes"
    cp "$auth_scratch/notes" "$auth_scratch/source/$changelog"
    (
        unset RELEASE_COMMIT
        cd "$auth_scratch/source"
        CARGO_RELEASE_METADATA_EDITOR="$editor" CARGO_RELEASE_METADATA_MAX_BYTES="$limit" \
            bash "$auth_release_root/scripts/ci/cargo-release-group.sh" prepare "$RELEASE_GROUP"
    )
    cp "$auth_scratch/receipt.json" "$auth_scratch/source/release-validation.json"
}
source scripts/ci/release-metadata-transaction.sh
check_payload() {
    local path
    for path in "${files[@]}"; do
        release_metadata_regular "$1" "$path" || fail "nonregular release metadata: $path"
        cmp -s "$1/$path" "$auth_scratch/source/$path" || fail "prepared metadata mismatch: $path"
    done
}
# shellcheck disable=SC2329 # Transaction callbacks.
install_candidate() {
    local path
    for path in "${files[@]}"; do cp "$auth_scratch/source/$path" "$2/$path" || return; done
}
# shellcheck disable=SC2329 # Transaction callback.
check_candidate() {
    check_payload . || return
    bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
}
case "$operation" in
    preflight)
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'selected source changed'
        if [[ -f "$receipt" ]]; then
            admit_changes
            read_receipt "$receipt"
            cp "$receipt" "$auth_scratch/receipt.json"
            render
            for path in "${files[@]}"; do
                if release_metadata_regular . "$path" && cmp -s "$path" "$auth_scratch/source/$path"; then continue; fi
                git show "$RELEASE_SOURCE:$path" > "$auth_scratch/base-file"
                cmp -s "$path" "$auth_scratch/base-file" && continue
                [[ "$path" == "$changelog" ]] && cmp -s "$path" "$auth_scratch/notes" && continue
                fail "conflicting interrupted metadata: $path"
            done
        else
            bash scripts/ci/check-release-source.sh --allow "$changelog"
            heading="$(awk '{ sub(/[ \t]+$/, ""); gsub(/[ \t]+/, " "); if (/^## /) { print; exit } }' "$changelog")"
            case "$heading" in
                "## [$RELEASE_VERSION]"|"## [$RELEASE_VERSION] - Unreleased") ;;
                *) fail "expected pending heading '## [$RELEASE_VERSION]' or '## [$RELEASE_VERSION] - Unreleased' for selected $RELEASE_KIND; found '$heading'. Release preparation adds the date." ;;
            esac
            awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" -v date="$RELEASE_DATE" \
                -f scripts/ci/finalize-release-changelog.awk "$changelog" > /dev/null
            cargo sort --workspace --grouped --check
            cargo fetch --locked
            make --no-print-directory install-tools
            make --no-print-directory tools-check
            bash scripts/dev/release-tools.sh install "$auth_scratch/base.lock" "$auth_scratch/base.toml"
            bash scripts/dev/release-tools.sh check "$auth_scratch/base.lock" "$auth_scratch/base.toml" > /dev/null
        fi
        ;;
    verify)
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'validation source mismatch'
        admit_changes
        cp "$changelog" "$auth_scratch/notes"
        VALIDATION_LOG_DIR="$state/validation-logs" make --no-print-directory ci
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'validation source changed'
        admit_changes
        cmp -s "$changelog" "$auth_scratch/notes" || fail 'notes changed during validation'
        git diff --quiet HEAD -- "${metadata_files[@]}" || fail 'metadata changed during validation'
        lock_digest="$(bash scripts/dev/run-host-file.sh hash-file Cargo.lock "$limit")"
        jq -n --arg source "$RELEASE_SOURCE" --arg previous "$RELEASE_PREVIOUS" --arg candidate "$RELEASE_VERSION" \
            --arg kind "$RELEASE_KIND" --arg date "$RELEASE_DATE" --rawfile notes "$auth_scratch/notes" \
            --arg group "$RELEASE_GROUP" --arg policy "$policy_digest" --arg lock "$lock_digest" --arg verified "$(date -u +%FT%TZ)" \
            '{schema:2, gate:"ci", group:$group, policy_sha256:$policy, source:$source, previous:$previous,
              candidate:$candidate, kind:$kind, date:$date, verified_at:$verified, lock_sha256:$lock, changelog:$notes}' > "$auth_scratch/receipt.json"
        bash scripts/dev/run-host-file.sh replace-private "$auth_scratch/receipt.json" "$receipt" 1048576
        ;;
    prepare-version|prepared-check|commit-check)
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'preparation source changed'
        admit_changes
        read_receipt "$receipt"
        cp "$receipt" "$auth_scratch/receipt.json"
        render
        if [[ "$operation" == prepare-version ]]; then
            release_metadata_prepare "$state/group-prepare.XXXXXX" install_candidate check_candidate "${prepare_files[@]}"
        else check_candidate; fi
        make --no-print-directory fmt-check
        if [[ "$operation" == commit-check ]]; then
            git diff --quiet -- "${files[@]}" || fail 'release index differs from prepared worktree'
            for path in "${files[@]}"; do git cat-file -e ":$path"; done
        fi
        ;;
    committed-check|tagged-check|push-check)
        selected_commit="${RELEASE_COMMIT:?exact release commit required}"
        [[ "$(git log -1 --format=%P "$selected_commit")" == "$RELEASE_SOURCE" ]] || fail 'release parent does not match validated source'
        mkdir "$auth_scratch/committed"
        for path in "${files[@]}"; do
            [[ "$path" != */* ]] || mkdir -p "$auth_scratch/committed/${path%/*}"
            git show "$selected_commit:$path" > "$auth_scratch/committed/$path"
        done
        read_receipt "$auth_scratch/committed/release-validation.json"
        cp "$auth_scratch/committed/release-validation.json" "$auth_scratch/receipt.json"
        render
        check_payload "$auth_scratch/committed"
        RELEASE_COMMIT="$selected_commit" bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
        git diff --name-only -z "$RELEASE_SOURCE" "$selected_commit" -- > "$auth_scratch/changed"
        while IFS= read -r -d '' path; do
            found=false
            for allowed in "${files[@]}"; do [[ "$path" != "$allowed" ]] || found=true; done
            [[ "$found" == true ]] || fail "release changed unvalidated source: $path"
        done < "$auth_scratch/changed"
        if [[ "$operation" != committed-check ]]; then bash scripts/ci/check-release-tag.sh "$selected_commit" "$RELEASE_VERSION" "$RELEASE_GROUP"; fi
        ;;
    *) fail "unknown adapter operation: $operation" ;;
esac
complete=true
