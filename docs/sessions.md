# Local application sessions

The working tree adds `ic_auth::session` behind `sessions`, enabling the existing
token verifier. It is not in published `0.1.4`. Default, signature-only and
token-only graphs remain selectable without the session engine. No feature
imports Canic, Toko, Solana, native Host tooling, CDK or stable storage.

`establish_session(store, token, request, token_context, policy)` verifies both
token proofs before new admission and performs session replacement and replay
consumption in one synchronous host transaction. `authorize_session` reads the
current session and checks the operation's required scopes with protected live
context. Neither operation reads an IC clock/caller or publishes certification.
These sessions are application credentials, not IC ingress delegations; resource
and business permission checks remain application-owned.

## Protected context and lifetime

The host supplies the actual caller and current audience, role, allowed scopes,
root-key enrollment and time through `TokenVerificationContext`. Its required
scopes apply to proof verification during admission and to later session
authorization. The narrowing request need not grant every admission endpoint
scope; later endpoints check their own requirements.

`SessionPolicy` supplies the protected generation, capability switch, current
subject eligibility, default/maximum TTL and proof lifetime bound. Store generation
must match policy generation. The host must atomically advance its persisted
generation with authority changes, including disable/re-enable, issuer approval
or revocation, IC trust-key changes and admission-policy changes that invalidate
retained authority. A client must never supply this policy. Generation exhaustion
fails without mutation; wrapping or resetting it would resurrect stale authority.

Every authorization and exact retry checks caller/subject, audience, role,
generation, current scope ceiling, root key identity, key/proof/registry floors
and protected key validity. A changed SEC1 key, key name, path, root principal or
version invalidates the retained record. Narrowing the root acceptance window
below the stored session deadline invalidates it immediately. Original expiry
cannot be extended by a later enrollment deadline. Hosts still own propagation of protected revocations;
the library does not poll another canister or infer issuer eligibility.

New proof admission requires strict current proof start and a lifetime within
the protected bound, at most 60 seconds. It does not use the token verifier's
future-start skew allowance to create a session early. TTL selection is positive,
at most the protected maximum and at most 1800 seconds. Nanosecond expiry addition
rejects overflow and is capped by the root key's acceptance deadline. Expiry is
exclusive. An admitted session may outlive its original token, issuer certificate
or batch: it is a local admission with its own lifetime and live policy checks.
Root grant revocation must update local generation or accepted authority floors;
proof lifetime alone does not revoke an admitted session.

## Narrowing, exact retry and replay

`SessionRequest::new` accepts 1–16 distinct scopes of at most 64 bytes each,
with at most 1024 aggregate bytes, using the canonical scope grammar. It sorts
before hashing and rejects duplicates. Every requested scope must be in the
verified local grant. The request hash preserves Canic's
`canic-application-session-request-v1` domain, u64 big-endian counts/lengths and
optional TTL in seconds. An omitted TTL differs from an explicit default TTL.
No signed token bytes or existing token models change.
Hosts bound outer token/request decoding before constructing these vectors.

One store belongs to one protected local application authority namespace. There
is one current session per exact caller. The proof-consumption fingerprint is
the canonical claims hash, including nonce and certificate hash, independently
of replaceable signature transport. State resolution occurs inside the same
transaction as proof verification and admission:

1. A matching current session and request hash returns `ExactRetry` only after
   all live session checks. Its expiry and stored bytes are unchanged, even
   after the proof expires. Altering proof transport does not create new authority.
2. A conflicting request or any other retained consumption of that fingerprint
   is rejected. Logout and replacement retain replay tombstones. Generation
   invalidation does not erase them.
3. An unused proof must pass complete token verification, strict proof eligibility,
   requested-scope narrowing, positive TTL and capacity checks before staging.

Replay tombstones remain until token expiry, when new proof admission is no
longer possible. An active session can still resolve its exact retry after the
tombstone is pruned. Clearing the session removes that retry opportunity. After
both records are gone, the expired proof still cannot create a new session.
Physical expired records count toward quotas until explicit cleanup; an admission
failure performs no incidental cleanup.
Replay cleanup requires a trusted clock that does not regress behind a completed
cleanup time. A host must not restore older replay/generation state while serving
new requests. Re-run `authorize_session` for every operation; a cached successful
result is not current authority.

## Transaction and storage ownership

`SessionStore::transaction` runs a closure over `SessionTransaction`. Record
reads and occupancy refer to one isolated pre-commit snapshot. `stage` admits
one touched session/replay pair. On any closure or commit error, the backend
must preserve prior records, expiry indexes, subject counts, generation and byte
accounting. No await, reentrancy or authority mutation may split these checks
from commit. Implementations must also reject mismatched pairs, duplicate
fingerprints, stale generation, capacity and encoded-byte overflow before writes.

`SessionLimits` allows lower host limits within 2048 sessions, 4096 replay
records, 256 replay records per subject and 8 MiB of encoded record bytes. A
replacement consumes a new replay slot but reuses the caller's session slot.
Encoded bytes are record payload accounting, not measured allocator/index heap
usage. Hosts must bound their physical backend and indexes independently.

`MemorySessionStore` is a volatile reference backend for native and Wasm. It
stages only the touched pair and applies it after a successful closure; it does
not clone or serialize the entire store. Ordered expiry indexes let `prune`
remove at most `min(budget,128)` expired records without a full-store scan.
`clear(actual_caller)` removes that caller's session and retains replay history.
There are no global memory regions, timers or implicit cleanup effects.

`Session::encode` and `Session::decode` handle one current CBOR record, bounded
to 2048 bytes, with structural identity/window/scope checks and rejection of
unknown fields. Decode is for trusted host storage, never a client credential
or proof-authentication substitute. A backend restores replay records with their
matching fingerprint, subject, generation and removal time and rebuilds indexes
and counters before exposure. Validate retained record sets and physical budgets
at the owning backend's restore boundary.

No stable backend or canister lifecycle adapter is implemented or qualified.
An embedded host owns stable-memory IDs, atomic durable writes, rollback/trap
behavior, protected configuration and generation persistence, restore validation
and maintenance scheduling. Volatile storage must not be substituted for durable
replay state in a deployed service. Canic's existing stable representation is
not read by this engine; adoption must coordinate its retained-state disposition
and remove the superseded owner in the separately authorized consumer change.

## Evidence

`make test-sessions` shares real secp256k1/BLS fixtures with token verification.
It covers proof rejection, exact retries, post-staging atomic failure, authority
and caller/scope changes, replacement/logout, capacity and byte failure, strict
starts, expiry overflow and bounded cleanup. The request hash has an independently
computed fixed vector. Clippy, Wasm compilation, packaging and graph checks
include the feature. These checks do not establish stable host adoption, upgrade
or ingress-delegation behavior; PocketIC remains required for those boundaries.
The [accepted design](design/extraction.md) and
[Canic #491](https://github.com/dragginzgame/canic/issues/491) own the larger extraction.
