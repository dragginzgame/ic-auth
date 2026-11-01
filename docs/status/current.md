# Current handoff — 2026-11-01

## Released source and current work

Auth **0.29.1** is released and published at
`54d1d2aadfc67830f630612d5654f28815a64119`, tag `v0.29.1`.
Own package metadata remains 0.29.1. Pending notes target **0.29.2** for the
current upstream adoption; no release, commit or publication is performed here.
Public Auth interfaces, signed/Wire/stored bytes and TEE/non-TEE authority policy
remain unchanged by this batch. Platform 0.16's BLS identity-key fix shipped in 0.29.1.

Memory/Storage 0.61 is used only by the private qualification host. Setup parses
its allocation `StableKey`, uses fresh-only `StableCell::create` and retains
bounded `StableCell::load` for restoration. Occupied creation preserves backing
bytes; runtime bootstrap's separate ledger commits are legitimate. No reset,
combined initialization or production session-backend adoption is introduced.

The unpublished backend conformance bundle has four named scenarios:
admission/retry, logout/reopen, replacement/generation and capacity.
`assert_backend_contract` always runs all four; a subset is not full conformance.
Reference and independent encoded-record backends cover replay saturation,
reopening, exact retries and partial admission/logout commit detection.

## Selected suppliers and governance

Selections are Build/Host/file CLI/editor **0.73.6**, Testkit **0.76.2**,
PocketIC **16.1.0**, Platform/macros/guard **0.16.0**, Types **0.15.1**,
Wire/derive **0.13.5**, Memory/Storage **0.61.0** and Metrics **0.24.5**.
The requested Storage API migration is private; the other requested upstream
refreshes preserve the public identity cohort. Public Auth graphs remain free
of Memory, Storage, Metrics, runtime, Candid and native tooling. Graph admission
continues to bound the two necessary support generations while retaining one
identity per Wasm curve/signature provider. DHAT remains native dev-only.

Shared Tooling **0.27.25**, revision
`982a99706e038810ee8108d5356a8637a7d42f67`, is adopted through the exporter;
all 136 payloads match that independently selected commit. Destination graphs
are admitted without planning a downstream publication cascade. Cargo still
owns versions and the single lock. Validation-proof reuse and Build's optional
compiler-wrapper recipe are not enabled; release gates and profiles are unchanged.
The existing Wasm target configuration retains upstream's qualified 32-bit backend.

## Consumer delivery

`make test-backend-package` now passes against registry Auth/protocol **0.29.1**
with one Platform 0.16/Types identity. The earlier Auth 0.29.0/Platform 0.15
mismatch is historical. Fresh source, archive and consumer inputs are sealed;
the excluded dev dependency, trait/error identity witness, all packaged scenarios
and twelve graph refusals pass. The candidate receipt is
`target/portable-fixtures/backend-package.iX0GHj/delivery.json`.
It records dirty source, not a delivered revision or Canic adoption.
See the [package instructions](../../apps/qualification/backend-conformance/README.md).
This remains opt-in delivery qualification after explicit registry preparation.

The private browser source still has no npm publication or production issuer
adapter here. All five regenerated files are unchanged under Wire 0.13.5.
Browser archive history is not durable consumer distribution or installed acceptance.

## Measurement and Canic ownership

Retain the canonical encoder. The [native allocation observations](../canonical-allocation-observations.md)
and [isolated Canic Wasm diagnostic](../wasm-attribution.md) justify no new Auth
optimization or verification-result cache. They do not measure the current
consumer cohort or successful production admission. Earlier target paths may
have been removed by ordinary cleanup; historical results keep their original scope.

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns production
enrollment/refresh, issuance/reconciliation, receiver transports/guards, actual
backend conformance and durable browser journeys. The last source inspection
selected Auth 0.28 and a closed authorization facade; this batch neither inspected
nor changed Canic. [#720](https://github.com/dragginzgame/canic/issues/720) owns its
storage-helper deduplication. Recorded diagnostic findings remain at
[#705](https://github.com/dragginzgame/canic/issues/705) and
[#724](https://github.com/dragginzgame/canic/issues/724), with their historical evidence.

## Current qualification and retained-state limits

Evidence is retained in `target/upstream-adoption.2T1aUH/`, including the incoming
lock/dirty baseline, source/selection observations and failed diagnostic attempts.
Ten native storage tests, fixture Clippy, native/Wasm Rust 1.91 checks, actual
dependency graphs and caller regressions pass. The prepared file/editor/server
selections, Host/Testkit/client setup, evidence and Make jobserver callers pass.
All 69 browser tests and 11 live PocketIC certification/upgrade/ingress/SDK tests
pass; optional Chromium is skipped. Local mocked release/recovery fixtures pass
with the updated editor and Shared helpers (`release-tools.log`). Full CI,
macOS, live IC and installed Canic are not claimed.

Current `AUTHNAT3`, session/login `0x415554485311`/`0x415554485313` and replay
`0x415554485302` formats remain. Memory 0.56's older-ledger retirement is separate
from this byte-compatible 0.61 API migration. Never erase backing or only its
allocation ledger to obtain admission. Preserve failures, binaries, retained
states and recovery custody. Hosts own clock, allocation, authority refresh,
atomicity and certification-root publication.
