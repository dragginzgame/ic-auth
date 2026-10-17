#!/usr/bin/env bash
set -euo pipefail
source "${BASH_SOURCE[0]%/*}/auth-dependency-policy.sh"
forbidden="$(auth_dependency_pattern packaged)"

[[ $# == 3 ]] || { echo 'usage: check-package-consumer.sh WORKSPACE_LOCK FIXTURE VERSION' >&2; exit 2; }
workspace_lock="$1"
fixture="$(cd -P "$2" && pwd)"
version="$3"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || exit 1

# Cargo owns resolution. Admit its exact unpacked package identities before
# compilation, rather than trusting a successful build of registry payloads.
jq -e --arg forbidden "$forbidden" --arg version "$version" --arg fixture "$fixture" '
    .packages as $packages
    | .resolve.root as $root
    | [.resolve.nodes[].id] as $resolved
    | ([ $packages[] | select(.id == $root) ] | length == 1)
    and ([ $packages[] | select(.id == $root) ][0]
        | .name == "ic-auth-msrv-consumer" and .version == "0.0.0"
        and .source == null and .manifest_path == ($fixture + "/Cargo.toml"))
    and all(["ic-auth", "ic-auth-protocol-types"][];
        . as $name
        | [ $packages[] | select(.name == $name) ] as $selected
        | ($selected | length == 1)
        and ($selected[0].version == $version and $selected[0].source == null
            and $selected[0].manifest_path == ($fixture + "/" + $name + "-" + $version + "/Cargo.toml")))
    and all($packages[]; . as $package | $resolved | index($package.id) != null)
    and all($packages[] | select(.source == null);
        .name == "ic-auth-msrv-consumer" or .name == "ic-auth" or .name == "ic-auth-protocol-types")
    and all($packages[];
        (.name | test($forbidden)) | not)
' "$fixture/metadata.json" > /dev/null || {
    echo 'package consumer did not select the exact isolated library graph' >&2; exit 1;
}
"$BASH" scripts/dev/run-host-tooling.sh verify-external-lock \
    "$workspace_lock" "$fixture/Cargo.lock" 8388608 || {
    echo 'package consumer changed an external lock selection' >&2; exit 1;
}
printf 'Exact archive manifests and selected external dependencies admitted\n'
