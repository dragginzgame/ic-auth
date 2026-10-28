#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
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
# Check actual inherited descriptors, rather than trusting advertised flags.
# GNU Make 3.81 uses fds; newer versions use auth. Force pipe style below.
perl -e '
    ($ENV{MAKEFLAGS} // "") =~ /--jobserver-(?:auth|fds)=(\d+),(\d+)/
        or die "missing inherited pipe jobserver\n";
    open my $reader, "<&=$1" or die "closed jobserver reader: $!\n";
    open my $writer, ">&=$2" or die "closed jobserver writer: $!\n";
'
printf '%s\n' "$*" >> "$AUTH_JOBSERVER_LOG"
if [[ "${1:-}" == --consumer ]]; then
    [[ $# == 3 && "$2" == "$PWD" && "$3" == --external-ic ]]
fi
if [[ "${1:-}" == publish ]]; then
    [[ $# == 14 && "$2" == --dry-run && "$3" == --locked &&
       "$4" == --registry && "$5" == crates-io && "$6" == --allow-dirty &&
       "$7" == --target-dir && "$9" == -p && "${10}" == ic-auth-protocol-types &&
       "${11}" == -p && "${12}" == ic-auth && "${13}" == -p && "${14}" == ic-auth-wire ]]
    staging="$8"
    [[ "$staging" == "$PWD/target/publication-dry-runs/"* &&
       -d "$staging" && ! -e "$staging/dispatched" ]]
    touch "$staging/dispatched"
    printf '%s\n' "$staging" >> "$AUTH_JOBSERVER_LOG.paths"
    exit "${AUTH_JOBSERVER_PUBLISH_STATUS:-0}"
fi
SCRIPT
chmod +x "$fixture/bin/cargo"
ln -s "$BASH" "$fixture/bin/bash"
export PATH="$fixture/bin:$PATH"
# Substitute application/installation/publication effects, keeping actual root
# recipes, includes, execution admission and Bash-to-Cargo descriptor handoff.
for script in dev/testkit-tools.sh dev/msrv-tools.sh dev/host-file-tools.sh dev/check-msrv.sh dev/check-wasm.sh \
    dev/test-qualification.sh dev/client-contracts.sh dev/test-client.sh dev/test-client-browser.sh dev/package-client.sh dev/release-tools.sh \
    ci/check-cargo-dependency-boundaries.sh ci/test-auth-boundaries.sh \
    ci/check-auth-boundaries.sh ci/test-client-tools.sh ci/check-dependency-pins.sh ci/read-cargo-workspace-version.sh \
    release/publish.sh release/metadata.sh release/test-tools.sh; do
    cat > "$consumer/scripts/$script" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
exec cargo "$@"
SCRIPT
done
printf '#!/usr/bin/env bash\nexit 0\n' > "$consumer/scripts/dev/install-host-tools.sh"
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
    /^[[:alnum:]_-]+([[:space:]]+[[:alnum:]_-]+)*:/ {
        targets=$0; sub(/:.*/, "", targets)
    }
    /^\t[@+-]*[+]/ {
        count=split(targets, names, /[[:space:]]+/)
        for (i=1; i<=count; i++)
            if (names[i] != "ci" && names[i] != "test-formatting-hooks") print names[i]
    }
' | sort -u > "$fixture/targets"
# Guard the parser against accidentally dropping known multiline/normal recipes.
rg -Fx test-portable-signatures "$fixture/targets" > /dev/null
rg -Fx test-client-browser "$fixture/targets" > /dev/null
rg -Fx test-client-package "$fixture/targets" > /dev/null
rg -Fx release-prepare-version "$fixture/targets" > /dev/null
while IFS= read -r target; do
    : > "$AUTH_JOBSERVER_LOG"
    status=0
    (cd "$consumer"; make "${options[@]}" "$target") > "$fixture/$target.log" 2>&1 || status=$?
    if [[ "$status" != 0 ]]; then
        printf 'Make jobserver target failed: %s (status %s); log: %s\n' \
            "$target" "$status" "$fixture/$target.log" >&2
        cat "$fixture/$target.log" >&2
        exit "$status"
    fi
    [[ -s "$AUTH_JOBSERVER_LOG" ]]
    if [[ "$target" == check-boundaries ]]; then
        rg -Fx -- "--consumer $consumer --external-ic" "$AUTH_JOBSERVER_LOG" > /dev/null
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
(cd "$consumer"; make "${options[@]}" publish-dry-run) > "$fixture/publish-repeat.log" 2>&1
status=0
(cd "$consumer"; AUTH_JOBSERVER_PUBLISH_STATUS=37 make "${options[@]}" publish-dry-run) \
    > "$fixture/publish-refusal.log" 2>&1 || status=$?
[[ "$status" == 2 ]]
rg -F 'Error 37' "$fixture/publish-refusal.log" > /dev/null
[[ "$(wc -l < "$AUTH_JOBSERVER_LOG.paths" | tr -d '[:space:]')" == 3 ]]
[[ "$(sort -u "$AUTH_JOBSERVER_LOG.paths" | wc -l | tr -d '[:space:]')" == 3 ]]
while IFS= read -r staging; do [[ -f "$staging/dispatched" ]]; done < "$AUTH_JOBSERVER_LOG.paths"
[[ "$(cat "$consumer/target/package/keep")" == 'retained staging cache' ]]

# Recursive marking must never make unsafe modes execute these effects, even
# when callers erase MAKEFLAGS. The shared independent guard stays authoritative.
for mode in -n -t -q -i; do
    for target in fetch metadata check-dependency-pins install-testkit-tools install-host-file generate-client-contracts test-client-browser publish-dry-run publish test-release-tools release-prepare-version; do
        : > "$AUTH_JOBSERVER_LOG"
        status=0
        (cd "$consumer"; make "$mode" --no-print-directory "$target" MAKEFLAGS=) \
            > "$fixture/refused-$mode-$target.log" 2>&1 || status=$?
        [[ "$status" != 0 && ! -s "$AUTH_JOBSERVER_LOG" ]]
    done
done
if rg 'warning: (overriding recipe|ignoring old recipe)' "$fixture"/*.log; then exit 1; fi
printf 'Actual Make Cargo descriptor handoff and unsafe-mode refusal verified (substituted effects)\n'
