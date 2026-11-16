# Auth architecture and sediment review

Verdict: **PASS WITH FINDINGS** for a bounded structural review. The rewrite has
kept its main ownership boundaries and removed some duplicated state. It has not
yet demonstrated substantially lower complexity for equivalent functionality.
There is early API and documentation sediment; no broad core redesign is justified
by this inspection. This is not a security, correctness, performance or replacement
readiness certification.

## Scope, identity and limits

Trigger: maintainer requested a pause to check whether development was repeating
legacy complexity. Reviewed receiver admission/authorization/logout, issuer
reservation/renewal/completion, browser journal/signing/admission/cleanup/HTTP,
public surfaces and qualification ownership. Feature work stayed paused.

- Source: `3839e035507666adbe3631591e54d8f43a729fc7`, including the 23 pre-existing
  modified/untracked files of the local 0.46.3 HTTP slice. Product Rust is unchanged
  by that slice. [Measurements](measurements.json) freeze file hashes, the dirty
  roster, tracked-diff hash, product authorities and reporter output.
- Reference: `dragginzgame/ic-auth` at
  `e52a40be75dd9809d0528189d84b43fe96952ad9`; source read with `git show` against
  committed objects. The sibling checkout was not modified.
- Method: [complexity and technical debt](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/audits/complexity-and-technical-debt.md),
  its common contract, simplicity guidance, flow-convergence companion and IC
  application addendum, all at revision
  `626da7cd9626cc2e152594862a2d82ea82ef51da`. Retrieved method hashes are recorded.
- Product authorities: AGENTS.md, TOKO-TOOLING.md, README, authentication,
  compatibility and DESIGN documents. Their hashes identify this provisional
  scope; no recurring audit overlay is adopted. The prior receiver-only audit is
  supporting evidence, not an equivalent whole-product baseline.
- Source inspection and counting only. No new behavioral tests, dependency changes,
  implementation repairs, commits, publication or remote mutations. Existing test
  source establishes what is exercised, not a new test pass. Cryptographic internals,
  deployed consumers, binary size and performance are outside this review.

## Footprint and comparable change

| Rust category | Prior receiver-audit snapshot | Current dirty checkout | Pinned legacy, prior measurement |
| --- | ---: | ---: | ---: |
| Product | 4,241 | 5,418 | 8,653 |
| Tests/qualification | 8,379 | 12,620 | 23,036 |
| Examples | 91 | 91 | 0 |
| Benchmarks/tooling | 0 | 0 | 747 |
| Total | 12,711 | 18,129 | 32,436 |

Current counts use the installed Toko 0.44.4 reporter over both workspaces.
Historical columns are reused from the [receiver report](../../../15/receiver-complexity/01/report.md),
which used the same reporter. Growth since that snapshot is 1,177 product lines and
4,241 test/qualification lines. Current diagnostics flag conditional attributes
and macro source counting; expansions are not counted. No unclassified Rust was
reported. These counts exclude TypeScript, JavaScript, generated build output,
fixtures, dependencies and documents; they say nothing about Wasm size or speed.

Separately, handwritten browser `src` files contain **858 physical lines now and
984 at the pinned reference**, including comments and blanks, excluding generated
files. Exact rosters are in measurements. This is not Toko LOC and must not be
added to the Rust numbers as though the units matched. Legacy already contains
readiness/coalescing, receiver receipt coordination and cleanup/disposal behavior
that the new browser does not yet implement. The small current difference is a
warning against forecasting a large browser reduction, not evidence of a defect.

Concrete structural changes, rather than whole-product savings:

- Receiver: the previous audit's two identity stores and duplicate active-slot
  index remain consolidated in `MemoryReceiver`/`CallerRecord`. Private
  `receiver_policy` now prevents external assembly of internal policy snapshots.
- Issuer: new `Operation` references one retained `Login`; `Phase` carries staged
  nonce/approval/dispatch/completion. Legacy `persistent/issuer/model.rs::Operation`
  repeats request, policy and login fields and has independent optional nonce,
  approval and result fields. The new representation narrows in-memory states;
  it has not replaced legacy serialized corruption admission or durable recovery.
- Browser: legacy `IndexedDbLogoutStore` exposes mutable-record operations used
  by `PortableLogoutClient`. New `LoginJournal` owns both transitions and the
  private transaction callback. `runSigningAttempt` converges bounded signing
  lifetime used by issuance, admission and cleanup. Legacy already had bounded
  calls, checked identities, one journal and a shared HTTP helper; those ideas
  are preserved, not newly invented savings.
- Three product crates remain. Protocol data, Wire contracts and Auth behavior
  have distinct dependencies. No speculative issuer crate, storage framework,
  endpoint installer, crypto implementation or replacement executor appeared.

## State space and decision ownership

| Independent axis / admitted states | Canonical owner | Interaction and invalid-state boundary |
| --- | --- | --- |
| Ed25519 / certified mechanism | Protected verifier profile and Platform primitives | Untrusted proof tag cannot choose host authority; both enter one receiver path |
| Receiver fresh / ready / closed; pending / accepted enrollment | Receiver lifecycle and enrollment module | Host authenticates replies; stale metadata blocks admission/read, without blocking safe logout |
| Caller absent / bound / denied; optional session, replay and device generation | Receiver records and private retry/login/logout policy | Exact retry retains its lease; replacement cannot erase generation or denial fences |
| Capacity / attempted-work credit | Receiver counts and separate proof/unknown-logout buckets | Expired rows remain charged until pruning; no shared bucket that lets unknown logout consume proof credit |
| Issuer reserved / challenged / approved / dispatched / completed | `issuer_memory::Phase`, reached through one `prepare`/completion owner | Ed25519 dispatch is distinct from local certified publication; uncertain external effects are lookup-only |
| Initial / in-flight / completed renewal sequence; epoch and boot fence | Issuer login, renewal index and ticket | Sequence advances only after completion; consumed-ID closure and restart callback fencing are distinct |
| Browser active / cleanup / finished; no / pending / completed credential | `LoginJournal` strict transaction | Logout invalidates and deletes evidence atomically; old generation/sequence cannot install a completion |
| Cleanup pending / leased / confirmed / exhausted or deadline-finished | Journal target counters, durable lease and ephemeral ticket | One cross-tab lease, eight attempts, fixed deadline; timeout or absence never means remote revocation |
| SDK attempt open / closed; cancelled / timed out | `runSigningAttempt` | Recheck around asynchronous journal/signing work; late work cannot reopen the signer |
| Gateway, root and method selection | Consumer host configuration, captured by HTTP adapter | SDK verifies certificates; no automatic root fetch, application replay or alternate crypto path |

These constrained axes are not multiplied into a synthetic combination score.
Test fault modes, independent fixtures and generated codecs are maintenance costs,
not independent product modes. Stable storage is a deferred capability, not a
currently supported configuration axis.

## Entry-to-effect traces and handoffs

| Behavior | Owner derives → carried contract → first protected effect / result |
| --- | --- |
| Receiver admission | Host inputs → one canonical decode, private retry/login/capacity decisions and complete verification → publish affected records, return detached receipt (`receiver_memory/mod.rs:375`) |
| Application access | Current host authority → `read` policy check → synchronous `authorize` callback (`receiver_memory/mod.rs:522,565`); a copied receipt is never continuing authority |
| Issuance and renewal | Account ingress or checked browser binding → shared `prepare` reserves login/result capacity → opaque ticket, nonce/approval, first dispatch; reconcile only projects original evidence (`issuer_memory/mod.rs:235`, `renewal.rs:52`) |
| Certified completion | Approved operation → borrowed `CertifiedPublication` → host leaf/root publication and retained completion in one transaction (`issuer_memory/certified.rs:67`) |
| Browser credential | Journal claim → one issue/renew/reconcile choice → temporary signer, detached reply, paired evidence commit (`issuance.ts:33`, `journal.ts:234,267`) |
| Browser admission | Saved credential → generated request and durable target inventory → guarded update, bound decoded receipt (`admission.ts:34`); no application call is replayed |
| Browser cleanup | Committed invalidation → durable lease and live ticket → guarded update, authenticated typed reply, journal settlement (`logout.ts:26`, `journal.ts:303,345`) |
| HTTP transport | Protected origin/root/method plus scoped signer → Platform agent → request-specific certified reply (`http.ts:18`); no second verifier |

Equivalent mechanisms converge before receiver publication. Issue and renewal
share operation storage and completion. Browser operation-specific guards ask the
journal rather than reclassifying a detached status snapshot. Guard checks repeated
before and after awaits are protective duplication: another tab, clock, generation
or sequence can change during that await. Removing those checks is not a valid
simplification. Early cleanup delay validation before reservation and journal
validation at settlement protect different boundaries; retain both.

Qualification remains independently owned: product manifests do not depend on
qualification. Build owns artifact production, Host FS owns directory access,
Testkit owns the single-subnet instance, Platform owns IC calls/crypto, and
Playwright owns normal browser shutdown. Shared setup lives in
`qualification/ic-auth-host-conformance/tests/support/mod.rs`; it has no stale
prebuilt fallback or replacement build/runtime engine.

The installed fixture surface consists of `auth_fixture_control`/`observe`,
`receiver_fixture_control`/`observe`, and `authority_fixture_configure`/`snapshot`.
Control/snapshot updates and observation queries use bounded Wire codecs;
authority configuration checks the controller, snapshot binds ingress caller,
issuer control binds its retained account, and receiver commands obtain actual
ingress/time. Init creates fresh volatile state; post-upgrade leaves lost state
closed and clears the certification contribution. These are qualification
commands, not production Auth endpoint wiring. Public Auth Wire contracts remain
schemas/codecs; Canic endpoint installation is still a consumer responsibility.

## Findings

### F1 — Browser package exposes two levels of orchestration (MEDIUM)

Classification: public-surface/ownership debt, not a demonstrated authorization
bypass. `packages/client/src/index.ts:1-11` exports both `runSigningAttempt`, the
concrete `LoginJournal` and stage types, and the higher-level retrieval/admission/
cleanup operations. The exported journal exposes claim, guard and completion
methods (`journal.ts:210-345`); the README teaches manual composition alongside
the operations that already compose them. Repository call tracing finds these
stages used by owning modules and qualification, without an application consumer
demonstrating a need for a second public composition route.

Consequence: adopting both levels would make consumers responsible for preserving
the same reservation → signer → reply → settlement ordering as Auth. Changing that
ordering would then require coordination across internal operations and external
callers. This is the same kind of boundary expansion that the private receiver
policy cleanup was intended to avoid. Trusted adapters are legitimate; making
every internal stage a supported integration API is a separate decision.

Owner: browser package. **Disposition: fix now under separate implementation
authority**, before adding a readiness API or first downstream adoption. Select
the supported consumer operations; keep stage/attempt machinery internal unless
the first real consumer proves a specific need. Tests may import internal modules.
Do not add a generic facade/backend framework simply to hide methods, and do not
delete the internal safety stages. Package privacy makes narrowing feasible;
absence of repository consumers is not proof that no external experiments exist.

### F2 — Current status has become an incremental journal (LOW)

Classification: stale documentation surface. `docs/status/current.md:94-95` says
actual HTTP reply authentication and remote cleanup remain unqualified, followed
by later paragraphs documenting cleanup dispatch and actual certified HTTP
acceptance. Earlier counts of thirty browser cases and later additions require
readers to reconstruct the current state. `docs/design/compatibility.md:88-125`
similarly interleaves successive evidence slices. Some exclusions correctly refer
only to the preceding slice, but the current-status wording is ambiguous globally.

Consequence: readiness can be under- or overstated depending on which paragraph a
reader uses. This is concrete sediment even though the later paragraphs correctly
distinguish raw HTTP from complete production integration.

Owner: repository documentation. **Disposition: fix now under separate
implementation authority**, before the next source checkpoint. Rewrite current
status as one concise capability/evidence/remaining-boundary summary. Keep detailed
historical results in existing evidence documents and changelog; do not create a
second backlog or another status file. Compatibility should state the present
acceptance boundary without requiring arithmetic across incremental prose.

## Material gaps and justified retention

These are explicit scope limits, not additional findings or claims of regressions:

- **Complete browser/consumer handoff is unproved.** Installed HTTP uses
  `auth_fixture_control` and an empty journal guard
  (`tests/browser/installed-http.mjs:20,34`). This is appropriate for its raw
  transport scope. It does not connect actual issuer response mapping, journal,
  admission, ordinary protected application effect and logout through consumer
  endpoints. The next integration work should prove that chain on one subnet;
  another isolated transport PASS would not close it.
- **Browser readiness and cleanup lifecycle remain incomplete.** Legacy
  `portable-admission.ts:120-256` contains coalescing/readiness/receipt behavior;
  `logout.ts:220` includes cleanup scheduling and disposal. Their absence here
  is deferred work, not a simplification credit. Keep receipts as metadata and
  reuse the existing journal/signing owners when completing the required flow.
- **Stable storage remains deferred.** The memory design has not proved future
  backend convergence, corruption admission or durable host rollback. Keep pure
  policy reusable, but wait for the real OP contract before choosing an adapter.
  Legacy restore/custom commit obligations also still require consumer disposition.
- **Installed enrollment late/overlapping replies and production Canic registry
  integration remain open.** Native semantic evidence and standalone fixtures do
  not qualify that entire supported single-subnet flow.
- **Forced browser descendant cleanup is not established.** The HTTP harness
  uses Host Process `Ownership::Direct`; Playwright closes Chromium normally.
  Killing Node before `finally` is a separate ownership/proof limit. Resolve it
  through supplier capabilities before relying on this as unattended recurring
  CI; no local process supervisor is justified by this audit.
- **Keep independent evidence.** Native model/negative controls, fixed signed
  bytes, generated Wire cross-decoding, real IndexedDB behavior and installed
  rollback/certificates prove different boundaries. No duplicate-test deletion is
  justified merely by the 12,620-line test footprint. Generated files are supplier
  output, not a reason to hand-write a smaller codec.

## Three change rehearsals

| Plausible next change | Expected owner and affected boundaries | Architectural result / blocker |
| --- | --- | --- |
| Add required readiness and guarded ordinary calls | Browser coordinator uses `retrieveCredential`/`establishSession`; journal remains generation/sequence owner, SDK attempt remains signing owner; Canic owns receiver application guard | Must decide F1 first. A second durable readiness journal or policy inferred from cached receipts would introduce duplicated authority. Actual consumer flow is missing proof. |
| Complete logout resumption and key disposal | Small lifecycle driver consumes `cleanupOnce` wait/settled/finished results; journal owns attempts/deadline, host owns keys | No new cleanup state machine in the driver. Finished-with-unconfirmed remains distinct from authenticated revocation; disposal timing needs explicit qualification. |
| Adopt eventual OP storage | Supplier-backed backend, pure semantic policy, authenticated host transactions and corruption admission | Cannot select an API before Storage contract/evidence exists. Do not turn current memory structures into a speculative generic store or claim a zero-cost backend swap. |

None is authorization to implement a new capability. The no-build choice is to
preserve the present owners, narrow the API boundary and prove the existing pieces
in one consumer flow before adding further modes or abstractions.

## Checks and disposition

Executed: pinned-object/source inspection, call-site searches, manifest boundary
review, installed `toko-cloc --json`, hash/roster recording and report integrity
checks. The first Node evidence collection encountered a sandbox subprocess
`EPERM`; direct read-only Git commands followed by filesystem-only Node collection
succeeded. A report-path inventory found no existing 2026-11-16 report to overwrite.
No new behavioral execution was necessary to support these structural findings;
this report does not renew previous test or CI claims.

Only this report and its measurements were added. Existing working changes were
preserved. Findings are immutable review evidence, not a new active local tracker;
any accepted implementation follow-up belongs in the owning GitHub issue workflow.
Recommended order: narrow the browser boundary, consolidate current status, then
resume with a complete consumer flow. Retain the core ownership and safety checks.
