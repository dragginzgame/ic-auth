# Current handoff — 2026-11-01

## Released source and current work

Auth **0.29.0** is released at `6c0411b5b9b16b695f7f7b90feed8ca12f6589cd`,
with tag `v0.29.0`. Own package metadata remains 0.29.0. Pending notes target
**0.29.1** for qualification support and upstream adoption. Public Auth interfaces,
Wire/stored bytes and TEE/non-TEE authority policy remain. Platform's BLS fix
additionally rejects forged identity-root certificates.
The earlier budget-error cut and independent conformance extraction are now
released history. The active adoption guide describes 0.29, rather than pending
0.28/0.29 work.

Development commit `d1ffc0a` splits the unpublished backend conformance into four
named scenarios: admission/retry, logout/reopen, replacement/generation and
capacity. `assert_backend_contract` always invokes all four. Individually named
calls aid diagnosis; a subset is not full conformance. Reference callbacks and
independent encoded records pass, including targeted admission/logout
write-then-error detection. Capacity covers global/per-caller replay saturation,
reopening, exact retry at saturation and independent nonsaturated callers.

`make test-backend-package` is opt-in delivery qualification after the matching
Auth cohort is published and explicitly cached. It packs fresh source, compiles
an excluded consumer dev dependency using one registry Auth/protocol identity,
runs every packaged scenario and tests source/dependency graph refusals. Source, archive and
extracted consumer input hashes are sealed; `delivery.json` binds source/dirty
status and archive SHA-256. Dirty candidate receipts do not establish release or
consumer adoption. Ordinary CI does not require a not-yet-published cohort.
See the [package instructions](../../apps/qualification/backend-conformance/README.md).

## Measurement decision

`make profile-canonical-allocations` observes unchanged canonical hash functions
on native synthetic small/wide inputs. DHAT 0.3.3 is only an unpublished tooling
dev dependency; no production allocator or profiler capability is added.
See [allocation observations](../canonical-allocation-observations.md) for results
and limits; these are not full verifier cost or Wasm savings. Retain the encoder.
No optimization or verification-result cache is implemented; the rejected streaming
trial remains historical evidence in Auth #113.

## Canic ownership and issue filing

The [Wasm diagnostic](../wasm-attribution.md) used isolated Canic `e753785` plus
its captured working patch, selecting Auth 0.28. Its public facade still denies;
successful admission cost is unmeasured. The normal checkout was untouched.
No new Auth cut is justified. Canonical Release remains blocked by
[Canic #705](https://github.com/dragginzgame/canic/issues/705); the distinct report
classification defect is filed in [Canic #724](https://github.com/dragginzgame/canic/issues/724).

All seven recommendations are mapped in
[Canic #491's current acceptance comment](https://github.com/dragginzgame/canic/issues/491#issuecomment-6555411790):
protected enrollment/refresh, receiver transports, application guards,
issue/reconcile, actual backend conformance and durable browser transport/journey
remain with that existing production integration owner. The distinct storage
helper deduplication is filed as
[Canic #720 — Reuse Auth pair and count validation in the persistent session adapter](https://github.com/dragginzgame/canic/issues/720).
Do not open duplicate trackers for the consolidated integration obligations.

Browser source/integrity receipt qualification from Auth 0.28 is historical;
ignored producer artifacts are not durable consumer distribution or evidence of
an installed Canic path. Select and qualify the intended production source at
its delivery owner. Neither npm publication nor a production issuer is supplied.

## Selected suppliers and governance

Selections are Build/Host/file CLI/editor **0.73.4**, Testkit **0.76.0**,
PocketIC **16.1.0**, Platform/macros/guard **0.16.0**, Types **0.15.1**,
Wire/derive **0.13.3**, Memory/Storage **0.60.6** and Metrics **0.24.4**.
Platform's owned certificate/key identities, K256 0.14 and Dalek 3 are adopted;
Auth retains signed bytes, strict verification and explicit high-S handling.
The native adversarial trigger selects Dalek 3. Wasm selects upstream's 32-bit
backend; graph policy bounds support generations while retaining one identity
per Wasm curve/signature provider. DHAT remains excluded from deployed graphs.
Shared snapshot **0.27.24**, revision `1211262f32ea193cc4edcd60d2750baf1f1d6425`,
is adopted. Keep Cargo-owned versions, one lock and existing recovery admission.

## Evidence and retained-state limits

Current command logs, preserved failures and source/lock baseline are in
`target/auth-integration-support.E0SVsi/`; delivery attempts and profiling results
are retained under their printed `target/portable-fixtures/` directories.
The prior-cohort receipt is `target/portable-fixtures/backend-package.iMZLhy/delivery.json`.
Workspace/packaged conformance, native tooling/harness Clippy, Rust 1.91,
ShellCheck, dependency ownership and pins pass. Explicit registry preparation
was separate from offline checks; failed preparation attempts remain retained.

Replay/boundary evidence in `target/conformance-boundaries.BjHneY/` covers seven
tests, twelve graph refusals, actual graph observations and focused lint/MSRV.
Wasm build, attribution and preserved failures are in
`target/canic-wasm-attribution.LEJv31/`; scope and final files are in the report.

Upstream adoption evidence is in `target/upstream-adoption.sdU2D0/`.
Registry versions/checksums match the selected lock. Core and adversarial crypto
tests, focused Clippy, native/Wasm Rust 1.91 and dependency graph checks pass.
All five regenerated browser files are unchanged; 69 client tests pass.
The updated fixture pairing passes native persistence/metrics/conformance and
11 PocketIC certification/upgrade/ingress/SDK tests; optional Chromium is skipped.
Host/Testkit caller and local mocked release/recovery fixtures pass with the
prepared CLI selections. Earlier refusal attempts are preserved.
The new source archive correctly refuses the old registry Auth 0.29.0 graph,
which still requires Platform 0.15. Its delivery proof awaits an aligned Auth
publication; do not weaken the single-Platform gate. Evidence is retained in
`target/portable-fixtures/backend-package.0mdt6w/`.

The jobserver follow-up substitutes both new opt-in profiling/package wrappers
and covers their inherited descriptors and unsafe-mode refusals. The focused
`make test-make-jobserver` and ShellCheck pass; evidence is retained in
`target/portable-fixtures/make-jobserver.4wuLVw/`. The reported full-gate failure
is preserved; the complete gate has not been rerun.

No staging, commit, release version bump, push, publication or deployment is
performed by this batch. Full CI, Chromium, macOS and installed
Canic acceptance are not claimed. The prior released handoff retains historical
qualification evidence; old target paths do not guarantee artifact availability.

Current `AUTHNAT3`, session/login `0x415554485311`/`0x415554485313` and replay
`0x415554485302` formats remain. Never erase Memory backing or only its allocation
ledger to gain admission. Preserve failures, binaries, backing states and recovery
custody. Hosts own clock, allocation, authority refresh, atomicity and certification.
