# Auth's ordered public package inventory. Source from the repository root.
# Release/retry evidence, package qualification and license checks share it.
packages=(ic-auth-protocol-types ic-auth ic-auth-wire)
package_args=()
for package in "${packages[@]}"; do package_args+=(-p "$package"); done
