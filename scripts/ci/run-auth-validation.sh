#!/usr/bin/env bash
# Add every Cargo-admitted member manifest to Auth's validation input seal.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
bash scripts/ci/check-make-execution.sh
mkdir -p target/validation-inputs
evidence="$(mktemp -d "$PWD/target/validation-inputs/run.XXXXXX")"
checkpoint=scripts/ci/validation-input-checkpoint.pl
checksum=scripts/ci/verify-file-checksum.sh
perl "$checkpoint" "$PWD" '' "$evidence/before.json" "$checksum" Cargo.toml Cargo.lock
cargo metadata --locked --offline --no-deps --format-version 1 > "$evidence/metadata.json"
# Refuse incomplete/redirected discovery rather than sealing a partial roster.
jq -er --arg root "$PWD" '
  . as $m | .workspace_members as $members |
  if .workspace_root != $root or ($members | length) == 0 or
     ($members | unique | length) != ($members | length) then error("invalid workspace inventory") else
    [$members[] as $id | [$m.packages[] | select(.id == $id)] |
      if length != 1 then error("missing workspace member") else .[0].manifest_path end] |
    if all(.[]; startswith($root + "/") and endswith("/Cargo.toml") and (test("[\\r\\n]") | not))
    then .[] | ltrimstr($root + "/") else error("invalid member manifest path") end
  end
' "$evidence/metadata.json" > "$evidence/manifests"
inputs=(--input Cargo.toml --input Cargo.lock --input scripts/ci/run-auth-validation.sh)
while IFS= read -r manifest; do inputs+=(--input "$manifest"); done < "$evidence/manifests"
perl "$checkpoint" "$PWD" "$evidence/before.json" "$evidence/discovered.json" "$checksum"
status=0
bash scripts/ci/run-validation-targets.sh "${inputs[@]}" "$@" || status=$?
# Discovery and execution must see the same root selection, including the gap
# before the runner takes over its expanded seal. Preserve the first failure.
perl "$checkpoint" "$PWD" "$evidence/before.json" "$evidence/after.json" "$checksum" || {
    [[ "$status" != 0 ]] || status=1
}
exit "$status"
