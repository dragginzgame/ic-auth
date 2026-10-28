# Current handoff — 2026-10-28

## Delivered base

Auth **0.19.3**, `d0eb2cfe10f08cb1d77f5fa7c099c2434107434f`, is pushed with
[successful complete CI](https://github.com/dragginzgame/ic-auth/actions/runs/40658884476)
on Linux, Apple Silicon macOS and Intel macOS. The prior eleven trackers and
Shared/dependency adoption #98 are closed against delivered source and evidence.
The working tree was clean when the availability work began.

Selections are Platform/macros **0.7.1**, Host/Build/Testkit **0.69.4**, Memory
**0.51.13**, Wire/derive **0.7.18**, browser Platform npm **0.1.1** over Core
**6.1.1**, and PocketIC **16.1.0**. Shared source is exact
`cc81a58a5b2dd287c577cbffd2c576c178b6785c`, VERSION **0.27.3**, **127 files**.
Rust **1.91** remains the package minimum; development uses **1.99.1**.

## Current Shared and native adoption

[Auth #101 — Adopt Shared Tooling 0.27.3 and IC Tooling 0.69.4](https://github.com/dragginzgame/ic-auth/issues/101)
records the canonical clean-checkout impact/refresh. The trusted verifier compares
every payload against the independently selected full source revision above;
ordinary builds retain offline integrity checks. Optional sccache remains unselected;
the reviewed unattended report-only policy activates no schedule or host job.

The maintainer's staged lock already selected the five Tooling 0.69.4 packages.
Those bytes and the entire existing staged patch are preserved exactly. Published
archive source is `fa3786c86a00e5e9f98a0c729d86bf45a1bd1a82`; package Rust sources
match 0.69.3, with changed declared dependency floors. Selected Testkit and release
editor were explicitly prepared. Evidence in `target/adoption-20261028b/` covers
trusted-source comparison, actual formatting/staged hooks and jobserver, four
native tooling tests, selected native Clippy and Rust 1.91. All twelve live cases
and actual Chromium contention/SDK qualification pass with the new selection.
Local/mocked source/metadata/release/recovery callers also pass. New adoption edits are
unstaged; the user-owned index and staged availability work remain untouched.

## Current availability batch

Pending **0.19.4** combines compatible tooling adoption, a private-client
availability fix and targeted server qualification. Auth versions are unchanged.
No Rust library/runtime, crypto, certificate/delegation checks, protected authority,
retention policy, signed bytes, Wire/storage format or TEE/non-TEE trust model changes.

[Auth #100 — Ensure browser logout makes progress under concurrent admission activity](https://github.com/dragginzgame/ic-auth/issues/100)
identified unbounded outstanding client admission entry work before per-destination
coalescing. `PortableAdmissionClient.establish` now admits at most **64** outstanding
invocations per coordinator, before journal I/O; overflow returns the existing
`storage_capacity` error. All completion/error/timeout paths return capacity.
The bound includes duplicate callers waiting on issuance/status work. Applications
reuse a coordinator within each tab/login; it is not an origin-wide scheduler or a promise
against browser main-thread/storage failure. Logout remains independently invoked
and committed before fan-out; late admissions cannot revive the login.

[Auth #99 — Bound admission and reservations without starving other clients](https://github.com/dragginzgame/ic-auth/issues/99)
was investigated without the audit's missing detailed report/reproduction. On
current source, anonymous/cheap refusals cannot consume verification credit or
reserve admitted-login slots. Invalid cryptographic proofs consume shared credit
but install no reservations. The new tests demonstrate exhaustion/refill, existing
session/exact-retry progress, bounded unknown capacity and committed expiry pruning
that permits another legitimate client. The anonymous/unverified 24-hour admitted
reservation claim did not reproduce. Unknown logout tombstones have a separate
pool and protocol-required maximum-login retention; do not shorten them or refund
failed crypto work. Sustained forged traffic and finite valid-credential capacity
remain host/issuer admission-policy obligations, without a Sybil-fairness claim.
See [contention contracts](../sessions.md#availability-under-contention) and
[client scheduling](../browser-client.md#durable-logout-and-guarded-identities).

## Evidence and scope

`target/availability-20261028/` retains the pre-fix failing queue-bound tests,
selected inputs, final runs and earlier setup evidence. Focused checks pass:

- 52 session/revocation cases, including 1,280 anonymous/cheap refusals, two-client
  budget contention, unknown saturation, failed pruning and legitimate recovery.
- Selected Rust session-test Clippy; actual Rust/TypeScript vectors and all 68
  client tests. Reinforced entry tests verify overflow creates zero transactions.
- Actual Linux Chromium **156.0.8078.4**, native strict IndexedDB and real SDK
  signing: hold issuance/admission with 64 callers; refuse overflow; commit repeated
  logout with a fixed deadline; finish injected unavailable cleanup without false
  acknowledgement; reject late results. Existing crash/reopen/outage and live
  SDK two-receiver/device revocation cases also pass.
- Snapshot, formatting, local documentation and diff checks. The client build
  compiles both the production client and browser harness.

The real-browser contention issuer/transport and lifecycle clock are injected;
its separate live SDK scenario pins the local IC root and uses prepared native
credentials. Tests retain known keys deliberately. No production issuer, hardware
key erasure/TEE or native macOS browser qualification is claimed. The prior Node
and live fixture evidence retains its own source/dependency identities.

The patch is uncommitted. No staging, commit, push, package-version change, full
local CI, release/publication, deployment or sibling checkout mutation ran. Local
investigation/fix preparation does not wait for hosted CI or crates.io. Owning
issues retain precise implementation, remaining limitations and delivery status.

## Preserved decisions and recovery

Keep the accepted certified verifier and admission cost; no verification cache or
crypto/trust optimization follows. Earlier profiles remain in
`target/rust-perf-20261027/` and `target/certified-profile-20261027/` with the
[operational report](../design/0.6-portable-authentication.md#certified-proof-operational-profile--2026-10-27).
Canic #491/#507 retain actual host/issuer/store/enrollment and production load
acceptance. Host writes remain atomic; clock, certification and memory stay owned.
Platform #50 retains the separate advisory; earlier consolidation is not a fix.
Preserve all build, failed and `.git/release-state` recovery evidence. The
[preceding handoff](https://github.com/dragginzgame/ic-auth/blob/d0eb2cfe10f08cb1d77f5fa7c099c2434107434f/docs/status/current.md)
retains the exact prior adoption and qualification paths.
