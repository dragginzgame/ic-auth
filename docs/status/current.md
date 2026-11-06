# Current handoff — 2026-11-06

## Published state

**Protocol 0.35.0** (`ic-auth-protocol-types`, `ic-auth-wire`) and **Engine 0.35.0**
(`ic-auth`) are published on crates.io. Both release commits and annotated tags
are pushed to `origin/main`:

- Protocol: `83028f07c75bb80d38867b69ae31cd9e0a2651c4`, tag `protocol-v0.35.0`
  (`fc8755ef3572af17d806aa6dfc8203f93350bc73`). Registry checksums:
  types `6204b7d777028c617275f1f63b15bf7d542d89a7b8735c952edaf6770f5421c2`,
  Wire `b1a16f6109fe1736d81512fb43e3e4e03efcb8e821e023f62d97d18f354136c3`.
- Engine: `941ebc2994f55f71976f097c8a47ddbbadda7af7`, tag `engine-v0.35.0`
  (`a213b823e1eaef4debda468b0c542b5825981f1a`). Registry checksum:
  `91b975995740e37756eeef717bb28bf2d3c0a591a7f346aacb89093d3019840f`.

Exact custody: `.git/publication-state/protocol-0.35.0.json` and
`.git/publication-state/engine-0.35.0.json`. Upload/readback evidence:
`target/publication-protocol-0.35.0.P5GKEs/` and
`target/publication-engine-0.35.0.8zh5jy/`. There is no unresolved upload.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) is closed as completed.
Canic production adoption remains separately owned by #491; Canic stayed read-only.

## Released contract and consumer action

Protected login windows support up to 90 days, retaining 60-second proofs and
30-minute session ceilings. Ledger construction requires the explicit protected
host login ceiling before receiver-local time. Shorter host policy remains valid;
private canister fixtures choose one day. Signed claims and session/replay layouts
and identities are unchanged.

The [clock-independent contract](../design/portable-clock-audit.md) replaces the
unqualified skew precondition. Known logout retains committed denial through its
authenticated immutable deadline. Unseen logout commits bounded temporary denial
and returns `Unconfirmed`, including retries. Pools/work credit remain separate.
No new inter-canister calls, receiver challenges or cryptographic relaxations are
added; ordinary authorization remains local.

The timestamp reset helper is retired. Complete durable restore plus fresh
authority remains supported; actual safety-state loss stays closed pending trusted
recovery. The fixture explicitly checkpoints never-configured, closed and retained
states, preserving closed state across further upgrades. Profiling resets use
explicit management reinstalls of disposable test canisters, not a receiver bypass.

The maintainer confirmed no retained affected Rust/private browser records.
Logout Wire identity is `0x415554480014`, login storage identity `0x415554485323`,
fixture payload `AUTHCLK1`, private journal `ic-auth-local-logout-journal-v1`.
Old bytes are refused without clearing. Update endpoints and generated clients
together; Auth's record confirmation does not authorize resetting Canic's stores.

The [remembered-login client](../browser-client.md#ninety-day-remembered-login)
persists renewal sequences and one exact credential separately from guard metadata.
Unresolved issuance is lookup-only; completed evidence permits browser-key renewal.
Initial issuance still needs account authentication. Readiness identities prepare
before new call/query signing after resume, bound expiry recovery to two attempts,
and never replay application effects or renew while polling. Logout fences late
results, deletes evidence atomically, disposes keys within finite local budgets and
retains uncertainty. Issuer endpoints, durable key custody, scheduling clock mapping
and actual Canic/Toko wiring remain consumer-owned.

## Qualified delivery and dependency selection

Both complete **30-stage release gates passed**, including native/Wasm Clippy,
isolated package/MSRV checks, 92 client tests/interoperability, all 11 ordinary
installed cases, local archives and delivery/recovery/publication fixtures.
Chromium remains opt-in/skipped. Gates:
`target/release-validation/protocol-gate.CAofib/logs/validation.JeBDEm` and
`target/release-validation/engine-gate.NkuuYS/logs/validation.zrOYRU`.

The clean delivered Engine commit also qualifies a private client archive at
`target/browser-client-packages/package.5x3Jeg/`, with its `delivery.json`, integrity
and packed-consumer receipt. This is source/archive delivery, not npm publication
or production browser/issuer acceptance. Preserve it at a consumer-owned location
before adoption; ignored producer artifacts remain removable by explicit clean.

The maintainer explicitly selected the tested dependency graph: Platform 0.23.2,
Testkit 0.89.0 and Metrics 0.29.3. The lock also selects Build 0.75.0, Storage 0.68.0,
Wire/derive 0.17.4, Types 0.18.2, Runtime/guard 0.19.3 and Host 0.74.4.
Shared Tooling 0.30.6 at `45c0295e23d3d23225e31f47c018c30ff0c75581` has
159 verified payloads. Rust 1.99.1 remains the compiler/minimum. PocketIC 16.1.0,
Build's editor and Platform browser SDK 0.2.0/Core 6.1.1 are qualified selections.
Storage's old collection framing is refused without clearing; fresh private fixture
allocation is required. No production store migration occurs.

## Deferred updates and recovery

Incoming updates are preserved locally, excluded from this release by instruction:
`deferred/upstream-20261106-platform-testkit` (`ec3f9b6`) and the superset
`deferred/upstream-20261106-metrics-platform-testkit` (`a420a85`). The latter selects
Metrics 0.29.4, Platform 0.23.3 and Testkit 0.89.1. Adopt separately after review;
do not merge its old local package-version rows over released metadata.

Preserve failed/interrupted gates `protocol-gate.U4hgFU`, `engine-gate.fCAAoT`
and `engine-gate.zt5gp3` under `target/release-validation/`. The first found the
fixture lifecycle/profile assumptions, which were corrected. The Engine attempts
were invalidated by external lock edits. The successful fresh gate above supersedes
them; no drift guard or receipt was bypassed. `target/delivery-035/` retains delivery
logs. Shared's frozen-release workflow remains separate future adoption work.

Preserve `.git/release-state/`, `.git/publication-state/` and required target custody.
Earlier 0.34.1 publication is complete; its retained attempts are historical, not
pending uploads. Retired checkout refs are `archive/ic-auth-capacity` and
`archive/ic-auth-release-033`, with evidence under
`target/checkout-retirement/20261104-siblings/`. Do not recreate checkouts or rewrite
historical evidence identities. No cleanup was included in this delivery.

The maintainer accepted Platform's residual Wasm increase on 2026-11-05 and lifted
the hold. Its matched Canic report found about 29 KB over 0.21 after correction,
not parity or an Auth optimizer change. Preserve the
[historical scope](https://github.com/dragginzgame/ic-auth/blob/ed785d7/docs/status/current.md).
No new Wasm size comparison is claimed.
