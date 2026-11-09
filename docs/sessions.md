# Local application sessions

The portable contract provides one portable application session engine in
`ic_auth::session`. Enable `sessions` for the engine and bounded volatile store;
select `portable-signature-verification` for Ed25519 or `certified-proof-verification` for
complete certified portable proofs. Fresh admission is available only with a complete
verifier selected; session-only builds retain authorization, logout and storage. It does not depend on Canic, Toko, CDK or stable memory.

## Host-selected capacity

`SessionLimits::new` accepts positive host-selected session, replay, per-caller
replay and encoded-byte budgets. It does not impose a deployment-wide session
ceiling or allocate backing. Admission and restoration still enforce the selected
budgets; individual record codec limits are unchanged. A host may select 100,000
sessions with proportional retention budgets, but must qualify its actual store,
heap indexes, reconstruction, cleanup and instruction costs. These values are
protected configuration, never token or caller input.

## Admission and ordinary calls

`establish_session` takes a host store, retained login ledger, shared work budget,
the complete `PortableCredential`, a checked `SessionRequest`,
`SessionAdmissionContext` and `SessionPolicy`. Admission owns the canonical
claims decode, digest and complete verification. There is no verification callback
or public decoded-admission value that a host can substitute.

Auth 0.24 consumes the checked `SessionRequest`. The fresh session takes
ownership of its scope strings; exact retries still use the same frozen request
hash, including absent versus explicit TTL. Retain raw request inputs or explicitly
clone a checked request if it is needed afterward. Stored and signed formats are
unchanged.

The Ed25519 verification context now borrows `PreparedIssuerKey` from
`ic_auth::portable_signature`. Admit protected issuer bytes once, retain the key
with enrollment and rebuild after decoding configuration. Its binding hash uses
the original bytes; its immutable Platform key uses the same parser and weak-key
checks as raw verification. This proves shape, not proof validity, enrollment or
TEE assurance. Every operation keeps live authority/budget checks, strict
signature verification and canonical-scalar admission. No verification result is
cached. The standalone byte-key verifier retains its API and rejection order.

Auth 0.25 reports scope grammar errors as
`CanonicalAuthError::Scope(ScopeLabelError)`, including `Empty`, `TooLong`,
`EmptySegment`, `InvalidSegmentStart` and `InvalidByte`. Match that cause instead
of the former `EmptyScope`/`InvalidScope` variants. Errors retain byte counts or
offsets, without copying the submitted label. Validation order, grammar and
public refusal categories are unchanged.

`ScopeLabel` and `ScopeLabelError` now belong to `ic_auth_protocol_types`.
`SessionRequest::scopes()` and `RequiredScopes::as_slice()` return checked labels;
use `as_str()` for borrowed text. Constructors still accept owned strings and
retain count-before-grammar refusal ordering. Labels preserve exact bytes and
cannot be mutated without admission. Raw proof grants remain strings so exact
retries retain their existing deferred proof validation.

`SessionAdmissionContext` contains the protected `PortableVerificationContext`,
receiver role and checked `SessionTtlPolicy`. That verifier selection supplies the
single `PortableClaimsContext` used by both preflight and proof verification.
The request cannot select a key or enrollment. `PortableClaimsContext` contains
shared `PortableAuthority` plus the proof issuance floor and login/proof TTL limits.
`SessionContext` instead contains current `PortableAuthority`, the role,
checked `RequiredScopes` and `SessionTtlLimit`; authorization needs no
proof policy or default request TTL.

`PortableAuthority::grants` borrows `ValidatedGrants`, which owns the host-selected
vector and validates nonempty canonical roles/scopes against the portable limits
once at construction. Call `ValidatedGrants::new(configured_grants)` at enrollment
installation and after decoding retained configuration; keep the value with that
protected enrollment and borrow it for each operation. It offers `as_slice()` for
read-only projections and `into_vec()` for consuming edits/transport conversion.
There is no unchecked deserializer or mutable grant access.

This type proves structure only. A client-supplied grant list cannot establish
trusted authority by passing its constructor. Hosts must select the current
protected value and replace it atomically with the authority generation when
policy changes. Session authorization still checks the live generation, issuer,
key binding, deadlines, login revocation and scope narrowing on every call. Raw
submitted claims retain independent shape and signature checks. Neither proof
verification results nor authorization decisions are cached.

Auth 0.23 requires `SessionContext::required_scopes` to borrow
`RequiredScopes`. Construct it once with `RequiredScopes::new(Vec<String>)` from
the protected operation definition; retain it instead of allocating labels and
validating their grammar per call. Empty requirements use `RequiredScopes::NONE`.
Arbitrary order and duplicates remain valid; the 16-label/64-byte grammar bounds
are unchanged. The immutable view has no unchecked deserializer: reconstruct
after decoding configuration. This is structural evidence only. Select current
operation requirements on every call, and keep live membership, authority and
revocation checks. It is not an authorization cache.

Construct `SessionTtlPolicy::new(default_secs, maximum_secs)` for admission.
It enforces `1 <= default <= maximum <= 1800`. Its `maximum()` supplies the
live authorization limit, or a host can construct `SessionTtlLimit::new(maximum_secs)`
without configuring an admission default. Narrowing that current maximum can
invalidate an existing lease; authority changes still require host-owned generation
updates. `SessionPolicy` carries only generation and protected capability/subject
eligibility. Constructor success does not authenticate any host input.

`portable_credential::verify_credential` and the mechanism-specific byte verifiers
remain independently validating standalone proof APIs. Their success is not session
admission. The engine reuses its own canonical decode instead of calling those
byte APIs again, and budget charges survive later refusals.

Both proof mechanisms enter `SessionError` through
`SessionError::Credential(PortableCredentialError::Ed25519(error))` or
`SessionError::Credential(PortableCredentialError::Certified(error))`.
Auth 0.17 removes the direct `Signature` / `Certified` variants and their
`From` conversions. The typed credential verifier already returns the shared
wrapper; standalone verifier callers explicitly map their error through
`PortableCredentialError::from` before converting to `SessionError`.
`admission_refusal()` preserves the existing public refusal categories.

Cheap binding, revocation, request, replay and capacity refusals precede signature
verification. Fresh admission spends receiver-wide verification credit, including
invalid signatures and later storage failures. Never recreate the budget per
request. Exact retries return a receipt of the unchanged live session without verification,
even after the admission proof expires. Changing the requested scope or lifetime
on a retry is a replay conflict. Renewal uses a fresh credential within the same
immutable issuer/account and login-start/deadline binding.

In Auth 0.23, every `SessionEstablished` variant holds a compact
`SessionReceipt`: browser caller, account issuer/subject and lease expiry. It does
not copy scopes, fingerprints or other stored payloads. A receipt is a response,
not authorization evidence. Use `SessionStore::read` to inspect a full record or
`authorize_session` to project a result after current live checks. The engine
prepares the receipt before committing; it performs no fallible post-commit read.

Admission validates the session record's encoded size without allocating bytes
that would be discarded. The store independently enforces its transactional
capacity checks; persistence explicitly encodes only when bytes are needed.
Record counting, encoding and decoding use bounded native IC Wire frames.
The host still owns persistence, transaction boundaries and corruption handling.

A session records the browser `caller`, account `(issuer, subject)`, audience,
mechanism/key binding, role/scopes, authority generation, login window, lease and
proof/request fingerprints. Two devices belonging to one account use distinct
fresh browser principals and independent sessions. Quotas count browser callers,
not account subjects. Ingress delegation preserving one principal is not a new
login identity.

`authorize_session(..., |session| (session.issuer(), session.subject()))`
projects a result only after all checks pass, inside the host read snapshot.
Use `|session| session.clone()` only when a complete owned record is required.
`SessionReader::session` returns `Option<Cow<'_, Session>>`: memory stores borrow
records, while persistent adapters may return `Cow::Owned` after bounded decoding.
A reference cannot escape `SessionStore::read`; authorization decisions must still
be repeated after an await before protected effects.

`authorize_session` uses the runtime caller and protected current authority,
generation, eligibility and required scopes. It checks the retained login
reservation, revocation and lease without cryptography or network calls.
Authorization does not extend a lease. Proof expiry limits admission; session
expiry is capped by the requested TTL, login deadline and key/enrollment expiry.
Refreshing authority never extends an existing lease. Applications separately
check resource ownership and decide which effects require reauthorization after
an await.

Auth 0.24 exposes `SessionError::read_refusal()` for proof-free readers:
missing/expired session, unavailable authority, rejected scope, revocation or
internal unavailability. Keep the original error locally for diagnostics. The
matching `ReadSession` contract uses identity `0x415554480013`; earlier
`0x415554480003` frames are refused without fallback or altered live checks.

## Atomicity and logout

The host MUST serialize the entire engine call with authority updates and ledger
operations, without await or reentrancy. `SessionStore::admit` and `logout` must
commit all session/replay/index and proposed login-record changes or leave them
unchanged. `read` exposes a read-only snapshot and performs no persistence.
Ledger commit callbacks receive the proposed record, including caller, domain,
retention, pool and immutable account/login binding. Durable hosts must persist it in the
same transaction; it is not a reusable credential or authority snapshot.

The ledger's `with_reservation<T, E>` and `with_revocation<E>` return the commit
callback's error type `E`, with `E: From<LoginRevocationError>`. Callback errors
propagate unchanged; ledger refusals convert through that bound. When inference
has no error context, select `with_reservation::<T, LoginRevocationError>` or
`with_revocation::<LoginRevocationError>` for the ledger-only error surface.
Admission selects `SessionError` and returns backend errors directly. Logout
projects backend commit failures to unavailable storage at its public result
boundary; a backend capacity error must not be confused with ledger capacity.
The ledger still installs a proposed record only after successful host commit.

The store separates immutable `read` snapshots from explicit `admit` and `logout`
operations. Each write requires its login change; logout derives its caller from
that change. The volatile store rejects mismatched domains, record associations
and reservation-versus-revocation states before modifying records. Durable
adapters reuse these checks through `MemorySessionStore::admit_with_commit` and
`logout_with_commit`. Admission returns whether it replaced an existing session. After all validation, a write-only
callback receives `SessionCommit`: an explicit `SessionWrite::Admission` containing
the proposed login record and touched session/replay pair, or `SessionWrite::Logout`
containing the proposed login revocation, and exact post-commit
session/replay counts and encoded-byte total. Reads and rejected operations do not
invoke persistence. A callback error leaves heap records, indexes and accounting
unchanged. Success must mean the durable transaction has completed; the reference
store then publishes the already-validated heap changes. No whole-store clone or
checkpoint is required for this callback. Durable atomicity, metadata generation,
clock/budget persistence, pruning and authority changes remain host obligations;
this hook does not itself supply a stable backend.

Auth 0.23 also makes `SessionLimits` immutable and valid by construction.
Use `SessionLimits::new(max_sessions, max_replays, max_replays_per_caller,
max_encoded_bytes)` at protected backend setup, retain the checked value and
return it from `SessionReader::limits`. Access the four bounds through their
same-named getters. Construct a new checked value when configuration changes or
is decoded. Range refusal occurs at construction; every admission/restore still
checks actual occupancy, associations, generation and encoded bytes.
`MemorySessionStore::new(generation, limits)` now returns the store directly;
remove its former `unwrap`, `expect` or `?`. Record restoration remains fallible.

Auth 0.23 changes `SessionStore::admit` and
`MemorySessionStore::admit_with_commit` to consume owned `Session` and `Replay`
values. Update backend signatures and move them into storage rather than cloning
the pair. The commit callback still borrows the staged records; validate and
prepare everything before durable success, then publish without a returned
failure. Rejected writes preserve the prior store and ledger. This Rust API
change does not alter persisted records or require a reset.

### Session commit write migration

Auth 0.19 is a hard cut at the Rust API; migrate each current caller together:

- Replace the claims slice and verification callback with the complete credential
  and `SessionAdmissionContext`. Select a verifier feature for fresh admission.
- Move shared clock, identity, grants, authority deadlines and login reset floor
  into `PortableAuthority`. Put proof-only inputs on `PortableClaimsContext`.
  Authorization uses `SessionContext` with current authority and maximum TTL.
- Replace raw `PortableAuthority::grants` slices with a retained `ValidatedGrants`
  constructed from protected configuration. Reconstruct after trusted decoding;
  replace it with authority/generation changes. Constructor success proves shape,
  and existing live checks remain required.
- Replace raw default/maximum arguments with `SessionTtlPolicy`; use its
  `maximum()` or checked `SessionTtlLimit` for live authorization.
- Implement `SessionReader` and `SessionStore::{read, admit, logout}` instead of
  mutable staging transactions. Reads cannot write. Every write validates current
  generation/association and commits the required records atomically; reads never
  invoke persistence. `admit` returns true only for replacement.
- Match `LoginRevocationChange::state()` as `LoginRecordState::Reserved(binding)`
  or `Revoked(Some(binding))` / `Revoked(None)`. `LoginBinding` has named
  issuer, subject, start and deadline fields. The state replaces the correlated
  revoked/pool flags and optional tuple getter; it is not a credential.
- Match `SessionCommit::write` as `SessionWrite::Admission { login, session, replay }`
  or `SessionWrite::Logout { login }`. The counts and encoded-byte total remain
  post-commit values. Replace the combined commit hook with the explicit admission
  or logout hook.

There are no aliases or fallback entry points. Every host commit must finish
atomically before reporting success, and returned failure must preserve storage.
Signed claims, public Wire frames and the browser journal keep their existing
formats. Auth 0.22 retired private CBOR storage as described below. Consumer adoption follows publication;
local fixture qualification does not establish a Canic deployment.

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

### Availability under contention

`SessionAdmissionBudget` bounds attempted cryptographic work across callers and
proof mechanisms. Anonymous callers, malformed claims/signatures, caller mismatch,
expired proofs, replay conflicts and full-state refusals are checked before
expensive verification credit is charged. Failed cryptographic verification does
consume credit; refunding it would admit unbounded expensive invalid requests.
A sustained source of well-formed forged proofs can therefore deny fresh
verification until credit refills. Existing valid sessions and exact retries do
not need fresh verification credit. The bucket provides no multi-client fairness
or Sybil protection. Hosts and issuers must qualify their own ingress, enrollment,
credential-issuance and workload admission policy; per-principal quotas alone do
not establish that policy.

Only completely verified admission can reserve an admitted-login slot. One
runtime caller retains one login binding; cheap refusals and failed commits
cannot install a reservation. A reservation lasts through its issuer-authenticated
original login deadline, even after its shorter session
expires. Hosts can constrain accepted login duration through protected claims
policy. Multiple valid issued credentials can fill a finite pool; admission must
then refuse rather than evict retained safety evidence.

Unknown-login logout uses its own pool and budget. Repeated logout by one caller
keeps one record and its original deadline; rotating self-authenticating keys can
fill that unknown pool without an issuer credential. This cannot take admitted
slots or prevent an admitted login from converting its reserved slot to a denial.
Unknown tombstones provide temporary local denial for the protected host login
ceiling, not confirmation about every unseen credential. The same original
signed credential may first become valid after that denial expires. Unknown
saturation can refuse other unknown logouts but never falsely acknowledge them.
Configure physical storage for both declared pools.

Expired physical rows continue occupying capacity until explicit bounded
maintenance commits. Persist deletions/counts atomically, retain failure and clock
observations, and schedule maintenance rather than resetting the ledger. The
contention regressions qualify cheap floods without credit loss, shared-budget
exhaustion/refill, independent unknown capacity, and successful legitimate
admission after bounded pruning. They do not establish availability under an
unlimited adversarial load or a production host's admission policy.

`logout_session` returns the shared `LogoutResult`: `Revoked { deny_until_ns }`
only for a known login after commit, `Unconfirmed { deny_until_ns }` for
temporary unseen-login denial, `InvalidCaller`, or typed `Unavailable` (capacity, storage or
clock). Hosts return `Unavailable(ResetPending)` before touching unavailable
startup state. The Wire unit request `()` accepts no client-selected identity or
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

Auth **0.22** retires all previous private CBOR records. The maintainer
confirmed no deployed or retained Auth records need preservation. There is no
legacy reader, migration or implicit clearing. A host adopting this cut must
initialize a coherent new session/replay/login checkpoint before serving; never
retain only part of a previous replay or revocation history.

Auth **0.25** separately retires the previous text-ID session/login records.
The maintainer confirmed **no retained current records; a hard cut is OK**.
New session/login records encode their two audience IDs as fixed 32-byte strings;
decoders refuse wrong lengths and former record identities. Public `CanonicalId`
serialization remains hexadecimal text. No fallback reader or implicit clearing
is provided. Adopt the complete session/replay/login checkpoint together, including
its encoded-byte totals. The private qualification checkpoint is `AUTHNAT3`;
`AUTHNAT2` is refused without clearing. This consent does not authorize erasing a
Memory allocation ledger or resetting backing storage.

The pending clock-independent hard cut separately retires the login-record and
private checkpoint identities. The maintainer confirmed no retained affected Rust
records on 2026-11-06. Login identity `0x415554485313` and checkpoint `AUTHNAT3`
are refused; the current checkpoint is `AUTHCLK1`. This does not authorize implicit
clearing of consumer storage. Session/replay layouts and identities stay unchanged.

Current records are native IC Wire request frames, using storage-only identities
`0x415554485311` (session), `0x415554485302` (unchanged replay) and `0x415554485323`
(login ledger); these are never mounted RPC methods. Byte bounds include the
11-byte header: 2048, 512 and 1024 respectively. Traversal is bounded at 1024
nodes, depth 16 and a 65,536-byte native allocation allowance, independent of the
byte ceiling. The login schema stores one enum with reserved, known-revoked and
unknown-revoked projections, without redundant boolean flags.

Decoding checks identity, direction, complete consumption and shape; it does not
authorize a record. Current authority and its matching retained login reservation
are required. Signed claims, public application Wire contracts and external IC
certificate/signature CBOR remain unchanged. Replay encoded bytes count toward
store capacity; raw replay serialization is unsupported.
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

After safety-state loss, stay closed pending trusted complete recovery. Local
time floors, local generation increments and fresh enrollment cannot fence every
old signed login. Normal durable upgrades restore complete session/replay/login
and authority metadata atomically, then authenticate fresh enrollment before
readiness. The ledger supplied here is volatile; a production durable host adapter
remains consumer-owned. See the [clock contract](design/portable-clock-audit.md).

The durable adapter must use `ic-storage` for host-composed allocation ownership,
with the host supplying the combined declarations and performing bootstrap before
stores open. Storage 0.68 supplies namespace admission, checked leases and its owned
collections; it does not supply Auth's record schema or transaction. The internal
checkpoint uses `ic_storage::DefaultMemoryImpl` and bounded
`adapter::StableCell` loading. Its allocator/collection formats require fresh
backing; older Memory images are refused without reset. All workspace packages
share Rust 1.99.1; see [minimum compiler qualification](msrv.md).
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
recovering incomplete storage cannot use local time as an invalidation fence. Canic's per-operation persistent adapter acceptance remains consumer-owned.
A durable implementation must cover protected reset/authority metadata. Returned
failures leave state unchanged while retaining charged work budgets. Normal upgrades must restore matching reservations
as well as sessions; incomplete recovery must stay closed rather
than invent missing reservations. [Canic #491](https://github.com/dragginzgame/canic/issues/491)
owns that consumer integration and its failure/upgrade qualification. Auth's
storage/reset interfaces are delivered; [Auth #58](https://github.com/dragginzgame/ic-auth/issues/58)
retains their historical implementation evidence. The qualification host selects
`ic-storage`; none of the three public libraries depends on it.

## Custom host transaction contract

The portable engine is a protocol change, not just a renamed transaction method.
Canic's reported integration stays in [Canic #491](https://github.com/dragginzgame/canic/issues/491).
Hosts can reuse `MemorySessionStore::admit_with_commit` and `logout_with_commit`
to avoid copying their validation/index rules. A custom store with its own indexes must enforce the same
contract before any persistent effects:

| Proposed operation | Required login change and checks |
| --- | --- |
| Read only | `read` exposes only `&dyn SessionReader`; it cannot stage writes or removals |
| Admission/replacement | `admit(session, replay, login)` calls `session.check_admission_pair(&replay, login, current_generation)` for reservation, pair association and generation; check replay uniqueness, record sizes and capacities |
| Session removal | `logout(login)` requires `LoginRecordState::Revoked` and removes only `login.caller()`; check any retained session association and retain replay history |
| Missing session removal | Still commit the proposed tombstone; absence is not evidence that a delayed admission cannot arrive |

`Session::check_login_change` exposes the existing association check without
revealing private storage fields. It checks caller/domain for both operations,
and issuer/subject/login-window/retention for admission. A revoked record may
have no login binding, so that an authenticated proofless logout can remove an
older session. Passing this check never permits admission with a tombstone or
replaces current authority, replay, capacity or atomicity checks. No submitted
record is a credential. Persist the login change with the session/replay/index
updates; return success only after the whole commit succeeds.

`SessionLimits::check_capacity(occupancy, replacing)` shares the reference store's
physical session/replay/per-caller count check. Include expired records until
pruning, compute replacement from the same snapshot and independently enforce
encoded-byte and backing/index budgets. `LoginRevocationChange::check_replacement`
checks a proposed record against a retained record for the same caller/domain at
trusted time: a live binding/pool, denial and retention deadline cannot be lost.
At exclusive expiry a new binding/pool may replace it. The helper authenticates
neither record provenance nor current authority and does not charge budget,
update clock/pool totals or commit. Preserve those obligations atomically.

The rest of the host cut must be propagated together:

- Replace old `DelegatedToken` session inputs with `PortableCredential` and a
  protected `SessionAdmissionContext` with its `PortableVerificationContext`.
  Pass the complete credential and shared budget to `establish_session`; Auth
  owns the single decode and complete verifier call. Do not wrap an old token as
  new signed claims or use standalone proof success as session authorization.
- Retain one login ledger and admission/unknown-logout budgets across requests.
  Pass the ledger to establishment, authorization and logout. Fresh allocation
  per call would lose denials and replenish abuse credit.
- Use `Replay::caller()`, `max_replays_per_caller` and `replays_for_caller` for
  login accounting. Application account identity remains `(issuer, subject)`;
  replacing subject indexes mechanically is not an account migration.
- Durable hosts select `session-restore` and use the bounded session/replay/login
  codecs. Remove raw replay deserialization and explicitly retire incompatible
  retained layouts. Restore complete matching metadata and records, then obtain
  fresh receiver-bound enrollment before serving. Volatile state loss stays closed
  pending trusted complete recovery; neither profile can fabricate missing reservations.
- Qualify failed commits, exact retries, same-account separate browser callers,
  session-only logout with replay retention, unknown-login logout and upgrade
  restoration/reset in the actual host. Auth's fixture is reference evidence,
  not production Canic adoption or a completed per-operation durable backend.

### Backend conformance scenarios

The dedicated unpublished
[`ic-auth-backend-conformance`](../crates/ic-auth-backend-conformance/README.md)
package exposes `Backend` and `assert_backend_contract`. It depends only on public
Auth/protocol APIs and Platform Ed25519, without private fixture DTOs, Metrics,
certified verification or simulator/runtime dependencies. Adapt its test-driver
hooks to isolated backing and run the scenarios against the actual `SessionStore`.
Keep this test support outside deployed graphs.

`make test-backend-conformance`
runs the same scenarios against reference commit callbacks and an independent
encoded-record backend. It also proves a write-then-error adapter is rejected.
The complete entrypoint runs four separately callable scenarios for admission/retry,
logout/reopen, replacement/generation and capacity; no scenario is optional for
complete conformance. See the package README for their names and driver hooks.

After publishing the matching Auth cohort, `make pack-backend-conformance` creates
a Cargo-normalized source archive for consumers. Use that normalized manifest,
with registry Auth dependencies, rather than importing the producer workspace's
Git/path identities. See the package README for extraction and driver requirements.
Scenarios cover failed admission/logout, spent verification credit, exact retry
after proof expiry without lease extension, replacement, replay/denial retention,
active/revoked reopen, generation invalidation and count/byte capacity refusal.
Replay-capacity scenarios cover both global and per-caller pools, original exact
retry at saturation, and independent admission for a nonsaturated caller.
Deliberate write-then-error adapters exercise both admission and logout detectors.
Core tests separately exercise malformed/mismatched pairs and live record
replacement. These checks do not establish crash atomicity, rollback protection,
durable clocks/authority, cleanup bounds or deployed consumer acceptance.
`make test-backend-package` independently qualifies the archive against registry
Auth identities and emits a source/integrity receipt. It requires explicitly
prepared registry dependencies and does not run as part of ordinary CI.

## Executable host integration references

Use the qualification receiver's
[`establish_portable_session` and `portable_session_read`](../crates/ic-auth-qualification-canister/src/sessions.rs)
as the endpoint wiring reference: runtime caller/time, retained ledger and budgets,
protected enrollment, and engine-owned complete verification.
Its controller-installed configuration is a test boundary; production Canic must
supply its authenticated Root enrollment and its own atomic durable store.

The certified-path
[`certified_proof_uses_the_shared_session_lifecycle`](../crates/ic-auth/tests/portable_certified/mod.rs)
regression uses the same public credential envelope and protected admission API. One signed proof is
admitted independently by two receivers; an exact retry after proof expiry retains
the original lease. Logout blocks re-admission before cryptography on that receiver
while the other receiver remains authorized. This is a native protocol regression,
not deployed cross-subnet or Canic endpoint qualification. Run `make test-sessions`.

### Host-owned admission diagnostics

The tested [admission recorder](../crates/ic-auth-qualification-fixtures/src/metrics.rs) uses
IC Metrics 0.17 `MeasurementCounter` fields in the unpublished qualification
host. Public Auth libraries have no Metrics dependency. The receiver calls
`host.metrics.observe(&result)` once, immediately after `establish_session` and
before mapping its reply. It does not parse a proof, reverify a retry or make an
authentication decision. Fixed categories count created, replaced, exact retry,
fresh proof-stage refusal, exhausted verification budget and other engine refusal.

Only requests reaching the engine enter this window; transport and guard errors
are excluded. Proof-stage refusal means a typed `SessionError::Credential` other
than budget failure, including pre-cryptographic mechanism/shape refusals. It is
not a count of cryptographic attempts or rejected identities. Budget clock/config
failures count as other refusals. Store failures never count as created/replaced.

Ordinary errors retain both observations and failed-attempt charges. On the IC,
a message trap rolls back that message's state and diagnostics together; hosts
must return ordinary refusals rather than trap away charges. Record successful
changes only after the host's atomic store commit succeeds. If an integration
defers commit beyond the engine call, it must defer success observation too.

The reference window is volatile and starts empty at installation/restoration.
Starting a new diagnostic window changes no budget, replay, denial, generation or
authority state. Recorders retain `u128` totals and Metrics' actual-overflow flags;
an exact maximum is distinct from actual overflow. They are not transport DTOs
and do not implement Serde. Canic's metrics adapter owns reporting and Wire/TypeScript
projection. No caller, principal, token, proof or credential-derived label is
recorded. The host owns window identity, persistence, report authorization,
replication and resets. Counter overflow must not change an admission result.
There is no report/reset RPC, production adapter or instruction measurement here.

Run the focused example tests with
`cargo test --locked --offline -p ic-auth-qualification --test session_metrics`.
They also run through `make test-qualification` in the complete gate.
[#106](https://github.com/dragginzgame/ic-auth/issues/106) owns this reference;
Canic retains its own production adoption and host policy. The accepted production
boundary is Auth's existing typed outcomes into Canic's shared metrics store and
existing metrics endpoint. Canic owns that recording/projection connection and
its reporting authorization/configuration. Each metrics producer needs no separate
endpoint; this reference requires no new public Auth metrics subsystem.

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

That checkpoint uses published `ic-memory` 0.55.0 with one host-bootstrapped
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
