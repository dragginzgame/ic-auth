#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH" RUSTUP_AUTO_INSTALL=0
mkdir -p target/portable-fixtures
fixture="$(mktemp -d "$root/target/portable-fixtures/lint-policy.XXXXXX")"
printf 'Lint policy evidence retained: %s\n' "$fixture"
for workspace in . qualification; do
    name=product
    [[ "$workspace" == . ]] || name=qualification
    yq -p toml -o json '.' "$workspace/Cargo.toml" > "$fixture/$name.json"
    cargo metadata --manifest-path "$workspace/Cargo.toml" --no-deps --locked --offline --format-version 1 > "$fixture/$name-metadata.json"
    while IFS= read -r manifest; do
        yq -p toml -o json '.' "$manifest" | jq -e '.lints.workspace == true' > /dev/null
    done < <(jq -r '.workspace_members as $members | .packages[] | select(.id as $id | $members | index($id)) | .manifest_path' "$fixture/$name-metadata.json")
    dir="$fixture/$name"
    mkdir -p "$dir/member/src"
    printf '[workspace]\nmembers = ["member"]\nresolver = "3"\n' > "$dir/Cargo.toml"
    awk '/^\[/ { copy = /^\[workspace\.lints\./ } copy' "$workspace/Cargo.toml" >> "$dir/Cargo.toml"
    cat > "$dir/member/Cargo.toml" <<'TOML'
[package]
name = "auth-lint-consumer"
version = "0.0.0"
edition = "2024"
[lints]
workspace = true
TOML
    printf '//! Lint admission control.\n/// Returns the fixed control value.\npub fn value() -> u8 { 1 }\n' > "$dir/member/src/lib.rs"
    cargo check --manifest-path "$dir/Cargo.toml" --offline > "$dir/control.log" 2>&1
    for lint in unsafe_code unused_must_use; do
        if [[ "$lint" == unsafe_code ]]; then
            printf '//! Rejection fixture.\n/// Forbidden export.\npub unsafe fn rejected() {}\n' > "$dir/member/src/lib.rs"
        else
            printf '//! Rejection fixture.\n/// Ignored fallible result.\npub fn rejected() { Ok::<(), ()>(()); }\n' > "$dir/member/src/lib.rs"
        fi
        if cargo check --manifest-path "$dir/Cargo.toml" --offline --message-format=json > "$dir/$lint.jsonl" 2> "$dir/$lint.log"; then
            echo "unexpected lint admission: $name/$lint" >&2; exit 1
        fi
        jq -es --arg lint "$lint" 'any(.[]; .reason == "compiler-message" and .message.level == "error" and .message.code.code == $lint)' "$dir/$lint.jsonl" > /dev/null
    done
    # Strict Rustdoc must reject public API debt and broken links on both fresh
    # and reused output, without a diagnostic baseline or missing-doc exception.
    for lint in missing_docs rustdoc::broken_intra_doc_links; do
        if [[ "$lint" == missing_docs ]]; then
            printf '//! Documentation rejection fixture.\npub fn undocumented() {}\n' > "$dir/member/src/lib.rs"
        else
            printf '//! Documentation rejection fixture.\n/// See [missing_item].\npub fn documented() {}\n' > "$dir/member/src/lib.rs"
        fi
        for attempt in fresh warm; do
            output="$dir/rustdoc-${lint//::/-}-$attempt"
            if RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path "$dir/Cargo.toml" --offline --no-deps --message-format=json > "$output.jsonl" 2> "$output.log"; then
                echo "unexpected Rustdoc admission: $name/$lint/$attempt" >&2; exit 1
            fi
            jq -es --arg lint "$lint" 'any(.[]; .reason == "compiler-message" and .message.level == "error" and .message.code.code == $lint)' "$output.jsonl" > /dev/null
        done
    done
    # The adopted Cargo namespace is inherited too: a display-only catalog
    # dependency must fail before source compilation, including on reused output.
    printf '//! Lint admission control.\n/// Returns the fixed control value.\npub fn value() -> u8 { 1 }\n' > "$dir/member/src/lib.rs"
    printf '\n[workspace.dependencies]\nunused-display = "1"\n' >> "$dir/Cargo.toml"
    for attempt in fresh warm; do
        if cargo check --manifest-path "$dir/Cargo.toml" --offline > "$dir/unused-workspace-$attempt.log" 2>&1; then
            echo "unexpected Cargo unused-catalog admission: $name/$attempt" >&2; exit 1
        fi
        rg -F 'unused_workspace_dependencies' "$dir/unused-workspace-$attempt.log" > /dev/null
    done
done
echo 'Both workspace inheritances and unsafe/must-use/Rustdoc/unused-catalog rejections passed'
