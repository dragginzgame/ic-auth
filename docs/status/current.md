# Current handoff — 2026-10-27

## Delivered source and pending batch

The maintainer pushed source release **0.19.1** at
`238cc56c3232b958b3bb7b8f55b50428e8b397fc`. Its
[required CI](https://github.com/dragginzgame/ic-auth/actions/runs/40546636649)
passed Linux and Apple Silicon macOS but failed Intel macOS in live qualification:
the logout-only device's 30-second session expired before the final revocation
assertion. [Auth #96 — Keep live logout qualification sessions valid on slower native hosts](https://github.com/dragginzgame/ic-auth/issues/96)
records the local repair: request the existing 60-second maximum for that device
and mint its new login deadline at the reset boundary. Default/runtime TTL policy
and final pre-expiry assertions remain. Required native Intel acceptance is pending;
do not close routine delivered-adoption issues against the failed gate.

The pending **0.19.2** draft is compatible tooling/qualification work: published
Tooling/Memory adoption, Testkit wrapper retention and opt-in real-browser evidence.
The maintainer committed that batch as `8ba6c55d58aba694d2c75f5310ee3fa5571d107b`.
Release validation then failed because the jobserver fixture reached the actual
optional browser wrapper. [Auth #97 — Substitute opt-in browser effects in the Make jobserver fixture](https://github.com/dragginzgame/ic-auth/issues/97)
owns the local fixture repair. Preserve the original failure under
`target/portable-fixtures/make-jobserver.vdX060/` and release logs under
`.git/release-state/validation-logs/validation.FUJonu`; no active release lock or
saved 0.19.2 plan was found. The repair does not restart or alter release state.
The focused jobserver fixture passes with `IC_AUTH_BROWSER_EXECUTABLE` unset,
including inherited descriptor checks and unsafe-mode refusal for the browser
target. Shell syntax, snapshot, documentation and diff checks pass; evidence is
in `target/jobserver-browser-20261027/`. No complete release gate was rerun.
Public libraries/client runtime,
APIs, signed/stored/public Wire formats and package versions remain unchanged.
No Auth staging, commit, push, release, publication or deployment ran in this batch.

Selections are Platform/macros **0.7.0**, Host/Build/Testkit **0.69.2**, Memory
**0.51.12** and Wire/derive **0.7.18**. Only five Tooling packages and Memory change
in Cargo.lock. Published Tooling source is `1e245a9e8791b04daa0bf971124ae8908d956821`;
Memory source is `41a04886ec29546e04fcec759821d5f818fd64ab`.
[Auth #95 — Adopt IC Tooling 0.69.2 and Memory 0.51.12](https://github.com/dragginzgame/ic-auth/issues/95)
records registry/archive identities and focused evidence under
`target/tooling-memory-20261027/`. Both selected CLIs were explicitly prepared.
Native/Wasm Clippy, Rust 1.91 checks, six fixture unit tests and four native tooling
tests pass. All 12 live cases and actual Chromium qualification pass with the new
selections; the timing repair's separate focused live run passes. Actual release
metadata/recovery callers also pass with local/mocked effects (`release-tools.log`).
Memory's additive cell-loading API needs no change to the bounded checkpoint adapter.
PocketIC remains **16.1.0**. Rust **1.91** is the package minimum;
development formatting/Clippy use **1.99.1**.

## Qualification work ready for delivery

[Auth #90 — Consolidate admission policy and simplify client and qualification state](https://github.com/dragginzgame/ic-auth/issues/90)
has its final deferred wrapper adoption implemented locally. `PocketIcInstance`
now owns client/server teardown throughout the fixture; mutable access starts live
gateways without splitting ownership. Twelve existing live cases pass, including
certification, upgrades, ingress and SDK recovery. Runner Clippy and Rust 1.91
checks pass; the Testkit tool-admission fixture passes. Leave source-delivery
reconciliation with the maintainer; normal sibling checkouts are untouched.

[Auth #3 — Qualify portable admission and logout client in real browsers](https://github.com/dragginzgame/ic-auth/issues/3)
now has an opt-in `make test-client-browser` harness. Explicit setup selects a
prepared absolute Chromium executable; checks download no browser. Locked private
dev tools are Playwright Core **1.62.1** and esbuild **0.28.2**. The checked browser
is Linux Chromium **156.0.8078.4**, using native IndexedDB and real SDK signing.
Cross-tab invalidation, strict commit/refusal, renderer crash, retained cleanup
leases/deadlines/uncertainty and reopen pass. Live SDK calls cover two devices and
receivers, original issuance recovery, lost logout replies and retained-key refusal.
See [browser evidence and limits](../ic-testkit-qualification.md#real-browser-qualification)
and [explicit setup](../development.md#browser-client-setup). The issuer is injected;
the lifecycle clock is controlled. No production issuer, TEE/hardware key erasure,
other engine or macOS browser claim follows. Ordinary CI compiles the harness;
actual browser execution remains opt-in. Existing 65 Node cases and Rust/TypeScript
vectors pass. Local logs and browser hashes are in
`target/issue-maintenance-20261027/`, including failed attempts kept with final runs.

## Prior delivered changes and accepted measurements

The exact Shared Tooling selection is **0.26.25**,
`6b6bbd9c9e5798f0ecf6105637392a7a55cd450e`, **126 files**. The producer's initial
VERSION mismatch was corrected upstream; no consumer override exists.
[Auth #92 — Adopt latest Shared Tooling revision](https://github.com/dragginzgame/ic-auth/issues/92)
records snapshot/hook/release-caller evidence in
`target/shared-tooling-02625-20261027/`. The runtime panic scanner remains unselected.

[Auth #93 — Reduce repeated Rust admission work with measured Wasm qualification](https://github.com/dragginzgame/ic-auth/issues/93)
ships shared scope lookup and the removed extra retry clone in 0.19.1. Matched
wide admission uses about **20%** fewer instructions, authorization **12.8%** fewer;
reference Wasm is **1,058 bytes smaller**. Small authorization adds **22 instructions**.
`target/rust-perf-20261027/` retains matched artifacts, rejected candidates and
focused behavior/lint/MSRV evidence. These are reference-fixture observations.

[Auth #94 — Certified-proof admission measurements](https://github.com/dragginzgame/ic-auth/issues/94)
ships the private probes and [operational report](../design/0.6-portable-authentication.md#certified-proof-operational-profile--2026-10-27).
Two BLS checks account for **98.8%** of the **440.25M** complete-verifier instructions.
The maintainer accepted keeping the existing verifier and cost. No optimization,
cache, crypto/signed-byte/certificate/delegation change or TEE/non-TEE trust-model
change is authorized. Further performance work requires actual production targets.
Evidence remains in `target/certified-profile-20261027/`.

## Consumer and security boundaries

The private client selects Platform npm **0.1.1** over SDK Core **6.1.1**.
Canic owns production receiver/store/reset/load integration and authority policy in
[Canic #491 — Extract reusable authentication, token and session machinery](https://github.com/dragginzgame/canic/issues/491),
and issuer/reconciliation acceptance in
[Canic #507 — Non-issuing reconciliation qualification](https://github.com/dragginzgame/canic/issues/507).
No standalone wallet service is implemented. Host session/replay/ledger commits
and authority generations remain atomic; clock, certification and memory stay host-owned.

[Auth #85 — Qualify the crypto-bigint advisory](https://github.com/dragginzgame/ic-auth/issues/85)
was retired by consolidation, not fixed. The active obligation is
[Platform #50 — Crypto capability qualification](https://github.com/dragginzgame/ic-platform/issues/50);
affected paths remain selected, with no blanket exemption.
Preserve release recovery plans, original artifacts, backups and failed evidence.
The [preceding handoff](https://github.com/dragginzgame/ic-auth/blob/238cc56c3232b958b3bb7b8f55b50428e8b397fc/docs/status/current.md)
retains the released API migration, earlier qualification paths and adoption details.
