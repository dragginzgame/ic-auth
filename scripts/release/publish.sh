#!/usr/bin/env bash
set -euo pipefail

# Explicit crates.io upload only. No release/tag creation or artifact cleanup.
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
fail() { echo "publication refused: $*" >&2; exit 1; }
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
source="$(git rev-parse --verify HEAD)"
[[ -z "$(git status --porcelain --untracked-files=all)" ]] || fail 'commit the complete publication source first'
bash scripts/ci/check-release-tag.sh "$source" "$version"
[[ -f release-validation.json ]] || fail 'the tagged release has no validation receipt'
# Bind the release metadata to its original validated source, including retries
# from a fresh clone. The shared adapter checks the selected commit, not HEAD.
RELEASE_PREVIOUS="$(jq -er .previous release-validation.json)" \
RELEASE_VERSION="$version" RELEASE_SOURCE="$(jq -er .source release-validation.json)" \
RELEASE_DATE="$(jq -er .date release-validation.json)" RELEASE_KIND="$(jq -er .kind release-validation.json)" \
RELEASE_COMMIT="$source" bash scripts/release/metadata.sh committed-check
destination="$(git remote get-url --push --all "${RELEASE_REMOTE:-origin}")"
[[ -n "$destination" && "$destination" != *$'\n'* ]] || fail 'publication requires one selected push URL'
tag="$(git rev-parse "refs/tags/v$version")"
remote_tag="$(git ls-remote --refs -- "$destination" "refs/tags/v$version")"
[[ "$remote_tag" == "$tag"$'\t'"refs/tags/v$version" ]] || fail 'the exact annotated release tag has not been pushed'
assert_source() {
    [[ "$(git rev-parse HEAD)" == "$source" && -z "$(git status --porcelain --untracked-files=all)" ]] || fail 'source changed during publication'
    [[ "$(git rev-parse "refs/tags/v$version")" == "$tag" ]] || fail 'selected release tag changed'
}

state="$(git rev-parse --absolute-git-dir)/publication-state"
[[ ! -L "$state" ]] || fail 'symlinked publication state'
mkdir -p "$state"
mkdir "$state/lock" 2>/dev/null || fail 'publication lock is occupied; inspect its owner before clearing stale state'
printf '%s\n' "$$" > "$state/lock/owner"
trap 'rm -f "$state/lock/owner"; rmdir "$state/lock"' EXIT
mkdir -p "${CARGO_TARGET_DIR:-$PWD/target}"
logs="$(mktemp -d "${CARGO_TARGET_DIR:-$PWD/target}/publication-$version.XXXXXX")"
echo "Publication evidence retained: $logs"
step() { "$@" 2>&1 | tee -a "$logs/output.log"; }
packages=(ic-auth-protocol-types ic-auth)
step make --no-print-directory check-package-licenses
step cargo package --locked --registry crates-io -p ic-auth-protocol-types -p ic-auth
assert_source
plan="$state/$version.json"
candidate="$logs/intent.json"
hashes=()
for package in "${packages[@]}"; do
    archive="${CARGO_TARGET_DIR:-$PWD/target}/package/$package-$version.crate"
    [[ -f "$archive" && ! -L "$archive" ]] || fail "missing package archive: $archive"
    hashes+=("$(sha256sum "$archive" | cut -d ' ' -f 1)")
done
jq -n --arg source "$source" --arg version "$version" --arg tag "$tag" \
    --arg types "${hashes[0]}" --arg auth "${hashes[1]}" \
    '{schema:1, source:$source, version:$version, tag:$tag, registry:"crates-io",
      packages:[{name:"ic-auth-protocol-types",checksum:$types},{name:"ic-auth",checksum:$auth}]}' > "$candidate"
if [[ -e "$plan" || -L "$plan" ]]; then
    if [[ -L "$plan" ]] || ! cmp -s "$candidate" "$plan"; then
        fail 'saved publication intent conflicts with the current source or archives'
    fi
else
    # Intent precedes every registry effect. Retries retain the same source,
    # registry, exact version and archive checksums.
    cp "$candidate" "$plan"
fi
observe() {
    local package="$1" checksum="$2" status observed
    status="$(curl --disable --silent --show-error --location --proto '=https' --proto-redir '=https' \
        --connect-timeout 10 --max-time 30 --output "$logs/$package.json" --write-out '%{http_code}' \
        --user-agent 'dragginzgame-ic-auth (https://github.com/dragginzgame/ic-auth)' \
        "https://crates.io/api/v1/crates/$package/$version")" || return 2
    case "$status" in
        404) return 1 ;;
        200)
            observed="$(jq -er --arg version "$version" '.version | select(.num == $version) | .checksum' "$logs/$package.json")" || return 2
            [[ "$observed" == "$checksum" ]] || fail "registry checksum conflict for $package $version"
            return 0 ;;
        *) echo "Registry observation unavailable: HTTP $status" >&2; return 2 ;;
    esac
}
for index in "${!packages[@]}"; do
    package="${packages[$index]}"
    checksum="${hashes[$index]}"
    status=0
    observe "$package" "$checksum" || status=$?
    if [[ "$status" == 0 ]]; then echo "Verified $package $version; skipping an identical upload"; continue; fi
    [[ "$status" == 1 ]] || fail "cannot establish registry state for $package; nothing dispatched"
    dispatched="$state/$version-$package.dispatched"
    [[ ! -e "$dispatched" ]] || fail "previous $package upload is unresolved; retain intent and wait for registry confirmation before retrying"
    assert_source
    printf '%s\n' "$source" "$checksum" > "$dispatched"
    status=0
    step cargo publish --locked --registry crates-io -p "$package" || status=$?
    # A lost reply or Cargo index-propagation timeout does not prove failure.
    # Reconcile the exact bytes even when the upload command reports failure.
    observed=0
    observe "$package" "$checksum" || observed=$?
    [[ "$observed" == 0 ]] || fail "upload result for $package is unresolved (Cargo $status); rerun to observe, without redispatch"
    assert_source
done
echo "Verified both IC Auth packages at $version on crates.io"
