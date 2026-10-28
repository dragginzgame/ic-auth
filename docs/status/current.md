# Current handoff — 2026-10-28

## Delivered source and completed reconciliation

The maintainer released Auth **0.19.2** at
`062c84e8e4a46f984631036da7c25ef40e98c0fc`.
[Its required CI](https://github.com/dragginzgame/ic-auth/actions/runs/40560297177)
passes Linux, Apple Silicon macOS and Intel macOS, including the repaired live
logout timing and Make jobserver/browser-wrapper cases. All eleven prior Auth
trackers were reviewed and closed against that delivery; historical failures
retain their original source identity and no longer gate ordinary acceptance.
The real-browser tracker closed with its explicitly bounded Linux Chromium
proof, not a claim of production issuer or macOS-browser adoption.

## Current compatible adoption

[Auth #98 — Adopt current Shared Tooling, Platform, Memory and IC Tooling](https://github.com/dragginzgame/ic-auth/issues/98)
owns the pending **0.19.3** batch. The incoming Cargo.lock already selected
Platform/macros **0.7.1**, Host/Build/Testkit **0.69.3** and Memory **0.51.13**;
these bytes are preserved exactly. Registry checksums and published archive
sources were verified. Wire/derive stays **0.7.18**, browser Platform npm **0.1.1**
over SDK Core **6.1.1**, and PocketIC **16.1.0**. Root requirements keep their
existing supported API floors. Auth package versions remain **0.19.2**.

Shared Tooling is refreshed canonically from clean isolated exact source
`7efce1e36275ff9e4bbbdd9ccab9a8d94520fd98`: exporter VERSION **0.27.2**, **127 files**.
The source also carries the undated 0.27.3 draft's merged hook-fixture correction;
no downstream version override or snapshot edit was made. The formatter now
imports its declared lexer companion. Auth's hook fixture selects the shared
entrypoint and derives the helper closure instead of repeating its file list.
Auth's explicit module paths still disable formatting reuse; no cache speedup
is claimed. Shared system/compiler bootstrap guidance and the single-owner
advisory rule are adopted without adding installers or changing security gates.

Published sources: Platform `5862651a57de3fba6c03cc5d7c502aaca3cdebcf`,
Tooling `e3a71a922b1604f53b3e5a1530e154aec96e2a81`, Memory
`3acffe19c7d2dc1c2bf70dc03313df95accebd91`. Native executable setup explicitly
prepared/admitted the lock-selected Testkit and release editor. The independent
file CLI pin, common tool catalogs and compiler selections are unchanged:
Rust **1.91** package minimum, development **1.99.1**. Complete native/browser
prerequisite ordering is documented in [setup](../development.md#setup).

## Focused evidence and delivery boundary

Evidence is retained in `target/upstream-20261028/`, including failed initial
selection attempts, exact incoming lock, registry/archive provenance and source
patch. Current passing checks cover:

- Actual formatting and staged/partial-stage hook preservation, companion closure
  and hook installation; jobserver descriptors and unsafe-mode refusal.
- Native/public-library and Wasm Clippy; affected public/internal Rust 1.91 checks.
- 122 selected Auth cases across canonical bytes, Ed25519/ECDSA/BLS verification,
  certificate freshness, certified delegation, session/replay and typed refusals.
- Eleven Wire/control contract cases, six host checkpoint/refusal units and four
  native tooling cases. All supported capability graph boundaries pass.
- All twelve live cases, including certification, checkpoint upgrades, protected
  authority, signed ingress, typed SDK refusal and original-proof/logout recovery.

Actual Linux Chromium qualification passes strict storage, cross-tab/crash/outage
recovery and SDK calls for two devices/receivers. Local/mocked release metadata,
failure admission, exact-source/tag/push, resume and publication reconciliation
also pass. Ordinary Node client sources and npm dependencies are unchanged;
their existing 0.19.2 CI evidence remains applicable. Generated contracts are
compared by live qualification; Wire was not upgraded. No public Auth library or
client runtime source, verification policy, signed/stored/Wire layout or TEE/non-TEE
trust assumption changed. No Auth staging, commit, push, version bump, full local
CI, real release/publication or deployment ran. This local draft has no hosted
macOS result; delivery and its required matrix remain with the maintainer.

## Preserved decisions and consumer boundaries

The measured Rust scope/retry improvement and its original matched artifacts
remain in `target/rust-perf-20261027/`. The accepted certified-proof decision is
unchanged: **keep the verifier and admission cost**, with no verification cache,
crypto/signed-byte/certificate/delegation/trust changes. See the
[operational report](../design/0.6-portable-authentication.md#certified-proof-operational-profile--2026-10-27)
and `target/certified-profile-20261027/`. Further optimization requires actual
production workload requirements.

Canic owns production receiver/store/reset/load and issuer/reconciliation adoption
under [Canic #491 — Auth extraction and adoption](https://github.com/dragginzgame/canic/issues/491)
and [Canic #507 — Non-issuing reconciliation qualification](https://github.com/dragginzgame/canic/issues/507).
No standalone wallet service exists. Atomic host commits, protected authority,
clock, certification publication and memory ownership remain host obligations.
[Platform #50 — Crypto capability qualification](https://github.com/dragginzgame/ic-platform/issues/50)
retains the shared advisory; retiring Auth #85 was consolidation, not a fix or exemption.
Preserve all prior artifacts and `.git/release-state`, including completed 0.19.2
and earlier failed attempts. Normal releases start fresh; exact saved-attempt
recovery must retain its source and state. The
[preceding handoff](https://github.com/dragginzgame/ic-auth/blob/062c84e8e4a46f984631036da7c25ef40e98c0fc/docs/status/current.md)
retains earlier proof identities, browser assumptions and evidence paths.
