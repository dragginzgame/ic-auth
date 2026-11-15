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

`make test-receiver-policy` runs seven independently written fact scenarios and
one qualification driver using the existing independent Ed25519/root/BLS fixtures.
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

Contracts were inspected at reference `e52a40be75dd9809d0528189d84b43fe96952ad9` in
`session/mod.rs`, `session/model.rs`, `portable/revocation/mod.rs` and
`docs/protocol/sessions.md`. Their hashes are recorded in `reference.sha256`.
The policy and scenarios are newly written with those source contracts available;
no legacy storage engine or record decoder is imported. This is not a clean-room
claim or full differential backend qualification. `ic-storage` remains interim;
`ic-storage-new` adoption still requires its own lifecycle and rollback evidence.
