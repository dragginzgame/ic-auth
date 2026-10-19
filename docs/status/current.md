# Current handoff — 2026-10-19

## 0.10.6 — release isolation, native CLI and browser admission

Source baseline is released 0.10.5 `d3c462546fd7cf9cbe15b209246d7f026fa8d8d3`.
Preserve the incoming lock-only Wire 0.4.7 update and the later concurrent 0.4.8
selection. Package versions remain 0.10.5; compatible notes are under 0.10.6.
Before-images and evidence are in `target/three-improvements-20261019/`.

The development guide now gives the dedicated clean release-checkout workflow,
including fast-forward admission, source isolation and same-checkout recovery
without deleting plans, publication intent, tools or evidence. This uses existing
Git/release commands, not a second release runner or a disabled input-drift guard.
No release checkout, release or publication was created/executed by this batch.

Clap owns the existing native CLI grammar, path/limit values, modes, help/version
and argument errors; refusal status remains 1 and operation bytes are unchanged.
Its explicit native-only features exclude color/suggestion backends. Generic
file-command removal remains [Auth #78](https://github.com/dragginzgame/ic-auth/issues/78),
blocked on [Shared #289](https://github.com/dragginzgame/shared-tooling/issues/289):
the reviewed installer cannot select Host's required `cli` feature or bind that
selection into receipt/cache identity. Keep the usable adapter until canonical
setup/admission and consumer qualification permit deletion. Parser delivery is
tracked separately in [Auth #77](https://github.com/dragginzgame/ic-auth/issues/77).

`httpAdmissionTransport` now supplies shared-Wire receiver admission through the
existing SDK update owner: fixed scopes/TTL, generated bounded encoding/decoding,
current guarded identity, one update submission, authenticated response and
same-caller success. The real two-receiver SDK test uses it. Issuer issue/reconcile
operations and Canic's own command envelope remain injected/consumer-owned.
[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[Canic #491](https://github.com/dragginzgame/canic/issues/491) and
[#507](https://github.com/dragginzgame/canic/issues/507) retain production acceptance;
read-only Canic inspection still found no portable browser issue/reconcile
endpoint contract. No wallet service, production issuer or sibling edit is claimed.

47 client tests, 11 native tooling tests, Rust 1.91 tooling compilation, strict
tooling Clippy, dependency boundaries, snapshot/pins/formatting and local links
pass. Installed Testkit 0.55.2/server 16.1.0 passes five native fixture tests and
seven canister cases, including the new HTTP adapter and real SDK logout.
The actual Build 0.21 editor/local mocked release fixture passes interruption,
resume, private-write preservation and uncertain-publication reconciliation.
Final sealed current-input checks pass with Wire 0.4.8, including all-target Rust
1.91 tooling compilation, development Clippy, tooling/client tests, boundaries and source/input
integrity. No full CI, native macOS, real-browser
IndexedDB or production issuer/deployment qualification was performed.

## Platform 0.1.18, Host 0.29.0 and Shared Tooling 0.25.1

Verified the latest non-yanked published Platform/macros 0.1.18 and Host
FS/Process/Artifacts 0.29.0 against their registry checksums and actual archives.
The incoming dirty root/lock already selected Host 0.29 and Platform 0.1.18;
retain those bytes/selections and raise the root Platform minimum to 0.1.18.
Package versions remain 0.10.4; compatible follow-up notes are under 0.10.5.

Shared's canonical exporter preview and refresh adopt reviewed remote HEAD
`d57c631206e3618755e45bebf2c88c673564ddc1` (0.25.1) with the existing entrypoint
closure. This updates the normative release-setup rule and shared guides, without
changing selected scripts or executable pins. Auth's existing automatic fresh
preflight setup and offline saved-source recovery already implement that rule.

Platform's published crypto exports are unchanged from 0.1.17; Auth already routes
K256, Dalek, Principal and its IC runtime/certificate APIs through them. Its
general-purpose SHA-256, CBOR and LEB128 consumers remain active. Keep the isolated
adversarial Dalek dev feature trigger and Testkit's native runtime boundary.
Host's new CLI, capability, certificate and Wasm text/encoding APIs replace no
remaining direct dependency in Auth. Existing Host file reads, durable writes and
bounded process capture retain their public contracts; no extra feature is needed.
Older transitive Host packages selected by Build/Testkit remain upstream-owned;
do not force their coordinated migration through consumer overrides.

Before-images, published archives, exporter preview and validation evidence are
under `target/upstream-adoption-20261019/`. Host tooling, chain-key and portable
signature tests pass, including adversarial feature unification. Rust 1.91
native/Wasm package payloads, private applications, development Wasm and strict
Clippy pass. Direct dependency admission/refusal and actual normal/build graph
checks pass; Host/Build/Testkit remain excluded from deployed library graphs.
No new dependency-boundary enforcement gap was found. Snapshot integrity, pins,
formatting and local links pass; the GitHub description remains accurate.

The isolated release/publication fixture passes automatic editor setup, exact
source/version admission, interruption/recovery and uncertain upload handling
using the admitted Build 0.20 editor and mocked transport. Installed Testkit
0.54.1/server 16.1.0 passes five native fixture tests and seven canister cases,
including certification, upgrades, ingress and actual SDK two-receiver logout.

The adoption tree was committed concurrently as `3c4930d`; no agent Git effects
were performed. A later lock-only Wire/macros 0.4.6 update was preserved. Local
canister qualification passes with that selection, and the final sealed focused
run rechecks type/Wire and tooling tests, MSRV, Wasm, Clippy and dependency graphs
against its exact lock. The separate existing full gate completed its release
fixture successfully but refused the concurrent lock drift; a full release gate
must restart against current inputs before release. No full gate was started by
this task. Siblings remain read-only; native macOS, production deployment and
consumer adoption are not qualified here.

## IC Platform 0.1.17 — ECDSA adoption

Selected latest non-yanked published Platform 0.1.17, archive SHA-256
`98b4d131913792a85fc9f134f40a436c7a3a36c39ebc3546200facbc4f6993d8`,
source `51784c904ec25d7da1ba10bdc526b9bf694f5487`. The independent `ecdsa`
capability re-exports K256's original key/signature/prehash traits with defaults
and RNG disabled. Auth and native signing fixtures now import those exports;
the root catalog and both direct K256 dependency edges are removed. Direct
boundary policy prohibits K256 in normal/build/dev/target tables, including aliases.
Auth's feature names, prehash bytes, SEC1 admission, high-S rejection, producer
normalization, error precedence and wire/storage contracts remain unchanged.

The incoming lock already selected Platform/macros 0.1.17; preserve that update.
No general-purpose serialization/hash crates are exposed by the published API;
keep those direct dependencies with their current consumers. Testkit owns the
qualification runtime/server boundary. The direct Dalek dev edge solely triggers
permissive feature unification and remains excluded from production graphs.

Before-images, published archive and qualification evidence are retained under
`target/platform-dependency-audit-20261019/`. Focused chain-key, certified,
session/restore and portable signature tests pass, including the adversarial
feature-unification harness. Rust 1.91 isolated package payloads and private
applications pass on native/Wasm; development Wasm and strict native/Wasm Clippy
pass. Direct dependency admission, consumer refusal fixtures and actual graph
exclusions pass. The isolated ECDSA Wasm normal/build graph selects Platform's
`ecdsa` and K256 0.13.4 without Candid, CDK or an OS RNG backend.

Installed Testkit 0.54.1/server 16.1.0 qualification passes: five native fixture
tests and seven real-canister cases, including certified proofs, upgrades,
signed ingress and actual SDK two-receiver admission/logout. Formatting, pins,
local links and the Shared snapshot pass. External registry package identities
match the incoming lock; only dependency edges changed in this task. No full CI,
native macOS, installed-IC production qualification, release or publication was
performed; this does not establish Canic adoption.

## Release editor selection repair — 2026-10-19

The reported clean-shell failure was an Auth adoption defect: metadata required
`CARGO_RELEASE_METADATA_EDITOR` before preflight could prepare tools. The release
fixture pre-exported that variable, masking the normal entrypoint failure.
Source baseline is `e4decd05a475fbcf2cc8d12a864c0065626c5a9c`; its package version
remains 0.10.2 with pending 0.10.3 notes. No package version or release ref changed.

Fresh release preflight now prepares and admits the editor only after source and
notes admission. Rendering independently admits its absolute path from the
validated Git-source lock, with no ambient export or mutable live-lock selection.
Recovery and ordinary checks remain offline and never install. Tests run with
both editor variables unset, assert saved-lock selection, inject setup/check
refusals and check that broken live metadata stays untouched without recovery
installation. Current locked Build 0.20.0 was prepared through the existing
approved setup command; initial missing-source/admission evidence is retained.
The complete local/mocked release/publication fixture passes with the actual
published Build 0.20.0 editor, including fresh-shell patch/minor/major entrypoints,
setup/check failure preservation, invalid-live-lock saved-source admission,
interrupted preparation and exact resume with no recovery installation.
Source-following ShellCheck, Make jobserver, snapshot, grouped formatting, pins,
local links and diff checks pass. Evidence, initial missing-tool refusal and
before-images are under `target/release-editor-selection-20261019/`.
No full CI, native macOS or real release/publication was performed.

Read-only sibling inspection confirms the same preflight export requirement on
IC Jobs `14d9005bee1d6a9d5ded9e68155024bae34df7fe`; Build and Metrics already select
inside their adapters. Shared remains pinned at `2a79f5f`; its renderer requires
an explicit path, while normal caller selection and setup remain consumer-owned.
The common caller/test gap is reported on
[Shared #273 — Adopt the IC Build Cargo editor](https://github.com/dragginzgame/shared-tooling/issues/273#issuecomment-6265505248).
Shared main has since advanced to `b66071b`; its own setup wiring does not update
frozen consumer adapters automatically. No sibling files or refs were edited.
Earlier manual-export guidance below is historical and superseded by this repair.

## Shared Tooling 0.25.0 source adoption

The snapshot now selects remote main commit
`2a79f5f93383f3d2a8a38d65406c7d52c2f77913`, whose version annotation is 0.25.0;
this is committed source adoption, not a claim of a matching published tag.
The canonical exporter preview and refresh retain the selected closure, retire
both Perl Cargo editors and add the upstream transaction/context helpers.
Shared now owns grouped formatting defaults, so paired local overrides are gone.
Dependency-role labels, release ordering checks and actual normal/build graph
exclusions remain in place; the snapshot alone does not qualify deployment.

Auth's release adapter calls the published IC Build editor with its bounded
manifest/lock interface. The caller explicitly prepares it using
`make install-release-tools` and retains the absolute admitted path in
`CARGO_RELEASE_METADATA_EDITOR` for the whole attempt. Auth owns the 8 MiB limit,
validation receipts and recovery checks. Saved-source member context comes from
Shared's Git reader rather than today's member files. The local manifest wrapper
is removed. CI explicitly prepares the optional editor before release fixtures;
ordinary checks never install it. Existing source and external lock selections,
including the incoming Host 0.28.1 update, are preserved.

Snapshot integrity, grouped formatting/hooks, dependency pins and local links
pass. Native/Wasm normal/build graph admission, consumer refusal fixtures,
Make jobserver handoff, tools/Testkit/client failure-evidence suites and the
release-runner command-stub suite pass. Tracked modes and package license inputs
are regular files; dependency selections and the incoming lock bytes are intact.
The GitHub description matches the implemented library/private-client scope.

The lock-selected IC Build 0.19.0 editor was explicitly prepared with maintainer
approval and admitted offline. The release/publication fixture passes against its
actual published renderer: coordinated requirements, external-lock preservation,
source refusal, local tag/push and exact resume, interrupted preparation with
both restored and committed dispositions, and uncertain upload reconciliation.
Git effects stay in local fixture repositories; registry/upload effects are
mocked. Before the next real release, export the admitted absolute path using
`CARGO_RELEASE_METADATA_EDITOR="$(make -s release-tools-check)"`.
[Auth #77 — Adopt Clap for native tooling CLI argument handling](https://github.com/dragginzgame/ic-auth/issues/77)
records the existing handwritten native parser under the new baseline; no broad
CLI refactor is folded into this adoption. Policy/snapshot adoption does not
claim that this independent conformance work is complete.

Before-images, preview and retained evidence are under
`target/shared-tooling-025-20261019/`. No full CI, native macOS, installed-canister
qualification, real release, commit, push or deployment is performed.

## IC Platform 0.1.16 — Ed25519 adoption

Selected non-yanked published Platform 0.1.16, archive SHA-256
`c11b873bf01a0a82774b073a3c0489ec6aa096eb1aacda7b438251a98e81d888`,
source `af02ca56327f239d05538bfa773dcdcf4956c1ca`. Its optional `ed25519`
capability re-exports the original Dalek types with defaults disabled. Production
and fixture cryptographic imports now use it; Auth's direct Dalek edges are
removed. Canonical-scalar checks, weak-key refusal, strict verification, budget
ordering, errors, public APIs and wire/storage contracts are unchanged.

The unpublished feature-unification consumer retains one direct Dalek dev edge
solely to enable `legacy_compatibility`; all API imports still use Platform. It
reuses the canonical signature/revocation suites to prove Auth remains strict
when another consumer enables permissive upstream parsing. Direct boundary policy
allows this exception only there and prohibits direct curve dependencies.
Native Clippy and the internal MSRV check include the harness.

The incoming Build 0.19 and lock changes are preserved. No external package
identity changed relative to the captured working-tree baseline; Dalek remains
transitive through Platform at curve 4.1.3 / Ed25519 2.2.0. Evidence, published
source and before-images are in `target/platform-dalek-20261019/`.
Portable signature/revocation suites pass both normally and with permissive
feature unification, including canonical-scalar and weak-key refusal. Session,
replay, logout and restore suites pass. Isolated Rust 1.91 native/Wasm package
checks and internal apps, development Wasm checks, strict native/Wasm Clippy,
actual graph/refusal checks, formatting, pins, local links and the 118-file
snapshot pass. The production portable graph has no Dalek default or legacy
features; the adversarial graph alone enables legacy parsing.

The selected Testkit 0.54.1 CLI/server 16.1.0 was explicitly prepared under the
existing setup authorization. Five native host tests and seven Linux local
Testkit runner cases pass, covering real certification/composed roots, Wire
signed bytes, ingress, checkpoint upgrades and two-receiver SDK logout.
No full CI, native macOS, deployed-network, release or deployment is claimed.
Earlier evidence below is historical.

## Signed-ingress runtime ownership — pending 0.10.2

PR #76 retains one caller-owned Tokio runtime across anonymous, delegated,
invalid and post-upgrade identity calls. Every call still builds a fresh Platform
Agent from its explicit identity, root bytes and existing expiry. No public
session wrapper, dependency, retry or trust policy is added. The branch includes
released 0.10.1 without rewriting shared history; package versions are unchanged.

Focused compilation, strict Clippy, Rust 1.91 checking and fresh release-profile
Wasm pass. The five native fixture cases and all seven installed-canister cases
pass with the explicitly prepared locked Testkit 0.54.1/server 16.1.0 pairing and
Platform 0.1.13. The first nested-checkout run exceeded Unix socket path limits;
its logs are retained. Rerunning the same build with a shorter repository-owned
state directory passes, including real SDK admission/logout, upgrade and signed
ingress. Evidence is in `target/backlog-20261018/pr76-*` and `target/q76.qTI8lF/`.
Native macOS and full hosted checks remain required before integration.

## Compatible 0.10.1 issue work

Auth #71 now uses a filesystem-valid Unicode name for shared Unix hashing
coverage, retaining spaces/tabs/newlines, ordering, duplicates and zero limits.
Actual invalid-UTF-8 filename creation runs on Linux; invalid raw input still
receives all-or-nothing rejection on every supported Unix host. Linux tests pass;
native Intel/ARM macOS qualification remains required on the delivered source.
No production path protocol or hashing behavior changes.

Auth #72 removes the unused direct protocol-types edge from native tooling.
The canister SHA-256 edge is required: `#[path = "../../sessions.rs"]` imports
the shared authority fixture, whose Ed25519 binding hashes its key. The attempted
removal correctly failed Wasm/Clippy checks and was restored with an ownership
comment. All external lock identities are retained. Failure and final evidence,
registry observations and before-images are in `target/canic-issues-20261018/`.

Read-only Canic observation uses base `5583496b` plus active dirty owner changes;
its captured catalog now selects Auth/types 0.10 but not `ic-auth-wire`. The
access facade deliberately returns Anonymous, CallerMismatch or AuthorityUnavailable
and has no successful session branch. This is a safe integration placeholder,
not proof that the shipped Auth engine is missing authorization. Canic #491/#525
own connecting and qualifying it with protected state and the real caller/clock;
#507 owns non-issuing issuer recovery. No sibling build, source or refs changed.

Ten tooling cases, strict tooling/Wasm Clippy, affected native/Wasm Rust 1.91
checks, the release-profile fixture Wasm, generated-client contracts, package
consumer refusals and actual graph admission pass. Format, pins, links and snapshot
pass. No new installed-canister or native macOS qualification is claimed.
Auth #66 is closed as already delivered in 0.10.0; the older PR #68 proposal is
superseded by that implementation and remains separate from this working tree.

## Tool failure evidence fixture

The isolated tools-evidence consumer now includes the catalog reader's declared
`StableVersion.pm` and IC pin-parser companions. Previously the missing Perl
module caused an unrelated preflight failure before the injected Cargo refusal.
Phase assertions now print unexpected status/output immediately. The focused
`make test-tools-evidence` passes, including logger-failure status preservation,
no later product dispatch, redirect exclusions and all nine retained-file hashes.
Evidence is in `target/tools-evidence-fix-20261018/`; shared payloads are unchanged.

The Testkit caller fixture also now copies the CI-sealed dependency policy.
Previously the real runner refused the missing input before reaching the intended
Testkit admission failure. A policy-drift case proves that changing it stops CI
before tool/Cargo dispatch; unexpected early failures now print the actual log.
`make test-testkit-tools` passes, including setup/check ordering, lock changes,
failure propagation, frontend admission and retained CI evidence. Task evidence:
`target/testkit-fixture-fix-20261018/`. The incoming Cargo.lock update is preserved.

## IC Platform 0.1.12 — direct CDK removal

Selected non-yanked published Platform 0.1.12, archive SHA-256
`e0ab384614a5d8fe251bc2c27c508263d4001d776edf31382b5b985a1a4ced2d`,
source `338b14baf97690abdfc0e386d6d5261b309a3ac8`. Its runtime now exports both
certificate helpers requested in [Platform #21](https://github.com/dragginzgame/ic-platform/issues/21).
The qualification host selects `cdk`/`certification`; all runtime calls and
init/query/update/pre/post-upgrade macros use `ic_platform::cdk`. Both direct CDK
declarations are removed. Direct policy prohibits CDK and Platform's implementation
macro package, including aliases and non-normal dependency declarations.

Only Platform is updated and its coordinated macro package added; existing
upstream runtime/crypto versions and the single executor identity are retained.
The host admits Platform's required Candid runtime bundle, while pure/passive and
private Wire-schema graphs continue refusing Candid/CDK. The feature refusal
fixture now applies to schemas; its first rerun incorrectly emitted Cargo feature
records in a package-only observation, then was corrected. Application protocols,
hidden/custom Wire hooks, decoder limits, controller checks, root composition,
checkpoint bytes/allocation and upgrade rollback policy are unchanged. There are
no remaining direct Candid, CDK-family or Principal package declarations.
Transitive Candid/CDK remain; this does not establish their absence from Wasm.

The isolated Rust 1.91 native/Wasm package capability matrix and internal apps
pass with input/archive seals intact. Actual dependency graph/refusal checks,
strict native/Wasm Clippy, formatting, dependency pins, links and the 118-file
snapshot pass. Five native host tests and seven runner cases pass on the explicitly
prepared Testkit 0.54/PocketIC 16.1.0, including real certification/composed roots,
malformed/unauthorized Wire refusal, failed-upgrade rollback, checkpoints, signed
ingress and two-receiver SDK admission/logout with exact proof bytes. Evidence,
published archives, failures and incoming before-images are retained under
`target/platform-next-20261018/`. Earlier Platform sections are historical.
No full CI, native macOS, deployed-network, commit, release or deployment is claimed.

## IC Platform 0.1.11 follow-up

Selected non-yanked published Platform 0.1.11, archive SHA-256
`8d55106ea1d1ccd65bd67c7d2a681b7d8199f3c0206ae990e24589b3ce502bdf`,
source `ffc36b6363a1c87a4f1f4e88ee14b0ddf55db167`. All other locked package
identities are preserved. Published source differs from 0.1.10 only by additional
CDK counter/cycle exports, a capability not currently selected by Auth. Principal,
crypto, tree, Agent and raw-system implementations remain unchanged.

The release still lacks both `certified_data_set` and `data_certificate`, including
their raw operations. [Platform #21](https://github.com/dragginzgame/ic-platform/issues/21)
continues to block removing the qualification-only direct CDK declaration; no
unpublished sibling, dependency alias or local certificate implementation is used.
The earlier 0.1.10 sections below record historical adoption evidence.

The isolated package Rust 1.91 native/Wasm capability matrix and internal app
checks pass, with archive/input seals intact. Actual dependency boundaries and
refusal fixtures, strict native/Wasm Clippy, formatting, dependency pins, local
documentation links and the 118-file snapshot pass. All five native host tests
and seven runner cases pass, including six local PocketIC scenarios, real
certificates, signed ingress, Wire proof bytes, checkpoint upgrades and two-receiver
SDK admission/logout. The initial sandbox run refused localhost binding; the
same prepared Testkit 0.54/server 16.1.0 and built Wasm pass outside that restriction.
Evidence and incoming before-images are in `target/platform-0111-20261018/`.
No full CI, native macOS, deployed-network, release or deployment is claimed.

## Jobserver fixture and CDK removal follow-up

The jobserver harness now substitutes both newly added boundary entrypoints,
matching the actual Make recipe. Its Cargo substitute checks the exact consumer
argument with spaces and inherited descriptors. This qualifies Make dispatch,
not the dependency checker's policy implementation. Focused harness evidence is
retained in `target/jobserver-platform-20261018/`.

A fresh live registry check still selects published Platform 0.1.10. Removing the
qualification host's final direct CDK dependency remains blocked by the absent
`cdk::api::{certified_data_set, data_certificate}` exports, tracked in
[Platform #21](https://github.com/dragginzgame/ic-platform/issues/21). The published
raw API also lacks those certificate operations. No local runtime replacement,
dependency alias or unpublished sibling selection is introduced. Public Auth
libraries have no direct CDK declarations; the fixture exception stays explicit.

## Shared graph observation adoption

The actual boundary script delegates observation, family exclusions and optional
crypto identity caps to Shared. Local policy sets and feature selections remain
unchanged. Full rendered source labels no longer collapse to name/version alone.

The snapshot selects committed Shared topic source `6eb8c41` from PR #285
(version annotation 0.24.0; unmerged, not a new release). Local package roots,
feature matrices and policy remain consumer-owned. Graph qualification runs
locked/offline against the copied/current source; it does not qualify deployment
artifacts, native macOS or installed-IC behavior.


## Supported boundary

The latest local release is 0.10.0 at `1d9f9397c1599e90ad6f37bb57f1952d354be324`.
All three packages are published and unyanked; adoption uses IC Wire 0.4 and
Platform 0.1.12. Earlier release/adoption observations below are historical.
Auth owns portable proof/certificate semantics, checked protocol types, local
session/replay/revocation machinery and its bounded Wire transport. The private
browser client coordinates admission and durable logout through injected identity,
issuer and storage boundaries. There is one application authentication flow:
proof admission, then caller-bound local session checks.

Canic owns fleet/member/role authority, protected issuer enrollment and renewal,
actual endpoints/guards, clock/caller acquisition, certification publication and
atomic host persistence. Toko owns account linking and application assets. Neither
is a dependency of Auth. See the [current adoption contract](../canic-adoption.md)
and [ownership design](../design/extraction.md).

Production wallet authentication, issuer/reconciliation, protected Root enrollment
and a production per-operation durable session backend remain unimplemented here.
The IC Memory-backed checkpoint and the CDK canister are qualification fixtures,
not those production capabilities. Both proof mechanisms share the same session
engine; standalone signature-store prepare/retrieve is a reusable primitive, not
the retired application-token lifecycle.

## IC Platform 0.1.10 and Wire qualification

Selected published `ic-platform` 0.1.10 after verifying its non-yanked registry
entry, archive SHA-256
`1d3eaa384ad518116caaece25b1aca069473b7010e46e17d2fa23f0a3e1d2898`
and source `4bf7341023939cda613f814a844142196d0ef374`. The protocol uses its
feature-free Principal re-export with unchanged upstream 0.1.5 identity and
serde/self-authenticating/conversion support. Native Agent/delegated identities
also come from Platform; no sibling path or local runtime facade is introduced.

The maintainer requested retiring the private Candid qualification architecture
and explicitly approved retiring its checkpoint format. Every fixture control,
query, diagnostic, init and upgrade argument now uses bounded IC Wire. The
Candid wrappers, projection models, frozen Candid vectors and service export are
removed. There are no direct Candid or Testkit Candid declarations, and the direct
policy now refuses both. CDK remains for runtime and raw hidden codec exports;
its Boolean controller helper and unexpected-result panic are preserved.
The private Wire schema's native/Wasm graphs reject Candid/CDK/tooling. The host
graph rejects Platform's Candid feature; negative fixtures protect both refusals.
[Platform #1](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6249604698)
and [#2](https://github.com/dragginzgame/ic-platform/issues/2#issuecomment-6249605141)
now record Auth's narrower remaining runtime requirement and retired limits exception.

The fixture checkpoint has Wire method identity `0x41555448f100` and `AUTHWIR2`
framing. Old `AUTHCELL` Candid bytes refuse without reset or migration. IC Memory
allocation and public/production storage contracts are unchanged. Restoration
still requires fresh matching controller authority and preserves exact retry,
leases, replay and denial state. The host now uses canonical `RootKeyPolicy`
instead of a duplicate domain DTO; the Wire projection remains explicitly bounded.
Application Wire contracts and the generated browser client are unchanged.

Published signing/key/hash, complete certificate-tree, signature-verifier and BLS
APIs now satisfy [#16](https://github.com/dragginzgame/ic-platform/issues/16#issuecomment-6250254370),
[#17](https://github.com/dragginzgame/ic-platform/issues/17#issuecomment-6250255038) and
[#18](https://github.com/dragginzgame/ic-platform/issues/18#issuecomment-6250255509). Implementation,
tests and qualification imports all use Platform; the four corresponding direct
upstream catalog/member declarations are removed and prohibited by policy.
The same upstream versions and crypto identities remain. Public Auth feature
names and behavior are unchanged; their optional features forward only required
Platform capabilities. BLS signing is selected only by native fixtures.

The new CDK boundary still omits certificate read/publication helpers, now tracked
in [#21](https://github.com/dragginzgame/ic-platform/issues/21). Keep one coherent
existing CDK frontend/runtime until the complete host surface is supported.
The bundled `cdk` feature enables Platform Candid, so a later host migration must
review that gate while retaining Wire-only schemas/exports. Counter-zero samples
use the identical raw Platform API; controller, clock and lifecycle stay intact.
Certified verification still reaches upstream Candid for Principal; CDK and Agent
also retain transitive Candid. No unpublished patch or smaller-Wasm claim is made.

Principal-only and portable/chain-key graph exclusions now explicitly include
independently selectable BLS. The older package-consumer test still expected all
Platform use to fail; its obsolete expectation is corrected, with an admitted
Platform control and retained external-lock/identity refusals. Direct dependency
checks, actual native/Wasm graphs and new BLS-negative fixtures pass. An isolated
published BLS-only consumer builds at Rust 1.91 on native/Wasm and excludes
creation, certification, Candid, ic0, CDK/executor and host tooling.

Testkit remains 0.54 with the explicitly prepared PocketIC 16.1.0; other locked
package versions and public package release metadata are unchanged. The 52
focused store/verifier/certified subjects pass. The exact isolated package MSRV
native/Wasm matrix and internal apps pass with archive/input seals intact.
Strict native/Wasm Clippy and boundary checks pass. Testkit 0.54/PocketIC 16.1.0
passes all six live scenarios plus the shared authority-projection case, including
real certification, composition/cleanup, failed-upgrade rollback, checkpoints,
signed ingress and actual two-receiver SDK admission/logout. The five host unit
cases also pass. Format, pins, links and the 118-file snapshot pass. Current
archive, before-images, graph/MSRV and test evidence are in
`target/platform-0110-20261018/`; earlier Wire qualification is retained under
`target/platform-wire-20261018/`.
No full CI, native macOS, deployed-network qualification, Git delivery, release
or deployment is claimed. Pending notes remain 0.10.0 for the independent public
Wire 0.4 cut.

## IC Platform 0.1.6 follow-up (historical)

That follow-up selected published `ic-platform` 0.1.6, verified non-yanked in
crates.io with archive SHA-256
`4c2a6b9c458b47f2913811998a9f6cc49394deaebb9a11e9d0a1bffb6b169cf2`.
Its published source is revision `d6edec2661f510297c8c9c77c5b9fa01741092e8`.
Compared with 0.1.5, it adds ledger DTOs, management history DTOs and raw
`debug_print`; Auth has no corresponding consumer, so no new feature is enabled.
Selected Candid and argument-length APIs are unchanged. The delegated/anonymous
Agent identity exports remain absent, and the existing direct Agent, CDK/macro
and Testkit limits exceptions remain necessary. No local replacement is added.
The incoming lockfile already selected 0.1.6; the root requirement now names it.
Unrelated resolver changes to Windows edges were restored, leaving the incoming
lockfile byte-identical. Public API, Wire format, storage and lifecycle are unchanged.
Evidence and incoming working-tree before-images are retained under
`target/ic-platform-016-20261018/`.

Rust 1.91 native/Wasm checks, strict native/Wasm Clippy and the frozen-wire
contract test pass. All three native canister tests and six prepared local
PocketIC cases pass, including signed ingress, two-receiver SDK admission/logout
and checkpoint upgrades. Shared direct checks, native/Wasm graph exclusions,
formatting, pins, links and the 116-file snapshot pass. The reference graph retains
Candid 0.10.38, CDK 0.20.3 and one executor 2.0.0 identity; no host parser, Agent or
Testkit enters Wasm. No full CI, native macOS or deployed-network qualification,
commit, release or deployment is claimed. Pending notes stay at 0.10.0 for the
independent public Wire 0.4 change; this Platform update is qualification-only.

## IC Platform 0.1.5 follow-up (historical)

That follow-up selected published stable release 0.1.5, archive SHA-256
`ad5262d2a30adfd89b55960da7c9faffb5a68b7cab4c219af5958f1ab2ce8fe4`.
Its actual archive was inspected. The incoming lockfile already selected 0.1.5;
the root requirement now names 0.1.5 explicitly. Earlier 0.1.4 evidence below is
historical; the 0.1.9 section above records the current selection.

The qualification canister selects Platform's new `ic0` feature and calls its
exact `msg_arg_data_size` export. Its direct ic0 member/catalog declarations are
removed and the Shared policy now prohibits direct ic0. Checkpoint decoding uses
Platform's new upstream `decode_args_with_config` and `DecoderConfig::new`. This
is exactly how the former upstream `decode_args` constructs its decoder, retaining
the same bytes, work quotas, diagnostic policy, typed error and trap context.
No storage representation or lifecycle policy changes.

Platform's native Agent feature omits `AnonymousIdentity`, `DelegatedIdentity`,
`Delegation` and `SignedDelegation`, required by Auth's signed-ingress fixture.
The existing native Agent dependency stays intact until the full identity surface
is published; no partial dual surface, PEM feature or local identity wrapper is
introduced. Auth's concrete missing exports and acceptance checks are on
[Platform #9](https://github.com/dragginzgame/ic-platform/issues/9#issuecomment-6247978961).
The CDK macro/runtime and Testkit limits exceptions on #1/#2 remain. Native Agent,
parser, management DTO and executor features are not selected by this follow-up.

The only lockfile changes move the ic0 ownership edge from the fixture to
Platform. Unrelated resolver normalization of Windows edges is restored to the
incoming compatible selections; locked metadata admits the retained graph. All
external package identities, own versions, public Rust and the real Git index/refs
are preserved. Evidence and before-images are in
`target/ic-platform-followup-20261018/`.

Rust 1.91 native/Wasm checks, native/Wasm strict Clippy, the frozen-wire contract
case, all three native canister cases and all six local PocketIC cases pass.
Shared direct/alias/target fixtures, actual native/Wasm exclusions, formatting and
hook preservation, pins, links and snapshot checks pass. The initial sandboxed
live attempt failed at loopback binding; the unchanged prepared server passes
outside that restriction. No full CI, native macOS, deployed-network qualification,
source commits, release or deployment is claimed.

## Initial published IC Platform adoption

Selected `ic-platform` 0.1.4 from the live crates.io sparse index, then inspected
its published archive and verified checksum
`174b06835875ba583f2ac542cf12edc3967b68fad717fd4ad9e85d9e8630b627`.
The published API supports Candid contracts/encoders, bounded decoders, an
optional executor and explicitly native-only host parsing, but not CDK system
APIs or entrypoint macros. The final registry observation found 0.1.4 published
after the initial 0.1.3 qualification; its Candid implementation is byte-identical,
and affected checks passed again with 0.1.4. Earlier archive/log evidence remains
retained. Only qualification
packages select its `candid` feature; no sibling path or unpublished API is used.

Private DTOs, derive paths, encoders and standalone bounded reply decoding now
use Platform. Two direct Candid package edges are removed (contracts and native
runner), and the projection's private `types::internal` reference uses the public
Platform contract. Three frozen pre-adoption role/audience/signature wire vectors
and byte/quota refusals pass. The native SDK control reply uses existing finite
control ceilings instead of its formerly unbounded decoder. No application Wire
schema, generated browser output, public-library implementation or storage layout
changes. Existing Candid projections remain necessary fixture adapters.

The canister retains direct Candid/CDK for macro-generated paths, runtime APIs,
service export and original checkpoint decoding, plus low-level ic0 argument
admission ([Platform #1](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6247608944)).
The native runner retains `ic-testkit-candid` because selected Testkit 0.53 APIs
require that limits/error type. A field projection shares the existing five
ceilings with Platform; no new decoder/transport is implemented
([Platform #2](https://github.com/dragginzgame/ic-platform/issues/2#issuecomment-6247609101)).
Both existing blockers now carry Auth's concrete consumer/acceptance evidence.

The Shared direct-dependency checker and its alias/build/dev/target/optional
fixtures are added through the unchanged 0.24.0 exporter. Its explicit policy is
sealed as a CI input. Public native/Wasm graphs reject Platform and host parsers;
the reference-Wasm graph rejects native tooling/parsers and requires exactly one
executor version/source identity. Duplicate tree visits are accepted; a second
version or source refuses. Root catalog entries were separately audited because
the Shared checker does not inspect unused catalog entries. Role comments and
external deployment source/artifact binding are still not automatic admission.

One new external package is added; all incoming external selections, own package
versions, public Rust, Git index/refs and unrelated pending work are preserved.
Rust 1.91 native/Wasm checks and strict Clippy pass for the affected qualification
packages. The frozen-wire contract case, all three native canister cases and six
local PocketIC certification/ingress/session/upgrade cases pass. Initial live
startup was blocked by sandbox loopback binding; the authorized unsandboxed rerun
uses the same selected server and passes. The first native floor check exposed a
missed child-module import, corrected before the final passing checks. Source,
registry, before-images and logs are retained in
`target/ic-platform-adoption-20261018/`.

No full CI, native macOS, deployed-network/production-IC qualification, release,
publication or source edits in siblings are claimed. The installed-IC callback
qualification needed to retire the CDK exception remains upstream work; this
adoption leaves that runtime unchanged.

## Shared Tooling 0.24.0 adoption

The 0.24.0 adoption initially selected 114 files; Platform boundary checks bring
the current snapshot to 116 canonical files at
`cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`. The exporter preview and refresh retain
all directly used entrypoints, including Auth's release fixture increment helper.
The new common Rust selection contains only Cargo sort and sort-derives; Auth has
no Candid extractor caller. The reduced bundle is prepared and offline aggregate
tool admission passes; older retained bundles are not removed.

Root dependency-role groups and member annotations separate production libraries
from qualification/native tooling. Certified verification reaches Candid, ic0 and
BLS transitively, so those identities remain production entries. Manifest values, package versions and the real Git index are preserved from the
incoming dirty tree. This adoption does not update dependencies. A concurrent
Cargo.lock change from IC Memory 0.48.1 to 0.48.2 appeared during qualification;
it is retained separately and is not attributed to this adoption.

Formatting and hooks use grouped ordering. Current release preflight checks it;
prepared metadata writes/checks it; committed checks use the canonical Shared
metadata helper against the selected Git source. The release fixture runs the
canonical companion-closure preparer against its independent copied source, with
substituted release/publication effects. Hook fixtures now perturb entries within
a production group instead of skipping dependency-table qualification.

Dependency admission covers normal/build transitive graphs for each library's
default, individual Auth capabilities, session/portable and combined capabilities
on the native host and Wasm. Wasm-only Agent, Tokio and Testkit leaks refuse in
negative fixtures. This closes the prior native-only exclusion coverage and
missing Agent/Tokio rules. The all-capability Wasm crypto-generation check also
includes build edges. Role labels themselves do not enforce graph admission.

There is no production deployment entrypoint here. Qualification intentionally
uses Memory/CDK and native Testkit. Canic/future service deployments still must
bind their exact source, lockfile, target/features, successful build and artifact
digest and run their own exclusions. Unlisted feature combinations and native
macOS are not qualified by this Linux adoption. No production artifact, release,
publication, deployment or sibling edit is claimed. Task evidence and before-images:
`target/shared-tooling-024-20261018/`.

Focused checks pass: grouped write/check formatting, staged preservation and
within-group sorting refusals, snapshot and pins, aggregate prepared tools,
package-consumer graph/lock refusals, native/Wasm boundaries, local links and
ShellCheck. Shared runner substitutions and Auth's complete local/mocked
release/publication fixture pass, including old selected-commit checks with an
unsorted current manifest. The first release-fixture attempt was invalidated by
an in-flight edit to its script; its failure log is retained alongside the stable
rerun. The release fixture captured the earlier lock selection; graph/metadata
admission is rechecked against the concurrent Memory selection, without claiming
its runtime/upgrade qualification. No full CI or Rust compilation suite ran for
this tooling change.

## Architectural cleanup in this working tree

Source inspection follows the adopted Shared Tooling flow-convergence and
complexity methods at `0b351cd2ba2bcd8927b783a5fec89afc8013f708` and Auth's
AGENTS overlay. It starts from released 0.9.1 plus the maintainer's incoming
Wire 0.4 and Testkit 0.53 manifest/lock selections, retained under
`target/sediment-audit-20261018/before/` for attribution.

Native package admission now calls published Build 0.16's bounded external-lock
comparison through Auth's existing no-follow reader. Auth keeps exact package,
source, feature and release policy plus its explicit 8 MiB per-lock allowance.
The duplicate yq/jq comparison is removed; Build is refused in public-library
graphs. The unused private CanonicalId Candid projection is removed; controller
and IC certificate projections remain because the live fixture uses them. The
incoming Testkit 0.53 selection had left seventeen callers on its removed
CandidCallExt API. They now call the canonical tuple-argument query/update
functions directly and project the one returned argument, preserving principals,
method/payload identity, application errors and explicit decoding limits.

README, instructions, ownership design and adoption guidance distinguish shipped
libraries from planned wallet/service work. Earlier handoffs and consumer notes
are preserved in [handoff history](history-through-0.9.1.md) and
[adoption history](../canic-adoption-history.md); they are evidence, not current
API alternatives or active task queues.

The incoming Wire 0.4 selection is a public dependency-identity change because
`ic-auth-wire` exposes Wire types. Pending notes therefore select 0.10.0; own
package versions remain 0.9.1. The cleanup introduces no additional public API,
signed-byte or retained-record change. The library enum representation is not
changed speculatively as part of this audit.

## Integration and follow-up ownership

The inspected Canic base is `5583496bfa95069809249e8e5d16c77f1ffd744f` plus
active dirty work: its catalog selects Auth/types 0.9 and its Root adapters use
`ic_auth::delegation`. Toko at `44d4e2c6d41e3575b2503e79ffa369129ff3ecf2`
still selects Canic `=0.110.48`. These are read-only source observations, not
consumer builds or deployed acceptance. Siblings were not edited.

- [Auth #66 — Adopt Build external-lock subset admission](https://github.com/dragginzgame/ic-auth/issues/66)
  owns delivery of the native-tooling replacement. The older PR #68 branch
  remains separate from these working-tree changes; no push or merge is implied.
- [Auth #3 — Qualify portable browser admission/logout with a production issuer adapter](https://github.com/dragginzgame/ic-auth/issues/3),
  [Auth #58 — Support host-selected session persistence with safe volatile reset admission](https://github.com/dragginzgame/ic-auth/issues/58),
  and [Auth #59 — Bound expensive invalid-proof admission](https://github.com/dragginzgame/ic-auth/issues/59)
  retain production qualification, not another application-token API.
- [Canic #491 — Extract reusable authentication, token and session machinery](https://github.com/dragginzgame/canic/issues/491)
  owns actual consumer endpoint/enrollment/transaction acceptance.
- [Testkit #119 — Own typed IC Wire query/update adapters](https://github.com/dragginzgame/ic-testkit/issues/119)
  and [Host #76 — Consider a published bounded filesystem CLI](https://github.com/dragginzgame/ic-host-tooling/issues/76)
  own proposed extractions. No published replacements were found in the inspected
  owners; the small current adapters remain the runnable integration paths.

## Validation

Before cleanup, the nine native tooling tests passed with the incoming Wire 0.4
selection. The initial minimum-compiler run exposed the pre-existing Testkit 0.53
stale imports/calls; the first adaptation also missed a child-module import.
Both refusals are retained alongside the corrected results under
`target/sediment-audit-20261018/`.

The final full isolated Rust 1.91 native/Wasm package capability matrix and
internal application compiler checks pass, with unchanged source/archive/lock
seals. All ten native tooling cases, package-consumer refusal fixtures, 44 browser
cases and Rust/TypeScript Wire directions, and three native canister cases pass.
Native tooling/qualification Clippy, actual dependency boundaries, formatting,
shell lint, pins/inheritance, local links and the 109-file Shared snapshot pass.
All 373 incoming lock selections are retained; ten Build-native selections are
added. Historical documents are verified as verbatim archive suffixes.

Initial offline admission refused the missing selected Testkit 0.53 CLI. After
explicit maintainer approval, `make install-testkit-tools` prepared/admitted that
CLI with server 16.1.0. All six `make test-qualification` live-canister cases pass,
including exact Rust/TypeScript certified proof bytes, real signed ingress,
two-receiver SDK admission/logout, local denial and checkpoint upgrades. The
initial missing-tool refusal is retained; no older CLI fallback was used.

No full CI, new release/publication, native macOS, production issuer, consumer
deployment or real-checkout Git index/ref mutation is claimed.

## Second architectural pass

The repeat audit preserved the preceding working-tree cleanup and dependency
selections. It found stale active guidance beyond the earlier archived handoffs:
Candid browser/logout schemas, pending 0.6/0.7 language, the old IC Memory
re-export/compiler floor and contradictory native dependency selections. Current
guides now point to the implemented domain/Wire contracts and canonical dependency
inputs. Signed domains and storage framing remain unchanged.

The local dependency policy previously excluded Testkit/PocketIC only for packaged
consumers. The shared native-tool exclusion now also covers ordinary default and
capability graphs, with positive and near-name refusal cases exercised through
both rg and jq. Package-consumer fixtures, actual dependency graphs, ShellCheck,
local documentation links, snapshot integrity and the diff pass. Evidence is in
`target/sediment-audit-second-20261018/`; no Rust/runtime changes required repeating
the preceding compiler or live-canister qualification.

Read-only inspection of Testkit, Host, Memory and Canic confirms the remaining
adapters have distinct jobs: Testkit still supplies no typed Wire call replacement;
Memory's fixture cell explicitly leaves payload identity/codec validation to its
caller; Candid projections serve controller and IC certificate fixtures only.
These observations do not establish deployed Canic adoption or a production
durable backend.
