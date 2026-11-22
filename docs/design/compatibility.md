# Compatibility and acceptance

## Current contract and historical provenance

Reference repository: https://github.com/dragginzgame/ic-auth-legacy
(originally named `dragginzgame/ic-auth`).

Reference commit: `e52a40be75dd9809d0528189d84b43fe96952ad9`.
The sibling checkout is convenient for reading; resolve reference files from this
commit, not its moving working tree. `reference.sha256` records selected file bytes.
The reference is historical provenance only. The maintainer authorized the hard
cut in [Auth #2](https://github.com/dragginzgame/ic-auth/issues/2): no existing users or
deployments need pre-recode Wire/API compatibility. No check executes that source
or requires its checkout. Current contracts and independent fixtures own acceptance;
method identities and layouts may evolve together with their current consumers.

## Current safety and interoperability requirements

- Explicit current Wire identities/layouts, byte/allocation/node/depth admission,
  error projections and Rust/TypeScript interoperability; no historical parity gate.
- Canonical signing domains and bytes, claims schemas 2 and 3, strict ordering and
  rejection behavior. Never sort untrusted signed inputs into validity.
- Credential grant bounds, browser-key/caller binding, scope narrowing, device
  fencing, protected authority generations and exclusive validity deadlines.
- Both proof mechanisms, strict signature policy and trusted enrollment inputs.
- Exact retries, replay retention across logout, interrupted issuance/renewal
  recovery and durable browser logout uncertainty.
- Bounded abuse budgets, failed-attempt charging and no mutation on applicable
  rejection paths, as established by the current contracts and tests.

Checkpoint v1 requires a predecessor that sealed the current Auth state. Each
owner verifies its format and protected configuration before reopening. Hosts
must retain their own authority entries, terminal revocations and certification
contributions. Volatile loss remains closed. See the
[stable checkpoint contract](DESIGN.md#stable-upgrade-checkpoints).

## Evidence required before replacement claims

Use the same behavioral fixtures in both modes. Installed upgrades and failed
upgrade rollback additionally exercise the stable allocation and lifecycle hooks.

| Boundary | Required evidence |
| --- | --- |
| Encoding | Independently assembled current bytes; Rust/browser decoding and generator output comparison |
| Verification | Valid and adversarial vectors for each selected mechanism, including wrong authority/caller/time |
| Backend semantics | Same behavioral contract on volatile and stable implementations; failed commits and exact retries |
| Lifecycle | Stable reopen/upgrade, corruption refusal, generation changes, replay/logout retention |
| Runtime | Native and actual Wasm host qualification; one coordinated executor/guard |
| Browser | Real SDK identity transport, Rust/TypeScript interoperability and interrupted logout recovery |
| Costs | Selected graph/size observations and comparable populated-store work; retain existing accepted ceilings |

## Current evidence by boundary

Evidence below applies to the indicated boundary, not to the full replacement.
Native, installed and browser checks complement one another. Retain independent
contract cases; do not copy every legacy test or delete tests to meet a size goal.

| Boundary | Evidence and owner | Limit |
| --- | --- | --- |
| Request and claims | [Request](../development/request-boundary.md) hash/TTL and scope normalization; [claims](../development/claims-boundary.md) independent signed bytes/digests and bounded rejection | Codec/policy evidence alone installs no session authority |
| Wire | [Wire comparison](../development/wire-boundary.md): four contracts, 37 independently assembled frames, 296 retained resource expectations and current generated browser interoperability | Current corpus coverage, not historical parity or production endpoint integration |
| Ed25519 | [Verification](../development/verification-boundary.md): independent signature and strict context/refusal/charging cases under the supported graph | Platform owns crypto; consumer graph remains an integration responsibility |
| Certified proof | [Issuer signature](../development/certified-issuer-boundary.md), [delegation bytes](../development/delegation-encoding-boundary.md), [root verification](../development/root-delegation-boundary.md) and [complete verification](../development/certified-verification-boundary.md): real BLS/OpenSSL fixtures, root/issuer composition and shared work budget | Submitted projection fields never establish enrollment provenance |
| Receiver policy | [Private policy](../development/receiver-policy-boundary.md): exact retry, immutable renewal/replacement, role narrowing, captured leases and physical capacity arithmetic | Fact drivers alone do not establish storage or transaction guarantees |
| Memory receiver | [Memory boundary](../development/memory-receiver-boundary.md): real proofs for both mechanisms, two issuers/devices/receivers, quota isolation, logout fences and proof-expired retry | Stable admission/reopen and complete consumer lifecycle remain separate |
| Browser | [Client](../../packages/client/README.md): native IndexedDB, real SDK signing, generated frames and one HTTP transport | Fixed installed journey is qualified; production integration/lifecycle remain open |
| Installed host | [Host qualification](../../qualification/ic-auth-host-conformance/README.md): real single-subnet Wasm, certificates, rollback and upgrade-loss refusal | Standalone Platform/Wire fixtures, not complete Canic integration |

The memory receiver's independent model compares 353 traces/7,307 operations and
rejects a deliberately broken write-then-error adapter. Native unwind injection
covers eight mutation paths and requires terminal closure, not rollback or recovery.
Inspection checks the 32-result/128-row bounds, one lookahead, exclusive cursors and
no mutation. Borrowed-receipt compile-fail checks and real-proof synchronous
application projections establish the native API boundary; copied receipts do not
carry authority across an await. Encoded-backend corruption admission remains a
future storage obligation.

The volatile issuer checks account-scoped intent identity, capacity reservation
before effects, nonce/approval binding, original windows, sequential browser
renewal, lookup-only unknown outcomes and exact-result reconciliation. Independent
Node/OpenSSL claims/signatures and Wire-derived renewal identity vectors preserve
reference bytes. Indexed pruning has a 32-row ceiling, reference-protected logins,
deterministic expiry ties, exact quota release and consumed-ID epoch closure
separate from restart fencing. Native fault injection covers partial publication,
pruning and restart. Certified completion checks synchronous leaf/root publication,
stale authority/seed refusal and exact retries without republishing. A Platform BLS
fixture verifies the same frozen claims with fresh certificates. Result-byte
reservations are logical native bounds, not equivalent serialized-store or heap
accounting.

Browser-bound issuer revocation has four native contract cases covering every
selector, other-account/login isolation, pending initial/renewal effects, retained
results/deadlines, clock/closed-state refusal and actual pruning. The installed
combined issuer/receiver journey uses this public operation and rejects an
account-signed request to its browser-only logout endpoint. This does not establish
a mounted Canic issuer-logout contract or external signing rollback.

## Installed transaction and enrollment evidence

Five single-subnet cases exercise issuer leaf/system-root/completion rollback,
receiver admission/logout/application rollback, ordinary bad-proof work charges,
proof-expired retry and retained revocation. Actual upgrades leave lost volatile
issuer/receiver state closed, including after another enrollment request. An intact
companion accepts the same valid credentials, showing that expired unknown denial
is neither durable recovery nor conclusive logout.

A real IC call to an installation-bound authority checks caller/challenge binding,
expiry, substituted keys, revision regression and terminal single-key revocation.
Callback traps restore receiver metadata and host entries together while preserving
the earlier request submission. Refresh cannot extend captured session leases.
This fixed-domain test exchange does not qualify complete Canic registry wiring,
production challenge/refresh scheduling or late/overlapping replies. Consumer
coverage is recorded separately below; it is not a result of Auth's fixture.
External signer recovery, complete application certification composition and
accepted operating budgets remain open.

The [installed cost observations](../development/installed-costs.md) now cover
issuer renewal/current witnesses, complete certified receiver verification,
independent session/byte/account/global replay limits, 1/8/32 populations,
mixed expiry and maximum scope dimensions. Paired controls check refusal without
identity allocation or inappropriate charging, retained exact-retry leases,
logout fences and bounded pruning. These are fixture observations with explicit
artifact identities and intervals, not accepted production budgets or maximum-load
qualification. Keep the detailed measurements with their existing owner.

## Pinned Canic consumer evidence

Read-only review of Canic **0.219.0**, commit
`3d2ef2719450e57484638b4f1db6cb6bcc41d64c`, identifies an implemented consumer
bridge, not a missing Auth API. Its [qualification documentation](https://github.com/dragginzgame/canic/blob/3d2ef2719450e57484638b4f1db6cb6bcc41d64c/qualification/README.md)
reports installed Root enrollment, issuer operations/browser-bound logout and a
Chromium profile journey with a separate issuer, two Roots and two receivers.
Its Rust supplier is registry Auth 0.47.2 with protocol-types/Wire 0.43.0;
the private browser source remains pinned to Auth 0.47.1 commit
`a05b5ddd4099a3f64b95deb173a0c16746d98ac5`.

The [Canic adapter](https://github.com/dragginzgame/canic/blob/3d2ef2719450e57484638b4f1db6cb6bcc41d64c/crates/canic/browser/src/issuer.ts)
consumes generated issuer contracts, captures original selectors and signer roles,
preserves canonical credential material and explicitly maps deadlines. Missing
lookup evidence cannot trigger issuance fallback. Issuer revocation translates
its selectors while receiver logout retains the unit request. Auth still owns
the journal, signing lifetime, readiness and cleanup; no Canic dependency or
endpoint installation belongs in this repository.

The [consumer harness](https://github.com/dragginzgame/canic/blob/3d2ef2719450e57484638b4f1db6cb6bcc41d64c/qualification/canic-qualification/tests/auth_browser.rs)
checks committed signature/profile-write counts and retained application backing
with closed Auth after upgrade. The documented browser cases cover renderer-loss
reconciliation, independent logins, owner isolation, renewal, interrupted cleanup,
Unconfirmed replies, cross-tab logout and late issuance. Enrollment cases separately
exercise overlapping refresh, expiry across awaits and authority loss.

The 0.219.0 profile witness uses local test keys and Ed25519 credentials. Its
application data survives an upgrade while its volatile Auth sessions close.

Canic **0.238.0** added opt-in stable receiver and issuer storage for fresh
installations. Its installed Ed25519 regression checks that a failed reopen
rolls back, a live session resumes after fresh enrollment, and logout and replay
refusal survive a second upgrade. It also checks reconciliation of completed
and unresolved issuer operations and rejection of a stale callback. This
handoff records inspection of the pinned consumer test. [Canic #50](https://github.com/dragginzgame/canic/issues/50)
and [installed regression](https://github.com/dragginzgame/canic/blob/9f5b1f51fa81d8d4e18f9078568c097e6c1f8bf0/qualification/canic-qualification/tests/auth_stable_tests.rs).

## Browser evidence and remaining acceptance

The Node suite checks ten SDK signing cases, six independent generated-codec cases,
eight HTTP cases and two public API boundary cases. Strict TypeScript negative
checks reject internal operation stages through consumer handles. Existing native
Chromium groups comprise eight journal, seven issuance, eight issuer-transport,
seven admission, seven cleanup, seven logout and two HTTP cases (46 total).

These cases cover strict durability, two-tab ordering, suspended signing,
lookup-only recovery, original signer/issuer/sequence binding, exact paired evidence,
renewal watermarks, corruption refusal, aborted commits, durable receiver inventory
and stale receipt rejection. Cleanup checks leases, eight-attempt limits, fixed
deadlines, consumed-ticket closure and uncertainty after lost replies or failed
settlement. Fixtures independently specify the frozen v1 profile and retained Wire
frames. The supported opening factory is exercised by the cleanup operation cases;
internal fault/transition qualification imports source modules directly.

The shared HTTP adapter uses Platform certificate verification with explicit trust
configuration, one submission and freshly guarded polling signatures. Native cases
reject HTTP acceptance and invalid certificates without confirming the journal.
Five separate Chromium/gateway cases on one PocketIC subnet accept actual fixture
Prepare/Complete replies and refuse wrong roots, tampering and a valid certificate
for a different request. An untampered completion retry still verifies. This gate
uses raw fixture commands, not production admission/logout endpoints.

Four further installed Chromium cases connect actual volatile issuer completion,
lookup-only recovery after a lost reply, journal evidence, Wire admission, current
host authorization of a protected write and certified logout. The supported browser
API drives the flow; deliberate raw caller probes separately establish host refusal
before admission and after logout. Wrong account/scope refuse, exact retry retains
its lease, and captured credential replay cannot bypass a retained denial. The
native harness independently checks one signing invocation, one application write,
zero sessions, one retained login fence and one issuer operation after the journey.

This is a combined issuer/receiver on one subnet with a fixed test account/browser,
installation-owned enrollment and local Platform signing. Credential expiry mapping
conservatively retains an earlier browser deadline across reconciliation. It does
not qualify external signing, general clock mapping or production endpoints, and
the successful protected write now uses the guarded browser application-call API.

Twelve additional native Chromium cases qualify the per-login coordinator: duplicate
admission coalescing, independent receivers sharing issuance/renewal, the 64-operation
ceiling before journal access, typed-expiry retry bounds, expired-result recovery,
reopening, explicit deadline mapping, metadata-only live hints, cancellation, timeout
capacity release and cross-tab logout fencing. The installed journey now uses this
coordinator for readiness and forced admission, including no additional admission
for a live hint. These receipts are scheduling metadata, not an SDK signer or current
application authority. Seven further native cases check ordinary-call coalescing
without callback coalescing, deadlines before/after signing and results, fixed
destinations, retained signer closure, cross-tab logout, uncertain outcomes,
cancellation, capacity and timeout. The installed successful write uses the same
guard; raw negative probes still check host refusal independently.

Seven scheduler cases cover typed retries across receivers, lost-response resumption,
cancellation, cross-tab leases, exhaustion/deadline uncertainty, pre-logout/storage
refusal and identity changes during waits. The installed journey uses scheduled
cleanup and still independently requires one logout effect and retained denial.

Three further cases terminate renderer processes belonging to a dedicated disposable
Chromium browser, observe its crash event and reopen the same IndexedDB storage.
They cover committed local invalidation, signed cleanup dispatch with no reply and
committed confirmation. Deadlines, counts and issuance uncertainty survive; a live
lease prevents redispatch and confirmed cleanup causes no further call. Identities
are supplied again by the test host: this does not qualify key persistence, whole
browser/OS failure, interruption before commit or external signer recovery.
Four issuance cases use the same abrupt renderer-loss boundary: pending initial
issuance, pending renewal, committed evidence and logout after reopening. Unknown
work stays lookup-only even under a stronger refresh request or a failed lookup;
initial recovery requires the account signer, renewal uses the browser signer,
committed evidence returns without transport, and logout blocks reconciliation.

Seven disposal cases qualify terminal/binding guards, detached retained evidence,
deadline uncertainty, conditional deletion/retry, cancellation and storage/clock
refusal. An eighth renderer-loss case deletes via a host custody test double and
retries after reopening with only the public binding. Actual host key storage,
all-copy erasure and whole-browser recovery are not qualified.

Ten native external-signer cases delay or reject real SDK signing through a host
adapter for account issuance and browser renewal. Rejection, cancellation, timeout,
identity change and logout retain the original sequence and prevent stale signature
release. Reopening stays lookup-only; failed lookup retains uncertainty, successful
lookup saves exact evidence, and late approval after recovery cannot revive an old
attempt. These qualify Auth's boundary, not a real wallet/device or cross-process
signer. Original host account/login binding remains a consumer obligation.

Four admission renderer-loss cases cover suspended signing, submitted/lost reply,
completed reply and logout on reopening. Cleanup inventory survives every boundary;
reopening does not submit, explicit admission reuses exact evidence without issuer
calls, and logout fences admission while cleaning both issuer and receiver. Requests
are observed outside the renderer by a transport test double; this adds browser
recovery evidence, not remote execution or certificate authentication evidence.

Three multi-receiver browser cases create an issuer/three-receiver inventory through
public admission and combine confirmations, unconfirmed replies and transport loss.
They qualify cross-tab lease handoff, refusal of a superseded response, exact
per-destination attempt exhaustion and deadline completion retaining unresolved
receivers. Transport replies are test doubles; deployed multi-canister coordination
and certificate authentication remain consumer/installed acceptance boundaries.

The pinned Canic witness above supplies a concrete Ed25519 multi-receiver consumer
path. Still required are production cutover/key custody, external signer recovery,
certified consumer integration and accepted operating budgets. No application operation may be
replayed to obtain that evidence.

## Historical entrypoints

These paths document provenance at the pinned commit; they are not a required
checkout, executable oracle or compatibility acceptance gate:

- `docs/design/0.6-auth-protocol.md`: signed protocol and trust contract.
- `docs/protocol/sessions.md`: session, replay, revocation and storage obligations.
- `docs/integration/persistent-auth.md`: host transaction and stable-store custody.
- `docs/protocol/browser-client.md`: browser lifecycle and uncertainty handling.
- `crates/ic-auth-wire/src/`: actual Wire definitions.
- `crates/ic-auth/tests/` and `qualification/ic-auth-backend-conformance/`:
  independent behavioral evidence to classify and retain.
