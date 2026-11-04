#!/usr/bin/env bash
# Auth caller wiring only; Shared owns generic budget/traversal regressions.
set -euo pipefail
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
mkdir -p target/portable-fixtures
evidence="$(mktemp -d "$root/target/portable-fixtures/auth-retention.XXXXXX")"
printf 'Auth retention caller evidence: %s\n' "$evidence"
mkdir "$evidence/repo" "$evidence/tmp" "$evidence/bin"
export TMPDIR="$evidence/tmp"
bash scripts/release/export-fixture-source.sh "$root" "$evidence/repo"
cd "$evidence/repo"
git init -q
git config user.name 'Auth retention fixture'
git config user.email 'auth-fixture@example.invalid'
git add .
git commit -qm 'Current Auth retention caller fixture'
export RELEASE_GROUP=engine RELEASE_KIND=patch RELEASE_DELIVERY=direct
source scripts/release/group.sh
RELEASE_PREVIOUS="$(auth_release_version)"
RELEASE_VERSION="$(bash scripts/ci/next-release-version.sh "$RELEASE_PREVIOUS" patch)"
RELEASE_DATE="$(date -u +%F)"
export RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE
RELEASE_SOURCE="$(git rev-parse HEAD)"
export RELEASE_SOURCE
# Admit GNU Make's execution probe but refuse any actual release hook. A broken
# early guard must fail this case without acquiring delivery authority.
IC_AUTH_RETENTION_REAL_MAKE="$(command -v make)"
export IC_AUTH_RETENTION_REAL_MAKE
cat > "$evidence/bin/make" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == '--no-print-directory -f - MAKEFILES=' ]]; then
    exec "$IC_AUTH_RETENTION_REAL_MAKE" "$@"
fi
exit 97
SH
chmod +x "$evidence/bin/make"
git remote add origin "$evidence/unused-remote.git"
mkdir -p .git/publication-state/retained/target
printf 'runner evidence\n' > .git/publication-state/retained/target/receipt
status=0
RELEASE_MAKE="$evidence/bin/make" bash scripts/ci/run-release.sh patch origin \
    "$(git symbolic-ref --short HEAD)" > "$evidence/runner-refused.log" 2>&1 || status=$?
[[ "$status" == 1 ]]
rg -F 'retained state requires review before release' "$evidence/runner-refused.log" > /dev/null
[[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" && ! -e .git/release-state/lock ]]
[[ -f .git/publication-state/retained/target/receipt ]]
rm -rf .git/publication-state/retained
# Each direct writer must refuse before setup/packaging/registry effects, keeping
# old evidence intact. No full gate, upload or remote transport is exercised.
for writer in metadata publish; do
    mkdir -p .git/publication-state/retained/target
    printf 'recovery evidence\n' > .git/publication-state/retained/target/receipt
    status=0
    bash "scripts/release/$writer.sh" retention-probe > "$evidence/$writer-refused.log" 2>&1 || status=$?
    [[ "$status" == 1 ]]
    rg -F 'build/cache/checkout tree belongs outside retained state' "$evidence/$writer-refused.log" > /dev/null
    [[ "$(cat .git/publication-state/retained/target/receipt)" == 'recovery evidence' ]]
    [[ ! -e target/release-metadata && ! -e .git/publication-state/lock ]]
    # This is disposable, deliberately invalid fixture data, not real recovery.
    rm -rf .git/publication-state/retained
done
status=0
bash scripts/release/metadata.sh retention-probe > "$evidence/metadata-scratch.log" 2>&1 || status=$?
[[ "$status" == 1 ]]
rg -F 'unknown adapter operation: retention-probe' "$evidence/metadata-scratch.log" > /dev/null
scratch=(target/release-metadata/metadata.*/source)
[[ ${#scratch[@]} == 1 && -f "${scratch[0]}/Cargo.toml" ]]
shopt -s nullglob
git_scratch=(.git/release-state/metadata.*)
[[ ${#git_scratch[@]} == 0 ]]
source scripts/release/retention.sh
auth_release_retention
# Simulate a failed adapter whose external command also leaves forbidden state.
# The exit guard must report the retention failure without hiding status 73.
IC_AUTH_RETENTION_REAL_GIT="$(command -v git)"
export IC_AUTH_RETENTION_REAL_GIT
cat > "$evidence/bin/git" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
"$IC_AUTH_RETENTION_REAL_GIT" "$@"
if [[ "$1" == archive ]]; then
    mkdir -p .git/publication-state/failed/target
    printf 'failed adapter evidence\n' > .git/publication-state/failed/target/receipt
    exit 73
fi
SH
chmod +x "$evidence/bin/git"
status=0
PATH="$evidence/bin:$PATH" bash scripts/release/metadata.sh retention-probe > "$evidence/metadata-failure.log" 2>&1 || status=$?
[[ "$status" == 73 ]]
rg -F 'build/cache/checkout tree belongs outside retained state' "$evidence/metadata-failure.log" > /dev/null
[[ -f .git/publication-state/failed/target/receipt ]]
printf 'Runner/direct writer refusal, target-owned scratch and original failure preservation passed\n'
