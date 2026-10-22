# Current handoff — 2026-10-22

## Source and selections

Delivered baseline is Auth **0.14.2** at
`50061f59bc2ab3ff9e068a009006e0398dcafaef`; package versions remain 0.14.2.
Compatible pending **0.14.3** notes cover upstream dependency adoption. Public
Rust APIs, signed bytes, Wire contracts and stored formats are unchanged.

The incoming lock selects published Platform/macros **0.6.7** and IC
Build/Host/Testkit **0.63.3**. Registry searches and a targeted update dry run
on 2026-10-21 confirmed these selections, Memory **0.50.10** and Wire/derive
**0.5.19**. Wire 0.5.20 was unpublished at that review and was not selected.
The incoming lock is preserved byte-for-byte; existing compatible manifest
requirements need no changes. There are no sibling/path overrides.

Published Platform and macro source trees are identical to 0.6.6; the Host
library source trees are identical to 0.63.2. Build reuses a completed artifact
copy's digest in its evidence export, which Auth does not call. Other source
changes are producer fixture cleanup. No Auth adapter or crypto policy changed.
Shared Tooling remains the 118-file **0.25.9** snapshot at
`85aded191d85ac4a8c0e5deab207f6be18895862`. Development Rust remains 1.99.1;
the supported package minimum is 1.91.

## Preceding dependency qualification

Evidence is in `target/upstream-0143/`, including published source/manifest
diffs, incoming lock bytes and its input seal. Passing checks:

- Complete existing Clippy target, including the reference Wasm fixture.
- Focused types/Wire, portable signatures, adversarial crypto feature unification,
  certified proofs, sessions/restoration and host tooling tests.
- Shared direct-dependency ownership and Auth public native/Wasm normal/build
  graph checks. Public libraries and reference Wasm retain their Candid exclusion.
- `make check-msrv`: isolated package payloads across all 18 public native/Wasm
  feature selections and the internal applications; archive and lock seals pass.
- All **54** browser tests, five exact generated-file comparisons and three
  bidirectional Wire vectors. Wire is unchanged; no regeneration was needed.
- All five native qualification-canister tests.

The issue-review pass received an additional Tokio **1.54.1** lock update.
It was preserved, but differs from the preceding dependency input seal; those
results do not qualify this additional change. No code tests were rerun for
issue reconciliation or documentation-only edits.

The selected **0.63.3** Testkit CLI and IC Build editor are missing. Explicit
preparation was requested through `make install-testkit-tools` and
`make install-release-tools`; it has not been authorized or executed in this
pass. Live certification/upgrade/ingress and release/recovery fixtures therefore
have **not** been rerun for this selection. Do not substitute older tool paths.
Full CI, native macOS, installed-IC and TEE qualification were not run. No
commit, package version change, release, publication or deployment ran.

## Delivered browser and topology acceptance

Auth 0.14.2 includes the [#83 logout attempt confinement](https://github.com/dragginzgame/ic-auth/issues/83)
and [#84 immutable admission destination](https://github.com/dragginzgame/ic-auth/issues/84)
fixes. The source and original qualification are preserved by the released
commit and `target/platform-066-0142/`; do not treat them as pending source work.
Both issues are now closed after verifying the default-main source and its
[complete three-host CI matrix](https://github.com/dragginzgame/ic-auth/actions/runs/39637576898).
All three Validate steps pass; delivered Linux logs explicitly show both fixes'
regressions, all 54 browser tests and eleven live/five native fixture cases.
That earlier Testkit **0.63.2/server 16.1.0** run passed all 11 live cases,
including two devices across two explicit application subnets in the Ed25519
SDK scenario. It does not qualify the new CLI selection. Issuer operations are
injected/read-only probes, clocks are controlled and IndexedDB is faked in Node;
there is no production issuer, persistence, subnet-cost or clock-skew claim.

## Consumer ownership

Production Root enrollment, retained receiver runtime/budgets and successful
scoped exports remain Canic-owned in [Canic #491](https://github.com/dragginzgame/canic/issues/491).
Portable issuance, lookup-only reconciliation and browser-key issuer logout
belong to [Canic #507](https://github.com/dragginzgame/canic/issues/507); future-start
endpoint acceptance belongs to [Canic #418](https://github.com/dragginzgame/canic/issues/418).
No new producer API blocker was established by this dependency review. Sibling
repositories were not edited. The private browser client is unpublished;
upgrading Rust crates alone cannot install its browser fixes.

[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58) and
[#59](https://github.com/dragginzgame/ic-auth/issues/59) retain production issuer,
persistence/reset and abuse/workload acceptance. Production skew activation
remains closed, with TEE evidence a separate gate; no test value grants approval.
