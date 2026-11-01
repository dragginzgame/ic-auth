# shellcheck shell=bash
# Source-only selection before setup, validation or effects. The shared owner
# admits the policy shape; locked graph admission remains with its group helper.
auth_release_group() {
    local policy="${1:-ci/cargo-release-groups.json}" selection release_root="${BASH_SOURCE[0]}"
    [[ "$release_root" == /* ]] || release_root="$PWD/$release_root"
    release_root="$(CDPATH='' cd -P "${release_root%/*}/../.." && pwd -P)" || return
    [[ -n "${RELEASE_GROUP:-}" ]] || {
        echo 'release selection refused: explicit RELEASE_GROUP=protocol or RELEASE_GROUP=engine required' >&2
        return 1
    }
    [[ "$policy" == - || ( -f "$policy" && ! -L "$policy" ) ]] || return 1
    selection="$(jq -L "$release_root/scripts/ci" -er --arg group "$RELEASE_GROUP" '
        include "cargo-release-groups";
        release_policy | [.groups[] | select(.name == $group)] |
        if length == 1 then .[0] | [.primary,.changelog] | @tsv
        else error("unknown release group") end' "$policy")" || return
    # shellcheck disable=SC2034 # Outputs consumed by source callers.
    IFS=$'\t' read -r primary changelog <<< "$selection"
}

# Bootstrap observation must also work between interrupted file replacements.
# The runner independently admits the full locked graph on a fresh release;
# recovery then reconciles every file against the validated saved-source plan.
auth_release_version() {
    auth_release_group || return
    local release_root="${BASH_SOURCE[0]}" manifest
    [[ "$release_root" == /* ]] || release_root="$PWD/$release_root"
    release_root="$(CDPATH='' cd -P "${release_root%/*}/../.." && pwd -P)" || return
    manifest="$(yq -p toml -o json '.' Cargo.toml | jq -L "$release_root/scripts/ci" -er --arg primary "$primary" '
        include "cargo-release-groups";
        .workspace.dependencies[$primary].path |
        if relative then . + "/Cargo.toml" else error("invalid primary manifest path") end')" || return
    # shellcheck source=scripts/ci/release-metadata-transaction.sh
    source "$release_root/scripts/ci/release-metadata-transaction.sh"
    release_metadata_regular . "$manifest" || return 1
    yq -p toml -o json '.' "$manifest" | jq -L "$release_root/scripts/ci" -er --arg primary "$primary" '
        include "cargo-release-groups";
        if .package.name == $primary and (.package.version | stable)
        then .package.version else error("invalid literal primary version") end'
}
