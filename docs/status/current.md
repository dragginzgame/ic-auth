# Current handoff — 2026-11-01

## Released source and pending work

Auth **0.28.0** is released and published. This checkout's HEAD is
`70527e6f25a0eb520edb3c6611fac12a57909e99`, tag `v0.28.0`.
The current Auth-only cleanup is uncommitted; package versions remain 0.28.0.
Pending notes target **0.29.0** because two public session budget variants and
`From<AdmissionBudgetError> for SessionError` are removed. Actual engine failures
already use credential/mechanism wrappers. `PortableCredentialError::budget_error()`
now supplies typed diagnostics; public refusals and charging remain unchanged.

Native conformance moved from private fixtures to the unpublished
`apps/qualification/backend-conformance` package. Its dependencies are public
Auth/protocol APIs and Platform Ed25519 only. It has no private DTOs, Metrics,
certified verifier, simulator or runtime edge. The old fixture feature and runner
copy are removed; `test-qualification`, Clippy, MSRV and graph admission include
its dedicated commands/checks.

`make test-backend-conformance` runs the existing scenarios. After matching Auth
libraries are published, `make pack-backend-conformance` produces a Cargo-normalized
source archive with registry dependencies. The current candidate archive is
`target/package/ic-auth-backend-conformance-0.28.0.crate` (dirty VCS metadata,
not a published package). It was tested independently against registry Auth 0.28.
The consumer dev-dependency proof has one Auth identity and excludes the foreign
bundle from the consumer workspace/release group. No Canic checkout was changed.

## Browser delivery and current guidance

A browser archive was qualified from the clean released 0.28 commit above:
`target/auth-consumer-cleanup.YXgRxC/released-browser/target/browser-client-packages/package.fONxAx/`.
`delivery.json` records `dirty: false`, source, tools and archive integrity, with
`packed-consumer` qualification. This is the delivered Rust source's client,
not a package built from the pending 0.29 API changes. Durable consumer distribution,
production issuer transport and npm publication remain separate host work.

The [active adoption guide](../canic-adoption.md) is one current contract, reduced
from 638 to 145 lines. Earlier migration text remains in
[adoption history](../canic-adoption-history.md). Current handoff links and the
Platform migration anchor used by released notes are retained/repaired.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) remains its production
integration tracker. Earlier inspection found Auth 0.26 and a closed/test-gated
receiver path; this implementation batch neither refreshed nor modified Canic.

## Selected suppliers and governance

The incoming Wire/derive **0.13.2** lock selection was preserved and qualified:
all five generated client files remain unchanged and client checks pass. Root's
compatible minimum remains 0.13.1. Other selections remain Platform/macros/guard
**0.15.3**, canonical Types **0.15.1**, Memory/Storage **0.60.3**, Metrics **0.24.1**,
Testkit **0.75.2**, PocketIC **16.1.0**, Build/Host/file CLI/editor **0.73.3**.
Testkit and Auth share arithmetic-only Metrics 0.24.1 without native metadata.
The latest adoption raises the root requirements and independent file CLI pin;
the already incoming lock selections were preserved. No public Auth dependency
or protocol/storage format changes result from this tooling-only update.

Shared snapshot **0.27.23**, revision
`619d194875369e0f72b45477b66adb430acbf6b7`, remains unchanged. Auth keeps one
Cargo-owned workspace/catalog/lock and coordinated release. No VERSION file or
pending-file release mode is added. The new local lock entry is the unpublished
conformance member; the current lock also includes the adopted supplier patches.
Public library graphs remain runtime/Candid/persistence/Metrics/tooling-free.
Native conformance likewise stays outside deployed graphs.

## Evidence and delivery limits

`target/auth-consumer-cleanup.YXgRxC/` retains the initial diff/lock, normalized
harness archives, independent registry consumers, graph observations, generated
contract checks and the clean browser release checkout. Local scenarios pass
against both workspace Auth and registry Auth 0.28, including broken-adapter
detection. Session/refusal, metrics and plain-session tests pass. Core/harness/
fixture/runner Clippy and native/certified-Wasm Rust 1.91 checks pass. All 69 client
tests pass with the incoming Wire selection. Boundary rejection fixtures and the
minimal real harness graph pass. The first standalone consumer attempt found an
unrelated ancestor backup manifest; its isolated retry and consumer dev-dependency
proof pass, with failed evidence retained.

Release/catalog qualification passed, including source preparation and mocked
publication/recovery scenarios; evidence is in `release-tools.log`. Source packing
never uploads this unpublished harness. No root staging, commit, version bump,
tag, push, release or live upload was performed. Full CI and Chromium were not rerun.
The conformance archive is a reviewable candidate; the browser receipt identifies
a clean already released source. Neither establishes installed Canic acceptance.

The [released handoff](https://github.com/dragginzgame/ic-auth/blob/70527e6f25a0eb520edb3c6611fac12a57909e99/docs/status/current.md)
retains earlier implementation/adoption evidence and publication-runner details.
Historical target paths are not proof that earlier artifacts remain available.

Build/Host/Testkit adoption evidence is retained in
`target/build-host-testkit-adoption.OKL2H2/`. Focused native Clippy, Rust 1.91,
canister Wasm build, dependency ownership/pins and Testkit caller fixtures pass.
All three executable selections are prepared. All four Host caller tests and
11 live certification/upgrade/ingress/SDK tests pass; optional Chromium is skipped.
Release-tool preparation/recovery and publication reconciliation also pass with
local/mocked delivery effects. Input hashes stayed stable; full CI was not rerun.

## Retained-state and trust boundaries

Current `AUTHNAT3`, session/login `0x415554485311`/`0x415554485313` and replay
`0x415554485302` formats remain. Earlier retirement consent grants no new reset
permission. Never erase Memory backing or only its allocation ledger to obtain
admission. Preserve retained failures, binaries, backing states and recovery data.
Hosts own clock, allocation, authority refresh, atomicity and certification.
Cryptographic checks, signed bytes, TEE/non-TEE trust and replay/charging are
unchanged. Reference qualification establishes no production wallet service,
issuer adapter, distribution or deployment.
