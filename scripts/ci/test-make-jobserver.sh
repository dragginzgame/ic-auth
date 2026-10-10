#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/make-jobserver.XXXXXX")"
printf 'Make jobserver evidence retained: %s\n' "$fixture"
consumer="$fixture/consumer with spaces"
mkdir -p "$consumer/make" "$consumer/ci" "$consumer/scripts/dev" \
    "$consumer/scripts/ci" "$consumer/scripts/release" "$fixture/bin"
for input in Makefile make/tools.mk make/release.mk make/rust-format.mk make/execution.mk \
    ci/tool-versions.env scripts/ci/check-make-execution.sh; do
    cp -p "$input" "$consumer/$input"
done
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
SCRIPT
chmod +x "$fixture/bin/cargo"
ln -s "$BASH" "$fixture/bin/bash"
export PATH="$fixture/bin:$PATH"
# Substitute application/installation/publication effects, keeping actual root
# recipes, includes, execution admission and Bash-to-Cargo descriptor handoff.
for script in dev/testkit-tools.sh dev/msrv-tools.sh dev/check-msrv.sh \
    dev/test-qualification.sh dev/client-contracts.sh dev/test-client.sh \
    ci/check-auth-boundaries.sh ci/read-cargo-workspace-version.sh \
    release/publish.sh release/metadata.sh release/test-tools.sh; do
    cat > "$consumer/scripts/$script" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
exec cargo "$@"
SCRIPT
done
cat >> "$consumer/Makefile" <<'MAKE'

install-host-tools check-package-licenses:
	@:
MAKE
options=(--no-print-directory -j4)
if make --help | rg -q -- --jobserver-style; then
    options+=(--jobserver-style=pipe)
fi
targets=(fetch metadata test-types test-protocol test-signatures test-signature-store \
    test-tokens test-sessions check-wasm clippy build-qualification-canister \
    test-qualification test-host-tooling check-boundaries check-msrv \
    install-testkit-tools testkit-tools-check msrv-tools-check \
    generate-client-contracts check-client-contracts test-client publish-dry-run publish test-release-tools \
    release-version release-preflight release-prepare-version release-prepared-check release-files \
    release-commit-check release-committed-check release-tagged-check release-push-check release-verify)
for target in "${targets[@]}"; do
    : > "$AUTH_JOBSERVER_LOG"
    (cd "$consumer"; make "${options[@]}" "$target") > "$fixture/$target.log" 2>&1
    [[ -s "$AUTH_JOBSERVER_LOG" ]]
done

# Recursive marking must never make unsafe modes execute these effects, even
# when callers erase MAKEFLAGS. The shared independent guard stays authoritative.
for mode in -n -t -q -i; do
    for target in fetch metadata install-testkit-tools generate-client-contracts publish test-release-tools release-prepare-version; do
        : > "$AUTH_JOBSERVER_LOG"
        status=0
        (cd "$consumer"; make "$mode" --no-print-directory "$target" MAKEFLAGS=) \
            > "$fixture/refused-$mode-$target.log" 2>&1 || status=$?
        [[ "$status" != 0 && ! -s "$AUTH_JOBSERVER_LOG" ]]
    done
done
printf 'Actual Make Cargo descriptor handoff and unsafe-mode refusal verified (substituted effects)\n'
