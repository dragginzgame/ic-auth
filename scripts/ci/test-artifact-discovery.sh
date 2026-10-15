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

# Execute the actual consumer workflow command; only HTTP and backoff are substituted.
yq -o=json '.jobs.rust.steps' .github/workflows/ci.yml > "$fixture/steps.json"
jq -e '
    ([39] | implode) as $quote
    | [.[] | select(.id == "tools-evidence-download")] as $downloads
    | ($downloads | length) == 1 and all($downloads[];
        .if == ("${{ !cancelled() && steps.tools-evidence-upload.outputs.artifact-id != " + $quote + $quote + " }}")
        and .["timeout-minutes"] == 3 and (."continue-on-error" // false) == false
        and .env.GH_TOKEN == "${{ github.token }}"
        and .env.EVIDENCE_REPOSITORY == "${{ github.repository }}"
        and .env.EVIDENCE_RUN == "${{ github.run_id }}"
        and .env.EVIDENCE_ARTIFACT == "${{ steps.tools-evidence-upload.outputs.artifact-id }}"
        and .env.EVIDENCE_DIGEST == "${{ steps.tools-evidence-upload.outputs.artifact-digest }}")
' "$fixture/steps.json" > /dev/null
jq -er '.[] | select(.id == "tools-evidence-download") | .run' "$fixture/steps.json" > "$fixture/download.sh"
mkdir "$fixture/bin" "$fixture/payload"
printf 'qualified failure bytes\n' > "$fixture/payload/evidence.tar.gz"
(cd "$fixture/payload" && zip -q "$fixture/artifact.zip" evidence.tar.gz)
digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$fixture/artifact.zip")"
cat > "$fixture/bin/gh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1 $2 $3 $4 $5" == 'api --hostname github.com --method GET' && "$GH_TOKEN" == fixture-token ]] || exit 90
printf '%s\n' "$6" >> "$PROBE/calls"
case "$6" in
    repos/owner/repo/actions/artifacts/7)
        count="$(cat "$PROBE/count")"; count=$((count + 1)); echo "$count" > "$PROBE/count"
        [[ "$SCENARIO" != api-error ]] || exit 23
        if [[ "$SCENARIO" == exhausted ]]; then echo 'gh: Not Found (HTTP 404)' >&2; exit 1; fi
        cat "$PROBE/metadata" ;;
    repos/owner/repo/actions/artifacts/7/zip)
        if [[ "$SCENARIO" == delayed && "$(cat "$PROBE/count")" -lt 3 ]]; then
            echo 'gh: Not Found (HTTP 404)' >&2; exit 1
        fi
        if [[ "$SCENARIO" == forbidden ]]; then echo 'gh: Forbidden (HTTP 403)' >&2; exit 1; fi
        if [[ "$SCENARIO" == corrupt ]]; then printf corrupt; else cat "$ARCHIVE"; fi ;;
    *) exit 91 ;;
esac
SCRIPT
cat > "$fixture/bin/sleep" <<'SCRIPT'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$PROBE/sleeps"
SCRIPT
chmod +x "$fixture/bin/gh" "$fixture/bin/sleep"
export PATH="$fixture/bin:$PATH" GH_TOKEN=fixture-token
export EVIDENCE_REPOSITORY=owner/repo EVIDENCE_RUN=42 EVIDENCE_ARTIFACT=7 PROBE SCENARIO
export EVIDENCE_DIGEST="$digest" ARCHIVE="$fixture/artifact.zip" GITHUB_WORKSPACE
for SCENARIO in immediate delayed exhausted api-error wrong-run missing-id corrupt forbidden; do
    PROBE="$fixture/$SCENARIO"; GITHUB_WORKSPACE="$PROBE"
    mkdir -p "$PROBE/target"
    printf '0\n' > "$PROBE/count"; : > "$PROBE/sleeps"; : > "$PROBE/calls"
    run=42; [[ "$SCENARIO" != wrong-run ]] || run=43
    printf '{"id":7,"expired":false,"workflow_run":{"id":%s},"digest":"sha256:%s"}\n' "$run" "$digest" > "$PROBE/metadata"
    EVIDENCE_ARTIFACT=7
    [[ "$SCENARIO" != missing-id ]] || EVIDENCE_ARTIFACT=''
    status=0
    "$BASH" "$fixture/download.sh" > "$PROBE/log" 2>&1 || status=$?
    case "$SCENARIO" in
        immediate|delayed)
            [[ "$status" == 0 ]] || { cat "$PROBE/log"; exit 1; }
            cmp "$fixture/payload/evidence.tar.gz" "$PROBE/target/tools-evidence-downloaded/evidence.tar.gz" ;;
        *) [[ "$status" != 0 && ! -e "$PROBE/target/tools-evidence-downloaded/evidence.tar.gz" ]] ;;
    esac
    case "$SCENARIO" in
        immediate) [[ "$(cat "$PROBE/count")" == 1 && ! -s "$PROBE/sleeps" ]] ;;
        delayed) [[ "$(cat "$PROBE/count")" == 3 && "$(cat "$PROBE/sleeps")" == $'2\n4' ]] ;;
        exhausted) [[ "$(cat "$PROBE/count")" == 6 && "$(cat "$PROBE/sleeps")" == $'2\n4\n8\n16\n30' ]] ;;
        missing-id) [[ "$status" == 2 && ! -s "$PROBE/calls" ]] ;;
        *) [[ "$(cat "$PROBE/count")" == 1 && ! -s "$PROBE/sleeps" ]] ;;
    esac
    [[ "$SCENARIO" != api-error || "$status" == 23 ]]
done
printf 'Auth workflow exact-ID/digest download, bounded retries and failure preservation passed\n'
fixture_complete=true
