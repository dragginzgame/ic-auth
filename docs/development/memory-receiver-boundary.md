# Volatile receiver admission

`ic-auth` now supplies an opt-in `receiver-memory` module, selected with
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

`read` performs proof-free current-policy checks and returns account facts. The
host must keep its authority/application transaction isolated and recheck after
any await. `advance_generation` invalidates old sessions without resetting state
or credit. `revoke_known` needs authenticated caller/time, not fresh enrollment;
it removes the session while retaining denial, device fencing and consumed replay.
Unknown callers return false and receive no acknowledgement or denial. This narrow
API does **not** implement the complete Wire logout protocol or its separately
budgeted unknown-login denial pool.

## Bounds and accounting

Admission selects only the caller, slot owner, matching replay and account counter.
Maps and indexes are bounded by protected known-login, session and replay counts;
record labels and scopes have their existing bounds. `prune` visits at most 128
expired session/replay entries per call, within the caller's smaller combined work
limit. It never time-prunes known device fences. Actual allocator/index overhead is
separate from logical byte accounting and still needs deployment-cost qualification.

Logical session/replay sizes preserve the pinned reference `SessionData` and
`ReplayData` representations, including the 11-byte frame overhead. The optional
backend uses published IC Wire 0.24.1 `encoded_len` and private borrowed Serde views;
it emits no stored frames, selects no persisted layout and introduces no old-format
reader. The registry index confirmed 0.24.1 as latest non-yanked on 2026-11-15.
Wire and Serde are optional backend edges using the existing root selections.
The reference field order and audience representation are tracked in
`reference.sha256` (`session/model.rs`, `storage_codec/mod.rs`).

## Lifecycle and evidence

`new_fresh` starts closed with empty credit. `activate_fresh` is a one-time host
assertion that authenticated boot enrollment is ready for a genuinely new receiver.
`close` is terminal. These methods do not authenticate boot or recover lost memory.
A host must not replace lost replay/revocation state with an empty receiver and
resume old authority. Recovery and boot challenges remain integration obligations.

Run `make test-memory`. Nine public-API Ed25519 scenarios use six independently
packed and OpenSSL-signed credentials. They cover failed verification without
reservation, renewal/replacement at capacity, replay quota and generation retention,
proof-expired retry, exclusive leases, scope checks, clock rollback, generation
invalidation, byte boundaries, pruning and revocation. The initial signature is
cross-checked against the frozen legacy vector. Logical sizes are independently
summed from reference Postcard fields, including a ten-byte generation boundary.

The certified-only qualification case uses independent root signatures and real
BLS certificate fixtures. Root and issuer failures retain charges without creating
state; success creates a session and proof-expired retry ignores alternate proof
material at full capacity. Existing standalone verifier and combined-profile cases
remain regression coverage. Native lint, docs and Wasm library compilation cover
the selected feature graphs; compilation is not installed-host qualification.

Stable storage, unknown-login denial, boot/recovery orchestration, host transaction
failure injection, browser lifecycle and deployment cost gates remain open. The
interim `ic-storage` backend and eventual `ic-storage-new` orthogonal persistence
must reuse semantic rules while retaining their own corruption and host rollback
boundaries. This increment makes no complete replacement or consumer-cutover claim.
