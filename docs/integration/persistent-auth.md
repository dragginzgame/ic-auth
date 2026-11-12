# Auth-owned persistent authentication

The optional Engine `persistent` capability provides `IssuerStore` and
`ReceiverStore` over published Storage collections. This is the implementation
boundary selected by [Auth #132](https://github.com/dragginzgame/ic-auth/issues/132).
Consumer adoption and deletion remain tracked by
[Canic #764](https://github.com/dragginzgame/canic/issues/764).

These stores do not install endpoints, allocate a stable-memory manager, obtain a
clock, select trusted authority or publish a certification root. The host supplies
those boundaries. Protocol/transport-only and ordinary session consumers do not
select Storage or Runtime. On Wasm, Storage's backing adapter transitively selects
Runtime IC0; the optional library backend selects no executor, guard or CDK. The
actual host composes one coordinated Runtime identity and its executor/guard.

## Memory and transaction custody

Declare two fixed governed allocations for an issuer: ordered records and a small
metadata Cell. Pass their exclusive `RuntimeMemory` leases as `StoreMemory`.
The receiver takes `ReceiverMemory`: that pair plus a separate caller table.
This keeps ordinary authentication out of the larger maintenance/replay index.
Allocation count is independent of user, login and operation counts. Metadata
writes do not traverse the growing record map. The host composes these leases
with its other storage and retains its one lifecycle/certification owner.

`create` requires both allocations to be empty. `open` requires the current
schema, the exact protected configuration and consistent physical counts. Neither
path resets foreign or missing state. Opening reads bounded metadata; it does not
rebuild a heap mirror. Each selected record and the indexes used by an operation
are checked at use. This is not an exhaustive forensic scan or protection against
coherent rollback of host-owned safety state.

Run an entire operation synchronously, serialized with authority/configuration
changes. All returned record/capacity refusals precede publication. Trusted clock
observations and failed-attempt budget charges intentionally survive ordinary
refusals. Writes are pre-encoded before publication. Collection IO, allocation
or codec faults trap: IC message rollback supplies atomicity. Native hosts need an
equivalent transaction boundary and must discard a poisoned runtime after a
mutation panic; catching a panic and continuing is unsupported. The stores do not
provide a general native filesystem transaction journal.

## Issuer custody

`prepare` accepts the runtime-authenticated account and exact requested intent,
protected lifetime policy and signing-authority selection. It reserves a readable
completed result before randomness, authority approval or signing. The host must
not project client fields as trusted authority. Canic's Fleet/Root grant approval
and secret key custody remain its authority adapter's responsibilities.

`challenge` retains one nonce; `approve` retains one host-verified response
commitment. Conflicting callbacks refuse. `claims_for_signing` returns the original
canonical claims only after these stages. `complete` retains the original local
signature or certified claims. Certified witness construction and root publication
compose through the existing host signature owner. These APIs do not authenticate
an arbitrary signature supplied by a client or create a new cryptographic verifier.

Login, caller/operation and renewal-sequence indexes are point lookups. Device
selection examines at most eight slots belonging to the selected account. Global
and per-account operation counts, physical login counts and completed-row byte
reservations are incremental. The same account cannot create unlimited retained
logins. Configure the per-account operation limit below the global limit so one
account cannot consume the entire operation table merely by choosing new IDs.
Expired rows still count until explicit pruning.

Renewal authenticates the original browser binding and preserves its original
login deadline, slot and generation. Sequence watermarks prevent recreation after
result reclamation. Retrieval returns original reconciliation evidence, including
expired results; it never extends a credential or authenticates an application
operation. Issuer revocation stops further signing/renewal; receiver revocation
and existing receiver leases have their own enforcement boundary.

Expiry indexes and login reference counts avoid cross-history cleanup scans.
`prune` removes at most `min(budget, 32)` records. Forgetting current-epoch consumed
IDs and closing that epoch share one commit. A login with retained operation work
cannot be reclaimed prematurely. `restart` explicitly fences old callbacks;
completed results remain readable and no private signing key is restored.

## Receiver authentication

`admit` invokes the existing complete proof/session engine with one canonical
claims decode. It retains session, replay, login and accounting changes atomically.
The verification and unknown-logout budgets are separate, persisted receiver-wide
buckets. Reopening does not replenish them.

One caller record contains either an active session or a retained fence. The
active session already carries its reserved login binding, so it is stored once;
removing that session materializes the fence in the same atomic transaction.
`authenticate` reads that record once and checks the current device
fence, protected issuer/audience/key/generation, expiry, revocation, role and
requested scopes through the existing engine. It invokes the projection with the
checked `Session`; callers must retain the identity bindings their application
needs, rather than treating an unqualified Boolean as an identity. No credential
verification result is cached between application calls. Applications separately
check resource ownership and recheck authorization after relevant awaits.

Logout removes only that caller's session and preserves replay history. Verified
higher-generation replacement removes the displaced caller atomically. Known slot
fences survive expiry and logout; account retirement still needs an explicit
protected host policy. Bounded pruning removes expired sessions/replays and
unknown denials. Generation changes must share the host transaction with changes
to protected authority.

## Format and adoption

This is a new optional backing layout, not a reader for Canic's issuer journal,
receiver vectors or earlier Auth checkpoints. Current Auth session/replay/login
payload identities and signed schema-2 claims remain unchanged inside the new
receiver layout. No deployed consumer records are reset by these changes.

Canic has approved an explicit reinstall-only release cut. Its adoption must
supply the leases, map authenticated host inputs, wire lifecycle/authority changes,
remove its displaced engine and qualify its actual endpoints. Other consumers
must decide their retained-state migration explicitly. Do not describe a passing
Auth fixture as completed Canic adoption or production key-custody qualification.

## Qualification scope

The maintainer accepts the measured 1,223,140-instruction cold authentication at
100,000 receiver records as an interim cost. The installed regression ceiling is
1,300,000 instructions, including the post-upgrade lookup. Auth remains on
published Storage 0.73.4; Storage locality optimization is a follow-up, not a
dependency of this batch. This is ordinary protected-call overhead, not a
one-time login cost. It does not waive correctness or lifecycle acceptance.

The focused native tests exercise 100,000 retained issuer logins, selected-record
IO, bounded pruning, capacity/byte refusal, retries, renewals, device limits,
revocation and reopen. Receiver scenarios use real Ed25519 credentials and the
existing session engine, including replacement, logout, replay and authority
invalidation. Installed probes measure individual issuer bookkeeping and ordinary
receiver authentication separately; their controller-only seeding endpoints are
qualification code, not production issuer or authentication endpoints.

The issue stays open until the remaining issuer enrollment/key-effect custody,
full installed certified and Ed25519 lifecycle acceptance, corruption/interruption
coverage and consumer handoff have been reconciled. Evidence and actual measured
acceptance belong in the [current handoff](../status/current.md).
