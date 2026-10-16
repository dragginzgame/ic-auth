# Browser token client

The private `packages/client/` package implements framework-independent
application-token lifecycle machinery for [IC Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
It is source/build qualified locally, not published on npm. It declares no
canister endpoints and has no Canic or Toko dependency. The opt-in IndexedDB
store has transaction qualification in Node and native Chromium on Linux. Wallet
login and a production issuer adapter remain separate work.

## Portable credential transport for 0.6

`PortableCredential` and `PortableProof` are generated from the Rust type owner.
`credentialBytes` and `credentialFromBytes` use the shared bounded Candid codec;
claims remain opaque canonical signed bytes. A roundtrip neither authenticates
those claims nor selects a trusted issuer key. Both proof variants retain their
existing cryptographic material. Rust/TypeScript vectors qualify both Candid
directions; certified transport also has Rust and TypeScript roundtrip checks.

The shared decoder guards each SDK input-stream read against its remaining bytes.
This rejects truncated final blobs that the selected SDK otherwise silently
shortens. The guard is local to that decode, does not mutate generated descriptors
or SDK prototypes, and preserves valid differing Rust/TypeScript Candid encodings.
The caller's outer byte maximum remains mandatory. Production canister decoding
must separately bound instructions/allocations; browser codecs do not provide that.

These data codecs do not migrate `TokenSessionClient`'s retained issuance intents.
The injected portable admission coordinator below supplies sequencing, not a
production issuer adapter. The old token lifecycle remains pending consumer
transition and explicit disposition of its unresolved issuance records.

### Portable admission coordinator

`PortableAdmissionClient` coordinates one portable proof for an already started
login. Construct it with the existing `PortableLogoutClient`, generation, direct
browser identity, protected issuer and a `PortableAdmissionTransport`. Call
`establish(receiver)` explicitly on first contact or when the receiver requires
admission. On authenticated admission success it returns the durable-generation
guarded identity for ordinary SDK calls; application authorization stays at the
receiver. There is no automatic retry of application operations.

The coordinator atomically claims issuance through the logout journal before
dispatch. Only the first claimant invokes `issue`; all other instances, reloads
and uncertain attempts invoke `reconcile`. That operation MUST only recover the
original credential or refuse as unresolved; it MUST NOT issue, renew, extend a
deadline or infer no effect from absent/pruned evidence. The transport owns exact
request persistence/reconciliation against its actual issuer contract, including
the crash between the journal claim and transport dispatch. Claims/proofs use the
Rust-owned credential codecs. The coordinator does not verify signatures or infer
trusted enrollment from response fields.

One detached credential is cached in memory until its original admission expiry
and reused across receivers. Every destination is durably recorded before admission
dispatch, including failed or uncertain attempts. The qualified adapter supplies
`admissionUntilNs` in the journal's clock domain, conservatively covering the
original proof window; recovery must not reset it relative to retrieval time.
Each transport call is bounded to at most five seconds, receives a cancellation
signal and a temporary signing identity that refuses after cancellation. Logout
invalidates late results and guarded signatures. Already signed/accepted requests
cannot be recalled. Receivers still enforce proof expiry and the complete session
admission contract, even when a client supplies malformed proof bytes.

`issue` and `reconcile` are bound to one issuer/login and immutable grant request;
`admit` must authenticate the update reply and resolve only on actual admission
success. HTTP acceptance alone is insufficient. Current tests use injected
transports, not deployed endpoints. There is no automatic renewal: expired proof
material refuses without allocating a new issuance. Existing receiver sessions
may remain usable through their separately enforced lease. Production transport,
renewal and old-store retirement remain in [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3)
and [Canic #507](https://github.com/dragginzgame/canic/issues/507).

The journal format is unchanged. Its `issuanceUnknown` flag remains conservative
even after successful admission; the coordinator never clears historical evidence.
Do not call `issuing` before this coordinator: it claims issuance itself, and a
preexisting marker intentionally selects reconciliation. Stored old token intents
are neither migrated nor deleted. The portable coordinator is an integration
component, not a second production login service or a replacement wallet provider.

### Retirement of the older token lifecycle

The portable coordinator has not yet replaced `TokenSessionClient`. Removing the
older coordinator/store is conditional on complete portable renewal and qualified
issuer/receiver adapters, including non-issuing recovery at the consumer endpoint.
The application must also account for retained old issuance intents before it
retires their reader: use the original caller, request ID, bytes and saved deadline
for qualified recovery, or retain the unresolved evidence under an explicit
application-owned disposition. Missing/pruned evidence never authorizes reissue
or proves no effect. Opening a new database or switching exports does not resolve
an uncertain operation. Existing formats, stores and exports remain supported
until that coordinated cut; [Auth #3 — Browser token-session client](https://github.com/dragginzgame/ic-auth/issues/3)
owns completion and retirement with [Canic #507 — Non-issuing reconciliation](https://github.com/dragginzgame/canic/issues/507).

Portable admission and logout share one private bounded transport-attempt helper.
Timeout always cancels that attempt's signal and late completion cannot settle the
returned call. Admission retains generation/signing checks; logout retains durable
lease claims, acknowledgement validation and retry policy. The helper neither
creates a retry nor changes stored journal/issuance state.

## Boundary

The [reviewed 0.6 design](design/0.6-portable-authentication.md#browser-origin-and-key-custody)
targets one trusted frontend origin/storage partition, fresh browser principals
separate from account identity, and durable logout generation changes before
network fan-out. Cross-origin SSO/signing brokers are excluded. Persistent login
requires an admitted key/storage adapter; suspended tabs and callbacks consult
durable state, not just broadcasts. This is a future consumer contract, not
behavior supplied by the current injected `SessionSource` described below.

### Implemented portable logout journal and worker

`IndexedDbLogoutStore` and `PortableLogoutClient` implement opt-in cleanup for one
fresh browser login in a dedicated, application-owned database. This is separate
from the token store: no token intents are migrated, cleared or reinterpreted.
The journal stores no credentials or signing keys. It does not supply a production
issuer adapter or portable login/admission transport.

Call `start` once with a fresh direct browser identity, unique generation and
protected issuer. The database cannot be reused for another login or overwrite
unfinished/unknown evidence. The application owns retention and database disposal.
Before issuance call `issuing`; before receiver admission call `destination` with
the conservative admission-window bound from the qualified clock adapter. Neither
method dispatches network calls. New admission windows must leave more than five
seconds inside the five-minute cleanup bound, so immediate logout can be committed.
Retained windows are never silently shortened. Inventory is bounded to 64 destinations including
the issuer. Cross-tab mutations use strict IndexedDB transactions; commit failure
must stop the associated dispatch. Consumers must consult this journal and cancel
ordinary signing/admission/renewal when its phase is no longer `active`, including
late callbacks and suspended tabs. A broadcast alone is insufficient.

Give ordinary SDK agents `guardedIdentity(generation, identity)` instead of the
retained raw identity. It checks the durable active phase, original caller and
generation both before signing and after asynchronous signing completes. Call and
query destinations must already be in the committed inventory; signed read-state
requests are also phase-gated. Storage/clock failures refuse signing. Logout in
another tab therefore prevents a late signature from being returned to the agent.
This does not recall a signature already returned, cancel an accepted request or
protect against application code retaining the raw key. Receiver revocation and
the protocol's residual windows still apply; consumers must separately guard late
application callbacks and cancel renewal/admission scheduling.

`begin` durably changes the phase to `cleanup` and fixes a deadline before any
logout network call. The deadline must cover every known admission window plus
five seconds for a final attempt, and be within five minutes. It cannot be extended
by retrying or reopening. `cleanup` only resumes committed cleanup, using the same
principal; it never reconnects. The injected adapter must send the empty shared
logout request to the selected issuer/receiver using that exact identity. Before
the deadline, a tab without a local signing handle returns the still-pending
cleanup snapshot; it neither finishes shared cleanup nor disposes another tab's
key. Another worker with the handle may continue. The caller may resume on handle
availability or at the fixed deadline, rather than busy-polling without a key.
At the deadline, or when every destination is confirmed, cleanup finishes even
without a local handle. Unknown destinations remain unknown. Disposing retained
key material and usable credentials is an idempotent application-owned operation;
its failure is surfaced and retried when cleanup is resumed.

Each destination has at most eight durable attempts, with bounded jitter and a
reserved final attempt after its known admission window. A retained five-second
claim serializes dispatch across tabs; a crash consumes the attempt. The cleanup
transport receives an identity gated before and after signing to that live lease,
fixed deadline and selected destination. It must use that supplied identity, never
a cached raw signer. Signed read-state polling shares the lease bound. Timeouts abort
the local request, but cannot cancel a request already accepted remotely. Delayed
replies cannot overwrite newer claims or finished cleanup. Only typed `Revoked`
confirms that destination; all other outcomes and transport uncertainty stay
unknown. An issuer confirmation does not erase historical unknown issuance intent.
The fixed deadline ends retries and triggers disposal even when destinations remain
unknown. Immediate global logout is not claimed.

`httpLogoutTransport({ host, methodName, dispose, rootKey?, fetch? })` supplies
the SDK update-call adapter for this worker. Configure the gateway and endpoint
name from application policy; both issuer and receiver endpoints must implement
the shared empty `LogoutRequest` and `LogoutResult`. The adapter creates a fresh
agent with the supplied lease-guarded identity for each attempt, submits once,
and uses authenticated read-state polling. HTTP acceptance is not confirmation.
SDK certificate verification retains its trusted root, target, request ID and
freshness checks; the decoded Candid result is capped at 1,024 bytes. The adapter
does not replace certificate verification or accept query replies as revocation.

The default trust anchor is the SDK's IC root. Local qualification must explicitly
provide its trusted root key; the adapter never fetches a root key from the gateway.
The optional fetch implementation is trusted application infrastructure and must
honor cancellation. Abort reaches submission and polling; after abort, late replies
are rejected. Already accepted requests cannot be cancelled. The journal owns
retry limits and the five-second attempt deadline; callers should use the adapter
through `PortableLogoutClient`, not as an independently scheduled retry loop.
Disposal remains the application's idempotent callback. Tests exercise actual SDK
signing with substituted HTTP responses, invalid certificates, abort and bounded
Candid decoding. Successful deployed issuer/receiver transport qualification and
Canic/Toko adoption remain pending; no endpoint is installed by this adapter.

The clock adapter must supply one qualified, reload-stable clock domain, conservative
IC expiry mapping and monotonic observations. An ordinary `Date.now()` clock is not
qualified by this implementation. Clock-domain changes and rollback reject rather
than extending cleanup. Key custody, clock qualification and production adapter
adoption remain host obligations. Tests exercise fake IndexedDB transactions,
crash/reopen, competing connections, capacity, late admission/replies, issuer
uncertainty and proof-expiry retries. The [native logout fixture](../packages/client/test/browser/logout.html)
also passed against real IndexedDB on Linux Chromium 155.0.8059.40: strict commit,
cross-connection invalidation, late-signature refusal, abort atomicity, reopen,
Capacity retry and disposal. It uses a fixture signing identity; Node tests use
actual SDK signing. Other browser engines/platforms remain unqualified.
After building the client, serve `packages/client/` on localhost and open
`/test/browser/logout.html`; the page reports pass/fail and deletes only its uniquely
named disposable database. This is a qualification fixture, not an application UI.

### Planned 0.6 logout retries

This section is the complete consumer contract. The journal/worker above implements
cleanup scheduling and persistence; production lifecycle wiring remains pending.
At durable logout commit the client MUST cancel admission/renewal retries and
record one finite cleanup deadline. The admitted clock adapter MUST conservatively
map known outstanding proof expiries to its monotonic wait clock, including its
qualified uncertainty margin. Cleanup MUST cover those remaining admission
windows; an unqualified browser wall clock cannot establish that bound. Unknown
issuance cannot be relabeled as a known final proof merely because a local
callback was cancelled. Such uncertainty remains visible if issuer revocation
cannot resolve it before the fixed cleanup deadline.

Until receiver confirmation or that deadline, `Capacity` and transport-unknown
outcomes MUST trigger bounded, jittered retries for that same receiver/principal.
Keep the login signing handle available only to the cleanup path in client code;
this is not a cryptographically attenuated logout key. Ordinary application,
admission and issuance signing remain disabled by the durable logout generation.
If establishment commits after an initial `Capacity`, it reserves a logout slot
and a later retry can remove it. Do not stop all retries merely because the proof
has just expired: a session may have committed before expiry. Where reachable,
make a final logout attempt after the covered admission window, still within the
fixed cleanup deadline. Backoff MUST NOT extend that deadline or increase load
without a finite attempt/concurrency bound.

At the deadline, delete the handle and usable credentials even if some targets
remain unreachable. Retain non-secret status distinguishing confirmed revocation
from unknown remote state; expiry alone is never confirmation of session removal.
If issuer revocation failed, replacement proofs may exist until the absolute
login deadline. A copied key can still exploit that residual window. Crash/reopen
resumes only bounded cleanup when the handle remains available, never login;
a missing handle makes authenticated cleanup unavailable, not successful.

Acceptance MUST cover Capacity followed by late establishment and successful
retry, repeated Capacity through proof expiry, a lost establishment reply whose
session outlives its proof, failed issuer logout with an in-flight renewal, and a
crash during cleanup. Also test `NotYetValid` admission retries with issuer-ahead
time, cancellation on logout, and expiry without automatically minting a new proof.
Consumers MUST retain the verified `(issuer, account_subject)` identity within
the trust domain; equal subject principals from different issuers are not linked
accounts. Wallet-origin behavior remains subject to the separate
[review gate](design/0.6-portable-authentication.md#wallet-account-authentication).

### Current injected client

`TokenSessionClient` accepts an existing authenticated ICP SDK `Identity` through
`SessionSource`. The application owns Internet Identity login, logout UI, account
linking, agent/network/root-key configuration and a globally fresh session
`generation` for every login/reconnect. The core never creates an anonymous
issuer identity or derives trusted context from returned claims.

Issuer operations are injected: `prepare`, `reconcile` and `retrieve`. Adapt
actual generated service methods to these typed outcomes without coercing
unrelated service declarations. `reconcile` must look up the original request or
replay its exact ID against a qualified idempotent endpoint. An issuer without
that capability cannot provide a supported recovery adapter. Unknown transport
errors do not mean pending, TTL rejection or expired authentication.

Exact-ID replay is safe only if the issuer cannot treat an expired or pruned
receipt as fresh issuance. A relative receipt TTL alone does not guarantee that:
replaying after retention can produce different claims with the same request ID.
The adapter must use a qualified replay contract or a lookup that never issues;
missing, expired or uncertain evidence remains `unresolved`, not a definitive
no-effect TTL rejection. A request's receipt TTL and the client's operation
deadline are distinct bounds. Reducing a local timeout does not prove that a
delayed network request cannot arrive after server retention.

The protected `TokenScope` selects issuer, deployment network, policy identity,
network-qualified audience, requested role grants and extension bytes. All of
these, plus authenticated principal and session generation, participate in cache
isolation. Grants must be sorted with unique targets; scopes must be sorted and
unique. Use a new `policyId` when the adapter, trust or issuance policy changes.
Do not obtain these values from untrusted token material.

Every completed flight checks its detached token against each caller's captured
request and current claim validity, after checking the session generation.
The retained key flattens extension bytes, so absent and present-empty extensions
share a namespace. A conflicting option rejects `invalid_material` rather than
returning the other caller's token or replacing its intent. Same-option callers
still share issuance. This fix preserves existing keys and the stored profile;
changing that namespace requires an explicit disposition of retained unknown
effects under [#3](https://github.com/dragginzgame/ic-auth/issues/3).
See [the flight repair](https://github.com/dragginzgame/ic-auth/issues/27).

The client checks Candid shape, bounded material, identity/issuer/scope binding,
claim times and equality between prepared and retrieved claims. It does **not**
verify the root/issuer cryptographic proof chain or confer role authority.
Every protected server call still uses complete local token verification and its
current protected authority. An application token is not an IC ingress identity
delegation or a grant of application resource ownership.

## Lifecycle and failures

Within one client, concurrent login and requests for the same cache key share a
flight. Renewal begins when the remaining token lifetime reaches
`renewBeforeNs`; expired completed material is removed using compare-and-swap.
Returned DTOs are detached, including for concurrent callers.

Before calling `prepare`, the client stores the exact Candid request and its
request ID. If the reply is lost, times out or is unknown, it retains that intent
and returns `prepare_uncertain`. A subsequent call or new client with the same
session/storage calls `reconcile`, preserving the request ID and TTL. It never
allocates a replacement issuance automatically. Only a definitive
`ttlUnavailable` outcome that guarantees no effect permits a smaller candidate
TTL and fresh operation ID. Exhausting those candidates returns `ttl_exhausted`.

The client checks the original exclusive operation deadline before allocating
a fresh request ID and reserving intent, including after a delayed storage read,
a lost CAS reservation or a definitive TTL rejection's storage clear. Completion
at or after expiry returns `retrieval_expired` without another reservation or
issuer call. Storage transactions begun before expiry keep their normal commit
and uncertainty semantics; expiry never authorizes discarding unknown effects.

Only typed `pending` retrieval outcomes are polled. Poll count, per-call timeout,
material size and original operation/retrieval deadlines are bounded explicitly.
A lost retrieval reply returns `transport` and retains prepared material. Poll
exhaustion returns `polling_exhausted`; an exclusive deadline returns
`retrieval_expired`. These outcomes do not discard uncertain issuance state.
Clock values must be monotonic unsigned nanoseconds; future-issued claims are
rejected without a clock-skew allowance. The owner supplies the clock and wait
implementation and must choose valid request/operation limits for its issuer.
An uncertain prepare keeps its original stored exclusive deadline across client
reloads, including when a later instance selects a longer operation lifetime.
At that deadline the client returns `retrieval_expired` without calling prepare,
reconcile or retrieve again, clearing retained intent, allocating a new request
ID or invalidating the authenticated session. Expiry bounds automatic work; it
does not prove whether the original issuance completed. Explicit reconciliation
of retained unknown effects remains an application/issuer recovery obligation.

Prepare and reconciliation also check that saved deadline after a reply arrives,
before interpreting the outcome or changing storage. At or after expiry, even a
prepared response, definitive no-effect TTL rejection or session rejection
returns `retrieval_expired`, preserving the original intent and revision. It
cannot trigger TTL backoff, allocate another ID or invalidate the session.
Replies one nanosecond before the deadline remain eligible for normal processing;
retrieval independently enforces the same cutoff. Session-generation checks
still run first, so a reply to an obsolete login remains `stale_generation`.

Retrieval checks that deadline again after the ready-token storage commit,
before returning completed material to any callers sharing the flight. A commit
finishing at or after expiry returns `retrieval_expired` and retains its bytes
and revision. A subsequent cache lookup may use the completed token while its
claims remain valid; it does not replay issuance or retrieval. The session
generation check still precedes expiry refusal.

A typed `sessionInvalid` result atomically invalidates only the generation seen
by that request. Generation/principal checks also reject late replies after
logout, reconnect or a cross-tab login change as `stale_generation`. Applications
handling a protected endpoint failure should capture its generation before
sending and call `invalidate(seenGeneration)` after a typed session rejection.
Cross-tab notifications update the session owner; the core does not own an
application broadcast protocol. The session owner must cancel a pending login
when logout races it, even if the SDK ignores cancellation. Logout here does not
revoke already issued IC delegations or perform application storage migration.

## Storage and generated contracts

`TokenStore` supplies atomic per-key revision compare-and-swap and detached
bytes. A shared durable implementation must preserve uncertain requests before
network dispatch and on reload, and admit no stale writer. Tokens and requests
use generated Candid codecs, retaining exact principals, blobs and `bigint`
values; there is no legacy JSON reader or fallback format.

`MemoryTokenStore(maximumEntries)` is a bounded volatile reference. Tombstones
retain revisions to prevent deletion/recreation races. All keys, including
inactive generations and tombstones, count against capacity; it performs no
implicit eviction or garbage collection. Owners of durable/shared stores must
supply an explicit retention policy that preserves unknown effects. The core
checks contention bounds but does not promise one shared flight across browser
tabs; atomic intent reservation prevents duplicate initial dispatch, while a
second client may receive `prepare_uncertain` during reconciliation.

`IndexedDbTokenStore.open(options)` implements the same contract using a dedicated
application-owned IndexedDB database and an explicitly supplied `IDBFactory`.
It never opens a default database, shares an application's business stores or
installs a storage polyfill. Supply `databaseName`, `maximumEntries`,
`maximumEntryBytes`, `maximumKeyBytes` and `timeoutMs` explicitly. Entry-byte limits
cover the sum of request/prepared bytes; size them for both Candid components.
For example, after the application has selected its namespace and limits:

```ts
const store = await IndexedDbTokenStore.open({
  factory: indexedDB,
  databaseName: applicationAuthDatabaseName,
  maximumEntries: 128,
  maximumEntryBytes: 2 * maxMaterialBytes,
  maximumKeyBytes: 131_072,
  timeoutMs: 10_000,
});
```

Each connection validates the frozen `ic-auth-token-store-v1` profile and exact
object-store schema. Bounds are part of that stored profile; all connections
must agree. Unknown schemas, corrupted entries and incompatible bounds are
refused without clearing, overwriting or upgrading retained data. Changing the
format, namespace or bounds requires an explicit owner-controlled disposition;
creating another database is not a recovery procedure for unknown issuance.

Compare-and-swap, revision allocation and capacity admission share one read/write
transaction across both stores. Writes require the browser's `strict` durability
hint and resolve only after the transaction's complete event; a successful `put`
request alone is insufficient. Unsupported strict transactions fail without a
fallback. See the [IndexedDB transaction contract](https://w3c.github.io/IndexedDB/#transaction-lifetime).
Native errors abort the whole operation; quota refusal maps to `storage_capacity`,
malformed stored state to `storage_corrupt`, and unavailable/closed storage to
`storage_unavailable`. Operations and opening are bounded by `timeoutMs`; a timeout
does not authorize discarding a possible committed intent. The store retains
all tombstones/uncertain records, performs no eviction or garbage collection,
and closes its connection on a version-change request. `close()` only closes
that connection; it does not clear data or revoke authentication.
Exhausting the unsigned 64-bit revision space also returns `storage_capacity`
without wrapping revisions or modifying retained intent.

The application owns browser storage custody, availability, session-generation
persistence and cross-tab login notifications. A committed request record does
not recreate an SDK identity or authorize a session after reconnect. Browser
storage loss/eviction and device failures are outside these transaction tests;
there is no automatic reset, localStorage fallback or legacy JSON reader.
`make test-client` uses the test-only `fake-indexeddb` implementation for transaction
failure injection. The [native fixture](../packages/client/test/browser/indexeddb.html)
also checks the actual Chromium IndexedDB engine on Linux. Other browser engines
and native macOS browser execution have not been qualified by this batch. The
fixture imports the compiled store/error modules from a localhost test server;
it is not an application UI, production adapter or remote endpoint.

The unpublished `ic-auth-tooling client-contracts generate DIRECTORY` command
exports data-only Candid and all Rust vectors from the protocol crate's DTOs.
`client-contracts check DIRECTORY` checks all TypeScript reply vectors together;
bounded Host file reads reject oversized inputs. Fixture generation and checking
are no longer packaged as a protocol-crate example. Locked
`@icp-sdk/bindgen` generates both runtime IDL descriptors and TypeScript types.
`check-client-contracts` generates into retained `target/browser-client/` and
compares exact checked-in outputs. Retained generation directories record input
digests through the existing Host utility and the actual tool versions. `generate-client-contracts` explicitly updates
them. Cross-language tests decode Rust Candid in TypeScript and decode the
TypeScript result in Rust, including a nat64 larger than JavaScript's safe integer
range. The single SDK decode type assertion follows runtime admission against
that exact generated descriptor; it is not a service declaration conversion.

## Setup and checks

Select Node from `packages/client/.nvmrc` and npm from its `packageManager`.
Then run:

```sh
make install-client-dependencies
make client-tools-check
make check-client-contracts
make test-client
```

Preparation uses `npm ci` with reviewed registry routing and lifecycle scripts
disabled. Ordinary checks use prepared dependencies and offline locked Cargo;
they do not install tools or regenerate the lock. `make test-client` compiles the
client and consumer example, runs Node tests and checks both Candid directions.
Use that Make target rather than running the npm test script without first
preparing the Rust fixture. Build output is ignored in `packages/client/dist/`.
The CI matrix explicitly prepares the selected Node/npm tools and dependencies
before the complete gate; native macOS results remain pending that execution.
No npm publication command or version/release contract is advertised.

## Consumer adoption

[The minimal existing Internet Identity example](../packages/client/examples/internet-identity.ts)
accepts the application's SDK login and issuer adapter, constructs an authenticated
SDK agent and guards late login completion after logout. It does not implement
an Internet Identity UI, issuer endpoints or application authorization.

The read-only Toko review at `44d4e2c6d41e3575b2503e79ffa369129ff3ecf2`
identified these replacement boundaries:

| Existing consumer function | Client replacement / retained owner |
| --- | --- |
| `retrievePreparedDelegatedToken` | Typed bounded retrieval, retaining prepared state on unknown replies |
| `prepareAndRetrieveDelegatedToken` | Persist-before-dispatch prepare/reconcile/retrieve lifecycle |
| `issueTokenWithTtlBackoff` / `retryIssueTokenWithTtlBackoff` | Explicit no-effect TTL outcomes; no parsing error messages |
| `mintDelegatedAuthToken` / `refreshDelegatedAuthToken` | Scope-isolated issuance and single-flight renewal |
| `expireSession` / `reconnectSession` | Captured-generation invalidation; application owns reconnect/login UI |
| Signature/token JSON normalization and caches | Generated Candid byte codecs and application-owned atomic storage |

Toko's current `issue_token(ttl_secs)` and related wrappers do not expose this
request metadata/reconciliation contract. Qualify that boundary before removing
its old implementation, and align the SDK major with the selected client SDK.
Canic's existing prepare workflow has caller/payload-bound replay receipts and
staged-response recovery, but a committed receipt can expire and be pruned by
another prepare. Replaying the original request after that can reserve a fresh
receipt with a new relative deadline. Its production adapter must therefore
qualify non-issuing reconciliation before using exact prepare replay as the
client's `reconcile` operation. This source review does not qualify a Canic
endpoint or provide a production adapter. The exact source, smallest proposal
and consumer acceptance are tracked in
[Canic #507](https://github.com/dragginzgame/canic/issues/507).

Registration, shard routing, project membership, roles, account records, NFT
semantics and business-storage migration remain in Toko. Fleet enrollment,
issuer approval/renewal and endpoint guards remain in Canic. Neither sibling was
edited. This batch implements the reusable core; issue #3 remains open for a
qualified real adapter and coordinated consumer adoption.
