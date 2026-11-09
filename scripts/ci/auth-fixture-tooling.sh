#!/usr/bin/env bash
# Auth caller fixtures substitute admission only after exporting intact source.
# The --tooling release cases retain the real declaration/lock/snapshot proof.
# Scenario-specific sealed-input fault injection is appended by its own fixture.
auth_fixture_tooling_check() {
    [[ $# == 1 && -d "$1/scripts/dev" ]] || return 2
    cat > "$1/scripts/dev/toko-tooling.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "${1:-}" == check ]]
SCRIPT
    cat > "$1/scripts/ci/check-qualification-workspace.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
SCRIPT
}
