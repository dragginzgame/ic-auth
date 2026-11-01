# shellcheck shell=bash
# Auth's ordered public inventory. `all` is read-only qualification; publication
# and dry-run entrypoints must explicitly select a declared release group.
packages=(ic-auth-protocol-types ic-auth-wire ic-auth)
case "${1:?select all or selected}" in
    all) ;;
    selected)
        source scripts/release/group.sh
        auth_release_group || return
        selected_packages=()
        for package in "${packages[@]}"; do
            if jq -e --arg group "$RELEASE_GROUP" --arg package "$package" \
                '.groups[] | select(.name == $group) | .members | index($package) != null' \
                ci/cargo-release-groups.json > /dev/null; then
                selected_packages+=("$package")
            fi
        done
        packages=("${selected_packages[@]}")
        ;;
    *) echo 'select all or selected package inventory' >&2; return 2 ;;
esac
package_args=()
for package in "${packages[@]}"; do package_args+=(-p "$package"); done
