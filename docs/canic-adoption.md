# Canic library adoption contract

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns implementation
and acceptance. This contract describes the published IC Auth boundary; it is not
a second task tracker or evidence that Canic has adopted it. The
[accepted design](design/extraction.md) and
[incorporation review](design/canic-source-review.md) retain ownership and signed
encoding requirements.

The maintainer's broader goal is to reduce duplication through all applicable
shared libraries. This document covers authentication; the
[wider Canic owner review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6078242947)
records other shared mechanisms and their contract boundaries. Library selection
must leave one canonical mechanism while preserving the consumer's authority,
transactions and recovery obligations.

## Published dependency and feature selection

Both `ic-auth` and `ic-auth-protocol-types` 0.1.13 are published and unyanked.
Their registry checksums match IC Auth release
`4421f991065a8bc9fc21d3d70776943b493dc740`. Rust 1.88 is the supported minimum.
Canic selects compatible registry requirements in its root catalog and records
the resolved versions in its own lockfile:

```toml
[workspace.dependencies]
ic-auth = { version = "0.1.13", default-features = false }
ic-auth-protocol-types = "0.1.13"
```

Child packages inherit these dependencies with `workspace = true`; their
existing capability features forward only the needed library features. This is
a proposed Canic selection, not an applied or consumer-qualified manifest.

| Existing Canic capability | IC Auth selection | Retained Canic responsibility |
| --- | --- | --- |
| Protocol DTOs and canonical encoding | Protocol-types and default `ic-auth::canonical` | Fleet/configuration envelopes and checked deployment-identity projections |
| Root/issuer canister-signature verification | `canister-signature-verification` | Protected signer, seed, network trust anchor, time and limits |
| Root/issuer canister-signature creation | `canister-signature-preparation` | Authorized message construction, runtime certificate, certification-root composition and lifecycle |
| `auth-delegated-token-verify` | `token-verification` | Independent key enrollment, fleet/role policy, authenticated caller and endpoint requirements |
| `auth-local-application-authorization` | `sessions` | Durable transaction backend, memory IDs, generation transitions and endpoint guards |
| Root chain-key signing and renewal | Existing Canic effect adapter | Management-canister signing, key eligibility and orchestration; there is no library signing endpoint |

Token verification enables signature verification; sessions enable token
verification. Signature preparation is independently selectable. No feature
imports Canic, Toko, Solana, CDK, stable memory or native Host/Testkit tooling.
Library adoption does not remove a cryptographic dependency from the transitive
graph merely by moving its direct declaration.

## Canonical declarations and protected inputs

Use the protocol package's token, proof, certificate, audience and grant types
as their canonical declarations. Remove superseded generic Canic declarations;
retain Canic's infrastructure/configuration DTOs and endpoint error mapping.
Coordinate exposed facade/macro/Candid surfaces with
[Canic #354](https://github.com/dragginzgame/canic/issues/354). A wire-compatible
data shape does not establish unchanged Rust imports or error semantics.

Project protected network and fleet IDs into `AudienceId` byte-for-byte through
`CanonicalId`; parse the protected role as `AuthRole`. Preserve the `Fleet`
Candid label and exact canonical domains/vectors. These validated identifiers
carry protocol identity, not fleet membership or authority. Fallible canonical
hashes propagate their errors; do not add infallible wrappers or duplicate
encoders to preserve the previous internal API.

Call `token::verify_token` with a `TokenVerificationContext` assembled from the
actual authenticated caller, protected local audience/role, scope ceiling,
endpoint requirements, independently enrolled `RootKeyPolicy`, IC network key,
clock and finite limits. Never populate expected authority from the token under
verification. Key name, SEC1 bytes and derivation path are protected enrollment
facts; a submitted chain-key proof is not their provenance. The
[token contract](tokens.md) specifies exact binding and live-policy checks.

Replace the generic proof callbacks and semantic/cache-only admission path with
the complete verifier. Its `VerifiedToken` cannot be publicly constructed or
deserialized. Do not retain a second positive proof cache that bypasses changed
policy, key windows or epochs. Fleet member/role decisions, issuer approval,
renewal orchestration and application resource authorization stay in Canic.

## Durable sessions and certification

Session admission calls `session::establish_session`; protected operations call
`session::authorize_session` with current context. Use `SessionRequest::new`
for canonical scope/TTL request identity. Do not wrap the old session engine
behind the library or use its public verified-authority constructor to bypass
proof verification.

Canic implements `SessionStore` and `SessionTransaction` against its actual
durable host store. Reads, capacity checks and the staged session/replay pair
observe one isolated synchronous transaction. Closure or commit failure leaves
records, indexes, counters, generation and accounting unchanged. Preserve replay
tombstones through logout, replacement and authority invalidation; advance
protected generation atomically with invalidating configuration changes.
`MemorySessionStore` is a volatile reference and cannot replace durable replay
state in production. The [session contract](sessions.md) owns limits, exact retry
and restore obligations.

IC Auth's `Session::encode`/`decode` is its current record format, not a reader
for existing Canic stable records. Canic owns the frozen-format inventory,
retained-state disposition and release boundary under
[Canic #459](https://github.com/dragginzgame/canic/issues/459). Resolve those
obligations before a storage cut; neither library adoption nor pre-1.0 status
authorizes resetting retained authority or replay evidence.

Signature preparation uses `SignatureStore`, whose `root_hash` is one labeled
`/sig` contribution. Canic composes and publishes the single certification root,
supplies real query certificates and matching `SignatureSibling` witnesses, and
owns pending retrieval behavior on upgrades. The store has no durable restore
codec. Its [preparation contract](signature-preparation.md) and the unpublished
[Testkit fixture](ic-testkit-qualification.md) do not qualify a Canic lifecycle
adapter or durable session backend.

## Consumer acceptance boundary

The maintainer's immediate priority is Canic adoption from the already-published
0.1.13 libraries. The pending 0.1.14 wire qualification introduces no required
API change and is not a prerequisite for starting consumer work.

Protocol-types tests now independently describe Canic's prepare request,
claims/prepare response and retrieval request at the reviewed source above.
They check both Candid directions and identical encoded bytes for metadata,
nested grants/audiences, fixed-width hashes/nonces, principals, unsigned deadlines
and absent/empty/nonempty optional extensions. Invalid nested roles/identities
reject. Native tests pass on Rust 1.99 and 1.88, and the test target checks for
Wasm on 1.88. [IC Auth #8](https://github.com/dragginzgame/ic-auth/issues/8) owns
this fixture evidence; it does not authenticate tokens or qualify actual Canic
generated Candid, macro endpoints or production adapters.

Canic acceptance requires its selected native/Wasm feature graphs, canonical
signed-byte vectors, both Candid directions, facade/macro callers and real host
certification/upgrade/ingress qualification. Durable host tests must cover failed
commit without partial replay consumption, exact retry after restart, logout and
replacement replay refusal, capacity/expiry boundaries and protected authority
changes. Report removed symbols and actual remaining dependency edges when the
superseded owners are retired.

The 2026-10-09 read-only review found no IC Auth dependency in Canic's root/Core
manifests and the local engines still active at committed
`c4c046f947b2b28f4342cbf6efe9221ba1ed5f70`; unrelated dirty Canic work was preserved.
That is source/declaration evidence, not a consumer build or deployment result.
Wallet login is not a prerequisite for these local Rust calls. The private
[browser client](browser-client.md#consumer-adoption) has a separate real-issuer
adapter and publication boundary.
