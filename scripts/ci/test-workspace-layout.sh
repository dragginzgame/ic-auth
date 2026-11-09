#!/usr/bin/env bash
# Actual Auth workspace/release wiring; no product build, upload or remote.
set -euo pipefail
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR RELEASE_COMMIT
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH" RUSTUP_AUTO_INSTALL=0
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
mkdir -p target/portable-fixtures
evidence="$(mktemp -d "$root/target/portable-fixtures/workspace-layout.XXXXXX")"
printf 'Auth workspace/release evidence retained: %s\n' "$evidence"
CARGO_RELEASE_METADATA_EDITOR="$(bash scripts/dev/release-tools.sh check)"
export CARGO_RELEASE_METADATA_EDITOR
export CARGO_RELEASE_METADATA_MAX_BYTES=8388608
actual_cargo="$(command -v cargo)"
export AUTH_WORKSPACE_CARGO="$actual_cargo"
mkdir "$evidence/consumer with spaces" "$evidence/bin"
consumer="$evidence/consumer with spaces"
bash scripts/release/export-fixture-source.sh "$root" "$consumer"
cd "$consumer"
git init -q
git config user.name 'Auth workspace qualification'
git config user.email 'auth-workspace@example.invalid'
git add .
git commit -qm 'Current Auth two-workspace fixture'
baseline="$(git rev-parse HEAD)"
cat > "$evidence/bin/cargo" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${AUTH_WORKSPACE_FAIL:-}" == true && "$1" == metadata &&
      "$PWD" == "$AUTH_WORKSPACE_CONSUMER/qualification" &&
      "$(yq -p toml -o json '.' "../$AUTH_WORKSPACE_PRIMARY" | jq -r .package.version)" == "$RELEASE_VERSION" ]]; then
    echo 'Injected Auth qualification graph failure after product rendering' >&2
    exit 42
fi
exec "$AUTH_WORKSPACE_CARGO" "$@"
SCRIPT
chmod +x "$evidence/bin/cargo"
export AUTH_WORKSPACE_CONSUMER="$consumer"
export PATH="$evidence/bin:$PATH"
export RELEASE_DELIVERY=direct RELEASE_DATE=2026-11-09
for RELEASE_GROUP in protocol engine; do
    export RELEASE_GROUP
    git reset --hard -q "$baseline"
    RELEASE_PREVIOUS=$(bash scripts/ci/cargo-release-group.sh version "$RELEASE_GROUP")
    RELEASE_VERSION=$(bash scripts/ci/next-release-version.sh "$RELEASE_PREVIOUS" patch)
    export RELEASE_PREVIOUS RELEASE_VERSION
    ledger=$(jq -r --arg group "$RELEASE_GROUP" '.groups[] | select(.name == $group) | .changelog' ci/cargo-release-groups.json)
    primary=$(jq -r --arg group "$RELEASE_GROUP" '.groups[] | select(.name == $group) | .primary' ci/cargo-release-groups.json)
    AUTH_WORKSPACE_PRIMARY=$(yq -p toml -o json '.' Cargo.toml | jq -r --arg primary "$primary" '.workspace.dependencies[$primary].path + "/Cargo.toml"')
    export AUTH_WORKSPACE_PRIMARY
    printf '# Fixture ledger\n\n## [%s]\n\n- Auth workspace caller.\n' "$RELEASE_VERSION" > "$ledger"
    git add "$ledger"
    git commit -qm 'Prepare selected fixture notes'
    candidate_source=$(git rev-parse HEAD)
    bash scripts/release/metadata.sh files > "$evidence/$RELEASE_GROUP-files0"
    tr '\0' '\n' < "$evidence/$RELEASE_GROUP-files0" > "$evidence/$RELEASE_GROUP-files"
    for path in qualification/Cargo.toml qualification/Cargo.lock release-validation.json; do
        [[ $(rg -Fxc "$path" "$evidence/$RELEASE_GROUP-files") == 1 ]]
    done
    ! rg '^qualification/.+/Cargo.toml$' "$evidence/$RELEASE_GROUP-files"
    cargo metadata --locked --offline --format-version 1 --manifest-path qualification/Cargo.toml > "$evidence/$RELEASE_GROUP-before.json"
    bash scripts/ci/cargo-release-group.sh prepare "$RELEASE_GROUP" > "$evidence/$RELEASE_GROUP-prepare.log" 2>&1
    bash scripts/ci/cargo-release-group.sh check "$RELEASE_GROUP" > "$evidence/$RELEASE_GROUP-check.log" 2>&1
    cargo metadata --locked --offline --format-version 1 --manifest-path qualification/Cargo.toml > "$evidence/$RELEASE_GROUP-after.json"
    jq -en --slurpfile before "$evidence/$RELEASE_GROUP-before.json" --slurpfile after "$evidence/$RELEASE_GROUP-after.json" \
        --arg primary "$primary" --arg version "$RELEASE_VERSION" '
        def private($m): [$m.packages[] | select(.id as $id | $m.workspace_members | index($id)) | {name,version}] | sort_by(.name);
        private($before[0]) == private($after[0]) and
        any($after[0].packages[]; .name == $primary and .version == $version)' > /dev/null
    # A failure in the actual qualification graph must restore every file in
    # Auth's selected plan, including both locks and the qualification catalog.
    git reset --hard -q "$candidate_source"
    status=0
    AUTH_WORKSPACE_FAIL=true bash scripts/ci/cargo-release-group.sh prepare "$RELEASE_GROUP" \
        > "$evidence/$RELEASE_GROUP-rollback.log" 2>&1 || status=$?
    [[ "$status" != 0 ]]
    rg -F 'Injected Auth qualification graph failure' "$evidence/$RELEASE_GROUP-rollback.log" > /dev/null
    git diff --exit-code > "$evidence/$RELEASE_GROUP-restored.diff"
done
git reset --hard -q "$baseline"
# Supplier incompatibility belongs to qualification, not product resolution.
sed 's/ic-testkit = { version = "[^"]*"/ic-testkit = { version = "=0.0.0"/' qualification/Cargo.toml > "$evidence/incompatible.toml"
cp "$evidence/incompatible.toml" qualification/Cargo.toml
cargo metadata --locked --offline --format-version 1 > "$evidence/independent-product.json"
status=0
cargo metadata --offline --manifest-path qualification/Cargo.toml --format-version 1 \
    > "$evidence/incompatible-qualification.json" 2> "$evidence/incompatible-qualification.log" || status=$?
[[ "$status" != 0 ]]
rg -F 'ic-testkit' "$evidence/incompatible-qualification.log" > /dev/null
git diff --exit-code -- Cargo.toml Cargo.lock
printf 'Auth two-group file plans, both-lock preparation, private versions, rollback and independent product resolution passed\n'
