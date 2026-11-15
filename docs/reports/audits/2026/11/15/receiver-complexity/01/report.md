# Volatile receiver complexity comparison

The replacement has a demonstrably simpler volatile state owner, but a large
reduction in equivalent-feature production code is not yet established. The
new receiver combines two identity stores and removes a duplicate device-slot
index. Its smaller source also omits legacy restore, custom commit and refusal
projection capabilities whose disposition is still open.

Verdict: **PASS WITH FINDINGS** for this bounded structural review. This is not
a security, behavioral-equivalence, performance or replacement-readiness verdict.

## Scope and identities

- Trigger: maintainer concern that the rewrite will reproduce legacy complexity
  and LOC; feature expansion paused for this review.
- New source: `421fb765780c2daed069cbfe1d0aeb009fce34af`, plus the existing local
  Auth 0.46.0 certified qualification additions. These additions change test
  coverage, version metadata and documentation, not receiver behavior.
- Legacy: clean checkout at `e52a40be75dd9809d0528189d84b43fe96952ad9`.
- Method: [complexity and technical debt](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/audits/complexity-and-technical-debt.md),
  with its common audit contract and simplicity guidance at that same revision.
- Product authorities: local `AGENTS.md`, `docs/design/DESIGN.md`, authentication
  and compatibility designs. Their volatile-first scope and signed/Wire contracts
  govern this review. This is a first, provisional scope, not adoption of a recurring
  audit overlay or comparison with a previously qualified audit.
- Inspection and source counting only. No implementation, supplier, publication
  or legacy checkout changes. Issuer/browser implementations, stable backends,
  cryptographic internals and deployed execution are outside this structural review.
  Their missing replacement evidence remains open.

[Measurements](measurements.json) record source file hashes, overlay and reporter
identities, source classifications, diagnostics and the pre-existing dirty scope.
The reporter comes from installed Toko Tooling 0.44.4. Rust is 1.100.0; Node is
24.21.0. Counts include all source configurations, not a selected compiled graph.

## Source counts and comparison limits

| Rust source category | New local 0.46.0 | Pinned legacy |
| --- | ---: | ---: |
| Product library/application | 4,241 | 8,653 |
| Tests and qualification | 8,379 | 23,036 |
| Examples | 91 | 0 |
| Benchmarks | 0 | 462 |
| Tooling | 0 | 285 |
| Total | 12,711 | 32,436 |

Both totals use the **same installed reporter**, across product and qualification
workspaces, counting shared source once. This reporter measures Rust. It excludes
the legacy TypeScript browser package, JavaScript fixture generators, data fixtures,
documentation, dependencies and generated build output. The earlier conversational
22–27k forecast mixed browser additions with this Rust-only basis and is withdrawn.
No replacement total is forecast by this audit.

The selected receiver-related production modules total **1,769 new versus 2,278
legacy LOC**, a nominal 509-line/22.3% reduction. The exact file roster is in the
measurements. New scope includes receiver memory/policy, verification budget and
shared bucket arithmetic. Legacy scope includes session modules, login revocation,
admission budget and shared storage codec. Both exclude proof verification, claims,
protocol types, request normalization and Wire contract definitions.

This is a module-footprint comparison, **not an equivalent-capability saving**:

- Legacy includes checkpoint restore/admission, custom commit interfaces and
  refusal projections. New code has no corresponding full capabilities yet.
- New code includes 139 production lines of enrollment metadata admission and
  77 of inspection. Legacy volatile enrollment is host-owned; its inspection
  implementation is in the excluded persistent backend.
- Legacy storage-codec helpers have consumers beyond the receiver. New logical
  record-size projections preserve those byte ceilings without implementing storage.
- Tests are classified separately. Different test packaging and missing installed
  qualification prevent interpreting the test-size difference as duplication removed.

The six new and eighteen legacy reporter diagnostics concern conditional code,
macro source counting and shared-source attribution. Macro expansions are not
counted. These are source-footprint observations, not binary-size measurements.

## State ownership and operation traces

| Maintained responsibility | Legacy volatile path | New volatile path |
| --- | --- | --- |
| Identity stores | `MemorySessionStore` plus `MemoryLoginRevocations` | One `MemoryReceiver`, with `CallerRecord` holding the login and optional session |
| Ordered maps/sets | Nine across those two stores | Seven inside the receiver |
| Device-slot indexes | Active-session index and retained login-fence index | One retained slot index |
| Work budgets | Separate host-passed proof and unknown-logout budgets | Same two semantic budgets, privately retained by the receiver |
| Login binding | Stored in both session data and revocation record | Stored in the caller's login; session facts borrow/project it |
| Commit coordination | Engine → ledger reservation callback → store admission → optional host commit callback → publication in both stores | Receiver preflight → complete verifier → publication of affected records; partial native publication closes the receiver |

Evidence: legacy `session/mod.rs::establish_decoded_session`,
`portable/revocation/mod.rs::with_reservation` and `commit_record`,
`session/memory.rs::admit_with_commit`; new `receiver_memory/mod.rs::admit`,
`records.rs::CallerRecord` and `receiver_policy/login.rs::assess_login`.

The two removed collections are the separate session caller map and its separate
active-session slot index. The surviving seven collections still have distinct
lookup, fence, replay, quota or bounded-maintenance obligations. Do not remove
them merely to lower a collection count.

Legacy already decoded claims once, shared an admission flow between mechanisms,
used bounded selected reads and avoided whole-store clones. Those properties are
preserved, not credited as new simplifications. Legacy's persistent adapter builds
a temporary ledger from at most two selected records; the replacement has no
durable adapter, so eliminating that transformation has not yet been demonstrated
for an equivalent stable path.

## Behavior axes and decision spread

| Axis and admitted states | Current semantic owner | Interaction or invalid boundary |
| --- | --- | --- |
| Ed25519 or certified proof | Protected `VerificationProfile`; mechanism verifiers | Submitted proof tag cannot select authority; both feed one receiver path |
| Fresh, ready, closed; pending/accepted enrollment with freshness and generation | Receiver lifecycle and private enrollment module | Admission/read need current metadata; close is terminal; metadata is not response authentication |
| Absent caller, known binding, unknown denial, optional session, retained denial | Private caller record plus login/logout/read policy | Fresh proof alone cannot erase a denial or replace a slot without a greater generation |
| Exact retry, fresh admission, renewal, replacement | Retry/login policy, orchestrated by `admit` | Request mismatch, consumed replay and changed immutable binding refuse |
| Capacity and work credit | Capacity policy and two retained buckets | Expired rows still occupy capacity; proof and unknown-logout credit remain independent |
| Host-controlled time, authority, eligibility and transaction | Host inputs; library independently checks structure/liveness | Public fields do not establish provenance; copied receipts do not authorize later work |

These constrained states are not multiplied into a synthetic complexity score.
Diagnostic pages, fixture cases and test hooks are maintenance costs, not product modes.

## Three change rehearsals

| Plausible change | Semantic edits and boundaries | Result |
| --- | --- | --- |
| Change account replay quota policy | New `receiver_policy/capacity.rs`, with `MemoryReceiver::admit` consuming its result. Legacy `SessionLimits::check_capacity` is also shared by engine/store. | Both have a central decision; no new simplification claim for this rule. Backend mutation and boundary tests remain necessary. |
| Change unknown-logout retry charging | New `plan_logout` owns classification and returns `charge_unknown`; receiver owns clock, budget and publication. Legacy ledger stages the record and coordinates the session store through a callback. | New ownership is narrower. Keep the charged-attempt and unconfirmed-result semantics distinct. |
| Add a future durable receiver | Shared pure policy is reusable, but publication, corruption admission, rollback and potentially orchestration still need a real supplier-backed implementation. | Current single-backend architecture has not proved cross-backend convergence. No abstraction is added by this rehearsal. |

## Findings and dispositions

1. **MEDIUM — Public policy assembly exceeds demonstrated product use.**
   `receiver_policy` publicly exposes `ReceiverSnapshot`, `LoginSnapshot`,
   `DeviceFacts`, `AdmissionFootprint`, decision enums and assessment functions.
   Repository call-site inspection finds one production consumer: the memory
   receiver; external calls here are tests/qualification. These public fields
   make a receiver representation change an API coordination problem before a
   custom backend has demonstrated the need. This is ownership/surface debt,
   not an authentication bypass: the functions explicitly establish no authority.
   Owner: Auth receiver API. Disposition: **fix when touched**, before first
   downstream adoption/publication or the next receiver API change. Separate the
   legitimate host inputs, limits, errors and inspection facts from internal
   assembly. Internal tests can exercise private policy; do not introduce a public
   testing facade. Actual downstream usage must be checked before removing APIs.

2. **MEDIUM — Full savings depend on unresolved capability disposition.**
   Legacy custom commit/restore APIs and host recovery obligations are not replaced
   by closing a poisoned memory instance. Counting their absence as removed debt
   would overstate the result and could lead to reintroducing the same layers later.
   Owner: Auth integration/design. Disposition: **accept until the consumer-contract
   inventory**, which must resolve each API before implementation or deletion.
   Installed host qualification and durable adoption remain separate gates.

3. **LOW — Compatibility byte accounting retains a second field description.**
   `records.rs::SessionSize` mirrors the reference record solely to measure the
   logical quota charge; `CallerRecord`/`Session` remain the real state. This is
   mechanical representation maintenance, not a second authority. Owner: memory
   records/accounting. Disposition: **accept until a record-field or accounting
   change**. Preserve the byte contract and independent size tests; assess a
   smaller projection only at that trigger. Do not add a codec framework or change
   accepted quotas merely to delete this description.

## Constraints for subsequent implementation

The companion design now requires an owner/transition map and an explicit retained,
removed or deferred legacy capability mapping before issuer/browser expansion.
Issuer state has one operation owner; browser coordination has one journal owner.
Use existing Platform identities, Wire generation and host transaction ownership.
Do not add a public backend framework, compatibility mode or adapter solely for
tests or a speculative future consumer. Keep pure rules reusable without exposing
their assembly as a supported extension contract.

Track production Rust, Rust test/qualification and browser TypeScript separately.
Evaluate equal capabilities and concrete ownership reductions. Preserve independent
fixtures, negative controls and required recovery evidence; test deletion needs a
specific redundant-contract mapping. This report is evidence, not a parallel backlog.

## Verification and limits

Executed `toko-cloc --json` and `toko-cloc -d 1` for each workspace, using the
new repository's reporter. The first legacy count attempted its default scratch
directory under read-only legacy `target/tmp` and failed. Repeating with
`TMPDIR=/tmp` succeeded without modifying legacy. Both reports have zero unclassified
Rust LOC. Source inspection verified map/index ownership, admission/publication
traces and assessment-function callers. Existing 0.46.0 behavioral evidence is
recorded in `docs/status/current.md`; it was not rerun or counted as new audit evidence.

No new behavioral tests or performance benchmarks were necessary for these
documentation-only changes. No security or runtime defect is asserted. Full
equivalent-feature size, deployment costs, custom consumer compatibility and
installed lifecycle behavior remain unproved; none receives a PASS here.
