# Current handoff — 2026-11-06

## Authorized delivery in progress

The maintainer authorized delivery of the pending Auth batch. Use the normal
checkout on `main`, direct `origin/main` delivery and crates.io publication:
Protocol 0.35.0 first, Engine 0.35.0 second. Run the standard exhaustive release
plans; focused implementation evidence is not a release gate. Canic stays read-only.

## Pending contract

Protected login windows now support up to 90 days, retaining 60-second proofs and
30-minute session ceilings. The ledger requires an explicit host maximum login
TTL before receiver-local time. Shorter hosts retain shorter policy/storage bounds.
Private canister fixtures still choose one day. Signed claims and session/replay
layouts and identities are unchanged.

The [remembered-login client](../browser-client.md#ninety-day-remembered-login)
persists issuance/renewal sequences and one exact credential separately from small
metadata. Unresolved sequences are lookup-only; completed evidence permits
browser-key renewal without the account provider. Initial issuance still needs
account authentication. Receiver-bound readiness identities prepare before new
call/query signing after resume, bound expiry recovery to two attempts, and never
replay application effects or renew during polling. Logout fences late results
and atomically deletes evidence. Key custody, issuer endpoints, scheduling clock
mapping and actual Canic/browser wiring remain consumer-owned.

[#127](https://github.com/dragginzgame/ic-auth/issues/127) is implemented locally;
its [clock-independent contract](../design/portable-clock-audit.md) replaces the
unqualified skew precondition. Known logout retains committed denial through its
authenticated immutable login deadline. Unseen logout commits bounded temporary
denial and returns `Unconfirmed`, including retries. Pools/work credit stay
separate. No new inter-canister call, receiver challenge, credential-bearing
logout or cryptographic relaxation is added; ordinary authorization remains local.

The timestamp reset helper is retired. Complete durable restore plus fresh
authority remains supported; actual safety-state loss stays closed pending trusted
recovery. The fixture refuses reinstall/fresh authority after losing its checkpoint.
Browser cleanup has finite local work/key-disposal budgets, retains uncertainty
and never infers remote confirmation from waiting or an expiry estimate.

The maintainer confirmed no retained affected Rust or private browser records.
Logout Wire identity is `0x415554480014`, login storage identity `0x415554485323`,
fixture payload `AUTHCLK1`, private journal `ic-auth-local-logout-journal-v1`.
Old bytes are refused without clearing; update endpoints/generated clients together.
Canic #491 owns consumer installation disposition and production acceptance.

## Current selections and implementation evidence

Build 0.75.0, Storage 0.68.0, Testkit 0.89.0, Wire/derive 0.17.4, Platform 0.23.2,
Types 0.18.2, Runtime/guard 0.19.2, Metrics 0.29.3 and Host 0.74.4 are selected.
Shared Tooling 0.30.6 at `45c0295e23d3d23225e31f47c018c30ff0c75581` has
159 canonical payloads. Rust 1.99.1 remains the compiler and minimum. Build's
editor and Testkit's PocketIC 16.1.0 are prepared/admitted offline. The browser
uses published Platform SDK 0.2.0/Core 6.1.1.

Storage's collection-image cut requires fresh private fixture allocation; old
framing is refused without clearing. Collection checksums are retired upstream;
Auth keeps its payload identity and bounded Wire admission. No production store
migration occurs. The unused private fixture Serde dependency is removed.

`target/evidence/issue127-fix/` contains passing 86 claims/ledger/session/Wire,
15 certified-proof, 15 backend conformance and 12 checkpoint cases; 92 client
tests, interoperability/packed-consumer checks, five installed session cases,
native/Wasm Clippy and docs/snapshot checks. Chromium remains opt-in/skipped.
Only the logout binding changes among the five regenerated browser files.
Earlier readiness evidence: `target/evidence/remembered-login-readiness/`.
Earlier upstream gate evidence: `target/evidence/testkit089-wire0174/` and
`target/evidence/upstream-20261106/`. None is a complete release gate for this cut.

## Previously published state and recovery

Engine 0.34.1 and Protocol 0.34.0 are published. Engine release commit
`e483aea9407d450f37c131de086ad648047176e0`, tag `engine-v0.34.1`; registry archive
SHA-256 `60bddb026bd4bce91da677e982fc363d73a4047cfcf41ddc9befc4e5c601826d`.
Protocol release commit `106fda655ed55a3542e9383331132f9491904768`, tag
`protocol-v0.34.0`. Custody: `.git/publication-state/engine-0.34.1.json` and
`.git/publication-state/protocol-0.34.0.json`.
Engine's complete 30-stage gate passed at
`target/release-validation/engine-gate.6W8p07/logs/validation.IMS6Ub`.
Publication evidence: `target/publication-engine-0.34.1.IAcRHm/`.

The maintainer accepted Platform's residual Wasm increase on 2026-11-05; the
publication hold is lifted. Platform's matched Canic report found about 29 KB
above 0.21 after correction, not parity or an Auth optimizer change. Original
scope is retained in [Git history](https://github.com/dragginzgame/ic-auth/blob/ed785d7/docs/status/current.md).
No new size comparison is claimed. Platform browser SDK 0.2.0 delivery remains
at source `d30e01c`, tag `browser-v0.2.0`, supplier `target/browser020-delivery/`.

Preserve `.git/release-state/`, `.git/publication-state/` and required target
artifacts, including failed attempts. Earlier 0.34.1 drift/recovery attempts are
superseded by the successful release, not unresolved uploads. Retired checkout
refs are `archive/ic-auth-capacity` and `archive/ic-auth-release-033`; evidence
is under `target/checkout-retirement/20261104-siblings/`. Do not recreate those
checkouts or rewrite historical source/evidence identities. Ignored artifacts
remain removable by explicit clean. No cleanup is part of this delivery.
