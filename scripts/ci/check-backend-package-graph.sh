#!/usr/bin/env bash
set -euo pipefail
[[ $# == 4 ]] || { echo 'usage: check-backend-package-graph.sh METADATA BUNDLE_MANIFEST ENGINE_VERSION PROTOCOL_VERSION' >&2; exit 2; }
source "${BASH_SOURCE[0]%/*}/auth-dependency-policy.sh"
jq -e --arg bundle "$2" --arg version "$3" --arg protocol "$4" \
    --arg forbidden "$(auth_dependency_pattern backend-conformance)" '
    .packages as $packages
    | .resolve.root as $root
    | .workspace_members == [$root]
    and ([ $packages[] | select(.id == $root) ] | length == 1)
    and ([ $packages[] | select(.id == $root) ][0]
        | .name == "auth-backend-package-consumer" and .source == null)
    and all(["ic-auth", "ic-auth-protocol-types"][];
        . as $name
        | (if $name == "ic-auth" then $version else $protocol end) as $expected
        | [ $packages[] | select(.name == $name) ] as $selected
        | ($selected | length == 1)
        and ($selected[0].version == $expected
            and $selected[0].source == "registry+https://github.com/rust-lang/crates.io-index"))
    and ([ $packages[] | select(.name == "ic-auth-backend-conformance") ] as $selected
        | ($selected | length == 1)
        and ($selected[0].source == null and $selected[0].version == $version
            and $selected[0].manifest_path == $bundle)
        and all($selected[0].dependencies[]; .path == null and
            .source == "registry+https://github.com/rust-lang/crates.io-index"))
    and all(["ic-platform", "ic-platform-types"][];
        . as $name | [ $packages[] | select(.name == $name) ] | length == 1)
    and all($packages[] | select(.source == null);
        .id == $root or .name == "ic-auth-backend-conformance")
    and all($packages[]; (.name | test($forbidden)) | not)
' "$1" > /dev/null || {
    echo 'backend package must select the exact registry Engine/Protocol versions and excluded source bundle' >&2
    exit 1
}
