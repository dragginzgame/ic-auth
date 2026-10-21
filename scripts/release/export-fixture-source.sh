#!/usr/bin/env bash
set -euo pipefail

# Select current maintained sources once; Shared Tooling owns file admission and
# copying. Worktree bytes, additions and deletions win over HEAD and the index.
[[ $# == 2 && -d "$1" && -d "$2" ]] || {
    echo 'usage: export-fixture-source.sh SOURCE EMPTY-FIXTURE' >&2; exit 2;
}
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
source_root="$(CDPATH='' cd -P -- "$1" && printf '%s/.' "$PWD")"
source_root="${source_root%/.}"
fixture_root="$(CDPATH='' cd -P -- "$2" && printf '%s/.' "$PWD")"
fixture_root="${fixture_root%/.}"
[[ "$(git -C "$source_root" rev-parse --show-prefix)" == '' ]] || {
    echo 'release fixture source must be a Git worktree root' >&2; exit 2;
}
shopt -s nullglob dotglob
for path in "$fixture_root/"*; do
    echo "release fixture destination must be empty: $path" >&2; exit 2
done
selection="$(mktemp "${fixture_root%/*}/fixture-source.XXXXXX")"
trap 'rm -f "$selection"' EXIT
git -C "$source_root" ls-files --cached --others --exclude-standard -z > "$selection"
files=()
while IFS= read -r -d '' path; do
    # Tracked working deletions must not be resurrected from HEAD. A dangling
    # symlink is still an input for the canonical helper to refuse.
    [[ -e "$source_root/$path" || -L "$source_root/$path" ]] || continue
    files+=("$path")
done < "$selection"
bash "${script_path%/*}/../ci/prepare-script-fixture.sh" \
    "$source_root" "$fixture_root" ${files[@]+"${files[@]}"}
