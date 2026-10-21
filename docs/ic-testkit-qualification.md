# IC Testkit proof, session and ingress qualification

`make test-qualification` admits the prepared Testkit server and browser tools,
runs the native fixture tests, builds `apps/qualification/canister/` and executes
`apps/qualification/runner/` against that server. These unpublished applications
qualify Auth's reference host, not a production wallet/issuer or durable backend.
Earlier pairings and benchmarks are preserved in
[qualification history](ic-testkit-qualification-history.md).

## Setup and ownership

The root catalog/lock select published Testkit 0.63.0 from
[IC Tooling](https://github.com/dragginzgame/ic-tooling) and Auth's Platform 0.6.0.
Application identities use owned Principal; direct Testkit/Agent calls convert
explicitly to their retained external type. Testkit also selects Platform 0.6
itself; no dependency override is needed. The remaining native Candid dependencies stay
outside public Auth library and deployed-Wasm graphs. Testkit no longer supplies
Candid helpers or selects Platform's `candid` feature; native Agent/PocketIC
still require their own Candid dependencies.
See [the native Candid paths](development.md#why-candid-appears-in-cargolock) for
the Agent, Testkit and PocketIC dependencies retained in the workspace lockfile.
Platform owns native Agent/delegated identity imports. Agent remains transitive
with its native PEM feature; caller/root-key and request policy remain with the
fixture. There is no direct Agent declaration or local facade. Testkit supplies
typed Wire query/update calls and bounded startup through its upstream runtime
re-export. Only the native runner enables its `wire` feature; IC Wire owns the
codec and finite method limits. Deliberately malformed inputs and byte-level
evidence continue to use raw calls. Testkit owns server selection, authenticated
asset preparation and bounded startup.
`make install-testkit-tools` explicitly prepares the selected CLI/server with
the required `cli` feature; library consumers keep it disabled;
`make testkit-tools-check` admits it offline. Tests do not install or select an
older executable. The browser fixture separately requires explicitly prepared
Node/npm dependencies; see [client setup](development.md#browser-client-setup).

IC Host supplies bounded no-follow Wasm reads and native process mechanics. The
fixture's startup limit is 30 seconds; state, stdout/stderr and SDK scratch inputs
remain under a unique `target/portable-fixtures/qualification.*` directory after
success or failure. The canister uses Platform runtime and generated entrypoint
macros with explicit Wire codec hooks; the direct CDK exception is removed
([Platform #21](https://github.com/dragginzgame/ic-platform/issues/21)).
IC Memory 0.50.3 from [IC Persistence](https://github.com/dragginzgame/ic-persistence)
supplies checkpoint-cell mechanics, and Platform supplies raw length
admission before CDK copying, counter-zero measurements, certified-tree types
and canister-signature key/hash operations. Its `cdk` capability now supplies
certificate read/publish helpers and the original runtime/controller semantics.
The selected Platform runtime and imported certified-verifier stack are Candid-free.
The reference Wasm rejects Candid; native Agent/Testkit still bring it transitively.
Controls and checkpoints remain Wire.
No direct Candid, CDK or Testkit Candid declaration remains.
These host dependencies are absent from Auth's public libraries.
Testkit owns generic typed call transport; Auth retains application refusal
projection, trust/session assertions and distinct profiling controls. Calls keep
explicit target/caller and method identity, bounded encoding/decoding and one
update without retry. Codec/transport errors remain distinct from encoded
application refusals. See
[Auth #75 — Adopt Testkit typed Wire calls](https://github.com/dragginzgame/ic-auth/issues/75).

## Host and trust boundaries

The reference canister uses actual runtime caller, time, canister ID and query
certificate. Controllers alone prepare signatures, change certified branches and
install/confirm test authority. Auth supplies signature-tree contributions;
the host composes assets, signatures and status and publishes the single root.
The caller cannot install trusted issuer/network/Root state through a credential.

The host exposes bounded Wire controller/certificate and application calls.
`ic-auth-wire` owns application layouts and generated TypeScript;
`apps/qualification/contracts/` owns only finite private controls, diagnostics,
init/upgrade arguments and their explicit method identities. Controllers still
authorize state changes after structural decoding. No Candid fallback, projection
wrapper, generated Candid service or parallel codec remains in this fixture.

## Exercised scenarios

The six live runner cases cover:

- Real canister signatures, malformed/domain/key refusal, composed root branches,
  bounded cleanup, malformed/old control transport refusal, failed-upgrade rollback
  and metadata restoration after upgrade.
- Real replica-checked ingress delegation, wrong owner/target refusal and stable
  ingress account identity across a fresh browser signing key and upgrade. This
  ingress identity test is distinct from caller-keyed application sessions.
- Root/issuer signature profiles composing one store without sharing authority.
- Complete portable certified proofs with live Root/issuer authority, signature
  binding, upgrade and rejection checks; no retired Rust token lifecycle is used.
- Two receivers/devices with local portable admission, proof expiry/exact retry,
  receiver-local logout, failed-proof credit retention and actual SDK transport.
  Its issuer callback injects a lost reply and unavailable lookup, then recovers
  exact original credential material after reopening the journal without issuing
  again. This callback does not implement Canic receipt or issuer policy.
- Same-contract checkpoint upgrades retaining sessions/replay/denials while
  starting closed until current matching test authority is confirmed.

Native fixture tests additionally cover memory initialization/retained opening,
competing owners and corruption/refusal. A fixture test and a library unit test
protect different integration/trust boundaries; neither replaces the other.

## Portable session restoration across independent receivers

The checkpoint host uses `ic_memory::testing::FixtureCell` in one explicit
host-owned runtime/allocation. Auth owns its `AUTHWIR2` Wire payload identity, 128 KiB
ceiling and checked metadata/session contents. IC Memory owns Cell framing,
capacity admission and pages. Init creates bounded fresh state; post-upgrade opens
retained state. Missing, corrupt or unexpectedly occupied state refuses. The raw
`AUTHQCP1` layout and the later `AUTHCELL` Candid fixture checkpoint are retired
with maintainer approval, without automatic reset or a second reader.
Qualification starts fresh and tests repeated current-format upgrades; it does
not prove historical ledger/image compatibility.

After upgrade, fresh matching controller confirmation is required before
admission, authorization or logout acknowledgement. Existing leases are not
extended. Restored exact retry works after proof expiry without fresh signature
verification; a retained denial rejects a newly signed renewal for that login,
while its peer's independent session can remain valid. Production receivers
require authenticated Root enrollment and per-operation atomic persistence;
the checkpoint is not the backend acceptance in
[Auth #58 — Support host-selected session persistence with safe volatile reset admission](https://github.com/dragginzgame/ic-auth/issues/58).

## SDK admission through the portable coordinator

The [SDK fixture](../packages/client/test/qualification/admission.ts) uses the
actual client and SDK against two reference receivers, with fake IndexedDB in
Node. Rust supplies one prepared signed credential; the injected issuer returns
it after the journal claims issuance. This does not exercise a production issuer
network call or reconciliation endpoint.

Each destination is journaled before admission. The host rejects raw arguments
above the Method limit before CDK copying, decodes the exact Wire request and
encodes its bounded reply. The SDK authenticates the update reply; caller mismatch
refuses, and exact retry preserves the lease. Ordinary operations carry no proof.
Generate artifacts explicitly with `make generate-client-contracts` after owned
layout/generator changes. Checks use comparison only and refuse stale outputs.

## SDK logout transport against live receivers

The [logout fixture](../packages/client/test/qualification/logout.ts) exercises
actual authenticated Wire updates through the guarded SDK identity. Durable
logout invalidation precedes fan-out; only verified acknowledgements count.
Retaining the raw test key does not bypass receiver revocation. Logout on one
receiver does not claim revocation on another; renewal and peer/device assertions
make the boundary explicit. Unknown issuance remains retained.

Native browser storage, wallet login, issuer signing/renewal, separate-subnet
latency and Canic/Toko deployment are not qualified by this Node/Testkit fixture.
The consumer owns its envelope/export adapter, protected enrollment, atomic
production store and issuer/reconciliation contract. See [Canic adoption](canic-adoption.md).

## Measurement boundary

The diagnostic admission export wraps the same handler as normal admission.
Instruction counter 0 includes domain projection, proof/session work and local
commit; it excludes outer Wire request decoding and diagnostic reply encoding.
Whole-update balance deltas include transport/platform/instrumentation charges.
Native credential/Root signature production is outside receiver samples.

Both proof mechanisms use the same combined Wasm and matching grants, policy and
occupancy progression; the runner also checks budget exhaustion and logout.
Recorded artifact identities and raw samples qualify only their actual source,
not a universal saving or SLA. Current code emits raw diagnostic samples; the
retired token-specific instruction sweep/IC Metrics summary is historical, not
an advertised current command. Earlier numerical comparisons remain in
[the immutable evidence archive](ic-testkit-qualification-history.md).
