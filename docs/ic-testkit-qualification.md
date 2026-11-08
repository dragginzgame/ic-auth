# IC Testkit proof, session and ingress qualification

`make test-qualification` admits the prepared Testkit server and browser tools,
runs the native fixture tests and [backend conformance scenarios](sessions.md#backend-conformance-scenarios), builds `apps/qualification/canister/` and executes
`apps/qualification/runner/` against that server. These unpublished applications
qualify Auth's reference host, not a production wallet/issuer or durable backend.
Earlier pairings and benchmark narratives remain in Git history; they do not
qualify the current dependency graph.

## Setup and ownership

The root manifest selects published Testkit 0.93.0 from
[IC Tooling](https://github.com/dragginzgame/ic-tooling), its maintained PocketIC
client and server 16.1.0. The qualified pairing shares Wire 0.17.12 types directly;
the focused Testkit 0.93 qualification and earlier complete Engine 0.35.6 gate
(which selected Testkit 0.92) are recorded in the
[current handoff](status/current.md).
`WireCallSpec` takes the canonical
contract limits without conversion; responses require only `DeserializeOwned`.
The temporary Wire 0.6 alias and limit bridge are removed following
[Tooling #248](https://github.com/dragginzgame/ic-tooling/issues/248).
Application identities use owned Principal; direct Testkit/Agent calls convert
explicitly to their external type. Auth and Testkit 0.93 share Platform 0.25.0
and canonical Types 0.18.2; the Storage host selects Runtime 0.20.0 directly. No alias or
dependency override is used.
Native CBOR errors use Testkit's `CborErrorKind` and retain the typed source;
Auth does not match the retired Platform-error tuple variant.
Native Agent/client dependencies stay outside public Auth library and
deployed-Wasm graphs; the certificate-only dependency exception is separate.
Testkit no longer supplies
Candid call helpers; its native management fixtures select Platform's `candid`
feature, and native Agent/PocketIC retain their Candid dependencies.
See [the native Candid paths](development.md#why-candid-appears-in-cargolock) for
the Agent, Testkit and PocketIC dependencies retained in the workspace lockfile.
Platform owns native Agent/delegated identity imports. Agent remains transitive
with its native PEM feature; caller/root-key and request policy remain with the
fixture. There is no direct Agent declaration or local facade. Testkit supplies
typed Wire query/update calls and bounded startup. The native runner imports
`PocketIc`, `PocketIcBuilder` and `Time` from `ic_testkit::pocket_ic` in Testkit
0.93. Clients require an explicit server endpoint and positive finite request
budget. Auth selects `PocketIcBuilder::new(Duration::from_secs(30))`; its managed
`build_instance(builder).await` supplies the endpoint through the admitted
`PocketIcStartupConfig::Spawn`. Testkit applies one request deadline through
dispatch, response-body reads and accepted-operation polling; the startup
deadline remains separate.
The separate Platform client dependency is removed. The fixture retains the
managed `PocketIcInstance` owner throughout setup and live calls. It borrows the
client through `instance()` or `instance_mut()`. Each live test owns one two-worker
Tokio runtime, awaits network/lifecycle/Wire calls and explicitly awaits
`close()` before stopping its managed server; deletion failures fail the test.
Managed-server Drop still stops/reaps its process group on setup/test panic.
There is no blocking facade or nested runtime. Only the native runner
enables Testkit's `wire` feature; IC Wire owns the
native Serde codec; Auth owns method identities and explicit traversal limits.
Deliberately malformed inputs and byte-level
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
success or failure. The canister selects independent Runtime and its generated entrypoint
macros with explicit Wire codec hooks; the direct CDK exception is removed
([Platform #21](https://github.com/dragginzgame/ic-platform/issues/21)).
Root-selected [IC Storage](https://github.com/dragginzgame/ic-storage)
supplies allocation and checksummed checkpoint cells, and Platform supplies raw length
admission before CDK copying, counter-zero measurements, certified-tree types
and canister-signature key/hash operations. Its `cdk` capability now supplies
certificate read/publish helpers and the original runtime/controller semantics.
The selected Runtime remains Candid-free. Platform's upstream certificate
helpers bring transitive Candid/raw IC0 into the reference Wasm under the
maintainer-approved certificate exception; native Agent/Testkit also bring Candid.
Controls and checkpoints remain Wire, with one Runtime executor/guard and
host-owned clocks, persistence and certification-root publication.
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
`apps/qualification/contracts/` owns only bounded native private controls, diagnostics,
init/upgrade arguments and their explicit method identities. Controllers still
authorize state changes after structural decoding. No Candid fallback, projection
wrapper, generated Candid service or parallel codec remains in this fixture.

## Exercised scenarios

The live runner cases cover:

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

The checkpoint host uses Storage 0.64's `ic_storage::adapter::StableCell` with one
host-owned runtime and checked allocation. Auth owns its `AUTHNAT3` native-Wire
payload identity, 128 KiB ceiling and metadata/session contents. Storage admits
namespace grants, exclusive leases, bounded loading and collection checksums.
Init requires an empty allocation; post-upgrade loads retained state with the
explicit payload ceiling. Creation and writes return checked errors. Missing,
corrupt or unexpectedly occupied state refuses. The
raw `AUTHQCP1`, `AUTHCELL` Candid, finite-Wire `AUTHWIR2` and older CBOR-record
`AUTHNAT1` layouts, plus the text-ID `AUTHNAT2` checkpoint, are retired
with maintainer approval, without automatic reset or a second reader.
For the Auth 0.25 binary-ID change, the maintainer separately confirmed no
retained current Auth session/login records. Old checkpoint bytes remain intact
on refusal; this grants no permission to erase backing or its allocation metadata.
Qualification starts fresh and tests repeated current-format upgrades; it does
not prove historical ledger/image compatibility.

Storage 0.64 replaces the old manager/ledger and collection layouts with its
owned formats. Fresh backing is required, even if an older image contains an
`AUTHNAT3` payload. A frozen Memory 0.63 manager-header test preserves a separate
application sentinel on refusal; no legacy decoder or automatic reset exists.
Native tests also verify read-only checksum-corruption and size refusals.
Installed tests use fresh backing and current-format upgrades. A real retained
installation requires a separately reviewed whole-image transition; this
qualification authorizes no reinstall or data loss.

After upgrade, fresh matching controller confirmation is required before
admission, authorization or logout acknowledgement. Existing leases are not
extended. Restored exact retry works after proof expiry without fresh signature
verification; a retained denial rejects a newly signed renewal for that login,
while its peer's independent session can remain valid. Production receivers
require authenticated Root enrollment and per-operation atomic persistence;
the checkpoint does not qualify Canic's production backend. That consumer
acceptance belongs to [Canic #491](https://github.com/dragginzgame/canic/issues/491).
[Auth #58](https://github.com/dragginzgame/ic-auth/issues/58) retains the delivered
library storage/reset contract's historical evidence.

## Scoped application effects

The private `session_set_asset` Wire update accepts a single `u64` and requires
the stored session's `write` scope. Its host-owned caller/time/authority projection
and shared `authorize_session` call are synchronous with the existing asset
mutation; no proof, claimed identity or client-selected required scope is accepted.
This one fixture asset has deliberately simple scope policy. Application account
mapping and resource ownership remain separate consumer responsibilities.

The [installed case](../apps/qualification/runner/tests/signatures/sessions/application.rs)
checks that a credential granting read/write can be narrowed to a read-only
session, and that real signed writes then refuse without changing the asset.
An independently admitted writer succeeds after its short admission proof expires.
Raw retained keys refuse after local logout, while the independently admitted
session on the other application subnet still writes. Advancing that receiver's
authority generation refuses its next write. Anonymous and missing-session calls
also preserve the application value. This adds no production export or checkpoint
field and does not qualify a production issuer or Canic's host integration.

## SDK admission through the portable coordinator

The [SDK fixture](../packages/client/test/qualification/admission.ts) uses the
actual client and SDK against two reference receivers, with fake IndexedDB in
Node. Rust supplies one prepared signed credential; the injected issuer returns
it after the journal claims issuance. This does not exercise a production issuer
network call or reconciliation endpoint.

Each destination is journaled before admission. The host rejects raw arguments
above the contract byte limit before CDK copying, decodes the exact Wire request and
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

## Real-browser qualification

The opt-in `make test-client-browser` command runs
[the native browser case](../apps/qualification/runner/tests/signatures/sessions/browser.rs)
with [the Chromium runner](../packages/client/test/browser/run.mjs) and
[browser scenarios](../packages/client/test/browser/qualification.ts).
Testkit keeps the client and managed server together in `PocketIcInstance`;
mutable access starts the live gateway without splitting ownership.

The recorded browser run used Testkit **0.69.4**, Linux Chromium **156.0.8078.4**,
Playwright Core **1.62.1**,
Platform npm **0.1.1** and its canonical SDK Core **6.1.1**. It uses an isolated
localhost HTTP origin/profile and actual IndexedDB strict transactions. Two tabs
race issuance; durable logout invalidates a delayed real signature; a real renderer
crash leaves its cleanup lease for another tab to recover. Callback/queued-write
aborts, relaxed durability, closed storage and reopening preserve the journal.
An unavailable transport reaches the original deadline without fabricating an
acknowledgement or discarding unknown issuance. This lifecycle clock is controlled.

The separate SDK scenario signs actual IC requests and verifies replies against
the pinned local IC root. Two application subnets receive two browser devices for
one account. A lost issuance reply is recovered through an injected lookup after
reopening storage; exact retries retain the admission deadline. A lost accepted
logout response is explicitly retried; retained raw keys then refuse at both
receivers, independently confirmed by the native runner.

Native credential signing and issue/lookup are bounded injected fixtures, not a
production issuer. Test keys are deliberately retained to prove revocation;
disposal callbacks do not establish hardware key erasure or TEE custody. Browser
origin authorization, production account/issuer policy, other browser engines and
native macOS browser execution remain separate acceptance. The harness downloads
nothing; [setup](development.md#browser-client-setup) selects a prepared executable.
Per-run output records executable/bundle hashes and retained state. Current local
evidence lives in `target/issue-maintenance-20261027/`.

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
an advertised current command. Earlier comparisons are described in
[the historical qualification journal](https://github.com/dragginzgame/ic-auth/blob/a11f7558bd2187c06fedc739c7a0c36b470ba9ca/docs/ic-testkit-qualification-history.md).
That narrative is not durable custody of its raw measurements.
