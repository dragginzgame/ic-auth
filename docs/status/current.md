# Current handoff — 2026-10-21

## Source and selections

Baseline is released Auth **0.13.2** at
`30cfea736114af74c6d9f9cb22106680a0fd83c9`; package versions remain 0.13.2.
Pending notes select **0.13.3** for compatible consumer guidance and integration
regressions.
Platform 0.6 owns Auth's public Principal/signature-key Rust identities;
consumers sharing them align to Platform 0.6. The protocol-types dependency
explicitly selects `principal` without runtime/Candid.
See [consumer migration](../canic-adoption.md#platform-06-migration).

The released 0.13.0 qualification selected IC Tooling **0.63.0** (Build, Testkit,
Host FS/Process/Artifacts), Memory **0.50.3**, Wire/derive **0.5.7**, native
Candid/derive **0.10.39** and Platform/macros **0.6.0**. Downloaded archive
checksums and packaged migration contracts were reviewed against that source. Testkit no longer selects IC Metrics or
Platform 0.5, removing both from the lock. The separately pinned file CLI stays
0.31.1. The current 118-file snapshot adopts reviewed Shared Tooling 0.25.9
source `85aded191d85ac4a8c0e5deab207f6be18895862`; there are no sibling/path
overrides or Auth version changes in this adoption.

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
Browser dependencies and Auth DTO bindings were unchanged by the 0.13 release. The committed runtime
was stale after the Wire selection changed; `make generate-client-contracts` now
refreshes it from locked Wire 0.5.7. Its registry-size cache is per codec operation,
with unchanged frames and alias/limit checks. `make test-client` passes all 47 tests
and Rust/TypeScript roundtrips. Checks still compare rather than repair outputs.
Repair evidence and before-images: `target/client-runtime-fix-20261021/`, against
committed adoption `1ee44227e8c52caef0dfd8d3d3db7d7875616830`.

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
not the independent incoming Wire 0.5.8 lockfile update.

Task before-images, incoming diff, published archive/source identities, graph
observations and checks are in `target/ic-tooling-adoption-20261021/`. Prior
cleanup and dependency evidence remains in `target/local-cleanup-20261021/` and
`target/candid-upstream-review-20261021/` respectively. Incoming unrelated work,
the real index, HEAD and package versions are preserved; siblings stay read-only.

## Production issue follow-through

Auth #82 is closed: published protocol-types 0.13 directly exchanges Principal
with Platform 0.6.1 in an isolated Rust 1.91 native/Wasm consumer. The three
remaining trackers retain their original acceptance: [issuer/browser integration
#3](https://github.com/dragginzgame/ic-auth/issues/3), [durable host acceptance
#58](https://github.com/dragginzgame/ic-auth/issues/58), and [production abuse
qualification #59](https://github.com/dragginzgame/ic-auth/issues/59). Corrected
current adoption guidance no longer tells consumers to use 0.10.7, upstream
Principal or wait for already-published 0.11/0.13 dependency cuts.

Read-only Canic source at `c63bdca652bc7b4dd82afafc2e16e0de7b6637b7`
plus active owner changes selects Auth 0.13. Its authorization facade still
returns only denials; the shared SessionStore module is test-only, with active
owner edits. The issuer API still exposes proof install/status, not production
portable issuance/reconciliation. These observations are not a Canic build or
deployment audit. Consumer acceptance remains in Canic #491/#507; do not replace
it with another Auth endpoint, stable allocator or fixture-based completion claim.

This documentation batch changes no authentication implementation, public API,
format, endpoint or backend. Formatting, local links, snapshot and diff checks
pass; Rust/product suites are not rerun for prose. The independently changing
Cargo.lock remains outside this batch; dependency updates need their own generated-client and product qualification before release. No issue is
closed solely because the reusable Auth pieces already shipped. Evidence and
before-images: `target/auth-production-followthrough-20261021/`.

## Repeated generated-runtime mismatch repair

At source `92f288e520812aa19bfb5a9246746fab1141232b`, the incoming lock selects
Wire/derive 0.5.9 while the checked-in browser runtime still predates Wire 0.5.8's
bounded float checks/shared DataViews. `make generate-client-contracts` refreshed
only `wire-runtime.ts`; all four Auth contract modules remain unchanged. No DTO,
method, frame or stored-record migration is introduced.

The actual stale check was reproduced before regeneration. It now reports the
regeneration command and retained candidate path, preserves the original compare
status, and leaves source bytes unchanged. The local agent instructions require
Wire selection changes to regenerate and run `make test-client` in the same
adoption batch. Checks still refuse drift rather than rewriting source.

All 47 client tests, both Rust/TypeScript directions, type/Wire tests and a fresh
exact-output comparison pass. Formatting, shell lint, local links and diff checks
pass. No full CI, native macOS or new production acceptance is claimed. The
incoming lockfile is retained without further dependency changes; qualification
of other incoming dependency updates is outside this focused repair. Evidence:
`target/client-runtime-refresh-20261021/`. Package versions, index and refs remain
unchanged; pending notes stay 0.13.1.

## Release retry PR follow-through

PR #81 is refreshed against released 0.13.1 without replacing its original
commits. The canonical Shared Tooling 0.25.9 runner refuses a second increment
from a committed candidate until its annotated tag and saved destination branch
prove Git delivery. Refusal retains the plan, receipt and dispatch history;
confirmed delivery permits fresh validation, not reuse of the previous receipt.
The current working-source export, saved editor features and browser generation
instructions remain intact. Only compatible contributor tooling changes are
pending for 0.13.2; library APIs, formats, dependencies and package versions stay
unchanged.

The runner's substituted-effect suite and Auth's actual metadata/recovery/local
bare-remote publication fixtures pass. Snapshot integrity, shell lint, local
documentation links and diff checks pass. Fixtures use the already prepared
editor/file tools; no executable installation, real release, publication or
production qualification is performed. Hosted checks for the refreshed PR remain
separate. Evidence: `target/issue-pr-followthrough-20261021/`.

Auth #3/#58/#59 now lead with their precise remaining production acceptance and
retain prior scope as historical evidence. Their delivered producer mechanisms
are not being reimplemented, and the issues remain open for the real Canic
issuer/store/facade and complete workload/browser acceptance in Canic #491/#507.
No sibling source was edited.

## Current Canic readiness review — Auth 0.13.2

Canic default main `4caeec25db7d81f6121db1c95ba7a2d91d319c60` still selects
Auth 0.10. The moving owner checkout over `c63bdca...` selects Auth 0.13.2,
Platform 0.6.3 and Wire 0.5.13. Both inspected facades remain fail-closed and
their shared session adapter is test-only. These are separate source states;
the owner's focused backend tests do not prove a delivered runtime login.

A registry-only consumer using all three published Auth 0.13.2 packages passes
Rust 1.91 native and Wasm checks with Platform 0.6.3 and Wire 0.5.13. It checks
the shared Principal identity, SessionStore read contract, issuer/subject getters,
both verifier selectors and Wire method limits. Its normal/build Wasm graph
excludes Candid/CDK/ic0, native clients/tooling and Solana. Without the complete
certified capability, the captured normal Canic Auth feature selection fails to
name the certified verifier; its dev dependency currently masks that omission.
The adoption guide now states explicit receiver feature profiles and requires
issuer-scoped successful account projection. No Auth API or format changes.

The concrete findings and completion order are on Canic #491/#507. Existing
effective-renewal, retry, cleanup and successful-path budget acceptance remain
with their owner issues. Related Wire/Testkit/memory/checkpoint PRs do not supply
the missing production issuer/store/facade and are not all qualified. Sibling
checkouts remain read-only; no Canic build, native macOS, installed-IC or full
workspace result is claimed. Source snapshots, reproduction and graph evidence:
`target/canic-integration-review-20261021/`. The incoming Auth lockfile remains
byte-for-byte unchanged; notes select compatible 0.13.3 and package versions stay
0.13.2.

## Producer integration regressions for 0.13.3

The session tests now exercise a pre-signed future-start credential: refusal one
nanosecond before its start leaves session/login state empty, exact start admits,
and an exact retry with exhausted credit reuses the verified session without
extending its lease. A second case admits equal subjects from distinct protected
issuers, refuses crossed authority and issuer substitution on the original
browser login, and keeps the other account usable after local logout.

All 14 volatile and 25 restoration-enabled session cases pass, including the
latter on Rust 1.91. The qualification runner also checks on Rust 1.91. Strict
Clippy passes for the session tests, runner and Wasm reference canister. The
explicitly approved Testkit 0.63.2 CLI is prepared and admitted offline. All five
native host tests and ten live qualification cases pass with a fresh reference
Wasm, including issuer/subject Wire projection, wrong-issuer refusal, independent
logout, certification, upgrades, signed ingress and SDK admission/recovery/logout.

The incoming Wire selection exposed another stale generated runtime. Canonical
generation changes only the runtime; all four Auth DTO modules remain unchanged.
The lock advanced externally from Wire 0.5.14 to 0.5.15 during the first live run.
The earlier pass was not reused: the newly selected sources were explicitly
prepared, generated source checked again, and affected checks repeated. All 47
client tests, Rust/TypeScript directions, type tests, runner MSRV/Clippy and the
fresh native/Wasm/live qualification pass for 0.5.15. Captured Rust/frontend input
hashes match before and after the final stages. The adopted dependency-boundary
checks also pass for public capabilities and the reference Wasm. No wire, signing
or stored format was changed by this batch.

This is producer/reference coverage, not a Canic runtime implementation or
production issuer acceptance. Incoming dependency edits are preserved; this task
changes no lock selections. The final graph includes Platform 0.6.4 and Testkit
0.63.2. Complete CI, native browser/macOS and production IC qualification are not
claimed. Evidence and task before-images are under
`target/canic-integration-regressions-0133/`; sibling source stays read-only.
