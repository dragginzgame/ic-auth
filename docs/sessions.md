# Local application sessions

The portable contract provides one portable application session engine in
`ic_auth::session`. Enable `sessions` for the engine and bounded volatile store;
select `portable-signature-verification` for Ed25519 or `certified-proof-verification` for
complete certified portable proofs. The engine itself does not require either
cryptographic backend. It does not depend on Canic, Toko, CDK or stable memory.

## Admission and ordinary calls

`establish_session` takes a host store, retained login revocation ledger, shared
work budget, canonical claims, checked `SessionRequest`, protected
`SessionContext` and `SessionPolicy`, plus a complete verification callback.
For a `PortableCredential` envelope, the callback receives the supplied budget
and an owned `DecodedPortableClaims`. Consume it with the protected verifier:

```rust,ignore
|budget, decoded| {
    Ok(decoded.verify(budget, &credential.proof, &verification_context)?)
}
```

The callback shares the
engine's canonical decode and SHA-256 digest with complete Ed25519/certified
verification, without cloning the claims. The opaque value is not authenticated;
its verifier rejects proof/enrollment mechanism mismatches. Pass the same protected
claims context to the engine and `PortableVerificationContext`. The sealed
`VerifiedPortableClaims` result must match the original digest and current time
before admission can commit. Budget charges survive later refusals.

`portable_credential::verify_credential` and the mechanism-specific byte verifiers
remain independently validating entry points for standalone proof callers. Do not
use them to decode the credential again inside the session callback.

Cheap binding, revocation, request, replay and capacity refusals precede signature
verification. Fresh admission spends receiver-wide verification credit, including
invalid signatures and later storage failures. Never recreate the budget per
request. Exact retries return the unchanged live session without verification,
even after the admission proof expires. Changing the requested scope or lifetime
on a retry is a replay conflict. Renewal uses a fresh credential within the same
immutable issuer/account and login-start/deadline binding.

Admission validates the session record's encoded size without allocating bytes
that would be discarded. The store independently enforces its transactional
capacity checks; persistence explicitly encodes only when bytes are needed.
Record encoding writes CBOR directly after the existing domain prefix, preserving
the same bytes, size bounds and error precedence.

A session records the browser `caller`, account `(issuer, subject)`, audience,
mechanism/key binding, role/scopes, authority generation, login window, lease and
proof/request fingerprints. Two devices belonging to one account use distinct
fresh browser principals and independent sessions. Quotas count browser callers,
not account subjects. Ingress delegation preserving one principal is not a new
login identity.

`authorize_session` uses the runtime caller and protected current authority,
generation, eligibility and required scopes. It checks the retained login
reservation, revocation and lease without cryptography or network calls.
Authorization does not extend a lease. Proof expiry limits admission; session
expiry is capped by the requested TTL, login deadline and key/enrollment expiry.
Refreshing authority never extends an existing lease. Applications separately
check resource ownership and decide which effects require reauthorization after
an await.

## Atomicity and logout

The host MUST serialize the entire engine call with authority updates and ledger
operations, without await or reentrancy. `SessionStore::transaction` must commit
all staged session/replay/index and proposed login-record changes or leave them
unchanged. Its new `login_change` argument is `None` for reads and
`Some(&LoginRevocationChange)` for admission/logout. Ledger commit callbacks now
receive that read-only proposed record, including caller, domain, retention,
pool and immutable account/login binding. Durable hosts must persist it in the
same transaction; it is not a reusable credential or authority snapshot.
The volatile store rejects missing changes, mismatched callers/domains and
reservation-versus-revocation operation mismatches before modifying records.
Durable adapters can reuse this staging through
`MemorySessionStore::transaction_with_commit`. After all validation, a write-only
callback receives `SessionCommit`: the proposed login record, the touched
session/replay pair for admission (or no pair for logout), and exact post-commit
session/replay counts and encoded-byte total. Reads and rejected operations do not
invoke persistence. A callback error leaves heap records, indexes and accounting
unchanged. Success must mean the durable transaction has completed; the reference
store then publishes the already-validated heap changes. No whole-store clone or
checkpoint is required for this callback. Durable atomicity, metadata generation,
clock/budget persistence, pruning and authority changes remain host obligations;
this hook does not itself supply a stable backend.
Maintenance uses the same ordering: session-store `prune_with_commit` supplies
at most 128 expired session/replay keys and exact resulting counts/bytes before
deletion. Ledger `prune_with_commit` supplies expired callers and resulting
admitted/unknown pool counts. Empty work skips persistence; returned failure
preserves records/indexes/accounting. The ledger still retains its trusted clock
observation, including on failure or empty work, so hosts must persist protected
clock state independently. `advance_generation_with_commit` commits the proposed
generation together with host authority before publishing it in memory; overflow
refuses before calling persistence. These callbacks require the same serialized,
atomic host boundary as admission/logout. Volatile methods delegate to these
implementations with in-memory commits.
The engine wraps the final transaction in the ledger callback, so a failed backend
commit cannot leave an admitted reservation or acknowledge a failed logout.
Verification and unknown-logout budget charges survive returned storage errors;
hosts must not trap and roll back those charges.

`logout_session` returns the shared `LogoutResult`: `Revoked { deny_until_ns }`
only after commit, `InvalidCaller`, or typed `Unavailable` (capacity, storage or
clock). Hosts return `Unavailable(ResetPending)` before touching unavailable
startup state. The empty `LogoutRequest` accepts no client-selected identity or
proof; hosts pass the authenticated runtime caller. Transport failures remain
unknown, never successful revocation. The qualification endpoint and generated
browser bindings use this same contract.

`logout_session` authenticates the actual caller, removes its local session and
retains replay evidence plus a login tombstone. It works with expired proofs,
expired enrollment and disabled admission, provided the host is initialized.
Admitted logins have reserved revocation capacity; absent logins use the separate
bounded pool and receiver-wide unknown-logout budget. Exhaustion returns failure,
never false success or eviction. Retrying an existing tombstone retains its fixed
deadline. See [the protocol](design/0.6-auth-protocol.md) for retention, browser
retry and cross-subnet logout requirements.

Each receiver has independent sessions and revocation state. The same permitted
credential can establish on A, B and C, but admission on A does not admit on B.
Ordinary calls never implicitly create sessions. Browser fan-out is necessary
for remote logout without inter-canister calls; immediate global revocation is
not guaranteed. Issuer logout and absolute login deadlines remain essential.

## Storage and reset

Use `Session::issuer`, `Session::mechanism` and `Session::key_binding_id` to select
the exact current protected enrollment entry for a retained or restored session.
One issuer may have several approved keys; issuer identity alone is insufficient.
These getters describe retained admission identity, not current authorization.
Build the context from protected enrollment and call `authorize_session` to
enforce current validity, grants, generation and logout state before serving it.

`MemorySessionStore` provides bounded indexed storage, per-caller replay limits,
encoded-byte accounting and explicit pruning of at most 128 expired records per
call. Logout preserves replay records; authority generation advancement invalidates
sessions without clearing replay evidence. Capacity includes expired records
until pruning.

Enable `session-restore` only for hosts restoring serialized records. It exposes
checked `Session::decode`, `Replay::decode`, `LoginRevocationChange::decode` and
ledger/store `restore_records`; volatile consumers do not need these decoders. Cargo
feature unification still applies, so inspect each deployed canister's selected
graph rather than treating workspace builds as size measurements.

The portable record begins with `IC-AUTH-PORTABLE-SESSION\0` before its CBOR body.
Old token-specific records are rejected, with no compatibility reader. Consumers
must explicitly retire incompatible sessions and coordinate retained
replay/revocation state before serving. Decoding alone does not authorize a record:
current authority and its matching retained login reservation are required.

Replay storage uses its own `IC-AUTH-PORTABLE-REPLAY\0` domain and 512-byte bound;
its encoded bytes count toward store capacity. Raw replay serialization is not
supported; no old-format reader is supplied.
`MemorySessionStore::session_records` and `replay_records` stream records under a
host-serialized checkpoint boundary. Preserve both physical counts, encoded byte
total and generation in the same atomic checkpoint as all records. Restoring into
a fresh store rejects duplicate identities, missing/excess entries, future
generations, inconsistent retained session/replay pairs and capacity violations.
It rebuilds expiry indexes and per-caller replay counts without extending leases.
Expired and old-generation records still occupy capacity until pruning. Replay
records survive logout, and an expired replay may have been pruned before its
session expires; restoration supports both states. Snapshot completeness and
rollback protection remain host obligations, including when counts match.

After volatile reset, advance login/proof floors using trusted boot time and the
qualified clock-skew bound, discard lost-state-dependent sessions and authenticate
fresh enrollment before readiness. This forces fresh browser login after receiver
upgrade. Durable hosts must retain sessions, replay, revocation and protected
reset/authority state atomically. A production durable host adapter is not yet
implemented; the ledger supplied here is volatile.

The durable adapter must use `ic-memory` for host-composed allocation ownership,
with the host supplying the combined declarations and performing bootstrap before
stores open. The selected IC Memory 0.48 supplies allocation admission and narrow
memory types; it does not supply Auth's record schema or transaction. The internal
checkpoint uses `ic_memory::DefaultMemoryImpl` and `testing::FixtureCell`, not the
retired `ic_memory::ic_stable_structures` re-export. All workspace packages share
the qualified Rust 1.91 minimum; see [minimum compiler qualification](msrv.md).
Keep this dependency in a separately selected durable adapter, outside the pure
engine and volatile receiver graph. It must not persist the local issuer key.

Adding a stable `SessionStore` alone is insufficient: the current engine takes a
concrete `MemoryLoginRevocations`. Its callback now supplies the exact proposed
reservation/tombstone to the store transaction before updating heap state, enabling
a durable host to persist both together. `LoginRevocationChange::encode` (under
`sessions`) provides a domain-separated record capped at 1024 bytes. Its checked
decoder is trusted-storage-only, never a client admission path. `restore_records`
consumes a freshly constructed ledger and the complete decoded record stream;
failure returns no partial ledger. It rejects wrong domains, duplicate callers,
invalid retention, excess/missing physical pool counts and clock rollback.
`records()` streams the complete physical ledger for a host-serialized checkpoint.
Expired records count until explicit pruning. Hosts persist `observed_at_ns()`
and both occupancy counts in the same atomic generation as the records, including
clock observations retained after refused operations. A successful restore keeps
the new trusted clock and never extends a reservation or tombstone.

These checks do not authenticate storage or detect a coherently rolled-back
snapshot. The host must protect the generation, configuration and completeness;
changing the qualified skew or recovering incomplete storage requires the reset
contract. The `ic-memory` adapter remains unimplemented. A durable implementation must also
cover protected reset/authority metadata. Returned failures leave state unchanged while
retaining charged work budgets. Normal upgrades must restore matching reservations
as well as sessions; incomplete recovery must enforce fresh-login reset rather
than invent missing reservations. [Auth #58](https://github.com/dragginzgame/ic-auth/issues/58)
owns this integration and its failure/upgrade qualification. The qualification host selects `ic-memory`; none of the three public libraries depends on it.
A production per-operation persistent backend remains unimplemented.

## Custom host transaction contract

The portable engine is a protocol change, not just a renamed transaction method.
Canic's reported integration stays in [Canic #491](https://github.com/dragginzgame/canic/issues/491).
Hosts can reuse `MemorySessionStore::transaction_with_commit` to avoid copying its
staging/index rules. A custom store with its own indexes must enforce the same
contract before any persistent effects:

| Proposed operation | Required login change and checks |
| --- | --- |
| Read only | `None`; reject staged writes/removals |
| Admission/replacement | `Some(change)` with `!change.revoked()`; call `session.check_login_change(change)`; check session/replay caller, fingerprint and generation, replay uniqueness and physical capacities |
| Session removal | `Some(change)` with `change.revoked()` and `change.caller() == removed_caller`; if a session exists, call its `check_login_change(change)`; retain all replay records |
| Missing session removal | Still commit the proposed tombstone; absence is not evidence that a delayed admission cannot arrive |

`Session::check_login_change` exposes the existing association check without
revealing private storage fields. It checks caller/domain for both operations,
and issuer/subject/login-window/retention for admission. A revoked record may
have no login binding, so that an authenticated proofless logout can remove an
older session. Passing this check never permits admission with a tombstone or
replaces current authority, replay, capacity or atomicity checks. No submitted
record is a credential. Persist the login change with the session/replay/index
updates; return success only after the whole commit succeeds.

The rest of the host cut must be propagated together:

- Replace old `DelegatedToken` session inputs with `PortableCredential` and a
  protected `SessionContext`/`PortableVerificationContext`. Call the complete
  `decoded.verify(budget, &credential.proof, &verification_context)` inside
  `establish_session`'s two-argument callback using its supplied owned claims,
  shared budget and the same protected authority. Do not wrap an old token as
  new signed claims or use the standalone token verifier to authorize a session.
- Retain one login ledger and admission/unknown-logout budgets across requests.
  Pass the ledger to establishment, authorization and logout. Fresh allocation
  per call would lose denials and replenish abuse credit.
- Use `Replay::caller()`, `max_replays_per_caller` and `replays_for_caller` for
  login accounting. Application account identity remains `(issuer, subject)`;
  replacing subject indexes mechanically is not an account migration.
- Durable hosts select `session-restore` and use the bounded session/replay/login
  codecs. Remove raw replay deserialization and explicitly retire incompatible
  retained layouts. Restore complete matching metadata and records, then obtain
  fresh receiver-bound enrollment before serving. Volatile hosts enforce reset
  floors instead; neither profile can fabricate missing reservations.
- Qualify failed commits, exact retries, same-account separate browser callers,
  session-only logout with replay retention, unknown-login logout and upgrade
  restoration/reset in the actual host. Auth's fixture is reference evidence,
  not production Canic adoption or a completed per-operation durable backend.

## Executable host integration references

Use the qualification receiver's
[`establish_portable_session` and `portable_session_read`](../apps/qualification/canister/src/sessions.rs)
as the endpoint wiring reference: runtime caller/time, retained ledger and budgets,
protected enrollment, and complete verification inside the admission callback.
Its controller-installed configuration is a test boundary; production Canic must
supply its authenticated Root enrollment and its own atomic durable store.

The certified-path
[`certified_proof_uses_the_shared_session_lifecycle`](../crates/ic-auth/tests/portable_certified/mod.rs)
regression uses the same public credential envelope and decoded-claims verifier. One signed proof is
admitted independently by two receivers; an exact retry after proof expiry retains
the original lease. Logout blocks re-admission before cryptography on that receiver
while the other receiver remains authorized. This is a native protocol regression,
not deployed cross-subnet or Canic endpoint qualification. Run `make test-sessions`.

## Evidence and remaining adoption

`make test-sessions` covers both proof mechanisms, independent devices/receivers,
renewal, exact retries, logout-before-admission, authority invalidation, commit
failures and checked record restoration. `make test-qualification` exercises the
volatile receiver in actual Wasm, upgrades and signed ingress, plus portable
certified admission using real IC certificates. An opt-in fixture checkpoint also
persists sessions, replay records, login reservations/tombstones, counts, floors
and authority generation through real upgrades. It restores closed, requires a
new controller confirmation of the unchanged live authority, preserves lease and
denial deadlines, and retains disabled authority across repeated upgrades. Work
budgets restart empty, conservatively discarding unused credit.

That checkpoint uses published `ic-memory` 0.40.0 with one host-bootstrapped
namespace and a bounded 128 KiB checkpoint allocation. The host owns the complete
declaration set, opens only admitted memory and rejects missing/malformed headers
before decoding. Synchronous upgrade writes use IC message rollback on traps.
This replaces raw CDK stable-save; older unpublished fixture installations must
be reinstalled because their stable-memory layout is incompatible. Its controller
confirmation models protected host input; it is not production Root enrollment.
The fixture format is internal and has no compatibility reader for older test
installations. The combined fixture Wasm includes storage and must not be used as a minimal
volatile consumer size measurement. This evidence does not qualify the production ic-memory adapter,
per-operation stable transactions, backup recovery, wallet issuance, browser
fan-out or cross-subnet latency.

Canic owns enrollment, endpoint/configuration adapters and fleet integration;
[Canic #491](https://github.com/dragginzgame/canic/issues/491) tracks adoption.
The existing standalone token verifier remains a proof primitive, not an
alternative token-per-call application session flow.
