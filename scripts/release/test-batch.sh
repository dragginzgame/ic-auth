# shellcheck shell=bash
# Sourced by test-tools.sh --batch: retain its real metadata/receipt adapters
# and existing external-effect substitutes, rather than another release harness.

auth_batch_native() {
    # This is a disposable qualification repository, never a delivery checkout.
    git init -q -b main
    git config user.name 'Frozen native fixture'
    git config user.email 'frozen-native@example.invalid'
    git add .
    git commit -qm 'Clean native frozen-flow qualification source'
    local source
    source="$(git rev-parse HEAD)"
    make --no-print-directory release-candidate-setup RELEASE_SOURCE="$source" \
        RELEASE_SETUP_MODE=prepare > "$fixture/native-setup.log" 2>&1
    # Explicitly prepare sources, then check the actual public package archives
    # and their locked native builds in this new source location.
    env CARGO_NET_OFFLINE=false cargo fetch --locked > "$fixture/native-fetch.log" 2>&1
    make --no-print-directory check-package-builds > "$fixture/native-archives.log" 2>&1
    [[ -z "$(git status --porcelain)" ]]
}

auth_batch_make() {
    # Match an ordinary shell invocation. Do not inherit the enclosing test
    # target's candidate marker or Cargo target into the controller.
    env -u CARGO_TARGET_DIR -u RELEASE_BATCH_ACTIVE make --no-print-directory "$@"
}

auth_batch_contract() {
    export FIXTURE_MODE=success IC_AUTH_BATCH_FIXTURE_ROOT="$PWD"
    local normal="$PWD" source batch state candidate gitdir release_source
    source="$(git rev-parse HEAD)"
    cp Cargo.lock "$fixture/frozen-original.lock"
    if env -u RELEASE_BATCH_ACTIVE CARGO_TARGET_DIR="$PWD/target" make --no-print-directory \
        release-patch > "$fixture/frozen-context-refusal.log" 2>&1; then
        echo 'inherited Cargo target unexpectedly admitted' >&2; exit 1
    fi
    rg -F 'unset CARGO_TARGET_DIR' "$fixture/frozen-context-refusal.log" > /dev/null
    # The first Protocol preparation is interrupted after a partial metadata
    # install. Its gate also changes only this fixture's normal working lock.
    if auth_batch_make release-patch RELEASE_GROUPS=protocol,engine \
        RELEASE_PUBLISH_TARGET=publish > "$fixture/frozen-interrupted.log" 2>&1; then
        echo 'partial frozen preparation unexpectedly completed' >&2; exit 1
    fi
    state="$(git rev-parse --absolute-git-dir)/release-state/frozen"
    batch="$(basename "$(find "$state" -mindepth 1 -maxdepth 1 -type d -name 'batch-*')")"
    [[ -n "$batch" && "$(git rev-parse HEAD)" == "$source" ]]
    candidate="$normal/target/release-frozen/$batch"
    gitdir="$state/$batch/repository.git"
    jq -e '.schema == 2 and .phase == "releasing" and .index == 0 and .candidate == "0.1.1"' \
        "$state/$batch/intent.json" > /dev/null
    [[ -f "$gitdir/release-state/protocol-0.1.1.plan" ]]
    [[ "$(yq -p toml -o json '.' "$candidate/crates/ic-auth-protocol-types/Cargo.toml" | jq -r .package.version)" == 0.1.0 ]]
    [[ "$(yq -p toml -o json '.' "$candidate/crates/ic-auth-wire/Cargo.toml" | jq -r .package.version)" == 0.1.1 ]]
    cp Cargo.lock "$fixture/frozen-deferred.lock"
    cmp "$fixture/frozen-original.lock" <(git show "$source:Cargo.lock")
    rg -F '# Deferred frozen-flow fixture update' Cargo.lock > /dev/null

    # Recovery is offline: a missing saved editor refuses without reinstalling,
    # and the original partial metadata and plan stay available.
    rm "$candidate/target/release-editor-ready"
    if auth_batch_make release-batch-resume BATCH="$batch" > "$fixture/frozen-missing-editor.log" 2>&1; then
        echo 'missing recovery editor unexpectedly admitted' >&2; exit 1
    fi
    [[ "$(tail -n 1 "$candidate/target/editor-events.log")" == check ]]
    touch "$candidate/target/release-editor-ready"
    # A real lost-upload-reply boundary remains uncertain in both journals.
    export FIXTURE_MODE=lost-reply
    if auth_batch_make release-batch-resume BATCH="$batch" > "$fixture/frozen-lost-reply.log" 2>&1; then
        echo 'lost publication reply unexpectedly completed' >&2; exit 1
    fi
    jq -e '.schema == 2 and .phase == "publishing" and .index == 0
        and (.release_commit | test("^[a-f0-9]{40}$"))' "$state/$batch/intent.json" > /dev/null
    [[ "$(jq -r .release_commit "$state/$batch/intent.json")" == "$(git -C "$candidate" rev-parse HEAD)" ]]
    [[ "$(cat "$fixture/uploads")" == ic-auth-protocol-types ]]
    cp "$state/$batch/intent.json" "$fixture/frozen-uncertain-intent.json"
    if auth_batch_make release-batch-resume BATCH="$batch" > "$fixture/frozen-uncertain.log" 2>&1; then
        echo 'uncertain publication was automatically redispatched' >&2; exit 1
    fi
    cmp "$state/$batch/intent.json" "$fixture/frozen-uncertain-intent.json"
    [[ "$(cat "$fixture/uploads")" == ic-auth-protocol-types ]]

    # Registry readback confirms the first upload. Auth's actual publication
    # owner reconciles it and publishes only the missing Protocol peer.
    touch "$fixture/observed-ic-auth-protocol-types"
    export FIXTURE_MODE=success
    (cd "$candidate"; make --no-print-directory publish RELEASE_GROUP=protocol \
        RELEASE_BATCH_ACTIVE=1) > "$fixture/frozen-reconciled-publication.log" 2>&1
    auth_batch_make release-batch-resume BATCH="$batch" PUBLICATION_RESULT=published \
        > "$fixture/frozen-complete.log" 2>&1
    jq -e '.phase == "complete" and .index == 2' "$state/$batch/intent.json" > /dev/null
    [[ "$(cat "$fixture/uploads")" == $'ic-auth-protocol-types\nic-auth-wire\nic-auth' ]]
    [[ "$(git rev-parse HEAD)" == "$source" ]]
    cmp Cargo.lock "$fixture/frozen-deferred.lock"
    # Both groups retain the captured external graph despite the normal edit.
    git show "$source:Cargo.lock" | yq -p toml -o json '.' | jq -cS '[.package[] | select(.source != null)]' > "$fixture/frozen-external-before.json"
    git -C "$candidate" show HEAD:Cargo.lock | yq -p toml -o json '.' | jq -cS '[.package[] | select(.source != null)]' > "$fixture/frozen-external-after.json"
    cmp "$fixture/frozen-external-before.json" "$fixture/frozen-external-after.json"
    local group argv
    while IFS=$'\t' read -r group argv; do
        case "$group" in
            protocol) [[ "$argv" == 'package --locked --registry crates-io -p ic-auth-protocol-types -p ic-auth-wire' ]] ;;
            engine) [[ "$argv" == 'package --locked --registry crates-io -p ic-auth' ]] ;;
            *) exit 1 ;;
        esac
    done < "$fixture/package-argv"
    for group in protocol engine; do
        local version=0.1.1
        [[ "$group" != engine ]] || version=0.9.1
        jq -e --arg group "$group" --arg version "$version" \
            '.group == $group and .version == $version and all(.packages[]; .checksum | test("^[a-f0-9]{64}$"))' \
            "$gitdir/publication-state/$group-$version.json" > /dev/null
    done
    cp "$fixture/uploads" "$fixture/frozen-uploads.complete"
    auth_batch_make release-batch-resume BATCH="$batch" > "$fixture/frozen-completed-resume.log" 2>&1
    cmp "$fixture/uploads" "$fixture/frozen-uploads.complete"
    auth_batch_make release-batch-status BATCH="$batch" > "$fixture/frozen-status.log" 2>&1
    if auth_batch_make release-batch-reconcile BATCH="$batch" > "$fixture/frozen-dirty-reconcile.log" 2>&1; then
        echo 'dirty normal checkout unexpectedly reconciled' >&2; exit 1
    fi
    cmp Cargo.lock "$fixture/frozen-deferred.lock"
    # Only the deliberate fixture edit is cleared, after preserving/asserting it.
    git restore -- Cargo.lock
    auth_batch_make release-batch-reconcile BATCH="$batch" > "$fixture/frozen-reconcile.log" 2>&1
    release_source="$(git -C "$candidate" rev-parse HEAD)"
    [[ "$(git rev-parse HEAD)" == "$release_source" ]]
    [[ "$(git --git-dir="$fixture/remote.git" rev-parse main)" == "$release_source" ]]
    auth_batch_make release-batch-retire BATCH="$batch" > "$fixture/frozen-retire.log" 2>&1
    [[ ! -e "$candidate" && ! -e "$state/$batch" ]]
    [[ "$(git tag --list)" == $'engine-v0.9.1\nprotocol-v0.1.1' ]]
    printf 'Auth frozen setup, native archives, two-group delivery, partial recovery, uncertain upload, deferred lock preservation and shared lifecycle passed\n'
}
