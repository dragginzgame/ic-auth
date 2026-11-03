#!/usr/bin/env bash
set -euo pipefail
validation_only=false
case "${1:-}" in
    '') [[ $# == 0 ]] ;;
    --validation) [[ $# == 1 ]]; validation_only=true ;;
    *) echo 'usage: test-tools.sh [--validation]' >&2; exit 2 ;;
esac

# The fixture owns its nested Make selections, including missing-group probes.
# An enclosing release must not force its command-line group into these calls.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES

# Local Git/bare-remote fixtures, real metadata transforms and mocked upload
# transport. No source commit, remote ref or registry effect in the real repo.
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
mkdir -p "$root/target/portable-fixtures"
fixture="$(mktemp -d "$root/target/portable-fixtures/release-tools.XXXXXX")"
echo "Release/publication fixture evidence: $fixture"
# The published owner renders candidates; this suite qualifies Auth's actual
# saved-source selection, coordinated requirements and recovery below.
# Run the normal entrypoints in a fresh shell: a pre-export used to mask a
# production preflight failure. Only substituted installer effects know this path.
editor="$(bash scripts/dev/release-tools.sh check)"
export IC_AUTH_FIXTURE_EDITOR="$editor"
IC_AUTH_REAL_HOST_FILE="$(bash scripts/dev/host-file-tools.sh check)"
export IC_AUTH_REAL_HOST_FILE
source ci/host-file.env
export IC_AUTH_FIXTURE_FILE_VERSION="$HOST_FILE_VERSION"
export IC_AUTH_FIXTURE_HOST_FILE="$fixture/host-file"
cat > "$IC_AUTH_FIXTURE_HOST_FILE" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${FIXTURE_MODE:-}" == marker-source-change && "${1:-}" == create-private ]]; then
    git commit --allow-empty -qm 'Concurrent fixture marker source'
fi
exec "$IC_AUTH_REAL_HOST_FILE" "$@"
SCRIPT
chmod +x "$IC_AUTH_FIXTURE_HOST_FILE"
unset CARGO_RELEASE_METADATA_EDITOR CARGO_RELEASE_METADATA_MAX_BYTES
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
if [[ "$validation_only" == false ]]; then
# Qualify both current feature admission and saved pre-CLI selections without
# installing another executable or depending on today's Cargo metadata.
mkdir -p "$fixture/editor-selection/scripts/dev"
cp "$root/scripts/dev/release-tools.sh" "$fixture/editor-selection/scripts/dev/"
cp "$root/Cargo.toml" "$fixture/editor-selection/Cargo.toml"
printf '[workspace]\n' > "$fixture/pre-cli.toml"
cat > "$fixture/editor-selection/scripts/dev/install-rust-tools.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == *' --lockfile saved.lock --bin ic-build-release-metadata --profile debug'* ]]
if [[ "${IC_AUTH_FIXTURE_WAIT:-no}" == yes ]]; then
    [[ "${@: -2:1}" == --wait-seconds && "${@: -1}" == 60 ]]
    set -- "${@:1:$#-2}"
fi
if [[ "$IC_AUTH_FIXTURE_EXPECT_CLI" == yes ]]; then
    [[ $# == 13 && "${11}" == --features && "${12}" == cli && "${13}" == --check ]]
else
    [[ $# == 11 && "${11}" == --check ]]
fi
exit_status="${IC_AUTH_FIXTURE_STATUS:-0}"
[[ "$exit_status" == 0 ]] || exit "$exit_status"
printf '%s\n' "$IC_AUTH_FIXTURE_EDITOR"
SCRIPT
for profile in current pre-cli; do
    if [[ "$profile" == current ]]; then
        manifest="$fixture/editor-selection/Cargo.toml"; expected=yes
    else
        manifest="$fixture/pre-cli.toml"; expected=no
    fi
    observed="$(IC_AUTH_FIXTURE_EXPECT_CLI="$expected" bash \
        "$fixture/editor-selection/scripts/dev/release-tools.sh" check saved.lock "$manifest")"
    [[ "$observed" == "$editor" ]]
done
observed="$(IC_AUTH_FIXTURE_EXPECT_CLI=yes IC_AUTH_FIXTURE_WAIT=yes bash \
    "$fixture/editor-selection/scripts/dev/release-tools.sh" check-wait saved.lock \
    "$fixture/editor-selection/Cargo.toml")"
[[ "$observed" == "$editor" ]]
status=0
IC_AUTH_FIXTURE_EXPECT_CLI=yes IC_AUTH_FIXTURE_WAIT=yes IC_AUTH_FIXTURE_STATUS=75 bash \
    "$fixture/editor-selection/scripts/dev/release-tools.sh" check-wait saved.lock \
    "$fixture/editor-selection/Cargo.toml" > "$fixture/busy-editor.out" || status=$?
[[ "$status" == 75 && ! -s "$fixture/busy-editor.out" ]]
printf '[workspace.metadata.ic-auth]\nrelease-editor-features = false\n' > "$fixture/invalid-editor-features.toml"
if IC_AUTH_FIXTURE_EXPECT_CLI=yes bash \
    "$fixture/editor-selection/scripts/dev/release-tools.sh" check saved.lock \
    "$fixture/invalid-editor-features.toml" > "$fixture/invalid-editor-features.log" 2>&1; then
    echo 'invalid saved editor features unexpectedly admitted' >&2; exit 1
fi
rg -F 'invalid release editor features' "$fixture/invalid-editor-features.log" > /dev/null
fi
mkdir "$fixture/repo"
# Qualify one coherent working source, including uncommitted additions/deletions.
bash "$root/scripts/ci/verify-shared-tooling-snapshot.sh"
if [[ "$validation_only" == false ]]; then bash "$root/scripts/ci/test-release-fixture-source.sh"; fi
bash "$root/scripts/release/export-fixture-source.sh" "$root" "$fixture/repo"
# Exercise the actual shared Make entrypoints before substituting the larger
# gate below. The opt-in includes and execution probe are explicit inputs.
if [[ "$validation_only" == false ]]; then
TMPDIR="$fixture" bash scripts/ci/check-release-commands.sh "$fixture/repo" \
    ci/tool-versions.env make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk \
    make/execution.mk scripts/ci/check-make-execution.sh \
    scripts/ci/check-release-delivery.sh ci/release-delivery.txt
fi
cd "$fixture/repo"
export RELEASE_GROUP=protocol
# Keep an inherited search path throughout release and publication flows.
# Machine-readable metadata and digest output must not acquire a cd banner.
export CDPATH=".:$fixture"
printf '%s\0' Cargo.toml Cargo.lock releases/protocol/CHANGELOG.md crates/ic-auth-protocol-types/Cargo.toml crates/ic-auth-wire/Cargo.toml release-validation.json > "$fixture/expected-files.nul"
bash scripts/release/metadata.sh files > "$fixture/observed-files.nul"
cmp "$fixture/expected-files.nul" "$fixture/observed-files.nul"
if RELEASE_DELIVERY="pr" bash scripts/release/metadata.sh files \
    > "$fixture/refused-delivery.stdout" 2> "$fixture/refused-delivery.stderr"; then
    echo 'unsupported delivery unexpectedly admitted' >&2
    exit 1
fi
[[ ! -s "$fixture/refused-delivery.stdout" ]]
rg -F 'release delivery refused' "$fixture/refused-delivery.stderr" > /dev/null
# Fixture releases always start at a synthetic 0.1.0, even after the real
# workspace advances. Do not make future CI depend on today's version.
for selected_group in engine protocol; do
    if [[ "$selected_group" == engine ]]; then selected_version=0.9.0; else selected_version=0.1.0; fi
    RELEASE_VERSION="$selected_version" bash "$PWD/scripts/ci/cargo-release-group.sh" plan "$selected_group" > "$fixture/synthetic-plan.json"
    initial="$(jq -r .previous "$fixture/synthetic-plan.json")"
    while IFS= read -r manifest; do
        jq --arg path "$manifest" '.manifests[] | select(.path == $path) | {version,dependencies}' \
            "$fixture/synthetic-plan.json" > "$fixture/edit.json"
        "$editor" --max-input-bytes 8388608 group-manifest "$manifest" "$initial" "$selected_version" \
            "$fixture/edit.json" > "$fixture/manifest"
        cp "$fixture/manifest" "$manifest"
    done < <(jq -r '.manifests[].path' "$fixture/synthetic-plan.json")
    local_packages=()
    while IFS= read -r package; do local_packages+=("$package"); done < <(jq -r '.members[]' "$fixture/synthetic-plan.json")
    "$editor" --max-input-bytes 8388608 lock Cargo.lock "$initial" "$selected_version" "${local_packages[@]}" > "$fixture/lock"
    cp "$fixture/lock" Cargo.lock
done
printf '# Changelog\n\n## [0.1.1]\n\n- Fixture release.\n' > releases/protocol/CHANGELOG.md
RELEASE_VERSION=0.9.1 bash "$PWD/scripts/ci/cargo-release-group.sh" plan engine > "$fixture/engine-before.json"
while IFS= read -r path; do
    mkdir -p "$fixture/unselected-engine/${path%/*}"
    cp "$path" "$fixture/unselected-engine/$path"
done < <(jq -r '.manifests[].path | select(. != "Cargo.toml"), empty' "$fixture/engine-before.json"; printf '%s\n' releases/engine/CHANGELOG.md)
# Substitute external effects at script boundaries; retain the actual Make
# recipes, ordered common/product admission and release routing without overrides.
cat > scripts/ci/run-validation-targets.sh <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ -f target/common-tools-ready && -f target/testkit-ready && -f target/host-file-ready && -f target/release-editor-ready ]] || exit 1
printf 'gate\n' >> target/preparation-dispatch.log
[[ ! -f target/fail-gate ]] || exit 1
echo fixture-complete-gate
# Substitute only stage execution. Keep the real group selector and receipt
# owner, with explicit runner-shaped events/logs rather than a whole-gate PASS.
plan=''
base_inputs=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --input) base_inputs+=("$2"); shift 2 ;;
        --plan) plan=$2; shift 2 ;;
        --fail-fast) shift ;;
        *) exit 2 ;;
    esac
done
[[ -n "$plan" ]]
logs="$VALIDATION_LOG_DIR/validation.fixture"
mkdir -p "$logs"
# Keep Shared's actual plan parser and checkpoint format. Substitute only the
# underlying stage bodies, so the canonical verifier admits the same protocol.
perl scripts/ci/validation-input-checkpoint.pl "$PWD" '' "$logs/inputs.json" \
    scripts/ci/verify-file-checksum.sh "${base_inputs[@]}" "$plan"
perl scripts/ci/release-validation-plan.pl "$PWD" "$plan" "$RELEASE_SOURCE" "$RELEASE_GROUP" \
    "$logs/inputs.json" "$logs/parsed-inputs.json" > "$logs/plan-selection.tsv"
inputs=()
while IFS=$'\t' read -r decision value _reason; do
    [[ "$decision" != input ]] || inputs+=("$value")
done < "$logs/plan-selection.tsv"
perl scripts/ci/validation-input-checkpoint.pl "$PWD" "$logs/parsed-inputs.json" \
    "$logs/plan-inputs.json" scripts/ci/verify-file-checksum.sh "${inputs[@]}"
printf 'target\tstate\treason\n' > "$logs/plan-events.tsv"
printf 'target\tresult\tseconds\tlog\n' > "$logs/timings.tsv"
while IFS=$'\t' read -r decision target reason; do
    case "$decision" in
        required) printf '%s\tblocked\t%s\n' "$target" "$reason" >> "$logs/plan-events.tsv" ;;
        not-applicable) printf '%s\tnot-applicable\t%s\n' "$target" "$reason" >> "$logs/plan-events.tsv" ;;
    esac
done < "$logs/plan-selection.tsv"
index=0
while IFS=$'\t' read -r decision target _reason; do
    [[ "$decision" == required ]] || continue
    log="$PWD/$logs/$index.log"
    printf 'Substituted Auth stage effects\n' > "$log"
    cp "$logs/plan-inputs.json" "$logs/$index-before-inputs.json"
    cp "$logs/plan-inputs.json" "$logs/$index-after-inputs.json"
    printf '%s\tPASS\t%s\n' "$target" "$log" >> "$logs/plan-events.tsv"
    printf '%s\tPASS\t0\t%s\n' "$target" "$log" >> "$logs/timings.tsv"
    index=$((index+1))
done < "$logs/plan-selection.tsv"
SCRIPT
cat > scripts/dev/install-host-tools.sh <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ " $* " == *' --check '* ]]; then
    printf 'common check\n' >> target/preparation-dispatch.log
    [[ ! -f target/fail-common-check && -f target/common-tools-ready ]] || exit 1
else
    printf 'common setup\n' >> target/preparation-dispatch.log
    [[ ! -f target/fail-common-setup ]] || exit 1
    touch target/common-tools-ready
fi
SCRIPT
for script in install-ic-tools msrv-tools; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "scripts/dev/$script.sh"
done
cat > scripts/dev/install-rust-tools.sh <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
# Common Rust effects are already substituted above. Editor selection must use
# the saved source, including when today's root lock cannot be parsed.
if [[ " $* " == *' --package ic-host-fs '* ]]; then
    [[ " $* " == *" --version $IC_AUTH_FIXTURE_FILE_VERSION "* && " $* " == *' --features cli '* ]] || exit 1
    if [[ " $* " == *' --check '* ]]; then
        printf 'file check\n' >> target/preparation-dispatch.log
        [[ ! -f target/fail-file-check && -f target/host-file-ready ]] || exit 1
    else
        printf 'file setup\n' >> target/preparation-dispatch.log
        [[ ! -f target/fail-file-setup ]] || exit 1
        touch target/host-file-ready
    fi
    printf '%s\n' "$IC_AUTH_FIXTURE_HOST_FILE"
    exit 0
fi
[[ " $* " == *' --package ic-build '* ]] || exit 0
lock=''
previous=''
for argument in "$@"; do
    if [[ "$previous" == --lockfile ]]; then lock="$argument"; fi
    previous="$argument"
done
[[ "$lock" == "$PWD/.git/release-state/metadata."*/base.lock && -f "$lock" ]]
git show "$RELEASE_SOURCE:Cargo.lock" | cmp - "$lock"
[[ " $* " == *' --features cli '* ]] || exit 1
if [[ " $* " == *' --check '* ]]; then
    printf 'check\n' >> target/editor-events.log
    [[ ! -f target/editor-busy ]] || exit 75
    [[ ! -f target/fail-editor-check && -f target/release-editor-ready ]] || exit 38
else
    printf 'install\n' >> target/editor-events.log
    [[ ! -f target/fail-editor-setup ]] || exit 37
    touch target/release-editor-ready
fi
printf '%s\n' "$IC_AUTH_FIXTURE_EDITOR"
SCRIPT
cat > scripts/dev/testkit-tools.sh <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    install)
        printf 'cli setup\n' >> target/preparation-dispatch.log
        [[ ! -f target/fail-cli-setup ]] || exit 1
        touch target/testkit-ready ;;
    check)
        printf 'cli check\n' >> target/preparation-dispatch.log
        [[ ! -f target/fail-cli-check && -f target/testkit-ready ]] || exit 1 ;;
    *) exit 2 ;;
esac
SCRIPT
cp scripts/release/metadata.sh scripts/release/fixture-metadata.sh
cat > scripts/release/metadata.sh <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
bash scripts/release/fixture-metadata.sh "$@" || exit "$?"
if [[ "$1" == prepare-version ]]; then
    if [[ -f target/interrupt-prepare ]]; then
        primary="$(jq -r --arg group "$RELEASE_GROUP" '.groups[] | select(.name == $group) | .primary' ci/cargo-release-groups.json)"
        git show HEAD:Cargo.toml > Cargo.toml
        git show "HEAD:crates/$primary/Cargo.toml" > "crates/$primary/Cargo.toml"
        exit 19
    fi
    if [[ -f target/interrupt-prepared ]]; then exit 19; fi
fi
SCRIPT
# Diagnose recipe drift rather than filtering Make stderr. Every nested parse
# uses this same production Makefile and its unchanged canonical includes.
make --no-print-directory help > "$fixture/make-parse.log" 2>&1
if rg 'warning: (overriding recipe|ignoring old recipe)' "$fixture/make-parse.log"; then exit 1; fi
git init -q -b main
git config user.name 'Release fixture'
git config user.email 'release-fixture@example.invalid'
git add .
git commit -qm 'Fixture source'
git init -q --bare "$fixture/remote.git"
git remote add origin "$fixture/remote.git"
git push -q origin main
mkdir target
printf 'retained artifact\n' > target/keep
before="$(git rev-parse HEAD)"
cp releases/protocol/CHANGELOG.md "$fixture/pending-notes"
if [[ "$validation_only" == false ]]; then
# Missing/empty/unknown selections refuse through the actual public entrypoints,
# before setup, the gate, package creation or any Git release effect.
for selected_group in '' unknown; do
    for target in release-version release-patch release-minor release-major release-resume publish publish-dry-run; do
        if make --no-print-directory "$target" RELEASE_GROUP="$selected_group" VERSION=0.1.1 \
            > "$fixture/group-${selected_group:-empty}-$target.log" 2>&1; then
            echo "unselected group unexpectedly admitted: $target/$selected_group" >&2; exit 1
        fi
        [[ ! -e target/preparation-dispatch.log && ! -e .git/publication-state ]]
        [[ "$(git rev-parse HEAD)" == "$before" && -z "$(git tag -l)" ]]
    done
done
if (unset RELEASE_GROUP; make --no-print-directory release-patch) > "$fixture/group-missing.log" 2>&1; then exit 1; fi
if make --no-print-directory release-minor > "$fixture/wrong-candidate.log" 2>&1; then exit 1; fi
[[ "$(git rev-parse HEAD)" == "$before" && -z "$(git tag -l)" ]] || exit 1
# Invalid pending syntax must name the actual heading and its correction before
# setup/fetch/validation, while preserving notes, index and release identity.
cp releases/protocol/CHANGELOG.md "$fixture/pending-notes"
cp .git/index "$fixture/pending-index"
for suffix in ' - Draft' ' - 2026-10-16'; do
    sed "s/^## \[0.1.1\]$/## [0.1.1]$suffix/" "$fixture/pending-notes" > releases/protocol/CHANGELOG.md
    git add releases/protocol/CHANGELOG.md
    git commit -qm 'Commit invalid fixture heading for preflight admission'
    before="$(git rev-parse HEAD)"
    cp .git/index "$fixture/pending-index"
    cp releases/protocol/CHANGELOG.md "$fixture/invalid-notes"
    if make --no-print-directory release-patch > "$fixture/pending-heading.log" 2>&1; then exit 1; fi
    rg -F "expected pending heading '## [0.1.1]'" "$fixture/pending-heading.log" > /dev/null
    rg -F "found '## [0.1.1]$suffix'" "$fixture/pending-heading.log" > /dev/null
    [[ ! -e target/preparation-dispatch.log ]] || exit 1
    cmp releases/protocol/CHANGELOG.md "$fixture/invalid-notes"
    cmp .git/index "$fixture/pending-index"
    [[ "$(git rev-parse HEAD)" == "$before" && -z "$(git tag -l)" ]] || exit 1
done
cp "$fixture/pending-notes" releases/protocol/CHANGELOG.md
git add releases/protocol/CHANGELOG.md
git commit -qm 'Restore valid fixture heading'
before="$(git rev-parse HEAD)"
# Common and CLI setup/admission must precede the gate and preserve release
# identity on either failure. Neither version preparation nor a tag may start.
for phase in common-setup cli-setup file-setup common-check cli-check file-check editor-setup editor-check; do
    : > target/preparation-dispatch.log
    touch "target/fail-$phase"
    if make --no-print-directory release-patch > "$fixture/$phase-failure.log" 2>&1; then exit 1; fi
    [[ "$(git rev-parse HEAD)" == "$before" && "$(make -s release-version)" == 0.1.0 ]] || exit 1
    [[ -z "$(git tag -l)" ]] || exit 1
    case "$phase" in
        common-setup) expected='common setup' ;;
        cli-setup) expected=$'common setup\ncommon setup\ncli setup' ;;
        file-setup) expected=$'common setup\ncommon setup\ncli setup\ncommon setup\nfile setup' ;;
        common-check) expected=$'common setup\ncommon setup\ncli setup\ncommon setup\nfile setup\ncommon check' ;;
        cli-check) expected=$'common setup\ncommon setup\ncli setup\ncommon setup\nfile setup\ncommon check\ncli check' ;;
        file-check|editor-setup|editor-check) expected=$'common setup\ncommon setup\ncli setup\ncommon setup\nfile setup\ncommon check\ncli check\nfile check' ;;
    esac
    [[ "$(cat target/preparation-dispatch.log)" == "$expected" ]] || exit 1
    rm "target/fail-$phase"
done
: > target/preparation-dispatch.log
touch target/fail-gate
if make --no-print-directory release-patch > "$fixture/gate-failure.log" 2>&1; then exit 1; fi
[[ "$(git rev-parse HEAD)" == "$before" && "$(make -s release-version)" == 0.1.0 ]] || exit 1
[[ -z "$(git tag -l)" && -f target/keep ]] || exit 1
[[ "$(cat target/preparation-dispatch.log)" == $'common setup\ncommon setup\ncli setup\ncommon setup\nfile setup\ncommon check\ncli check\nfile check\ngate' ]] || exit 1
rm target/fail-gate
[[ "$(tail -n 2 target/editor-events.log)" == $'install\ncheck' ]] || exit 1

# Normal release source admission refuses a dirty lock before validation or
# preparation, without changing source or index bytes.
printf '\n' >> Cargo.lock
cp Cargo.lock "$fixture/dirty-lock"
cp .git/index "$fixture/dirty-index"
cp target/preparation-dispatch.log "$fixture/dirty-dispatch.log"
if make --no-print-directory release-patch > "$fixture/lock-source.log" 2>&1; then exit 1; fi
rg -F 'commit or restore worktree changes before starting a release' "$fixture/lock-source.log" > /dev/null
if rg -F 'fixture-complete-gate' "$fixture/lock-source.log" > /dev/null; then exit 1; fi
cmp Cargo.lock "$fixture/dirty-lock"
cmp .git/index "$fixture/dirty-index"
cmp target/preparation-dispatch.log "$fixture/dirty-dispatch.log"
[[ "$(git rev-parse HEAD)" == "$before" && -z "$(git tag -l)" ]] || exit 1
git restore Cargo.lock

# A staged edit restored only in the worktree must still be rejected.
printf '\n// unrelated staged source\n' >> crates/ic-auth/src/lib.rs
git add crates/ic-auth/src/lib.rs
git show HEAD:crates/ic-auth/src/lib.rs > crates/ic-auth/src/lib.rs
cp .git/index "$fixture/staged-index"
cp crates/ic-auth/src/lib.rs "$fixture/staged-worktree"
if make --no-print-directory release-patch > "$fixture/staged-source.log" 2>&1; then exit 1; fi
rg -F 'commit or restore staged changes before starting a release' "$fixture/staged-source.log" > /dev/null
cmp .git/index "$fixture/staged-index"
cmp crates/ic-auth/src/lib.rs "$fixture/staged-worktree"
git restore --staged crates/ic-auth/src/lib.rs
fi
# Standard preparation accepts the numbered marker and replaces it with its
# saved UTC date before committing/tagging. Earlier refusal cases cover the bare
# numbered spelling; do not require a manual heading edit for this path.
awk '{ if ($0 == "## [0.1.1]") print "##\t[0.1.1]  -\tUnreleased \t"; else print }' "$fixture/pending-notes" > releases/protocol/CHANGELOG.md
git add releases/protocol/CHANGELOG.md
git commit -qm 'Prepare numbered fixture heading'
make --no-print-directory release-patch > "$fixture/patch.log" 2>&1
[[ "$(make -s release-version)" == 0.1.1 ]] || exit 1
[[ "$(awk '/^## / { print; exit }' releases/protocol/CHANGELOG.md)" == "## [0.1.1] - $(jq -r .date release-validation.json)" ]] || exit 1
commit="$(git rev-parse HEAD)"
bash scripts/ci/check-release-tag.sh "$commit" 0.1.1 "$RELEASE_GROUP"
[[ "$(git --git-dir="$fixture/remote.git" rev-parse main)" == "$commit" ]] || exit 1
[[ "$(git --git-dir="$fixture/remote.git" rev-parse protocol-v0.1.1)" == "$(git rev-parse protocol-v0.1.1)" ]] || exit 1
[[ -f target/keep ]] || exit 1
git show "$before:Cargo.lock" | yq -p toml -o json '.' | jq -S '[.package[] | select(.source != null)]' > "$fixture/external-before.json"
yq -p toml -o json '.' Cargo.lock | jq -S '[.package[] | select(.source != null)]' > "$fixture/external-after.json"
cmp "$fixture/external-before.json" "$fixture/external-after.json"
make --no-print-directory release-resume VERSION=0.1.1 > "$fixture/resume.log" 2>&1
[[ "$(git rev-parse HEAD)" == "$commit" ]] || exit 1

# Fake only package creation/upload and registry reads. Metadata, tag checks,
# clean-worktree admission and publication intent/reconciliation stay real.
mkdir "$fixture/bin"
FIXTURE_WC="$(command -v wc)"
export FIXTURE_WC
# BSD wc pads line counts. Exercise that output on every host, preserving the
# real count while testing publication dispatch/reconciliation numerically.
cat > "$fixture/bin/wc" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
count="$("$FIXTURE_WC" "$@")"
printf '%8s\n' "$count"
SH
actual_cargo="$(command -v cargo)"
export FIXTURE_CARGO="$actual_cargo" FIXTURE_LOG="$fixture" FIXTURE_MODE=unknown
cat > "$fixture/bin/cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    package)
        [[ "$FIXTURE_MODE" != package-failure ]] || exit 101
        mkdir -p "$CARGO_TARGET_DIR/package"
        source scripts/release/packages.sh selected
        for package in "${packages[@]}"; do
            version="$(yq -p toml -o json '.' "crates/$package/Cargo.toml" | jq -r .package.version)"
            printf '%s' "$package" > "$CARGO_TARGET_DIR/package/$package-$version.crate"
        done
        if [[ "$FIXTURE_MODE" == source-change ]]; then git commit --allow-empty -qm 'Concurrent fixture source'; fi
        ;;
    publish)
        [[ $# == 6 && "$2" == --locked && "$3" == --registry && "$4" == crates-io && "$5" == --package ]]
        selector="$6"
        package="${selector%@*}"
        version="$(yq -p toml -o json '.' "crates/$package/Cargo.toml" | jq -r .package.version)"
        [[ "$selector" == "$package@$version" ]]
        case "$RELEASE_GROUP/$package" in protocol/ic-auth-protocol-types|protocol/ic-auth-wire|engine/ic-auth) ;; *) exit 2 ;; esac
        printf '%s\n' "$package" >> "$FIXTURE_LOG/uploads"
        if [[ "$FIXTURE_MODE" == lost-reply ]]; then exit 101; fi
        if [[ "$FIXTURE_MODE" == interrupted ]]; then kill -TERM "$PPID"; exit 143; fi
        touch "$FIXTURE_LOG/observed-$package"
        ;;
    *) exec "$FIXTURE_CARGO" "$@" ;;
esac
SH
cat > "$fixture/bin/curl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == '--disable --version' ]]; then
    if [[ "$FIXTURE_MODE" == old-curl ]]; then printf 'curl 8.3.0\n'; else printf 'curl 8.4.0\n'; fi
    exit 0
fi
output=''
diagnostics=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --output) output="$2"; shift 2 ;;
        --stderr) diagnostics="$2"; shift 2 ;;
        *) url="$1"; shift ;;
    esac
done
[[ -z "$diagnostics" ]] || : > "$diagnostics"
package="$(basename "$(dirname "$url")")"
version="${url##*/}"
checksum="$(bash scripts/dev/run-host-file.sh hash-file "$CARGO_TARGET_DIR/package/$package-$version.crate" 67108864)"
case "$FIXTURE_MODE" in
    unknown) printf 503 ;;
    conflict)
        jq -n --arg crate "$package" --arg version "$version" '{version:{crate:$crate,num:$version,checksum:("0" * 64),yanked:false}}' > "$output"
        printf 200 ;;
    wrong-crate|wrong-version|invalid-checksum|invalid-yanked|multiple-json)
        jq -n --arg crate "$package" --arg version "$version" --arg checksum "$checksum" --arg mode "$FIXTURE_MODE" '
            {version:{crate:$crate,num:$version,checksum:$checksum,yanked:false}} |
            if $mode == "wrong-crate" then .version.crate = "other-crate"
            elif $mode == "wrong-version" then .version.num = "0.1.2"
            elif $mode == "invalid-checksum" then .version.checksum = "wrong"
            elif $mode == "invalid-yanked" then .version.yanked = "false"
            else . end
        ' > "$output"
        if [[ "$FIXTURE_MODE" == multiple-json ]]; then printf '{}\n' >> "$output"; fi
        printf 200 ;;
    malformed) printf '{' > "$output"; printf 200 ;;
    oversized) head -c 1048577 /dev/zero > "$output"; printf 200 ;;
    transport) printf '{' > "$output"; printf 000; exit 28 ;;
    *)
        if [[ -f "$FIXTURE_LOG/observed-$package" ]]; then
            jq -n --arg crate "$package" --arg version "$version" --arg checksum "$checksum" --arg mode "$FIXTURE_MODE" \
                '{version:{crate:$crate,num:$version,checksum:$checksum,yanked:($mode == "yanked")}}' > "$output"
            printf 200
        else printf 404; fi
        ;;
esac
SH
chmod +x "$fixture/bin/cargo" "$fixture/bin/curl" "$fixture/bin/wc"
export PATH="$fixture/bin:$PATH" CARGO_TARGET_DIR="$PWD/target"
if [[ "$validation_only" == false ]]; then
export FIXTURE_MODE=package-failure
# Group-qualified intents cannot evade an uncertain coordinated upload of the
# same crate version. Refuse before even the substituted package operation.
mkdir -p .git/publication-state
printf 'legacy fixture intent\n' > .git/publication-state/0.1.1.json
if bash scripts/release/publish.sh > "$fixture/legacy-intent.log" 2>&1; then exit 1; fi
rg -F 'legacy publication intent exists' "$fixture/legacy-intent.log" > /dev/null
[[ "$(cat .git/publication-state/0.1.1.json)" == 'legacy fixture intent' && ! -e .git/publication-state/lock ]]
mv .git/publication-state/0.1.1.json "$fixture/legacy-intent.retained"
printf 'legacy fixture dispatch\n' > .git/publication-state/0.1.1-ic-auth-protocol-types.dispatched
if bash scripts/release/publish.sh > "$fixture/legacy-dispatch.log" 2>&1; then exit 1; fi
rg -F 'legacy ic-auth-protocol-types upload custody exists' "$fixture/legacy-dispatch.log" > /dev/null
[[ "$(cat .git/publication-state/0.1.1-ic-auth-protocol-types.dispatched)" == 'legacy fixture dispatch' && ! -e .git/publication-state/lock ]]
mv .git/publication-state/0.1.1-ic-auth-protocol-types.dispatched "$fixture/legacy-dispatch.retained"
if bash scripts/release/publish.sh > "$fixture/package-failure.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" && ! -e .git/publication-state/protocol-0.1.1.json ]] || exit 1
export FIXTURE_MODE=unknown
if bash scripts/release/publish.sh > "$fixture/unknown.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]] || exit 1
export FIXTURE_MODE=conflict
if bash scripts/release/publish.sh > "$fixture/conflict.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]] || exit 1
grep -F 'registry checksum conflict' "$fixture/conflict.log" > /dev/null
for mode in wrong-crate wrong-version invalid-checksum invalid-yanked multiple-json malformed oversized transport old-curl; do
    export FIXTURE_MODE="$mode"
    if bash scripts/release/publish.sh > "$fixture/$mode.log" 2>&1; then exit 1; fi
    [[ ! -f "$fixture/uploads" && ! -e .git/publication-state/protocol-0.1.1-ic-auth-protocol-types.dispatched ]] || exit 1
    grep -F 'cannot establish registry state' "$fixture/$mode.log" > /dev/null
done
export FIXTURE_MODE=marker-source-change
if bash scripts/release/publish.sh > "$fixture/marker-source-change.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]] || exit 1
git reset --hard "$commit" > /dev/null
# This isolated fixture observed no dispatch at all. Restore its known-failed
# marker before testing uncertainty separately; never touch real repo intent.
rm .git/publication-state/protocol-0.1.1-ic-auth-protocol-types.dispatched
export FIXTURE_MODE=lost-reply
if bash scripts/release/publish.sh > "$fixture/lost-reply.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" -eq 1 ]] || exit 1
attempt="$(sed -n 's/^Publication evidence: //p' "$fixture/lost-reply.log")"
[[ -d "$attempt" && ! -e .git/publication-state/.cargo-publication.lock ]] || exit 1
printf 'publish\n%s\ncrates-io\nic-auth-protocol-types@0.1.1\n' "$commit" > "$fixture/expected-attempt-intent"
cmp "$attempt/intent" "$fixture/expected-attempt-intent"
printf '%s\0' cargo publish --locked --registry crates-io --package ic-auth-protocol-types@0.1.1 > "$fixture/expected-attempt-arguments"
cmp "$attempt/arguments" "$fixture/expected-attempt-arguments"
[[ "$(cat "$attempt/cargo-status")" == 101 && "$(cat "$attempt/log-status")" == 0 && "$(cat "$attempt/status")" == 101 ]] || exit 1
cp "$attempt/intent" "$fixture/lost-reply-attempt-intent"
cp "$attempt/cargo.log" "$fixture/lost-reply-cargo.log"
# Every observation has separate retained facts, including the pre/post-dispatch
# 404s. Later reconciliation must not overwrite evidence from the lost reply.
grep -F 'Registry observation evidence retained:' "$fixture/lost-reply.log" | \
    sed 's/^Registry observation evidence retained: //' > "$fixture/lost-reply-evidence.paths"
[[ "$(wc -l < "$fixture/lost-reply-evidence.paths")" -eq 2 ]] || exit 1
while IFS= read -r observation; do
    [[ "$(cat "$observation/http-status")" == 404 && "$(cat "$observation/curl-exit")" == 0 ]] || exit 1
    bash scripts/ci/verify-file-checksum.sh --print sha256 "$observation/http-status" >> "$fixture/lost-reply-status.hashes"
done < "$fixture/lost-reply-evidence.paths"
if bash scripts/release/publish.sh > "$fixture/unresolved.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" -eq 1 ]] || exit 1
touch "$fixture/observed-ic-auth-protocol-types"
export FIXTURE_MODE=success
bash scripts/release/publish.sh > "$fixture/publish.log" 2>&1
[[ "$(cat "$fixture/uploads")" == $'ic-auth-protocol-types\nic-auth-wire' ]] || exit 1
bash scripts/release/publish.sh > "$fixture/publish-retry.log" 2>&1
[[ "$(wc -l < "$fixture/uploads")" -eq 2 ]] || exit 1
export FIXTURE_MODE=yanked
bash scripts/release/publish.sh > "$fixture/yanked-reconcile.log" 2>&1
[[ "$(wc -l < "$fixture/uploads")" -eq 2 ]] || exit 1
[[ "$(find .git/publication-state -name intent -type f | wc -l)" -eq 2 ]] || exit 1
cmp "$attempt/intent" "$fixture/lost-reply-attempt-intent"
cmp "$attempt/cargo.log" "$fixture/lost-reply-cargo.log"
# Registry reconciliation and shared interruption custody are separate. Even an
# all-published read-only reconciliation must leave an existing helper lock alone.
mkdir .git/publication-state/.cargo-publication.lock
printf 'fixture owner\n' > .git/publication-state/.cargo-publication.lock/owner
bash scripts/release/publish.sh > "$fixture/published-with-retained-lock.log" 2>&1
[[ -d .git/publication-state/.cargo-publication.lock && "$(wc -l < "$fixture/uploads")" -eq 2 ]] || exit 1
mv .git/publication-state/.cargo-publication.lock "$fixture/reconciled-helper-lock"
while IFS= read -r observation; do
    bash scripts/ci/verify-file-checksum.sh --print sha256 "$observation/http-status" >> "$fixture/lost-reply-status-after.hashes"
done < "$fixture/lost-reply-evidence.paths"
cmp "$fixture/lost-reply-status.hashes" "$fixture/lost-reply-status-after.hashes"

# An actual interrupted shared attempt in a separate disposable checkout. The
# mock Cargo signals only its runner parent; it never uploads or touches the
# real checkout. Registry absence cannot release either recovery obligation.
git clone -q --no-hardlinks "$PWD" "$fixture/interrupted-repo"
(
    cd "$fixture/interrupted-repo"
    export CARGO_TARGET_DIR="$PWD/target" FIXTURE_LOG="$fixture/interrupted-evidence" FIXTURE_MODE=interrupted
    mkdir -p "$FIXTURE_LOG" "$CARGO_TARGET_DIR"
    # The clone has no ignored setup markers. Explicitly prepare the fixture's
    # substituted file/editor/common selections before offline admission.
    touch target/host-file-ready target/release-editor-ready target/common-tools-ready
    if bash scripts/release/publish.sh > "$FIXTURE_LOG/interrupted.log" 2>&1; then exit 1; fi
    [[ "$(wc -l < "$FIXTURE_LOG/uploads")" -eq 1 && ! -e .git/publication-state/lock && -d .git/publication-state/.cargo-publication.lock ]] || exit 1
    attempt="$(sed -n 's/^Publication evidence: //p' "$FIXTURE_LOG/interrupted.log")"
    [[ -f "$attempt/dispatch" && ! -e "$attempt/status" && ! -e "$attempt/cargo-status" ]] || exit 1
    cmp "$attempt/intent" "$fixture/expected-attempt-intent"
    cp .git/publication-state/protocol-0.1.1-ic-auth-protocol-types.dispatched "$FIXTURE_LOG/saved-dispatch"
    if bash scripts/release/publish.sh > "$FIXTURE_LOG/absent-retry.log" 2>&1; then exit 1; fi
    [[ "$(wc -l < "$FIXTURE_LOG/uploads")" -eq 1 ]] || exit 1
    grep -F 'previous ic-auth-protocol-types upload is unresolved' "$FIXTURE_LOG/absent-retry.log" > /dev/null
    touch "$FIXTURE_LOG/observed-ic-auth-protocol-types"
    export FIXTURE_MODE=success
    if bash scripts/release/publish.sh > "$FIXTURE_LOG/confirmed-but-locked.log" 2>&1; then exit 1; fi
    grep -F 'shared publication attempt lock is retained' "$FIXTURE_LOG/confirmed-but-locked.log" > /dev/null
    [[ "$(wc -l < "$FIXTURE_LOG/uploads")" -eq 1 && ! -e .git/publication-state/protocol-0.1.1-ic-auth-wire.dispatched ]] || exit 1
    # Explicit fixture operator disposition, after reviewing the stopped attempt
    # and confirming its exact registry bytes. Preserve custody; no lock stealing,
    # upload replay or automatic recovery is implemented by the actual caller.
    mv .git/publication-state/.cargo-publication.lock "$FIXTURE_LOG/reconciled-helper-lock"
    bash scripts/release/publish.sh > "$FIXTURE_LOG/recovered.log" 2>&1
    [[ "$(cat "$FIXTURE_LOG/uploads")" == $'ic-auth-protocol-types\nic-auth-wire' ]] || exit 1
    [[ ! -e "$attempt/status" && -f "$attempt/dispatch" ]] || exit 1
    cmp "$attempt/intent" "$fixture/expected-attempt-intent"
    cmp .git/publication-state/protocol-0.1.1-ic-auth-protocol-types.dispatched "$FIXTURE_LOG/saved-dispatch"
)
# Already exported in the parent; replace the value after the isolated scenario.
FIXTURE_MODE=source-change
if bash scripts/release/publish.sh > "$fixture/source-change.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" -eq 2 ]] || exit 1
git reset --hard "$commit" > /dev/null
export FIXTURE_MODE=success

# Reject untagged or dirty publication before even querying the registry.
printf '\n' >> README.md
if bash scripts/release/publish.sh > "$fixture/dirty.log" 2>&1; then exit 1; fi
git restore README.md
git tag -d protocol-v0.1.1 > /dev/null
if bash scripts/release/publish.sh > "$fixture/untagged.log" 2>&1; then exit 1; fi
git tag -a protocol-v0.1.1 "$commit" -m 'Release 0.1.1'

export PATH="${PATH#"$fixture/bin:"}"
else
    # A fresh publication clone has no ignored raw gate directory. Keep the
    # actual self-contained record admission, local Git and mocked upload path.
    git clone -q "$PWD" "$fixture/fresh-publication"
    (cd "$fixture/fresh-publication";
        mkdir target;
        touch target/host-file-ready target/release-editor-ready target/common-tools-ready;
        CARGO_TARGET_DIR="$PWD/target" FIXTURE_MODE=success bash scripts/release/publish.sh) > "$fixture/fresh-publication.log" 2>&1
    [[ "$(wc -l < "$fixture/uploads")" -eq 2 ]]
    export PATH="${PATH#"$fixture/bin:"}"
fi
# Late metadata checks must select the old release commit even after later
# source fixes, rather than silently validating against the current HEAD.
printf '\n// Later fixture source fix.\n' >> crates/ic-auth/src/lib.rs
git add crates/ic-auth/src/lib.rs
git commit -qm 'Later fixture source fix'
# Current working manifests are deliberately unsorted: the check must inspect
# the selected committed source instead, without rewriting either source.
cp Cargo.toml "$fixture/before-unsorted-current.toml"
perl -0777 -pe 's/^(serde_bytes = [^\n]*\n)(serde_cbor = [^\n]*\n)/$2$1/m or die "missing selected dependency pair"' \
    Cargo.toml > "$fixture/unsorted-current.toml"
cp "$fixture/unsorted-current.toml" Cargo.toml
# Recovery must keep the saved CLI feature set, not today's edited policy.
sed 's/release-editor-features = \["cli"\]/release-editor-features = ["not-the-saved-feature"]/' \
    Cargo.toml > "$fixture/changed-editor-features.toml"
cp "$fixture/changed-editor-features.toml" Cargo.toml
cp Cargo.toml "$fixture/unsorted-current.toml"
cp Cargo.lock "$fixture/before-broken-current.lock"
printf 'deliberately invalid live lockfile\n' > Cargo.lock
cp Cargo.lock "$fixture/broken-current.lock"
editor_installs="$(awk '/^install$/ { n++ } END { print n+0 }' target/editor-events.log)"
touch target/editor-busy
status=0
RELEASE_PREVIOUS=0.1.0 RELEASE_VERSION=0.1.1 RELEASE_KIND=patch \
RELEASE_DATE="$(jq -r .date release-validation.json)" \
RELEASE_SOURCE="$(jq -r .source release-validation.json)" RELEASE_COMMIT="$commit" \
    bash scripts/release/metadata.sh committed-check > "$fixture/busy-commit-check.log" 2>&1 || status=$?
[[ "$status" == 75 ]]
rg -F 'release metadata busy:' "$fixture/busy-commit-check.log" > /dev/null
cmp "$fixture/unsorted-current.toml" Cargo.toml
cmp "$fixture/broken-current.lock" Cargo.lock
[[ "$(awk '/^install$/ { n++ } END { print n+0 }' target/editor-events.log)" == "$editor_installs" ]]
rm target/editor-busy
RELEASE_PREVIOUS=0.1.0 RELEASE_VERSION=0.1.1 RELEASE_KIND=patch \
RELEASE_DATE="$(jq -r .date release-validation.json)" \
RELEASE_SOURCE="$(jq -r .source release-validation.json)" RELEASE_COMMIT="$commit" \
    bash scripts/release/metadata.sh committed-check > "$fixture/old-commit-check.log" 2>&1
cmp "$fixture/unsorted-current.toml" Cargo.toml
cmp "$fixture/broken-current.lock" Cargo.lock
[[ "$(awk '/^install$/ { n++ } END { print n+0 }' target/editor-events.log)" == "$editor_installs" ]] || exit 1
cp "$fixture/before-broken-current.lock" Cargo.lock
cp "$fixture/before-unsorted-current.toml" Cargo.toml
# Shared owns version arithmetic and generic phase permutations. Auth retains
# one real Protocol release, a partial group install with exact resume, both
# committed-disposition preparations below, and an independent Engine release.
candidate=0.2.0
{ printf '# Changelog\n\n## [%s]\n\n- Fixture release.\n\n' "$candidate"; tail -n +3 releases/protocol/CHANGELOG.md; } > target/notes
cp target/notes releases/protocol/CHANGELOG.md
git add releases/protocol/CHANGELOG.md
git commit -qm 'Prepare interrupted Protocol fixture notes'
touch target/interrupt-prepare
if make --no-print-directory release-minor > "$fixture/interrupted-prepare.log" 2>&1; then exit 1; fi
[[ "$(make -s release-version)" == 0.1.1 && -z "$(git tag -l protocol-v0.2.0)" ]] || exit 1
# Refuse changed original logs while metadata is already partially installed.
receipt=".git/release-state/validation-protocol-$(git rev-parse HEAD)-$candidate.json"
run=$(jq -er '.validation.proof.run' "$receipt")
cp "$run/0.log" "$fixture/retained-stage.saved"
printf 'changed retained evidence\n' >> "$run/0.log"
if make --no-print-directory release-resume VERSION="$candidate" > "$fixture/retained-corruption.log" 2>&1; then exit 1; fi
rg -F 'retained Shared evidence changed' "$fixture/retained-corruption.log" > /dev/null
cp "$fixture/retained-stage.saved" "$run/0.log"
rm target/interrupt-prepare
cp target/preparation-dispatch.log "$fixture/interrupted-dispatch.log"
editor_installs="$(awk '/^install$/ { n++ } END { print n+0 }' target/editor-events.log)"
make --no-print-directory release-resume VERSION="$candidate" > "$fixture/minor.log" 2>&1
cmp target/preparation-dispatch.log "$fixture/interrupted-dispatch.log"
[[ "$(awk '/^install$/ { n++ } END { print n+0 }' target/editor-events.log)" == "$editor_installs" ]] || exit 1
[[ "$(make -s release-version)" == "$candidate" ]] || exit 1
bash scripts/ci/check-release-tag.sh "$(git rev-parse HEAD)" "$candidate" "$RELEASE_GROUP"
[[ -f target/keep ]] || exit 1
# Qualify the consumer's actual local metadata/receipt owner when a maintainer
# commits the disposition of interrupted preparation. Shared 0.26.18 starts
# afresh from that metadata without reading or replaying the historical journal.
if [[ "$validation_only" == false ]]; then
for disposition in restored prepared; do
    base="$(make -s release-version)"
    saved_candidate="$(bash scripts/ci/next-release-version.sh "$base" patch)"
    printf '# Changelog\n\n## [%s]\n\n- Interrupted preparation fixture.\n' "$saved_candidate" > releases/protocol/CHANGELOG.md
    git add releases/protocol/CHANGELOG.md
    git commit -qm 'Prepare interrupted fixture notes'
    saved_source="$(git rev-parse HEAD)"
    state="$(git rev-parse --absolute-git-dir)/release-state"
    if [[ "$disposition" == restored ]]; then marker=interrupt-prepare; else marker=interrupt-prepared; fi
    touch "target/$marker"
    if make --no-print-directory release-patch > "$fixture/$disposition-interrupted.log" 2>&1; then exit 1; fi
    rm "target/$marker"
    old_plan="$state/protocol-$saved_candidate.plan"
    old_receipt="$state/validation-protocol-$saved_source-$saved_candidate.json"
    [[ "$(tail -n 2 "$old_plan")" == $'prepare\nprotocol' && ! -e "$old_plan.files" ]] || exit 1
    cp "$old_plan" "$fixture/$disposition-saved.plan"
    cp "$old_receipt" "$fixture/$disposition-saved-receipt.json"
    printf 'malformed historical journal\n' > "$state/0.0.0.plan"
    cp "$state/0.0.0.plan" "$fixture/malformed-historical.plan"
    if [[ "$disposition" == restored ]]; then
        # Restore the whole partially written roster, then request a different
        # increment. The superseded patch must not control today's selection.
        for path in Cargo.toml Cargo.lock releases/protocol/CHANGELOG.md crates/ic-auth-protocol-types/Cargo.toml crates/ic-auth-wire/Cargo.toml release-validation.json; do
            git show "$saved_source:$path" > "$path"
        done
        kind=minor
    else
        # The fully written local candidate is committed without any runner
        # staging, release commit, tag or push. Fresh validation may advance it.
        kind='patch'
    fi
    base="$(make -s release-version)"
    candidate="$(bash scripts/ci/next-release-version.sh "$base" "$kind")"
    printf '# Changelog\n\n## [%s]\n\n- Committed preparation disposition fixture.\n\n## [%s] - 2026-10-05\n\n- Earlier fixture metadata.\n' "$candidate" "$base" > releases/protocol/CHANGELOG.md
    git add -- Cargo.toml Cargo.lock releases/protocol/CHANGELOG.md crates/ic-auth-protocol-types/Cargo.toml crates/ic-auth-wire/Cargo.toml release-validation.json
    git commit -qm 'Commit fixture metadata disposition'
    current_source="$(git rev-parse HEAD)"
    git merge-base --is-ancestor "$saved_source" "$current_source"
    cp target/preparation-dispatch.log "$fixture/$disposition-before-restart.log"
    # Exact resume remains source-bound. Dirty/untracked normal restart must
    # retain the old intent and refuse before any setup or validation dispatch.
    if make --no-print-directory release-resume VERSION="$saved_candidate" > "$fixture/$disposition-exact-resume.log" 2>&1; then exit 1; fi
    printf 'untracked fixture input\n' > restart-input
    if make --no-print-directory "release-$kind" > "$fixture/$disposition-untracked.log" 2>&1; then exit 1; fi
    rm restart-input
    cmp "$old_plan" "$fixture/$disposition-saved.plan"
    cmp "$old_receipt" "$fixture/$disposition-saved-receipt.json"
    cmp target/preparation-dispatch.log "$fixture/$disposition-before-restart.log"
    [[ ! -e "$state/lock" && "$(git rev-parse HEAD)" == "$current_source" ]] || exit 1
    # Refuse the fresh gate once: ignored historical intent cannot permit
    # reuse of the old source's successful validation receipt.
    touch target/fail-gate
    if make --no-print-directory "release-$kind" > "$fixture/$disposition-fresh-gate-failure.log" 2>&1; then exit 1; fi
    rm target/fail-gate
    cmp "$old_plan" "$fixture/$disposition-saved.plan"
    cmp "$state/0.0.0.plan" "$fixture/malformed-historical.plan"
    [[ ! -e "$state/protocol-$candidate.plan" && "$(make -s release-version)" == "$base" ]] || exit 1
    [[ "$(git rev-parse HEAD)" == "$current_source" && -z "$(git tag -l "protocol-v$candidate")" ]] || exit 1
    git diff --quiet HEAD -- Cargo.toml Cargo.lock releases/protocol/CHANGELOG.md release-validation.json
    git diff --cached --quiet HEAD --
    { cat "$fixture/$disposition-before-restart.log"; printf 'common setup\ncommon setup\ncli setup\ncommon setup\nfile setup\ncommon check\ncli check\nfile check\ngate\n'; } > "$fixture/$disposition-expected-failed-dispatch.log"
    cmp target/preparation-dispatch.log "$fixture/$disposition-expected-failed-dispatch.log"
    # Qualify Auth's newly source-bound preparation and independent prepared
    # check. Generic commit/tag/push permutations belong to Shared; both groups'
    # actual complete delivery remains exercised elsewhere in this fixture.
    remote_before="$(git --git-dir="$fixture/remote.git" rev-parse main)"
    touch target/interrupt-prepared
    if make --no-print-directory "release-$kind" > "$fixture/$disposition-restarted.log" 2>&1; then exit 1; fi
    rm target/interrupt-prepared
    RELEASE_PREVIOUS="$base" RELEASE_VERSION="$candidate" RELEASE_SOURCE="$current_source" \
    RELEASE_KIND="$kind" RELEASE_DATE="$(jq -r .date release-validation.json)" \
        bash scripts/release/metadata.sh prepared-check > "$fixture/$disposition-prepared-check.log" 2>&1
    [[ "$(make -s release-version)" == "$candidate" && -f target/keep ]] || exit 1
    [[ "$(jq -r .source release-validation.json)" == "$current_source" && "$(jq -r .previous release-validation.json)" == "$base" ]] || exit 1
    [[ "$(jq -r .candidate release-validation.json)" == "$candidate" && "$(jq -r .kind release-validation.json)" == "$kind" ]] || exit 1
    [[ "$(git rev-parse HEAD)" == "$current_source" && -z "$(git tag -l "protocol-v$candidate")" ]] || exit 1
    [[ "$(git --git-dir="$fixture/remote.git" rev-parse main)" == "$remote_before" ]] || exit 1
    cmp "$old_plan" "$fixture/$disposition-saved.plan"
    cmp "$state/0.0.0.plan" "$fixture/malformed-historical.plan"
    cmp "$old_receipt" "$fixture/$disposition-saved-receipt.json"
    if [[ "$disposition" == prepared ]]; then
        [[ -z "$(git tag -l "protocol-v$saved_candidate")" ]]
        [[ -z "$(git --git-dir="$fixture/remote.git" tag -l "protocol-v$saved_candidate")" ]]
    fi
    yq -p toml -o json '.' Cargo.lock | jq -S '[.package[] | select(.source != null)]' > "$fixture/$disposition-external.json"
    cmp "$fixture/external-before.json" "$fixture/$disposition-external.json"
    git add -- Cargo.toml Cargo.lock releases/protocol/CHANGELOG.md crates/ic-auth-protocol-types/Cargo.toml crates/ic-auth-wire/Cargo.toml release-validation.json
    git commit -qm 'Retain qualified restarted group metadata'
done
fi
# All Protocol releases above left Engine package versions and notes unchanged.
while IFS= read -r path; do
    cmp "$path" "$fixture/unselected-engine/$path"
done < <(jq -r '.manifests[].path | select(. != "Cargo.toml"), empty' "$fixture/engine-before.json"; printf '%s\n' releases/engine/CHANGELOG.md)
[[ "$(make -s release-version RELEASE_GROUP=engine)" == 0.9.0 ]]

# The user's motivating case: a breaking Engine release must preserve both
# Protocol manifests, their local lock identities and their complete ledger.
for path in crates/ic-auth-protocol-types/Cargo.toml crates/ic-auth-wire/Cargo.toml releases/protocol/CHANGELOG.md; do
    mkdir -p "$fixture/unselected-protocol/${path%/*}"
    cp "$path" "$fixture/unselected-protocol/$path"
done
yq -p toml -o json '.' Cargo.lock | jq -S '[.package[] | select(.name == "ic-auth-protocol-types" or .name == "ic-auth-wire")]' > "$fixture/protocol-lock-before.json"
printf '# Engine changelog\n\n## [0.10.0] - Unreleased\n\n- Breaking Engine-only fixture change.\n' > releases/engine/CHANGELOG.md
git add releases/engine/CHANGELOG.md
git commit -qm 'Prepare Engine fixture notes'
export RELEASE_GROUP=engine
make --no-print-directory release-minor > "$fixture/engine-minor.log" 2>&1
[[ "$(make -s release-version)" == 0.10.0 ]]
bash scripts/ci/check-release-tag.sh "$(git rev-parse HEAD)" 0.10.0 engine
for path in crates/ic-auth-protocol-types/Cargo.toml crates/ic-auth-wire/Cargo.toml releases/protocol/CHANGELOG.md; do
    cmp "$path" "$fixture/unselected-protocol/$path"
done
yq -p toml -o json '.' Cargo.lock | jq -S '[.package[] | select(.name == "ic-auth-protocol-types" or .name == "ic-auth-wire")]' > "$fixture/protocol-lock-after.json"
cmp "$fixture/protocol-lock-before.json" "$fixture/protocol-lock-after.json"
make --no-print-directory release-resume VERSION=0.10.0 > "$fixture/engine-resume.log" 2>&1
jq -e '.group == "engine" and .schema == 4 and .gate == "release-plan"' release-validation.json > /dev/null
export PATH="$fixture/bin:$PATH" FIXTURE_MODE=success
bash scripts/release/publish.sh > "$fixture/engine-publish.log" 2>&1
[[ "$(cat "$fixture/uploads")" == $'ic-auth-protocol-types\nic-auth-wire\nic-auth' ]]
bash scripts/release/publish.sh > "$fixture/engine-publish-retry.log" 2>&1
[[ "$(wc -l < "$fixture/uploads")" -eq 3 ]]
jq -e '.group == "engine" and [.packages[].name] == ["ic-auth"]' .git/publication-state/engine-0.10.0.json > /dev/null
if RELEASE_GROUP=protocol bash scripts/release/publish.sh > "$fixture/wrong-group-publication.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" -eq 3 ]]
echo 'Both release groups, unselected identities/notes, failure admission, exact tags/push, resume and publication reconciliation passed (local/mocked effects)'
