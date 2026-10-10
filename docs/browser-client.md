# Browser token client

The private `packages/client/` package implements framework-independent
application-token lifecycle machinery for [IC Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
It is source/build qualified locally, not published on npm. It declares no
canister endpoints and has no Canic or Toko dependency. The opt-in IndexedDB
store has transaction qualification in Node and native Chromium on Linux. Wallet
login and a production issuer adapter remain separate work.

## Boundary

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

The Rust owner exports data-only Candid with `export_candid`. Locked
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
