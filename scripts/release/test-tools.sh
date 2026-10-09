#!/usr/bin/env bash
set -euo pipefail

# Local Git/bare-remote fixtures, real metadata transforms and mocked upload
# transport. No source commit, remote ref or registry effect in the real repo.
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$root/.tools/rust/bin:$PATH"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
mkdir -p "$root/target/portable-fixtures"
fixture="$(mktemp -d "$root/target/portable-fixtures/release-tools.XXXXXX")"
echo "Release/publication fixture evidence: $fixture"
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
tar -C "$root/packages" --exclude=node_modules --exclude=dist -cf - client | tar -xf - -C "$fixture/repo/packages"
overlays=(.gitignore Makefile Cargo.toml Cargo.lock CHANGELOG.md .shared-tooling.snapshot
    ci/ic-auth-tools.tsv
    scripts/release/metadata.sh scripts/release/publish.sh scripts/release/test-tools.sh
    scripts/release/rewrite-manifest.pl
    scripts/dev/run-host-tooling.sh
    scripts/dev/client-tools.sh scripts/dev/client-contracts.sh scripts/dev/test-client.sh
    scripts/dev/test-qualification.sh
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
# A substitute gate is deliberate: prove runner/adapter effects and failure
# contracts without recursively running full CI from a focused tooling test.
cat >> Makefile <<'MAKE'

ci:
	@test ! -f target/fail-gate
	@echo fixture-complete-gate

release-prepare-version:
	@bash scripts/release/metadata.sh prepare-version
	@if test -f target/interrupt-prepare; then git show HEAD:Cargo.toml > Cargo.toml; exit 19; fi
MAKE
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
[[ "$(git rev-parse HEAD)" == "$before" && -z "$(git tag -l)" ]]
touch target/fail-gate
if make --no-print-directory release-patch > "$fixture/gate-failure.log" 2>&1; then exit 1; fi
[[ "$(git rev-parse HEAD)" == "$before" && "$(make -s release-version)" == 0.1.0 ]]
[[ -z "$(git tag -l)" && -f target/keep ]]
rm target/fail-gate

# Initial admission names a changed lock and refuses before validation or
# preparation, without changing source or index bytes.
printf '\n' >> Cargo.lock
cp Cargo.lock "$fixture/dirty-lock"
cp .git/index "$fixture/dirty-index"
if make --no-print-directory release-patch > "$fixture/lock-source.log" 2>&1; then exit 1; fi
rg -F 'unstaged: Cargo.lock' "$fixture/lock-source.log" > /dev/null
rg -F 'this attempt has not started validation or version preparation' "$fixture/lock-source.log" > /dev/null
if rg -F 'fixture-complete-gate' "$fixture/lock-source.log" > /dev/null; then exit 1; fi
cmp Cargo.lock "$fixture/dirty-lock"
cmp .git/index "$fixture/dirty-index"
[[ "$(git rev-parse HEAD)" == "$before" && -z "$(git tag -l)" ]]
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
[[ "$(make -s release-version)" == 0.1.1 ]]
commit="$(git rev-parse HEAD)"
bash scripts/ci/check-release-tag.sh "$commit" 0.1.1
[[ "$(git --git-dir="$fixture/remote.git" rev-parse main)" == "$commit" ]]
[[ "$(git --git-dir="$fixture/remote.git" rev-parse v0.1.1)" == "$(git rev-parse v0.1.1)" ]]
[[ -f target/keep ]]
git show "$before:Cargo.lock" | yq -p toml -o json '.' | jq -S '[.package[] | select(.source != null)]' > "$fixture/external-before.json"
yq -p toml -o json '.' Cargo.lock | jq -S '[.package[] | select(.source != null)]' > "$fixture/external-after.json"
cmp "$fixture/external-before.json" "$fixture/external-after.json"
make --no-print-directory release-resume VERSION=0.1.1 > "$fixture/resume.log" 2>&1
[[ "$(git rev-parse HEAD)" == "$commit" ]]

# Fake only package creation/upload and registry reads. Metadata, tag checks,
# clean-worktree admission and publication intent/reconciliation stay real.
mkdir "$fixture/bin"
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
output=''
while [[ $# -gt 0 ]]; do
    case "$1" in --output) output="$2"; shift 2 ;; *) url="$1"; shift ;; esac
done
package="$(basename "$(dirname "$url")")"
case "$FIXTURE_MODE" in
    unknown) printf 503 ;;
    conflict)
        jq -n '{version:{num:"0.1.1",checksum:"wrong"}}' > "$output"
        printf 200 ;;
    *)
        if [[ -f "$FIXTURE_LOG/observed-$package" ]]; then
            checksum="$(bash scripts/dev/run-host-tooling.sh hash-file "$CARGO_TARGET_DIR/package/$package-0.1.1.crate" 67108864)"
            jq -n --arg checksum "$checksum" '{version:{num:"0.1.1",checksum:$checksum}}' > "$output"
            printf 200
        else printf 404; fi
        ;;
esac
SH
chmod +x "$fixture/bin/cargo" "$fixture/bin/curl"
export PATH="$fixture/bin:$PATH" CARGO_TARGET_DIR="$PWD/target"
if bash scripts/release/publish.sh > "$fixture/unknown.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]]
export FIXTURE_MODE=conflict
if bash scripts/release/publish.sh > "$fixture/conflict.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]]
export FIXTURE_MODE=marker-source-change
if bash scripts/release/publish.sh > "$fixture/marker-source-change.log" 2>&1; then exit 1; fi
[[ ! -f "$fixture/uploads" ]]
git reset --hard "$commit" > /dev/null
# This isolated fixture observed no dispatch at all. Restore its known-failed
# marker before testing uncertainty separately; never touch real repo intent.
rm .git/publication-state/0.1.1-ic-auth-protocol-types.dispatched
export FIXTURE_MODE=lost-reply
if bash scripts/release/publish.sh > "$fixture/lost-reply.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" == 1 ]]
if bash scripts/release/publish.sh > "$fixture/unresolved.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" == 1 ]]
touch "$fixture/observed-ic-auth-protocol-types"
export FIXTURE_MODE=success
bash scripts/release/publish.sh > "$fixture/publish.log" 2>&1
[[ "$(cat "$fixture/uploads")" == $'ic-auth-protocol-types\nic-auth' ]]
bash scripts/release/publish.sh > "$fixture/publish-retry.log" 2>&1
[[ "$(wc -l < "$fixture/uploads")" == 2 ]]
export FIXTURE_MODE=source-change
if bash scripts/release/publish.sh > "$fixture/source-change.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$fixture/uploads")" == 2 ]]
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
        [[ "$(make -s release-version)" == 0.1.1 && -z "$(git tag -l v0.2.0)" ]]
        rm target/interrupt-prepare
    fi
    make --no-print-directory "release-$kind" > "$fixture/$kind.log" 2>&1
    [[ "$(make -s release-version)" == "$candidate" ]]
    bash scripts/ci/check-release-tag.sh "$(git rev-parse HEAD)" "$candidate"
done
[[ -f target/keep ]]
echo 'Release metadata, failure admission, exact tags/push, resume and publication reconciliation passed (local/mocked effects)'
