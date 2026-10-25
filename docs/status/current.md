# Current handoff — 2026-10-24

## Delivered source and local work

The maintainer completed release **0.16.0** at `bf0b9ce`; its saved release plan
is marked complete. The earlier instructions to restore the 0.15.4 release base
are obsolete and must not be repeated.

Pending **0.16.1** preserves the incoming lockfile selection of published
Host/Build/Testkit **0.68.1** and Memory **0.51.0**. Platform/macros **0.6.15**,
Wire/derive **0.7.6**, the independent File CLI **0.31.1** and Shared Tooling
**0.26.14** (`e14050bcc0c627f9c8a9064051699cac44fd7ab3`) are unchanged.
Tooling's published source is `c9c5ef7c44e3d0c3b23c8531a5d43c1800656c0a`.
The selected Tooling crates have identical `src` trees to their previous patch
release. Memory 0.51.0 changes the separate Metrics adapter release boundary,
which Auth does not consume. The pending Wire 0.7.6 lock update is preserved. The updates align published dependencies and producer
qualification; Auth uses no new API or storage format. No capability is enabled.

The browser cleanup batch seals generator inputs before compilation and refuses
build/generation drift before output publication. Testkit busy status 75 now
recommends retry rather than installation. Private logout claim/scheduling/result
transitions remain synchronous under the existing journal transactions, preserving
public APIs, stored format, attempt limits and deadlines. Browser byte caps are
explicit local policy: the generated credential limit is 512 KiB, while admission
defaults to 128 KiB. A structurally valid larger certified envelope is covered;
this does not establish a cryptographically valid proof or a generator mismatch.

Focused Linux validation: `make test-client` passes 57 browser tests and Rust/TS
vectors with unchanged generated sources; `make test-client-tools` and
`make test-testkit-tools` pass source-drift and contention regressions using
substituted effects. ShellCheck, formatting and doc links pass. Evidence is in
`target/browser-cleanup/`. This batch did not run live-canister qualification,
macOS validation, the complete release gate or executable installation.

The follow-on cleanup captures the browser issuer as immutable canonical text,
shares only structural authority validation between proof/session paths, and
prepares validated durable session deltas in one exhaustive match before commit.
Private Wire forwarding and session/replay/revocation byte ceilings are
consolidated without changing public signatures, method IDs, bounds or records.
Independent crypto/rule/revocation test fixtures remain separate: they use distinct
keys and clock/grant windows, including fixed signature vectors. Sharing them
would couple evidence that should remain independent.

`target/cleanup-next/` records passing session/admission/refusal/restore suites,
protocol/type/Wire tests, 58 browser tests and Rust/TS interoperability, isolated
Rust 1.91 native/Wasm package checks and native/Wasm Clippy. Tests distinguish
stored-session liveness from proof-admission policy and exercise mutable issuer
input. Generated browser sources are unchanged. Doc links, formatting and diff
checks pass. No live Testkit run, macOS qualification or full CI is claimed.

The audit follow-ups detach configured HTTP trust-root bytes from Buffer/subarray
storage and preserve a separate copy per SDK attempt. Browser contract seals now
include `.cargo/config.toml`. Source/config mutation fixtures cover both build
and generation, in check/generate modes, with no output publication on drift.
`target/audit-followups/` records 59 passing browser tests, Rust/TS interoperability
and client-tool fixtures. ShellCheck, formatting, doc links and diff checks pass.
The root-key regression inspects SDK construction; it does not qualify certificate
verification or replace the outstanding live-canister run. Public APIs and
generated contract bytes are unchanged.

Shared Tooling #306 supplies explicit read-only admission waiting and busy status
75. Auth's saved-source release editor uses a 60-second wait and preserves busy
status without installing, repairing or selecting a partially prepared graph.
Ordinary offline checks remain immediate. The exact saved source continues to
own the lockfile, manifest and editor features.

Wire 0.7 decodes once without reserialization. Auth transport has no normalizing
maps/sets/custom DTO visitors: it preserves lists and opaque signed bytes.
Semantic scope/grant checks remain in Auth. Primitive encodings remain strict;
`Error::Noncanonical` is removed and malformed primitives return `Malformed`.
Keep original frame bytes for exact retries, hashes and signatures.

The native runner passes canonical limits directly to Testkit's `WireCallSpec`.
The temporary alias/helper and response-only Serialize bounds are removed,
adopting [Tooling #248](https://github.com/dragginzgame/ic-tooling/issues/248).
All five generated browser files were refreshed. Only the shared runtime changes,
adding Packed<i8>/Int8Array support; Auth's four schemas remain byte-identical.
The optional Wire bulk-schema advice is not enabled and no fields are migrated.

## Implementation and consumer boundary

Auth delivers both portable proof verifiers, one session engine, host-selected
storage/reset interfaces, bounded Wire contracts and the private browser
admission/logout coordinator. Canic owns topology, authority approval/renewal,
endpoint guards, production enrollment, deployment configuration and its durable
host adapter. The standalone wallet service is not implemented.

The [Canic adoption guide](../canic-adoption.md) describes config, Wire endpoint
wiring, capability profiles and the distinction between `(issuer, subject)` and
browser ingress caller. Consumers sharing Rust Wire types must align to 0.7.
Auth 0.15's admission method ID remains `0x415554480012`. Do not reset portable
journals, replay unknown issuance or restore retired private checkpoint formats.

[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382815988)
retains runtime/store/reset/load acceptance. [Canic #507](https://github.com/dragginzgame/canic/issues/507#issuecomment-6382816186)
retains issuer policy, issuance abuse, reconciliation and browser integration.
Auth #58/#59 were superseded coordination, not production acceptance. Auth
[#3](https://github.com/dragginzgame/ic-auth/issues/3) owns real-browser journal
qualification. The browser package stays private; fixtures do not prove production
Canic adoption, clock skew or TEE custody.

## Retained interfaces and follow-up

`AuthRequestMetadata`, Rust's empty `LogoutRequest` and older session error
conversions remain supported during Canic integration. See the
[consumer review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382404160).
This adoption does not change certificate freshness handling for
[Platform #51](https://github.com/dragginzgame/ic-platform/issues/51).
[Auth #85](https://github.com/dragginzgame/ic-auth/issues/85) retains the
crypto-bigint advisory correction/exemption and qualification through
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).

## Validation and evidence

Current adoption evidence is under `target/deps0681/`, including the incoming
lockfile and diff. The lockfile update predates this task and is preserved.
The selected Testkit and Build 0.68.1 executables are missing; explicit setup
approval is pending before real-canister and release/recovery qualification.
Type/transport and Host tooling tests, dependency boundaries, isolated native/Wasm
MSRV, Clippy and all six fixture unit tests pass. Snapshot integrity, dependency
pins, formatting and local documentation links pass; the incoming manifest/lock
selection remained unchanged. Earlier 0.68.0 qualification is recorded in
`target/upstream-adoption-*.log`; it does not qualify the new executable selection.

No release, commit, push, publication or deployed application update was run for
this batch. The root package version remains 0.16.0; notes target 0.16.1.

Memory 0.51 adoption evidence is retained under `target/memory051/`. Its manifest
requirement moves from 0.50 to 0.51 only in the native/test catalog; production
Auth libraries still do not depend on Memory. The published Memory `src` tree
is byte-identical to 0.50.17. Type/transport and all 54 browser tests, six native
checkpoint fixture tests, dependency boundaries, native/Wasm MSRV and Clippy pass.
The incoming Wire 0.7.6 selection regenerated all five browser files with no byte
changes. Snapshot, pin, formatting and documentation checks pass. Manifest/lock
inputs stayed fixed. Live qualification remains pending the previously requested
Testkit 0.68.1 setup approval; no old executable was substituted.
