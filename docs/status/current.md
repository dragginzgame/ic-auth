# Current handoff — 2026-10-21

## Source and selections

Baseline is released Auth 0.12.1 at
`88edebd5aa4394f70fd6c9f1ecb949905c470164`. Own package versions remain 0.12.1.
Pending notes now select **0.13.0** because published Platform/macros **0.6.0**
change the public Rust Principal/signature-key crate identity. Consumers sharing
these types must align to Platform 0.6; type-only consumers explicitly select
`principal`. Auth's protocol-types manifest selects it without runtime/Candid.
See [consumer migration](../canic-adoption.md#platform-06-migration).

Adopt published coordinated IC Tooling **0.63.0** (Build, Testkit, Host
FS/Process/Artifacts). Exact downloaded archives match Cargo.lock checksums;
packaged source and migration contracts were reviewed. Preserve the incoming
Memory **0.50.3**, Wire/derive **0.5.6**, native Candid/derive **0.10.39** and
Platform/macros **0.6.0** selections. Testkit no longer selects IC Metrics or
Platform 0.5, removing both from the lock. The separately pinned file CLI stays
0.31.1. Shared Tooling 0.25.8 and its canonical 118-file snapshot are unchanged;
there are no sibling/path overrides or Auth version changes.

## Boundary and cleanup

Auth owns proof/credential semantics, checked types, local session/replay/denial
machinery and bounded Wire transport. Canic owns protected fleet authority,
production stores/guards and issuer orchestration. Production acceptance remains
tracked in Auth #3/#58/#59 and Canic #491/#507. No sibling files were edited.

Release fixtures now select the current Git working sources once, including
nonignored additions and working deletions, rather than archived HEAD plus
manually maintained overlays. Shared Tooling still owns admission, companion
closure and copying. Regression coverage verifies working bytes over staged
bytes, executable modes, spaced paths, ignored build/dependency output,
source/index preservation and atomic refusal of unsafe sources. Real release
clean-source, metadata, recovery and publication policies are unchanged.

Private grant/scope/label ceilings now live in Auth's existing limits module.
Independent validators retain all values and error precedence; a grant can offer
32 scopes while a session request selects at most 16. Existing maximum-shape and
rejection tests cover both. Current README/development guides describe current
contracts and all three publication packages; release history stays in CHANGELOG.

Platform 0.6 adds explicit Principal feature selection and omits unused imported
creation-only delegation/CBOR helpers. Auth keeps its crypto policy and exact
signed bytes. No storage or browser-journal migration is introduced. SHA-2/Dalek
major changes remain separate; the adversarial Dalek trigger remains test-only.
Browser dependencies and generated contracts are unchanged.

## Tooling features, recovery and native Candid

Build and Testkit library declarations keep defaults disabled. Resolved native
metadata enables only `wire` for Testkit and no Build feature; neither selects
`cli`. Their prepared executable recipes explicitly select `cli` as required by
0.63. Testkit's removed Candid helpers and Metrics error identities have no Auth
consumers; no adapter or compatibility reexport is needed.

Release-editor executable features are declared once in root workspace metadata.
Recovery passes both the validated source manifest and its lock to offline
admission, preserving source-owned features even when the live manifest changes.
Older manifests with no feature declaration retain their pre-CLI selection;
the previously prepared Build 0.62.1 editor still passes actual offline admission.
Invalid feature policy refuses. Checks never install or use a fallback editor.
Prior tool installations and PocketIC 16.1.0 server bytes remain retained.

There are no direct Candid or full-CDK declarations in Auth's manifests. Testkit
0.63 removes its Candid adapter and stops enabling Platform's `candid` feature.
Native Agent/PocketIC and their upstream transport/management DTOs retain Candid;
all selected Auth/Testkit Platform dependencies now share 0.6. Explicit external
Principal conversions remain. Owned application endpoints stay Wire. See
[the native explanation](../development.md#why-candid-appears-in-cargolock).

Isolated normal/build deployment graphs reject Candid, raw CDK and native
clients/tooling where prohibited. Alias, build/dev/target declarations and
negative fixtures are checked independently. Normal workspace feature unification
is not deployment evidence; the exact supported target/feature graph is.

## Validation and limits

For this tooling adoption, full native/Wasm Clippy passes. Type/Wire and native
host-tooling tests, dependency-boundary checks, all Wasm capability checks and
isolated Rust 1.91 package/native/Wasm matrices plus internal applications pass.
Normalized package archives, sources, manifests and locks stay unchanged during
the MSRV checks. Earlier proof, signature and session checks remain recorded in
`target/local-cleanup-20261021/`; no Auth Rust code changed in this tooling batch.

Build/Testkit 0.63.0 executables are explicitly prepared and admitted offline.
Live qualification passes all nine tests: certification, composition, protected
upgrades, real ingress and SDK two-receiver admission/recovery/logout. This is
local Linux Testkit evidence, not installed production IC or native macOS
qualification. Caller fixtures qualify CLI feature selection, failed/missing
admission, source seals and fail-fast native/Wasm lint; Make jobserver checks pass.
Local bare-remote/mocked release fixtures qualify metadata, failure admission,
exact tags/push, recovery and uncertain publication reconciliation, including
saved/current/pre-CLI editor features. No real Auth staging, commit, release,
publication or deployment is performed.

Formatting, shell lint, documentation links, snapshot and dependency pins pass.
No complete CI, production wallet issuer, durable backend or Canic deployment
acceptance is claimed. The released receipt qualifies its older source/lock,
not this working batch. Pending numbered notes remain 0.13.0.

Task before-images, incoming diff, published archive/source identities, graph
observations and checks are in `target/ic-tooling-adoption-20261021/`. Prior
cleanup and dependency evidence remains in `target/local-cleanup-20261021/` and
`target/candid-upstream-review-20261021/` respectively. Incoming unrelated work,
the real index, HEAD and package versions are preserved; siblings stay read-only.
