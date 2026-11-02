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
consumer="$evidence/consumer with spaces"
mkdir -p "$consumer/scripts/ci" "$consumer/first/src" "$consumer/added member/src" "$evidence/bin"
for helper in run-auth-validation.sh run-validation-targets.sh check-make-execution.sh verify-file-checksum.sh validation-input-checkpoint.pl release-validation-plan.pl; do
    cp "scripts/ci/$helper" "$consumer/scripts/ci/"
done
cat > "$consumer/Cargo.toml" <<'TOML'
[workspace]
members = ["first"]
resolver = "3"
TOML
for member in first 'added member'; do
    name="${member// /-}"
    printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2024"\n' "$name" > "$consumer/$member/Cargo.toml"
    printf '// Fixture library.\n' > "$consumer/$member/src/lib.rs"
done
cat > "$consumer/Makefile" <<'MAKE'
pass:
	@touch passed
mutate-member:
	@echo '# drift' >> 'added member/Cargo.toml'
mutate-root:
	@echo '# drift' >> Cargo.toml
later:
	@touch later
MAKE
(cd "$consumer"; cargo generate-lockfile --offline; bash scripts/ci/run-auth-validation.sh pass) > "$evidence/initial.log" 2>&1
[[ -f "$consumer/passed" ]]
rm "$consumer/passed"
perl -pi -e 's/members = \["first"\]/members = ["first", "added member"]/' "$consumer/Cargo.toml"
(cd "$consumer"; cargo generate-lockfile --offline; bash scripts/ci/run-auth-validation.sh pass) > "$evidence/added.log" 2>&1
[[ -f "$consumer/passed" ]]
rm "$consumer/passed"
for stage in mutate-member mutate-root; do
    if (cd "$consumer"; bash scripts/ci/run-auth-validation.sh "$stage" later) > "$evidence/$stage.log" 2>&1; then
        echo "validation accepted $stage drift" >&2; exit 1
    fi
    [[ ! -e "$consumer/later" ]]
    rg -F 'validation input changed' "$evidence/$stage.log" > /dev/null
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
printf 'Cargo member discovery, added member sealing, drift and discovery refusals passed\n'
