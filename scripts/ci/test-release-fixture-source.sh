#!/usr/bin/env bash
set -euo pipefail

cd -P "${BASH_SOURCE[0]%/*}/../.."
root="$PWD"
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/release-source.XXXXXX")"
printf 'Release source export evidence: %s\n' "$fixture"
mkdir "$fixture/source with spaces" "$fixture/export"
source_root="$fixture/source with spaces"
git -C "$source_root" init -q
git -C "$source_root" config user.name Fixture
git -C "$source_root" config user.email fixture@example.invalid
printf '/target/\n/.tools/\n**/node_modules/\n/packages/client/dist/\n' > "$source_root/.gitignore"
printf 'committed\n' > "$source_root/changed"
printf 'obsolete\n' > "$source_root/deleted"
git -C "$source_root" add .
git -C "$source_root" commit -qm initial
printf 'staged\n' > "$source_root/changed"
git -C "$source_root" add changed
printf 'working\n' > "$source_root/changed"
rm "$source_root/deleted"
printf '#!/usr/bin/env bash\necho added\n' > "$source_root/new helper"
chmod +x "$source_root/new helper"
mkdir -p "$source_root/target" "$source_root/.tools" \
    "$source_root/packages/client/node_modules" "$source_root/packages/client/dist"
for path in target .tools packages/client/node_modules packages/client/dist; do
    printf 'not a source\n' > "$source_root/$path/generated"
done
git -C "$source_root" rev-parse HEAD > "$fixture/head.before"
cp "$source_root/.git/index" "$fixture/index.before"
cp "$source_root/changed" "$fixture/working.before"
cp "$source_root/new helper" "$fixture/helper.before"
bash scripts/release/export-fixture-source.sh "$source_root" "$fixture/export"
cmp "$source_root/changed" "$fixture/export/changed"
cmp "$source_root/new helper" "$fixture/export/new helper"
[[ -x "$fixture/export/new helper" && ! -e "$fixture/export/deleted" ]]
[[ ! -e "$fixture/export/target" && ! -e "$fixture/export/.tools" && ! -e "$fixture/export/packages" ]]
cmp "$source_root/.git/index" "$fixture/index.before"
cmp "$source_root/changed" "$fixture/working.before"
cmp "$source_root/new helper" "$fixture/helper.before"
git -C "$source_root" rev-parse HEAD > "$fixture/head.after"
cmp "$fixture/head.before" "$fixture/head.after"
if bash scripts/release/export-fixture-source.sh "$source_root" "$fixture/export" \
    > "$fixture/nonempty.log" 2>&1; then
    echo 'nonempty destination unexpectedly admitted' >&2; exit 1
fi
mkdir "$fixture/refused"
ln -s missing "$source_root/unsafe"
if bash scripts/release/export-fixture-source.sh "$source_root" "$fixture/refused" \
    > "$fixture/symlink.log" 2>&1; then
    echo 'symlink source unexpectedly admitted' >&2; exit 1
fi
rg -F 'missing regular source: unsafe' "$fixture/symlink.log" > /dev/null
[[ -z "$(ls -A "$fixture/refused")" ]]
echo 'Working source selection, deletions, exclusions and source preservation passed'
