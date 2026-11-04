# shellcheck shell=bash
# Auth's direct metadata/publication writers share the runner's retained-state
# policy, including a distinct Git common directory when present. Never clean.
auth_release_retention() {
    local git_state common_state root
    git_state="$(git rev-parse --absolute-git-dir)" || return
    common_state="$(git rev-parse --path-format=absolute --git-common-dir)" || return
    local roots=()
    for root in "$git_state" "$common_state"; do
        roots+=(--root "$root/release-state" --root "$root/publication-state"
            --root "$root/retired-checkouts")
    done
    perl scripts/ci/check-retained-state.pl "${roots[@]}"
}
