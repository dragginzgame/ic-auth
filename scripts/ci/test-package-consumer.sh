#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH"
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/package-consumer.XXXXXX")"
printf 'Package graph admission evidence retained: %s\n' "$fixture"
version=0.3.5
cat > "$fixture/original.lock" <<'TOML'
version = 4
[[package]]
name = "external"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
[[package]]
name = "unused-workspace-selection"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
TOML
perl -0777 -pe 's/\[\[package\]\]\nname = "unused-workspace-selection".*\z//s' \
    "$fixture/original.lock" > "$fixture/base.lock"
jq -n --arg fixture "$fixture" --arg version "$version" '{
    packages: [
        {id: "consumer", name: "ic-auth-msrv-consumer", version: "0.0.0", source: null, manifest_path: ($fixture + "/Cargo.toml")},
        {id: "auth", name: "ic-auth", version: $version, source: null, manifest_path: ($fixture + "/ic-auth-" + $version + "/Cargo.toml")},
        {id: "types", name: "ic-auth-protocol-types", version: $version, source: null, manifest_path: ($fixture + "/ic-auth-protocol-types-" + $version + "/Cargo.toml")},
        {id: "wire", name: "ic-auth-wire", version: $version, source: null, manifest_path: ($fixture + "/ic-auth-wire-" + $version + "/Cargo.toml")},
        {id: "external", name: "external", version: "1.0.0", source: "registry+https://github.com/rust-lang/crates.io-index"}
    ],
    resolve: {root: "consumer", nodes: [{id: "consumer"}, {id: "auth"}, {id: "types"}, {id: "wire"}, {id: "external"}]}
}' > "$fixture/base-metadata.json"
cp "$fixture/base.lock" "$fixture/Cargo.lock"
cp "$fixture/base-metadata.json" "$fixture/metadata.json"
"$BASH" scripts/ci/check-package-consumer.sh "$fixture/original.lock" "$fixture" "$version" > "$fixture/admitted.log"

reject_graph() {
    cp "$fixture/base.lock" "$fixture/Cargo.lock"
    jq "$2" "$fixture/base-metadata.json" > "$fixture/metadata.json"
    status=0
    "$BASH" scripts/ci/check-package-consumer.sh "$fixture/original.lock" "$fixture" "$version" > "$fixture/$1.log" 2>&1 || status=$?
    [[ "$status" == 1 ]] || { echo "unexpected graph admission: $1" >&2; exit 1; }
}
reject_graph registry-auth '.packages[1].source = "registry+https://github.com/rust-lang/crates.io-index"'
reject_graph live-types '.packages[2].manifest_path = "/live-workspace/crates/ic-auth-protocol-types/Cargo.toml"'
reject_graph registry-wire '.packages[3].source = "registry+https://github.com/rust-lang/crates.io-index"'
reject_graph live-wire '.packages[3].manifest_path = "/live-workspace/crates/ic-auth-wire/Cargo.toml"'
reject_graph missing-wire '.packages |= map(select(.id != "wire"))'
reject_graph wrong-version '.packages[1].version = "0.3.4"'
reject_graph wrong-name '.packages[2].name = "other-types"'
reject_graph duplicate-types '.packages += [.packages[2]]'
reject_graph missing-auth '.packages |= map(select(.id != "auth"))'
reject_graph unresolved-auth '.resolve.nodes |= map(select(.id != "auth"))'
reject_graph wrong-root '.resolve.root = "auth"'
reject_graph live-root '.packages[0].manifest_path = "/live-workspace/Cargo.toml"'
reject_graph extra-local '.packages += [{id: "extra", name: "extra", source: null}] | .resolve.nodes += [{id: "extra"}]'
for forbidden in canic-core toko solana-sdk ic-siws ic-host-fs ic-build ic-auth-qualification-contracts ic-auth-tooling ic-testkit ic-agent ic-platform tokio pocket-ic ic-cdk ic-stable-structures ic-memory ic-timers; do
    reject_graph "$forbidden" ".packages[4].name = \"$forbidden\""
done
cp "$fixture/base-metadata.json" "$fixture/metadata.json"
for field in name version source checksum; do
    TEST_LOCK_FIELD="$field" perl -pe 's/^\Q$ENV{TEST_LOCK_FIELD}\E = ".*"$/$ENV{TEST_LOCK_FIELD} = "unselected"/' \
        "$fixture/base.lock" > "$fixture/Cargo.lock"
    status=0
    "$BASH" scripts/ci/check-package-consumer.sh "$fixture/original.lock" "$fixture" "$version" > "$fixture/lock-$field.log" 2>&1 || status=$?
    [[ "$status" == 1 ]] || { echo "unexpected lock admission: $field" >&2; exit 1; }
done
# The shared reader also refuses ambiguous or malformed lock records.
for mode in duplicate malformed; do
    if [[ "$mode" == duplicate ]]; then
        cp "$fixture/base.lock" "$fixture/Cargo.lock"
        sed -n '/^\[\[package\]\]/,$p' "$fixture/base.lock" >> "$fixture/Cargo.lock"
    else
        printf 'package=[false]\n' > "$fixture/Cargo.lock"
    fi
    status=0
    "$BASH" scripts/ci/check-package-consumer.sh "$fixture/original.lock" "$fixture" "$version" > "$fixture/lock-$mode.log" 2>&1 || status=$?
    [[ "$status" == 1 ]] || { echo "unexpected lock admission: $mode" >&2; exit 1; }
done
printf 'Package identity, isolation and external-lock refusal cases passed on Bash %s\n' "$BASH_VERSION"

source scripts/ci/auth-dependency-policy.sh
for engine in rg jq; do
    while read -r policy package refused; do
        pattern="$(auth_dependency_pattern "$policy")"
        if [[ "$engine" == rg ]]; then
            if printf '%s\n' "$package" | rg -q "$pattern"; then found=yes; else found=no; fi
        else
            if jq -ne --arg package "$package" --arg pattern "$pattern" '$package | test($pattern)' >/dev/null; then found=yes; else found=no; fi
        fi
        [[ "$found" == "$refused" ]] || { echo "dependency policy mismatch: $engine $policy $package" >&2; exit 1; }
    done <<'CASES'
default canic-core yes
default canicular no
default candid yes
default candid_derive yes
deployment candid_parser yes
default ic-testkit yes
default ic-testkit-candid yes
default pocket-ic yes
default pocket-ice no
default ic-agent yes
default ic-agentic no
default tokio yes
default tokiology no
deployment ic-platform yes
deployment ic-agent yes
deployment tokio yes
deployment ic-cdk yes
deployment ic-memory yes
deployment candid no
deployment ic0 no
deployment ic-verify-bls-signature no
capability ic-testkit yes
capability ic-testkit-candid yes
capability pocket-ic yes
capability pocket-ice no
capability ic-auth-qualification-contracts yes
packaged ic-auth-tooling yes
pure candid yes
certificate-free candid yes
session candid yes
passive ic0 yes
capability ic0 no
pure ed25519-dalek yes
pure sha2 no
certificate-free ic-certification yes
session ed25519-dalek no
session k256 yes
packaged ic-testkit yes
packaged pocket-ic yes
packaged ic-host-fs yes
packaged ic-build yes
packaged candid no
CASES
done
if auth_dependency_pattern unknown >/dev/null 2>&1; then exit 1; fi
