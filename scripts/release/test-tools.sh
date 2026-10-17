#!/usr/bin/env bash
set -euo pipefail

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
# The shared renderer owns only workspace.package.version. Auth retains the
# exact local dependency selector, with no partial output on either refusal.
cat > "$fixture/render input.toml" <<'TOML'
[workspace.package]
version = "0.1.0" # preserve this comment
[workspace.dependencies]
ic-auth-protocol-types = { version = "0.1.0", path = "crates/ic-auth-protocol-types" }
[package.metadata.unrelated]
version = "0.1.0"
TOML
cat > "$fixture/render expected.toml" <<'TOML'
[workspace.package]
version = "0.1.1" # preserve this comment
[workspace.dependencies]
ic-auth-protocol-types = { version = "0.1.1", path = "crates/ic-auth-protocol-types" }
[package.metadata.unrelated]
version = "0.1.0"
TOML
cp "$fixture/render input.toml" "$fixture/render original.toml"
perl scripts/release/rewrite-manifest.pl "$fixture/render input.toml" 0.1.0 0.1.1 > "$fixture/render actual.toml"
cmp "$fixture/render expected.toml" "$fixture/render actual.toml"
cmp "$fixture/render original.toml" "$fixture/render input.toml"
for refusal in missing duplicate wrong-version wrong-workspace; do
    case "$refusal" in
        missing) sed '/^ic-auth-protocol-types =/d' "$fixture/render input.toml" ;;
        duplicate) cat "$fixture/render input.toml"; sed -n '/^ic-auth-protocol-types =/p' "$fixture/render input.toml" ;;
        wrong-version) sed '/^ic-auth-protocol-types =/s/0.1.0/0.0.9/' "$fixture/render input.toml" ;;
        wrong-workspace) sed '/^version =.*# preserve/s/0.1.0/0.0.9/' "$fixture/render input.toml" ;;
    esac > "$fixture/render refused.toml"
    cp "$fixture/render refused.toml" "$fixture/render refused-original.toml"
    if perl scripts/release/rewrite-manifest.pl "$fixture/render refused.toml" 0.1.0 0.1.1 \
        > "$fixture/render refused-output" 2> "$fixture/render refused-error"; then
        echo "manifest renderer unexpectedly accepted $refusal" >&2
        exit 1
    fi
    [[ ! -s "$fixture/render refused-output" ]]
    cmp "$fixture/render refused-original.toml" "$fixture/render refused.toml"
done
# The optional third selector preserves historical releases that predate Wire.
# When present, it must move with both other package identities or refuse whole.
for case_name in input expected; do
    wire_version=0.1.0
    [[ "$case_name" != expected ]] || wire_version=0.1.1
    awk -v version="$wire_version" '
        /^\[package.metadata.unrelated\]/ {
            printf "ic-auth-wire = { version = \"%s\", path = \"crates/ic-auth-wire\" }\n", version
        }
        { print }
    ' "$fixture/render $case_name.toml" > "$fixture/wire-$case_name.toml"
done
perl scripts/release/rewrite-manifest.pl "$fixture/wire-input.toml" 0.1.0 0.1.1 > "$fixture/wire-actual.toml"
cmp "$fixture/wire-expected.toml" "$fixture/wire-actual.toml"
sed '/^ic-auth-wire =/s/0.1.0/0.0.9/' "$fixture/wire-input.toml" > "$fixture/wire-refused.toml"
if perl scripts/release/rewrite-manifest.pl "$fixture/wire-refused.toml" 0.1.0 0.1.1 \
    > "$fixture/wire-refused-output" 2> "$fixture/wire-refused-error"; then
    echo 'manifest renderer accepted a mismatched Wire selector' >&2
    exit 1
fi
[[ ! -s "$fixture/wire-refused-output" ]]
# Exercise the actual shared Make entrypoints before substituting the larger
# gate below. The opt-in includes and execution probe are explicit inputs.
TMPDIR="$fixture" bash scripts/ci/check-release-commands.sh "$root" \
    ci/tool-versions.env make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk \
    make/execution.mk scripts/ci/check-make-execution.sh
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
mkdir "$fixture/repo"
git -C "$root" archive HEAD | tar -xf - -C "$fixture/repo"
# Qualify the actual adopted snapshot, including additions not committed yet.
bash "$root/scripts/ci/verify-shared-tooling-snapshot.sh"
while IFS=$'\t' read -r record _digest _mode path; do
    [[ "$record" == file ]] || continue
    mkdir -p "$fixture/repo/$(dirname "$path")"
    cp -p "$root/$path" "$fixture/repo/$path"
done < "$root/.shared-tooling.snapshot"
# Use the complete working package trees so package moves and import changes
# are tested together, rather than mixing current manifests with archived code.
rm -rf "$fixture/repo/crates"
cp -Rp "$root/crates" "$fixture/repo/crates"
rm -rf "$fixture/repo/apps"
cp -Rp "$root/apps" "$fixture/repo/apps"
# Include the independent client build root, never its prepared dependencies.
mkdir -p "$fixture/repo/packages"
rm -rf "$fixture/repo/packages/client"
tar -C "$root/packages" --exclude=node_modules --exclude=dist -cf - client | tar -xf - -C "$fixture/repo/packages"
overlays=(.gitignore Makefile Cargo.toml Cargo.lock CHANGELOG.md .shared-tooling.snapshot
    scripts/release/metadata.sh scripts/release/publish.sh scripts/release/test-tools.sh
    scripts/release/rewrite-manifest.pl
    scripts/dev/run-host-tooling.sh
    scripts/dev/client-tools.sh scripts/dev/client-contracts.sh scripts/dev/test-client.sh
    scripts/dev/prepare-client-ci.sh scripts/ci/test-client-tools.sh .github/workflows/ci.yml
    scripts/ci/test-artifact-discovery.sh
    scripts/dev/test-qualification.sh scripts/dev/testkit-tools.sh
    scripts/ci/test-testkit-tools.sh
    scripts/dev/msrv-tools.sh scripts/dev/check-msrv.sh
    scripts/ci/check-release-tag.sh scripts/ci/rewrite-local-lock-versions.pl)
for path in "${overlays[@]}"; do
    mkdir -p "$fixture/repo/$(dirname "$path")"
    cp -p "$root/$path" "$fixture/repo/$path"
done
cd "$fixture/repo"
# Keep an inherited search path throughout release and publication flows.
# Machine-readable metadata and digest output must not acquire a cd banner.
export CDPATH=".:$fixture"
printf '%s\0' Cargo.toml Cargo.lock CHANGELOG.md release-validation.json > "$fixture/expected-files.nul"
bash scripts/release/metadata.sh files > "$fixture/observed-files.nul"
cmp "$fixture/expected-files.nul" "$fixture/observed-files.nul"
# Fixture releases always start at a synthetic 0.1.0, even after the real
# workspace advances. Do not make future CI depend on today's version.
initial="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
if [[ "$initial" != 0.1.0 ]]; then
    perl scripts/release/rewrite-manifest.pl Cargo.toml "$initial" 0.1.0 > "$fixture/manifest"
    yq -p toml -o json '.' Cargo.lock | jq -r '.package[] | select(.source == null) | .name' > "$fixture/local-packages"
    local_packages=()
    while IFS= read -r package; do local_packages+=("$package"); done < "$fixture/local-packages"
    perl scripts/ci/rewrite-local-lock-versions.pl Cargo.lock "$initial" 0.1.0 "${local_packages[@]}" > "$fixture/lock"
    cp "$fixture/manifest" Cargo.toml
    cp "$fixture/lock" Cargo.lock
fi
printf '# Changelog\n\n## [0.1.1]\n\n- Fixture release.\n' > CHANGELOG.md
# Substitute external effects at script boundaries; retain the actual Make
# recipes, ordered common/product admission and release routing without overrides.
cat > scripts/ci/run-validation-targets.sh <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ -f target/common-tools-ready && -f target/testkit-ready ]] || exit 1
printf 'gate\n' >> target/preparation-dispatch.log
[[ ! -f target/fail-gate ]] || exit 1
echo fixture-complete-gate
SCRIPT
cat > scripts/dev/install-host-tools.sh <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${*: -1}" == --check ]]; then
    printf 'common check\n' >> target/preparation-dispatch.log
    [[ ! -f target/fail-common-check && -f target/common-tools-ready ]] || exit 1
else
    printf 'common setup\n' >> target/preparation-dispatch.log
    [[ ! -f target/fail-common-setup ]] || exit 1
    touch target/common-tools-ready
fi
SCRIPT
for script in install-ic-tools install-rust-tools msrv-tools; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "scripts/dev/$script.sh"
done
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
    if [[ -f target/interrupt-prepare ]]; then git show HEAD:Cargo.toml > Cargo.toml; exit 19; fi
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
if make --no-print-directory release-minor > "$fixture/wrong-candidate.log" 2>&1; then exit 1; fi
[[ "$(git rev-parse HEAD)" == "$before" && -z "$(git tag -l)" ]] || exit 1
# Invalid pending syntax must name the actual heading and its correction before
# setup/fetch/validation, while preserving notes, index and release identity.
cp CHANGELOG.md "$fixture/pending-notes"
cp .git/index "$fixture/pending-index"
for suffix in ' - Unreleased' ' - 2026-10-16'; do
    sed "s/^## \[0.1.1\]$/## [0.1.1]$suffix/" "$fixture/pending-notes" > CHANGELOG.md
    cp CHANGELOG.md "$fixture/invalid-notes"
    if make --no-print-directory release-patch > "$fixture/pending-heading.log" 2>&1; then exit 1; fi
    rg -F "expected undated pending heading '## [0.1.1]'" "$fixture/pending-heading.log" > /dev/null
    rg -F "found '## [0.1.1]$suffix'" "$fixture/pending-heading.log" > /dev/null
    [[ ! -e target/preparation-dispatch.log ]] || exit 1
    cmp CHANGELOG.md "$fixture/invalid-notes"
    cmp .git/index "$fixture/pending-index"
    [[ "$(git rev-parse HEAD)" == "$before" && -z "$(git tag -l)" ]] || exit 1
done
cp "$fixture/pending-notes" CHANGELOG.md
# Common and CLI setup/admission must precede the gate and preserve release
# identity on either failure. Neither version preparation nor a tag may start.
for phase in common-setup cli-setup common-check cli-check; do
    : > target/preparation-dispatch.log
    touch "target/fail-$phase"
    if make --no-print-directory release-patch > "$fixture/$phase-failure.log" 2>&1; then exit 1; fi
    [[ "$(git rev-parse HEAD)" == "$before" && "$(make -s release-version)" == 0.1.0 ]] || exit 1
    [[ -z "$(git tag -l)" ]] || exit 1
    case "$phase" in
        common-setup) expected='common setup' ;;
        cli-setup) expected=$'common setup\ncommon setup\ncommon setup\ncli setup' ;;
        common-check) expected=$'common setup\ncommon setup\ncommon setup\ncli setup\ncommon check' ;;
        cli-check) expected=$'common setup\ncommon setup\ncommon setup\ncli setup\ncommon check\ncli check' ;;
    esac
    [[ "$(cat target/preparation-dispatch.log)" == "$expected" ]] || exit 1
    rm "target/fail-$phase"
done
: > target/preparation-dispatch.log
touch target/fail-gate
if make --no-print-directory release-patch > "$fixture/gate-failure.log" 2>&1; then exit 1; fi
[[ "$(git rev-parse HEAD)" == "$before" && "$(make -s release-version)" == 0.1.0 ]] || exit 1
[[ -z "$(git tag -l)" && -f target/keep ]] || exit 1
[[ "$(cat target/preparation-dispatch.log)" == $'common setup\ncommon setup\ncommon setup\ncli setup\ncommon check\ncli check\ngate' ]] || exit 1
rm target/fail-gate

# Initial admission names a changed lock and refuses before validation or
# preparation, without changing source or index bytes.
printf '\n' >> Cargo.lock
cp Cargo.lock "$fixture/dirty-lock"
cp .git/index "$fixture/dirty-index"
cp target/preparation-dispatch.log "$fixture/dirty-dispatch.log"
if make --no-print-directory release-patch > "$fixture/lock-source.log" 2>&1; then exit 1; fi
rg -F 'unstaged: Cargo.lock' "$fixture/lock-source.log" > /dev/null
rg -F 'this attempt has not started validation or version preparation' "$fixture/lock-source.log" > /dev/null
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
rg -F 'staged: crates/ic-auth/src/lib.rs' "$fixture/staged-source.log" > /dev/null
rg -F 'unstaged: crates/ic-auth/src/lib.rs' "$fixture/staged-source.log" > /dev/null
rg -F 'this attempt has not started validation or version preparation' "$fixture/staged-source.log" > /dev/null
cmp .git/index "$fixture/staged-index"
cmp crates/ic-auth/src/lib.rs "$fixture/staged-worktree"
git restore --staged crates/ic-auth/src/lib.rs
make --no-print-directory release-patch > "$fixture/patch.log" 2>&1
[[ "$(make -s release-version)" == 0.1.1 ]] || exit 1
commit="$(git rev-parse HEAD)"
bash scripts/ci/check-release-tag.sh "$commit" 0.1.1
[[ "$(git --git-dir="$fixture/remote.git" rev-parse main)" == "$commit" ]] || exit 1
[[ "$(git --git-dir="$fixture/remote.git" rev-parse v0.1.1)" == "$(git rev-parse v0.1.1)" ]] || exit 1
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
    run)
        if [[ "$FIXTURE_MODE" == marker-source-change && " $* " == *' create-private '* ]]; then
            git commit --allow-empty -qm 'Concurrent fixture marker source'
        fi
        exec "$FIXTURE_CARGO" "$@"
        ;;
    package)
        mkdir -p "$CARGO_TARGET_DIR/package"
        printf types > "$CARGO_TARGET_DIR/package/ic-auth-protocol-types-0.1.1.crate"
        printf auth > "$CARGO_TARGET_DIR/package/ic-auth-0.1.1.crate"
        printf wire > "$CARGO_TARGET_DIR/package/ic-auth-wire-0.1.1.crate"
        if [[ "$FIXTURE_MODE" == source-change ]]; then git commit --allow-empty -qm 'Concurrent fixture source'; fi
        ;;
    publish)
        package="${*: -1}"
        printf '%s\n' "$package" >> "$FIXTURE_LOG/uploads"
        if [[ "$FIXTURE_MODE" == lost-reply ]]; then exit 101; fi
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
checksum="$(bash scripts/dev/run-host-tooling.sh hash-file "$CARGO_TARGET_DIR/package/$package-0.1.1.crate" 67108864)"
case "$FIXTURE_MODE" in
    unknown) printf 503 ;;
    conflict)
        jq -n --arg crate "$package" '{version:{crate:$crate,num:"0.1.1",checksum:("0" * 64),yanked:false}}' > "$output"
        printf 200 ;;
    wrong-crate|wrong-version|invalid-checksum|invalid-yanked|multiple-json)
        jq -n --arg crate "$package" --arg checksum "$checksum" --arg mode "$FIXTURE_MODE" '
            {version:{crate:$crate,num:"0.1.1",checksum:$checksum,yanked:false}} |
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
            jq -n --arg crate "$package" --arg checksum "$checksum" --arg mode "$FIXTURE_MODE" \
                '{version:{crate:$crate,num:"0.1.1",checksum:$checksum,yanked:($mode == "yanked")}}' > "$output"
            printf 200
        else printf 404; fi
        ;;
esac
SH
chmod +x "$fixture/bin/cargo" "$fixture/bin/curl" "$fixture/bin/wc"
export PATH="$fixture/bin:$PATH" CARGO_TARGET_DIR="$PWD/target"
if bash scripts/release/publish.sh > "$fixture/unknown.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]] || exit 1
export FIXTURE_MODE=conflict
if bash scripts/release/publish.sh > "$fixture/conflict.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]] || exit 1
grep -F 'registry checksum conflict' "$fixture/conflict.log" > /dev/null
for mode in wrong-crate wrong-version invalid-checksum invalid-yanked multiple-json malformed oversized transport old-curl; do
    export FIXTURE_MODE="$mode"
    if bash scripts/release/publish.sh > "$fixture/$mode.log" 2>&1; then exit 1; fi
    [[ ! -f "$fixture/uploads" && ! -e .git/publication-state/0.1.1-ic-auth-protocol-types.dispatched ]] || exit 1
    grep -F 'cannot establish registry state' "$fixture/$mode.log" > /dev/null
done
export FIXTURE_MODE=marker-source-change
if bash scripts/release/publish.sh > "$fixture/marker-source-change.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]] || exit 1
git reset --hard "$commit" > /dev/null
# This isolated fixture observed no dispatch at all. Restore its known-failed
# marker before testing uncertainty separately; never touch real repo intent.
rm .git/publication-state/0.1.1-ic-auth-protocol-types.dispatched
export FIXTURE_MODE=lost-reply
if bash scripts/release/publish.sh > "$fixture/lost-reply.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" -eq 1 ]] || exit 1
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
[[ "$(cat "$fixture/uploads")" == $'ic-auth-protocol-types\nic-auth\nic-auth-wire' ]] || exit 1
bash scripts/release/publish.sh > "$fixture/publish-retry.log" 2>&1
[[ "$(wc -l < "$fixture/uploads")" -eq 3 ]] || exit 1
export FIXTURE_MODE=yanked
bash scripts/release/publish.sh > "$fixture/yanked-reconcile.log" 2>&1
[[ "$(wc -l < "$fixture/uploads")" -eq 3 ]] || exit 1
while IFS= read -r observation; do
    bash scripts/ci/verify-file-checksum.sh --print sha256 "$observation/http-status" >> "$fixture/lost-reply-status-after.hashes"
done < "$fixture/lost-reply-evidence.paths"
cmp "$fixture/lost-reply-status.hashes" "$fixture/lost-reply-status-after.hashes"
export FIXTURE_MODE=source-change
if bash scripts/release/publish.sh > "$fixture/source-change.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" -eq 3 ]] || exit 1
git reset --hard "$commit" > /dev/null
export FIXTURE_MODE=success

# Reject untagged or dirty publication before even querying the registry.
printf '\n' >> README.md
if bash scripts/release/publish.sh > "$fixture/dirty.log" 2>&1; then exit 1; fi
git restore README.md
git tag -d v0.1.1 > /dev/null
if bash scripts/release/publish.sh > "$fixture/untagged.log" 2>&1; then exit 1; fi
git tag -a v0.1.1 "$commit" -m 'Release 0.1.1'

export PATH="${PATH#"$fixture/bin:"}"
# Late metadata checks must select the old release commit even after later
# source fixes, rather than silently validating against the current HEAD.
printf '\n// Later fixture source fix.\n' >> crates/ic-auth/src/lib.rs
git add crates/ic-auth/src/lib.rs
git commit -qm 'Later fixture source fix'
RELEASE_PREVIOUS=0.1.0 RELEASE_VERSION=0.1.1 RELEASE_KIND=patch \
RELEASE_DATE="$(jq -r .date release-validation.json)" \
RELEASE_SOURCE="$(jq -r .source release-validation.json)" RELEASE_COMMIT="$commit" \
    bash scripts/release/metadata.sh committed-check > "$fixture/old-commit-check.log" 2>&1
for kind in minor major; do
    if [[ "$kind" == minor ]]; then candidate=0.2.0; else candidate=1.0.0; fi
    { printf '# Changelog\n\n## [%s]\n\n- Fixture release.\n\n' "$candidate"; tail -n +3 CHANGELOG.md; } > target/notes
    cp target/notes CHANGELOG.md
    git add CHANGELOG.md
    git commit -qm "Prepare fixture $kind notes"
    if [[ "$kind" == minor ]]; then
        touch target/interrupt-prepare
        if make --no-print-directory release-minor > "$fixture/interrupted-prepare.log" 2>&1; then exit 1; fi
        [[ "$(make -s release-version)" == 0.1.1 && -z "$(git tag -l v0.2.0)" ]] || exit 1
        rm target/interrupt-prepare
        cp target/preparation-dispatch.log "$fixture/interrupted-dispatch.log"
    fi
    make --no-print-directory "release-$kind" > "$fixture/$kind.log" 2>&1
    if [[ "$kind" == minor ]]; then
        cmp target/preparation-dispatch.log "$fixture/interrupted-dispatch.log"
    fi
    [[ "$(make -s release-version)" == "$candidate" ]] || exit 1
    bash scripts/ci/check-release-tag.sh "$(git rev-parse HEAD)" "$candidate"
done
[[ -f target/keep ]] || exit 1
# Qualify the consumer's actual local metadata/receipt owner when a maintainer
# commits the disposition of interrupted preparation. The runner must archive
# the old intent and validate the current source before preparing again.
for disposition in restored prepared; do
    base="$(make -s release-version)"
    saved_candidate="$(bash scripts/ci/next-release-version.sh "$base" patch)"
    printf '# Changelog\n\n## [%s]\n\n- Interrupted preparation fixture.\n' "$saved_candidate" > CHANGELOG.md
    git add CHANGELOG.md
    git commit -qm 'Prepare interrupted fixture notes'
    saved_source="$(git rev-parse HEAD)"
    state="$(git rev-parse --absolute-git-dir)/release-state"
    if [[ "$disposition" == restored ]]; then marker=interrupt-prepare; else marker=interrupt-prepared; fi
    touch "target/$marker"
    if make --no-print-directory release-patch > "$fixture/$disposition-interrupted.log" 2>&1; then exit 1; fi
    rm "target/$marker"
    old_plan="$state/$saved_candidate.plan"
    old_receipt="$state/validation-$saved_source-$saved_candidate.json"
    [[ "$(tail -n 1 "$old_plan")" == prepare && ! -e "$old_plan.files" ]] || exit 1
    cp "$old_plan" "$fixture/$disposition-saved.plan"
    cp "$old_receipt" "$fixture/$disposition-saved-receipt.json"
    if [[ "$disposition" == restored ]]; then
        # Restore the whole partially written roster, then request a different
        # increment. The superseded patch must not control today's selection.
        for path in Cargo.toml Cargo.lock CHANGELOG.md release-validation.json; do
            git show "$saved_source:$path" > "$path"
        done
        kind=minor
    else
        # The fully written local candidate is committed without any runner
        # staging, release commit, tag or push. Increment from that metadata.
        kind='patch'
    fi
    base="$(make -s release-version)"
    candidate="$(bash scripts/ci/next-release-version.sh "$base" "$kind")"
    printf '# Changelog\n\n## [%s]\n\n- Committed preparation disposition fixture.\n\n## [%s] - 2026-10-05\n\n- Earlier fixture metadata.\n' "$candidate" "$base" > CHANGELOG.md
    git add -- Cargo.toml Cargo.lock CHANGELOG.md release-validation.json
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
    # Refuse the fresh gate once: archiving intent cannot imply preparation or
    # permit reuse of the old source's successful validation receipt.
    touch target/fail-gate
    if make --no-print-directory "release-$kind" > "$fixture/$disposition-fresh-gate-failure.log" 2>&1; then exit 1; fi
    rm target/fail-gate
    set -- "$state/$saved_candidate".attempt.*/"$saved_candidate.plan"
    [[ $# == 1 && -f "$1" && ! -L "$1" ]] || exit 1
    archive_plan="$1"
    cmp "$archive_plan" "$fixture/$disposition-saved.plan"
    [[ ! -e "$old_plan" && ! -e "$state/$candidate.plan" && "$(make -s release-version)" == "$base" ]] || exit 1
    [[ "$(git rev-parse HEAD)" == "$current_source" && -z "$(git tag -l "v$candidate")" ]] || exit 1
    git diff --quiet HEAD -- Cargo.toml Cargo.lock CHANGELOG.md release-validation.json
    git diff --cached --quiet HEAD --
    { cat "$fixture/$disposition-before-restart.log"; printf 'common setup\ncommon setup\ncommon setup\ncli setup\ncommon check\ncli check\ngate\n'; } > "$fixture/$disposition-expected-failed-dispatch.log"
    cmp target/preparation-dispatch.log "$fixture/$disposition-expected-failed-dispatch.log"
    make --no-print-directory "release-$kind" > "$fixture/$disposition-restarted.log" 2>&1
    [[ "$(make -s release-version)" == "$candidate" && -f target/keep ]] || exit 1
    [[ "$(jq -r .source release-validation.json)" == "$current_source" && "$(jq -r .previous release-validation.json)" == "$base" ]] || exit 1
    [[ "$(jq -r .candidate release-validation.json)" == "$candidate" && "$(jq -r .kind release-validation.json)" == "$kind" ]] || exit 1
    bash scripts/ci/check-release-tag.sh "$(git rev-parse HEAD)" "$candidate"
    [[ "$(git --git-dir="$fixture/remote.git" rev-parse main)" == "$(git rev-parse HEAD)" ]] || exit 1
    cmp "$archive_plan" "$fixture/$disposition-saved.plan"
    cmp "$old_receipt" "$fixture/$disposition-saved-receipt.json"
    yq -p toml -o json '.' Cargo.lock | jq -S '[.package[] | select(.source != null)]' > "$fixture/$disposition-external.json"
    cmp "$fixture/external-before.json" "$fixture/$disposition-external.json"
done
echo 'Release metadata, failure admission, exact tags/push, resume and publication reconciliation passed (local/mocked effects)'
