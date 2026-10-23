# Current handoff — 2026-10-23

## Delivered source and pending adoption

Delivered baseline is Auth **0.14.4** at
`c84f82b3bbd7b073cd0c960ab311607a735ec358`. Package versions remain 0.14.4;
pending **0.15.0** notes describe the incoming Wire 0.6 transport/API cut.
This batch selects published Host FS/Process, Build and Testkit **0.65.0**,
Wire/derive **0.6.6**, Platform/macros **0.6.9** and Memory **0.50.12**.
The incoming root already selected Wire 0.6, Platform 0.6.9 and Memory 0.50.11;
the later Memory lock update is preserved. No sibling/path overrides, own version
change or Git delivery ran.

Shared Tooling is the **119-file 0.26.7** snapshot at
`3e8d7cd3e7e9b7b79a5e800c30daf0be2710dab0`, including its CI/release integrity
audit. The canonical exporter owns these files. Development Rust remains
1.99.1 and the supported package minimum remains 1.91.

Both lock-selected **0.65.0** executables were explicitly approved, prepared and
admitted: Testkit CLI with PocketIC **16.1.0**, and the IC Build release editor.
The separately pinned file CLI remains 0.31.1. Checks stay offline; no release,
publication or deployment ran.

## Transport ownership and consumer migration

Published Testkit 0.65 supplies native Serde query/update calls with an explicit
`WireCallSpec`; the former Wire 0.5 blocker is resolved for local adoption.
The runner imports simulator types from Platform and uses Testkit for startup,
call dispatch and bounded decoding. Auth does not recreate the retired `Method`
trait, finite serializer or typed retained-byte API.

Public Auth contract markers now provide concrete native encode/decode functions
with Auth-owned IDs and limits. DTOs use Serde; only the host generator enables
Wire metadata. All five generated browser files were reviewed. Admission changes
its ID from `0x415554480002` to `0x415554480012` because refusal tags now follow
zero-based Serde order. Update receivers and generated clients together; retired
admission frames are refused rather than reinterpreted. Credential/read/logout
identities and frozen frames are retained. Signed claims, production
session/replay/login records and portable browser journals keep their formats.
Do not reset journals or replay unknown issuance as new work.

The unpublished qualification controls and checkpoint use new identities and
native Serde. Fresh disposable installations are required; old fixture payloads
are refused without clearing their bytes. This is not a production memory or
storage migration. The fixture uses concrete CDK codec hooks and retains
pre-copy argument admission and trapping reply-encoding failure for rollback.
Native traversal allocations were qualified separately from unchanged byte and
field ceilings.

## Current qualification

Evidence is retained under `target/shared-0267-host-065/`. Source selection
advanced from Wire 0.6.5 to 0.6.6 and Memory 0.50.11 to 0.50.12 during the batch.
The first offline MSRV attempt refused an uncached selection before compilation.
Explicit locked fetching and fresh affected checks qualify the final selection;
the original failure and intermediate evidence remain retained. Published Wire
0.6.6 changes only equivalent small-u16/u32 serialization; its bytes and generator
are unchanged from 0.6.5. Memory's source tree is unchanged between these patches.

Passing final-selection evidence:

- Strict native and reference-Wasm Clippy; isolated Rust 1.91 native/Wasm package
  capability checks and internal applications, with unchanged archive/input seals.
- All eight public transport and two private contract cases, all **54** browser
  tests, all **11** host-tooling cases and exact generated comparisons with three
  bidirectional vectors.
- All six native fixture tests and **11 live** Testkit cases, including signed
  ingress, certification/composition, upgrades/checkpoint restoration and the SDK
  two-device/two-receiver journey on distinct application subnets.
- Direct dependency ownership and native/Wasm normal/build graph exclusions;
  public libraries and reference Wasm remain Candid-free and exclude native tools.
- Actual consumer Make jobserver, setup/check failure, validation-seal, formatting
  preservation, evidence archive and substituted release-runner checks. Release
  metadata, failure admission, exact delivery, resume and mocked publication
  reconciliation pass with the selected editor in disposable repositories.
- Formatting, snapshot integrity and whitespace checks.

Portable signature, certified proof and session suites also passed before the
Wire/Memory patch advancements; those changes do not affect their selected
implementation graphs. Earlier release evidence remains under
`target/host-064-0144/` with its original selections.

This is Linux reference-host evidence. Full CI, native macOS, installed IC and
TEE qualification were not run for this batch. Issuer operations are injected
probes, clocks are controlled and IndexedDB is faked in Node. These tests do not
establish production issuer, browser persistence, throughput or a deployment
clock-skew bound.

## Consumer acceptance

Production Root enrollment, retained receiver runtime/budgets and scoped exports
remain [Canic #491](https://github.com/dragginzgame/canic/issues/491). Actual issuer
issuance, lookup-only reconciliation and browser-key logout remain
[Canic #507](https://github.com/dragginzgame/canic/issues/507); future-start endpoint
acceptance remains [Canic #418](https://github.com/dragginzgame/canic/issues/418).
Siblings were not edited. The [pending transport migration notice](https://github.com/dragginzgame/canic/issues/491#issuecomment-6361056217)
records the coordinated consumer change. [Tooling #236](https://github.com/dragginzgame/ic-tooling/issues/236#issuecomment-6361055807)
has published API and Auth adoption evidence; its inspected producer Intel macOS
release job was still pending, so the issue remains open. Auth's private browser
package is unpublished; updating a Rust dependency does not install it into Canic.

[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58) and
[#59](https://github.com/dragginzgame/ic-auth/issues/59) retain production issuer,
persistence/reset and abuse/workload acceptance. Production skew activation
remains closed and TEE evidence is separate; no test value grants approval.
