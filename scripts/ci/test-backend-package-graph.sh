#!/usr/bin/env bash
# Exercise Auth's real graph admission against mutations of observed Cargo data.
set -euo pipefail
[[ $# == 5 ]] || { echo 'usage: test-backend-package-graph.sh METADATA BUNDLE_MANIFEST ENGINE_VERSION PROTOCOL_VERSION EVIDENCE' >&2; exit 2; }
metadata="$1" bundle="$2" version="$3" protocol_version="$4" evidence="$5"
mkdir -m 700 "$evidence"
checker="${BASH_SOURCE[0]%/*}/check-backend-package-graph.sh"
bash "$checker" "$metadata" "$bundle" "$version" "$protocol_version"
reject() {
    local name="$1" filter="$2" status=0
    jq "$filter" "$metadata" > "$evidence/$name.json"
    bash "$checker" "$evidence/$name.json" "$bundle" "$version" "$protocol_version" > "$evidence/$name.log" 2>&1 || status=$?
    [[ "$status" == 1 ]] || { echo "backend graph mutation was not refused: $name" >&2; exit 1; }
}
reject local-auth '(.packages[] | select(.name == "ic-auth")).source = null'
reject wrong-cohort '(.packages[] | select(.name == "ic-auth")).version = "0.0.1"'
reject wrong-protocol '(.packages[] | select(.name == "ic-auth-protocol-types")).version = "0.0.1"'
reject duplicate-auth '.packages += [.packages[] | select(.name == "ic-auth")]'
reject workspace-bundle '.workspace_members += [.packages[] | select(.name == "ic-auth-backend-conformance") | .id]'
reject live-bundle '(.packages[] | select(.name == "ic-auth-backend-conformance")).manifest_path = "/live/Cargo.toml"'
reject local-dependency '(.packages[] | select(.name == "ic-auth-backend-conformance") | .dependencies[0]).path = "/live/auth"'
reject private-fixtures '.packages += [{name:"ic-auth-qualification-fixtures",source:"registry+https://github.com/rust-lang/crates.io-index"}]'
reject duplicate-platform '.packages += [.packages[] | select(.name == "ic-platform-types")]'
for forbidden in dhat ic-platform-runtime-guard ic-certification canic_core; do
    reject "$forbidden" ".packages += [{name:\"$forbidden\",source:\"registry+https://github.com/rust-lang/crates.io-index\"}]"
done
printf 'Foreign-source, cohort, workspace, private-graph and profiler refusals passed\n'
