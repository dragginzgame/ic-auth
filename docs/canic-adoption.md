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

Both `ic-auth` and `ic-auth-protocol-types` 0.3.2 are published and unyanked.
Their registry checksums match IC Auth release
`c3d0877c1ad5481dedb59dc810a25c18182d0a00`. Rust 1.88 is the supported minimum.
For adoption of this release, select compatible registry requirements in Canic's
root catalog and record the resolved versions in its own lockfiles:

```toml
[workspace.dependencies]
ic-auth = { version = "0.3", default-features = false }
ic-auth-protocol-types = "0.3"
```

Child packages inherit these dependencies with `workspace = true`; their
existing capability features forward only the needed library features. This is
the recommended published selection. The read-only review below distinguishes Canic's
in-progress source adoption from consumer qualification.

Releases 0.3.0 through 0.3.2 change developer tooling; their authentication APIs and
signed bytes are unchanged from 0.2.11. Canic's changing root manifest and lock select
0.3/0.3.2 in the read-only review below; selected versions alone do not establish
runtime adoption or removal of duplicated implementations.

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
libraries. Published 0.2.0 carries complete proof-transport qualification and
the developer-tooling hard cut; authentication APIs and signed bytes are unchanged.
The current read-only inspection at Canic committed base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6` finds its changing working tree
selecting compatible `0.3` requirements and resolving both libraries at
0.3.2 with matching registry checksums. Complete token verification is already available. Consumer dependency
changes require its own qualified lockfiles and remain owned by Canic.

The pending compatible Auth 0.3.3 batch adds
[complete-token Testkit qualification](ic-testkit-qualification.md), joining the
published bounded Merkle builder and complete verifier with an actual certified
issuer. Live caller/audience/scope, root deadline/epoch, network-key and certificate
freshness rejection, followed by upgrade and re-preparation of the same claims,
pass locally. Root ECDSA signing is native test-only signing. This reinforces the
existing adoption boundary without a new runtime API or a production issuer
adapter; Canic's actual endpoint acceptance remains separate.

Protocol-types tests now independently describe Canic's prepare request,
claims/prepare response and retrieval request at immutable Canic
`c4c046f947b2b28f4342cbf6efe9221ba1ed5f70`.
They check both Candid directions and identical encoded bytes for metadata,
nested grants/audiences, fixed-width hashes/nonces, principals, unsigned deadlines
and absent/empty/nonempty optional extensions. Invalid nested roles/identities
reject. Native tests pass on Rust 1.99 and 1.88, and the test target checks for
Wasm on 1.88. [IC Auth #8](https://github.com/dragginzgame/ic-auth/issues/8) owns
this fixture evidence; it does not authenticate tokens or qualify actual Canic
generated Candid, macro endpoints or production adapters.

Released 0.2.0 adds complete retrieved-token and delegation-proof
fixtures, including certificate/leaf audiences and grants, batch headers, both
Merkle sibling directions, seed bindings, key identifiers, nested derivation
paths and opaque root/issuer signatures. Empty and nonempty issuer material is
preserved as transport; synthetic keys/signatures are deliberately unauthenticated.
Malformed roles/identities in the certificate or issuer leaf reject even within
a complete token. All eleven wire tests pass on Rust 1.99 and 1.88; the test
target checks for Wasm on 1.88. These independent fixtures describe the immutable
Canic DTO source, not the concurrently edited consumer adapter.
Released 0.2.0 also cuts the developer PocketIC setup over to Testkit
under Shared Tooling 0.2.0; that tooling change does not require a new Rust
authentication API or delay Canic's runtime verifier adoption.

Canic acceptance requires its selected native/Wasm feature graphs, canonical
signed-byte vectors, both Candid directions, facade/macro callers and real host
certification/upgrade/ingress qualification. Durable host tests must cover failed
commit without partial replay consumption, exact retry after restart, logout and
replacement replay refusal, capacity/expiry boundaries and protected authority
changes. Report removed symbols and actual remaining dependency edges when the
superseded owners are retired.

The initial 2026-10-09 review found no IC Auth dependency in Canic's root/Core
manifests at committed `c4c046f947b2b28f4342cbf6efe9221ba1ed5f70`. During the next
read-only inspection, the consumer working tree began selecting 0.1.14 and
adapting its canonical/proof code. This is uncommitted source evidence, not
consumer qualification or deployment from this source inspection. The owner
subsequently recorded 202 passing focused encoding/configuration/facade cases in
[its adoption handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6079111719).
Those qualify its retained input snapshots, not later graph changes or complete
verifier/host adoption. The next boundary below remains in the same owning issue.
Wallet login is not a prerequisite for these local Rust calls. The private
[browser client](browser-client.md#consumer-adoption) has a separate real-issuer
adapter and publication boundary.

## Current Canic adapter boundary

### Published batch construction extraction

Published IC Auth **0.2.11** provides the pure, default-feature
`chain_key_batch::merkle_root_and_witnesses(&leaf_hashes, max_leaves)` constructor;
see the [batch contract](chain-key-batches.md) and
[Auth #15](https://github.com/dragginzgame/ic-auth/issues/15).
The reviewed Canic helper matches committed
`ac55e50334dd6479ec36f404e89e60bcfe9184d6`; the wider consumer tree is dirty.

The batch construction caller in
`ops/auth/delegation/chain_key_batch/mod.rs` can pass its existing ordered leaf
hashes and protected 64-issuer limit to the library. Preserve principal-byte
sorting and duplicate issuer rejection before construction, then zip returned
witnesses with the same leaf order. The shared builder replaces the generic
Merkle construction machinery. Canic retains its authorized leaf records,
batch identity/header policy, management signing, stable state and renewal.
Retirement requires the consumer's actual builder/verifier, signed-byte and
selected-feature tests; Auth's golden vector and real-signature tests alone do
not qualify that adapter. Both batch construction and complete verification are
available for consumer adoption now.

### Complete verifier adoption

The in-progress Canic `ops/auth/delegated/canonical/mod.rs` calls the library's
encoding/hash functions through checked role, audience, grant, certificate and
claims projections in `ops/auth/delegated/protocol/mod.rs`. Root policy and
delegated registry snapshot framing remain Canic-owned. The protocol DTO
re-exports include metadata, chain-key header/witness/signature atoms and issuer
signature types. Remaining local token/certificate declarations and projections
do not establish canonical type adoption complete. Preserve the owner release
boundary and retained-data obligations while converging these callers.

The next reusable engine remains the runtime verifier:

| Current Canic caller | Library boundary | Canic obligation |
| --- | --- | --- |
| `ops/auth/token/verification.rs::verify_with_embedded_proofs` and `delegated/verify.rs::verify_delegated_token` | `token::verify_token` | Project the complete token once; supply independently enrolled key policy, actual caller, protected fleet/role and endpoint requirements |
| `ops/auth/token/verifier_config.rs` | `RootKeyPolicy` and `TokenVerificationLimits` | Keep build-network/key-name admission and all numeric limits in protected configuration |
| `verification.rs::verify_from_positive_cache` | Live policy on every `verify_token` call | Retire the old proof-skipping path; do not feed a semantic-only result into session admission |
| `ops/auth/application_authorization.rs::local_application_authorization_authority` | Protected `allowed_scopes` and session context | Use the local configured ceiling where enabled; define the token-only verifier's protected ceiling separately instead of taking it from submitted grants |

Preserve endpoint error/metric mapping with typed library errors. Qualify changed
key windows, epochs and local ceilings using the same token on successive calls,
as well as caller/audience/seed/signature rejection. Complete verifier adoption
does not require a new library endpoint or wallet service. Stable session storage
and certification preparation retain their separately described host contracts.

The 2026-10-10 read-only review of Canic at base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6` finds both libraries resolved to 0.2.7
in its dirty root lock. Its installation adapter calls the standalone library
verifier, while the complete-token adapter still uses local callbacks and the
positive cache. This identifies the next consumer boundary, not qualification
of its uncommitted graph. [Canic #58](https://github.com/dragginzgame/canic/issues/58)
owns that repair; 0.2.8 adds no required Rust API for it.

The complete adapter must assemble these fields before calling the library:

| Library input | Protected Canic source or decision |
| --- | --- |
| `caller`, `now_ns` | Actual authenticated ingress caller and host clock at this invocation |
| `audience`, `role` | Current fleet binding and runtime role, through checked protocol projections |
| `root_key` | Existing independently enrolled chain-key policy, retaining network/key-name admission and all live floors/windows |
| `ic_root_public_key_raw` | Network-selected BLS trust anchor, after existing build-network validation |
| `allowed_scopes` | Sorted, unique protected ceiling for this local role; the existing local application configuration supplies it where enabled |
| `required_scopes` | The operation's endpoint requirements, independently of the ceiling |
| `limits` | Existing host TTL/skew policy plus explicit certificate age, material, signature and witness bounds |

Token-only verification without local session configuration needs an explicit
protected ceiling owned by Canic; neither submitted grants nor endpoint-required
scopes define it. The library rejects any scope in the selected local grant
outside this ceiling, even when the operation only requires a smaller subset.
It does not silently intersect or normalize the grant. The existing issuer
proof callback has no certificate-age or signature-size inputs, so those finite
bounds must be selected deliberately rather than inferred from received proof
bytes. Consumer limits are not new IC Auth defaults.

Project the complete token once and pass it to `token::verify_token`. After
success, use `VerifiedToken` accessors for authenticated claims, selected role,
local scopes, claims hash and the policy-capped exclusive expiry. Resolve
Canic-owned resource permissions and session/replay admission separately.
Keep typed error/metric mapping at the adapter and qualify the real entry point
with identical signed token bytes while changing the protected ceiling, root
window/epoch, IC trust anchor and certificate age. Retiring the cache together
with local proof callbacks leaves one verification owner; keeping a cap on the
old cache alone does not enforce these live inputs.

The pre-token installation caller has a separate boundary:
`ops/auth/delegation/active.rs::install_active_delegation_proof` checks an issuer's
`DelegationProof` before a completed token exists. Released 0.2.2 adds
`token::verify_delegation_proof` with `DelegationProofVerificationContext` and
`DelegationProofVerificationLimits`; see the
[standalone contract](tokens.md#standalone-root-delegation-verification).
Supply the actual issuer from protected runtime/configuration, enrolled root
policy, host clock and finite bounds. The result is proof evidence, not issuance
approval. Its certificate hash includes checked issue metadata that is absent
from the signed root leaf; complete token verification still binds that exact
hash through signed issuer claims.

[IC Auth #10](https://github.com/dragginzgame/ic-auth/issues/10) records completed
library qualification; consumer acceptance stays under Canic #491. Keep Canic's installation authority,
protected key/network/audience/grant policy, renewal, storage and live authority
checks. Preserve its strict installation not-before rule even if configured
cryptographic future skew is nonzero. Verify before storage mutation and use
the policy-capped deadline when recording proof freshness. Retire its duplicate
pre-token root verifier after consumer acceptance. This boundary does not block
adoption of the existing complete token verifier. No Canic source changes or host acceptance are implied.
