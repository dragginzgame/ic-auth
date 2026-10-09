# Browser token client

The private `packages/client/` package implements framework-independent
application-token lifecycle machinery for [IC Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
It is source/build qualified locally, not published on npm. It declares no
canister endpoints and has no Canic or Toko dependency. Wallet login, durable
browser storage and a production issuer adapter remain separate work.

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

Only typed `pending` retrieval outcomes are polled. Poll count, per-call timeout,
material size and original operation/retrieval deadlines are bounded explicitly.
A lost retrieval reply returns `transport` and retains prepared material. Poll
exhaustion returns `polling_exhausted`; an exclusive deadline returns
`retrieval_expired`. These outcomes do not discard uncertain issuance state.
Clock values must be monotonic unsigned nanoseconds; future-issued claims are
rejected without a clock-skew allowance. The owner supplies the clock and wait
implementation and must choose valid request/operation limits for its issuer.

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
Registration, shard routing, project membership, roles, account records, NFT
semantics and business-storage migration remain in Toko. Fleet enrollment,
issuer approval/renewal and endpoint guards remain in Canic. Neither sibling was
edited. This batch implements the reusable core; issue #3 remains open for a
qualified real adapter and coordinated consumer adoption.
