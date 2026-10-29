# Current handoff — 2026-10-29

## Delivered base and working selections

The maintainer committed the pending batch as `11b23c749255e81a34a12c4e38c49a336f46eae8`.
Its full gate stopped in `test-testkit-tools`: nested console output omits raw
input-drift excerpts. [#108](https://github.com/dragginzgame/ic-auth/issues/108)
fixes the caller to inspect isolated retained logs. The focused nested reproduction
fails before the patch and the actual validation-runner target passes afterward
(9 seconds); evidence is in `target/testkit-nested-fix-20261029/`. This fix is
uncommitted and the full gate has not been rerun. Release recovery is untouched.

Auth **0.20.0**, `6b27ef1bdd4eebd405a2dd161204a840472ce929`, is committed.
The [release-source handoff](https://github.com/dragginzgame/ic-auth/blob/6b27ef1bdd4eebd405a2dd161204a840472ce929/docs/status/current.md)
retains prior qualification and Canic contribution evidence. Rust **1.91** remains
the package minimum; development uses **1.99.1**. Shared Tooling is **0.27.6**,
exact `5a72359b5be19d5221334cf2c9b3204621d3f3f6`, **127 files**.

Current selections: Memory/Storage **0.54.0**, Host/Build/Testkit and the independently
pinned file CLI **0.69.11**, Platform/macros **0.7.5**, Wire/derive **0.7.21**,
browser Platform **0.1.1** over Core **6.1.1**, PocketIC **16.1.0**, native AWS LC
**1.19.0** / sys **0.47.0**. All three selected CLI helpers were explicitly prepared
and admitted offline. Auth uses the governed cell adapter, so Persistence's Metrics
0.17 adapter transition needs no Auth storage migration. Metrics **0.17.0** is
selected separately by the unpublished host/runner diagnostic example only.

## Current grant-validation and cleanup batch

Pending **0.21.0** changes `PortableAuthority::grants` to `&ValidatedGrants`.
This owned, immutable set validates the existing canonical 16-role/32-scope
limits at protected configuration installation/restoration. The qualification
host stores it directly in enrollment; it retains no second raw grant owner.
Public callers must retain the checked value, borrow it for authority inputs and
use `as_slice()` for raw projections. There is no mutable access or unchecked
Serde path. See [session migration](../sessions.md) and [Canic handoff](../canic-adoption.md).

Constructor success proves shape only. Hosts still select current protected
inputs and advance generation atomically with authority changes. Live clocks,
issuer/key bindings, generation, eligibility, revocation and scope checks remain;
untrusted claims are independently decoded/verified. Signed/public Wire/stored
bytes and package versions remain unchanged. Canic's adoption remains its work;
no sibling source was edited. Earlier caller-predicate/codec cleanup is retained.

Focused constructor/bounds/order, claims/session/revocation, signed-vector,
strict-signature feature-unification and immutable-access doc tests pass. The
configuration projection refuses malformed grants. Independent before/after
builds match the original Wasm byte-for-byte; separate target directories exclude
the retained initial stale-cache attempt. The default cache was rebuilt.

Reversed-order installed reruns reproduce **7,520 → 6,848 instructions** (~9%)
for the small grant set and **313,510 → 29,601** (~91%) at 16 roles × 32 scopes.
The saving applies to ordinary authorization's repeated grammar/order scan, not
BLS/admission or end-to-end login. Installation/restoration still validates grants.
The 0.69.10/0.7.4 dependency build reproduces the post-change counters. Grant-only raw Wasm
is **2,499,267 → 2,497,828 bytes**; after dependency adoption, before the diagnostic
example below, it is **2,497,808**. These are fixture samples, not production workloads.

Prior adoption evidence includes 8 checkpoint/config cases, 32 session cases,
constructor bounds, 68 client/interoperability/packed-consumer cases, strict Clippy,
MSRV, release Wasm and graph/pin admission. Installed certification/composed roots,
authority/checkpoints after upgrade and Node SDK/signed ingress pass. Formatting
preservation, caller/lock refusal, package identity and local/mocked release fixtures
pass. The updated runner preserves live diagnostics and one outer final excerpt.

Shared export is verified against the independently selected source commit.
Prior probe/grant evidence remains in `target/cleanup-20261029/` and
`target/grants-20261029/`; this batch retains inputs, independent builds, native
states and logs in `target/adoption-sanity-20261029/`. Canonical encoder/BLS rules
remain unchanged. No full CI, macOS acceptance, release, staging or commit was run.

## Incoming library adoption

[#107](https://github.com/dragginzgame/ic-auth/issues/107) records the maintainer's
selection of incoming Tooling/file CLI **0.69.11** and Platform/macros **0.7.5**.
All three helpers are explicitly prepared/admitted. Platform's source changes
only optional external Candid/ledger adapters; crypto/macro sources are unchanged.
Metrics/checkpoint tests, strict host/native Clippy, Rust 1.91 Wasm and graph/pin
checks pass. All 68 client tests, unchanged generated contracts, packed consumer
and actual package/external-lock caller cases pass. Four installed certification,
composed-root, checkpoint/authority and SDK/signed-ingress cases pass. Ordinary
authorization reproduces **6,848 / 29,601** instructions on this final selection.
Actual release/recovery/publication callers pass with local/mocked effects.
Inputs/builds/logs are retained in `target/incoming-adoption-20261029/`; its
reference Wasm is **2,500,763 bytes** including host diagnostics. Delivery/CI remains.

## Host admission diagnostics

[#106](https://github.com/dragginzgame/ic-auth/issues/106) now has a local tested
[host recorder](../../apps/qualification/session_metrics.rs), wired after the
engine's final result. Fixed counters retain committed create/replace, retries,
proof-stage/budget/other refusals and numeric quality; volatile reset is independent
of authentication state. No public Metrics dependency, endpoint or checkpoint
format change. See [event/rollback policy](../sessions.md#host-owned-admission-diagnostics).
The maintainer confirmed production reporting belongs in Canic's shared metrics
store/endpoint, observing Auth's existing typed results. This reference needs no
new public Auth metrics API or per-producer endpoint; Canic owns that connection.
Five native cases, strict host/test Clippy, eight checkpoint cases, Rust 1.91 Wasm,
graph/pin admission and an installed checkpoint/authority upgrade case pass.
Initial evidence at Tooling 0.69.10 / Platform 0.7.4 is in
`target/session-metrics-20261029/`; that reference Wasm is **2,498,619 bytes**
(+811 for host diagnostics). Delivery/required CI remains.
[#105](https://github.com/dragginzgame/ic-auth/issues/105) closed using delivered
0.20.0 and its [successful three-host CI](https://github.com/dragginzgame/ic-auth/actions/runs/40838536688).

## Delivered consumer contracts

Auth 0.20.0 changed session reads to `Cow` and authorization to a checked snapshot
projection. Consumers must adopt those APIs plus pending 0.21's validated grants;
see [sessions](../sessions.md) and [Canic adoption](../canic-adoption.md).
The private browser package remains **0.0.0**, unpublished. Local archives retain
source seals/receipts, strict declarations and explicit offline preparation.
Consumer archive custody, production issuer and actual browser acceptance remain
separate; see [client delivery](../browser-client.md#local-archive-delivery).

## Canic integration boundary

Read-only review on 2026-10-29 used Canic `b9ed7f298bdaf8c9cceac142edf7e56b15d33208`
plus its uncommitted work: Auth/protocol 0.20.0, a durable `SessionStore` adapter
and the unpublished `canic-auth` scaffold that still refuses mounting. Canic's
IcyDB adapter consumes upstream reports; Blob contributes to the one application
sampler. No new shared Auth engine API was established. Canic still needs the
`ic-auth-wire` dependency and actual admission/read/logout bindings; Auth owns
the canonical contracts. This inspection did not qualify Canic's dirty source.

[PR #701](https://github.com/dragginzgame/canic/pull/701), source `3c559fd6e`,
supplies retained persistent receiver ownership, ledger/budgets and CI fixture
repairs. Stacked [PR #703](https://github.com/dragginzgame/canic/pull/703), source
`0ab0d1739`, supplies bounded enrollment producer/installer components; its Root
method is reserved, not mounted. Both were open at that review; their current
GitHub status was not rechecked. Their native/codec/graph evidence and earlier
isolated clone were recorded in the PRs and `target/canic-integration-20261028/`.
Normal sibling working checkouts remain untouched.

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns integration
and installed persistent receiver acceptance. Root RPC mounting, approved issuer
registration, refresh scheduling, clock/skew/budget policy and endpoint/lifecycle
wiring remain. [#507](https://github.com/dragginzgame/canic/issues/507) owns
production issuance, lookup-only reconciliation and browser integration. Account
authentication, browser ingress, application authorization and resource ownership
remain distinct; software signatures do not establish TEE custody.

Keep the accepted certified verifier/admission cost; no verification cache or
crypto/trust optimization follows. Preserve all build/release recovery evidence.
The [operational profile](../design/0.6-portable-authentication.md#certified-proof-operational-profile--2026-10-27)
and [adoption contract](../canic-adoption.md) retain the governing boundaries.
