#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
# This fixture selects its own controller roster/publication cases, including
# when the enclosing real release has explicitly selected registry delivery.
unset RELEASE_GROUPS RELEASE_PUBLISH_TARGET RELEASE_SETUP_MODE
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/make-jobserver.XXXXXX")"
printf 'Make jobserver evidence retained: %s\n' "$fixture"
consumer="$fixture/consumer with spaces"
mkdir -p "$consumer" "$fixture/bin"
bash scripts/release/export-fixture-source.sh "$root" "$consumer"
export AUTH_JOBSERVER_LOG="$fixture/cargo-effects.log"
cat > "$fixture/bin/cargo" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
# Record attempts even if descriptor admission fails, so unsafe-mode checks
# cannot mistake a failed Cargo dispatch for refusal before dispatch.
printf '%s\n' "$*" >> "$AUTH_JOBSERVER_LOG"
# Check actual inherited descriptors, rather than trusting advertised flags.
# GNU Make 3.81 uses fds; newer versions use auth. Force pipe style below.
perl -e '
    ($ENV{MAKEFLAGS} // "") =~ /--jobserver-(?:auth|fds)=(\d+),(\d+)/
        or die "missing inherited pipe jobserver\n";
    open my $reader, "<&=$1" or die "closed jobserver reader: $!\n";
    open my $writer, ">&=$2" or die "closed jobserver writer: $!\n";
'
if [[ "${1:-}" == --consumer ]]; then
    [[ $# == 3 && ( "$2" == "$PWD" || "$2" == "$PWD/qualification" ) && "$3" == --external-ic ]]
fi
if [[ "${1:-}" == metadata ]]; then
    # The license caller now queries metadata. This fixture checks descriptor
    # inheritance only; real package/license admission has its own focused check.
    printf '%s\n' '{"packages":[{"name":"fixture","manifest_path":"Cargo.toml","license_file":"LICENSE.md","license":null}]}'
    exit 0
fi
if [[ "${1:-}" == publish ]]; then
    [[ "$2" == --dry-run && "$3" == --locked &&
       "$4" == --registry && "$5" == crates-io && "$6" == --allow-dirty &&
       "$7" == --target-dir && "$9" == -p ]]
    case "$RELEASE_GROUP" in
        protocol) [[ $# == 12 && "${10}" == ic-auth-protocol-types && "${11}" == -p && "${12}" == ic-auth-wire ]] ;;
        engine) [[ $# == 10 && "${10}" == ic-auth ]] ;;
        *) exit 2 ;;
    esac
    staging="$8"
    [[ "$staging" == "$PWD/target/publication-dry-runs/"* &&
       -d "$staging" && ! -e "$staging/dispatched" ]]
    touch "$staging/dispatched"
    printf '%s\n' "$staging" >> "$AUTH_JOBSERVER_LOG.paths"
    exit "${AUTH_JOBSERVER_PUBLISH_STATUS:-0}"
fi
if [[ "${1:-}" == package && "${3:-}" == --registry ]]; then
    [[ $# == 13 && "$2" == --locked && "$3" == --registry && "$4" == crates-io &&
       "$5" == --allow-dirty && "$6" == --target-dir && "$8" == -p &&
       "$9" == ic-auth-protocol-types && "${10}" == -p && "${11}" == ic-auth-wire &&
       "${12}" == -p && "${13}" == ic-auth ]]
    staging="$7"
    [[ "$staging" == "$PWD/target/package-qualification/"* &&
       -d "$staging" && ! -e "$staging/dispatched" ]]
    touch "$staging/dispatched"
    printf '%s\n' "$staging" >> "$AUTH_JOBSERVER_LOG.paths"
    exit "${AUTH_JOBSERVER_PACKAGE_STATUS:-0}"
fi
SCRIPT
chmod +x "$fixture/bin/cargo"
ln -s "$BASH" "$fixture/bin/bash"
export PATH="$fixture/bin:$PATH"
# Substitute application/installation/publication effects, keeping actual root
# recipes, includes, execution admission and Bash-to-Cargo descriptor handoff.
for script in dev/testkit-tools.sh dev/msrv-tools.sh dev/host-file-tools.sh dev/check-msrv.sh dev/check-wasm.sh \
    dev/test-qualification.sh dev/client-contracts.sh dev/test-client.sh dev/test-client-browser.sh dev/package-client.sh dev/release-tools.sh \
    dev/profile-canonical-allocations.sh dev/test-backend-package.sh \
    ci/check-cargo-dependency-boundaries.sh ci/test-auth-boundaries.sh \
    ci/check-auth-boundaries.sh ci/test-client-tools.sh ci/check-dependency-pins.sh ci/test-auth-validation.sh ci/test-release-validation.sh \
    release/publish.sh release/metadata.sh release/candidate-setup.sh release/test-tools.sh release/validation-stage.sh; do
    cat > "$consumer/scripts/$script" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
exec cargo "$@"
SCRIPT
done
printf '#!/usr/bin/env bash\nexit 0\n' > "$consumer/scripts/dev/install-host-tools.sh"
# This fixture substitutes snapshot-owned installers and release effects. Root
# tooling admission has its own intact-payload proof in test-tools.sh --tooling;
# source cleanup cannot run against this synthetic non-Git fixture.
source "$root/scripts/ci/auth-fixture-tooling.sh"
auth_fixture_tooling_check "$consumer"
cat > "$consumer/scripts/dev/clean-empty-source-dirs.pl" <<'SCRIPT'
print "Source cleanup substituted for jobserver qualification.\n";
SCRIPT
# Admission does not invoke Cargo; its ordinary recipe need not inherit the
# jobserver descriptors. Qualification still exercises its actual Cargo recipes.
cat > "$consumer/scripts/dev/client-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == check ]]
SCRIPT
options=(--no-print-directory -j4)
if make --help | rg -q -- --jobserver-style; then
    options+=(--jobserver-style=pipe)
fi
# Root recipes marked recursive must pass the descriptors. Derive coverage from
# those recipes so adding a Cargo target cannot silently omit it from this test.
# The full gate and Git-hook fixture intentionally have their own harnesses.
awk '
    {
        line=line $0
        if (sub(/\\$/, "", line)) next
        print line; line=""
    }
' "$consumer/Makefile" | awk '
    /^[^[:space:]#][^:]*:/ { targets="" }
    /^[[:alnum:]_-]+([[:space:]]+[[:alnum:]_-]+)*:/ {
        targets=$0; sub(/:.*/, "", targets)
    }
    /^\t[@+-]*[+]/ {
        count=split(targets, names, /[[:space:]]+/)
        for (i=1; i<=count; i++)
            # Workspace admission is substituted without Cargo here; its real
            # metadata/file-plan execution has a separate consumer fixture.
            if (names[i] != "ci" && names[i] != "test-formatting-hooks" && names[i] != "check-qualification-workspace") print names[i]
    }
' | sort -u > "$fixture/targets"
# Pattern rules are not ordinary target declarations. Expand the release wrapper
# through its canonical roster and check the real Make-to-script FD handoff.
jq -r '.stages[]' "$consumer/ci/release-validation.json" >> "$fixture/targets"
sort -u "$fixture/targets" > "$fixture/sorted-targets"
mv "$fixture/sorted-targets" "$fixture/targets"
# Guard the parser against accidentally dropping known multiline/normal recipes.
rg -Fx test-portable-signatures "$fixture/targets" > /dev/null
rg -Fx test-client-browser "$fixture/targets" > /dev/null
rg -Fx test-client-package "$fixture/targets" > /dev/null
rg -Fx profile-canonical-allocations "$fixture/targets" > /dev/null
rg -Fx test-backend-package "$fixture/targets" > /dev/null
rg -Fx check-package-builds "$fixture/targets" > /dev/null
rg -Fx release-prepare-version "$fixture/targets" > /dev/null
while IFS= read -r target; do
    : > "$AUTH_JOBSERVER_LOG"
    status=0
    (cd "$consumer"; make "${options[@]}" "$target" RELEASE_GROUP=protocol) > "$fixture/$target.log" 2>&1 || status=$?
    if [[ "$status" != 0 ]]; then
        printf 'Make jobserver target failed: %s (status %s); log: %s\n' \
            "$target" "$status" "$fixture/$target.log" >&2
        cat "$fixture/$target.log" >&2
        exit "$status"
    fi
    [[ -s "$AUTH_JOBSERVER_LOG" ]]
    if [[ "$target" == check-boundaries ]]; then
        rg -Fx -- "--consumer $consumer --external-ic" "$AUTH_JOBSERVER_LOG" > /dev/null
        rg -Fx -- "--consumer $consumer/qualification --external-ic" "$AUTH_JOBSERVER_LOG" > /dev/null
    fi
done < "$fixture/targets"

# Auth's declared delivery policy refuses unsupported modes before adapter dispatch.
: > "$AUTH_JOBSERVER_LOG"
if (cd "$consumer"; make --no-print-directory RELEASE_DELIVERY=pr release-version) \
    > "$fixture/refused-delivery.log" 2>&1; then
    echo 'unsupported delivery unexpectedly dispatched' >&2
    exit 1
fi
[[ ! -s "$AUTH_JOBSERVER_LOG" ]]
rg -F 'release delivery refused' "$fixture/refused-delivery.log" > /dev/null

# Every dry run needs a fresh temporary-registry identity, even before a version
# bump. Keep earlier evidence and propagate Cargo refusal through the recipe.
mkdir -p "$consumer/target/package"
printf 'retained staging cache\n' > "$consumer/target/package/keep"
(cd "$consumer"; make "${options[@]}" publish-dry-run RELEASE_GROUP=protocol) > "$fixture/publish-repeat.log" 2>&1
status=0
(cd "$consumer"; AUTH_JOBSERVER_PUBLISH_STATUS=37 make "${options[@]}" publish-dry-run RELEASE_GROUP=engine) \
    > "$fixture/publish-refusal.log" 2>&1 || status=$?
[[ "$status" == 2 ]]
rg -F 'Error 37' "$fixture/publish-refusal.log" > /dev/null
status=0
(cd "$consumer"; AUTH_JOBSERVER_PACKAGE_STATUS=37 make "${options[@]}" check-package-builds) \
    > "$fixture/package-refusal.log" 2>&1 || status=$?
[[ "$status" == 2 ]]
rg -F 'Error 37' "$fixture/package-refusal.log" > /dev/null
[[ "$(wc -l < "$AUTH_JOBSERVER_LOG.paths" | tr -d '[:space:]')" == 5 ]]
[[ "$(sort -u "$AUTH_JOBSERVER_LOG.paths" | wc -l | tr -d '[:space:]')" == 5 ]]
while IFS= read -r staging; do [[ -f "$staging/dispatched" ]]; done < "$AUTH_JOBSERVER_LOG.paths"
[[ "$(cat "$consumer/target/package/keep")" == 'retained staging cache' ]]

# Harmless discovery remains usable without executing admission probes.
(cd "$consumer"; make -np --no-print-directory help MAKEFLAGS=) \
    > "$fixture/discovery-help.log" 2>&1
if rg -q '^tasks:' "$fixture/discovery-help.log"; then
    echo 'Producer-only tasks target must not be installed in Auth' >&2
    exit 1
fi

# Recursive marking must never make unsafe modes execute these effects, even
# when callers erase MAKEFLAGS. The shared independent guard stays authoritative.
for mode in -n -t -q -i; do
    for target in fetch metadata check-dependency-pins install-testkit-tools install-host-file generate-client-contracts test-client-browser profile-canonical-allocations test-backend-package check-package-builds publish-dry-run publish test-release-tools release-prepare-version; do
        : > "$AUTH_JOBSERVER_LOG"
        status=0
        (cd "$consumer"; make "$mode" --no-print-directory "$target" MAKEFLAGS=) \
            > "$fixture/refused-$mode-$target.log" 2>&1 || status=$?
        [[ "$status" != 0 && ! -s "$AUTH_JOBSERVER_LOG" ]]
        # Question mode can stop at a nonrecursive prerequisite before the guard.
        if [[ "$mode" != -q ]]; then
            rg -F 'Toko Tooling requires Make recipe execution and failure propagation' \
                "$fixture/refused-$mode-$target.log" > /dev/null
        fi
    done
done
# Exercise Auth's actual includes and context policy with the frozen controller
# substituted at its process boundary. No release source or remote is touched.
export AUTH_FROZEN_ROUTING_LOG="$fixture/frozen-routing.jsonl"
cat > "$consumer/scripts/ci/run-release-batch.pl" <<'PERL'
use strict;
use warnings;
use JSON::PP;
die "controller inherited Cargo target\n" if exists $ENV{CARGO_TARGET_DIR};
open my $log, '>>', $ENV{AUTH_FROZEN_ROUTING_LOG} or die $!;
print {$log} JSON::PP->new->encode(\@ARGV), "\n";
close $log or die $!;
exit($ENV{AUTH_FROZEN_ROUTING_STATUS} // 0);
PERL
cat > "$consumer/scripts/ci/run-release.sh" <<'SCRIPT'
#!/usr/bin/env bash
echo 'selected group escaped frozen capture' >&2
exit 97
SCRIPT
for kind in patch minor major; do
    while IFS= read -r group; do
        env -u CARGO_TARGET_DIR -u RELEASE_BATCH_ACTIVE make -C "$consumer" \
            --no-print-directory "release-$kind" RELEASE_GROUP="$group" \
            RELEASE_REMOTE=routing RELEASE_BRANCH=routing > "$fixture/routing-$kind-$group.log" 2>&1
        tail -n 1 "$AUTH_FROZEN_ROUTING_LOG" | jq -e --arg root "$consumer" --arg group "$group" --arg kind "$kind" '
            def value($key): . as $args | index($key) as $i | $args[$i + 1];
            value("--root") == $root and value("--groups") == $group and value("--kind") == $kind
            and value("--remote") == "routing" and value("--branch") == "routing"
            and value("--setup-target") == "release-candidate-setup"
            and index("--publish-target") == null' > /dev/null
    done < <(jq -r '.groups[].name' "$consumer/ci/cargo-release-groups.json")
done
cp "$AUTH_FROZEN_ROUTING_LOG" "$fixture/routing-before-refusal.jsonl"
if env -u CARGO_TARGET_DIR -u RELEASE_BATCH_ACTIVE make -C "$consumer" --no-print-directory \
    release-patch RELEASE_GROUP=engine RELEASE_GROUPS= > "$fixture/refused-empty-roster.log" 2>&1; then
    echo 'empty roster unexpectedly bypassed frozen capture' >&2; exit 1
fi
rg -F 'A selected release group requires batch capture' "$fixture/refused-empty-roster.log" > /dev/null
cmp "$AUTH_FROZEN_ROUTING_LOG" "$fixture/routing-before-refusal.jsonl"
for retired in RELEASE_FROZEN_GROUPS RELEASE_FROZEN_PUBLISH_TARGET RELEASE_FROZEN_SETUP_TARGET RELEASE_FROZEN_SETUP_MODE FROZEN_RELEASE_ACTIVE; do
    if env -u CARGO_TARGET_DIR -u RELEASE_BATCH_ACTIVE make -C "$consumer" --no-print-directory \
        release-patch RELEASE_GROUP=engine "$retired=" > "$fixture/refused-$retired.log" 2>&1; then
        echo "retired setting unexpectedly admitted: $retired" >&2; exit 1
    fi
    rg -F "Retired release settings: $retired" "$fixture/refused-$retired.log" > /dev/null
done
cmp "$AUTH_FROZEN_ROUTING_LOG" "$fixture/routing-before-refusal.jsonl"
env -u CARGO_TARGET_DIR -u RELEASE_BATCH_ACTIVE make -C "$consumer" --no-print-directory \
    release-patch RELEASE_GROUPS=protocol,engine RELEASE_PUBLISH_TARGET=publish \
    > "$fixture/routing-multiple.log" 2>&1
tail -n 1 "$AUTH_FROZEN_ROUTING_LOG" | jq -e '
    . as $args | $args[index("--groups") + 1] == "protocol,engine"
    and $args[index("--publish-target") + 1] == "publish"' > /dev/null
if rg 'warning: (overriding recipe|ignoring old recipe)' "$fixture"/*.log; then exit 1; fi
printf 'Actual Make Cargo descriptor handoff and unsafe-mode refusal verified (substituted effects)\n'
printf 'Automatic single-group frozen capture, empty-roster refusal and explicit multi-group publication routing verified\n'
