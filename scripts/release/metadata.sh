#!/usr/bin/env bash
set -euo pipefail

# Consumer metadata/evidence adapter. Git effects belong to the shared runner.
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
operation="${1:?operation required}"
fail() { echo "release metadata refused: $*" >&2; exit 1; }
[[ "${RELEASE_DELIVERY:-direct}" == direct ]] || fail 'this repository qualifies direct delivery only'
files=(Cargo.toml Cargo.lock CHANGELOG.md release-validation.json)
if [[ "$operation" == files ]]; then printf '%s\0' "${files[@]}"; exit 0; fi
for value in "${RELEASE_PREVIOUS:?}" "${RELEASE_VERSION:?}"; do
    [[ "$value" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || fail 'invalid selected version'
done
[[ "${RELEASE_SOURCE:?}" =~ ^[0-9a-f]{40,64}$ && "${RELEASE_DATE:?}" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || fail 'invalid source/date'
state="$(git rev-parse --absolute-git-dir)/release-state"
[[ ! -L "$state" ]] || fail 'symlinked release state'
mkdir -p "$state"
receipt="$state/validation-$RELEASE_SOURCE-$RELEASE_VERSION.json"
scratch="$(mktemp -d "$state/metadata.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT

admit_changes() {
    bash scripts/ci/check-release-source.sh \
        --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md --allow release-validation.json
}
read_receipt() {
    [[ -f "$1" && ! -L "$1" ]] || fail 'missing validation receipt; run the release through the shared runner'
    jq -e --arg source "$RELEASE_SOURCE" --arg previous "$RELEASE_PREVIOUS" \
        --arg candidate "$RELEASE_VERSION" --arg kind "${RELEASE_KIND:?}" --arg date "$RELEASE_DATE" '
        .schema == 1 and .gate == "ci" and .source == $source and .previous == $previous
        and .candidate == $candidate and .kind == $kind and .date == $date
        and (.changelog | type == "string") and (.lock_sha256 | test("^[a-f0-9]{64}$"))
    ' "$1" > /dev/null || fail 'validation receipt does not match the selected source/release'
    git show "$RELEASE_SOURCE:Cargo.lock" > "$scratch/base.lock"
    # Recovery may have a partially prepared manifest/lock pair. Use the
    # adopted portable reader here: rebuilding a workspace utility would depend
    # on the very metadata this boundary is reconciling.
    [[ "$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$scratch/base.lock")" == "$(jq -r .lock_sha256 "$1")" ]] || fail 'validated lockfile mismatch'
}
render() {
    git show "$RELEASE_SOURCE:Cargo.toml" > "$scratch/base.toml"
    yq -p toml -o json '.' "$scratch/base.toml" | jq -e --arg previous "$RELEASE_PREVIOUS" \
        '.workspace.package.version == $previous and .workspace.dependencies."ic-auth-protocol-types".version == $previous' > /dev/null || fail 'source version mismatch'
    perl scripts/release/rewrite-manifest.pl "$scratch/base.toml" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" > "$scratch/Cargo.toml"
    # The saved source owns its local package set. Late checks must also admit
    # releases that predate the addition of the native utility.
    yq -p toml -o json '.' "$scratch/base.lock" | jq -r '.package[] | select(.source == null) | .name' > "$scratch/packages"
    local_packages=()
    while IFS= read -r package; do local_packages+=("$package"); done < "$scratch/packages"
    [[ ${#local_packages[@]} -gt 0 ]] || fail 'validated source has no local packages'
    perl scripts/ci/rewrite-local-lock-versions.pl "$scratch/base.lock" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" \
        "${local_packages[@]}" > "$scratch/Cargo.lock"
    # -j adds no extra LF: the receipt preserves the original notes byte-for-byte.
    jq -j .changelog "$scratch/receipt.json" > "$scratch/notes"
    awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" -v date="$RELEASE_DATE" \
        -v allow_finalized=1 -f scripts/ci/finalize-release-changelog.awk "$scratch/notes" > "$scratch/CHANGELOG.md"
    cp "$scratch/receipt.json" "$scratch/release-validation.json"
}
compare_manifest() {
    yq -p toml -o json '.' "$1" | jq -S . > "$scratch/observed.json"
    yq -p toml -o json '.' "$2" | jq -S . > "$scratch/expected.json"
    cmp -s "$scratch/observed.json" "$scratch/expected.json" || fail 'unexpected manifest changes'
}
check_payload() {
    compare_manifest "$1/Cargo.toml" "$scratch/Cargo.toml"
    cmp -s "$1/Cargo.lock" "$scratch/Cargo.lock" || fail 'prepared lockfile differs from the selected graph'
    cmp -s "$1/CHANGELOG.md" "$scratch/CHANGELOG.md" || fail 'prepared changelog mismatch'
    cmp -s "$1/release-validation.json" "$scratch/receipt.json" || fail 'prepared receipt mismatch'
}

case "$operation" in
    preflight)
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'selected source changed'
        if [[ -f "$receipt" ]]; then
            admit_changes
            # Recovery after an interrupted preparation: admit only exact base
            # or prepared bytes for each file, then rebuild the saved payload.
            read_receipt "$receipt"
            cp "$receipt" "$scratch/receipt.json"
            render
            for path in "${files[@]}"; do
                if [[ -f "$path" ]] && cmp -s "$path" "$scratch/$path"; then continue; fi
                if git cat-file -e "$RELEASE_SOURCE:$path" 2>/dev/null; then
                    git show "$RELEASE_SOURCE:$path" > "$scratch/base-file"
                    cmp -s "$path" "$scratch/base-file" && continue
                elif [[ ! -e "$path" ]]; then continue; fi
                [[ "$path" == CHANGELOG.md ]] && cmp -s "$path" "$scratch/notes" && continue
                fail "conflicting interrupted metadata: $path"
            done
            exit 0
        fi
        bash scripts/ci/check-release-source.sh --allow CHANGELOG.md
        [[ "$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)" == "$RELEASE_PREVIOUS" ]] || fail 'current version mismatch'
        heading="$(awk '/^## / { print; exit }' CHANGELOG.md)"
        [[ "$heading" == "## [$RELEASE_VERSION]" ]] || fail "pending changelog must select $RELEASE_VERSION (selected $RELEASE_KIND)"
        awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" -v date="$RELEASE_DATE" \
            -f scripts/ci/finalize-release-changelog.awk CHANGELOG.md > /dev/null
        cargo fetch --locked
        ;;
    verify)
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'validation source mismatch'
        admit_changes
        cp CHANGELOG.md "$scratch/notes"
        VALIDATION_LOG_DIR="$state/validation-logs" make --no-print-directory ci
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'validation source changed'
        admit_changes
        cmp -s CHANGELOG.md "$scratch/notes" || fail 'notes changed during validation'
        git diff --quiet HEAD -- Cargo.toml Cargo.lock release-validation.json || fail 'metadata changed during validation'
        lock_digest="$(bash scripts/dev/run-host-tooling.sh hash-file Cargo.lock 8388608)"
        jq -n --arg source "$RELEASE_SOURCE" --arg previous "$RELEASE_PREVIOUS" --arg candidate "$RELEASE_VERSION" \
            --arg kind "$RELEASE_KIND" --arg date "$RELEASE_DATE" --rawfile notes "$scratch/notes" \
            --arg lock "$lock_digest" --arg verified "$(date -u +%FT%TZ)" \
            '{schema:1, gate:"ci", source:$source, previous:$previous, candidate:$candidate,
              kind:$kind, date:$date, verified_at:$verified, lock_sha256:$lock, changelog:$notes}' > "$scratch/receipt.json"
        bash scripts/dev/run-host-tooling.sh replace-private "$scratch/receipt.json" "$receipt" 1048576
        ;;
    prepare-version|prepared-check|commit-check)
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'preparation source changed'
        admit_changes
        read_receipt "$receipt"
        cp "$receipt" "$scratch/receipt.json"
        render
        if [[ "$operation" == prepare-version ]]; then
            for path in Cargo.lock CHANGELOG.md; do cp "$scratch/$path" "$path"; done
            cp "$scratch/receipt.json" release-validation.json
            # The canonical version is replaced last so an interrupted attempt
            # can rerun preparation using the original selected version.
            cp "$scratch/Cargo.toml" Cargo.toml
            cargo sort --workspace
        fi
        check_payload .
        cargo metadata --locked --offline --format-version 1 > /dev/null
        make --no-print-directory fmt-check
        if [[ "$operation" == commit-check ]]; then
            git diff --quiet -- "${files[@]}" || fail 'release index differs from prepared worktree'
            for path in "${files[@]}"; do git cat-file -e ":$path"; done
        fi
        ;;
    committed-check|tagged-check|push-check)
        selected="${RELEASE_COMMIT:?exact release commit required}"
        [[ "$(git log -1 --format=%P "$selected")" == "$RELEASE_SOURCE" ]] || fail 'release parent does not match validated source'
        mkdir "$scratch/committed"
        for path in "${files[@]}"; do git show "$selected:$path" > "$scratch/committed/$path"; done
        read_receipt "$scratch/committed/release-validation.json"
        cp "$scratch/committed/release-validation.json" "$scratch/receipt.json"
        render
        check_payload "$scratch/committed"
        git diff --name-only -z "$RELEASE_SOURCE" "$selected" -- > "$scratch/changed"
        while IFS= read -r -d '' path; do
            case "$path" in Cargo.toml|Cargo.lock|CHANGELOG.md|release-validation.json) ;;
                *) fail "release changed unvalidated source: $path" ;;
            esac
        done < "$scratch/changed"
        if [[ "$operation" != committed-check ]]; then
            bash scripts/ci/check-release-tag.sh "$selected" "$RELEASE_VERSION"
        fi
        ;;
    *) fail "unknown adapter operation: $operation" ;;
esac
