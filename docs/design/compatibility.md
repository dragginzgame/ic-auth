# Compatibility and acceptance

## Fixed reference

Reference repository: https://github.com/dragginzgame/ic-auth

Reference commit: `e52a40be75dd9809d0528189d84b43fe96952ad9`.
The sibling checkout is convenient for reading; resolve reference files from this
commit, not its moving working tree. `reference.sha256` records selected file bytes.
The reference is evidence, not a dependency on the old engine in the new product.

## Preserve

- Wire method IDs, request/reply layouts, tags, byte/allocation/node/depth budgets,
  error projections and Rust/TypeScript interoperability.
- Canonical signing domains and bytes, claims schema 2, strict ordering and
  rejection behavior. Never sort untrusted signed inputs into validity.
- Credential grant bounds, browser-key/caller binding, scope narrowing, device
  fencing, protected authority generations and exclusive validity deadlines.
- Both proof mechanisms, strict signature policy and trusted enrollment inputs.
- Exact retries, replay retention across logout, interrupted issuance/renewal
  recovery and durable browser logout uncertainty.
- Bounded abuse budgets, failed-attempt charging and no mutation on applicable
  rejection paths, as established by the current contracts and tests.

Wire compatibility does not imply stable-record compatibility. No storage format
change is selected by this initialization. Before a backend cut, explicitly
choose preserved current records or a hard cut with legacy refusal; never silently
reinterpret/reset storage or erase a host's data. This repo has no migration authority.

Volatile storage is the current milestone; stable storage is deferred until a
future milestone with a ready supplier contract. No interim `ic-storage` adapter
is planned. `ic-storage-new` with orthogonal persistence remains the intended
durable destination. That future milestone must preserve logical Auth behavior
without assuming package/API or stored-format compatibility, reusing independent
semantic cases and adding retained-state upgrade, schema conversion, incompatible
layout refusal and recovery/rollback evidence. This direction authorizes no import,
reset or migration of a consumer's live state.

## Evidence required before replacement claims

The current milestone qualifies volatile Auth only. Stable reopen, cross-backend
equivalence and persistence conversion below belong to the future stable milestone;
their deferral does not waive volatile loss, host transaction or browser evidence.

| Boundary | Required evidence |
| --- | --- |
| Encoding | Independent retained golden bytes; old/new cross-decoding and generator output comparison |
| Verification | Valid and adversarial vectors for each selected mechanism, including wrong authority/caller/time |
| Backend semantics | Same behavioral contract on volatile and stable implementations; failed commits and exact retries |
| Lifecycle | Stable reopen/upgrade, corruption refusal, generation changes, replay/logout retention |
| Runtime | Native and actual Wasm host qualification; one coordinated executor/guard |
| Browser | Real SDK identity transport, Rust/TypeScript interoperability and interrupted logout recovery |
| Costs | Selected graph/size observations and comparable populated-store work; retain existing accepted ceilings |

Port independent contract tests and vectors before replacing their implementation.
Do not copy every test mechanically or delete tests to reach a size goal. Preserve
unique guarantees and explain redundant coverage removal. The initial
[request-boundary slice](../development/request-boundary.md) has focused scope,
request-hash and TTL evidence; the [claims codec](../development/claims-boundary.md)
has independent signed-byte/digest and bounded rejection evidence. Neither
completes a replacement gate above. The first
[Wire boundary](../development/wire-boundary.md) covers all four contracts with
37 independent reference frames, generated-browser interoperability and separate
reference/current native resource thresholds. This is corpus-based codec
evidence. The [Ed25519 verifier](../development/verification-boundary.md) now has
an independent signature vector, protected context/refusal cases and a deliberately
legacy-enabled scalar negative control. The
[certified issuer component](../development/certified-issuer-boundary.md) has real
BLS certificate fixtures for issuer/seed/network binding and freshness, including
subnet-range delegation controls. [Delegation signing commitments](../development/delegation-encoding-boundary.md)
have independent byte/hash evidence; the [root verifier](../development/root-delegation-boundary.md)
has independent OpenSSL evidence for enrolled authority, grant binding and proof bounds.
The [complete certified verifier](../development/certified-verification-boundary.md)
now has independent root/issuer composition and shared-budget evidence.
The [request policy](../development/request-boundary.md#shared-narrowing-and-lease-calculation)
also checks single-role narrowing and captured lease ceilings through both proof
mechanisms, without claiming session or replay admission.
The [receiver policy boundary](../development/receiver-policy-boundary.md) checks
retained-state read/retry decisions and proof-expired retries. Its fact drivers
are not evidence of backend admission, retention, rollback or host integration.
Login preflight adds immutable renewal, higher-generation replacement and physical
known-login capacity cases; successful classification cannot install authority.
Physical session/replay count and byte arithmetic also have independent boundary
cases. Protected credential dispatch is exercised through Wire/domain drivers;
neither establishes atomic admission or a retained-storage format.
The private browser signing-attempt slice now exercises actual Platform SDK
identities with injected generation checks. Ten Node cases cover fixed caller/
destination, suspended-signature invalidation, late completion, cancellation,
timeout and read-state polling. Thirty native Chromium cases additionally qualify
strict IndexedDB local invalidation, two-tab ordering and suspended SDK signing,
lookup-only issuance recovery, exact completion, renewal watermarks, aborted paired
writes, missing/overflowed sequences and corrupt metadata/evidence. Fixtures
independently specify the frozen private v1 record. Credential tests use retained
reference frames; four Node codec cases and a generation drift check cover the
adopted Rust-generated module/runtime. Issuer adapter cases use actual SDK signing
and injected replies to check account/browser roles, original issuer/sequence
binding, uncertain outcomes and durable completion. Admission cases add exact
generated request bytes, durable receiver inventory, all existing refusal tags,
caller/issuer receipt binding and logout/expiry/sequence fences. Successful reply
fixtures are packed independently from the retained Wire layout. This does not establish HTTP
reply authentication, production endpoint integration, remote cleanup, renderer-
interruption recovery or a complete coordinator.
Seven further native Chromium cases qualify the cleanup journal: strict attempt
reservation across tabs, lost-ticket reload, eight-attempt limits, fixed-deadline
completion without confirmation, retained issuance uncertainty, aborted settlement
and permanent ticket closure around actual SDK signing. The v1 record is unchanged;
confirmation inputs remain injected, so authenticated remote cleanup is still open.
The one-step cleanup adapter adds seven native browser cases for exact Wire dispatch,
all typed nonconfirmations, malformed/oversized replies, cancellation/timeout, aborted
settlement and cross-tab exclusion. Two Node cases compare the generated Logout
codec with retained caller-contract frames. HTTP authentication, automatic scheduling
and key disposal remain outside this evidence.
Complete adversarial acceptance, actual SDK/browser lifecycle and backend qualification
remain outstanding.

Reference entrypoints:

- `docs/design/0.6-auth-protocol.md`: signed protocol and trust contract.
- `docs/protocol/sessions.md`: session, replay, revocation and storage obligations.
- `docs/integration/persistent-auth.md`: host transaction and stable-store custody.
- `docs/protocol/browser-client.md`: browser lifecycle and uncertainty handling.
- `crates/ic-auth-wire/src/`: actual Wire definitions.
- `crates/ic-auth/tests/` and `qualification/ic-auth-backend-conformance/`:
  behavioral evidence to classify and retain.

The first [memory receiver](../development/memory-receiver-boundary.md) now adds
real retained-state admission/read/revocation cases for both proof mechanisms,
plus renewal/replacement and independently calculated logical record-byte bounds.
This supplies focused volatile-backend evidence only. Stable admission,
boot/recovery, installed host rollback and browser acceptance
remain required before replacement claims.

Auth 0.44.0 adds native known/unknown logout sequences, independently signed future
credential acceptance after temporary denial, separate pool/credit accounting and
exact existing Wire result frames. These do not qualify host readiness or recovery.

Auth 0.45.0 checks bound enrollment metadata and native readiness transitions:
response source/domain/challenge, receipt/freshness deadlines, monotonic revision
and generation, exact repeats and conflicting content. Real signed admissions
exercise captured leases, refresh, generation invalidation and safe outage logout.
This is not qualification of IC response authentication, full enrollment-entry
installation, terminal key revocation, persistent reopen or lost-state recovery.

Passive memory inspection retains the reference's 32-result/128-row bounds with
one lookahead, exclusive cursors and empty-page progress. Instrumented consumption,
512 denial records and real Ed25519/certified sessions check bounded work and no
clock/budget/record mutation. Stable read/corruption admission remains backend-specific
and is not qualified by these memory cases.

An independent fixed-scenario ledger now compares 353 generated traces/7,307
memory operations, including reads, retained quota/fence state and work accounting
across two receivers. A write-then-error negative control must fail its state
comparison. This is native semantic evidence for the selected signed fixtures,
not the complete encoded-backend/host atomicity or recovery matrix.

Native publication tests enumerate injected unwind points across eight memory
mutation paths, requiring closed-state refusal after partial publication and a
successful control run. They prove fail-closed custody, not restoration of memory,
actual allocator failure behavior or installed IC rollback. The hooks are absent
from deployed builds. Encoded durable-backend qualification is a future milestone.

The native volatile scenario now includes two issuer-qualified accounts (equal
subject IDs), two devices on one account and two independent receivers. Independent
signed fixtures test quota separation, replacement isolation, failed proof work,
proof-expired retry, renewal deadlines and local logout. Synchronous authorization
projection checks cover both proof mechanisms; callback refusal and borrowed-receipt
compile-fail cases do not establish installed host isolation or SDK await behavior.

Auth 0.46.0 repeats the multi-issuer/device/receiver lifecycle using complete
certified proofs. Independent claim bytes and a second signed root authorization
cover quota isolation, root/issuer failure without displacement, replacement,
proof-expired retry, renewal and local logout. Certified enrollment changes retain
captured deadlines and fences; pre-admission denial creates no identity. These
native cases do not qualify actual boot/callback authentication or host recovery.

The unreleased initial volatile issuer now checks exact account-scoped intent
identity, original deadlines and result reservation before effects. Independent
effect traces exercise pauses, unknown signing responses, lookup-only reconciliation
and stale callbacks after authority changes, revocation or restart.
The retained Node/OpenSSL reference vector checks its canonical claims and real
Ed25519 verifier interoperability. Native fault injection requires terminal closure
after partial publication. Browser renewal now adds actual caller binding,
read-only preflight with rechecks, sequential completion watermarks, original-window
clamping and cross-epoch/expiry/revocation reconciliation. Independent frame/hash
vectors check the retained Wire-derived operation identity, including wide fields.
Pruned result evidence cannot recreate consumed work. These cases
also qualify native indexed pruning: positive captured reconciliation retention,
the 32-record ceiling, operation-first selection, reference-protected logins,
deterministic expiry ties, exact quota release and higher-generation slot reuse.
Real pruning replaces the synthetic missing-result setup. Closed operation epochs
prevent identity recreation while a separate restart fence preserves unrelated
live callbacks. Fault injection closes partial pruning/restart publication;
overflow refuses before deleting evidence. Certified completion now exercises
synchronous leaf/root publication and terminal closure on a host error/unwind or
interruption between completion writes. Stale authority (including seed changes),
wrong mechanism, missing approval and exclusive deadlines refuse before the host
callback. Exact retries never recreate a leaf. A separate Platform BLS fixture
certifies the actual publication input under a composed application/signature
root; two fresh certificates verify the same frozen claims fingerprint. These
are native publication-boundary checks, not installed-host rollback, stable
custody or host authentication/recovery qualification.
Result-byte reservations are a new native logical bound,
not a claim of equivalent legacy serialized-storage or allocator accounting.

Installed issuer evidence now complements those native checks. A Platform/Wire
fixture traps after leaf mutation, certified-root publication and retained
completion, and verifies that actual IC rollback restores all three. A host error
is converted into an enclosing trap; ordinary caller/deadline/revocation refusals
remain nontrapping. Successful claims verify against an actual query certificate.
A real upgrade loses volatile state and leaves the fixture closed without a reset
route. See [setup and scope](../../qualification/ic-auth-host-conformance/README.md).
The same installed fixture now exercises receiver admission/logout and protected
application projection rollback, ordinary proof-refusal charging, expired-proof
exact retry and retained revocation. Real upgrades leave lost known/unknown
logout state closed even when enrollment is requested again. An intact companion
accepts the same valid credentials, including an issuer-ahead credential after its
temporary denial expires. This is containment, not recovery or conclusive logout.
Receiver enrollment now arrives through a real IC call to an installation-bound
authority. An installed single-key responder binds the actual caller and challenge;
malformed bindings, expired replies, substituted keys and revision regression
refuse. Receiver metadata and host entries commit together in the callback:
traps undo callback writes but retain the earlier request submission. A committed
revocation cannot be reversed even by greater revision/generation, and refresh
cannot extend captured session leases. All installed canisters share one simulated
subnet. Late/overlapping-reply integration checks remain outstanding for the
supported single-subnet Canic flow; native semantic checks are retained.
This remains a test-specific exchange with
fixed domain/grants and deterministic challenges. Complete Canic registry/endpoint
integration, production challenge/refresh scheduling, external signer recovery,
composed application certification and stable storage remain unqualified.
