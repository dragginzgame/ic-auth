#!/usr/bin/env bash
# Auth caller proof: discover new workspace manifests and seal their bytes.
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
cd -P "${script_path%/*}/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
mkdir -p target/portable-fixtures
evidence="$(mktemp -d "$root/target/portable-fixtures/auth-validation.XXXXXX")"
printf 'Auth validation input evidence retained: %s\n' "$evidence"
export VALIDATION_LOG_DIR="$evidence/logs" VALIDATION_FAILURE_LOG_DIR="$evidence/failure-logs"
consumer="$evidence/consumer with spaces"
mkdir -p "$consumer/scripts/ci" "$consumer/first/src" "$consumer/added member/src" "$evidence/bin"
for helper in run-auth-validation.sh run-validation-targets.sh check-make-execution.sh verify-file-checksum.sh validation-input-checkpoint.pl release-validation-plan.pl ValidationPlan.pm cargo-release-group.sh cargo-release-groups.jq cargo-qualification-release.sh cargo-qualification-release.jq release-metadata-transaction.sh compare-stable-versions.pl; do
    cp "scripts/ci/$helper" "$consumer/scripts/ci/"
done
cat > "$consumer/Cargo.toml" <<'TOML'
[workspace]
members = ["first"]
resolver = "3"
[workspace.dependencies]
first = { path = "first", version = "0.0.0" }
TOML
for member in first 'added member'; do
    name="${member// /-}"
    printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2024"\n' "$name" > "$consumer/$member/Cargo.toml"
    printf '// Fixture library.\n' > "$consumer/$member/src/lib.rs"
done
cat > "$consumer/ci-policy.json" <<'JSON'
{"schema":1,"shared_metadata_files":[],"groups":[{"name":"first","primary":"first","version":"package","members":["first"],"changelog":"FIRST.md","dependencies":["first"]}]}
JSON
mkdir -p "$consumer/ci"
cp "$consumer/ci-policy.json" "$consumer/ci/cargo-release-groups.json"
printf '# Fixture notes\n' > "$consumer/FIRST.md"
printf '# Fixture notes\n' > "$consumer/ADDED.md"
cat > "$consumer/Makefile" <<'MAKE'
pass:
	@touch passed
mutate-member:
	@echo '# drift' >> 'added member/Cargo.toml'
mutate-root:
	@echo '# drift' >> Cargo.toml
mutate-qualification-root:
	@echo "# drift" >> qualification/Cargo.toml
mutate-qualification-lock:
	@echo "# drift" >> qualification/Cargo.lock
mutate-qualification-member:
	@echo "# drift" >> qualification/probe/Cargo.toml
mutate-lock:
	@echo '# tooling selection drift' >> Cargo.lock
later:
	@touch later
MAKE
(cd "$consumer"; cargo generate-lockfile --offline; bash scripts/ci/run-auth-validation.sh pass) > "$evidence/initial.log" 2>&1
[[ -f "$consumer/passed" ]]
rm "$consumer/passed"
perl -pi -e 's/members = \["first"\]/members = ["first", "added member"]/' "$consumer/Cargo.toml"
printf '\nadded-member = { path = "added member", version = "0.0.0" }\n' >> "$consumer/Cargo.toml"
jq '.groups += [{name:"added",primary:"added-member",version:"package",members:["added-member"],changelog:"ADDED.md",dependencies:["added-member"]}]' \
    "$consumer/ci-policy.json" > "$consumer/ci/cargo-release-groups.json"
# Unequal group versions must remain valid through the shared admission.
sed 's/version = "0.0.0"/version = "0.1.0"/' "$consumer/added member/Cargo.toml" > "$evidence/added.toml"
cp "$evidence/added.toml" "$consumer/added member/Cargo.toml"
sed 's/path = "added member", version = "0.0.0"/path = "added member", version = "0.1.0"/' \
    "$consumer/Cargo.toml" > "$evidence/root.toml"
cp "$evidence/root.toml" "$consumer/Cargo.toml"
(cd "$consumer"; cargo generate-lockfile --offline; bash scripts/ci/run-auth-validation.sh pass) > "$evidence/added.log" 2>&1
[[ -f "$consumer/passed" ]]
rm "$consumer/passed"
# A separate qualification catalog/member/lock must enter the same input seal.
mkdir -p "$consumer/qualification/probe/src"
sed 's/members = \["first", "added member"\]/members = ["first", "added member"]\nexclude = ["qualification"]/' "$consumer/Cargo.toml" > "$evidence/root-with-exclude.toml"
cp "$evidence/root-with-exclude.toml" "$consumer/Cargo.toml"
cat > "$consumer/qualification/Cargo.toml" <<'TOML'
[workspace]
members = ["probe"]
resolver = "3"
[workspace.dependencies]
first = { path = "../first", version = "0.0.0" }
TOML
cat > "$consumer/qualification/probe/Cargo.toml" <<'TOML'
[package]
name = "qualification-probe"
version = "0.0.0"
edition = "2024"
publish = false
[dependencies]
first.workspace = true
TOML
printf '// Qualification input.\n' > "$consumer/qualification/probe/src/lib.rs"
jq '.qualification_workspace = "qualification"' "$consumer/ci/cargo-release-groups.json" > "$evidence/qualification-policy.json"
cp "$evidence/qualification-policy.json" "$consumer/ci/cargo-release-groups.json"
(cd "$consumer"; cargo generate-lockfile --offline --manifest-path qualification/Cargo.toml; bash scripts/ci/run-auth-validation.sh pass) > "$evidence/qualification.log" 2>&1
rm "$consumer/passed"
for stage in mutate-member mutate-root mutate-lock mutate-qualification-root mutate-qualification-lock mutate-qualification-member; do
    if (cd "$consumer"; VALIDATION_LOG_DIR="$evidence/$stage-logs" bash scripts/ci/run-auth-validation.sh "$stage" later) > "$evidence/$stage.log" 2>&1; then
        echo "validation accepted $stage drift" >&2; exit 1
    fi
    [[ ! -e "$consumer/later" ]]
    # Nested runners intentionally retain some diagnostics without echoing them.
    # Prove the expected manifest changed in the actual post-stage checkpoint.
    runs=("$evidence/$stage-logs"/validation.*)
    [[ "${#runs[@]}" == 1 && -f "${runs[0]}/0-after-inputs.json" ]]
    manifest=Cargo.toml
    [[ "$stage" != mutate-member ]] || manifest='added member/Cargo.toml'
    [[ "$stage" != mutate-lock ]] || manifest=Cargo.lock
    [[ "$stage" != mutate-qualification-root ]] || manifest=qualification/Cargo.toml
    [[ "$stage" != mutate-qualification-lock ]] || manifest=qualification/Cargo.lock
    [[ "$stage" != mutate-qualification-member ]] || manifest=qualification/probe/Cargo.toml
    jq -e --arg manifest "$manifest" --slurpfile after "${runs[0]}/0-after-inputs.json" '
      (.[$manifest] | type == "string") and
      ($after[0][$manifest] | type == "string") and
      (.[$manifest] != $after[0][$manifest]) and
      (del(.[$manifest]) == ($after[0] | del(.[$manifest])))
    ' "${runs[0]}/inputs.json" > /dev/null
done
cat > "$evidence/bin/cargo" <<'SH'
#!/usr/bin/env bash
if [[ "${AUTH_METADATA_FAILURE:-}" == true ]]; then exit 37; fi
printf '{"workspace_members": ["missing"], "packages": [], "workspace_root": "%s"}\n' "$PWD"
SH
chmod +x "$evidence/bin/cargo"
for failure in true false; do
    status=0
    (cd "$consumer"; PATH="$evidence/bin:$PATH" AUTH_METADATA_FAILURE="$failure" bash scripts/ci/run-auth-validation.sh pass) > "$evidence/refusal-$failure.log" 2>&1 || status=$?
    [[ "$status" != 0 && ! -e "$consumer/passed" ]]
    if [[ "$failure" == true ]]; then [[ "$status" == 37 ]]; fi
done
printf 'Shared group-plan discovery, unequal versions, member/root/Cargo-lock sealing and drift/refusals passed\n'

# Qualify Auth-specific two-workspace isolation and release caller wiring.
bash scripts/ci/test-workspace-layout.sh
