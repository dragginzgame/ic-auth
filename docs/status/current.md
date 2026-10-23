# Current handoff — 2026-10-23

## Delivered source and pending adoption

Auth **0.15.0** is delivered on remote main at
`089d49d5fb92e46450638ce7fe6cfcfdafc53707` and available in the registry.
Package versions remain 0.15.0. Pending **0.15.1** notes cover published Host
FS/Process, Build and Testkit **0.66.0**, plus Shared Tooling **0.26.8**.
This is compatible at Auth's public boundary: APIs, signed material, method IDs,
transport bytes, storage and browser journal formats are unchanged from 0.15.0.

The incoming lock already selected Wire/derive **0.6.7**; it is preserved.
Platform/macros remain **0.6.9**, Memory **0.50.12** and the separately pinned
file CLI **0.31.1**. Only the requested five Host/Build/Testkit package selections
changed during the bounded update. No sibling/path overrides or storage migration
were added. No own version change, commit, push, release or publication ran.

Shared Tooling is the **119-file 0.26.8** snapshot at
`0b07c2a3163a5a66328e100b553baf93afd8ed07`. This is reviewed committed main source;
its changelog still marks the version unreleased. The canonical exporter retained
the selected file closure and changed only the setup guide and two build helpers.
Its Cargo setup fix tolerates an admitted owner's lock withdrawal, and source
cache initialization no longer uses a global installation lock.

## Ownership and setup

Published Host FS/Process/Artifacts and Build source trees are unchanged from
0.65.0. Testkit 0.66 makes its PocketIC owner fields private. The reference runner
borrows the client during setup with `instance()`, then deliberately takes
`into_parts()` and retains client-before-server teardown in its existing fixture.
Setup panic paths retain the original owner. Auth adds no wrapper, runtime policy
or public capability. Native simulator clients and DTOs still come from Platform;
Testkit owns startup, lifecycle and typed Wire dispatch.

Wire 0.6.7 only reuses equivalent signed-i64 serialization. Its TypeScript
generator/runtime sources are unchanged from 0.6.6. The required explicit browser
regeneration and interoperability checks are part of this batch.

The new build-helper digest requires explicit executable setup. Initial offline
admission correctly refused old-policy selections. An early host-tooling run also
reached the unprepared file CLI; its refusals are retained, not product regressions.
The user approved `make install-tools` and `make install-release-tools`, including
common Rust tools, Testkit/Build 0.66 and the unchanged file CLI under the new policy.
Preparation and affected qualification are in progress. Checks never repair or
fetch tools implicitly. Development Rust remains 1.99.1; package MSRV is 1.91.

## Qualification

Evidence is retained under `target/shared-0268-host-066/`, including the incoming
lock, canonical snapshot preview, published source comparisons and input seal.
Strict native tooling/runner and reference-Wasm Clippy pass. Public types/Wire and
private contract cases passed before the host-tooling setup refusal. Final
browser, MSRV, dependency-boundary, live-canister and release/recovery qualification
will be recorded after admitted setup and focused reruns.

Earlier complete Linux evidence remains under `target/shared-0267-host-065/`
with the original 0.65/Wire 0.6.6 selections. It does not substitute for the new
Testkit pairing. Full CI, native macOS, installed IC and TEE are not established
by this batch. Reference clocks are controlled, issuer operations are injected
probes and IndexedDB is faked in Node; no production skew bound, throughput or
browser persistence guarantee follows.

## Consumer acceptance

Auth 0.15's admission ID is `0x415554480012`; receivers and generated clients must
use that released contract together. Credential/read/logout identities and frozen
frames remain unchanged. Do not reset portable journals or replay unknown issuance.
The unpublished finite-Wire fixture checkpoint remains retired; production memory
and storage are independent.

Production enrollment, retained receiver runtime/budgets and scoped exports remain
[Canic #491](https://github.com/dragginzgame/canic/issues/491). Issuance, lookup-only
reconciliation and browser-key logout remain
[Canic #507](https://github.com/dragginzgame/canic/issues/507); future-start endpoint
acceptance remains [Canic #418](https://github.com/dragginzgame/canic/issues/418).
These tooling updates require no new Canic public API migration. Siblings were
not edited; Auth's private browser package remains unpublished.

[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58) and
[#59](https://github.com/dragginzgame/ic-auth/issues/59) retain production issuer,
persistence/reset and abuse/workload acceptance. Skew activation remains closed;
TEE evidence and production acceptance are separate.
