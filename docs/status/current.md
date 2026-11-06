# Current handoff — 2026-11-06

## Published state

**Engine 0.35.1** (`ic-auth`) and **Protocol 0.35.0**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
Release commits and annotated tags are pushed to `origin/main`:

- Engine: `d4f89fd25bf10c7616aa2c69b3477e888d48feb5`, tag `engine-v0.35.1`
  (`62c3a01904c039d4ba2616251f704e1453f8ecc9`). Registry checksum:
  `be50d3e65fc140bf5ed383bf16e61f23ba0434d58bdd23915261c9bc14fdb478`.
- Protocol: `83028f07c75bb80d38867b69ae31cd9e0a2651c4`, tag `protocol-v0.35.0`
  (`fc8755ef3572af17d806aa6dfc8203f93350bc73`). Registry checksums:
  types `6204b7d777028c617275f1f63b15bf7d542d89a7b8735c952edaf6770f5421c2`,
  Wire `b1a16f6109fe1736d81512fb43e3e4e03efcb8e821e023f62d97d18f354136c3`.

Custody: `.git/publication-state/engine-0.35.1.json` and
`.git/publication-state/protocol-0.35.0.json`. Upload/readback evidence:
`target/publication-engine-0.35.1.cnu14y/` and
`target/publication-protocol-0.35.0.P5GKEs/`. There is no unresolved upload.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) is closed as completed.
Canic production adoption remains owned by #491; Canic stayed read-only.

## Released contract and consumer action

Engine 0.35.1 adds the unknown-logout backend checks and adopts Shared Tooling
0.30.8; it does not change production Auth behavior, record formats or crypto.
The [Canic adoption contract](../canic-adoption.md) describes the released cohort.

The 0.35 contract supports protected login windows up to 90 days, retaining
60-second proofs and 30-minute session ceilings. Hosts select their protected
maximum login TTL explicitly; shorter policies remain supported. Known logout
retains denial through its signed immutable deadline. Unseen logout commits
bounded temporary denial and returns `Unconfirmed`, including retries. Keep work
budgets separate. No per-operation inter-canister calls or crypto relaxations
were added; ordinary authorization stays local.

The [clock-independent contract](../design/portable-clock-audit.md) retires
unqualified timestamp-based reopening. Complete durable restore plus fresh
authority remains supported; actual safety-state loss stays closed pending
trusted recovery. The fixture checkpoints never-configured, closed and retained
states. Profiling resets reinstall disposable canisters; they are not a receiver
recovery bypass. The maintainer confirmed no retained affected Rust/private
browser records. Logout Wire identity is `0x415554480014`, login storage identity
`0x415554485323`, fixture payload `AUTHCLK1`, and private journal
`ic-auth-local-logout-journal-v1`. Old bytes refuse without clearing; this does not
authorize resetting Canic's stores.

The [remembered-login client](../browser-client.md#ninety-day-remembered-login)
retains exact issuance evidence and renewal sequences separately from guard
metadata. Unresolved issuance is lookup-only; durable completion permits
browser-key renewal. Initial issuance requires account authentication. Readiness
prepares before new signing, bounds expiry recovery and never replays application
effects or renews while polling. Logout fences late results and retains uncertainty.
Issuer endpoints, durable key custody, clock mapping and Canic/Toko wiring remain
consumer-owned.

## Qualified delivery and dependency selection

Engine 0.35.1 passed all **30 release-gate stages**, including native/Wasm Clippy,
package/MSRV and boundary checks, 92 client tests/interoperability, all 11 ordinary
installed cases, local archives and release/recovery/publication fixtures.
Chromium remains opt-in/skipped. Gate:
`target/release-validation/engine-gate.PlZn5W/logs/validation.tWIw7o`.
Its source is `b4f58af`; `target/delivery-0351/` retains delivery logs.
The release-control fixture took 764 seconds; no timing savings are claimed.
Protocol's prior complete gate remains at
`target/release-validation/protocol-gate.CAofib/logs/validation.JeBDEm`.

The release uses Platform 0.23.2, Testkit 0.89.0, Metrics 0.29.3, Build 0.75.0,
Storage 0.68.0, Wire/derive 0.17.4, Types 0.18.2, Runtime/guard 0.19.3 and Host 0.74.4.
Rust 1.99.1 remains compiler/minimum; PocketIC is 16.1.0. Shared Tooling 0.30.8
at `dc9df707b3b1e616973494eab961c1c214d12260` has 161 verified payloads.
Its generated-underscore recovery fix is included. Frozen release batches remain
disabled pending Auth hook/recovery qualification; this release used the ordinary
guarded workflow. Adoption evidence is under `target/evidence/shared-tooling-0308/`.

The earlier clean private browser archive at
`target/browser-client-packages/package.5x3Jeg/` retains its original 0.35 source
and receipt. It is source/archive delivery, not npm publication or production
issuer acceptance. Storage's old collection framing refuses without clearing;
fresh private fixture allocation is required. No production migration occurred.

## Clean backend-conformance source delivery

The complete entrypoint runs nine scenarios with the existing `Backend` hooks.
Unknown logout covers failed initial/retry commits, spent work credit, non-sliding
`Unconfirmed` retries, mixed known/unknown restoration and isolation from known
logout. Independent encoded backing and broken-backend controls pass.

The matching Engine 0.35.1 / Protocol 0.35.0 archive passed registry-consumer
qualification and all 18 packaged tests. Receipt:
`target/portable-fixtures/backend-package.rm50vk/delivery.json`, clean source
`e78d1a76f3f277e3600eac3b31247cd29f0daf3d`, `dirty: false`, archive SHA-256
`153e106f5bdffe49691f0c7030a96573ac026e17fe8be5a2914ebb0785451fca`.
Archive: that evidence directory's
`target/package/ic-auth-backend-conformance-0.35.1.crate`. Retain archive and receipt
at the consumer before producer cleanup. Canic passes the expanded stable-adapter
entrypoint using its earlier 0.35.0 candidate; new archive adoption and installed
production acceptance remain consumer-owned. Earlier archive receipts remain intact.

## Deferred updates and recovery

The pending dependency diff was reapplied unchanged over released Auth metadata.
It selects Metrics 0.29.5, Platform 0.23.4, Runtime/macros/guard 0.19.4, Testkit
0.89.1 and TOML updates. These selections were excluded from this release and
archive qualification. Their recovery patch is under
`.git/release-state/deferred-engine-0351/`; the working lock retains Engine 0.35.1.
Earlier deferred branches `deferred/upstream-20261106-platform-testkit` (`ec3f9b6`)
and `deferred/upstream-20261106-metrics-platform-testkit` (`a420a85`) remain historical.
Do not merge their old package-version rows over released metadata.

Preserve `.git/release-state/`, `.git/publication-state/` and required target custody.
Older release gates, publication attempts and archive receipts remain historical
under their original identities; no cleanup or recovery bypass was performed.
Retired checkout refs are `archive/ic-auth-capacity` and `archive/ic-auth-release-033`,
with evidence under `target/checkout-retirement/20261104-siblings/`.

The maintainer accepted Platform's residual Wasm increase on 2026-11-05 and lifted
the hold. Its matched Canic report found about 29 KB over 0.21 after correction,
not parity or an Auth optimizer change. Preserve the
[historical scope](https://github.com/dragginzgame/ic-auth/blob/ed785d7/docs/status/current.md).
No new Wasm size comparison is claimed.
