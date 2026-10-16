# IC Testkit signature, token, session and ingress qualification

`make test-qualification` first admits the prepared Testkit CLI/server, then
builds the internal Wasm host in
`apps/qualification/canister/` and runs `apps/qualification/runner/` through
published `ic-testkit` against its admitted server. Both packages are
unpublished. They are
qualification fixtures, not a wallet login service or a durable session backend.

Setup is explicit through `make install-tools` and `make fetch`. Tests run offline:
the wrapper asks the locked Testkit CLI to check its prepared server and consumes
only the returned absolute path. There is no test-time setup, server pin catalog
or retained-bundle fallback. `make install-testkit-tools` explicitly prepares the
CLI and server; `make testkit-tools-check` performs offline admission alone.
The released 0.5.7 lock selects `ic-testkit` 0.35.0, which owns the PocketIC
dependency; runtime types are imported through
its re-export, and typed Candid calls preserve application results separately
from encoding, decoding and replica errors. There is no direct `pocket-ic`
dependency or exact-version pin exception in this workspace.
Released 0.5.3 qualified Testkit 0.34.3 with Host 0.14.5. The earlier
0.34.4 / Host 0.14.8 pairing passes the refreshed complete local gate under
[Auth #36](https://github.com/dragginzgame/ic-auth/issues/36). Testkit
0.34.4 changes its setup CLI to cancel owned downloader descendants on
interruption and retain failure evidence; its library/runtime sources are
unchanged. Consumer checks still require explicit CLI preparation and offline
admission, rather than a test-time download.
Released 0.5.5 selects Testkit 0.34.5 / Host 0.14.9. The earlier Testkit 0.34.6 /
Host 0.14.10 preparation is reviewed under
[Auth #41](https://github.com/dragginzgame/ic-auth/issues/41): published production
Rust sources are unchanged, while two upstream lock-reacquisition tests gain
subprocess isolation. That complete local gate passes all 29 stages,
including four real certification/token/session/upgrade/ingress cases with that
pairing and server 16.1.0. This is Linux/Bash 3.2 evidence; native macOS acceptance
of that source/lock remains separate. The delivered 0.5.6 source instead selects
Testkit 0.34.7 with direct Host filesystem/artifacts 0.15.0 and Testkit's transitive
Host 0.14.10 family. Pending 0.5.7 preserves those selections and qualifies them
under [Auth #42](https://github.com/dragginzgame/ic-auth/issues/42): all 29 local
Linux/Bash 3.2 stages pass, including four real canister cases. Native macOS
acceptance of that source remains separate. Older receipts
do not qualify the later graph. Testkit's library/runtime behavior is unchanged
in 0.34.7; its packaged notes describe contributor tooling adoption.
The actual delivered 0.5.7 graph already selects Testkit 0.35.0 and three Host
packages at 0.15.1. Testkit's removed generic build/IC-tool reexports have no
Auth caller, so no compatibility route or direct IC Build dependency is added.
These native dependencies remain absent from both public library graphs.
The earlier 0.34.7/Host 0.15.0 receipt above remains historical; it does not
qualify the later graph. The pending 0.5.8 gate binds the actual unchanged native
selection to the extended fixture under
[Auth #43](https://github.com/dragginzgame/ic-auth/issues/43).

The selected transitive client and server are both 16.1.0. The IC agent is 0.49.2
and the host CDK is 0.20.3; versions and dependencies are owned by the virtual root.
PocketIC 16.1.0 pins `thiserror` 2.0.18, so the shared lock selects that version;
the existing authentication capabilities are tested with that selection too.

Wasm bytes are read through published `ic-host-fs` with a 16 MiB limit and no
symlink following, through the root-selected filesystem crate. Testkit owns bounded
server startup and retained stdout/stderr; each fresh instance has a 30-second
startup budget, while upstream operation limits remain 30 seconds. Each invocation
prints a unique retained state directory beneath
`target/portable-fixtures/qualification.*`. Its instances use distinct child paths;
SDK temporary files also use that root. Test logs and state remain inspectable
after success or failure. This does not prescribe a production identity store.

The shared `ci/ic-tools.tsv` now selects five tools and has no PocketIC row.
The former consumer matrix and alignment/binary checkers are removed under
[IC Auth #7](https://github.com/dragginzgame/ic-auth/issues/7). The canonical Rust
installer prepares `ic-testkit-server` from the one locked registry Testkit
package, with an explicit `debug` profile and versioned executable/receipt slot.
Testkit setup/check uses `.tools/testkit-server`; its owner authenticates assets
and admits the server. Older executable slots and six-tool bundles remain
retained. A dependency update that changes the selected Testkit package requires
explicit setup and qualification, rather than a test-time download.
If the locked Testkit selection changes during setup/check, the caller refuses
to report the former selection as success and requires another explicit invocation.

## Host and trust boundaries

The host uses the actual runtime canister ID, caller, clock and query certificate.
Controllers alone may prepare signatures, prune leaves or change the other
certified branches. Signing messages has no wallet eligibility meaning here.
Installation supplies a non-anonymous resource owner as protected metadata.
An ingress request cannot select that owner.

The fixture composes `assets`, `sig` and `status` under one published root.
Branch updates preserve the signature contribution; signature cleanup preserves
the other owners' roots. The host publishes after every successful mutation.
Queries obtain an actual data certificate and pass protected sibling hashes to
the library's [retrieval API](signature-preparation.md). The native verifier uses
the PocketIC instance's independently obtained network root, approved signer,
protected expected seed and host clock. Tests configure NNS and application subnets
so the certificate's real subnet trust chain is checked.

The controller-only preparation call now selects explicit seed/domain/message
inputs in this unpublished fixture. A combined-profile scenario uses the existing
`canic-root-role-attestation` and `canic-issuer-delegated-token` identities with
the same payload in one store. Both actual proofs verify under their own protected
profiles; crossed seed/domain use rejects. The 16-leaf fixture budget is shared,
failed capacity admission preserves both profiles, and live duplicate preparation
retains the original deadline. Expiring/pruning the short-lived Root leaf preserves
the issuer proof and frees a slot for that profile. Asset-branch updates and
upgrades preserve host metadata; upgrade clears both pending profiles, whose
re-preparation returns the same public keys. These are signing-profile and composed
root checks, not certified HTTP serving or application authority policy.
See [Auth #35](https://github.com/dragginzgame/ic-auth/issues/35) and
[Canic #60](https://github.com/dragginzgame/canic/issues/60).

The fixture's single stable-memory owner preserves its resource owner and branch
metadata with CDK storage. Pending signatures intentionally remain volatile.
Upgrades restore protected metadata, create an empty signature store and republish
the composed root. This is an explicit fixture lifecycle contract, not a stable
codec for the reusable library or a wallet-service upgrade policy.

## Exercised scenarios

The certificate scenario rejects unauthorized preparation, verifies an actual
Canic-domain signature, preserves retry deadlines, changes another branch without
losing signatures, upgrades the host, rejects lost pending retrievals, and
expires/prunes/reprepares a leaf while retaining other certified state.
It checks the DER identity is unchanged after upgrade and new preparation.

The ingress scenario enables a local HTTP gateway. The IC agent's standard
`Delegation::signable()` owns request-ID encoding; the fixture signs that payload
under `ic-request-auth-delegation`. The agent verifies the canister-signed link
against the local root and submits real signed update envelopes. The replica
authenticates the caller before the independent resource canister checks its
stored owner. Tests distinguish application denial of an authenticated non-owner
or anonymous caller from replica rejection of invalid credentials.

Fresh Ed25519 session keys authenticate as the same fixture principal before and
after both signer and resource upgrades. Previously issued, unexpired delegations
still authenticate after pending leaves are cleared. Wrong target, altered signed
expiration, mismatched session key and expired delegation are rejected by the
replica. Each negative ingress assertion requires HTTP 400/403 with the expected
authentication refusal reason; timeouts, connection failures and application
denial cannot satisfy it. Negative tests deliberately use the IC agent's
unchecked constructor
to bypass its early validation and reach the replica; no library proof verifier
is replaced or bypassed in production code.

## Evidence and limits

The complete-token scenario joins the public bounded Merkle constructor,
native test-key ECDSA root signing and an actual Testkit-certified issuer proof
with `token::verify_token`. It verifies the narrowed local grant, then rejects
the same token after changing caller, audience, scope ceiling, root-policy
deadline, key/proof/registry floors or network trust anchor. Advancing the actual
Testkit clock makes the issuer certificate stale while the claims remain valid.
After actual upgrade, pending retrieval rejects; controller-authorized
re-preparation of the unchanged claims verifies again with the same issuer
identity and preserved composed asset state. See
[Auth #18](https://github.com/dragginzgame/ic-auth/issues/18).

Released 0.4.2 routes that scenario through public
[token preparation and finishing](token-preparation.md): explicit host authority,
presenter, operation, clock and bounds produce the canonical signing hash; the
actual retrieved certified issuer proof is then attached. Subsequent live-policy,
certificate freshness and upgrade assertions still use complete verification.
This exercises the producer API requested in
[Auth #25](https://github.com/dragginzgame/ic-auth/issues/25).

The same certified token, retrieved after issuer upgrade, also passes through
`session::establish_session` and `session::authorize_session` against the bounded
volatile reference store. A different network trust anchor or ungranted narrowing
request rejects without consuming the proof; admission of the identical token
then succeeds. Advancing the actual Testkit clock beyond claims expiry rejects
fresh token verification, while exact retry returns the still-live session with
its original deadline and unchanged storage occupancy. A narrowed protected
scope ceiling or advanced generation rejects authorization and retry. Logout
preserves replay refusal, including after generation invalidation.

The released 0.3.5 extension injects host transaction refusal after successful
verification/staging and before the reference backend's commit. Fresh admission
returns `StorageFailure` with no session/replay records, quota usage, accounting
or expiry-index residue; the identical certified token then admits. Exact retry
past proof expiry also propagates transaction refusal, preserving the committed
pair and original lease before a normal retry succeeds. These are substituted
host failures with real certification and an actual host clock, separate from
consumer durable commits and restart recovery.
[Auth #20](https://github.com/dragginzgame/ic-auth/issues/20) tracks this session
extension separately from completed certified-token acceptance under Auth #18.

The released 0.4.0 extension admits that actual certified token under a protected
root enrollment whose deadline precedes both signed claims expiry and the
requested session lease. Verification preserves signed claims expiry while
reporting the capped proof deadline; session admission caps the committed lease.
Actual host-clock advancement checks unchanged exact retry and authorization
before the cutoff, then rejection at the exclusive cutoff without changing
accounting. Bounded pruning removes only the expired session. Restoring a
currently eligible enrollment still cannot consume the same proof again; replay
remains until signed claims expiry, after which bounded pruning reclaims it with
no accounting residue. [Auth #22](https://github.com/dragginzgame/ic-auth/issues/22)
owns this qualification. Canic must still decide its authority semantics and
qualify its real durable backend; this test changes no library policy.

Root signing in this scenario uses a deterministic native fixture key. It does
not qualify management-canister signing, a production token issuance endpoint,
browser reconciliation, stable session storage or Canic's adapter. The canister
remains a controller-guarded signature fixture.

The focused command is part of the configured CI/release gate on Linux and both
supported macOS architectures. Released 0.4.0's complete gate passes on all three
hosts in [matching-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38128802347).
The released 0.4.2 preparation journey passes the complete local Linux gate and
strict native/Wasm Clippy. Its [matching CI](https://github.com/dragginzgame/ic-auth/actions/runs/38138739513)
passes Linux; both macOS jobs remain queued at inspection. Previous-release
results do not qualify the current browser-client repair.

Fresh session keys stand in for browser keys; no browser UI, Solana wallet proof,
challenge/replay service or wallet-to-principal mapping is exercised. The fixed
fixture seed does not choose the wallet service's identity namespace. No stable
session transaction backend, Canic adoption, production deployment or performance
improvement is established. Wallet identity/linking/recovery, allowed origins and
targets, and maximum credential lifetimes still require the explicit decisions
in the [accepted design](design/extraction.md). The existing
[Canic extraction tracker](https://github.com/dragginzgame/canic/issues/491)
remains the adoption owner. Exact local evidence belongs in the
[current handoff](status/current.md).

## Portable session restoration across independent receivers

The portable receiver test uses the same caller-bound credential on two actual
canisters. One enables a host-owned checkpoint and revokes a login locally;
the peer retains that login and another device's independent session. After
proof expiry and a same-contract upgrade, the checkpoint receiver refuses
admission, authorization and logout acknowledgement until its controller confirms
current matching authority. A production receiver must obtain authenticated
Root enrollment instead of using this fixture-only controller endpoint.

After confirmation, the original live session is unchanged. An exact retry with
the expired claims and an empty signature returns that same lease, demonstrating
that restored exact retry avoids fresh cryptography. The local denial survives
and rejects a newly signed credential for the same immutable login; that exact
new credential succeeds on the peer receiver. A second upgrade preserves disabled
authority and denial retention, while peer sessions remain usable. Logging out
the peer's login then denies it locally without disturbing the other device.

The executable case is
`checkpoint_upgrade_preserves_sessions_denials_and_requires_fresh_authority` in
the [native runner](../apps/qualification/runner/tests/signatures/sessions.rs).
Run it with the existing focused `make test-qualification` command. It covers
reference-host protocol/lifecycle behavior, not Canic's actual facade, production
per-operation stable writes, receiver-bound Root enrollment, separate-subnet
latency, browser renewal or deployed acceptance. The fixture retains state by an
explicit upgrade checkpoint; it is not the production durable adapter sought by
[Auth #58 — Persistence and safe reset policy](https://github.com/dragginzgame/ic-auth/issues/58).

## Checkpoint runtime ownership with IC Memory 0.44

The qualification host retains one `MemoryRuntime<DefaultMemoryImpl>` per boot.
Initialization and post-upgrade supply the platform backing; the host constructs
and bootstraps that runtime with its existing declarations, pool and
`GenericAllocationPolicy`. Checkpoint operations open their existing permanent
key through this retained runtime. There is no library-owned default manager or
parallel memory owner. IC Memory 0.44 removed the old global convenience helpers;
this is an adapter update, not a Rust compiler-minimum change.

The namespace, checkpoint key, schema, bucket defaults, header, payload bytes and
128 KiB bound are unchanged. Native tests reject opens before initialization and
a competing bootstrap without replacing the checkpoint, then drop/reconstruct
the runtime over retained backing and check the same allocation and bytes. The
actual-canister suite covers repeated upgrades and restored session/replay/login
state with fresh-authority readiness. These checks remain reference-host evidence,
not a production durable session adapter.

The upstream 0.44 physical ledger contract is incompatible with 0.43. A separate
controlled test installed the retained 0.43 Wasm and upgraded to this 0.44 host;
bootstrap refused the old ledger with `unknown field store, expected physical`.
The old physical marker is `ICMESNAP`; the new one is `ICMDIRCT`. Auth's unchanged
checkpoint payload does not provide compatibility for the containing allocation
ledger. Each normal qualification run already installs fresh disposable canisters;
use fresh 0.44 fixture state and retain old failure evidence separately. Never
silently reset retained memory or add an old-format fallback to pass bootstrap.
Production retained-state decisions remain with the owning consumer and
[Memory #44 — Allocation pooling and downstream composition](https://github.com/dragginzgame/ic-memory/issues/44).

## SDK admission through the portable coordinator

The [admission fixture](../packages/client/test/qualification/admission.ts) runs
`PortableAdmissionClient` against two live reference receivers using the real
SDK, explicit trusted Testkit root and the existing journal/guarded identity.
Rust first confirms this fresh browser principal has no session on either
receiver, then supplies one signed portable credential. The injected issuer
operation returns that prepared credential only after the coordinator commits
its unknown-issuance marker; there is no production issuer network operation.

The private adapter maps the reference `SessionInput` in
[the fixture contracts](../apps/qualification/sessions.rs), using the generated
portable credential descriptor. It authenticates an update reply before accepting
admission, records each destination before dispatch and permits one update
submission per attempt. A mismatched ingress caller refuses. One prepared proof
admits both receivers, and an explicit exact retry preserves the original lease.
Ordinary calls carry no proof and use the returned guarded identity.

The same journal then completes actual SDK logout on both receivers, retaining
unknown issuance and invoking its disposal callback. Guarded signing refuses
before dispatch; deliberately retaining the raw test key still yields receiver
authorization refusal. Rust also submits newly signed renewal credentials and
checks they refuse while proof/authority validity remains live. This separates
browser invalidation from receiver revocation and expiry.

This is private reference-endpoint qualification with fake IndexedDB in Node.
Canic still owns its endpoint wrappers, receiver-bound enrollment, durable atomic
store and production issuer/reconciliation contract. No public adapter factory,
wallet login, issuer signing cost, automatic renewal, native-browser storage,
cross-subnet latency or final consumer facade budget is qualified here.

## SDK logout transport against live receivers

The portable session/ingress case now invokes the actual browser-client
`httpLogoutTransport` against two live canisters after Rust admits the same fresh
browser principal on both. The SDK receives Testkit's trusted network root from
the fixture; it does not fetch a trust anchor from the gateway. It authenticates
signed update/read-state traffic and decodes the shared Rust-owned logout result.

The Node client commits both destinations in its journal, begins logout before
network I/O and runs the existing journal worker. It requires exactly one update per
receiver, two authenticated `Revoked` results, completion/disposal-callback
invocation, retained unknown issuance evidence and refusal of the ordinary signing capability after
logout. Rust then checks receiver authorization and re-admission refusal.

`make test-qualification` admits the already prepared Node/npm/dependency selection
and rebuilds current TypeScript offline. Prepare it explicitly with
`make install-client-dependencies`; checks never install. The native runner uses
published `ic-host-process` with a 30-second child deadline and separate 64 KiB
stdout/stderr limits, retaining diagnostics on refusal. No Host process machinery
is selected by either published library.

The [SDK fixture](../packages/client/test/qualification/logout.ts) uses real SDK
signatures, network calls and certificate verification, with fake IndexedDB in
Node for the journal. It is not a native-browser storage test, production issuer
renewal/reconciliation adapter, Canic endpoint qualification or cross-subnet
latency measurement. The canister acting as the journal's issuer destination is
another reference receiver; it does not implement a production issuer service.
Existing [transport rejection tests](../packages/client/test/logout-http.test.ts)
independently cover invalid certificates, abort, single submission and bounds.

## Matched portable receiver costs and failed-proof throttling

`make test-qualification` now selects both complete portable verifiers in the same
internal receiver Wasm. Controller-installed `ReceiverAuthority` chooses the
mechanism, key binding, Root policy and network anchor independently of requests.
Both arms use the same application subnet, role/grants, lease policy, endpoint and
volatile store, with three fresh browser logins and matching occupancy progression.
The first proof is also admitted on a second receiver. Native credential/root
signing and issuer preparation are outside every receiver sample.

The profile endpoint wraps the actual admission implementation. Counter 0 excludes
Candid request decoding and response encoding; cycle-balance deltas include the
whole instrumented update and platform charges. Raw samples, artifact hash/size
and lifecycle outcomes are retained in the qualification log. These are small
controlled observations, not a throughput distribution or a production price.

Final local observations, Testkit 0.48.0/server 16.1.0, IC Memory 0.43.0:

| Receiver operation | Ed25519 instructions / cycles | Certified instructions / cycles |
| --- | --- | --- |
| Fresh admission, median of three | 2,712,150 / 13,082,165 | 443,640,053 / 459,554,099 |
| Admission on a second receiver | 2,727,784 / 12,866,537 | 457,897,630 / 473,647,681 |
| Exact retry after proof expiry | 59,767 / 10,431,472 | 64,296 / 15,986,087 |
| Invalid proof | 393,966 / 10,526,653 | 239,954,025 / 255,545,956 |
| Next browser caller, verification credit exhausted | 244,729 / 10,606,567 | 97,058 / 15,937,206 |

The invalid Ed25519 case corrupts an issuer signature; the certified case uses a
valid-point network key from another network, reaching BLS signature rejection.
They are distinct negative inputs, not universal worst-case rejection costs.
Both deny session use. With one receiver-wide credit and a one-second refill,
the next principal is throttled before proof verification; logout still succeeds
using its separate budget. Switching principals cannot obtain fresh credit.
Fixture-controlled reinstall resets the state before the valid-cost comparison;
it is not an enrollment-refresh procedure for production.

Both ordinary session authorization windows measured 9,640 instructions; whole
session-read updates burned 7,889,379 / 7,435,474 cycles respectively. Local logout
preserved the peer receiver's session, and expired-proof retry kept its original
lease. The fresh-admission cycle medians differ by about 97.15% under these inputs.
This measures ordinary Ed25519 verification, without TEE hardware qualification;
it excludes wallet login, issuer signing/renewal, durable writes and subnet latency.

The combined qualification Wasm is 3,148,668 bytes, SHA-256
`855fd3feed660db23582d8f187e2119620ffa194f1bddad9421a6297eb3db2f8`.
It contains both verifiers and other fixtures; this is not a minimal receiver
footprint comparison. State directories are fresh per run. The private fixture
configuration/checkpoint now includes its authority and budget policy; rebuild
and start fresh qualification state rather than importing older fixture snapshots.
No public credential or session storage format changed.

[Auth #59 — Bound expensive invalid-proof admission](https://github.com/dragginzgame/ic-auth/issues/59)
retains production issuer/enrollment, abuse and workload acceptance.
[Canic #525 — Session authorization instruction budget](https://github.com/dragginzgame/canic/issues/525)
still requires the actual endpoint facade at maximum durable state; these isolated
receiver samples do not satisfy that consumer gate.

## Isolated session instruction qualification

The certified-token case also produces one-scope and maximum-width (16 scopes,
64 bytes each) sessions through complete real certificate/proof admission. The
fixture controller installs the encoded trusted record and independent protected
role/audience/scope/Root policy in the internal canister. Unauthorized installation
is refused. A query uses actual caller/time and measures `Session::decode` and
`authorize_session` separately with the message instruction counter (0); context creation,
Candid decoding, host stable-row projection and response encoding are excluded.

The native runner uses published `ic-metrics` 0.9.0 to summarize five consecutive
allowed-query observations for each installed record shape. Decode, authorization
and total instructions have separate ordered summaries, labelled with scope count,
record bytes and units. Reports include sample count, floor mean and maximum.
Each raw observation is retained and independently checked against the budget;
refusal/recovery/time-change cases are not pooled into these summaries. Sampling
and reporting run outside the measured canister. Metrics is a runner-only
dev-dependency; the measured Wasm and published authentication graphs do not
select it. These small fixture samples are not universal performance estimates.

The read-only transaction adapter traps if authorization asks for admission
limits, replay/occupancy or writes. Tests cover repeated reads, generation changes,
Root key replacement, a narrowed Root deadline, scope narrowing and missing
required scope, missing records, disable, combined stale/scope rejection and exact
expiry after advancing the replica round. Every measured SDK segment stays below
one million instructions; samples and certified fixture state are retained by
`make test-qualification`. This bounds these fixture inputs, not a universal
budget or Canic's complete facade. Canic's own measurements and denial mapping
remain [#525](https://github.com/dragginzgame/canic/issues/525).

The fixture remains volatile across upgrades. Controller-installed records are
trusted restoration inputs, never login credentials or a stable backend. The
measurement creates no new wallet endpoint, signing authority or public library
API, and cannot qualify Canic's atomic storage, timers, logout or browser adapter.
