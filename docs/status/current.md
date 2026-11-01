# Current handoff — 2026-11-01

## Released source and current work

Auth **0.29.2** is published at `529a5212c1730d2cc5df7820d9ecf1a15a086e6b`,
tag `v0.29.2`. The maintainer authorized adoption and delivery of the pending
Auth work, including Memory/Storage 0.62.1 and Wire 0.14.2. Own package metadata
remains 0.29.2 until the standard runner prepares each release. Delivery will use
the isolated checkout `/tmp/ic-auth-release.EqLgnm`, Protocol first, then Engine;
full release gates and registry admission remain mandatory. No new release, tag,
push or upload has occurred yet. Protocol (`ic-auth-protocol-types`, `ic-auth-wire`) and Engine
(`ic-auth` plus private qualification/tooling) own separate literal versions and
`releases/<group>/CHANGELOG.md`. Root notes index those ledgers and retained
coordinated history. Both pending entries now target **0.30.0** because canonical
Principal/parse-error and Wire codec/limits identities change for Rust consumers.
Release Protocol first, then Engine against it. Later Engine-only releases do not
advance Protocol's version or ledger. IcyDB and Canic checkouts are unchanged.

The protocol selects Types directly; the engine selects its self-authentication
capability explicitly. Platform remains optional through crypto features; passive,
default and session-only graphs exclude the facade. Align consumers on Types 0.16
and Wire 0.14; select external Principal conversions explicitly at native boundaries.
Signed bytes, schemas, stored formats, cryptographic checks and TEE/non-TEE trust
policy are unchanged. Platform's BLS identity-key rejection remains covered.

## Adopted suppliers and boundaries

Published registry selections are Platform/PocketIC client **0.17.0**, Types
**0.16.0**, direct Runtime/macros/guard **0.17.1**, Wire/derive **0.14.2**, Testkit
**0.78.0**, PocketIC server **16.1.0**, Metrics **0.25.0**, and Build/Host/file CLI/
editor **0.73.6**, and Memory/Storage **0.62.1**. Persistence now shares Wire
0.14 with RPCs, removing the remaining Wire/derive 0.13 entries. Private Metrics aligns with Testkit's
new selection, retaining one arithmetic provider. Public Auth graphs exclude
Metrics, Memory, Storage, runtime, Candid and native tooling; DHAT stays dev-only.

The qualification host now selects Runtime directly for CDK adapters, IC0 and
entrypoint generation. Graph policies require one Runtime/generator/guard and
reject facade runtime features in the host. Public capabilities still exclude
runtime entirely. The host also requires one Wire provider, with an old/new Wire
duplicate refusal fixture. Native clients/DTOs come from Platform's maintained PocketIC
implementation. The old Principal-schema/client/Wire-limits blockers are cleared
by the registry cohort; no local schema/client shim or dependency patch is used.
Historical failed diagnostics remain in `target/platform-017-adoption.BMVSYc/`
and `target/cohort-adoption/`. Owning history is at
[Wire PR #41](https://github.com/dragginzgame/ic-wire/pull/41),
[Testkit #307](https://github.com/dragginzgame/ic-tooling/issues/307) and
[Tooling PR #310](https://github.com/dragginzgame/ic-tooling/pull/310).

Shared Tooling **0.27.25**, `982a99706e038810ee8108d5356a8637a7d42f67`, is
unchanged. One workspace/lock remains. Compiler-wrapper recipes and validation
proof reuse stay opt-in; Wasm retains the qualified 32-bit backend configuration.

## Current qualification

The 0.62.1/0.14.2 registry patch selections are fetched and locked. Browser contract
regeneration passes with all five generated files unchanged. The earlier evidence
below predates those patches; the isolated delivery must run the complete gate
against the exact committed inputs for each release group. Registry observations
and adoption logs are retained in `target/persistence-wire-delivery/`.

The latest Memory/Storage 0.62 evidence is in `target/memory-storage-adoption/`:
ten native storage/checkpoint cases, Rust 1.91 Wasm compilation, host Clippy,
release Wasm build and eleven live PocketIC cases pass. Occupied fresh-state and
old/malformed/oversized restore refusals preserve backing; allocation/Cell layouts,
Auth checkpoint bytes and recovery policy remain unchanged. Direct/resolved graph
admission and the new duplicate-Wire refusal pass. No Auth Rust API adaptation,
backing reset or format migration was needed; public library graphs remain free
of persistence dependencies. The prior qualified Wasm is retained in that directory.

Evidence is in `target/cohort-adoption/retry/`, including the before inputs,
registry/lock observations, failed diagnostics, generated-file comparisons and logs.
Protocol/Wire/engine all-feature native tests pass. Five admission-metrics and ten
host storage/checkpoint tests pass. Rust 1.91 native runner and Wasm host checks
(including public TypeScript schemas) pass; host Wasm Clippy and release build pass.
Direct/resolved dependency policies and all substituted boundary refusals pass,
including new direct-Runtime/facade-feature cases. The prepared registry Testkit
0.78 CLI admits the existing 16.1.0 server through its actual setup/check owner.

All five generated browser files remain byte-for-byte unchanged. All 69 browser
tests, packed-client acceptance and Rust/TypeScript interoperability pass. Eleven
live PocketIC tests pass: certification/composed roots, protected metadata upgrades,
certified-token admission, signed ingress and SDK admission/logout journeys.
One expensive profiling test is intentionally ignored. This is internal fixture
qualification, not a production issuer, wallet service or installed Canic result.
No full CI, macOS or live IC qualification is claimed.

A selected Engine registry dry run correctly exposed old Protocol's Types 0.15
before the new Protocol package is published (`engine-package-diagnostic.log`).
The complete gate now uses `check-package-builds` to package and verify all three
current archives together through normal Cargo admission. It passes with Types
0.16 (`package-gate.log`); fresh evidence is under `target/package-qualification/`.
No `--no-verify` or consumer dependency override enters this gate or publication.
Actual publication remains explicitly group-selected and registry-admitted.
The Make jobserver, package-refusal and unsafe-mode caller fixture passes.

## Release custody and earlier evidence

Release/resume/publication require explicit `RELEASE_GROUP=protocol|engine`.
Shared owns group/file projection and Cargo/changelog transactions; Auth owns the
complete gate, group/policy-bound receipts, saved-source reconciliation and upload
custody. Tags, journals and publication intents are group-qualified. The primary
version is installed last; old/prepared files are reconciled against saved source.
Legacy publication intents/dispatch markers cannot be bypassed by new namespaces.
Recover historical attempts with their original adapter in their dedicated release
checkout; never reinterpret or discard old journals, receipts or shared locks.

Earlier group evidence is in `target/release-groups-work/`: different-version
archives, both groups' release/recovery/publication cases, final-adapter rollback,
partial-install recovery and receipt/legacy-custody refusals pass with documented
local/substituted effects. Earlier Types and released-cohort evidence remains in
`target/types-boundary.BvpfC1/` and `target/upstream-adoption.2T1aUH/`. Retained
observations keep their original scope; explicit cleanup may remove old artifacts.

## Ownership, measurements and retained state

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns issuer/renewal,
fleet/guards/receiver composition, durable backend and browser acceptance;
[#720](https://github.com/dragginzgame/canic/issues/720) owns storage deduplication.
Historical diagnostics remain at [#705](https://github.com/dragginzgame/canic/issues/705)
and [#724](https://github.com/dragginzgame/canic/issues/724). Last source inspection
selected Auth 0.28; this batch neither inspected nor changed Canic.
The [allocation observations](../canonical-allocation-observations.md) and
[isolated Wasm attribution](../wasm-attribution.md) justify no further encoder/
verifier optimization or result cache and do not measure production admission.

The opt-in backend bundle requires all four scenarios for full
`assert_backend_contract`: admission/retry, logout/reopen, replacement/generation,
capacity. Its [archive instructions](../../apps/qualification/backend-conformance/README.md)
require independently selected published Engine/Protocol identities. Earlier
registry 0.29.1 delivery evidence describes candidate source, not current adoption.
The current bundle requires the pending 0.30 suppliers after publication; its
unchanged 0.29.2 metadata is not registry delivery readiness.
The private browser client still has no npm publication or production issuer.

Memory/Storage remains private qualification support, not a production backend.
Use parsed `StableKey`, fresh-only `StableCell::create` and bounded retained loading;
occupied creation preserves backing. Keep `AUTHNAT3`, session/login
`0x415554485311`/`0x415554485313` and replay `0x415554485302`. The older Memory 0.56
ledger retirement is separate. Never erase backing or its ledger for admission.
Preserve failures, binaries, retained states and recovery custody. Hosts own clock,
allocation, authority refresh, atomicity and certification-root publication.
