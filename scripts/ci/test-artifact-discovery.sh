#!/usr/bin/env bash
set -euo pipefail

script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH"
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/auth-artifact-discovery.XXXXXX")"
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else printf 'Auth artifact discovery fixture retained: %s\n' "$fixture" >&2; fi
    exit "$status"
}
trap finish EXIT

# Exercise the maintained workflow command, with read-only API effects replaced.
yq -o=json '.jobs.rust.steps' .github/workflows/ci.yml > "$fixture/steps.json"
jq -e '
    ([39] | implode) as $quote
    | [.[] | select(.id == "tools-evidence-discovery")] as $gates
    | ($gates | length) == 1 and all($gates[];
        .if == ("${{ !cancelled() && steps.tools-evidence-upload.outputs.artifact-id != " + $quote + $quote + " }}")
        and .["timeout-minutes"] == 2 and (."continue-on-error" // false) == false
        and .env.GH_TOKEN == "${{ github.token }}"
        and .env.EVIDENCE_REPOSITORY == "${{ github.repository }}"
        and .env.EVIDENCE_RUN == "${{ github.run_id }}"
        and .env.EVIDENCE_ARTIFACT == "${{ steps.tools-evidence-upload.outputs.artifact-id }}")
    and ([.[] | select((.uses // "") | startswith("actions/download-artifact@"))] as $downloads
        | ($downloads | length) == 1 and all($downloads[];
            .if == ("${{ !cancelled() && steps.tools-evidence-discovery.outcome == " + $quote + "success" + $quote + " }}")
            and .uses == "actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333"
            and .with["artifact-ids"] == "${{ steps.tools-evidence-upload.outputs.artifact-id }}"
            and .with.repository == "${{ github.repository }}"
            and .with["run-id"] == "${{ github.run_id }}"
            and .with["github-token"] == "${{ github.token }}"
            and .with["digest-mismatch"] == "error"
            and (."continue-on-error" // false) == false))
' "$fixture/steps.json" > /dev/null
jq -er '.[] | select(.id == "tools-evidence-discovery") | .run' "$fixture/steps.json" > "$fixture/discover.sh"
mkdir "$fixture/bin"
cat > "$fixture/bin/gh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == 'api --hostname github.com --method GET --paginate repos/owner/repo/actions/runs/42/artifacts?per_page=100' && "$GH_TOKEN" == fixture-token ]] || exit 90
count="$(cat "$PROBE/count")"
count=$((count + 1))
printf '%s\n' "$count" > "$PROBE/count"
[[ "$SCENARIO" != api-error ]] || exit 23
if [[ "$SCENARIO" == exhausted || ( "$SCENARIO" == delayed && "$count" -lt 3 ) ]]; then
    printf '{"artifacts":[{"id":6,"expired":false,"workflow_run":{"id":42}}]}\n'
elif [[ "$SCENARIO" == wrong-run ]]; then
    printf '{"artifacts":[{"id":7,"expired":false,"workflow_run":{"id":43}}]}\n'
else
    printf '{"artifacts":[]}\n{"artifacts":[{"id":7,"expired":false,"workflow_run":{"id":42}}]}\n'
fi
SCRIPT
cat > "$fixture/bin/sleep" <<'SCRIPT'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$PROBE/sleeps"
SCRIPT
chmod +x "$fixture/bin/gh" "$fixture/bin/sleep"
export PATH="$fixture/bin:$PATH" GH_TOKEN=fixture-token
export EVIDENCE_REPOSITORY=owner/repo EVIDENCE_RUN=42 EVIDENCE_ARTIFACT=7 PROBE SCENARIO
for SCENARIO in immediate delayed exhausted api-error wrong-run missing-id; do
    PROBE="$fixture/$SCENARIO"
    mkdir "$PROBE"
    printf '0\n' > "$PROBE/count"
    : > "$PROBE/sleeps"
    EVIDENCE_ARTIFACT=7
    [[ "$SCENARIO" != missing-id ]] || EVIDENCE_ARTIFACT=''
    status=0
    "$BASH" "$fixture/discover.sh" > "$PROBE/log" 2>&1 || status=$?
    case "$SCENARIO" in
        immediate) [[ "$status" == 0 && "$(cat "$PROBE/count")" == 1 && ! -s "$PROBE/sleeps" ]] ;;
        delayed) [[ "$status" == 0 && "$(cat "$PROBE/count")" == 3 && "$(cat "$PROBE/sleeps")" == $'2\n4' ]] ;;
        exhausted) [[ "$status" == 1 && "$(cat "$PROBE/count")" == 6 && "$(cat "$PROBE/sleeps")" == $'2\n4\n8\n16\n30' ]] ;;
        api-error) [[ "$status" == 23 && "$(cat "$PROBE/count")" == 1 && ! -s "$PROBE/sleeps" ]] ;;
        wrong-run) [[ "$status" != 0 && "$(cat "$PROBE/count")" == 1 && ! -s "$PROBE/sleeps" ]] ;;
        missing-id) [[ "$status" == 2 && "$(cat "$PROBE/count")" == 0 && ! -s "$PROBE/sleeps" ]] ;;
    esac
done
printf 'Auth workflow exact-ID discovery, bounded retries and strict download gates passed\n'
fixture_complete=true
