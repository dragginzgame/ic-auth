# Volatile receiver admission

`ic-auth` now supplies an opt-in `receiver-stable` module, selected with
`ed25519-verification`, `certified-verification`, or both. It owns caller records,
device indexes, consumed replays, issuer-qualified replay counters, expiry indexes,
physical totals, a trusted-clock high-water mark and one weighted verification
budget. Protocol-only and standalone-verifier builds do not select this backend.
Canic remains responsible for endpoint installation and authenticated host inputs.

## Operation and ownership

Admission canonically decodes claims once and checks browser/caller binding before
lookup. A private decoded value lazily computes the exact-byte fingerprint once;
the complete verifier consumes that same decode. No public prepared write,
verification callback or caller-provided success flag is accepted.

An exact live retry checks current authority, request identity, scopes and retained
login/device association, then returns the original receipt without proof work.
It can outlive its admission proof, including after replay pruning. Changed TTL,
revocation, stale authority and expired leases cannot revive a session.

Fresh admission uses the existing shared context, login and capacity rules before
expensive verification. Only complete proof success permits publication. The
receiver prepares bounded records and the receipt before updating the caller,
slot, replay, account and expiry indexes. Replacement reuses a known-login row
and any displaced session, but never deletes old replays or replenishes account
quota. Renewals retain the immutable binding and absolute login deadline.

Ordinary refusal leaves authorization records unchanged. Required clock observations
and attempted-work charges remain. All ordinary fallible work precedes publication;
there is no fallible read after commit and no complete-store clone. Publication
marks the instance closed before any map allocation, restoring readiness only on
completion. A panic cannot expose partial state through subsequent operations.
Allocator failure/host traps still require the host's abort/recovery boundary;
this is not evidence of installed IC rollback or native allocator fault injection.

Native publication tests inject unwind failures between the actual record, index
and counter mutations. They enumerate every instrumented boundary for fresh
admission, renewal, replacement, expired unknown-to-known transfer, known logout,
new/expired unknown logout and combined pruning. After each interruption, admission,
read, logout, pruning, generation changes, enrollment and inspection must all refuse
as closed. A fresh run beyond the last injected boundary must complete and restore
readiness. The thread-local hooks and controls are compiled only into unit-test
builds; there is no public injection API, callback or deployed fault state.

These tests share the signed fixture driver with the public integration cases.
They establish fail-closed behavior after native synthetic publication panics,
not recovery of the partial state. Hosts must discard poisoned instances and own
actual rollback/recovery. Real allocator failures, IC traps and retained-storage
restoration still need their respective host qualification.

`read` performs proof-free current-policy checks and returns account facts. The
host must keep its authority/application transaction isolated and recheck after
any await. `authorize` performs those same checks and invokes one synchronous
application projection with a borrowed receipt only on success. An Auth refusal
never calls it; application errors remain inside the successful Auth result.
Borrowed receipts cannot escape the invocation or be retained in a returned
future, as checked by compile-fail cases. Copied facts are still only facts;
the API does not confer authority on deferred application work or supply rollback.
Obtain current host inputs and call `authorize` again after every await.

`advance_generation` invalidates old sessions without resetting state
or credit. `logout` needs authenticated caller/time, not fresh enrollment; it
removes a known session while retaining denial, device fencing and consumed replay.
Known-login logout succeeds independently of unknown pool/credit exhaustion and
retains its original deadline even after expiry.

Unknown logout uses `MemoryConfig::unknown_logout`: a positive physical pool bound,
operation capacity and refill interval. Temporary retention uses the protected
maximum login lifetime. Its separate bucket begins empty; proof credit is untouched.
Live retries retain their original deadline without charging or refilling. Expired
rows count until explicit pruning, reuse by that caller, or successful verified
admission into the independently bounded known pool. No automatic eviction occurs.
An expired-row transfer that fails capacity or proof verification preserves both
pools. The combined caller map remains the sole record owner; only known rows have
device indexes, while only unknown rows have a denial-expiry index.

`LogoutReceipt::Unconfirmed` never acknowledges every outstanding credential,
even when its denial is committed or retried. Independently signed future evidence
may become valid after that deadline. `Revoked` requires a retained authenticated
login binding. The qualification host projects results/refusals into the unchanged
Wire variants and checks exact frames; product crates install no endpoint. Host
clock, unavailable-time and response authentication remain the adapter's responsibility.

Replacing a logged-out device before its denial expires transfers the displaced caller
to the unbound denial pool through the original deadline. This costs one pool entry
and one expiry-index entry. It spends no unknown-logout credit. If the pool has
no room after accounting for the incoming caller, replacement returns capacity
before signature verification and the original denial stays in force.

## Host scheduling and sizing

The receiver shares verification credit across callers. A failed signature consumes
credit, so hosts should throttle fresh admissions by authenticated caller before
verification and bound aggregate ingress across callers. Budget exhaustion can
delay new admissions even while established sessions and exact retries still work.
Schedule bounded `prune` calls regularly and continue draining expired entries
when a call reaches its work limit. The work count is the sum of sessions,
replays, unknown denials and device slots in the prune report.

## Bounds and accounting

Size `login.max_retained_device_slots` for the number of distinct
(issuer, account, device slot) combinations that need simultaneous protection.
The default 65,536 rows cover 8,192 accounts using eight devices each, or
65,536 using one each. The independent `capacity.max_sessions` limits live leases.

The receiver retains a known slot through the login start plus the receiver's
maximum login lifetime, any longer retained denial, and any later protection
deadline inherited from the previous owner. Issuers assign increasing generations
in trusted start-time order and renewals preserve the original login window.
At or after the deadline, bounded pruning can remove the slot and caller row.
The account replay counter survives if replay records still exist.

To raise the slot ceiling, supply the larger limit in the trusted host configuration
to `StableReceiver::reopen` during upgrade. Reopen validates the saved rows against
their original limit and checks the proposed configuration against the 64 MiB
checkpoint budget. A decrease or a change to any other field refuses the upgrade.
The host must trap on refusal for IC rollback. Prune visits at most 128 entries
per call across all four categories.

The starting profile budgets 20,000 retained session rows and 80,000 replay rows
with a shared 32 MiB encoded byte limit. Admission and reopen measure the same
session body and replay row encoding. A replay retains the fingerprint, account
and expiry. The receiver reserves checkpoint space for caller keys, dormant
device fences, unknown denials, and chunk framing before it accepts a profile.

A receiver v1 checkpoint stores caller and replay rows in bounded chunks.
Reopen validates the records and rebuilds secondary indexes. The native
full-profile test reopened a 22,926,743-byte Cell payload with 20,000 sessions
carrying 32-byte extensions, 80,000 replays, and 65,536 device slots. The
standalone PocketIC probe measured 5.14 billion instructions to seal and 13.51
billion to reopen. Measure the whole consuming canister at its expected payload
and replay retention before setting production upgrade budgets.

`inspect_sessions` borrows passive session facts in Principal order. Each call
returns at most 32 sessions, visits at most 128 caller rows (or the smaller selected
scan budget), and reads at most one further row to determine continuation. Denials
and known fences count toward the scan even though they return no session. Follow
the exclusive `next_after` cursor on empty/short pages; inspection never loops to
fill a page internally. A removed cursor still works. Inserts before it need a new
traversal: separate calls do not hold a common snapshot.

Expired/stale-generation sessions remain visible until explicit removal. The
returned `SessionFacts` establish no current authorization; actual admission/read
checks remain mandatory. Hosts own administrative access. Inspection can observe
fresh state awaiting enrollment or an enrollment outage, but refuses poisoned/closed
state. Its supplied time must not precede the retained clock; it neither advances
that clock nor refills budgets, prunes records or refreshes authority. Only the
bounded projection vector is allocated; stored roles/scopes are borrowed.

The page/cursor/scan contract and independent 512-denial scenario are retained from
the pinned reference's persistent receiver inspection and test files, identified
in `reference.sha256`. This is a new memory traversal, without stable record
decoding or a storage adapter. An instrumented iterator measures actual consumption
at both hard limits; public API tests cover mixed sessions/fences, empty pages,
deleted cursors, interleaved insertion, passive expiry/generation observations and
invalid bounds/clock/lifecycle refusal. Both real proof mechanisms supply sessions.

Logical session/replay sizes preserve the pinned reference `SessionData` and
`ReplayData` representations, including the 11-byte frame overhead. The optional
backend uses published IC Wire 0.24.2 `encoded_len` and private borrowed Serde views;
it emits no stored frames, selects no persisted layout and introduces no old-format
reader. The registry index confirmed 0.24.1 as latest non-yanked on 2026-11-15.
Wire and Serde are optional backend edges using the existing root selections.
The reference field order and audience representation are tracked in
`reference.sha256` (`session/model.rs`, `storage_codec/mod.rs`).

## Lifecycle and evidence

`new_fresh` starts closed with empty credit and installed receiver/authority identities.
It remains a host assertion of genuinely new safety state, never a loss/recovery API.
The former `activate_fresh(generation)` shortcut has been removed. The host calls
`request_enrollment` with a fresh unpredictable challenge and local receipt deadline,
then `install_enrollment` with metadata from the exact installed authority's
IC-authenticated response. Replacing a pending request fences its late response;
an in-flight refresh does not extend or discard currently accepted enrollment.

The receiver checks response source, receiver/domain/challenge, exclusive receipt
deadline, issue time and positive freshness of at most 3,600 seconds. It retains
revision/generation high-water marks and the accepted metadata. Exact response
repetition is idempotent within its receipt/freshness window. New snapshots require
a greater revision; authority-content changes require a greater generation.
Refusal leaves accepted metadata, authorization records and budgets intact while
retaining trusted clock observations. The fixed-size metadata commit allocates
nothing and has no fallible tail.

`EnrollmentMetadata` is a trusted host projection, not a proof or serialized
enrollment contract. The host computes `snapshot_digest` over the complete canonical
snapshot and `authority_digest` over its entries, revocations and authority policy,
excluding response/freshness metadata. It validates all entries and terminal key
revocation, authenticates the actual response source, and atomically installs the
complete snapshot alongside Auth's metadata. Per-operation verifier contexts must
come from that same isolated snapshot, including tighter deployment approval
deadlines. No digest or caller-provided source value establishes provenance by itself.
This introduces no new signed format, response verifier, timer or executor.

Admission/read independently require current accepted enrollment and matching
generation. Accepted freshness caps the captured session lease even if a supplied
context permits a later deadline. Refresh does not extend old leases. Generation
changes invalidate old sessions without scanning/removing records or resetting
proof/unknown-logout credit. Expired enrollment still permits safe self-revocation.
`close` is terminal: neither a new challenge, local generation nor authenticated
enrollment can reopen a lost/poisoned instance. A host must not construct empty
replacement state after loss. Terminal-revocation installation and actual IC boot/callback
authentication remain host qualification obligations.

Run `make test-memory`. The public-API Ed25519 scenarios use twelve independently
packed and OpenSSL-signed credentials. They cover failed verification without
reservation, renewal/replacement at capacity, replay quota and generation retention,
proof-expired retry, exclusive leases, scope checks, clock rollback, generation
invalidation, byte boundaries, pruning and logout. Unknown-denial cases cover
non-sliding retries, separate pools/budgets, successful and failed pool transfer,
future signed credentials, admission/logout ordering, clock overflow and isolation.
Seven readiness cases additionally cover bound boot responses, superseded requests,
receipt/clock/freshness boundaries, rollback and conflicting snapshots, outage logout,
generation invalidation and non-extending refresh with real admission credentials.
Four inspection cases and an instrumented iterator check add bounded read-only
maintenance evidence. The same target runs the iterator check before the signed cases.

Projection cases check exactly-once invocation, application-error separation and
zero invocation after scope/eligibility refusal, logout, authority invalidation
or lease expiry. The multi-account case admits two devices for one account and
an equal subject ID under a different issuer on two independent receivers. It
checks account quota saturation without blocking the other issuer, failed and
successful replacement at full known capacity, unaffected other devices/accounts,
proof-expired retry, renewal with the original deadline, local logout and retained
fences after cleanup. The two additional fixtures use independent Node/OpenSSL
packing/signing; previous fixture bytes are unchanged. These cases establish native
Ed25519 multi-account behavior. Certified evidence is described below; SDK and
installed-host qualification remain open.

The same test binary also runs an independent sequence ledger in
`crates/ic-auth/tests/receiver_memory/memory_model.rs`. Its fixed scenario has one account,
one device slot, three browser callers, four signed credentials and two independent
receivers. It uses no Auth policy, encoding or verification function for expected
decisions. A small logical ledger and elapsed-time-minus-charges arithmetic compare
outcomes, reads, physical session/replay occupancy, independently calculated byte
totals, known fences, unknown denials and both retained budgets after every command.
Both receivers are checked, including the one not targeted by the command.

Coverage includes all ordered command pairs with expiry/pruning tails, 64 repeatable
longer traces and an exhausted-work trace: 353 traces and 7,307 operations in total.
Exact retry with changed proof material, request conflicts, failed charged proofs,
quota retention through replacement/logout, fresh renewal, generation invalidation
and explicit pruning interact in these sequences. Refusals are compared as rejected
outcomes; the existing focused cases own precise error/charging-stage projections.
The model deliberately avoids bucket saturation and unknown credential shapes;
the separate budget, codec and proof tests cover those boundaries.

A negative control publishes a real replacement then reports a recoverable error
without restoring identity state. The ledger must detect the changed occupancy.
This proves that the harness rejects that broken adapter; it does not implement
rollback, an encoded stable adapter, a product commit callback or fault injection
at each physical mutation. The full cross-backend and installed failure matrix
remains required. The model is test-only and introduces no product dependency.
The initial signature is
cross-checked against the frozen legacy vector. Logical sizes are independently
summed from reference Postcard fields, including a ten-byte generation boundary.

Five certified-only qualification cases use independent root signatures and real
BLS certificate fixtures. Root and issuer failures retain charges without creating
state; success creates a session and proof-expired retry ignores alternate proof
material at full capacity. The extended scenario uses seven independently packed
claims and two independently root-authorized issuer identities, with two devices
on one account and two receivers. It checks quota isolation, replacement without
displacing another device, charged root/issuer failures, renewal, local logout and
retained fences. Additional cases check enrollment refresh without lease extension,
generation invalidation, outage logout and unknown denial before admission.
See the [fixture provenance](../../qualification/ic-auth-certified-conformance/README.md)
for independent construction and scope. Existing standalone verifier and combined-profile cases
remain regression coverage. Native lint, docs and Wasm library compilation cover
the selected feature graphs; compilation is not installed-host qualification.

The installed host suite exercises upgrade and loss handling, rollback, and
browser admission and logout. Qualify the consuming canister at its chosen
session, replay and retained-device population before cutover.
