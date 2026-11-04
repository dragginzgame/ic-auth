#!/usr/bin/env bash
set -euo pipefail

# Explicit crates.io upload only. No release/tag creation or artifact cleanup.
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
fail() { echo "publication refused: $*" >&2; exit 1; }
source scripts/release/retention.sh
auth_release_retention || fail 'retained state requires review before publication'
source scripts/release/packages.sh selected
version="$(bash "$PWD/scripts/ci/cargo-release-group.sh" version "$RELEASE_GROUP")"
source="$(git rev-parse --verify HEAD)"
[[ -z "$(git status --porcelain --untracked-files=all)" ]] || fail 'commit the complete publication source first'
bash scripts/ci/check-release-tag.sh "$source" "$version" "$RELEASE_GROUP"
[[ -f release-validation.json ]] || fail 'the tagged release has no validation receipt'
# Bind the release metadata to its original validated source, including retries
# from a fresh clone. The shared adapter checks the selected commit, not HEAD.
RELEASE_PREVIOUS="$(jq -er .previous release-validation.json)" \
RELEASE_VERSION="$version" RELEASE_SOURCE="$(jq -er .source release-validation.json)" \
RELEASE_DATE="$(jq -er .date release-validation.json)" RELEASE_KIND="$(jq -er .kind release-validation.json)" \
RELEASE_COMMIT="$source" bash scripts/release/metadata.sh committed-check
destination="$(git remote get-url --push --all "${RELEASE_REMOTE:-origin}")"
[[ -n "$destination" && "$destination" != *$'\n'* ]] || fail 'publication requires one selected push URL'
tag_ref="refs/tags/$RELEASE_GROUP-v$version"
tag="$(git rev-parse "$tag_ref")"
remote_tag="$(git ls-remote --refs -- "$destination" "$tag_ref")"
[[ "$remote_tag" == "$tag"$'\t'"$tag_ref" ]] || fail 'the exact annotated release tag has not been pushed'
assert_source() {
    [[ "$(git rev-parse HEAD)" == "$source" && -z "$(git status --porcelain --untracked-files=all)" ]] || fail 'source changed during publication'
    [[ "$(git rev-parse "$tag_ref")" == "$tag" ]] || fail 'selected release tag changed'
}

state="$(git rev-parse --absolute-git-dir)/publication-state"
[[ ! -L "$state" ]] || fail 'symlinked publication state'
# New namespaces must not bypass uncertain coordinated uploads of the same
# registry version. Reconcile legacy custody with its original adapter/source.
[[ ! -e "$state/$version.json" && ! -L "$state/$version.json" ]] ||
    fail "legacy publication intent exists for $version; reconcile it with the original release adapter before group publication"
for package in "${packages[@]}"; do
    [[ ! -e "$state/$version-$package.dispatched" && ! -L "$state/$version-$package.dispatched" ]] ||
        fail "legacy $package upload custody exists for $version; reconcile it with the original release adapter"
done
mkdir -p "$state"
mkdir "$state/lock" 2>/dev/null || fail 'publication lock is occupied; inspect its owner before clearing stale state'
printf '%s\n' "$$" > "$state/lock/owner"
finish() {
    local status=$? retention_status=0
    auth_release_retention || retention_status=$?
    [[ "$status" != 0 ]] || status=$retention_status
    rm -f "$state/lock/owner" || { [[ "$status" != 0 ]] || status=1; }
    rmdir "$state/lock" || { [[ "$status" != 0 ]] || status=1; }
    exit "$status"
}
trap finish EXIT
mkdir -p "${CARGO_TARGET_DIR:-$PWD/target}"
logs="$(mktemp -d "${CARGO_TARGET_DIR:-$PWD/target}/publication-$RELEASE_GROUP-$version.XXXXXX")"
echo "Publication evidence retained: $logs"
step() {
    auth_release_retention || return
    local status=0 retention_status=0
    "$@" 2>&1 | tee -a "$logs/output.log" || status=$?
    auth_release_retention || retention_status=$?
    [[ "$status" != 0 ]] || status=$retention_status
    return "$status"
}
step make --no-print-directory check-package-licenses
step cargo package --locked --registry crates-io "${package_args[@]}"
assert_source
plan="$state/$RELEASE_GROUP-$version.json"
candidate="$logs/intent.json"
hashes=()
package_records="$logs/packages.jsonl"
for package in "${packages[@]}"; do
    archive="${CARGO_TARGET_DIR:-$PWD/target}/package/$package-$version.crate"
    [[ -f "$archive" && ! -L "$archive" ]] || fail "missing package archive: $archive"
    checksum="$(bash scripts/dev/run-host-file.sh hash-file "$archive" 67108864)"
    hashes+=("$checksum")
    jq -n --arg name "$package" --arg checksum "$checksum" \
        '{name:$name,checksum:$checksum}' >> "$package_records"
done
jq -s --arg source "$source" --arg version "$version" --arg tag "$tag" --arg group "$RELEASE_GROUP" \
    '{schema:2, group:$group, source:$source, version:$version, tag:$tag, registry:"crates-io",
      packages:.}' "$package_records" > "$candidate"
if [[ -e "$plan" || -L "$plan" ]]; then
    if [[ -L "$plan" ]] || ! cmp -s "$candidate" "$plan"; then
        fail 'saved publication intent conflicts with the current source or archives'
    fi
else
    # Intent precedes every registry effect. Retries retain the same source,
    # registry, exact version and archive checksums.
    bash scripts/dev/run-host-file.sh create-private "$candidate" "$plan" 65536
fi
observe() {
    local package="$1" checksum="$2" evidence observed
    evidence="$(mktemp -d "$logs/$package-observation.XXXXXX")" || return 2
    observed="$(bash scripts/ci/check-crates-io-version.sh \
        --metadata "$evidence/registry" "$package" "$version")" || return $?
    # Shared owns transport and admitted metadata; Auth owns payload acceptance.
    # Yanked matching bytes still reconcile an earlier upload, as before.
    [[ "$(jq -r '.checksum' <<< "$observed")" == "$checksum" ]] ||
        fail "registry checksum conflict for $package $version"
}
for index in "${!packages[@]}"; do
    package="${packages[$index]}"
    checksum="${hashes[$index]}"
    status=0
    observe "$package" "$checksum" || status=$?
    if [[ "$status" == 0 ]]; then echo "Verified $package $version; skipping an identical upload"; continue; fi
    [[ "$status" == 1 ]] || fail "cannot establish registry state for $package; nothing dispatched"
    dispatched="$state/$RELEASE_GROUP-$version-$package.dispatched"
    [[ ! -e "$dispatched" && ! -L "$dispatched" ]] || fail "previous $package upload is unresolved; retain intent and wait for registry confirmation before retrying"
    # The shared lock has one stable parent across all versions/invocations.
    # Matching registry observations above may reconcile Auth's intent, but do
    # not clear an incomplete shared attempt or authorize another dispatch.
    attempt="$state/${logs##*/}-$package"
    [[ ! -e "$state/.cargo-publication.lock" && ! -L "$state/.cargo-publication.lock" ]] ||
        fail "shared publication attempt lock is retained; inspect its owner/evidence before another dispatch: $state/.cargo-publication.lock"
    [[ ! -e "$attempt" && ! -L "$attempt" ]] || fail "publication attempt already exists; retain and reconcile: $attempt"
    assert_source
    marker="$logs/$package.dispatched"
    printf '%s\n' "$source" "$checksum" > "$marker"
    bash scripts/dev/run-host-file.sh create-private "$marker" "$dispatched" 1024
    assert_source
    status=0
    step bash scripts/ci/run-cargo-publication.sh publish "$attempt" "$source" crates-io \
        --package "$package@$version" || status=$?
    # A lost reply or Cargo index-propagation timeout does not prove failure.
    # Reconcile the exact bytes even when the upload command reports failure.
    observed=0
    observe "$package" "$checksum" || observed=$?
    [[ "$observed" == 0 ]] || fail "upload result for $package is unresolved (Cargo $status); rerun to observe, without redispatch"
    assert_source
done
echo "Verified IC Auth $RELEASE_GROUP packages at $version on crates.io"
if [[ -e "$state/.cargo-publication.lock" || -L "$state/.cargo-publication.lock" ]]; then
    echo "Registry reconciliation does not release the retained shared-attempt lock: $state/.cargo-publication.lock"
fi
