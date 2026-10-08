# IC canister-signature preparation and retrieval

The working tree adds optional `canister-signature-preparation` and
`ic_auth::signature_store` for the pending `0.1.6` release. This capability is
independent of `canister-signature-verification`, token verification and sessions.
Encoding-only consumers select neither signature feature.

`SignatureStore` is a bounded, volatile reference store. Preparing a leaf is a
signing decision: the host must authorize the exact payload, seed and domain
before calling `prepare`. The library does not decide role eligibility, issuer
approval, endpoint access or which caller may retrieve a pending proof.
Canic's operation/caller records and application policy remain with its adapter.

## Protected inputs and bounds

Construct the store with the host's actual canister principal and positive
`SignatureLimits` from protected configuration. Anonymous signers are rejected.
Neither identity nor limits may come from an untrusted request.

| Input or operation | Bound |
| --- | --- |
| Physical retained signatures | Host limit, at most 4096; includes expired records awaiting cleanup |
| Key seed | At most 32 bytes; covers both existing Canic seeds and 32-byte identity seeds |
| Signature domain | At most 255 bytes |
| Raw payload | Positive host byte limit, at most 64 KiB; checked before hashing |
| Retrieval retention | Positive duration, at most 60 seconds, with checked deadline addition |
| Query certificate | Positive host byte limit, at most 64 KiB; checked before decoding |
| Returned signature CBOR | Positive host byte limit, at most 128 KiB |
| Host composition path | At most 16 sibling branches, innermost first |
| Explicit expiry cleanup | At most 128 removed records per call |

These are record/input/output bounds, not a measurement of allocator overhead.
The certified tree and both retention indexes are bounded by the physical record
limit. Certificate decoding retains Serde CBOR's recursion limit. Serialization
occurs only after certificate size, witness record count and composition depth
are bounded; its output limit is checked before returning the proof.

`SignatureInputs` supplies the raw message, domain and seed. DFINITY's upstream
hashing primitives construct the exact domain-separated message hash:
`SHA256(one-byte domain length || domain || raw message)`. For Canic, the raw
message is its canonical 32-byte payload hash. Passing an already prefixed
message would sign different bytes. Both existing Canic seeds/domains are
preserved: `canic-root-role-attestation` and `canic-issuer-delegated-token`.
Seeds beyond 32 bytes are outside this preparation API, even where the separate
verification API accepts their short-form DER representation.

## Retention and mutation

`prepare(inputs, now_ns, retention_ns)` inserts one empty leaf under
`/sig/SHA256(seed)/SHA256(domain-separated message)`. It returns the exclusive
retrieval deadline and the digest of the labeled `/sig` contribution. A live
duplicate preserves the original deadline and leaves the current root unchanged;
it does not extend retention.
Repreparing an expired key replaces its expiry index entry without using another
record. Each key has exactly one retention entry; repeated requests cannot build
an unbounded expiry queue or leave a stale expiry that deletes a refreshed key.

There is no implicit cleanup. A new key fails at capacity, even if unrelated
records have expired. `prune(now_ns, budget)` removes the earliest expired
records, including those at the exact deadline, up to its explicit budget.
Zero is valid; budgets above 128 fail. `remove(inputs, now_ns)` withdraws one
pending key. Removing the final message under a seed removes the empty subtree.
Neither operation scans or clones the whole store.

All fallible preparation checks precede mutation. Rejected inputs, clock,
overflow, retention and capacity leave state unchanged. The store remembers the
last accepted mutation time and rejects earlier clocks on mutations and reads.
Query reads do not retain a later clock: the host must obtain a trusted monotonic
clock on every operation and must not regress it or restore an older live state.
The host owns any enclosing transaction and other application state.

Retention limits retrieval, not cryptographic proof validity. Deleting or pruning
a leaf cannot revoke a certificate/signature already returned to a client.
Receiver freshness, signed payload validity and current protected authority still
apply through the separate [signature](signatures.md) and [token](tokens.md)
verifiers. A prepared message is neither a resource grant nor, by itself, an IC
ingress delegation.

## One host-owned certification root

`root_hash()` includes the `sig` label; it is a contribution, not necessarily the
canister's final root. The host reserves `/sig` for this store, keeps other labels
ordered, composes all owners' digests and publishes the result during its update
transaction. For example, with assets before `sig` and status after it, the host
publishes `fork_hash(fork_hash(assets_root, signature_root), status_root)`.
Its matching query composition is
`[SignatureSibling::Left(assets_root), SignatureSibling::Right(status_root)]`.
Each sibling is represented by a pruned hash; the library cannot overwrite that
owner's certified state. This matches the IC's
[canister-signature tree contract](https://docs.internetcomputer.org/references/ic-interface-spec/#canister-signatures).

`retrieve(inputs, now_ns, certificate, siblings)` reconstructs a current witness
only for a retained, unexpired key. The certificate must come from the actual
host query runtime, never from the client. Its bounded decode must contain
`/canister/actual_canister_id/certified_data` equal to the composed witness digest.
Missing, malformed, mismatched-canister or obsolete-root certificates fail.
Changing a prepared leaf, removing one or pruning requires the host to republish
the composed root before using a matching new query certificate. Repreparation
of the same leaf can leave the digest unchanged; retention is not in the tree.

The result is `IcCanisterSignatureProofV1`, with the existing self-describing CBOR
certificate/tree envelope and DFINITY's DER canister key. Retrieval checks
structure and root coherence; it does not authenticate a client-supplied BLS
chain or enforce certificate freshness. Consumers must still use the full
verifier with their protected network, signer, seed and clock policy.

The store never calls an IC clock, acquires a data certificate, publishes a
global certification root or allocates a stable-memory region. Native `ic-host-*`
file utilities remain outside this pure library graph. DFINITY owns tree
hashing/witness generation, key encoding and signature verification.

## Lifecycle and qualification

Reset/upgrade handling belongs to the host. This reference store has no durable
codec: replacing it with an empty store loses pending retrievals. The host must
recompose and publish its root before queries, preserve other certified owners,
and define pending-proof behavior independently of durable user identities,
controller keys, recovery and session authority. There is no imported
reinstall-only lifecycle rule or qualified stable restoration adapter.

`make test-signature-store` covers exact Canic paths/keys, real BLS verification
with multiple host branches, changed messages, stale certificates, inclusive
limits, retention, duplicate preparation, bounded cleanup and failure atomicity.
Default and preparation-only graphs compile for Wasm and pass dependency guards;
all-feature tests also exercise existing token/session capabilities.
These are pure mechanism checks. Real canister publication, query timing,
upgrade restoration and IC ingress delegation still require a host/PocketIC
qualification under the [accepted design](design/extraction.md) and
[Canic #491](https://github.com/dragginzgame/canic/issues/491). Canic's active
implementation is unchanged; adoption requires a separately authorized change.
