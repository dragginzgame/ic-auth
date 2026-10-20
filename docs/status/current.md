# Current handoff — 2026-10-20

## Source and pending release

Baseline is released Auth 0.12.0 at
`6f0d68db56a0d10fc27ac0bacbd76acaeb8d5b2f`. Own package versions remain 0.12.0;
pending notes select **0.12.1**. The current incoming manifest/lock are preserved
byte-for-byte. This pass synchronizes adoption notes with the coordinated
upstream releases; it does not perform a dependency update, commit, release,
publication or deployment. Earlier qualification below covers the older graph.

The registry and archive checksums verify published Host FS/Process, Build and
Testkit **0.62.0**, now owned by [IC Tooling](https://github.com/dragginzgame/ic-tooling),
and Memory **0.50.0**, now owned by [IC Persistence](https://github.com/dragginzgame/ic-persistence).
Wire/derive **0.5.3**, Metrics **0.14.12** and Platform/macros **0.5.0** remain current. These are
registry releases, not sibling/path overrides. Testkit now selects Platform 0.5,
removing the older 0.4 copy from the lock. Testkit, Build and Auth's direct native
callers now share Host 0.62; the old Host 0.31/0.32 copies are gone without overrides.
The independently pinned file CLI remains 0.31.1.

## Compatibility and ownership

Published Host, Build, Testkit and Memory implementation bodies are unchanged
from the previously qualified selections; one Host process-test expression changes.
Their package/dependency identities advance. Tooling packages declare Rust 1.91,
matching Auth's existing minimum. Testkit retains simulator
external Principals, explicit Wire calls, limits, server 16.1.0 and lifecycle
policy. Wire 0.5.2 changed initial bounded encoder allocation, not frames or IDs;
0.5.3 retains that implementation. Metrics 0.14.12 has unchanged source bodies.
The inspected source and dependency declarations indicate no Auth public API,
signed-byte, browser journal or storage migration. Memory's normal dependencies
are unchanged; Auth uses it only in the unpublished checkpoint fixture.
The released Platform 0.5 migration and explicit external Principal
conversions remain in force; see [consumer guidance](../canic-adoption.md).

The existing boundary policy excludes Candid from public library capability
graphs and the reference Wasm; its checks have not been rerun for this selection.
Native Agent/Testkit retain Candid through Platform. Host/Build/Testkit remain
outside the public library graphs. No raw CDK dependency or local replacement
crypto/runtime code is introduced. The Shared Tooling 0.25.8 snapshot, full early
Clippy gate and source-bound release recovery contracts remain unchanged.

## Other dependency observations

Testkit 0.62 uses a compatible `thiserror` requirement rather than 0.60's exact
2.0.18 pin, so that previous patch-update blocker is gone. The lock still selects
2.0.18; this documentation pass does not update it. SHA-2 0.11 and Dalek 3 are separate major migrations:
Auth keeps SHA-2 0.10 shared with Platform, and the adversarial Dalek 2.2 feature
trigger must match Platform's verifier identity.

The previous browser check found SDK 6.1.1 and fake-indexeddb 6.2.6 patch updates.
They are not included in this incoming Rust batch; its prepared client remains
SDK 6.1.0/fake-indexeddb 6.2.5. TypeScript 7.0.2 is current; Node 26 type packages
do not match the selected Node 24 runtime. No npm lock or installation changed.

## Validation and delivery

The previous Host 0.32/Build 0.24/Testkit 0.60/Memory 0.49.3 graph passed full
native/Wasm Clippy, isolated Rust 1.91 package/native/Wasm checks, all Auth
protocol/type/signature/store/certified-proof/session suites, Wasm capabilities,
11 host-tooling tests and 47 browser-client tests. Dependency exclusions,
snapshot, pins, Testkit caller/early-lint fixtures and the substituted release
runner passed. All three package dry runs passed with live registry metadata and
without uploads. No full CI or native macOS qualification is claimed.

The maintainer-approved Testkit 0.60 and Build 0.24 executables are prepared and
admitted offline. Five native fixture tests, fresh release Wasm and all nine
local canister scenarios pass, including certification, composed roots, upgrade
recovery, signed ingress and SDK two-receiver admission/logout. The issuer lookup
and IndexedDB remain injected boundaries, not production issuer/persistence
acceptance. The actual local/mocked release/recovery fixtures pass with the new
editor, including interrupted preparation and publication reconciliation. These
results were repeated against Wire 0.5.3/Metrics 0.14.12 after the late lock
update; the final input seal remained unchanged. Logs prefixed `late-lock-`
record this final selection; earlier logs retain the original selection.

Those results do not qualify the incoming Tooling 0.62/Memory 0.50 graph. This
pass verified published metadata, checksums and source/dependency differences,
then checked documentation links, snapshot integrity and the diff. No Rust
suite or executable installation was run for the notes correction. The new
Testkit/Build selections require their explicit setup/admission before live
canister and release/recovery qualification; earlier setup covered 0.60/0.24.

Auth owns reusable proofs, checked types, local session/replay/denial machinery
and bounded Wire transport. Canic owns protected fleet authority, production
stores/guards and issuer orchestration. Remaining production acceptance belongs
to Auth #3/#58/#59 and Canic #491/#507; no sibling edits were made.
Evidence and exact incoming before-images are in
`target/upstream-adoption-notes-20261020/`. Previous qualification remains in
`target/dependency-refresh-20261020/`; earlier release evidence remains separate.
