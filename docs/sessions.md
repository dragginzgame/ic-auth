# Local application sessions

The pending 0.6 hard cut provides one portable application session engine in
`ic_auth::session`. Enable `sessions` for the engine and bounded volatile store;
select `portable-signature-verification` for Ed25519 or `token-verification` for
complete certified portable proofs. The engine itself does not require either
cryptographic backend. It does not depend on Canic, Toko, CDK or stable memory.

## Admission and ordinary calls

`establish_session` takes a host store, retained login revocation ledger, shared
work budget, canonical claims, checked `SessionRequest`, protected
`SessionContext` and `SessionPolicy`, plus a complete verification callback.
For a `PortableCredential` envelope, use `portable_credential::verify_credential`
in that callback with the supplied budget and protected `PortableVerificationContext`.
The dispatcher rejects proof/enrollment mechanism mismatches and reuses the complete
Ed25519/certified verifiers. Pass the same protected claims context to the engine
and dispatcher. The lower-level verifiers remain available to proof-only callers. The result is
sealed `VerifiedPortableClaims`; decoding claims alone cannot admit a session.
The engine rechecks the result's claims and verification time before committing.

Cheap binding, revocation, request, replay and capacity refusals precede signature
verification. Fresh admission spends receiver-wide verification credit, including
invalid signatures and later storage failures. Never recreate the budget per
request. Exact retries return the unchanged live session without verification,
even after the admission proof expires. Changing the requested scope or lifetime
on a retry is a replay conflict. Renewal uses a fresh credential within the same
immutable issuer/account and login-start/deadline binding.

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
all staged session/replay/index changes or leave them unchanged. The engine wraps
the final transaction in the ledger reservation callback, so a failed backend
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

`MemorySessionStore` provides bounded indexed storage, per-caller replay limits,
encoded-byte accounting and explicit pruning of at most 128 expired records per
call. Logout preserves replay records; authority generation advancement invalidates
sessions without clearing replay evidence. Capacity includes expired records
until pruning.

Enable `session-restore` only for hosts restoring serialized records. It exposes
checked `Session::decode`; volatile consumers do not need that decoder. Cargo
feature unification still applies, so inspect each deployed canister's selected
graph rather than treating workspace builds as size measurements.

The portable record begins with `IC-AUTH-PORTABLE-SESSION\0` before its CBOR body.
Old token-specific records are rejected, with no compatibility reader. The 0.6
consumer transition must explicitly retire old sessions and coordinate retained
replay/revocation state before serving. Decoding alone does not authorize a record:
current authority and its matching retained login reservation are required.

After volatile reset, advance login/proof floors using trusted boot time and the
qualified clock-skew bound, discard lost-state-dependent sessions and authenticate
fresh enrollment before readiness. This forces fresh browser login after receiver
upgrade. Durable hosts must retain sessions, replay, revocation and protected
reset/authority state atomically. A production durable host adapter is not yet
implemented; the ledger supplied here is volatile.

The durable adapter must use `ic-memory` for host-composed allocation ownership,
with the host supplying the combined declarations and performing bootstrap before
stores open. The reviewed IC Memory contract at revision
`752263e22520e5009471b7040e2db0f14e0142b4` supplies allocation admission and
re-exports stable structures; it does not supply Auth's record schema or transaction.
That source declares Rust 1.91, above Auth core's qualified 1.88 floor; qualify
the adapter's compiler independently rather than raising the pure libraries' floor.
Keep this dependency in a separately selected durable adapter, outside the pure
engine and volatile receiver graph. It must not persist the local issuer key.

Adding a stable `SessionStore` alone is insufficient: the current engine takes a
concrete `MemoryLoginRevocations`, and its callback commits session/replay state
before installing the heap reservation or tombstone. A durable implementation must
first provide one commit boundary covering both sets of records and protected
reset/authority metadata. Returned failures must leave them unchanged while
retaining charged work budgets. Normal upgrades must restore matching reservations
as well as sessions; incomplete recovery must enforce fresh-login reset rather
than invent missing reservations. [Auth #58](https://github.com/dragginzgame/ic-auth/issues/58)
owns this integration and its failure/upgrade qualification. No `ic-memory`
dependency or persistent backend is advertised as implemented yet.

## Evidence and remaining adoption

`make test-sessions` covers both proof mechanisms, independent devices/receivers,
renewal, exact retries, logout-before-admission, authority invalidation, commit
failures and checked record restoration. `make test-qualification` exercises the
volatile receiver in actual Wasm, upgrades and signed ingress, plus portable
certified admission using real IC certificates. It does not qualify production
wallet issuance, browser fan-out, durable persistence or cross-subnet latency.

Canic owns enrollment, endpoint/configuration adapters and fleet integration;
[Canic #491](https://github.com/dragginzgame/canic/issues/491) tracks adoption.
The existing standalone token verifier remains a proof primitive, not an
alternative token-per-call application session flow.
