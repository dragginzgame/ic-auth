# Receiver read and retry policy

`ic-auth::receiver_policy` assesses a bounded, borrowed set of retained facts.
It is the shared semantic layer for future receiver backends, not an installed
receiver, storage interface, authorization token or session constructor. Its
public fact views are provisional Rust inputs, not serialized record layouts;
the consumer API inventory and final backend boundary remain open.

The host selects current caller, clock, audience, issuer/key, role, grants,
receiver generation and account eligibility. Session, login and device facts
must come from a complete isolated snapshot of previously admitted state. A
snapshot assembled from submitted credential fields establishes no authority.
Stable backends must admit corruption and index associations before projection;
volatile state must preserve its structural invariants. The views permit borrowing
from one authoritative caller record and its derived indexes; they do not require
physical duplication, a temporary login ledger or a generic storage framework.

`assess_session` checks current policy and clock order, live denial, receiver
generation, session structure/liveness, current authority and retained lease
ceilings, narrowed scopes, login reservation and device association, then the
operation's required scopes. Required scopes may be empty, duplicated or unordered
within the 16-label bound. Stored scopes and authority grants must remain canonical.
There is one private role-selection/scope-membership owner shared with fresh
verification and request narrowing; no scopes are combined across roles.

`assess_retry` takes the exact canonical claims digest and a checked unsigned
request. A matching session fingerprint with a different request hash conflicts
before session liveness. An exact request still undergoes all current live checks
and returns only the original deadline. Otherwise a consumed replay conflicts;
an unconsumed fingerprint yields `FreshProofRequired`, which permits further
checking only. It says nothing about fresh authority, capacity, denial binding,
proof validity or admission. No identity or reservation can be installed from it.

Proof expiry, proof bytes and work allowance are absent from retry assessment.
An exact live retry can succeed after proof expiry and at saturated capacity;
lease expiry, revocation or stale authority cannot revive it. An ordinary read
also checks that the whole retained lease fits current key/enrollment ceilings
and the current maximum original duration. It does not silently shorten an
old lease into validity, and refreshed enrollment does not extend its deadline.

The helpers are read-only. They compare trusted time to the supplied retained
observation but do not advance it. The operation owner must preserve required
clock observations on normal refusals, retain work charges at their existing
boundary, and keep all facts and decisions inside one host transaction. They
perform no callbacks or effects; a copied success is not later authorization.
Readiness, replay lookup integrity, loss/reopen rules, capacity, publication and
rollback remain obligations of the future receiver/backend owner. There is no
Wire refusal mapping or Canic endpoint installation in this increment.

## Evidence

`make test-receiver-policy` runs thirteen independently written fact scenarios and
two qualification drivers using the existing independent Ed25519/root/BLS fixtures.
The request identity comes from a retained reference digest. Expected outcomes
and deadlines are literal contract examples, not produced by these helpers.
Scenarios cover proof-expired retry, unsigned scope reordering, changed scopes/TTL,
consumed replay after removal/logout, denial precedence, exclusive lease expiry,
current authority/eligibility changes, scope narrowing, missing reservations,
issuer-qualified device fences, malformed records and clock rollback.

The qualification driver projects already verified fixture evidence into borrowed
facts, then assesses at time 185 after proof expiry 180 and before lease expiry
190. Both proof mechanisms return the same original deadline; time 190 refuses.
This is native policy integration, not actual storage admission or durable retry.

## Login renewal and replacement preflight

`assess_login` consumes a proposed binding, protected clock/limits and at most two
selected login rows plus their slot index and physical known-login count. It
classifies a new login, exact-binding renewal or possible displacement. Those
results are not reservations or prepared write sets. Proposed identity remains
untrusted until the complete verifier succeeds over the exact same claims and
the receiver rechecks current facts in the same uninterrupted transaction.

Caller selection comes from authenticated ingress. Slot lookup uses the proposed
issuer-qualified account only after bounded decode and browser/caller binding.
The backend must establish complete lookup results; missing or corrupt records
cannot become `None`. Same-caller views must agree, and a retained known caller
cannot lose its device index. Stable corruption admission remains backend-owned.

A known caller cannot change issuer, subject, slot, generation, login start or
absolute deadline. Known denials and generation fences remain effective after
expiry. An unknown denial refuses before its exclusive retention deadline and
can be superseded afterward. A different browser needs a strictly greater
generation to replace the selected slot; comparison never increments or wraps
generation values. Replacement may supersede the old browser's known denial,
but a live denial for the new browser still refuses. Complete proof verification
and atomic session/replay/login/index/accounting publication remain mandatory.

Fresh known-login rows require positive protected physical capacity. Expired rows
still count; renewing or replacing an existing known slot reuses its allocation.
An expired unknown denial does not release known-login capacity. Renewal preserves
any longer retained deadline. This is only the known-login count check: replay,
session, unknown-denial, encoded-byte and actual backing limits remain separate.

Six login scenarios check immutable binding, full-pool reuse, denied/expired
fences, unknown-denial expiry, incomplete/cross-account indexes, protected bounds
and maximum generations. The signed-fixture driver proposes generation seven
against a denied generation-six browser in a full pool. Both valid mechanisms
verify, while malformed root/Ed25519 signatures refuse with their retained work
charges. All selected facts stay unchanged; this establishes no store rollback
or publication guarantee. Fresh proof/time/authority and session admission still
belong to the future operation owner, not this structural preflight helper.

Contracts were inspected at reference `e52a40be75dd9809d0528189d84b43fe96952ad9` in
`session/mod.rs`, `session/model.rs`, `portable/revocation/mod.rs` and
`docs/protocol/sessions.md`. Their hashes are recorded in `reference.sha256`.
The policy and scenarios are newly written with those source contracts available;
no legacy storage engine or record decoder is imported. This is not a clean-room
claim or full differential backend qualification. `ic-storage` remains interim;
`ic-storage-new` adoption still requires its own lifecycle and rollback evidence.

## Session/replay capacity

`assess_capacity` checks positive protected limits, selected account/count/record
facts, physical row capacities and checked encoded-byte arithmetic. Its returned
`SessionUsage` contains projected totals only, not a reservation or applicable
write set. The backend must calculate exact record sizes and establish which one
retained session, if any, is actually replaced in the same transaction.

Every fresh admission adds one replay to both global and issuer-qualified account
occupancy. Browser churn, logout and replacement never refund replay quota.
Expired rows remain charged until explicit successful pruning. Replacement reuses
one session row and subtracts only that session's exact size before adding the new
session and replay. A retained device/login fence without a session releases no
session capacity. Exact retries bypass this fresh-admission check.

The reference session/replay record ceilings remain 2048/512 encoded bytes. These
limits do not select a storage format or claim a new encoder; no old record reader
is introduced. Login rows, unknown-denial pools, heap/index costs and actual
Storage backing limits are separate. All arithmetic is checked, including byte
subtraction underflow and additions at the platform's `usize` boundary. Backends
still own complete accounting/index corruption admission and atomic publication.

`make test-admission-capacity` runs six literal count/byte scenarios derived from
`session/model.rs` and `session/memory.rs` at the pinned reference. Cases cover
independent count limits, full-pool replacement, account ownership across issuers,
replay retention after sessions disappear, exact byte ceilings, malformed sizes,
inconsistent selected facts and arithmetic extremes. Extreme integer scenarios
exercise arithmetic, not claims that such a populated store fits real memory.

A read-only inspection of the cached published `ic-storage` 0.70.0 adapter confirms
that reserving region capacity is not a transaction and native insert failures
can follow writes (`adapter/region.rs`). This is not a supplier freshness review
or adoption: no Storage dependency is added here. Actual host rollback and
backend publication remain necessary before an atomic receiver can be claimed.

## Memory operation owner

The optional [memory receiver](memory-receiver-boundary.md) now owns actual
admission/read/revocation/pruning operations around these rules. It supplies its
own selected facts and physical accounting; callers cannot submit a snapshot or
prepared policy result as write authority. The stable backend remains unimplemented
and will need independent persisted-corruption and host-rollback qualification.
