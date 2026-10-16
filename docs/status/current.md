# Current handoff — 2026-10-16

## Current Canic acceptance reference (pending 0.7.2)

The maintainer delivered 0.7.1 at `9d41664677e976ec293b99d2e1f13d11b83b4d95`;
remote main and the annotated tag were verified. Its retained enrollment selectors
supersede the original PR #64, now closed without merging the old branch. The
older pending headings below retain preparation-time evidence.

The Canic guide now selects the released portable API and Rust 1.91 minimum,
corrects reset floors to PortableClaimsContext with the login-start skew margin,
and distinguishes proof admission expiry, unchanged leases and login denial
retention. Historical token adapters/deadlines are labeled as earlier evidence.
The existing actual-canister checkpoint case now uses two receivers: after proof
expiry and restoration it retains the exact-retry lease, rejects fresh signed
material for the revoked login, accepts that material on the unaffected peer,
and preserves denial/disabled authority through another upgrade. Peer logout
leaves the other device's session usable. Public APIs and signed/stored formats
are unchanged; this is an internal fixture extension, not a production backend.

`make test-qualification` passes all six canister cases in
`target/canic-071-adoption-qualification.log`. Focused runner Clippy and formatting
pass in `target/canic-071-adoption-clippy.log`; local links and the 103-file
snapshot in `target/canic-071-adoption-docs.log`. No full CI, macOS, real browser,
Canic build/edit or production acceptance is claimed. Source/issue review remains
at [Canic #491 — Auth adoption](https://github.com/dragginzgame/canic/issues/491).
Typed installed-outcome propagation and fresh completion checks are recorded in
[Canic #506 — Effective renewal deadlines](https://github.com/dragginzgame/canic/issues/506#issuecomment-6212867923);
replacement portable endpoint recovery in
[Canic #507 — Non-issuing reconciliation](https://github.com/dragginzgame/canic/issues/507#issuecomment-6212868307);
and the need to requalify successful production access in
[Canic #525 — Actual facade budget](https://github.com/dragginzgame/canic/issues/525#issuecomment-6212868682).
These remain consumer-owned; the current guard is fail-closed and the shared
SessionStore is test-only, so historical successful-facade costs are not a
current-path pass.


## Matched receiver costs and abuse qualification (pending 0.7.1)

The internal receiver now selects protected Ed25519 or certified authority through
one admission endpoint and configurable receiver-wide verification budget. Both
mechanisms run in the same Wasm with matched grants/leases and fresh occupancy.
After one invalid proof spends the only credit, another browser principal is
throttled before verification and separate-budget logout still succeeds. Three
fresh logins, second-receiver admission, exact retry after proof expiry and local
logout/peer isolation also pass. Fixture checkpoints retain the selected policy;
the changed private fixture schema uses fresh state, not old snapshot import.
Public credential/session storage formats and pure library features are unchanged.

All six canister cases pass in `target/matched-receiver-qualification-abuse.log`.
The same-artifact median fresh admissions were 13,082,165 Ed25519 cycles versus
459,554,099 certified cycles; ordinary authorization measured 9,640 instructions
for both. [Qualification evidence](../ic-testkit-qualification.md#matched-portable-receiver-costs-and-failed-proof-throttling)
records raw-sample scope, negative inputs, artifact identity and exclusions. This
is not issuer/TEE/production throughput or Canic facade acceptance. Strict fixture
Clippy and Rust 1.91 native/Wasm checks pass in `target/matched-receiver-checks.log`;
staged-formatting preservation passes in `target/matched-receiver-formatting-final.log`.
No full CI, macOS or native browser run was performed. Incoming selector work and
lock selections were preserved; no sibling source, commit, package version or
release effect changed.

[Auth #60 — Shared workspace renderer adoption](https://github.com/dragginzgame/ic-auth/issues/60#issuecomment-6212073475)
was closed after exact v0.6.2 delivery and Linux/Intel/ARM macOS CI verification.
[Canic #418 — Future-start admission diagnostics](https://github.com/dragginzgame/canic/issues/418#issuecomment-6212142096)
now has the current typed NotYetValid consumer projection requirement. Production
issuer/enrollment, durable/browser adoption and consumer performance remain open
at their existing GitHub trackers.


## Retained enrollment selectors (pending 0.7.1)

Ported [PR #64 — Expose retained session enrollment selectors](https://github.com/dragginzgame/ic-auth/pull/64)
onto the current 0.7.0 source. Read-only session mechanism/key-binding accessors
let a host select exact current enrollment among same-issuer keys. Selection does
not authorize use; the restoration regression also rejects wrong keys, wrong
mechanisms and expired enrollment, with retained state preserved after refusal.
Signed/storage formats and the 0.7 admission callback are unchanged.

All 22 restoration-selected session tests pass in `target/session-selectors-tests.log`.
Strict library/session-test Clippy, Rust 1.91 native/Wasm restoration checks,
engine-only compilation and formatting pass in `target/session-selectors-checks.log`.
The original PR branch remains unchanged/unmerged; these are local 0.7.1 changes,
not delivered consumer adoption. No full CI, canister runtime, sibling edit,
commit, package version or release effect was performed for this additive batch.


## Shared claims preparation and bounded browser calls (pending 0.7.0)

Fresh session admission now passes its owned DecodedPortableClaims to the complete
verifier, sharing one canonical decode/digest without cloning claims. This changes
the callback to `(budget, decoded)`, consuming `decoded.verify(...)` with the proof
and protected context. Standalone byte/envelope verification remains independently
validating. The engine checks the returned digest and current time before commit;
cheap refusals and exact retries still avoid cryptography, and failures retain
charged credit. Candid, signed bytes and stored record formats are unchanged.
The numbered pending changelog selects minor 0.7.0; manifests and Git refs remain
unchanged. Consumers must adopt the callback cut when selecting that release.

Portable admission/logout share a private bounded transport attempt, retaining
separate generation, signing, durable lease and retry checks. All 94 browser cases
and both Candid directions pass in `target/bounded-client-tests.log`; focused
session and standalone verifier suites pass in `target/shared-claims-tests.log`
and `target/shared-claims-verifiers.log`. New regressions reject other-claims or
other-clock callback results without committing or refunding verification credit.
All six canister cases pass in `target/shared-claims-qualification-unconfined.log`
after the sandbox refused the server's loopback bind. Library/fixture strict
Clippy, selected Wasm graphs and the isolated Rust 1.91 native/Wasm package matrix
pass in `target/shared-claims-clippy.log`, `target/shared-claims-fixture-clippy.log`
and `target/shared-claims-msrv.log`. No full CI, native browser or macOS run, new
performance measurement, sibling edit, commit or release was performed.

Old-browser-client retirement remains blocked by portable renewal, production
adapters/non-issuing recovery and explicit retained-intent disposition. Existing
exports/formats and uncertain effects were preserved. Completion is recorded in
[Auth #3 — Browser token-session client](https://github.com/dragginzgame/ic-auth/issues/3#issuecomment-6211158111),
with the pending callback adoption in
[Canic #491 — Auth extraction and adoption](https://github.com/dragginzgame/canic/issues/491#issuecomment-6211209112).


## Release pending-heading correction (pending 0.6.4)

The pending 0.6.4 heading now uses the baseline's numbered, undated form. The
previous `- Unreleased` suffix was rejected before release preparation. Preflight
now names the expected and observed headings and explains that preparation adds
the date. Release fixtures reject both Unreleased and dated pending headings
without changing notes, index, versions or tags; the valid release/recovery path
still passes. `make test-release-tools` passes with local remotes and mocked
registry transport in `target/changelog-preflight-tests.log`. No real release,
publication, version changes or cleanup were executed.

## Canic endpoint adoption reference (pending 0.6.4)

The certified session lifecycle regression now admits the public PortableCredential
through verify_credential on two independent receiver stores/ledgers/budgets. It
checks unchanged exact retry after proof expiry, local logout denial before
cryptography and the other receiver's retained authorization. The host adoption
guide links this regression and the actual qualification receiver's endpoint wiring.

`make test-sessions` passes in `target/canic-unblock-tests.log`. This is producer
protocol evidence, not a Canic build, real cross-subnet run or production durable
backend qualification. No runtime API or persisted format changes. Canic endpoint
migration and acceptance remain in Canic #491, effective renewal deadlines in #506,
non-issuing reconciliation in #507 and final endpoint budget qualification in #525.
Auth #3's production browser adapter depends on the actual consumer endpoints;
its injected initial-admission coordinator is not a production transport adapter.


## Record sizing and dependency policy cleanup (pending 0.6.3)

Session/replay admission and restoration accounting now count bytes with the same
CBOR serializer and domain framing instead of allocating discarded encodings.
Checked counting preserves record validation, byte limits and serializer-error
precedence. The separate encoders still own stored bytes; no format changed.
Auth's named dependency policies now serve workspace graphs, isolated MSRV graphs
and packaged metadata; each caller still independently collects/adopts its graph.
Policy cases cover allowed boundaries, package-prefix matching and rg/jq parity.

Session tests passed in `target/cleanup5-validation.log`; that run subsequently
found a Clippy warning in the previous batch's vector test. Its chunk iteration
was corrected. Counting boundary/error/overflow tests pass in
`target/cleanup5-counting.log`; strict Clippy, graph policies and the complete
isolated native/Wasm MSRV matrix pass in `target/cleanup5-checks.log`. No full CI,
native browser, real canister lifecycle or macOS run was performed for this batch.

Read-only Build inspection found private external-lock admission already owned by
its archive engine. Auth's request for a narrow published interface and semantic
requirements is recorded in
[Build #25](https://github.com/dragginzgame/ic-build/issues/25#issuecomment-6199516820).
The local yq/jq lock comparison remains until adoption can be qualified. No sibling
source, package versions or Git refs changed; incoming working edits were retained.

## Contract fixture relocation and certificate buffer cleanup (pending 0.6.3)

The browser declaration/vector CLI now lives in unpublished native tooling.
Protocol DTO ownership and generated outputs are unchanged; the published protocol
crate no longer carries the exporter example or its printer dev dependency.
Generation emits the declaration and three vectors in one invocation, and checking
validates all three TypeScript replies in one invocation. Host owns bounded file
reads and durable writes; malformed/oversized replies fail without changing inputs.
The lockfile change for this task adds only the tooling package's two direct edges;
incoming external dependency selections were preserved.

Canonical certificate encoding appends directly to the proof's buffer, preserving
the domain prefix, field order and validation. This removes the temporary full
certificate allocation/copy; no runtime speedup has been measured.

All 92 browser tests with generated-contract comparison and bidirectional vectors,
protocol tests, file-tool tests and strict Clippy pass in `target/cleanup4-tests.log`.
The expanded tool rejection tests and isolated native/Wasm MSRV matrix pass in
`target/cleanup4-followup.log`, including release/publication recovery fixtures
with local remotes and mocked transport. Documentation/snapshot/diff checks pass. No full CI,
fresh native browser, canister runtime or macOS qualification was run.

Signed-ingress transport extraction is proposed in
[Testkit #106](https://github.com/dragginzgame/ic-testkit/issues/106).
Auth retains its local helper until a published replacement passes real ingress
and lifecycle qualification. No sibling source, release version or Git ref changed.

## Shared opening, lifetime ceilings and compiler selection (pending 0.6.3)

Every package now inherits the root MSRV, including the qualification canister.
Setup/check tooling selects it once; the separate internal Wasm check remains.
Browser stores share private IndexedDB opening/abandonment handling while keeping
schema creation, profile validation and error mapping local. Schema exceptions
abort and reject without allowing a late connection to escape. Login, proof and
session lifetime ceilings now have one private Rust owner; validation boundaries,
values, error ordering and persisted formats are unchanged.

`make test-client test-sessions test-protocol clippy check-msrv` passed in
`target/cleanup3-validation.log`, including 92 browser tests and the isolated
native/Wasm MSRV feature matrix. New opening regressions cover timeout/blocked
abandonment, late upgrade/connection events and schema refusal. Shell syntax,
documentation links, snapshot integrity and diff checks pass. No full CI, fresh
native browser, canister runtime or macOS run was performed. Incoming dependency
and MSRV changes were preserved; no versions, Git refs or sibling sources changed.

## Workspace MSRV alignment (pending 0.6.3, 2026-10-15)

The workspace minimum is now Rust 1.91.0, matching the qualification canister.
Development formatting and Clippy remain on 1.99.0. Existing setup/check scripts
read the declared floors; no compiler selection is hard-coded downstream.
`make check-msrv` passed the isolated package native/Wasm feature matrix and
internal application checks in `target/msrv-191.log`, preserving its sealed package
and lock inputs. Documentation links and snapshot integrity also pass. No full CI
or native macOS qualification was run. The incoming Testkit update was preserved;
this task did not select dependencies, change release versions or commit changes.

## Portable browser admission orchestration (pending 0.6.2, 2026-10-15)

The private client now coordinates initial portable issuance/reconciliation and
receiver admission using injected transports and the existing logout journal.
One atomic issuance claim selects `issue`; subsequent instances and uncertain
attempts select non-issuing `reconcile`. One detached proof serves multiple
receivers until its original admission expiry. Destination recording precedes
admission, and late issuance/admission replies cannot restore a logged-out login.
Transport calls have bounded cancellation-scoped signing identities. This adds
no journal schema, silent old-intent migration or application-call retries.

All 90 browser tests and both generated Candid directions pass in
`target/portable-admission-client-final.log`. New regressions cover multi-receiver
reuse, competing claims, lost reply plus store reopen, unresolved recovery, logout
during issuance/admission, timeout/late signing, invalid identity, storage failure
and oversized/expired proof material. Transports are substitutes; no production
issuer, receiver endpoint or cryptographic proof acceptance is established by
these tests. No fresh native browser, Rust suite or full CI run was performed.

Production adapter/clock qualification, renewal and old-token lifecycle retirement
remain in [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3), with Canic #507's
non-issuing recovery prerequisite. The coordinator deliberately cannot renew or
replace an expired/unknown issuance. Existing receiver leases remain independent
of admission expiry. See the [browser contract](../browser-client.md#portable-admission-coordinator).
Incoming working changes, dependency selections, package versions and sibling
sources were preserved; no commit, push or publication was performed.

## Storage framing cleanup and retirement review (pending 0.6.2, 2026-10-15)

Session, replay and login-revocation storage codecs share private domain/CBOR
framing and outer byte limits. Record validation and error projection remain with
each record owner; restore-only decoding stays behind `session-restore`. No public
API, stored format or dependency selection changed. Focused session tests, strict
Clippy, Wasm and dependency boundaries pass in `target/next-cleanups-checks.log`.
The affected library features also pass Rust 1.88 in `target/next-cleanups-msrv.log`.
The new regression checks cross-record refusal, roundtrips and capacity-error
precedence; the initial test incorrectly expected a framing error for an oversized
cross-record input (`target/next-cleanups-sessions.log`) and was corrected without
changing production error order. No full CI, browser, canister lifecycle or native
macOS run was performed for this cleanup.

Two proposed removals remain gated after deeper review. Published Memory 0.42.0's
base `FixtureCell` reads retained payloads before Auth can bound them and its
underlying Cell initializer resets invalid magic to an empty cell. The existing
128 KiB checkpoint wrapper remains; bounded fresh/retained admission belongs to
[Memory #104](https://github.com/dragginzgame/ic-memory/issues/104#issuecomment-6197200398).
The old browser token lifecycle also remains: portable data codecs/logout are not
a production issuance/admission adapter, and retiring unresolved intents requires
qualified non-issuing reconciliation and explicit storage disposition. The review
is recorded in [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3#issuecomment-6197200986),
with the existing Canic #507 prerequisite. No sibling source or retained browser
state was changed; no replacement capability or completed migration is claimed.

## Browser transaction and claims cleanup (pending 0.6.2, 2026-10-15)

The token store and logout journal now share one private strict IndexedDB
transaction lifecycle. Their schemas, error projection and mutations remain
separate. Results publish only on completion; aborts, missing results, timeout
and even falsy callback failures cannot report a successful commit.

Session admission and both portable proof verifiers hash the original claims
bytes after canonical decoding, eliminating repeated encoding at those hash
points. Independent decoding remains at public boundaries. APIs, wire format,
validation order and budget charging are unchanged; no speedup is measured.
The regression compares encoding and fingerprints for both mechanisms, wide
claims and accepted mutations of the frozen vector.

The 82 browser tests and focused protocol, session and portable signature tests
pass in `target/three-cleanups-focused.log`. Strict Clippy, Wasm, dependency
boundaries and isolated-package MSRV checks pass in
`target/three-cleanups-checks.log`; documentation links and the 103-file snapshot
also pass. No full CI, fresh native browser, canister lifecycle, benchmark or macOS
run was performed for this cleanup. The generic filesystem CLI extraction
still needs a published Host executable; the concrete adoption requirements are
in [Host #76](https://github.com/dragginzgame/ic-host-tooling/issues/76#issuecomment-6196527711).
The local CLI remains until that replacement can be qualified. No sibling source,
package version or Git refs changed. These cleanups do not complete the production
browser adapter or Canic migration.

## Shared manifest renderer adoption (pending 0.6.2, 2026-10-15)

Auth's release metadata adapter now delegates workspace version substitution to
Shared Tooling's pure renderer. The canonical exporter extends the existing
0.15.0 snapshot at `3bef3f2daec7f64f875397ed4d53d5b9e2e68a08` by one selected
helper (103 files total); no other snapshot bytes or revision change. Auth retains
its exact protocol dependency selector, parsed candidate comparison, saved-source
custody and interruption/recovery policy. No subprocess failure can publish a
partial candidate manifest.

Focused `make test-release-tools` passed in
`target/shared-renderer-release-tests.log`, covering byte preservation, unrelated
version fields, missing/duplicate/wrong dependency selectors, wrong workspace
versions, historical sources, mixed metadata recovery and publication uncertainty.
All release/push/upload effects were confined to local fixtures or substituted
transport. No real package version, Git refs, publication or sibling source
changed. This completes local implementation of
[Auth #60](https://github.com/dragginzgame/ic-auth/issues/60); delivered-source
acceptance remains pending. No full suite or native macOS run was performed.

Auth 0.6.1 is pushed at `8cedb6ef75ea34c49fc49597b35bcc500944ef36`;
its [release CI](https://github.com/dragginzgame/ic-auth/actions/runs/38759995596)
was still running at inspection. There was no newer Canic #491 consumer feedback;
the transaction/workflow migration and production durable backend remain open.

## Custom session-store binding check (pending 0.6.1, 2026-10-15)

Custom stores can now call the existing session/login-change association check
through the public API rather than decode private fields or copy the reference
store's rules. Runtime behavior is unchanged: caller/domain must match; admission
also matches issuer/account/login window and retention. A tombstone can clear an
older session without an account binding, but can never authorize admission.
See the [complete host migration contract](../sessions.md#custom-host-adoption-from-the-pre-06-session-api).

Read-only Canic inspection at HEAD `644d4b13012b395d6211a6907e2e6aafdf391db9`
plus its existing working tree confirms broader propagation remains: the custom
store uses the old transaction/accounting contract and the workflow still passes
old token/context inputs. Canic must retain ledger/budgets, persist login deltas
atomically, implement session-only removal and adopt portable inputs and checked
codecs. No Canic source/build was changed/run. Adoption remains in
[Canic #491](https://github.com/dragginzgame/canic/issues/491); production durable
storage remains [Auth #58](https://github.com/dragginzgame/ic-auth/issues/58).

The external public-API regression covers mismatched caller/domain/issuer/account
and both login-window endpoints, admitted logout and absent-reservation logout.
All 19 restoration-selected session tests pass in `target/canic-binding-tests.log`.
The standard session feature tests, strict Clippy, Wasm, dependency boundaries
and isolated-package MSRV pass in `target/canic-binding-checks.log`.
Incoming lockfile edits were preserved; package versions and Git refs are unchanged.
No full suite, fresh canister lifecycle run or native macOS qualification claimed.

## IC Memory checkpoint qualification (working tree, 2026-10-15)

The internal canister now uses published `ic-memory` 0.40.0 for a single
host-declared checkpoint allocation. It replaces raw CDK stable-save with a
bounded 128 KiB payload, explicit header checks and host bootstrap before opens.
Old unpublished fixture installations require reinstall; there is no dual layout
reader. The public libraries and volatile session dependency graph stay unchanged.
The combined qualification Wasm includes storage and is not a minimal volatile
consumer size benchmark.

The canister declares Rust 1.91.0 independently; public libraries, native tooling
and Testkit runner retain 1.88.0. Explicit MSRV setup/checks now cover both declared
compilers. No existing registry selection changed; IC Memory and its missing
transitives were added to the single lockfile.

The checkpoint bounds/replacement test and all six real-canister lifecycle tests
passed in `target/memory-checkpoint-qualification.log`. Repeated upgrades retain
sessions, replay records and denials, restore closed, and require fresh fixture
controller authority confirmation. Changed-package native/Wasm Clippy, dependency
boundaries and isolated-package/fixture MSRV checks passed in
`target/memory-checkpoint-checks.log`.

This qualifies host allocation and upgrade checkpoint recovery, not a production
per-request stable adapter or Root enrollment. Atomic per-operation writes,
protected metadata and capacity/failure recovery remain in
[Auth #58](https://github.com/dragginzgame/ic-auth/issues/58). No package version,
commit, push, release, full suite, macOS run or sibling edit was performed.

## Durable maintenance commit boundaries (working tree, 2026-10-15)

Session/replay pruning, login-ledger pruning and authority-generation advancement
now support host commit callbacks before heap mutation. Pruning stages at most
128 keys through existing expiry indexes, reports exact resulting counts/bytes,
and skips callbacks for empty work. Returned storage errors preserve records,
indexes and accounting; the ledger retains trusted clock observations. Generation
overflow refuses before persistence. Volatile methods reuse these algorithms.
See [atomicity](../sessions.md#atomicity-and-logout).

Focused session/proof tests passed (`target/maintenance-commit-tests.log`),
including failed commits, exact retry deletions, empty work, generation failure
and overflow. Clippy, Wasm, boundaries and package MSRV passed in
`target/maintenance-commit-checks.log`. All six real-canister tests passed in
`target/maintenance-commit-qualification.log`, including authority invalidation
and repeated checkpoint upgrades. Links, snapshot and diff checks also passed.

The ic-memory adapter remains unfinished under
[Auth #58](https://github.com/dragginzgame/ic-auth/issues/58); these hooks avoid
duplicating core maintenance logic in that adapter. No dependencies, compiler
floor, package versions, Git refs or sibling sources changed in this increment.
No full-suite or native macOS validation is claimed.

## Validated per-write host commit boundary (working tree, 2026-10-15)

The reference session store now exposes `transaction_with_commit` so a durable
adapter can reuse its staging and validation. The write-only callback receives
the proposed login record, touched admission pair (or logout), and exact resulting
counts/bytes. It runs before heap publication; reads/refusals never persist and
returned failures leave records/indexes/accounting unchanged. This requires no
whole-store cloning. See [atomicity](../sessions.md#atomicity-and-logout).

The failure regression now injects an error at the real host commit boundary,
removing the test-only second-transaction counter. Focused session tests passed
in `target/session-commit-tests.log`; Clippy, Wasm, boundaries and package MSRV
passed in `target/session-commit-checks.log`. All six real-canister tests passed
in `target/session-commit-qualification.log`, including checkpoint restoration,
volatile reset, logout and signed ingress.

Durable atomic writes, protected metadata, maintenance/generation updates and
clock/budget persistence still belong to the unfinished ic-memory adapter under
[Auth #58](https://github.com/dragginzgame/ic-auth/issues/58). No stable dependency,
compiler-floor or package-version changes, full-suite/macOS validation, commits,
pushes or sibling edits were performed by this increment.

## Real receiver checkpoint upgrades and Testkit 0.45 (working tree, 2026-10-15)

Explicit Testkit setup completed (`target/testkit-045-setup.log`), selecting
Testkit 0.45.0 and server 16.1.0. Qualification callers now use its fallible Candid
API with explicit transport assertions; the retired panic helpers caused the
first compile failure, retained in `target/checkpoint-qualification.log`.

All six real-canister tests passed in `target/checkpoint-qualification-fixed.log`.
The new opt-in fixture checkpoint restores sessions, replays, login records and
protected metadata through real upgrades. It opens only after a new controller
authority confirmation, preserves existing deadlines and both known/unknown-login
tombstones, and retains disabled authority across a second upgrade. Existing
volatile-reset and signed-ingress tests remain. See [evidence scope](../sessions.md#evidence-and-remaining-adoption).

Focused session tests, changed-package Clippy and native/Wasm package MSRV passed
in `target/checkpoint-focused.log`. Dependency inputs were checked against
`target/checkpoint-dependency-inputs.sha256`; the independently selected Host 0.19
and Testkit 0.45 changes were preserved, not authored by this increment. The
previous Testkit preparation/qualification blocker is resolved.

The checkpoint uses the fixture's existing stable-save owner and resets work
budgets empty. It is not a production ic-memory adapter, per-operation stable
transaction or Root enrollment implementation. Those remain under
[Auth #58](https://github.com/dragginzgame/ic-auth/issues/58). No full-suite or
macOS validation, package-version changes, commits, pushes or sibling edits.

## Checked session/replay reconstruction (working tree, 2026-10-15)

The pending store now streams checkpoint records and restores complete checked
session/replay sets with physical counts, encoded bytes and generation. It rebuilds
expiry indexes and per-caller counts, preserves replay history after logout and
generation advancement, and rejects partial, duplicate, future-generation or
over-capacity state. Replay uses a bounded domain-separated codec instead of
public raw Serde. See [storage and reset](../sessions.md#storage-and-reset).

Focused tests, Wasm, dependency boundaries, Clippy and isolated-package MSRV
passed in `target/store-restore-final.log`. Tests cover authorized/exact-retry
use after reconstruction, consumed-proof denial after logout, pruning and decoder
corruption. These checks completed with Testkit 0.44.0 selected. The subsequent
independent Cargo manifest/lock update to Testkit 0.45.0 was preserved; its CLI
was not prepared, so real-canister qualification stopped at admission in
`target/store-restore-qualification.log`. Run explicit `make install-testkit-tools`
before repeating that qualification; no older-runtime substitution was made.
Core library inputs are unaffected by that native fixture dependency change.

No stable backend or real persistent-upgrade qualification is claimed. The
ic-memory adapter and atomic protected metadata integration remain under
[Auth #58](https://github.com/dragginzgame/ic-auth/issues/58). No full-suite or
macOS check was run. This increment changed no package versions or Git refs;
the independent dependency edits and all prior work remain intact.

## Checked login-ledger restoration (working tree, 2026-10-15)

The pending 0.6 library now encodes bounded login records and, with
`session-restore`, rebuilds a ledger from a complete trusted atomic generation.
It rejects malformed records, duplicates, wrong domains, invalid retention,
clock rollback and missing/excess physical pool entries. It preserves original
login bindings and tombstone deadlines; failed restoration exposes no partial
ledger. Host-owned storage provenance, generation protection, configuration and
fresh enrollment remain required. See [storage and reset](../sessions.md#storage-and-reset).

Focused session/proof tests, Wasm, dependency boundaries and Clippy passed in
`target/ledger-restore-final.log`; isolated package MSRV checks passed in
`target/ledger-restore-msrv.log`. Tests cover restored reservations authorizing
retained sessions/exact retries and restored tombstones denying both. The initial
test compilation failure and its repaired run remain in
`target/ledger-restore-tests.log` and `target/ledger-restore-tests-fixed.log`.
This is native library evidence, not a durable canister upgrade qualification.
No full-suite or native macOS check was run for this increment.

The ic-memory adapter, complete session/replay restoration and atomic protected
metadata integration remain under [Auth #58](https://github.com/dragginzgame/ic-auth/issues/58).
No dependency, package version, index, Git ref or sibling source was changed.

## Shared Tooling adoption and durable transaction input (working tree, 2026-10-15)

Adopted Shared Tooling 0.15.0 at `3bef3f2daec7f64f875397ed4d53d5b9e2e68a08`
through its canonical exporter (102 files). CI now uses its exact-ID,
digest-verified evidence downloader. IC Host's latest published packages remain
0.18.6, already selected here; dependency and tool catalogs needed no changes.
The adopted baseline uses focused checks for ordinary development/handoff;
the complete CI/release gate remains unchanged.

SessionStore transactions now receive the proposed login reservation/tombstone
so durable hosts can commit it with session/replay/index state. The volatile
adapter rejects missing or mismatched proposals before effects, including
cross-domain logout. See [the storage contract](../sessions.md#storage-and-reset).
Checked ledger restoration and an ic-memory adapter remain unfinished under
[Auth #58](https://github.com/dragginzgame/ic-auth/issues/58); Canic adaptation
belongs in [Canic #491](https://github.com/dragginzgame/canic/issues/491).

Focused Linux checks passed: formatting-hook, evidence, release-runner/release-tool
fixtures and host tooling (`target/shared-015-focused.log`); session/proof tests
(`target/login-change-tests.log`); MSRV, boundaries, Wasm and Clippy
(`target/login-change-boundaries.log`); and real-canister qualification
(`target/login-change-qualification-localhost.log`). The initial qualification
attempt was blocked by sandbox localhost binding, retained separately in
`target/login-change-qualification.log`. No current full-suite or native macOS
result is claimed. Package versions, index, refs and sibling sources are unchanged.

## Library batch push review (working tree, 2026-10-15)

The current library batch is prepared for a review push, not a claim that the
complete production 0.6 system is ready. The draft changelog now describes the
final portable API rather than superseded intermediate contracts; README/setup
separate implemented machinery from issuer, durable-host and consumer adoption.
Published changelog history is unchanged. Package versions remain 0.5.10.

Code and dependency inputs are unchanged from the passing complete Linux gate
`target/portable-cost-ci.log` (80 browser tests and five real-canister tests).
This documentation-only review reuses that evidence; local links, snapshot and
diff checks pass in `target/push-review-docs.log`. Native macOS acceptance of the
future pushed source remains separate. No index, Git refs or release effects
were changed. Include the new untracked source/contracts/tests in the reviewed
commit alongside the tracked edits and intentional old-session-file removals.

## Complete Ed25519 receiver cost and durable boundary (working tree, 2026-10-15)

The actual volatile receiver qualification now records cycle-balance deltas and
its Wasm hash for full admission, forged-proof rejection, exact retry, ordinary
authorization and logout. Fresh admissions across two runs measured 12.68–12.95M
cycles; ordinary session updates measured 7.18M. These include platform charges,
exclude native producer signing and do not establish a percentage saving against
the older certified fixture. See [measurement scope](../design/0.6-portable-authentication.md#capacity-and-abuse).

The [storage contract](../sessions.md#storage-and-reset) records the IC Memory
allocation boundary and separate adapter compiler requirement. Persisting only
SessionStore would leave the concrete volatile revocation ledger outside the
durable commit; [Auth #58](https://github.com/dragginzgame/ic-auth/issues/58) owns
that remaining integration. No dependency/backend was added by this inspection.

Complete Linux `make -j4 ci` passed in `target/portable-cost-ci.log`, including
80 browser tests and five real-canister tests. Focused qualification passed in
`target/portable-cost-qualification.log`. No sibling source, dependency/compiler
selection, package versions or Git refs changed.

## Shared SDK logout transport (working tree, 2026-10-15)

`httpLogoutTransport` now connects the portable cleanup worker to the SDK's
authenticated update-call path, using the current guarded identity per attempt.
It sends the shared empty request, permits one submission per journal attempt,
cancels polling and rejects unauthenticated or oversized Candid replies. Gateway,
endpoint name, optional local trust anchor and key disposal remain explicit host
configuration. See [browser integration](../browser-client.md). Production issuer
and receiver acceptance, login issuance and consumer adoption remain open.

Complete Linux `make -j4 ci` passed (`target/logout-http-ci.log`), including 80
browser tests and five real-canister tests. The new transport tests use real SDK
signing with substituted HTTP; they do not qualify successful deployed logout.
Focused evidence is in `target/logout-http-client-retry.log`; the initial fixture
assertion failure remains in `target/logout-http-client.log`. No sibling source,
dependency selections, package versions or Git refs changed.

## Keyless-tab cleanup continuation (working tree, 2026-10-15)

A tab without a local signing handle no longer finishes shared cleanup or invokes
key disposal before confirmation/deadline. It returns the pending snapshot so a
key-holding worker can continue. New destination windows reserve the five-second
final-attempt margin inside the existing five-minute cleanup bound. Retained
windows/intents are not rewritten. The two regression tests fail against the
previous implementation (`target/keyless-regression-before.log`) and pass after
repair. No production consumer adoption is implied.

Complete Linux `make -j4 ci` passed (`target/keyless-ci.log`), including 74 client
tests and five real-canister tests. The updated native Chromium fixture passed
with real IndexedDB (`target/keyless-native.json`). Focused results remain in
`target/keyless-client.log`. No sibling source, stored intents, versions or Git
refs were changed; production key/clock/transport integration remains open.

## Portable signing boundary and native browser (working tree, 2026-10-15)

`guardedIdentity` gates ordinary SDK signing before and after asynchronous signing
against the durable active generation/caller and destination inventory. Cleanup
transports receive an identity gated to their live lease, deadline and destination.
Already returned signatures and remotely accepted requests cannot be recalled.
Production key custody, clock qualification and application scheduling/callback
wiring remain consumer obligations; no Canic/Toko adoption is claimed.

The maintained native logout fixture passed on Linux Chromium 155.0.8059.40 with
real IndexedDB: strict commit, cross-connection logout, late-signature refusal,
abort atomicity, reopen, Capacity retry and disposal. Evidence is retained in
`target/signing-native.json`; the isolated browser and localhost server were closed.
The browser fixture uses a synthetic identity; Node tests use actual SDK signing.

Complete Linux `make -j4 ci` passed in `target/signing-ci-final.log`, including
72 client tests and five real-canister tests with selected Testkit 0.44.0. Its CLI
was explicitly prepared after the initial gate refused the missing selected tool;
`target/signing-ci.log` retains that failure. Focused evidence is in
`target/signing-client-complete.log`. Dependency selections were preserved; no
sibling source, package versions or Git refs were changed.

## Portable browser logout cleanup (working tree, 2026-10-15)

The opt-in `IndexedDbLogoutStore` and `PortableLogoutClient` now persist a
non-secret per-login cleanup boundary, target inventory, fixed deadline and
attempt counts. Cleanup consumes the shared typed results, reserves a final
post-admission retry and retains uncertainty across crashes. Existing token
store intents are untouched. See [the browser integration obligations](../browser-client.md).
Production identity/clock/transport wiring and native-browser qualification of
this new journal remain pending; this does not complete Canic/Toko adoption.

Complete Linux `make -j4 ci` passed: `target/cleanup-ci.log`, including 67 browser
tests and all five real-canister tests. Focused browser evidence is retained in
`target/cleanup-client-complete.log`; earlier failed attempts remain separately.
No sibling files, retained token intents, versions or Git refs were changed.

## Typed logout boundary (working tree, 2026-10-15)

The shared session engine now returns Rust-owned `LogoutResult`, also used by the
actual qualification endpoint and generated browser bindings. Only committed
local revocation returns `Revoked`; absent startup state returns `ResetPending`.
The empty request accepts no identity or proof. Browser cleanup orchestration and
issuer integration remain pending; this is not a global logout acknowledgement.

`make -j4 ci` passed on Linux: `target/logout-ci-complete.log`, including
57 browser tests, all five actual canister tests, MSRV, Wasm and release fixtures.
Earlier failed attempts remain in `target/logout-ci.log`,
`target/logout-ci-final.log` and `target/logout-qualification.log`.
No sibling files, stored intents, package versions or Git refs were changed.


## Portable credential boundary (working tree, 2026-10-14)

`PortableCredential`/`PortableProof` now provide the Rust-owned Candid envelope,
with generated browser descriptors and bounded byte codecs. Protected
`PortableVerificationContext` selects the complete verifier through
`verify_credential`; both qualification paths use that dispatcher within the
shared session lifecycle. Signed claims remain unchanged and opaque in transport.
Production issuer/browser orchestration and durable host adoption remain pending.

Final-tree `make -j4 ci` passed on Linux; evidence is in
`target/credential-ci.log`, including both mechanism builds, mixed-budget checks,
Rust/TypeScript Candid directions and all five real-canister tests. The shared
browser decoder now rejects truncated blob reads without rejecting valid differing
Candid encodings. [Auth #62](https://github.com/dragginzgame/ic-auth/issues/62)
records the SDK finding and delivery acceptance; Canic #491 owns adoption.
Earlier fixture rejection evidence remains in `target/credential-focused.log`.
No stored issuance intents were migrated, no sibling files edited and no release
or Git-ref effects performed.


## Portable session lifecycle hard cut (working tree, 2026-10-14)

The shared session engine now admits portable certified and Ed25519 credentials,
with caller-bound sessions and issuer-scoped accounts. It atomically couples
session/replay commits to login reservations, handles exact retries after proof
expiry, checks current authority and revocation on authorization, and removes
sessions with retained logout tombstones. The old token-specific session adapter
and record encoding are retired; old persisted records must not be reinterpreted.
See [the current API and host obligations](../sessions.md).

Final-tree `make -j4 ci` passed on Linux with selected Testkit 0.43.0; evidence
is retained in `target/session-cut-ci-final.log`. All five real-canister tests
pass. The fixture measured a 654-byte portable record, 38,377 decode instructions
and 8,412 local authorization instructions, not production cycles or latency.
Earlier failed attempts are retained in `target/session-cut-qualification.log`
(sandbox localhost denial) and `target/session-cut-ci.log` (fixture clock
assumptions corrected without relaxing admission checks).


The reference backend is volatile. Production issuer/enrollment orchestration,
browser fan-out and durable host integration remain open under Auth #59/#58/#3
and Canic #491. No Canic files, release versions or Git refs were changed.
The qualification fixture exercises two receivers/devices, actual signed ingress
and upgrade readiness/reset refusal; it is not a production wallet service or
cross-subnet latency benchmark. Earlier receipts below describe their own source
and must not be treated as qualification of this hard cut.


The maintainer accepted the larger Canic authentication extraction, selected the
name `ic-auth`, and confirmed `/home/adam/projects/ic-auth` as the local repository.

The current committed release is `c5cf568e102cb8cfac26ff5e0e07d64e8310533b`
(`0.5.10`) on official `main`, with annotated tag object
`97d7e3bc178cec1dc534729734d97363241566ee`. Main, peeled tag and retained
publication intent agree. Both unyanked registry entries match intent: types
`393b7edc8577eded061830af056888bd3970f609995efcb3200c87759a548f36`, auth
`b9bd46905c070da0339615c93467660d23c2029ca7a4a36a173b820f332ad739`.
[Exact-source 0.5.10 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38399120317)
is queued at inspection; #48/#49 remain open for native acceptance. The 0.5.9
run was cancelled, retaining its earlier Linux pass without completed macOS
acceptance. #43–#47 also remain open. Incoming Cargo.lock changes select Host
0.17.2 and are preserved separately from the shape API work below.

[0.5.8 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38373594603)
was cancelled after its Linux pass; its macOS acceptance remains incomplete.
Its source selects direct Host 0.16.0/Testkit 0.35.1 with Testkit Host 0.15.2.
Prior [0.5.7 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38284363288)
passes all three native hosts and closes #38–#42 at that source. It does not
qualify later producer or Canic inputs. Earlier partial/cancelled release runs
remain historical evidence in their preparation receipts below.
The preceding
[Exact-source 0.5.4 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38247803905)
now passes the complete gate on Linux, Intel macOS and ARM macOS, completing
#36/#37 native acceptance at that exact released source. Released
0.5.4 source selects Shared 0.8.4, Host 0.14.8 and Testkit 0.34.4/server 16.1.0.
The prior 0.5.3 Intel client-setup failure stays in
[Auth #38](https://github.com/dragginzgame/ic-auth/issues/38).
Released 0.5.0's
[complete three-host evidence](https://github.com/dragginzgame/ic-auth/actions/runs/38160094000)
qualifies its pure request, chain-key and certificate APIs at that older source.
Earlier pending/publication observations below remain historical. The original
bootstrap was `7a03102e52b7a400588530997a9f7fb91c83de9d` (`0.1.0`).
The public remote [dragginzgame/ic-auth](https://github.com/dragginzgame/ic-auth)
was created at the maintainer's request and is configured as `origin` using
`https://github.com/dragginzgame/ic-auth.git`.
The accepted [design](../design/extraction.md) covers signatures, application
tokens and local sessions, plus an independent Solana wallet-auth service.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) remains the existing
extraction and Canic adoption tracker. New IC Auth-specific issues belong in
the public repository's issue tracker.
The [bootstrap evidence comment](https://github.com/dragginzgame/canic/issues/491#issuecomment-6055817375)
records this batch without closing the larger extraction.

The virtual Rust workspace contains `ic-auth-protocol-types`, `ic-auth` and the
unpublished native application in `apps/tooling/`, with one lockfile, root-owned
dependency selections and current manifest version `0.5.10`.
The passive token/proof contracts and canonical encoding are adapted from the
clean Canic source at `e286b3fd98460c98670336853f80658a920966e0`; see the
[source review](../design/canic-source-review.md) for exact ownership and API
adaptations. Both packages now allow crates.io publication of their implemented
contracts/encoding, with inherited repository/README metadata and matching MIT
notices in the package payloads. Released `0.1.6` includes optional complete token
and IC signature verification, session/replay machinery and bounded signature
preparation/retrieval. Released `0.1.7` includes the internal canister/runner
packages under `apps/qualification/` and their IC Testkit qualification.
There is no stable canister adapter or wallet endpoint yet. The private browser
client now implements injected token lifecycle machinery; real issuer adapter
qualification, npm publication and consumer adoption remain pending.
Canic adoption is in progress in the consumer's held integration batch. Its owner
records published Auth 0.5.0 request hashing, certificate rules and enabled
chain-key checks/normalization, with direct production K256 retired from Core,
in [the existing tracker](https://github.com/dragginzgame/canic/issues/491#issuecomment-6120497751).
The earlier 0.4.2 preparation checkpoint remains historical evidence below.
That evidence binds its reported consumer inputs, not later changing sibling
files. Complete-token, durable-session and deployed acceptance remain separate.
No sibling repository was modified or built by IC Auth work.

## 0.6 portable admission and browser logout design — 2026-10-14

The accepted custody simplification supersedes the earlier universal TEE gate
and controller-call-path/snapshot-take retirement requirements below. Governance
is trusted under local and chain-key signing alike. Local issuer keys use IC
randomness and heap-only storage, rotate on code upgrade and require fresh Root
approval; receivers cache authenticated enrollment. Ordinary-subnet local custody
requires explicit operator/replica trust, while TEE is optional assurance with its
own evidence. Threshold signing remains an alternative. Receiver session storage
is still independently volatile or persistent. Finite authority, rollback,
retirement and logout semantics remain unchanged. The proposal now separates
`enrolled_ed25519` from custody selection; no config or issuer endpoint exists yet.
Applications needing only an IC caller/account mapping need not adopt portable
credentials. This is a documentation-only simplification, not changed runtime
verification or a claim of completed production qualification.

Portable context now supplies `advance_reset_floors` for the accepted state-loss
rule. It uses trusted boot time plus qualified skew for login start, boot time
plus one for proof issuance, and never lowers retained floors. Arithmetic failure
leaves both unchanged. Tests cover both mechanisms, issuer-ahead clocks, renewed
old logins, fresh-login timing and overflow. Hosts still own atomic invalidation,
retained cutoff policy, clock qualification and fresh enrollment before readiness;
this does not implement the full volatile host adapter. Complete Linux
`make -j4 ci` passed (`target/reset-floors-ci.log`), including canister qualification,
native/Wasm MSRV and mocked release/recovery fixtures.

Unknown-login revocation now requires its own retained receiver-wide budget,
reusing `SessionAdmissionBudget` independently of proof-verification credit.
Fresh unknown attempts charge before storage, failure retains charges, and
exhaustion returns `Capacity` without false success. Live reservations and
tombstone retries bypass unknown-login credit; expired entries need fresh credit.
Tests cover principal floods, failed commits, reset/rollback, refill boundaries,
cheap refusals and admitted logout while unknown credit is exhausted. This is
ledger machinery, not an installed portable logout endpoint. Complete Linux
`make -j4 ci` passed (`target/logout-budget-ci.log`), including both proof profiles,
canister qualification, native/Wasm MSRV and mocked release/recovery fixtures.

Portable lease calculation now requires the protected receiver role and narrows
requested scopes to that role's signed grant. It cannot combine grants or accept
an absent role. The existing session engine uses the same request-narrowing owner;
signed claims and request hashes remain unchanged. Focused tests cover both proof
mechanisms, multi-role separation and existing session rejection behavior.
Complete Linux `make -j4 ci` passed (`target/grant-narrowing-ci.log`), including
canister qualification, native/Wasm MSRV and mocked release/recovery checks.
This is admission machinery; complete portable session storage/authorization
remains pending.

Revocation reservation now exposes a bounded preflight refusal check, sharing its
rules with final authenticated reservation. Hosts can reject revoked callers,
changed login bindings and full capacity before spending cryptographic credit.
Preflight does not reserve or authenticate anything; final reservation rechecks
state, including logout/capacity changes at the same timestamp. Composed host tests
cover this ordering and retained work credit, without claiming a complete endpoint.
Full Linux `make -j4 ci` passed (`target/reservation-preflight-ci-retry.log`).
The first attempt stopped because the selected Testkit 0.42.0 CLI was not locally
prepared; explicit `make install-testkit-tools` succeeded before the retry.
Dependency selections were preserved.

The shared receiver-wide admission budget now lives outside session/BLS feature
selection and is mandatory for both portable proof verifiers. Failed cryptography
and later storage failure retain charges; changing callers or switching mechanisms
cannot replenish credit. The old session engine uses the same owner and retains
its existing admission errors. Constructor errors now use `AdmissionBudgetError`;
consumers import the budget from `admission_budget`. Focused tests cover both
mechanism directions, cheap refusals, failed commits, empty reset and rollback.
Complete Linux `make -j4 ci` passed (`target/shared-budget-ci.log`), including
combined-mechanism checks, native/Wasm MSRV and real canister qualification.
Full portable session/record authorization integration is still pending.

Portable lease calculation now shares the existing engine's checked request TTL
policy. Both proof verifiers capture key/enrollment expiry in the verified result;
leases may outlive proof expiry but cannot exceed login or captured authority
limits. Tests cover both proof mechanisms, refresh isolation, policy bounds and
clock overflow. Full `make -j4 ci` passed, including certified/Ed25519 lease
checks and canister qualification (`target/portable-leases/ci.log`). Full portable
session admission remains pending.

Both portable proof verifiers now return one read-only `VerifiedPortableClaims`
result carrying canonical fingerprint and verification time. The revocation ledger
requires this result, its configured audience and the matching runtime caller/time;
ordinary decoded or edited claims cannot reserve a login. Reservations retain the
issuer/account pair and original login start/deadline, refusing even freshly signed
renewals that change those fields. Host authority and replay checks must still be
serialized with verification and commit; the result is not a reusable authority
cache. Tests now pass real Ed25519 and certified verification results through the
ledger. Evidence is under `target/verified-login/`. This integrates proof results
with reservation bookkeeping, not the full portable session engine or endpoints.
The complete Linux `make -j4 ci` gate passes (`ci.log`), including the compile-fail
construction test, both Dalek feature profiles, native/Wasm MSRV, real-canister
regressions, package dry runs and mocked release/recovery fixtures.

Portable Certified verification now reuses the existing Root/delegation and IC
signature verifiers under `token-verification`, without duplicating cryptography
or changing the old token signed domain. It requires shared portable context,
strict time checks, protected Root/network keys, exact certificate binding,
delegated audience/grants/windows and bounded aggregate proof material. Its IC
message framing domain is fixed as `ic-auth-portable-credential`. Tests reuse the
existing Root/BLS fixture producers and cover complete proofs, tampering, old
domain rejection, Root policy invalidation and input bounds. Evidence is retained
in `target/portable-certified/`. This completes the lower-level proof branches,
not the portable session/enrollment lifecycle or deployed logout flow.
Complete Linux `make -j4 ci` passes (`ci.log`), including native/Wasm MSRV, four
real-canister regressions, package dry runs and mocked release/recovery fixtures.
No package version, commit, tag or publication changed.

`MemoryLoginRevocations` now supplies bounded volatile bookkeeping for one
installed trust domain, with separate admitted/unknown pools, login-deadline
reservations, fixed skew-aware denial retention, monotonic clock checks and
bounded indexed pruning. Reservation/revocation commits only after the host
callback successfully commits its session/replay changes; errors retain records
and capacity. Hosts must serialize both sides and supply atomic callback behavior.
Tests exercise both orderings of admission/logout, callback failure, exact-retry
denial, pool flooding, retention boundaries and arithmetic failure. Evidence is
in `target/portable-revocations/`. This is not an installed logout endpoint:
portable proof/enrollment/session integration, a receiver-wide logout work budget,
startup/reset gates and durable storage remain required.
The complete Linux `make -j4 ci` gate passes for this batch (`ci.log`), including
native/Wasm MSRV, four real-canister regressions, package dry runs and mocked
release/recovery fixtures. No package versions or release refs changed.

Portable claims now have one pure protected-context checker for caller, issuer,
key, audience, grants, exact time windows, enrollment expiry and reset floors.
The optional Ed25519 verifier requires it before cryptography. Focused tests cover
both mechanism contexts, separate devices, issuer-scoped accounts, old-login
renewal after reset and timestamp boundaries. Evidence is retained under
`target/portable-rules/`. The complete Linux `make -j4 ci` gate passes in
`ci-retry.log` after explicit setup repaired the missing/invalid selected Testkit
CLI; the initial setup refusal is retained in `ci.log`. This does not implement
portable session admission:
certified-envelope integration, protected enrollment, atomic replay/tombstones
and browser cleanup remain required before the single session-engine cut.

Historical third review (custody restrictions superseded above) added live-key
snapshot take/read retirement even when
code/controllers are unchanged, explicit Root governance-chain trust and a
qualified NNS proposal/registry watcher for claims of pre-change suspension.
It requires `(issuer, account_subject)` session/account namespaces, typed future-
time retry without admission grace, jittered receiver refresh and a Root-outage
drill. Browser Capacity retries cover known admission windows but cannot equate
proof expiry with logout: a session may already have committed. Unknown renewal
and failed revocation retain bounded cleanup plus explicit uncertainty.
Both normative contracts now link wallet-origin and package-isolation gates;
direct threshold Ed25519 signing is explicitly wire-compatible, with a separate
non-TEE approval path still to qualify. The parenthesized residual formula and
existing tombstone/reset clock margins remain in force. ERC-4361 adoption remains
pending against the earlier protocol exclusion. These are documentation changes,
not implemented enrollment, retry endpoints or consumer adoption.

The maintainer's dependency update selects Curve25519 Dalek 5.0.0, Ed25519 Dalek
3.0.0 and K256 0.14.0. Auth adapts K256's always-returned normalized signature and
renamed SEC1 encoder without changing high-S rejection or signed formats. The
qualification producers use the same new encoder. Direct SHA-2 stays on 0.10.9;
the updated crypto graph also selects SHA-2 0.11.1, while older transitive crypto
versions remain required by IC dependencies. The existing Rust 1.88 native/Wasm
package matrix passes with these selections; no compiler-floor increase is needed.
Focused signature/token tests, Wasm and Clippy evidence is retained in
`target/crypto-upgrade/`. These checks qualify the dependency adaptation, not
portable session integration or a production TEE issuer.
The complete Linux `make -j4 ci` gate also passes for this adaptation, including
four real-canister cases, both package dry runs and mocked release/recovery
fixtures (`target/crypto-upgrade/ci.log`). The maintainer's manifest and lockfile
selections were preserved byte-for-byte; no package version or publication changed.

Historical second review (custody restrictions superseded above) made controller/
snapshot custody normative: reviewed
governance or qualified controllerless deployment, controller-only snapshots,
and retirement on every module-hash/controller change. It adds threshold-Schnorr
and batching tradeoffs, a separate future issuer crate, an explicit residual
duration formula, and deployment-qualified clock margins for reset/tombstones.
No production skew bound is yet qualified. Proofless logout stays bounded and
caller-authenticated; optional expired-proof admission is deliberately deferred.
ERC-4361-compatible Ethereum text without a SIWE dependency is proposed to retain
wallet domain warnings, but changes the maintainer's earlier protocol exclusion
and is pending approval. No wallet-origin protection is claimed from custom text.
These are design revisions, not new runtime behavior or a changed codec tag.

The optional `portable-signature-verification` capability now verifies ordinary
Ed25519 signatures over the canonical claims digest against protected host
selection of audience, issuer and public key. An explicit upstream scalar check
preserves canonical-S refusal even if downstream feature unification enables
Dalek's legacy scalar handling. The default graph remains free of this capability;
production code owns no signing, private-key storage or issuer crate. Returned
claims are not admitted sessions: protected enrollment provenance, replay,
revocation and atomic session integration remain required under #59. The later
shared-context step above now enforces time/caller checks in this verifier.
Independent Python/OpenSSL signature vectors and malformed/tampered/weak-key
cases exercise the primitive, including a noncanonical S + group-order signature.
The package MSRV fixture now extracts with fresh mtimes: normalized archive times
previously allowed shared Cargo output to reuse stale protocol-types metadata.

Verifier qualification passes the complete Linux `make -j4 ci` gate, including
both strict/legacy-feature test configurations, native/Wasm MSRV, Clippy, browser
checks, four real-canister regressions and local/mocked release fixtures. Final
evidence is `target/portable-verifier/ci-final.log`. The initial full-gate run
hit stale release-Wasm types metadata; scoped package artifact cleanup and a
complete successful rerun are recorded alongside the failed attempt. This is
local qualification, not production deployment or Canic adoption.

The first implementation step adds passive portable claims/mechanism types and
the Rust-owned bounded canonical codec in `ic_auth::portable`, including signing
digest and direct Ed25519 browser-principal derivation. Structural decoding never
establishes authority. Frozen byte/hash vectors, every-field digest binding,
truncated/oversized/invalid input and maximum-shape cases cover this foundation.
No new admission endpoint or parallel authentication flow is enabled: protected
enrollment, proof verification integration, login-level revocation and browser
transport still require implementation under Auth #59 and Canic #491.

Codec/protocol tests, the native/Wasm Rust 1.88 package matrix, Clippy, browser
checks, all four real-canister cases and local/mocked release/evidence fixtures
pass. Qualification startup now uses the incoming Testkit 0.40 owned-instance
API and retains the server through instance teardown, including setup failure.
Evidence is in `target/portable-claims/`. Earlier default-target package checks
failed to find the portable exports. The verifier batch corrected that diagnosis:
a fresh repository-local target builds both real package payloads successfully
with uploads aborted, proving the failure was stale build metadata rather than
an unavoidable published-version conflict. Evidence is retained in
`target/portable-verifier/package-isolated.log`; the failed attempts remain.
Coordinated 0.6 package/dependency version changes remain release-owned; no
versions, commits, publication or sibling source were changed.

Wallet authentication will use our own challenge format and Solana/Ethereum
adapters, without SIWS/SIWE, while retaining established signature primitives.
Exact wallet challenge encoding and adapter qualification remain unimplemented.
Wallet choice does not change the independent memory/persistent session profiles
or make account mappings and issuer keys volatile.

Review feedback is resolved in the [protocol contract](../design/0.6-auth-protocol.md)
and [TEE enrollment gate](../design/0.6-tee-enrollment.md). The application caller
is a fresh browser principal per login, separate from stable account identity.
Logout targets that authenticated caller without a proof and works after proof
expiry, absent establishment or issuer-key retirement. Fixed schema/domain/limits,
result types, server commit/recheck ordering and enrollment revision/generation
rules are design inputs, not new runtime capabilities.

The browser profile is one trusted frontend origin with durable generation and
logout intent before fan-out; cross-origin brokers are excluded. Memory reset
rejects immutable signed pre-reset login starts, including newly renewed proofs;
persistent normal upgrades retain tombstones. Backup recovery invalidates old
logins instead of trusting rollback state. All hosts reopen only after a fresh
authenticated authority snapshot; unavailable authority keeps auth closed.
This replaces the earlier optional durable-tombstone/restoration choices with
definite profiles. Current code's issuance floor alone is not sufficient.

The maintainer selected [two portable proof mechanisms](../design/0.6-portable-authentication.md)
into one session flow: existing certified/BLS admission and opt-in TEE-backed
Ed25519 admission. Only the issuer needs qualified confidential hosting for the
latter; receivers can run on ordinary subnets. Canic's proposed `auth-tee` feature
and protected config/key enrollment are consumer work in #491, not implemented
flags. The online issuer-call experiment below is retained evidence and is
superseded as the recommended multi-subnet architecture: routine receiver
admission must not require cross-subnet ICCs.

The browser will coordinate logout at known receivers and revoke issuer renewal,
with bounded expiry for missed destinations. Login-level revocation must reject
late admission and already-issued renewal credentials. Neither TEE nor browser
fan-out promises instantaneous global/all-device revocation. Local qualification
uses real cryptography without claiming hardware secrecy; production deployment
assurance remains separately required. Auth #59 owns protocol/cost acceptance,
Canic #491 integration, and Toko #1803 browser adoption.

Proof expiry is an admission deadline, not the current session lease deadline:
proofs are capped at 60 seconds and sessions at 1800 seconds by current policy.
The design preserves that separation and adds a proposed authenticated absolute
login deadline. Residual logout exposure includes late admission plus its lease;
the earlier conversational statement that sessions must expire with proofs was
too restrictive and does not describe the implementation.

The original design-review batch changed documentation and issue handoffs only.
The subsequent claims-codec implementation and its validation are recorded above;
neither establishes a Canic flag, TEE issuer, new session admission/storage format
or browser logout implementation. Maintained certified admission and incoming
implementation work are preserved.

## Pending 0.6 host-selected session storage

The [2026-10-14 issuer-call experiment](https://github.com/dragginzgame/ic-auth/issues/59#issuecomment-6163232483)
measures an alternative admission transport, not a second production flow. Two
local Testkit runs with three receivers each compare current budgeted portable
proof admission against a fixed-policy authenticated issuer-to-receiver call.
With matching size-optimized builds, cold admission averages 485.08M versus
1.74M measured Wasm instructions, and actual local cycle deductions average
498.11M versus 18.92M (issuer plus receiver). The roughly 26.3x cycle reduction
is a fixture result, not a production guarantee or mainnet latency measurement.
Actual rogue-issuer, caller/grant, replay, revocation and expiry refusals pass.
General Root enrollment, production SessionStore/lifecycle, uncertain delivery
and same/cross-subnet latency remain outside this probe's qualification. Its
bounded experimental record is not an alternate public Session constructor.
Source, raw logs, profiles, hashes and limitations are retained under
`target/issuer-rpc-experiment/`; the existing issue owns the protocol decision.
Maintained Rust and the current budgeted portable-proof contract are unchanged.

Fresh admission now requires a host-owned `SessionAdmissionBudget`, shared across
callers/endpoints and retained separately from session transaction rollback.
It starts empty, bounds accumulated/refilled verification attempts and charges
failed proofs and failed commits. Cheap capacity/eligibility/TTL/scope checks
precede cryptography; exact retries and existing sessions remain uncharged.
See the [host contract](../sessions.md#admission-work-budget) and
[Auth #59](https://github.com/dragginzgame/ic-auth/issues/59). This is an attempt
budget, not a claim that signup is Sybil-proof, all ingress work is bounded, or
honest admissions cannot be starved. Canic must commit debits on normal error
responses, cover every admission endpoint and qualify trap/upgrade behavior.
No alternate issuer-to-receiver protocol is implemented.

[Auth #58](https://github.com/dragginzgame/ic-auth/issues/58) owns the accepted
memory/persistent session storage contract. Hosts select an existing
`SessionStore` implementation at build time; IC Auth remains independent from
Canic configuration. Memory mode omits restoration and logs sessions out on
upgrades; persistent mode retains checked sessions, replay history and protected
authority together. The required new `SessionPolicy::min_proof_issued_at_ns`
checks the verified token issuance time on new admissions. After replay-state
loss, hosts supply checked trusted reset time + 1, refusing overflow or clock
rollback. A local generation increment alone cannot reject an old signed token.
The [lifecycle contract](../sessions.md) also covers mode changes and revocation.

The same valid permitted token admits separately on each receiver. Admission on
A does not create B/C sessions; B's reset does not invalidate A/C. Ordinary calls
never implicitly admit. Client recovery is scoped to receiver/identity/grants
and may require fresh issuance after expiry or a reset cutoff, without an
intrinsic extra wallet signature. Canic build configuration, real upgrade wiring
and client orchestration remain pending in the
[consumer handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6150481447).

Focused validation passes 19 minimal-session tests, the decoder-access compile
failure check, 21 restoration-enabled session tests, Wasm feature/dependency
checks, strict Clippy and isolated native/Wasm package MSRV 1.88.0 checks.
All four existing native canister qualification tests also pass. New signed
reset and multi-receiver regressions model isolated stores; they do not claim
Canic's actual post-upgrade implementation is qualified. Evidence is under
`target/session-storage-policy/`. No package version, release, full CI or sibling
source changes are claimed. Nanosecond units remain: fixed-width memory/signed
encoding is unchanged; CBOR seconds would save only four bytes per present-day
timestamp and would not remove the decoder.

## Pending 0.6 session restoration capability

`session-restore` now explicitly enables `Session::decode` and the private
session record deserializers. Plain `sessions` keeps admission, in-memory storage,
encoding and scoped authorization without that restoration code. The stored
format and corruption checks are unchanged. The qualification canister opts in;
Canic's durable adapter must do likewise, while other roles avoid forwarding the
feature. Cargo feature unification makes each production canister's resolved
graph and release artifact the consumer acceptance boundary; no universal
200 KiB saving is claimed. See [session capability selection](../sessions.md) and
[Canic's adoption handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6150227591).

Focused checks pass: 16 sessions-only tests, a compile-fail decoder-access check,
18 opted-in session tests, native/Wasm package MSRV 1.88.0 feature checks,
Wasm checks, dependency boundaries, strict Clippy and all four native canister
qualification tests. Evidence is in `target/session-restore-cut/`. Package
versions remain unchanged; no full CI, release or consumer adoption is claimed.

## Accepted 0.6 application-flow cut

On 2026-10-13 the maintainer selected **0.6.0** for a coordinated hard cut to
one application-authentication flow: complete-token admission into a caller-bound
local session, then scoped session checks on ordinary endpoints. The pending
0.5.11 notes now belong to the single 0.6.0 candidate; published history and
package versions remain unchanged. Earlier validation receipts below bind their
recorded source, not completed 0.6 consumer acceptance.

The library already implements that admission/authorization engine. Its standalone
token verifier remains a lower-level proof primitive; the endpoint guard and
frontend cut belongs to Canic and Toko under
[Canic #491](https://github.com/dragginzgame/canic/issues/491) and
[Toko #1803](https://github.com/dragginzgame/toko/issues/1803).
The [adoption contract](../canic-adoption.md) requires coordinated guard, Candid,
frontend and lifecycle changes, without a parallel per-call token fallback.
Neither the contract update nor the 0.6 notes claim those consumer changes are
implemented. Sibling checkouts remain read-only.

The [measured comparison](https://github.com/dragginzgame/canic/issues/491#issuecomment-6149634805)
uses real Wasm verification, the historical Canic 0.110.48 positive cache, and
heap/stable-read session probes. Its
[size attribution](https://github.com/dragginzgame/canic/issues/491#issuecomment-6149738317)
finds record decoding dominates the incremental size, not the stable-memory
allocation. The session engine does not call `Session::decode`; Canic's durable
adapter does when reading encoded records. Keep checked restoration and durable
replay obligations. This earlier contract-only decision did not change stored
CBOR or host lifecycle. The later accepted memory/persistent choice above adds
an explicit volatile-reset contract without changing stored bytes. A smaller
checked codec still needs its own evidence and retained-state contract. Auth [#43](https://github.com/dragginzgame/ic-auth/issues/43)
retains producer cost qualification.

This contract-only slice checks links, source attribution, snapshot integrity and
the diff. It does not rerun Rust/portable suites or claim new runtime acceptance.

## Browser encoding and checked session inputs (pending 0.6.0)

[Auth #55](https://github.com/dragginzgame/ic-auth/issues/55) encodes preparation
once, then decodes and validates the exact detached snapshot whose bytes are
stored for retrieval. Callback mutation of issuer-owned material cannot alter
that admitted snapshot. No issuance, CAS, deadline or generation boundary is
removed. [Auth #56](https://github.com/dragginzgame/ic-auth/issues/56) routes grant
and extension comparisons through the existing bounded Candid encoder; exact
bytes, option distinctions and typed material refusals remain qualified.

[Auth #57](https://github.com/dragginzgame/ic-auth/issues/57) carries the immutable
token, context and early canonical fingerprint in one private, non-deserializable
session input. Fresh verification reuses those facts and executes the same Root
and issuer signature owner as the public verifier. Standalone material validation
retains identity/certificate-before-hash order; session entry retains early size/
fingerprint admission and direct canonical error projection. Exact retries still
bound transport and recheck current session authority; no public unchecked API,
caller-supplied hash, mode flag or authentication shortcut is exposed.

Focused checks pass 33 token tests, 18 session tests and 54 browser cases, covering
both signatures, mixed-invalid refusal order, bounded retries, detached prepared
material, Candid limits and exact encoding. Evidence is under
`target/browser-token-reuse/`. The initial session test used equality on a
non-PartialEq verified value; the corrected test compares its typed error. The
initial mutation test acted before encoding; the final case targets validation.

Native release-profile measurements use the same signed fixture in isolated
before/after source copies, seven samples of 40 fresh admissions per workload.
Early sequential runs are retained and inconclusive. Four alternating ABBA
cycles yield 56 samples per source/shape: medians before/after are 1.7456/1.7477 ms
(one scope) and 1.7586/1.7623 ms (16 scopes of 64 bytes). Differences below 0.3%
do not establish a speedup; this is local native timing, not Canic Wasm cost or
deployment acceptance. `benchmark-balanced.json` binds sample order and binary
hashes; `benchmark.json` binds source/compiler/profile inputs. The first wide
fixture requested an ungranted scope and refused; its failed log is retained.
All 29 stages of `make -j4 ci` pass locally, including native/Wasm isolated
MSRV and feature graphs, real canister lifecycle/certification/ingress, browser
Candid/lifecycle, Clippy, package dry runs and local/mocked release recovery.
`ci-before.json` and `ci-after.json` match source bytes, modes, HEAD and index.
Native macOS acceptance and delivery/publication remain pending.
Incoming held source and Cargo.lock remain
preserved; no versions, index, refs, release or sibling changes are made.

## Borrowed profile checks, capacity and client-tool convergence (pending 0.6.0)

[Auth #52](https://github.com/dragginzgame/ic-auth/issues/52) removes temporary
session requests from controller-installed profile validation. Borrowed canonical
checks preserve count-before-grammar, all-label grammar-before-duplicate, then
ceiling-order and required-label refusal. No normalization or authority changes.
Real canister qualification passes nine additional mixed-invalid cases per
workload and confirms the previous admitted profile survives refused installation.

[Auth #53](https://github.com/dragginzgame/ic-auth/issues/53) shares session count
capacity decisions while preserving both admission and backend staging checks.
Direct backend tests bypass SDK admission and qualify exact replay/per-subject
quotas, replacement at full session capacity, refusal of a different caller, and
unchanged records, replay membership, counters and bytes after refusal.

[Auth #54](https://github.com/dragginzgame/ic-auth/issues/54) shares Node/npm pin
selection and exact version admission locally. CI setup still owns installation,
selected-prefix activation, dependency preparation and late PATH publication.
Offline checks never install. The actual Make/CI fixture includes the new helper,
selection failure and offline npm refusal, with first exit status and unrelated
installation/PATH preservation. Its absence assertions now explicitly exit on
unexpected events; standalone negation previously bypassed Bash errexit.

Focused session/client-tool checks and four real canister cases pass. The initial
session test compile failure captured a mutable fixture through a closure; a
fixed deadline removes that borrow without changing test inputs. ShellCheck
initially flagged a compound condition and preexisting fixture export/absence
checks; corrected ShellCheck passes. All 29 `make -j4 ci` stages pass locally,
including isolated native/Wasm MSRV, optional features, browser tests, real canister
qualification, Clippy, package dry runs and local/mocked release recovery.
`ci-before.json` and `ci-after.json` match bytes, modes, HEAD and index. Incoming
Cargo.lock is unchanged. Native macOS acceptance and release/publication remain
pending; no staging, manifest bump, commit, tag, push or deployment occurred.
Evidence and the incoming dirty tree are in `target/borrowed-capacity-client/`.
All prior held 0.5.11 changes and Host lock selections remain intact. No new public
API, dependency, wire/storage format, manifest version or sibling changes.

## Narrowing convergence and upstream extraction proposals (pending 0.6.0)

[Auth #51](https://github.com/dragginzgame/ic-auth/issues/51) consolidates token
preparation and verification role/scope subset checks in the existing grant-rules
owner. Preparation retains first-match narrowing before certificate hashing;
verification hashes canonical parents before narrowing. No public API, signed
bytes, storage format, dependency or validation order changes. Two public
boundary regressions cover duplicate parent roles and mixed-invalid errors.
The preparation regression also passed against the pre-consolidation source.
Focused `make test-protocol test-tokens` and all 29 stages of `make -j4 ci`
pass locally, including MSRV/native/Wasm feature graphs, real canister lifecycle,
Clippy, package dry runs and local/mocked release recovery. `ci-before.json` and
`ci-after.json` match source bytes, modes, HEAD and the index. The incoming lock
is unchanged. Exact-source macOS acceptance and delivery/publication remain pending.
Evidence is in `target/narrowing-convergence/`, with the incoming held 0.5.11
shape work and Host lock selections preserved in `before/`.

[Host #76](https://github.com/dragginzgame/ic-host-tooling/issues/76) requests
ownership review for the bounded file-command adapter, and
[Shared #159](https://github.com/dragginzgame/shared-tooling/issues/159) requests
review of pure Cargo manifest rendering/comparison. Both are proposals with
consumer contracts and retirement conditions, not implemented upstream APIs.
The existing Auth file CLI and release helpers remain their qualified runnable
owners until published replacements are adopted and tested. Siblings remain
read-only. No version, index, commit, ref, push or publication changes are made.

## Standalone shape validation (pending 0.6.0)

[Auth #50](https://github.com/dragginzgame/ic-auth/issues/50) adds the public
const-capable detailed label validator, canonical scope-list validation and
standalone bounded grant validation. Existing canonical errors, issuance/token
projections, signed domains and bytes remain unchanged. The internal grammar
has one owner, and existing certificate/token callers reuse the public grant
validator. There are no new features or dependencies in either library graph.

The [API contract](../certificate-issuance.md#standalone-scope-and-grant-shape-checks-pending-0511)
and [consumer handoff](../canic-adoption.md#pending-standalone-shape-apis-0511)
cover static and runtime diagnostics, empty configuration, explicit bounds and
raw-role projection precedence. Canic owns adoption and retirement through #491;
no sibling files or builds are changed. The compatible undated 0.5.11 changelog
is prepared; manifests and refs remain at 0.5.10.

Focused `make test-protocol` passes, including four new public boundary tests and
existing frozen encoding/preparation vectors. Evidence is in
`target/shape-validation-0.5.11/`; the initial compile failure used Display for
passive roles, which implement Debug instead. The corrected focused log is
`focused-final.log`. All 29 stages of `make -j4 ci` pass locally in `ci.log`,
including isolated native/Wasm package MSRV checks, all optional features, four
real canister cases, 52 browser cases, Clippy, package dry runs and local/mocked
release recovery. `ci-before.json` and `ci-after.json` match exactly, including
source bytes, modes, HEAD and the real index; the incoming Host lock is unchanged.
MacOS acceptance and 0.5.11 delivery/publication remain pending. No staging,
manifest bump, commit, tag, push, publication or deployment was performed.

## Shared Tooling 0.10.0 adoption (pending 0.5.10)

[Auth #48](https://github.com/dragginzgame/ic-auth/issues/48) adopts verified official
Shared main `ed9c26f015691859f67b50b512a94709dcdc0c6c` (0.10.0). The canonical
exporter uses an isolated clean checkout at that exact revision. Existing catalog
values and every IC digest match; no exception catalog is needed. Both catalogs
were already explicitly exported in the prior snapshot. Their redundant roots
are now removed because installers require them as companions: 66 roots still
produce 93 payloads. Make reads the catalog through the reviewed snapshot root;
standalone setup, formatting, cache inputs and evidence collection converge on
those defaults. Host overrides no longer implicitly select Rust pins.

The adopted 0.9.15/0.9.16 changes reject CR/LF directory aliases in maintained
helpers and preserve release-lock acquisition errors/status. Auth retains direct
release delivery. Optional CI-inspection/fleet scripts are not selected here.
Incoming Rust/Cargo selections and earlier handoff changes are preserved. Initial
refresh preparation refused a temporary source URL with an extra `.git` suffix,
then refused temporarily removed catalog destinations. Restoring unchanged old
payloads and using the manifest's exact remote resolved those checks; no ownership
record or shared payload was patched. Preview and repeated refresh succeed.
Evidence is retained in `target/shared-0.10.0/`. Explicit common/Testkit setup,
offline admission and locked source preparation pass without changing selections.
Five canonical catalog/distribution and two command/maintenance fixtures pass
on Linux/Bash 5.2.21. The first full gate rejected an absolute Make include in a
checkout path containing spaces; the relative root-layout include was restored,
and actual jobserver, Testkit caller and failure-evidence fixtures pass. That
failed gate remains in `validation.uZkQvE`. The corrected complete gate passes all
29 stages on Linux/Bash 5.2.21, including four real canister cases and local/mocked
release recovery. Logs: `target/shared-0.10.0/validation-logs/validation.zsdVVU`;
receipt: `target/shared-0.10.0/qualification-receipt.json`. All 58 sealed-input
observations match, with identical source bytes/modes, index and HEAD before/after
the gate. Incoming Rust/Cargo bytes remain unchanged. Final prose receives
separate link/snapshot/diff checks. Native macOS acceptance remains tied to the
future delivered source; no commit/version/tag/publication effects occur.
Compatible library patch 0.5.10
adds no API, signed/stored format or Canic adoption prerequisite.

## IC Metrics qualification summaries (pending 0.5.10)

[Auth #49](https://github.com/dragginzgame/ic-auth/issues/49) adds published IC
Metrics 0.9.0 only to the unpublished native runner's dev-dependency graph.
`MeasurementSummary` owns counts, checked floor means and maxima for five
consecutive allowed queries per installed profile. One-scope and maximum-width
records have independent decode/authorization/total instruction series. Raw
observations and individual budget checks remain; refusal/recovery samples are
not pooled. The measured canister source/graph and public library graphs remain
unchanged. No histogram, store or runtime observer is added to authentication.

The official unyanked registry checksum matches
`f37740e17c25ed7792cad59abf4f72670ca29b85b2ae9d1f8cd022ff1b6a7e18`.
The lock adds exactly Metrics and its runner edge; all previous external
selections and incoming Shared/Cargo work are preserved. All four focused real
canister cases pass; six summary lines match their five raw allowed observations
in each workload/segment. All 31 raw observations remain visible. Dependency-tree
checks confirm Metrics is absent from the all-feature Auth and measured-canister
normal/build graphs. The first complete gate refused a concurrently locked shared
IC tool installation before compilation; the later standalone tools check passes
after that owner finishes. Failed evidence remains in `validation.TVrZim`.
The final complete gate passes all 29 stages on Linux/Bash 5.2.21, including four
real canister cases. Logs: `target/metrics-0.5.10/validation-logs/validation.UWtrJW`;
receipt: `target/metrics-0.5.10/qualification-receipt.json`. All 58 sealed-input
observations match and source bytes/modes, index and HEAD are identical across the
gate. Final total instruction floor means/maxima are 53,052 for the one-scope,
776-byte record and 211,095 for the 16-scope, 1,827-byte record, five samples each.
These use message counter 0; they exclude consumer context/stable/transport costs
and do not establish Canic's facade budget. Focused observations retain their own
record bytes and inputs. Final prose is checked separately; delivery/native
macOS acceptance remains pending, with no sibling or release effects.

## Current Canic integration review

The preceding source review binds clean local Canic
`ce45935307e1f95a7f72ea3e338b19b509e17210` (0.112.0 preparation). Remote main
remains `25b73cd403b20b4e148c6e340f8c760a26d97f6e` (0.111.0 release). The root
lock selects published Auth/protocol-types 0.5.8; both private Blob consumer
locks select 0.5.7. The previous held/dirty checkpoints remain historical.

The later issue follow-through sees the same Canic HEAD with dirty Cargo/status
work and root Auth/protocol-types 0.5.9; both Blob locks still select 0.5.7.
[Canic #525's concrete denial matrix](https://github.com/dragginzgame/canic/issues/525#issuecomment-6143196133)
traces the production facade through protected verifier resolution before the
Auth session lookup. Every verifier/TTL resolution error maps to StaleAuthority,
including combinations with missing or expired sessions. The smaller pure probe
would return MissingSession/Expired when given its local snapshot; it cannot
represent Root-key resolution failure. Canic's normative denial-order wording
therefore needs an explicit production contract decision and actual-facade
combined-failure qualification. This is a source-traced diagnostic/precedence gap,
not a reproduced authorization bypass or a new Auth API requirement. The owning
issue has the concrete matrix and repair choices; no duplicate live checks or
weakened Auth verification are proposed. Nine inspected file hashes are retained
in `target/canic-followup-20261013/denial-source.json`.

The current committed source still has early receipt/local-intent cleanup
failure before session pruning (#419), discarded installed issuer deadlines
(#506), the pure denial probe rather than actual SDK-facade denial acceptance
(#525), full-state logout rewriting (#380), and fresh preparation rather than
non-issuing browser reconciliation (#507). No new producer API prerequisite or
runtime defect reproduction emerges from this inspection. Earlier owner
performance/storage results retain their reported inputs; they do not qualify
all current lock graphs. Canic can continue on its selected compatible releases.

Publication, issue observations and fourteen source hashes are retained in
`target/integration-0.5.9-live/`. The review follows the adopted flow-convergence
method and checks existing owners without creating a parallel tracker. Only
Auth guidance/status is updated. Incoming Cargo bytes, index and HEAD are
preserved; local links, snapshot integrity and diff receive prose-only checks.
No sibling edits/builds, compiled Auth gate or release effects occur.

[Canic #533's Auth storage follow-through](https://github.com/dragginzgame/canic/issues/533#issuecomment-6144678469)
compares released 0.111 with committed `0d6779da5d9f1f7e92ef532ef32663f28fff4677`:
the old authority cell contains session/replay arrays; the candidate separates
rows, requires shared session bytes and rejects unknown header fields. Both name
the same authority memory key, so exact physical mapping/retained-byte disposition
must be qualified before cross-build restoration. Missing SDK Root provenance
must not be invented, and session invalidation does not permit forgetting live
consumed proofs. Five committed source hashes are in
`target/shared-0.10.0/canic-upgrade-source.json`. This is consumer storage evidence,
not a migration implementation, live reproduction or new producer API request.

## Released 0.5.9 simplification evidence

[Auth #46](https://github.com/dragginzgame/ic-auth/issues/46) consolidates protected
scope-ceiling validation in token verification and live session authorization on
the existing canonical validator. No new policy type, public API or cache is
introduced. The session boundary retains its InvalidPolicyOrRecord mapping for
noncanonical order; label errors and earlier authority/expiry checks retain their
precedence. Request normalization/count bounds, stored-record validation and
required-scope membership remain separate because their contracts differ.

The focused review applies the adopted flow-convergence method to those three
validation owners at released source `12bae9fa3d16209b427f0e9f2f558b9bdd452969`.
The maintenance concern is repeated grammar/order logic, not a reproduced
acceptance bug or measured Canic performance improvement. Characterization tests
pass before and after the change through token verification, authorization and
exact retry, including mixed-invalid errors and unchanged retained sessions.
Canic's host cleanup, renewal, facade-denial, logout and browser repairs remain
with its existing #419/#506/#525/#380/#507 trackers; none requires a new producer
API from this compatible patch.

The complete `make -j4 ci` gate passes all 29 stages on Linux/Bash 5.2.21,
including four real canister cases, package/compiler qualification and local/mocked
release recovery. Logs: `target/complexity-0.5.9/validation-logs/validation.Y2hdUi`;
receipt: `target/complexity-0.5.9/qualification-receipt.json`. All 58 sealed-input
observations match. Source bytes/modes, index and HEAD match from the early
tools-check checkpoint, taken before compilation, through completion. The attempted
pre-entry capture referenced removed prior-release scratch files; this receipt
does not claim a gate-entry checkpoint or Bash 3.2 execution. Final prose receives
separate link/snapshot/diff checks. No consumer performance benefit or native
macOS acceptance is claimed; package versions and release refs remain unchanged.

The follow-up scope review removes unreachable aggregate byte accounting from
`session_request::check_scopes`: its 16-label count guard and canonical 64-byte
label guard already enforce the 1,024-byte bound. Both request construction and
trusted record decoding retain that check, their distinct ordering rules and
error precedence. Existing exact-bound/hash cases and a 1,025-byte rejection case
pass with protocol/session checks. This continues #46 without a public contract
change; no consumer speedup is claimed. The follow-up complete gate passes all
29 stages on Linux/Bash 5.2.21, with 58 matching sealed-input observations and
identical source bytes/modes, index and HEAD before/after execution. Logs:
`target/scope-bounds-0.5.9/validation-logs/validation.EGfe24`; receipt:
`target/scope-bounds-0.5.9/qualification-receipt.json`. Final prose is checked
separately for local links, snapshot integrity and diff whitespace.

The next bounded review, [Auth #47](https://github.com/dragginzgame/ic-auth/issues/47),
removes unchanged retention-map/index writes on live signature preparation retry.
Input, clock, retention and overflow checks still precede the return, and successful
retries still advance the accepted clock. Expired replacement and new-key capacity
checks retain their paths. Before/after characterization covers live retry at
capacity, invalid retention/overflow without clock advancement, later clock
regression and exact expiry pruning. Existing composed-witness tests also pass.
This reduces mutation work without a measured Canic speedup or public API change.

Incoming Cargo selections now use direct Host 0.17.0 and Testkit 0.36.0 with its
Host 0.16.0 graph; those incoming bytes are preserved by this cleanup. Offline
Testkit admission initially refused the uninstalled selected CLI; explicit
`make install-testkit-tools` then prepared it and server 16.1.0. The failed check
and setup logs remain in `target/signature-retry-0.5.9/`. All 29 complete gate
stages pass on Linux/Bash 5.2.21, including real canister qualification on this
graph. All 58 sealed-input observations match; source bytes/modes, index and HEAD
are identical before/after execution, and incoming Cargo bytes are preserved.
Logs: `target/signature-retry-0.5.9/validation-logs/validation.zBW9Jr`; receipt:
`target/signature-retry-0.5.9/qualification-receipt.json`. Final prose receives
separate link/snapshot/diff checks. Earlier receipts describe their older inputs;
this local run does not establish native macOS or Canic consumer acceptance.

## Released 0.5.8 Shared/Host preparation evidence

The requested latest committed Shared snapshot is 0.9.13
`35cad65677ff989b5fb24a88d947fda37d2366e8`, verified against official main.
The sibling's dirty 0.9.14 work is excluded. Exact-source canonical export retains
68 roots/93 payloads; preview preserves incoming source and Git state, and repeat
refresh is identical. Shared #145/#146 preserve actual lock/source-acquisition
errors and statuses; #147 permits separate locked source preparation. Auth does
not select the optional CI inspector changed by #148.

Incoming Cargo files already select direct Host filesystem/artifacts 0.16.0 and
Testkit 0.35.1, with Testkit's Host fs/artifacts/process 0.15.2 and server 16.1.0.
All six official unyanked registry checksums match. Locked fetch succeeds without
changing selections. Host 0.16's feature change is confined to `ic-host-tools`,
which Auth does not select; its filesystem calls require no source migration.
[Auth #45](https://github.com/dragginzgame/ic-auth/issues/45) tracks native pairing
qualification, while [#44](https://github.com/dragginzgame/ic-auth/issues/44)
continues the unshipped Shared adoption. Public library contracts, own package
versions and the existing 0.5.8 qualification work remain unchanged.

Explicit common/selected tool setup and separate offline admission pass. Four
canonical fixtures pass on genuine Bash 3.2.57: Rust tools, Host tools/store,
IC tools/store and snapshot distribution. An initial direct invocation of the
sourced store companion failed without its parent fixture; that log is retained,
and the owning Host/IC fixtures supply the valid passing evidence. ShellCheck passes.
The complete Linux gate passes all 29 stages, including four real canister cases,
52 browser tests and release recovery. Logs:
`target/upstream-0.9.13/validation-logs/validation.NlpFFN`; receipt:
`target/upstream-0.9.13/qualification-receipt.json`. All 58 sealed observations
match; source bytes/modes/index/HEAD remain identical across the gate. Incoming
Rust/Cargo bytes are preserved. Final prose receives separate link/snapshot/diff
checks. Native macOS acceptance remains tied to the future pushed source; no
sibling edits/builds or Git/version/release effects occur. Compatible 0.5.8 is
locally ready.

## Shared 0.9.10 adoption and Canic audit (pending 0.5.8)

[Auth #44](https://github.com/dragginzgame/ic-auth/issues/44) tracks exact committed
Shared Tooling 0.9.10 at `7e55adecf6f0b196c050f104b37f8c69b6bf434e`, verified
against upstream main and VERSION. The canonical exporter selects 68 roots/93
payloads, adding the directly used Cargo cache identity and change-reporting rule.
Preview preserves incoming source/index/HEAD and repeated refresh is identical.
CI restores reusable builds before explicit setup/admission and saves verified
builds on main pushes before tests; PRs only restore. The new strict Cargo offline
configuration/build policy requires explicit common/selected tool setup, which
passes with separate offline admission. All incoming Cargo selections and prior
session qualification source are preserved. Four canonical Bash 3.2.57 fixtures
pass: Cargo tools/cache, PR release, fixture retention and snapshot distribution.
The complete Linux/Bash 3.2.57 gate passes all 29 stages, with four real canister
cases, 52 browser tests, package/MSRV checks and local/mocked release recovery.
Logs: `target/shared-0.9.10/validation-logs/validation.gRNlBv`; receipt:
`target/shared-0.9.10/qualification-receipt.json`. All 58 sealed-input observations
match; source bytes/modes, index and HEAD remain unchanged across the gate.
Incoming Rust/Cargo bytes are preserved and no duplicate-recipe warnings occur.
Checked tracked-diff SHA-256:
`c8e6f8be3b20237bf6ce7e3663628a17e736e47dcbe28aa096f71728c530a42c`.
Actionlint and ShellCheck pass. Final evidence edits are prose-only and checked
separately. Compatible 0.5.8 preparation is locally ready; native macOS and hosted
cache reuse require its future pushed source. Package versions and release refs
remain user-owned.

The [current Canic audit](https://github.com/dragginzgame/canic/issues/491#issuecomment-6141222338)
observes local base `3914b477f60c51cfcbbfafcaf09911f9ddcee9b2` (0.112.0) plus
held changes, while official main remains released 0.111.0. Its root and both
private Blob consumer locks select Auth/protocol-types 0.5.7. Production SDK establishment/authorization and keyed
stable pair admission are present; the owner reports maximum authorization,
cleanup/recovery and selected native/Wasm passes. Actual-facade denial acceptance
remains #525, cleanup custody #419, logout cost #380 and non-issuing browser
reconciliation #507. The audit also reconfirms #506: installation drops typed
effective-deadline evidence and renewal state retains declared deadlines. Existing
trackers own repairs; no new producer API prerequisite is established. Thirteen
reviewed source hashes and issue observations are retained under ignored
`target/canic-audit-20261013/`. Sibling source/builds remain untouched.

## Current continuation (pending 0.5.8)

The maintainer requests work on the recorded integration issues. Sibling code
stays read-only; Canic retains storage/cleanup/logout/browser ownership. The
Auth-side producer measurement is tracked in
[Auth #43](https://github.com/dragginzgame/ic-auth/issues/43), supporting
[Canic #525](https://github.com/dragginzgame/canic/issues/525). The internal
canister now measures actual Wasm session decode/live authorization with
controller-installed record/context, using records from complete certified-token
admission. No library source or wire/stored format changes. The only lockfile
edit adds inherited serde to the unpublished runner; all incoming registry
versions stay unchanged. Current registry checks authenticate Testkit 0.35.0
and Host 0.15.1 against official unyanked entries and cached archives.

Focused real qualification passes all four cases with one-scope and maximum-width
records. Actual caller/time, Root key/deadline, scope, generation, missing/disabled
and exact-expiry refusals use the shared engine and a strictly read-only backend.
Initial compiler/replica-clock fixture failures are retained under ignored
`target/session-cost/`; the corrected expiry case advances a replica round before
query observation. These measurements exclude consumer context/stable projection
and are not a Canic facade pass. The final focused run measures about 53k
instructions for a one-scope record and up to 292k for the maximum-width record.
The complete Linux/Bash 3.2.57 gate passes all **29 stages**, including native/
Wasm Rust 1.88 public package paths, four real canister cases with 25 measured
SDK decisions, 52 browser tests and release recovery. Logs:
`target/session-cost/validation-logs/validation.i2vp2T`; receipt:
`target/session-cost/qualification-receipt.json`. All 58 sealed-input observations
match and source bytes/modes/index/HEAD remain unchanged. Checked tracked-diff
SHA-256: `828c6477ca0790d24d75d25b6208c2b4226fd319b53e769458a6d64b96d28747`.
The first gate's formatting fixture lacked new module companions; the corrected
Make overlay passes and the original failure is retained. No duplicate-recipe
warnings occur. Final evidence edits are prose-only, with separate links,
snapshot and diff checks. Compatible **0.5.8** preparation is locally ready;
package versions and public APIs/formats stay unchanged. Native macOS acceptance
belongs to its future pushed source. No Git/release/publication effects occur.

## Earlier Canic integration review (2026-10-12)

The [owning tracker checkpoint](https://github.com/dragginzgame/canic/issues/491#issuecomment-6129463090)
records current source progress over dirty base
`25b73cd403b20b4e148c6e340f8c760a26d97f6e`. Canic's held lock selects published
Auth/protocol-types 0.5.6. The production facade now runs shared authorization
before Canic-specific subject/scope denial classification, and admission commits
one stable session/replay pair with incremental indexes. Earlier duplicate
precheck observations are historical. Retained owner storage/Clippy/managed
recovery logs include passes; the changing source and local Testkit 0.35.0
override are not released consumer acceptance. Older Wasm logs retain Auth
0.5.5 inputs. No sibling build/edit is performed by this review.

[Canic #525](https://github.com/dragginzgame/canic/issues/525) records the managed
earlier maximum-state authorization failure. The
[owner now reports](https://github.com/dragginzgame/canic/issues/525#issuecomment-6130059637)
989,121 maximum-state instructions with the 1,000,000 limit unchanged; one/median
and bounded cleanup/managed recovery cases also pass at those held inputs.
Actual-facade single/combined denial qualification remains open. These are
consumer-owner observations, not an Auth rerun or deployed acceptance.
Actual-facade denial qualification is
separate from the retained pure-value probe. The existing
[cleanup issue](https://github.com/dragginzgame/canic/issues/419) is updated with
current early-return/timer-stop custody, and
[whole-state performance finding](https://github.com/dragginzgame/canic/issues/380)
now distinguishes keyed admission from the retained bulk logout path.
[Proof-start disposition](https://github.com/dragginzgame/canic/issues/418)
preserves Auth's intentional strict starts and separates Canic's retry diagnostic.
Browser non-issuing reconciliation remains
[Canic #507](https://github.com/dragginzgame/canic/issues/507)/
[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3). No new Auth library
bug or API prerequisite is established. Source hashes and retained logs are
captured under ignored `target/canic-integration-review/`; observed drift is
explicit, and older logs are not attributed to every later source byte.

This review changes current Auth delivery/adoption prose only. No package
metadata, compiled source, snapshot payload, Git index/ref or finalized changelog
changes occur. Local links, snapshot integrity and the diff are checked; no Rust
or portable suite is run for prose. Publication is verified read-only; native CI
has passed Linux, with macOS pending. The following preparation receipts retain
their historical
input identities.

## Released 0.5.7 preparation evidence

The maintainer requests continued Auth work and the latest Shared Tooling.
[Auth #42](https://github.com/dragginzgame/ic-auth/issues/42) owns adoption of
committed Shared **0.9.6 `7e9f062f53baad2059a8585e2fffe2b98f03bafd`**, matching
verified upstream main and VERSION. An isolated clean exact-source exporter
selects 66 roots/91 payloads, adding the directly called artifact discovery
helper. Preview preserves source bytes/modes/index/HEAD; repeated no-option
refresh is identical. Eleven existing payloads change, one is added and none
retire. Incoming 0.5.6 Cargo bytes and all existing library contracts remain
unchanged; compatible **0.5.7** notes do not change package versions.
The clean incoming release already contains direct Host filesystem/artifacts
0.15.0 and Testkit 0.34.7 (transitive Host 0.14.10, server 16.1.0); the earlier
0.14.10/0.34.6 preparation receipt below is historical. All three newer package
checksums match official unyanked registry entries and cached published archives.
Offline tool/CLI/server admission passes for the actual selection; no setup or
dependency resolution is needed.

Auth CI now waits for exact-ID discovery before its pinned download transport,
with bounded retries and a two-minute step limit. Its actual workflow command
is exercised by the existing tool-evidence gate with substituted API/sleep
effects, covering delayed success, exhaustion and terminal identity/API errors.
Shared also supplies bounded host/IC download retries, complete Git-ID lengths
and strict PR merge strings; Auth retains direct delivery. Six focused canonical
fixtures pass on genuine Bash 3.2.57, with substituted Git/API/install effects
and real curl classification over loopback HTTP. Auth's maintained workflow
fixture, ShellCheck 0.11.0 and Actionlint 1.7.12 pass too.

The complete Linux/Bash 3.2.57 gate passes all **29 stages**, including native/
Wasm Rust 1.88 package paths, four real canister cases, 52 browser tests, package
dry runs and local/mocked release recovery. Logs:
`target/shared-0.9.6/validation-logs/validation.NyrnWZ`; receipt:
`target/shared-0.9.6/qualification-receipt.json`. All 58 observations of eight
sealed inputs match; source bytes/modes and real index/HEAD remain unchanged.
Incoming Cargo bytes and all application/library/client source remain preserved.
Checked tracked-diff SHA-256:
`bbff896728310b98f8d5b5a559b2b9018d5dd8f9fdc50f5ef1df1a8bd6ca8689`.
No duplicate-recipe warnings occur. Final evidence edits are prose-only and
receive separate links, snapshot and diff checks. Compatible **0.5.7** preparation
is locally ready; native macOS acceptance belongs to its future pushed source.

Canic feedback adds no new producer API prerequisite. Consumer durable
acceptance remains [#491](https://github.com/dragginzgame/canic/issues/491), and
browser non-issuing reconciliation remains
[#507](https://github.com/dragginzgame/canic/issues/507)/
[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3). Siblings stay read-only;
no sibling build/edit or user-owned Git/version/release effects occur.

## Released 0.5.6 preparation evidence

The following receipt describes preparation before maintainer delivery; pending
labels below retain their historical source identity.

The maintainer confirms 0.5.5 is live and requests **0.5.6** notes plus continued
Canic integration work. The compatible undated changelog collects the incoming
native/build graph and reconciled consumer guidance; APIs, formats, compiler
floor and own package versions remain unchanged.
[Auth #41](https://github.com/dragginzgame/ic-auth/issues/41) records the exact
incoming Host 0.14.10/Testkit 0.34.6 and Syn/AWS-LC selections. Official checksums
match all eight unyanked packages; Host/Testkit production Rust is unchanged,
with only upstream test isolation changes. Explicit locked fetch passes and
initial offline CLI admission correctly refuses the unprepared selection.
Explicit setup and separate offline CLI/server admission succeed. The complete
Linux/Bash 3.2.57 gate passes all **29 stages**, including public package native/
Wasm Rust 1.88 paths, four real canister cases and 52 browser tests. Logs:
`target/continuation-0.5.6/validation-logs/validation.DHBjKa`; receipt:
`target/continuation-0.5.6/qualification-receipt.json`. All 58 declared-input
observations match; the source byte/mode checkpoint, real index/HEAD and incoming
Cargo files stay unchanged. Compiled code also matches the incoming capture;
all earlier prose work is preserved. Checked tracked-diff SHA-256:
`f055d8ec4e59c168919194eac5e60d6d963e4c5db54643a2d5a04a69dbe49182`.
No duplicate-recipe warnings occur. Final evidence edits are prose-only and
receive links, snapshot and diff checks separately. Earlier source gates remain
historical. Compatible **0.5.6** preparation is ready; native acceptance of this
source/lock remains separate after maintainer delivery.

### Earlier issue review and current consumer contracts

The maintainer now prioritizes issues and Canic adoption. Twelve completed
producer issues are closed with source-bound delivery, retained native behavior
and existing consumer evidence. Earlier cancelled/failed release runs remain
historical; closure does not relabel them as passes. The pure preparation,
certificate, chain-key and request APIs are delivered and adopted in Canic's
held graph; full session/lifecycle and consumer release acceptance stay in
[Canic #491](https://github.com/dragginzgame/canic/issues/491) and its owning
trackers. No new producer API prerequisite emerges from current feedback.

The [read-only verifier review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6126777217)
binds Canic's unchanged committed base `25b73cd403b20b4e148c6e340f8c760a26d97f6e`
and dirty root Auth 0.5.4 graph. Complete token verification now calls the SDK
with protected token scopes, enrolled key/network policy and finite bounds;
the copied verifier is removed. Root/issuer wrappers share one SignatureStore
host. The new durable store adapter is present. A
[later source checkpoint](https://github.com/dragginzgame/canic/issues/491#issuecomment-6126878501)
replaces the local admission workflow with a call to `establish_session`; the
endpoint reaches that wrapper and a new authorization wrapper calls
`authorize_session`. Backend and guard-call propagation, coherent compilation
and durable transaction/lifecycle acceptance remain Canic-owned. The final wiring must
retain atomic session/replay commits, protected generation, independent replay
expiry and owner-controlled retained-state disposition. Browser non-issuing
reconciliation remains [Canic #507](https://github.com/dragginzgame/canic/issues/507)
and [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).

The Metrics-reported verifier-only Wasm failure is traced to two Canic-owned
capability declarations. A concrete source proposal is posted to #491. The owner
changes those files during review to the create-or-test conditions, preserving
the enabled metric label. The updated source matches the proposal; compilation
and rerunning the blocked case remain owner acceptance, not an Auth pass.
All sibling files remain read-only and no sibling build runs. The Auth adoption
guide and release descriptions now reflect current source and publication.
The earlier issue-review slice changed prose only and was checked with links,
snapshot integrity and the diff. The new incoming dependency graph requires its
own complete validation under #41. No sibling build/edit or Git/release effects
are performed; package versions remain 0.5.5.

## Released 0.5.5 preparation evidence

The following receipt describes preparation before maintainer delivery. Pending
versions and local evidence below retain their original source identity.

[Auth #39](https://github.com/dragginzgame/ic-auth/issues/39) now advances the
same compatible batch to committed Shared Tooling **0.9.2
`c118062a070a2e238f1eb3989b6fd700f369ce1e`**, matching official remote main
and source VERSION. The sibling is clean; an isolated exact-source exporter
reuses 65 roots/90 payloads, updates six files and adds/retires none. Preview
preserves all source bytes/modes/index/HEAD; repeated no-option refresh is
byte-identical. Incoming Cargo files and the prior CI repair remain preserved.

Shared 0.9.1/0.9.2 consolidate host/IC installer cleanup, release the shared lock
after checkout cleanup failure, retain original installation errors/candidates,
reject NUL before evidence hashing and reject supplied/resolved CR/LF consumer
paths before configuration or setup. Four focused canonical fixtures pass on
genuine Bash 3.2.57 with substituted installation effects; fleet-only helpers
remain upstream. This is not native macOS execution evidence.

[Auth #40](https://github.com/dragginzgame/ic-auth/issues/40) qualifies the incoming
lock, which independently advances four Host packages **0.14.8 -> 0.14.9**
and Testkit **0.34.4 -> 0.34.5**. It is retained unchanged. Explicit locked fetch
passes; initial offline CLI admission correctly refuses the unprepared selection.
Sandboxed setup attempts refuse shared-store writes and are retained; no existing
lock is removed. Setup through the documented installer with appropriate store
access succeeds. All five official sparse-index checksums match unyanked packages;
authenticated published archives contain no Rust-source changes from the prior
pairing. Separate offline admission succeeds for server 16.1.0.

The initial complete gate stops at qualification because its sandbox prevents
the verified server from binding localhost; all four startup refusals are retained
in `target/shared-0.9.2/validation-logs/validation.UOzxlj`, with unchanged source
and inputs. A fresh gate with the required local-server access passes all **29
stages** on Linux with genuine Bash 3.2.57, including all four real canister
journeys and 52 browser tests. Logs:
`target/shared-0.9.2/validation-logs/validation.rxbvla`; receipt:
`target/shared-0.9.2/qualification-receipt.json`. All 58 input observations match
the eight-file baseline; every tracked/new source byte and mode, real index and
HEAD, and incoming Cargo files remain unchanged across validation. Checked
tracked-diff SHA-256 is
`6833e0f65b172f66aa7ef0cc2b408a3a1784a56e518f4d185c5f39fcbb3d6436`.
No duplicate-recipe warnings occur. Final evidence edits are prose-only and
receive links/snapshot/diff checks separately.

The combined batch is ready for compatible **0.5.5** preparation: public APIs,
formats and compiler requirements remain unchanged. Native CI for this pending
source remains separate from released 0.5.4's completed matrix. No sibling
source/build or package version, staging, commit, ref or release effect changes.

The following 0.9.0 preparation receipt remains historical input evidence:

[Auth #39](https://github.com/dragginzgame/ic-auth/issues/39) adopts committed
Shared Tooling **0.9.0 `aec7a149f581a42b9531fc747b46024fc0314fcf`** from an
isolated clean exact-source checkout, matching official remote main/source
VERSION. Dirty sibling 0.9.1 installer cleanup is excluded. Canonical export
reuses 65 roots/90 payloads: only the verifier and snapshot guidance change.
Preview preserves source/index state, and no-option preview/refresh reuses roots
with byte-identical payload/manifest bytes. Incoming Cargo files stay unchanged.

The unified reader validates one captured manifest before emitting records,
rejects NUL/trailing fields and keeps payload integrity/ownership separate.
Four focused canonical export/retirement/tooling-reader fixtures pass on genuine
Bash 3.2, separately from Auth’s actual syntax-only and integrity checks. Actual
NUL/extra-field probes refuse with no partial record output; valid Auth records
and normal snapshot verification pass. No fleet helper/gate is added here.

[Auth #38](https://github.com/dragginzgame/ic-auth/issues/38) repairs CI preparation
routing: exact npm installs into the selected Node installation, PATH selects it
through Make, and the admitted bin directory is published for later steps after
complete preparation. Versions/paths/prefixes are retained; workflow strict mode
preserves the first failure through tee. Ordinary checks retain exact admission
and gain observed-version/path diagnostics, without installing implicitly.
The actual workflow block and Make recipes pass selected-prefix/later-step,
wrong-Node, setup-failure, wrong-prepared-npm, dependency-failure and missing-path
controls on Bash 3.2 with substituted npm effects. Native macOS root cause/repair
acceptance is not claimed from these Linux fixtures; its failed job omitted
actual versions/paths. Release-fixture overlays include the new maintained inputs.

The new `test-client-tools` target extends the complete gate to **29 stages**.
The earlier 0.9.0-source validation passes on Linux with genuine Bash 3.2.57:
`target/shared-0.9.0/validation-logs/validation.zJKKw4` contains all **29 PASS**
results, including four canister qualification tests and 52 browser tests.
All 58 before/after input observations match the eight-file baseline; every
tracked/new source byte and mode, index and HEAD stays unchanged across the gate.
Incoming Cargo manifest/lock bytes remain identical. The retained receipt is
`target/shared-0.9.0/qualification-receipt.json`, with checked tracked-diff SHA-256
`0e4bf5caaa2853c6c5d086e50843bf2b6a85f2299fd33ac74fbbb88e000dec40`.
No duplicate-recipe warnings occur. Final evidence edits are prose-only and
receive link, snapshot and diff checks separately; no hosted run is inferred for
this pending source from the released 0.5.4 run.
Library APIs, signed/stored formats, tool pins and Rust floor remain unchanged;
the compatible candidate is **0.5.5**. Versions/index/commits/refs/release effects
remain maintainer-owned. Consumer full-verifier/session and browser-reconciliation
contracts remain in their existing Canic trackers; no sibling source/build is
changed here.

## Released 0.5.4 preparation evidence

The following receipt describes preparation before maintainer delivery. Its
pending versions and native observations are historical; current delivery is above.


[Auth #37](https://github.com/dragginzgame/ic-auth/issues/37) now advances the same
compatible batch to committed Shared Tooling **0.8.4
`dca2bdd018afa6be598f03bb3cce1bab79c66973`**, matching official remote main and
its source `VERSION`. Dirty sibling 0.9.0 manifest-reader work is excluded, and
no release tag is inferred. The isolated clean canonical exporter reuses the
same 65 roots/90 payloads: only `scripts/dev/cloc.sh` and snapshot guidance update,
with no addition or retirement. Preview preserves all consumer bytes/modes and
index/HEAD; no-option preview/refresh reproduces identical payload/manifest bytes.
Incoming Cargo files and every unrelated pending change are preserved.

The first refreshed gate correctly stops at `test-host-tooling` after an external
update changes only Cargo.lock's four Host selections **0.14.7 -> 0.14.8** during
validation. Cargo's locked refusal and the declared-input checkpoint preserve
the failure; no later stage runs. Original lock hash
`67c79bb6385d9f2860458cf30f96c48991095ef1de277be2d689cf1d3976e11b`,
observed hash `b1ee15e0181480939a6d2b500a80334830c0e1fa6954df2ffd8d1c9c0de83f25`.
The external lock is retained rather than reverted. Explicit `make fetch` and
locked offline metadata now pass without changing its bytes or the manifest.
All four official Host 0.14.8 checksums match unyanked packages; their Rust sources
are unchanged from 0.14.7. The exact pairing is requalified with Testkit
0.34.4/server 16.1.0; prior passing stages do not qualify the new lock. Both lock captures, narrow diff,
failed gate and cache-preparation evidence remain under `target/shared-0.8.4/`.

The adopted LOC helper adds numeric JSON rows/totals without changing the human
table. Focused canonical LOC/aggregation fixtures pass on genuine **Bash 3.2.57**,
including independent workspace/output selection and a count failure returning
its original status with no partial JSON document. Actual Auth's human helper
output stays byte-identical; JSON rows and totals match all five package counts.
`make cloc` also passes. Fleet reports/checkers remain Shared-owned; no new Auth
gate, sibling scan, tool pin or library contract is added.

The restarted complete **Bash 3.2.57 gate passes all 28 targets**, with **zero
recipe override warnings**, including four actual canister cases, 52 browser
cases/both Candid directions, isolated native/Wasm MSRV, strict Clippy, both
package dry runs without uploads and local/mocked release/evidence fixtures.
Eight declared hashes remain equal across **56 observations**; every tracked/new
byte/mode, the preserved external lock, original manifest, real index and HEAD
remains unchanged through the restarted gate. Tracked diff SHA-256
`0d511784e1e06296f7c64682a8c36e1441c420f9d3684555685209e4daef2eab`.
Older and failed receipts do not qualify this source. Evidence under
`target/shared-0.8.4/` includes `qualification-receipt.json`, exact-source
clone/preview/reuse proofs, LOC fixtures/projections, original and external locks,
registry/source comparison, retained drift failure, explicit cache preparation,
restarted gate and unchanged input/source captures. Final prose is checked
separately; the GitHub description remains accurate.

The pending candidate remains compatible **0.5.4**: no Auth public API,
signed/stored format or compiler floor change. Versions remain 0.5.3; this
uncommitted snapshot/lock has no hosted native CI of its own. Release versions,
index, commits, refs, publication and deployment remain maintainer-owned.

The latest [Canic review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6123102349)
confirms its held root selects Auth 0.5.3 and Root/issuer signing has begun moving
to a shared SignatureStore host. That source advanced during inspection; Canic
owns final runtime/HTTP/lifecycle acceptance. Complete-token and durable-session
adapters and browser non-issuing reconciliation remain in their existing issues.
No sibling file/build is changed here. Auth's Intel CI setup failure remains
[Auth #38](https://github.com/dragginzgame/ic-auth/issues/38); local passing gates
and old-release CI cannot supply the new batch's required native acceptance.

### Earlier 0.8.3 qualification

The following receipt binds the prior snapshot and stays historical after this
refresh. Versions and release effects remain maintainer-owned.

[Auth #37](https://github.com/dragginzgame/ic-auth/issues/37) extends this same
compatible batch to committed Shared Tooling 0.8.3
`8cca7e67b03ce6e701b6e7dcec2e9753eb98337e`, matching official remote main and
its committed `VERSION`. No release tag is observed or inferred. The canonical
exporter runs from an isolated clean checkout; both previews preserve every
consumer byte/mode and the real index/HEAD. Existing roots are reused, adding
only the Cargo-tool freshness task document to complete its catalog link.
The resulting 65 entrypoints select 90 files: 11 updated payloads, one addition
and no retirements. No-option preview/refresh reuses recorded roots with identical
payload/manifest bytes. Incoming Cargo files and all prior 0.5.4 work are preserved.

The adoption includes strict snapshot field validation, retained original/observed
Cargo build identities with redirected-output refusal, selected Rust formatting
catalog consistency, quiet successful PCRE2 checks and ownership-checked automatic
retirement/atomic payload replacement. The online freshness checker remains in
Shared Tooling, with no new Auth command, CI gate or schedule. Auth library APIs,
signed/stored formats, tool pins and compiler floor are unchanged. Focused canonical export/retirement, Cargo-tool build-identity/refusal and
selected-catalog Make fixtures pass on genuine **Bash 3.2.57**, separately from
Auth execution. They cover automatic closure/retirement, strict records, atomic
copy refusal/retry, distinct host/Rust catalogs and redirected observed-evidence
leaves preserving outside files, prior activation and failure archives. Their
Cargo/release effects are substitutes, not native macOS or real installer builds.

The complete refreshed **Bash 3.2.57 Auth gate passes all 28 targets**, including
all four actual canister cases, 52 browser cases/both Candid directions, isolated
native/Wasm MSRV, strict Clippy, both dry-run package builds without uploads and
local/mocked release/evidence fixtures. Zero recipe override warnings; successful
actual host PCRE2 admission is quiet. All eight declared input hashes remain equal
across **56 observations**; every tracked/new byte/mode, incoming manifest/lock,
real index and HEAD remains unchanged through the gate. Tracked diff SHA-256
`22cd51c64a5d61fdcc3618465a77b1ce4eebc029c4510817cd1979c9a9f6d76e`.
The earlier Host-only receipt below remains historical and does not qualify the
new snapshot. Evidence under `target/shared-0.8.3/` includes
`qualification-receipt.json`, exact-source clone/preview/reuse proofs, focused
fixture logs, full gate and unchanged source/input captures. Final prose is
checked separately; the GitHub description remains accurate.

This remains compatible **0.5.4**. Versions, index, commits, refs, publication and
deployment remain maintainer-owned; the uncommitted batch has no hosted native
CI of its own. Released 0.5.3's Linux pass/running Intel/queued Apple Silicon jobs
do not qualify the new snapshot or lock. Delivery and matching-source native
acceptance remain in Auth #37/#36.

### Prior Host/Testkit qualification

[Auth #36](https://github.com/dragginzgame/ic-auth/issues/36) qualifies the incoming
native Host 0.14.7 / Testkit 0.34.4 lock with server 16.1.0. The four published
Host packages have unchanged Rust sources relative to 0.14.5. Testkit changes
only two setup CLI source files to cancel owned downloader descendants on
interruption, retain failure evidence and return the signal status; its runtime
library remains unchanged. All five official registry checksums match the
incoming unyanked selections. No dependency resolution or library API change occurs.

Initial offline Testkit admission refuses its unprepared selected CLI. Explicit
`make install-testkit-tools` and separate offline admission succeed. The complete genuine **Bash 3.2.57 gate passes all 28 targets**, with **zero recipe
override warnings**. It includes isolated native/Wasm MSRV and graph checks,
52 browser cases/both Candid directions, library/proof/session checks, all four
actual certification/upgrade/signed-ingress cases, strict Clippy, both package
dry runs without uploads and local/mocked release/evidence fixtures. Eight declared
input hashes stay stable across **56 stage observations**. Every tracked/new byte
and mode, incoming manifest/lock, real index and HEAD remains unchanged during the
gate; tracked diff SHA-256
`ee610abb57df259b17046bef302a829863a23f6adbc534cb854ae5a6fdd56d27`.
Evidence under `target/integration-0.5.4/` includes `qualification-receipt.json`,
checksum-matched official package archives/source comparison, initial refusal,
explicit setup/offline admission, gate logs and source/input captures. Final prose
is checked separately. The GitHub description remains accurate.

This compatible **0.5.4** batch changes no public Auth API, signed/stored format
or compiler floor. Versions remain 0.5.3; this uncommitted pairing has no hosted
native CI of its own. Published 0.5.3's Linux pass and pending macOS jobs do not
qualify the new lock. Delivery and exact-source native acceptance remain in Auth #36.

Published 0.5.3's [producer delivery](https://github.com/dragginzgame/ic-auth/issues/35#issuecomment-6122596945)
and [Canic handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6122606090)
separate delivered mixed-profile evidence from consumer implementation. Read-only
Canic Root/issuer source remains unchanged from the reported hashes below.
Its next cuts already have published APIs: host-owned signature composition under
[Canic #60](https://github.com/dragginzgame/canic/issues/60), protected full-token
verification and atomic durable sessions under
[Canic #491](https://github.com/dragginzgame/canic/issues/491). No new Auth API
prerequisite emerged. Browser adoption remains dependent on non-issuing
reconciliation under [Canic #507](https://github.com/dragginzgame/canic/issues/507)
and [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3). No sibling source
is edited or built. Release versions, index, commits, refs and publication remain
maintainer-owned.

## Released 0.5.3 preparation evidence

The following records preparation before maintainer delivery. Its pending versions
and native observations are historical; current delivery is above.

[Auth #35](https://github.com/dragginzgame/ic-auth/issues/35) adds concrete
producer qualification for Canic's remaining signature-tree/certification
ownership gap under [Canic #60](https://github.com/dragginzgame/canic/issues/60).
Only the unpublished qualification host/runner changes: controller-authorized
preparation now takes explicit seed/domain/message inputs, and its caller and
staged-formatting fixture include the changed/new sources. Both public libraries,
their features, signed/stored formats and compiler floor remain unchanged.

One actual Testkit case prepares the existing Root role-attestation and issuer-token
profiles in the same SignatureStore and composed assets/sig/status root. Identical
payloads yield distinct expected DER keys; each IC certificate verifies under its
approved profile, while crossed seed/domain use and missing crossed retrieval
reject. The fixture's 16-leaf capacity is shared, failed new admission preserves
both proofs, and a live duplicate at capacity retains the original deadline.
Root expiry and bounded pruning preserve issuer retrieval and free its next slot.
Asset-branch changes coexist; actual upgrade preserves host metadata while clearing
both pending profiles, whose re-preparation returns the same public keys. These
are signing-profile/composition checks, not certified HTTP serving, application
authority, durable signature/session storage or wallet login qualification.

Focused `make test-qualification` passes **all four actual cases**, including the
existing complete-token/session, certification/upgrade and signed-ingress journeys.
The incoming **Testkit 0.34.3 / Host 0.14.5 / server 16.1.0** pairing is preserved
byte-for-byte. Its checksum matches the official unyanked registry package.
Initial offline admission refuses the unprepared CLI; explicit
`make install-testkit-tools` and separate offline admission succeed. Testkit's
published patch aligns locked Host and contributor snapshot/evidence mechanics;
runtime/server selection remains unchanged. No dependency resolution occurs.

The complete genuine **Bash 3.2.57 gate passes all 28 targets**, with **zero recipe
override warnings**. It includes isolated native/Wasm package MSRV and graphs,
real staged formatting, 52 browser cases/both Candid directions, library/proof/
session checks, all four actual runtime cases, strict Clippy, both package dry
runs without upload and local/mocked release/evidence fixtures. Eight prepared
hashes remain stable across **56 observations**; every tracked/new byte/mode,
incoming manifest/lock, real index and HEAD is unchanged throughout. Tracked diff
SHA-256 `e0ca887af48788a5d3604943e508d38a961c74d4d208633652bd0c55da35d959`;
new profile fixture SHA-256
`7e6e497622c804cc142e89b7f1ac94ff37adc18debb1e6dee2e39c9677196105`.
Evidence is retained under `target/integration-0.5.3/`, including
`qualification-receipt.json`, setup/admission and focused/full logs, unchanged
input captures and read-only consumer source review. Final prose is checked
separately; the GitHub description remains accurate.

Read-only Canic source at base `25b73cd403b20b4e148c6e340f8c760a26d97f6e`
still has the two SignatureMap globals and direct certification-root writes.
Captured module hashes bind that dirty-source finding, not Canic execution.
The [producer handoff](https://github.com/dragginzgame/ic-auth/issues/35#issuecomment-6122176570),
[Canic #60 adoption guidance](https://github.com/dragginzgame/canic/issues/60#issuecomment-6122176888)
and [extraction feedback](https://github.com/dragginzgame/canic/issues/491#issuecomment-6122177208)
describe one host-owned `/sig` tree with applicable profiles, a single composed
root publisher and retained Canic caller/operation metadata, quota/cleanup and
lifecycle decisions. Forking competing `/sig` owners is not the qualified design.
The library API is already published in 0.5.2; this adds evidence without a new
integration prerequisite. Consumer entrypoint/HTTP/lifecycle acceptance stays
open; no sibling code is edited or built here.

The [0.5.2 delivery update](https://github.com/dragginzgame/ic-auth/issues/34#issuecomment-6122177547)
keeps current native acceptance distinct from the pending batch, which has no
hosted run of its own. Versions remain 0.5.2. Index, commits, refs, publication
and deployment remain maintainer-owned. This qualification batch is compatible
**0.5.3**; full verifier/session authority, browser reconciliation and consumer
exposed-family delivery remain at their existing trackers.

## Released 0.5.2 preparation evidence

The following records preparation before maintainer delivery. Pending versions,
dependency selections and native CI observations are historical; current delivery
is above.

[Auth #34](https://github.com/dragginzgame/ic-auth/issues/34) adopts committed
Shared entrypoint work **`8e8cc21265272e0234326c53a8c7bf9e7d8d77df`** and qualifies
the incoming native **Host 0.14.5 / Testkit 0.34.2 / server 16.1.0** pairing.
Shared's pending heading is **0.7.0**, while its exact committed `VERSION` and
snapshot annotation remain **0.6.6**. No 0.7.0 publication is inferred.

Canonical refresh uses an isolated clean checkout and records **64 explicit
entrypoints with the same 89 payload paths**. Four selected files update;
none are added or retired. Preview preserves checkout bytes/modes, and subsequent
no-option preview/refresh reuses recorded roots with a byte-identical manifest.
The initial exact-origin refusal is corrected only in the isolated clone to
match existing provenance. Local commands, optional fixture inputs, governance,
tool pins, formatting and release policy remain selected. Future selection uses
`--entrypoint`/`--add-entrypoint`; individual retirement requires local caller and
other-bundle review plus exact ownership proof. Canonical upstream distribution
and retirement fixtures pass on genuine Bash 3.2.57, separately from actual Auth
snapshot verification; their growing upstream roster is not added to Auth CI.

The incoming manifest/lock is preserved byte-for-byte. Only the four Host
packages differ from released HEAD, advancing 0.14.4 to published 0.14.5 with
official unyanked checksum matches and identical Rust source. Committed Testkit
0.34.2's prepared CLI initially passes offline admission. The initial Host-only
28-target gate passes; evidence is under `target/host-0.14.5-issues/`, with its
binary diff stable throughout and byte/mode capture taken during initial admission.

The first post-refresh gate stops before builds after system-package updates
change the native Cargo-tool fingerprint, despite unchanged source/compiler/tool
pins and eight prepared inputs. Its old binaries and receipts remain present.
Native inventory comparison shows desktop updates along with native packages;
only `build.native` differs in old/new Cargo-sort inputs. The first explicit setup
refuses an active matching-store lock. After its owner finishes, explicit
`make install-tools` and separate offline admission pass, reusing matching
generations. The old Cargo-sort binary hash remains unchanged. Original failure,
lock refusal and inventory comparison are retained. The
[Shared #120 report](https://github.com/dragginzgame/shared-tooling/issues/120#issuecomment-6121298164)
asks the canonical owner to identify changed inputs accurately and review inventory
scope with qualified native reuse; no downstream receipt bypass or fork is added.

The repeated genuine **Bash 3.2.57 gate passes all 28 targets**, with **zero recipe
override warnings**, including isolated native/Wasm MSRV, 52 browser cases/both
Candid directions, proof/token/session checks, all three actual certification/
upgrade/signed-ingress cases, strict Clippy, both package dry runs without upload
and local/mocked release/evidence fixtures. Eight prepared hashes are identical
across **56 stage observations**. Every tracked/new file byte and mode, incoming
manifest/lock, real index and HEAD remains unchanged throughout; tracked diff
SHA-256 `1b6f5de7367ce3cecdb39eacd5408ba51897d2b81ed666a2712c7df6004a1a74`.
Evidence is under `target/shared-entrypoints-8e8/`, including
`qualification-receipt.json`, roots/previews, reuse proofs, canonical fixture logs,
native identity comparison, setup/admission and all gate attempts. Final prose
is checked separately. The GitHub description remains accurate.

Canic guidance now reflects its reported held adoption and accepted projection-first
raw-role diagnostic hard cut, with combined-invalid zero-proof-callback controls.
The earlier order choice is resolved locally; consumer delivery, complete verifier/
session authority and deployment remain distinct. Browser issuer/reconciliation
adoption remains [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3) /
[Canic #507](https://github.com/dragginzgame/canic/issues/507). No new Auth API
prerequisite emerged from this review. Older exact-release runs for #24–#28 are
cancelled; their former queued status supplies no missing native proof.

The [Auth qualification handoff](https://github.com/dragginzgame/ic-auth/issues/34#issuecomment-6121476319),
[Shared entrypoint evidence](https://github.com/dragginzgame/shared-tooling/issues/131#issuecomment-6121476738)
and [Canic feedback](https://github.com/dragginzgame/canic/issues/491#issuecomment-6121477054)
keep delivery and matching-source native acceptance at their owning trackers.
This compatible **0.5.2** batch has no public API, signed/stored format or floor
change and no hosted CI of its own. Versions remain 0.5.1; index, commits, refs
and release/publication effects remain maintainer-owned. Siblings remain read-only.

## Released 0.5.1 preparation evidence

The following records preparation before maintainer delivery. Pending-version,
dependency and native CI observations remain historical; current delivery is above.

[Auth #33](https://github.com/dragginzgame/ic-auth/issues/33) advances the same
compatible batch to committed Shared Tooling **0.6.6
`a15eb9d406d287697df9bc8fa96db8cda6721a71`**, through the unchanged 89-file
selection, and published **Host 0.14.4** across all four native packages.
Canonical refresh uses an isolated clean checkout and preserves the earlier
entrypoint, no-warning fixtures and prepared-input checkpoint work. Shared's
three changed selected files supply the new tool pin and setup/hook guidance.

Explicit `make install-rust-tools` and offline `make rust-tools-check` prepare
and admit cargo-sort-derives **0.14.1**. Auth has no affected macro definitions or
corruption-workaround markers; its default Cargo-sort/rustfmt policy is unchanged.
The committed upstream real frontend/Rust hook fixture passes on genuine Bash
**3.2.57**, with exact Node 24.21.0, npm 11.19.0 and Prettier 3.9.6 explicitly
prepared under Auth's ignored target directory. Both unprotected derive forms
retain sorted literal `$crate` paths in refreshed staged bytes; the exported
macro library and separate consuming crate compile and run. Ordinary sorting,
repeat stability, unrelated-file/index preservation and refusal controls pass.
Auth's actual formatter/staged hook and six native file cases pass separately.
The selected tool is built with the development compiler; this is not separate
Rust 1.88 tool-build qualification. The library floor remains independently
qualified at 1.88.

The incoming lock selects Host 0.14.3; only those four packages advance to
0.14.4, with official unyanked checksum matches and unchanged Host Rust source.
Cargo's narrow updates also reselect six unrelated broad-range Windows edges;
their original permitted selections are restored and locked offline metadata
admits them. All unrelated versions, checksums and dependency edges remain
unchanged. Candid/derive 0.10.38, unicode-ident 1.0.27 and Testkit **0.34.1** with
server **16.1.0** stay selected. Registry inspection now observes Testkit 0.34.2;
that separate CLI pairing is not selected by this Host upgrade. Native Host and
Testkit remain outside both public library graphs.

The complete genuine **Bash 3.2.57 gate passes all 28 targets**, with **zero
recipe override warnings**. This includes isolated native/Wasm package MSRV,
52 browser cases/both Candid directions, library/proof/session checks, all three
real certification/upgrade/signed-ingress cases, strict Clippy, both package dry
runs without upload and local/mocked release/evidence fixtures. The eight declared
input hashes stay stable across **56 before/after observations**. Every tracked/new
file byte and mode, qualified upgraded lock, original root manifest, real index
and HEAD remains unchanged during validation; tracked checkpoint diff SHA-256
`52fc98053d925ddca3429a7154e9633226816b71a5f77939b9d9eb9180466ee8`.
Evidence under `target/shared-0.6.6/` includes `qualification-receipt.json`, registry
and Host source comparisons, incoming/Cargo-generated/qualified locks, setup,
macro/focused logs and full gate captures. Final prose is checked separately.
The GitHub description still matches the independent implemented library scope.

The [Auth handoff](https://github.com/dragginzgame/ic-auth/issues/33#issuecomment-6120679650)
and [Shared macro-pin evidence](https://github.com/dragginzgame/shared-tooling/issues/113#issuecomment-6120684759)
retain delivery and matching-source Linux/both macOS acceptance. Released 0.5.0's
hosted matrix does not qualify this uncommitted batch. Pending **0.5.1** changes
no public Auth API, signed/stored format or supported compiler requirement.
Manifest release versions remain 0.5.0; index, commits, refs and release effects
remain maintainer-owned, and sibling repositories remain read-only.

### Prior 0.6.5 adoption checkpoint

The following receipt predates the 0.6.6/Host extension above and cannot qualify
its new tool pin, hook fixture or upgraded lock.

[Auth #33](https://github.com/dragginzgame/ic-auth/issues/33) now advances the pending
batch to committed Shared Tooling **0.6.5
`42f2091768b5f71c98a684d3dab563f408ca2dd9`**, with the same 89-file selection.
Canonical refresh uses an isolated clean checkout and preserves all existing
0.5.1 work and incoming dependency bytes. Newer dirty 0.6.6 work, including its
derive-sorter pin, is excluded.

The actual CI roster declares eight Rust/frontend manifests, lock and
compiler/tool pin files. Their hashes are checked before/after every stage;
changes, deletion or observation failure stop later dispatch with original
failure status and retained evidence. Nested gates in the same checkout retain
the checkpoint; independent fixtures establish their own. This is early drift
detection, not source isolation within a running stage or automatic preparation.
The Testkit fixture exercises actual `make ci` with changed Rust/frontend locks,
a deleted compiler selection and drift combined with an original failing stage;
all refuse before common/product admission or Cargo dispatch. Existing Testkit
selection/order controls and the actual formatter/staged hook checks pass.

Isolated canonical 0.6.5 fixtures pass on genuine Bash 3.2.57: staged formatting
can admit original prepared receipts while checking exported pins/compiler,
missing/corrupt/mismatched receipts and setup in exports refuse without changing
the index, and unrelated checkouts cannot borrow the hook context. Validation
fixtures cover nested checkpoints, observation errors, unchanged inputs and
retained original failures. These are upstream substitute fixtures, separately
from Auth's actual consumer and runtime checks. Evidence is retained under
`target/shared-0.6.5/`.
The incoming lock already selects Testkit **0.34.1**, with Host 0.14.2 and
Candid 0.10.38 preserved. Its initial full-gate admission correctly refuses the
unprepared selected CLI before any build, while all eight checkpoint inputs stay
unchanged. Explicit `make install-testkit-tools` and separate offline admission
now pass. The packaged changelog records Make/jobserver and failed-download
status fixes, with unchanged library APIs and server contract. The server stays
16.1.0; all three actual certification/upgrade/ingress cases pass with this pairing
in the repeated full gate. No dependency resolution or lock mutation is performed.

The extended complete genuine **Bash 3.2.57 gate passes all 28 targets** with zero
recipe override warnings. Retained checkpoint JSON confirms all eight declared
hashes unchanged across **56 before/after stage observations**. All tracked/new
file bytes and modes, incoming manifest/lock, real index and HEAD remain identical
throughout; tracked checkpoint diff SHA-256
`6cd35e0048a84521cd3aa287fafb77c3ef9a8b93735685a705ddf91af1fe4298`.
This includes isolated native/Wasm package MSRV, browser/Candid and library/proof/
session checks, actual runtime qualification, strict Clippy, both package dry runs
without upload and local/mocked release/evidence fixtures. Initial admission
refusal and explicit setup logs remain retained. Final prose is checked separately.
The verified [Auth adoption handoff](https://github.com/dragginzgame/ic-auth/issues/33#issuecomment-6120257401),
[prepared-input consumer proof](https://github.com/dragginzgame/shared-tooling/issues/129#issuecomment-6120257774)
and [hook/entrypoint proof](https://github.com/dragginzgame/shared-tooling/issues/130#issuecomment-6120257970)
keep delivery and matching-source native acceptance open. Released 0.5.0's hosted
matrix does not qualify this uncommitted batch.
The selected next version stays compatible **0.5.1**, with no public auth API,
signed/stored format or compiler floor change.

### Prior 0.6.4 adoption checkpoint

The following receipt predates the 0.6.5 extension above and cannot qualify its
new runner/hook inputs.

Auth #33 initially adopted committed Shared Tooling 0.6.4 `6fc8eb9a94f83fb0b9d0a26377d4232508cdbe2d` through a reviewed 89-file
snapshot. The sibling has newer dirty work; canonical preview/refresh uses an
isolated clean checkout of the selected commit. Its initial dirty-source refusal
is retained. `make/consumer.mk` replaces equivalent component includes and local
hook/snapshot recipes; `check-snapshot` retains its public name and delegates to
`shared-tooling-check`. Tool ordering, formatting, Make admission/jobserver and
consumer-owned full gate and release metadata remain at their existing owners.

Release, tool-evidence, Testkit and jobserver fixtures now substitute script
effects while using the real Make recipes, instead of appending overrides.
Captured real Make logs reject recipe override warnings; stderr is never filtered.
Focused jobserver, setup/check refusal, Testkit ordering, formatter/staged hook
and release cases pass. The initial full Bash 3.2 run catches a new substitute
relying on implicit branch `errexit`; an injected setup refusal continues within
its disposable repository and the harness correctly fails. Explicit refusal
exits and child-status propagation fix it. The repeated genuine Bash 3.2 release
fixture and ShellCheck pass; the initial failure and its stable inputs remain
retained under `target/shared-0.6.4/`.

The corrected complete genuine **Bash 3.2.57 gate passes all 28 targets** with
**zero recipe override warnings**, including isolated native/Wasm package MSRV,
52 browser cases/both Candid directions, all library/proof/session controls,
actual certification/upgrades/signed ingress, strict Clippy, package dry runs
without upload and local/mocked release/evidence fixtures. Every tracked/new
input and mode, incoming manifest/lock, real index and HEAD remain identical;
tracked checkpoint diff SHA-256
`f08db2af58c982ac95d23318ee84f94b7ac434c825ffba814a565f677831375f`.
Final prose is checked separately. The verified
[Auth #33 handoff](https://github.com/dragginzgame/ic-auth/issues/33#issuecomment-6119872251),
[Shared warning repair](https://github.com/dragginzgame/shared-tooling/issues/128#issuecomment-6119872460)
and [entrypoint adoption](https://github.com/dragginzgame/shared-tooling/issues/130#issuecomment-6119872683)
retain delivery and matching-source native acceptance. The new batch has no
hosted CI of its own; released 0.5.0's matrix does not qualify these changes.

The incoming lock is preserved byte-for-byte: Candid/derive 0.10.38, Host 0.14.2
across all four native packages and unicode-ident 1.0.27. Testkit stays 0.34.0
with admitted server 16.1.0. Official selected checksums are verified unyanked;
the current pairing passes real certification, upgrade and ingress separately
from offline CLI admission. Contributor Make adoption
and warning repair are compatible **0.5.1**; public auth APIs, signed/stored
formats and compiler floor are unchanged. Manifests remain 0.5.0. Index, refs,
commits, release/publication and deployment remain user-owned; siblings remain
read-only. Canic #491 owns adoption of the now-published pure request helper;
#30's raw-role projection/refusal-order acceptance gap remains separate.
The [verified delivery handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6119675390)
records the published 0.5.0 APIs; producer issues remain open for consumer
acceptance. No Canic adoption or deployment is inferred from producer CI.

## Released 0.5.0 preparation evidence

The following records preparation before maintainer delivery; pending-version,
publication and native CI observations are historical. Current delivery is above.

[Auth #32](https://github.com/dragginzgame/ic-auth/issues/32) extracts the existing
canonical request owner into the default-feature
[`session_request` module](../session-requests.md), with `SessionRequestError` for
pure shape/canonical refusals. The optional session engine shares that owner;
request hash framing, stored session bytes, admission policy and replay/atomicity
remain unchanged. There is no old-path alias. This public import/constructor-error
cut requires **0.5.0**, not a patch; manifests and lock remain at released 0.4.7.

Five default-feature request cases pass, including independent frozen vectors,
absent/explicit/max-u64 TTL, exact 16-scope/64-byte/1024-byte bounds, grammar and
combined-invalid refusal order, input detachment and framing distinction. All
14 session cases pass. Isolated normalized-package MSRV consumers now call this
public API on native/Wasm, and their default normal/build graphs refuse K256,
certificate verification, CDK and stable storage. Stored-record corruption checks
retain the engine's prior shape/canonical errors.

The complete genuine Bash 3.2.57 gate passes all **28 targets**, including all
three actual certification/upgrade/ingress cases, isolated package MSRV, feature
boundaries, strict Clippy, 52 browser cases, both package dry runs without upload
and local/mocked release/evidence fixtures. Every tracked/new input and mode,
the incoming manifest/lock, real index and HEAD remain identical throughout;
tracked checkpoint diff SHA-256
`27d69fab9cb1b5c51b66d4d1e92ae028d08895f973b12cf3211b077ac484f459`.
Evidence is retained under `target/session-request-0.5.0/`; final documentation is
checked separately. Shared Tooling stays 0.6.3, Testkit stays 0.34.0 and all four
Host packages stay 0.14.1 with the admitted 16.1.0 server.

The verified [Auth producer handoff](https://github.com/dragginzgame/ic-auth/issues/32#issuecomment-6111498271)
and [Canic adoption handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6111498429)
record the public cut and exact reviewed consumer inputs. Publication and matching
native/consumer acceptance remain open. Released 0.4.7's
[exact-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38155990585)
passes Linux and Apple Silicon; Intel macOS fails client Node/npm admission
before Rust checks.
[Auth #38](https://github.com/dragginzgame/ic-auth/issues/38) owns that setup failure;
this snapshot update does not repair it. That release
run does not qualify this uncommitted 0.5.0 batch.

Canic can adopt after publication to retire its establishment-request hash,
without enabling cryptographic session features. Keep its typed prechecks and
error precedence, protected generation, endpoint authentication, operation and
replay/lease policy. #491 retains consumer acceptance and #517 its release impact.
No sibling edit or build is performed. Index, source versions, lock, commits,
refs, release/publication and deployment remain user-owned.

## Released 0.4.7 preparation evidence

The following records preparation before maintainer delivery; pending-version,
publication and CI observations are historical. Current verified delivery is above.

[Auth #31](https://github.com/dragginzgame/ic-auth/issues/31) adds explicit
`chain_key_signature::normalize_secp256k1_signature` under the existing
independent chain-key capability. K256 supplies parsing and low-S conversion;
the library returns a fixed 64-byte array and reuses the same compact parser
with strict verification. No new feature, error variant, state, signing effect
or authority policy is added. Both isolated and complete verification still
reject high-S input and never invoke normalization implicitly.

Nine real signature cases and all 30 token/delegation cases pass, including
high-S conversion to the signed low-S control, byte-identical already-low-S
output, idempotence, unchanged input, malformed/DER/recoverable/scalar rejection,
and normalized but cryptographically invalid material. Strict high-S and Root
policy/witness refusal order stay intact. The prior normalization checkpoint's
complete genuine Bash 3.2 gate
passes all **28 targets**, including real canister certification/upgrades/signed
ingress, isolated native/Wasm package MSRV, 52 browser cases, strict Clippy, both
package dry runs without upload and existing local/mocked release/evidence cases.
Every tracked/new input and mode, both incoming manifest/lock bytes, real index
and HEAD remains identical throughout; tracked diff SHA-256
`79484f75b89f5716af1e79d47a0c48fecf0c78c2b68e59cb54928435a7b2c1a1`.
That receipt precedes the certificate issuance implementation below; final
handoff/adoption prose was checked separately.
Canic can adopt this after publication to remove its remaining normalization
primitive, retaining its explicit management-reply policy and original key/hash.
Production K256 removal still requires its final caller audit; test signing
stays test infrastructure. No Canic source or build is changed here.

The incoming manifest and lock update selects Testkit 0.34.0 and converges all
four Host packages on 0.14.1. Both incoming files are captured and preserved.
These native dependencies remain outside the published library graphs. Published
checksums are verified unyanked. The initial offline selected-CLI admission
correctly refuses an unprepared selection; explicit `install-testkit-tools` and
subsequent offline admission pass with server 16.1.0. This setup is separate from
runtime certification/upgrade/ingress qualification, whose three real cases pass
in the complete gate with the newly selected CLI/runtime pairing.
Testkit's archive migration note describes its completed-process-result hard cut;
Auth's actual callers do not use the replaced process result boundary.

Evidence stays under `target/integration-0.4.7/`, retaining the initial tool
admission refusal. Root package versions stay 0.4.6; **0.4.7** is the next
compatible patch. Index, commits, versions, refs, release/recovery, publication
and deployment remain user-owned. Auth #29 retains 0.4.6 publication/native
acceptance; Canic #491 retains consumer adoption and broader authority decisions.
The new source has no remote CI of its own. Released 0.4.6's
[exact-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38152329332)
passes Linux, with Intel macOS running and Apple Silicon queued at final bounded
inspection. It does not qualify this uncommitted patch. The final publication
recheck still finds protocol-types 0.4.6 checksum-matched and unyanked, but no
auth 0.4.6 index entry and an API 404; #29 retains this incomplete delivery.

[Auth #30](https://github.com/dragginzgame/ic-auth/issues/30) now implements
default-feature [certificate issuance validation](../certificate-issuance.md)
over the passive contract with independently supplied Root and explicit TTL/grant
bounds. It preserves the requested Root/window/certificate-TTL/token-TTL/grants/
binding-hash refusal order, exact equality and unchanged signed fields. Role
grammar remains enforced by passive construction/decoding. Success establishes
no signing authority, installation, principal eligibility or current-clock
acceptance. Verification reuses lifetime/binding primitives with its existing
identity, clock, cardinality and crypto order; token preparation reuses grant
checks without changing public causes, nonce/claims bytes or its admitted-proof
precondition. No feature, dependency, wire or stored model changes.

Nine issuance cases and all 31 token/delegation cases pass, including a new
combined-bound/clock/grant/binding regression for the existing verifier surface.
The seven preparation cases retain the frozen certificate/nonce/claim vectors.
Formatting fixtures overlay the added module and affected source/tests so their
disposable checkout checks the actual current module graph. Complete validation
of the extended batch initially passes all 28 targets, but a concurrent lock update
from syn 3.0.7 to 3.0.8 prevents a stable-input receipt. Source/modes other than the
lock, manifest, real index and HEAD remain unchanged. The incoming lock update is
preserved. The next offline attempt refuses the missing selected archive;
explicit locked `make fetch` prepares it without changing the lock. The repeated
complete genuine Bash 3.2 gate passes all **28 targets** with every tracked/new
input and mode stable, including all three real runtime cases, isolated native/
Wasm package MSRV, feature boundaries, Clippy, both package dry runs without
upload and local/mocked release/evidence fixtures. The incoming manifest, captured
updated lock, real index and HEAD stay unchanged; tracked diff SHA-256
`d2061890e43235c262a0299d21aba59e4341d8ed8d6f35570efb8878b17e7d77`.
Final prose is checked separately. All attempts and exact reviewed consumer source
hashes are retained under `target/certificate-issuance-0.4.7/`. After publication
Canic can project the typed causes and retire
both construction and token-material certificate rules under #491; no sibling
edit or build is performed. Eager raw DTO role projection may alter combined-invalid
rejection order before the passive validator runs; #30/#491 retains that concrete
consumer acceptance gap. This remains additive compatible pending **0.4.7**.
Canic's exposed upstream type-family/release boundary remains
[Canic #517](https://github.com/dragginzgame/canic/issues/517); an additive Auth
patch does not classify the larger consumer offloading release.

## Released 0.4.6 preparation evidence

[Auth #29](https://github.com/dragginzgame/ic-auth/issues/29) adds pure
`chain_key_signature::validate_secp256k1_public_key` and
`verify_secp256k1_prehash` with typed malformed-key, malformed-signature,
high-S and invalid-signature errors. The independent `chain-key-verification`
feature selects K256 without certificate machinery; default features stay empty.
Token verification enables both signature capabilities and reuses the same
parsed-material boundary after its existing policy, binding and witness checks.
Parsing remains before canonical header hashing. Signed bytes, existing Root
errors, stored formats and authority rules are unchanged.

Six isolated real-key/signature cases and 30 complete-token/delegation cases
pass, including combined policy/witness versus malformed-crypto rejection.
Formatting and the actual selected dependency-boundary checks pass. Native/Wasm,
isolated archive MSRV, Make jobserver and complete-gate coverage include the new
capability. The complete genuine Bash 3.2 `make -j4 ci` passes all **28 targets**,
including real canister certification/upgrades/signed ingress, isolated archive
native/Wasm MSRV, 52 browser-client cases, warning-denied Clippy and both package
dry runs without upload. Local/bare-remote release/publication recovery and
canonical runner/evidence fixtures also pass with their existing substituted
effects. All tracked/new inputs and modes, held lock, index and HEAD remain
identical throughout the gate; tracked diff SHA-256
`c82e788e39b83e801ca3ae8be379515128f454c030a401ddc25e6c701ed4b153`.
Final handoff/adoption prose is checked separately.
The initial negative fixture mistakenly used a K256-admitted compact SEC1 tag;
it is retained separately and corrected to a reserved tag, without changing
production parser semantics. The first complete gate identifies missing new
module inputs in the owned formatting export; its explicit overlay now includes
the changed module, manifest and test inputs and the focused fixture passes.
Both findings remain separate from subsequent complete-gate qualification.
Evidence stays under `target/integration-0.4.6/`.

The incoming Host 0.14.1 lock edit is captured and preserved byte-for-byte;
its published FS/artifact checksums and the held Testkit 0.33.8 checksum are
verified unyanked. Real qualification uses that locked Testkit and its admitted
server 16.1.0. Testkit's transitive Host 0.13.5 selection remains distinct from
the applications' direct Host 0.14.1 graph. The held external dependency
selection is separate from this additive library API.
Manifests remain 0.4.5; the next compatible version is **0.4.6**. No version,
index, commit, ref, release/recovery, publication, deployment or sibling source
mutation is performed. [The new contract](../chain-key-signatures.md) names the
future consumer cut; Canic #491 owns adoption after publication. Its feature
selection, original signing hash/key custody and endpoint error projection stay
local. Its schema key admission can also use the new helper; its signing-reply
normalization still uses K256 and is not implicitly removed. No consumer
adoption is inferred from the current dirty Canic lock.

[Auth #28's delivery update](https://github.com/dragginzgame/ic-auth/issues/28#issuecomment-6110398842)
verifies published 0.4.5 and its Shared 0.6.3 snapshot. Matching-source
[0.4.5 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38149454914) passes
Linux, with Apple Silicon running and Intel macOS queued at the final bounded
review. That released-source run cannot qualify this uncommitted 0.4.6 batch.
Earlier 0.4.1–0.4.4 runs were cancelled and
do not close their remaining native acceptance. The fixture warning request is
[Shared #128](https://github.com/dragginzgame/shared-tooling/issues/128).

## Released 0.4.5 preparation evidence

[Auth #28](https://github.com/dragginzgame/ic-auth/issues/28) qualifies committed
Shared Tooling **0.6.3 `4fd979d13c32adb3cb2f51f213ebfc4f98339b3d`** under the
canonical recovery owner [Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126).
The clean canonical 87-file refresh adopts automatic direct-preparation restart
from a clean descendant with committed local metadata disposition, preserving
the saved intent and source/candidate validation receipts. It selects the requested
increment from current metadata and repeats preflight and complete validation.
Exact resume remains source-bound; dirty/divergent source, staging evidence and
changed destinations refuse. No local runner or plan-retirement helper is added.

Auth's preparation adapter owns only local manifest/lock, notes and validation
receipt, with the manifest written last. Git effects stay in the shared runner.
The actual local/bare-remote consumer regression fails on the released 0.6.1
snapshot: the superseded patch blocks the requested minor before fresh validation
or plan archival. Its original intent/receipt remain retained. The same cases now
exercise canonical 0.6.3 with restored and fully prepared committed metadata,
exact-resume/untracked refusal, failed fresh validation and successful current-source
delivery. Setup/full-gate/upload substitutes remain explicit.
The actual consumer cases pass on current Bash and in the completed Bash 3.2 gate.
The canonical command-stub runner also passes on genuine Bash 3.2.57. They prove
old-plan archival before failed fresh validation, current-source receipt/parent
selection after successful validation, exact-resume/untracked refusal without
dispatch, preserved old receipts/external lock graph and exact local delivery.
Separate complete offline tool admission, snapshot/link checks and ShellCheck pass.

The complete Bash 3.2 `make -j4 ci` passes all **27 targets**, including the new
actual-consumer cases, real certification/upgrades/signed ingress, isolated
native/Wasm MSRV, library/client checks, Clippy, package dry runs without upload
and local/mocked release/publication recovery. Every tracked/new input hash and
mode, tracked diff, held lock and staged entry stays identical through the gate,
SHA-256 `18ff1aedd2ddccbc0da5641a1b89e7dea5c76e6b1d2bf62c20fbaf9430b311e5`.
HEAD stays at the released source. Final handoff prose is checked separately.
Evidence stays under `target/issues-0.4.5/`, retaining the old-runner regression
and initial lint finding separately.

Published 0.4.4's [exact-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38145427136)
passes Linux; both macOS jobs are queued at the final bounded inspection.
Shared 0.6.3's [exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38145666246)
passes Linux native/portable and lint/security, with Intel macOS running and
Apple Silicon queued. Neither run qualifies this uncommitted Auth adoption.
Delivery and matching-source three-host acceptance remain under #28/Shared #126;
older released acceptance remains in its existing issues. No current failed job
or additional actionable Auth product issue is found in the final inspection.

The published 0.4.4 graph remains Host 0.13.5/Testkit 0.33.7/server 16.1.0.
The Cargo build policy and installer/store bytes are unchanged from qualified
0.6.1; no new dependency selection, runtime API or tool reinstall is introduced.
Canic's published 0.4.2 pure preparation adoption remains available without
waiting for this contributor-tooling patch. Real browser non-issuing recovery
remains [Canic #507](https://github.com/dragginzgame/canic/issues/507).

The preceding evidence qualified the compatible 0.4.5 preparation over 0.4.4.
The maintainer subsequently delivered 0.4.5 as verified above; the original
source and native acceptance limitations remain historical evidence. Staging,
commits, versions, refs, release/recovery, publication and deployment remain
user-owned.

## Released 0.4.4 preparation evidence

The common Cargo setup correction is now committed and pushed in Shared Tooling
**0.6.1 `731909525038c58d6f80b498e80f8630b11c685d`**. The reviewed canonical
87-file refresh changes the installer, Cargo build companion and shared store;
no dirty upstream release-runner work is included. Under
[Auth #26](https://github.com/dragginzgame/ic-auth/issues/26) and
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120),
lock, receipt, reference and activation failures propagate out of common setup.
Cleanup releases only acquired locks, keeps the original failure status and
retains previous complete selections and attempt evidence.

The incoming dirty lock selects Testkit 0.33.7; its bytes are captured and preserved.
The published unyanked checksum is verified. This upstream patch changes developer
tooling/evidence rather than runtime APIs. A separate Host 0.13.5 lock update
arrives before the complete gate; its published checksums are verified and its
bytes preserved. The first gate correctly refuses its missing offline cache;
explicit locked cache preparation then succeeds. This Host patch also changes
contributor tooling, preserving runtime APIs. Server 16.1.0 is still selected and
admitted by Testkit. Explicit complete setup and separate offline admission pass.
Two explicit offline setup/check retries from a disposable consumer select the
identical four physical common-tool/Testkit generations without copied binaries;
their payload/receipt/source/build-log bytes, mtimes and generation inventory stay
unchanged. All 19 previously captured payload/evidence hashes still verify.

The canonical Cargo refusal fixture passes on current Bash and genuine Bash
3.2.57, including lock contention, malformed receipt and reference/activation
failure propagation with the old common set and foreign owner preserved.
Canonical host/IC store fixtures pass on Bash 3.2; the owned nine-file failure
collector and Testkit caller fixtures also pass. These substituted installer
effects remain distinct from the actual native qualification below.

The complete Bash 3.2 `make -j4 ci` passes all **27 targets**, including actual
certification/upgrades/signed ingress, isolated native/Wasm MSRV, library/client
checks, Clippy, package dry runs without upload and local/mocked release/recovery.
Every tracked/new input hash and mode and the tracked diff stay identical during
the completed gate, SHA-256
`82e37c40f509adacc0047248a9b1e075b773985014b2b692d19f8fd33e3a5a23`.
The held lock stays unchanged; staged entries still match initial HEAD.
ShellCheck 0.11.0 passes the three changed canonical scripts. Final handoff prose
is checked separately. Evidence stays under `target/issues-0.4.4/`, retaining the
cache refusal separately.

Published 0.4.3's [matching CI](https://github.com/dragginzgame/ic-auth/actions/runs/38142059219)
passes Linux, with Intel macOS running and Apple Silicon queued at bounded
inspection. It supplies hosted failure-artifact round-trip evidence for its
0.6.0 snapshot, not this correction. Shared 0.6.1's
[exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38141614488)
passes Linux native/portable and lint jobs; both macOS jobs are cancelled.
Delivery and matching-source native/hosted acceptance remain in Auth #26 and
the upstream trackers; neither cancelled nor older runs close that requirement.

The preceding qualification was completed before the maintainer's 0.4.4 release.
Its preparation identities and original native-CI limitations are retained above.
Both published unyanked package checksums now match the retained publication
intent for remote source/tag. Matching-source CI remains incomplete at inspection;
[#26](https://github.com/dragginzgame/ic-auth/issues/26) keeps that acceptance.

## Released 0.4.3 preparation evidence

[Auth #27](https://github.com/dragginzgame/ic-auth/issues/27) identifies a shared
browser-flight context mismatch. The retained key flattens extension bytes,
colliding absent and present-empty options. A joining caller previously accepted
the other caller's detached token without checking its own captured request.
Both deterministic directions fail on released source with missing expected
rejection; the initial log is retained under `target/issues-0.4.3/`.

The client now gives all completed flights one return boundary: session generation
first, detached material, then each caller's captured claims/scope and current
claim validity. It rejects conflicting options without changing keys/profile,
clearing uncertain intent, dispatching another prepare or allocating an ID.
Matching present-empty callers still share issuance. Existing stored-key
ambiguity remains explicit: redesign requires owner-controlled retained-state
disposition under [#3](https://github.com/dragginzgame/ic-auth/issues/3), not a
hidden reset or new namespace that abandons unknown effects.

All 52 focused client cases and both Candid directions pass. The six added cases
cover both conflict directions, unchanged uncertain intent/revision across reloads,
matching empty-option sharing/detachment and return-time claim expiry boundaries.
Existing stale-generation and retrieval-deadline cases remain passing.
The complete `PATH=<genuine Bash 3.2.57 profile> make -j4 ci` passes all 27 targets,
including actual certification/upgrade/ingress, isolated native/Wasm MSRV,
library/client checks, Clippy, package dry runs without upload and local/mocked
release/recovery fixtures. Every tracked/new input and the tracked diff remain
identical across the gate, SHA-256
`c585d58f72f97498ede5ddc22b027f2c5c17b2428d82b406182d3ac034d5e61d`.
The held lock and real index are unchanged. Evidence remains separate under
`target/issues-0.4.3/`; final handoff prose is checked afterward.
[Auth #27 evidence](https://github.com/dragginzgame/ic-auth/issues/27#issuecomment-6109230404)
records local implementation; delivery and matching-source native acceptance
remain open.

A separate Host 0.13.4 lock update arrives during implementation and is preserved.
The released 0.4.2 lock actually selects Host 0.13.3, superseding the earlier
0.13.2 preparation checkpoint below. Testkit stays 0.33.6 with server 16.1.0.
Explicit locked cache preparation succeeds without changing the held lock.
Host's additive private-sidecar API does not change Auth's existing native file
callers or introduce a native dependency into either public library.

Released 0.4.2's [matching CI](https://github.com/dragginzgame/ic-auth/actions/runs/38138739513)
passes Linux; both macOS jobs remain queued at bounded inspection. Issues #24,
#25 and #26 retain their source-bound native/hosted acceptance. Canic has adopted
#25's pure API; its full-verifier finite freshness/protected scopes and independent
committed-session lease remain owner decisions under #491/#459. The real browser
issuer adapter still depends on non-issuing recovery under
[Canic #507](https://github.com/dragginzgame/canic/issues/507).

The delivered Shared 0.6.0 native jobs fail at the canonical Rust fixture,
tracked in [Shared #124](https://github.com/dragginzgame/shared-tooling/issues/124).
The additional temporary-root fixture alias issue is
[Shared #127](https://github.com/dragginzgame/shared-tooling/issues/127).
Those central fixtures are outside Auth's selected snapshot. The separate common
setup lock-refusal defect is inherited by Auth and remains
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120); #125 closes
only as its duplicate. Successful setup/reuse without contention does not qualify
that failure path. A dirty upstream repair is not adopted: a reviewed committed
canonical correction and matching Auth native CI remain under
[Auth #26](https://github.com/dragginzgame/ic-auth/issues/26#issuecomment-6109193839).
No vendored installer fork or sibling edit is introduced.

The preceding qualification was completed before the maintainer's 0.4.3 release.
Its original source identities and native-CI limitations are retained above.
Both published 0.4.3 package checksums now match the retained publication intent
for the remote source/tag. Its matching-source native CI is queued at inspection;
[#27](https://github.com/dragginzgame/ic-auth/issues/27) remains open for that gate.

## Released 0.4.2 preparation and adoption evidence

The following checkpoints retain their original pre-delivery identities and
limitations. Subsequent publication and consumer adoption are recorded above.

[Auth #25](https://github.com/dragginzgame/ic-auth/issues/25) identifies Canic's
remaining pure token issuance duplication. The new default-feature
`token_preparation` API receives an installed host-admitted proof, protected
authority deadline, authenticated presenter, retained operation ID/clock and
explicit request bounds. Immutable prepared claims/hash/proof preserve Canic's
exact nonce domain, canonical signed bytes, TTL and ordered typed refusals.
Finishing attaches host issuer material without verification. Signing, issuer
approval, Root enrollment, retry custody and certification publication remain
host-owned. See the [preparation contract](../token-preparation.md) and
[consumer adoption contract](../canic-adoption.md).

The first real certified-token fixture now obtains its signing hash through this
API, attaches the actual retrieved issuer proof, then performs complete live-policy
verification, actual upgrade and re-preparation. Fixed independent nonce/claims
vectors, deadline equality and overflow, narrowing, request bounds and refusal
precedence are covered without optional signing features. This is producer
qualification, not a deployed issuer or consumer adoption.

At inspection Canic's committed base remains
`25b73cd403b20b4e148c6e340f8c760a26d97f6e`, with a changing consumer-owned tree
and Auth 0.4.0 locked. Its owner has confirmed typed Root-cause projection under
[Auth #23](https://github.com/dragginzgame/ic-auth/issues/23#issuecomment-6107838096).
The new preparation handoff remains in existing
[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6108691341); complete-token
freshness/scope and durable committed-session policy remain separate consumer
decisions. No sibling source or consumer lockfile is modified.

The incoming dirty lock selection is preserved: Host 0.13.2 and Testkit 0.33.6.
Official unyanked archive checksums/VCS identities are checked. These upstream
patches adopt Shared Tooling 0.5.0 for contributors without changing runtime APIs.
Locked cache fetch and selected Testkit CLI/server setup complete explicitly;
offline admission and complete qualification are recorded separately. The server
selection remains 16.1.0. Native dependencies remain outside both public libraries.
Evidence is retained under `target/issues-0.4.2/`.

Released 0.4.0's exact-source
[CI run](https://github.com/dragginzgame/ic-auth/actions/runs/38128802347) now passes
the complete gate on Linux, Intel macOS and Apple Silicon macOS. This supplies
the outstanding producer evidence for earlier release assertions, transaction
refusal, root-deadline qualification and typed Root causes. It does not qualify
the current uncommitted preparation API or Canic's changing consumer tree.
Before the Shared 0.6.0 adoption below, the complete local
`PATH=<genuine Bash 3.2.57 profile> make -j4 ci` passes all
27 targets for this batch, including the seven pure preparation cases, real
certification/upgrade/ingress, isolated native/Wasm package MSRV, strict Clippy,
client/library checks, package dry runs without upload and local/mocked release
fixtures. The tracked diff remains identical before/during/after the successful
gate, SHA-256 `56321332124de3e2900ad87062481c430bb6076b61c8eac0612b17f6d35b8900`.
All tracked and new-file input hashes verify; the incoming lock and real index
remain unchanged. Initial compile, formatting-export and obsolete-import refusals
remain retained. The owned formatting fixture now exports the new Rust module,
test and shared scope validation into its disposable checkout. Final evidence
prose is checked separately afterward.

[Auth #25 evidence](https://github.com/dragginzgame/ic-auth/issues/25#issuecomment-6108685240)
records implementation and qualification; it remains open for delivery/publication
and matching-source native acceptance. The earlier producer issues #19, #20,
#22 and #23 close against the completed released 0.4.0 native gate and the
separate Canic typed-cause acceptance. Auth 0.4.1's
[matching CI](https://github.com/dragginzgame/ic-auth/actions/runs/38134938851)
passes Linux at the latest bounded inspection; both macOS jobs remain queued.
#24 retains that acceptance.
The private client still requires a qualified real issuer adapter under #3.
No current local test qualifies Canic adoption or deployment.

The compatible pending version is **0.4.2**: this adds a public pure API without
changing existing APIs, signed bytes, stored formats or consumer build requirements.
Manifests remain 0.4.1; staging, commits, release versions and refs remain user-owned.

### Shared Tooling 0.6.0 adoption

The maintainer explicitly requested Shared Tooling 0.6.0. The clean committed
source `75180980a0c4e4252d8a915aaecfaefd92bf9a4e` matches upstream `main` at
inspection and is reviewed through an isolated clean Git clone. Canonical
preview/refresh adopts 87 files, adding the required Cargo build companion and
the canonical substituted failure producer. AGENTS identifies this revision.
The initial clone's local origin is refused by manifest provenance admission;
the verified canonical origin is then configured in that disposable clone.
No sibling file is edited.

[Auth #26](https://github.com/dragginzgame/ic-auth/issues/26) owns consumer adoption.
This extends compatible pending 0.4.2: contributor installation changes do not
change Auth APIs, runtime, signed bytes, stored formats or MSRV. The preparation
work under #25 and its evidence remain retained. Host 0.13.2, Testkit 0.33.6 and
server 16.1.0 remain selected; no package or dependency version is changed here.

Offline Rust admission first refuses the old local selection. Canonical setup
must explicitly prepare the shared common tools and lock-selected Testkit CLI;
their compiler/native/source/packaged-lock identity and standard environment
replace the previous local slot selection. Common PATH remains
`.tools/rust/common/bin`; the existing Testkit adapter already consumes the
installer's returned path. Older binaries, receipts and build evidence remain
retained, with pre-adoption payload hashes under `target/shared-0.6.0/`.

Owned Rust failure qualification now uses the canonical compiler-local Cargo
substitute, exact exit/log propagation and the referenced failed shared build.
The actual adopted collector round-trips nine expected files, local prior-slot
evidence and the unfollowed redirected link, excluding an unrelated generation.
Hosted collection selects the same fixture store through a step-scoped input.
The Testkit caller fixture includes canonical companions and a substituted
managed source cache so offline refusal reaches missing CLI selection admission.
An initial fixture refusal exposes group-writable seeded cache directories;
private fixture creation fixes it without weakening store admission. Both focused
fixtures pass on Linux under genuine Bash 3.2.57. No fixture calls a registry or
performs a real Cargo installation.

Actual complete setup succeeds, followed by separate offline admission of the
common tools and selected Testkit CLI/server. A second disposable consumer selects
the same four physical Cargo generations with explicit offline policy. Binary,
receipt, source/input and install-log hashes, inodes and modification times, and
each referenced slot's generation inventory remain identical. No new build
generation or download occurs, and no executable payload is copied into the
second consumer. All recorded prior local binary/receipt hashes still verify
after setup and the complete gate.

The canonical Cargo fixture passes under Bash 3.2.57. The canonical host/evidence
fixture also passes full/compact collection, corrupted or changing selections and
four rejected-store forms: admitted local bytes and diagnostics are archived,
then the completeness step refuses success. This qualifies the actual collector
locally, without claiming a hosted upload.

The complete `PATH=<genuine Bash 3.2.57 profile> make -j4 ci` passes all 27 targets
after adoption, including actual certification/upgrade/ingress, package MSRV,
library/client checks, Clippy, package dry runs without upload and local/mocked
release/recovery fixtures. Every tracked/new input hash and the tracked diff
remain identical throughout this gate, SHA-256
`c1923028d7f6d0761f3eecd103a5ac8bb77b0bbf0a220887c73e817f941f2fc1`.
Incoming Cargo.lock and the real index remain unchanged. Inputs outside the
reviewed adoption changes still match the incoming preparation batch.
Evidence stays under `target/shared-0.6.0/`, separate from the earlier 0.5-based
preparation gate. Final evidence prose is checked separately afterward.

[Auth #26 evidence](https://github.com/dragginzgame/ic-auth/issues/26#issuecomment-6108872295)
records consumer implementation and local qualification. The exact-source
[upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38136593268)
passes Linux portable/native reuse/retention and lint jobs; both macOS jobs remain
running at bounded inspection. Matching Auth native CI and hosted upload remain
separate acceptance under #26. Previous-release results do not qualify this
uncommitted adoption. Manifests remain 0.4.1 with compatible pending 0.4.2 notes;
no real staging, commit, version/ref, release, publication or deployment occurs.

## Earlier 0.4.1 adoption and release preparation

The maintainer committed the adoption as
`ba997219d245cf3b650f002a745bb2c64ec6909e`, with package versions still 0.4.0.
The subsequent reported release gate fails at IC admission. At inspection, this
checkout has no `.tools/ic` reference or common Rust projection, while the
previously admitted shared IC generation remains present. The disappearance's
cause is not established. Canonical complete setup restores the selections;
offline admission passes. The consumer release adapter previously prepared only
Host/Testkit, so it could reach the complete gate without the common sets.

The adapter now calls canonical `install-tools` and `tools-check` after locked
cache preparation and before validation. This remains inside coherent-source
preflight; interrupted metadata recovery does not provision a partial graph.
The existing local/mocked release fixture requires common preparation as well
as Testkit, checks all four setup/admission failure boundaries before gate/version
effects, and fails against the earlier adapter. The complete `make -j4 ci`
passes all 27 targets under genuine Bash 3.2.57, including the repaired fixture
and real certification/upgrade/ingress. ShellCheck and separate offline complete
admission pass. Tracked diffs stay identical before/during/after the gate,
SHA-256 `029d6c69534837e1801bbdf0cc2a813e39f890ae046dd23bee317878aa1309ee`;
selected input hashes verify and the real index remains unchanged. Evidence
prose is checked separately afterward. No version/ref/release effects occur in
the real checkout. Evidence remains under
`target/common-release-preparation/`. Shared slot contention is a separate
upstream-owned finding already recorded in
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120#issuecomment-6108305533);
no canonical lock guard is bypassed here.

Shared Tooling **0.5.0** `a306180ce534227fcb98b882fba9a526d5d747e0`
is adopted through canonical preview/refresh. The 85-file snapshot adds the
required `scripts/dev/tool-store.sh` companion and refreshes both installers,
the evidence selector and storage guidance. The published Auth contract remains
unchanged, so this extends the compatible 0.4.1 draft; manifest versions remain
0.4.0. Existing Rust slot/PATH and Testkit server ownership are unchanged.

Offline checks first refuse the retained checkout-local host/IC bundles.
Explicit canonical setup then selects verified per-user bundles through the
existing `.tools/host/bin` and `.tools/ic/bin` paths. Every regular file in both
old physical bundles still matches its recorded SHA-256. A second disposable
consumer selects the identical physical bundles with curl replaced by a refusing
recorder: no downloads occur and no executable payload is copied into that
consumer. Canonical host/IC store and evidence fixtures pass on Linux under
genuine Bash 3.2.57, including corruption, locks, independent selections,
read-only refusal and full/compact retention. IC Auth's own failure-evidence
fixture also passes with the adopted collector. Evidence is retained under
`target/shared-0.5.0/`.

[Auth #24](https://github.com/dragginzgame/ic-auth/issues/24) owns local adoption;
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120) owns shared
storage and outstanding native acceptance. Cross-checkout Cargo tool sharing is
not implemented by this release. The exact-source upstream
[CI run](https://github.com/dragginzgame/shared-tooling/actions/runs/38132790945)
is queued at initial inspection; local Linux evidence does not qualify native
macOS. At the final bounded inspection, its Linux portable and lint jobs pass,
including native two-checkout reuse and full/compact uploaded-evidence checks;
both macOS jobs remain queued.

The complete local `make -j4 ci` passes all 27 targets under Bash 3.2.57,
including actual certification, upgrades and signed ingress, isolated native/Wasm
MSRV, client/library tests, strict Clippy, package dry runs without upload and
local/mocked release/recovery fixtures. The recorded input hashes and tracked
diff remain identical before, during and after this gate; the real index remains
unchanged. Final evidence prose is checked separately afterward.

The canonical native failure producer is also qualified from an isolated clean
Git checkout of the exact Shared revision, using a separate temporary store.
Its intentional exit 1 is admitted only after its completion output and oracle
exist. The actual adopted collector round-trips all 15 expected files, unusual
filename bytes/mode and the unfollowed link. Real upstream Quill bytes exercise
checksum rejection; Cargo installation is substituted. The earlier archive-only
source invocation is retained but excluded from source-bound acceptance because
its provenance lookup inherited the parent Auth checkout. The clean Git repeat
records the correct Shared source identity. This is local Linux collection,
not a hosted artifact-upload result; upstream Linux CI supplies that distinct
evidence. Testkit stays at 0.33.5 with Host 0.13.1 and server 16.1.0.
No package-version mutation, real staging, commit, tag, push, publication,
deployment or sibling source edit is performed.

## Earlier Shared 0.4.2 adoption

Shared Tooling 0.4.2 `8db24059613c63ba9be36d4225fb74f598f49df2` is adopted
through the canonical preview/refresh flow, retaining the existing complete
84-file selection. Its revised package-impact rule classifies contributor setup
separately from the published library contract. This adoption stays in the
compatible **0.4.1** draft; it does not require Auth 0.5.0. Auth 0.4.0's earlier
public error change still correctly required its own minor boundary.

The shared installer, Make include, formatting hook/check and evidence action
are refreshed together. Consumer CI, release fixtures and shell instructions now
select `.tools/rust/common/bin`. Common setup uses the canonical versioned Cargo
slots and activates the complete three-tool projection after admission; previous
in-place installations, slots and failed attempts remain retained. The per-user
store described by 0.4.2 is policy/planned work under
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120), not an
implemented storage migration. No independent store or fallback is added.

The existing IC Auth failure-evidence fixture initially fails against the new
installer because its substitute admits only the old nine-argument Cargo route.
It now checks selected binary/registry dispatch, locates the actual retained
`cargo-attempt.*/build` output and asserts common-set refusal. The same adopted
collector round-trips the original setup/check status/logs, failed build bytes
and prior-slot evidence while preserving a redirected child as a link without
following its target. The corrected focused fixture passes; the red run remains
retained. Native setup and offline admission pass; repeated setup succeeds with
Cargo replaced by a refusing recorder, with zero Cargo dispatch and unchanged
old-tool hashes. Script/workflow lint passes too. Evidence remains under
`target/shared-0.4.2/`.

The first complete gate reports all 27 targets passing, but its final input
check observes the workspace Testkit lock selection changing from 0.33.4 to
0.33.5 during the run. Its cause is not established; an isolated exact publication
dry run preserves its lock, so no Cargo defect is claimed. The initial gate is
retained as `ci-bash32-mixed-inputs.log`, without source-stable acceptance.
The current 0.33.5 selection is preserved. Its official unyanked checksum is
`238bbd015b93bfaf27cd033425e7ac939e82488ca895d9973016c34d123a2d8d`,
VCS source `12959b1ecb110a94584e9c0dde9dfd2cbc55d187`; the published runtime
Rust source is unchanged from 0.33.4. Explicit CLI preparation and separate
offline admission pass against unchanged server 16.1.0. The final complete
`make -j4 ci` passes all 27 targets on Linux with genuine Bash 3.2.57, including
actual certification/composition, upgrade, token/session and signed ingress,
isolated native/Wasm MSRV, library/client cases, strict Clippy, live-metadata
package dry runs and local/mocked release/recovery/evidence fixtures.
Recorded tracked diffs remain identical through the final gate, SHA-256
`d9f03f9c6cdb5d3108fe65fa551cc9e7a6103ece245b882462f06a1c4538e5df`;
all hashes in `final-qualified-inputs.sha256` verify afterward. Saved diffs and
`ci-bash32.log` bind that qualification; final evidence prose is updated and
checked separately afterward. No user change is discarded or attributed to an
unproven command.

[Auth #24](https://github.com/dragginzgame/ic-auth/issues/24) owns consumer
adoption. The upstream native-retention fixture mismatch is already recorded
under [Shared #118](https://github.com/dragginzgame/shared-tooling/issues/118);
canonical upstream files are retained and upstream/native consumer acceptance
remain separate. Shared 0.4.2's
[Linux job](https://github.com/dragginzgame/shared-tooling/actions/runs/38130935497/job/114444943674)
fails its uploaded-byte qualifier before `RETENTION_QUALIFIED` is established;
the reviewed CI-only qualifier still uses the old Cargo arguments/path/diagnostic.
It is not selected in this snapshot and no collector-loss claim is made.
Released Auth 0.4.0 CI passes Ubuntu and Apple Silicon macOS, with Intel macOS
still running at this bounded inspection. Those hosted results do not qualify
the current uncommitted adoption. No package version, staged change, commit, push, release,
publication, deployment or sibling source edit is performed here. Root
manifest/lock versions remain 0.4.0; the current native graph selects Testkit
0.33.5 and exactly one Host 0.13.1 graph.
The revised adoption plan and final stable qualification are recorded in
[Auth #24](https://github.com/dragginzgame/ic-auth/issues/24#issuecomment-6108117714);
the current upstream qualifier evidence is recorded in
[Shared #118](https://github.com/dragginzgame/shared-tooling/issues/118#issuecomment-6108019389).
Both posted bodies are read back against their prepared text.

## Earlier 0.4.1 post-release documentation review

The following records the earlier inspection before the maintainer's revised
Shared Tooling plan and committed 0.4.2 adoption. Its adoption-blocker and
tooling-only minor interpretation are superseded by the current section above.

The maintainer confirms 0.4.0 is live. Official remote main and peeled tag match
the source above; annotated tag `ac49519897b6c8141414c51598f27709ac4a855c`
and both unyanked registry checksums match `.git/publication-state/0.4.0.json`:
Auth `9666628a415cccad9d92534b77c9299560422129d1d2c7a098c5a6978d019f42`,
protocol-types `8fd1124ef4628c27a04d536812dd820399e41fc2a5d4887dec6e8b8fe5f4ceb4`.
[Exact-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38128802347)
passes Ubuntu and Apple Silicon macOS; Intel macOS remains queued at inspection.

The actual release already selects published Testkit 0.33.4
`e32ef7e9dcf023cfacf53a0a7a5bbdfb4e69d226` and one copy of each Host crate at
0.13.1 `95fa4828996a6116e84b5ed01909b6e8590a355d`. Published native source is
unchanged from the previous selections; the patches correct upstream guidance
and qualification scripts. Cached archives match the official checksums.
The already prepared lock-selected CLI passes offline admission of server
16.1.0. Precise dependency probes confirm those versions are already selected;
incidental Windows resolver churn is discarded, preserving the release lock.
Earlier 0.4 preparation logs below qualify their own 0.33.3/0.13.0 inputs,
not this later release graph. Current release/registry, archive and offline
observations are retained in `target/continuation-0.4.1/`.

Shared 0.3.12 `52e846369c39a1da371eac47ff97d94da925217e` remains the reviewed
committed baseline. [Auth #24](https://github.com/dragginzgame/ic-auth/issues/24)
coordinates Shared 0.4's Rust installer/PATH hard cut. During this review its
implementation is committed as `e0acfdbb2cef9c29462686d3454c1bd993963627`,
while canonical VERSION remains 0.3.12 and the 0.4.0 notes remain a draft.
Its [exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38129617386)
is queued. Matching native acceptance is still required before refresh; no
common PATH change, upstream file copy or installer fallback is adopted here.

The compatible 0.4.1 draft corrects published examples, diagnostic migration and
native selection documentation. Root manifest/lock versions remain 0.4.0;
authentication APIs, signed bytes, storage and installation policy are unchanged.
Canic's dirty source at base `25b73cd403b20b4e148c6e340f8c760a26d97f6e`
now selects both 0.4.0 libraries and projects the typed causes through one Root
adapter, removing its post-rejection comparisons/hash. The
[consumer contract](../canic-adoption.md) records source hashes and the remaining
complete-token/signature/session boundaries. No sibling source or build is changed.
This documentation batch receives snapshot, local-link and diff checks; no Rust
or portable suite is rerun for prose. No commit, push, release, publication or
deployment is performed in this continuation.
Delivery and source migration are recorded in
[Auth #23](https://github.com/dragginzgame/ic-auth/issues/23#issuecomment-6107776477),
[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6107776635)
and [Auth #22](https://github.com/dragginzgame/ic-auth/issues/22#issuecomment-6107776935).
The committed Shared source and remaining adoption prerequisite are recorded in
[Auth #24](https://github.com/dragginzgame/ic-auth/issues/24#issuecomment-6107776783).
Posted bodies are read back against the prepared text.

## Earlier 0.4.0 preparation (delivered)

The following records preparation before maintainer delivery. Its pending-version,
untracked-file and unpublished wording is historical, and its results remain
bound to the original inputs rather than the later release graph.

The maintainer authorized continuing issue implementation after the bounded
0.3.9 review. [Auth #23](https://github.com/dragginzgame/ic-auth/issues/23) now
implements separate `RootSignatureBindingMismatch` and
`RootKeyPolicyBindingMismatch` variants for algorithm, key ID and derivation-path
hash in the one canonical Root verifier. Both complete-token and delegation-only
entry points share those causes. Within each field, inconsistent submitted
signature/header material takes precedence over an enrollment mismatch; the
surrounding guard order and established cryptographic checks remain intact.

Real signed-proof cases distinguish signature-only, header-only, matching
header/signature versus enrollment, and changed protected policy. Re-signed
header controls verify cryptographically and accept with matching enrollment;
the earlier rejection is not an accidentally invalid signature. Both public
entry points return the expected cause without mutating the token. Empty and
oversized path material, policy expiry and earlier-field precedence are covered.
All 29 focused token/delegation cases pass natively. Algorithm has only one
representable variant, so no artificial
algorithm/protocol addition is made to manufacture a negative test.

The entire undated draft is relabelled 0.4.0 from released 0.3.8, carrying forward
the previous root-deadline/replay qualification under #22. The two new public
error variants and changed causes break exhaustive matches/diagnostic projections,
requiring the pre-1.0 minor boundary. The package version stays 0.3.8 until
separately authorized release preparation; no 0.3.9 release is created. Signed
bytes, successful acceptance, session policy, stored formats and compiler floor
are unchanged. No retained-state reset is needed for this diagnostic change.
The [migration contract](../tokens.md#root-binding-diagnostics) and Canic adoption
notes specify mapping the typed origins and retiring post-rejection comparisons
and path hashing in the consumer. Canic's actual adoption and its durable/session
policy obligations stay under #491/#459; no sibling code/build is changed.

The final complete gate passes all 27 targets on Linux under genuine Bash
3.2.57, including actual Testkit certification/composition, upgrade, token/session
and signed ingress, isolated native/Wasm MSRV payloads, browser contracts, strict
Clippy, live-metadata package dry runs and local/mocked release/recovery fixtures.
The carried #22 root-deadline/replay fixture passes against the changed verifier.
`target/integration-0.4.0/ci-bash32.log` retains the full run. Before/after tracked
diffs are identical, SHA-256
`b3561e392d849ca82c2e524da3c422a4766482acb8d1242665eba5dc0f61998b`;
the new untracked `crates/ic-auth/tests/root_bindings/mod.rs` is separately bound
by `30ebcf55c1a89d2dc3719d5081ca5c81624554df4d53cc71864a26b8148800fb`.
All source, lock and snapshot hashes in `qualified-inputs.sha256` verify after
the gate. Final evidence prose is updated afterward and checked separately.

Incoming 0.3.9 diff/evidence is preserved separately; the earlier complete gates
below remain bound to their original inputs. Implementation and migration are
recorded in [Auth #23](https://github.com/dragginzgame/ic-auth/issues/23#issuecomment-6107547549),
[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6107547724)
and [Auth #22](https://github.com/dragginzgame/ic-auth/issues/22#issuecomment-6107547862).
The posted bodies are read back against the prepared text. At the bounded final
inspection, released 0.3.8 CI still passes Ubuntu with both macOS jobs queued;
those results do not qualify this pending source. Producer delivery/native
acceptance and consumer adoption remain separate. No commit, push, release,
publication or deployment is performed by this implementation.

## Earlier continuation (0.3.9 draft, superseded by pending 0.4.0)

The following results qualify the earlier compatible inputs before Auth #23's
diagnostic hard cut. They do not qualify the later changed verifier/error API.

The maintainer confirms 0.3.8 is live. Official remote main and peeled tag match
the source above; annotated tag `f42369e33b3735588c8932b5a7daa4c67678f8da` and both
unyanked official registry checksums match `.git/publication-state/0.3.8.json`:
protocol-types `9379a7ebd8518cb16c3593fdc4df72f8ab01865742364fd0708072883469bb0c`,
Auth `7236cd31af8cdbf13a1c4ce6c8619a996c46125125540971b0fab33c6732a3e2`.
[Exact-source 0.3.8 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38124650642)
passes Ubuntu at final inspection; both macOS jobs remain queued. Released
0.3.7's Ubuntu lane passed, but both
macOS lanes were cancelled; those unfinished gates cannot close Auth #19/#20.

Shared 0.3.12 `52e846369c39a1da371eac47ff97d94da925217e` and Host 0.13.0
`7c58c92c72789139ec460475c429f9090581008f` remain current at official upstream
main. The released 0.3.8 lock differs from the preceding draft qualification:
it already selects published Testkit 0.33.3
`464eb3847d8443511ed3e95fda6ac39d20695c45` and exactly one Host 0.13.0 graph.
The released changelog's 0.33.2/split-graph description and earlier evidence are
historical; published history is preserved. Actual cached archives match the
official unyanked Testkit and all four Host hashes. No dependency mutation is
needed; the precise Testkit update is a no-op. The already prepared selected CLI
separately admits unchanged server 16.1.0 offline. Testkit #54 retains its
public-reexport compatibility-boundary/guidance correction and native acceptance.
No upstream dirty corrective documentation or sibling build is adopted.

The compatible 0.3.9 fixture extension under
[Auth #22](https://github.com/dragginzgame/ic-auth/issues/22) addresses the concrete
Canic adoption deadline distinction using the existing real certified token
retrieved after upgrade. A short protected root window caps verification and
the requested session lease without changing signed claims expiry. Actual host
time verifies unchanged retry/authorization before the cutoff and rejection at
the cutoff. Failed operations preserve accounting; bounded session pruning
retains replay, and a currently eligible enrollment cannot reuse that proof.
After the existing independent-lease journey advances actual time beyond signed
proof expiry, bounded pruning reclaims the retained replay and all accounting.
This is volatile producer qualification; Canic owns its authority decision and
durable commit/restart under #491/#459. No public authentication APIs, signed
bytes, stored formats, compiler floor or consumer code change.

Focused actual Testkit certification/composition, upgrade, complete token/session
and signed ingress qualification passes on Linux with the released 0.33.3/Host
0.13 graph. The first fixture compile failure (moved test context) is retained
separately from the corrected passing run. Logs, native archive hashes and
registry observations remain under `target/continuation-0.3.9/`.
Both final complete gates pass all 27 targets on Linux, first on Bash 5 and then
on explicitly admitted genuine Bash 3.2.57. Actual canister lifecycle, native/Wasm
archive/internal-app MSRV, library/client cases, strict Clippy, package dry runs
and local/mocked release/recovery/evidence checks pass. Before/after diffs remain
identical through both gates: SHA-256
`b30db46aa812fa2a678faeb9cb8ed9e461b46589d91d1647ac2f25e82d9105a4`.
Saved diffs, verified input hashes, `ci-bash5.log` and
`ci-bash32-confirmed.log` bind those results; final evidence prose is updated
afterward. The first gate's original `ci-bash32.log` filename is misleading: its
MSRV wrapper reports Bash 5, and only the confirmed log is Bash 3.2 evidence.
The preceding release's temporary Bash profile and
`target/continuation-0.3.8/` evidence directory are no longer present at inspection;
their recorded results below are historical, not freshly inspected logs. The
new explicitly checked profile and interpreter observation are retained with
this continuation. No current evidence is deleted.
This is one compatible qualification batch in the undated 0.3.9 draft; root
package version remains 0.3.8. Native macOS acceptance is separate. No commit,
push, release, publication, deployment, sibling edit or sibling build occurs.
Qualified evidence is recorded in
[Auth #22](https://github.com/dragginzgame/ic-auth/issues/22#issuecomment-6107320351),
[Testkit #54](https://github.com/dragginzgame/ic-testkit/issues/54#issuecomment-6107320501)
and [Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6107320650).
All posted bodies are read back and match the prepared text. Auth #20 remains
open for complete native gates; source-bound current delivery is recorded there.

A subsequent bounded check finds Shared/Host/Testkit upstream unchanged and
0.3.8's Ubuntu gate passing with both macOS jobs still queued. Canic now resolves
Auth 0.3.8 and uses one protected-context Root-proof adapter for installation and
token presentation; its owner reports 103 focused Auth cases and selected Clippy
passing in the dirty 0.111.1 tree. The adoption contract records that newer
evidence separately from the earlier cache-removal slice. No consumer build or
sibling edit is performed here. New [Auth #23](https://github.com/dragginzgame/ic-auth/issues/23)
identifies ambiguous submitted-signature versus enrolled-policy error causes.
That public diagnostic hard cut requires a separate minor boundary (0.4.0) and
consumer mapping qualification; it does not expand this compatible 0.3.9 batch.
Only adoption/evidence prose changes after the two complete gates; snapshot,
local links and diff checks cover that update.

## Earlier tooling qualification (0.3.8, released)

The following observations qualify the retained earlier draft inputs. The final
release's Testkit 0.33.3 graph and current hosted CI are described above; these
earlier source, graph and CI observations remain historical.

The maintainer confirms 0.3.7 is live. Remote main and the peeled tag match the
source above; annotated tag `36c7b449c678806491bffeddb86ba4ebf0ab5f36` and both
unyanked official registry checksums match `.git/publication-state/0.3.7.json`:
protocol-types `8a9e44e6be274807c382e1eaf7db0b9134f08e3846dc55b5ed9b612a49ac754a`,
Auth `2c4bfbc4dd9e78b5b8dd118cd5177fe070132cb06377a9eb886a94b0dfba176b`.
Its [exact-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38122056624)
passes Ubuntu; both macOS jobs were queued at the initial inspection. This does not
qualify the repaired complete macOS gate.

At the final 0.3.12 inspection, Intel is running and Apple Silicon is still
queued. No new native complete-gate acceptance is inferred.

Shared Tooling 0.3.11 `db143ae9ae140eb66826c413360723d23c078f66` is verified at
official main and canonically exported from an isolated clean source as the
unchanged 84-file selection. Its committed VERSION now agrees with the snapshot,
resolving Shared #114. Its strengthened diagnostic regression separates each
family beyond neighboring summary context. That canonical fixture passes on
Bash 5 and genuine Bash 3.2 on Linux. Selected production scripts are unchanged
from the delivered Auth snapshot.

This continuation adopts subsequent Shared 0.3.12
`52e846369c39a1da371eac47ff97d94da925217e`, verified at official main and exported
canonically from a clean isolated source with the same 84-file selection. The
formatting checker now exports its selected physical temporary parent before
running nested Make/hooks/formatters. Auth's actual previous checker refuses a
safe nested Make baseline with relative TMPDIR; the retained attempt is in
`target/continuation-0.3.8/shared-0312/`. The new checker passes actual Auth
preservation on Bash 5 with a relative root and Bash 3.2 with an aliased relative
root. Canonical upstream hook fixtures pass on both interpreters, including
absolute/relative/alias success, failed prerequisite retention and source/index
preservation. Shared #116 owns native upstream acceptance.

Canic's owner has removed its proof-skipping cache in the dirty 0.111.1 slice
based on committed `25b73cd403b20b4e148c6e340f8c760a26d97f6e`; both Auth packages
resolve to published 0.3.7 with matching checksums. Local proof callbacks remain.
The adoption contract now records that progress and makes proof eligibility,
committed lease and signed-proof replay retention distinct. The library session
lease is also capped by enrolled root policy, whereas Canic's current workflow
accepts an independent expiry. Consumer authority semantics require explicit
alignment under Canic #491/#459 before replacing that engine; root windows cannot
be fabricated to bypass the cap. No consumer compilation or source edit occurs.

Incoming dirty changes select Testkit 0.33.2 and direct Host fs 0.13. They are
preserved separately in `target/continuation-0.3.8/incoming.diff` and resolved
through a narrow Cargo update and explicit locked fetch. The added Host fs and
artifacts 0.13.0 archives match official unyanked hashes and source
`7c58c92c72789139ec460475c429f9090581008f`. Testkit 0.33.2 matches its archive/hash
and source `06553d704fa111083946b3787e95f1933c00493f`. Its selected CLI is explicitly
prepared, then separately admitted offline with unchanged server 16.1.0.
Host's only Rust API removal is unused by Auth; no compatibility adapter is
needed. Testkit's incompatible Host 0.12 requirements retain the existing four
0.12.10 selections alongside direct fs/artifacts 0.13.0. No unrelated external
selection or dependency edge changes. [Testkit #54](https://github.com/dragginzgame/ic-testkit/issues/54)
owns a qualified upstream hard cut and published convergence.

Retained exact-source 0.3.6 native logs show `fmt-check`, actual formatting-hook
preservation, standard Make release adapters and untouched-archive native/Wasm
MSRV passing on both macOS architectures before the later owned release-fixture
failure. Those scoped successes supply acceptance evidence to close
[Auth #11](https://github.com/dragginzgame/ic-auth/issues/11) and
[#21](https://github.com/dragginzgame/ic-auth/issues/21); the complete gate still
failed under #19. Auth #20 requires successful complete native gates, and #3
retains real issuer/consumer adoption. The unfinished macOS jobs in the 0.3.7 run
cannot yet close those acceptance gaps. The retrieved logs are retained
under `target/continuation-0.3.8/`, independently of prior qualification evidence.

This compatible tooling/dependency batch selects the undated 0.3.8 draft; public
authentication APIs, signed bytes, storage formats, compiler floor and root
version 0.3.7 remain unchanged. At Shared 0.3.11, all 27 complete-gate targets pass on Linux with
genuine Bash 3.2, including actual Testkit certification/upgrade/ingress, untouched
archive native/Wasm MSRV, library/client cases, Clippy, package dry runs and
local/mocked release/recovery/evidence. The tracked diff stayed unchanged during
the gate: SHA-256 `e6e03ab9faf4bd2747880f56767b84052c531a5b04650d7e61addc4c80837bb2`.
Saved before/after diffs, input hashes and `ci-bash32.log` remain under
`target/continuation-0.3.8/`; final qualification prose is updated afterward.
This new graph still requires native macOS acceptance. No commit, push, release,
upload, deployment, sibling source change or
sibling build occurs. Canic's existing published APIs remain sufficient for its
next runtime-verifier/cache, signature-store and durable-session adoption under
Canic #491/#58/#459; this native update adds no consumer runtime prerequisite.

The preceding complete-gate evidence remains bound to Shared 0.3.11. A separate
final complete gate for Shared 0.3.12 passes all 27 targets on Linux with genuine
Bash 3.2, including the same actual canister lifecycle, package and release
qualification. Before/after tracked diffs match SHA-256
`11c76153b2adf8ca9608e1d9c74a1de8de8c79c4495d1f9143e8e96dfc5f18cb`;
saved diffs, verified input hashes and `ci-bash32.log` remain under
`target/continuation-0.3.8/shared-0312/`. Final evidence prose is updated afterward.
This extends the same compatible 0.3.8 draft, with unchanged package/dependency
selections and public authentication contracts. Native macOS acceptance remains
separate. The concrete Canic deadline findings are recorded in
[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6107017266).

## Earlier tooling qualification (0.3.7, released)

Released 0.3.6 is verified at remote main and peeled tag; annotated tag
`8b025e65a984642526ca816727090baebd7d3b26` and both unyanked official registry
checksums match `.git/publication-state/0.3.6.json`: protocol-types
`275bbfcc2c3ece4c6237d744bd656d319ba57ad053fb5a9ebf8c42f669bd5082`, Auth
`131bcfd312937934bf392781bc9d97116d87193092dabac0196d68016966d2f9`.
The delivered lock selects Host 0.12.9. Its
[exact-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38073930159)
passes on Ubuntu, but both native macOS lanes fail in `test-release-tools`.
Released 0.3.5's two macOS lanes also fail in that target. Neither failed complete
run supplies native full-gate acceptance. Downloaded logs are retained separately
under `target/upstream-0.3.7/`; earlier qualifications below remain bound to their
original inputs.

The requested update adopts Shared Tooling revision
`37af29b2f8fb4532744b600c5f85ce716a8b57cf`, verified at official remote main and
exported from an isolated clean committed source. Its committed VERSION remains
0.3.9 despite the 0.3.10 subject/changelog; the canonical snapshot preserves that
authority. [Shared #114](https://github.com/dragginzgame/shared-tooling/issues/114)
already tracks the producer correction. Dirty sibling 0.3.11 work is excluded.
Its 84-file snapshot includes the preceding 0.3.9 fixes for
Cargo-tool setup/check working directories and consistent live/retained diagnostic
classification. The canonical Rust-tool and validation-runner fixtures pass on
Bash 5 and genuine Bash 3.2 at preceding revision `1ed2ca5` with substituted
effects; these implementations are unchanged in the newly adopted revision.
The new release-check implementation anchors relative/symlinked temporary roots
and nested Make probes. Its canonical release-command and host-tool fixtures pass
on both interpreters with relative and symlinked temporary parents; exact-source
logs remain in `target/upstream-0.3.7/shared-0310/`. New derive-macro protection
guidance is adopted; Auth has no Rust macro definitions or dollar-prefixed derive
paths requiring markers. No shared source or sibling files are edited.

All four Host packages now select published 0.12.10
`5074de1792f920112c6539629605a6624f573863`; actual archive digests, official
unyanked registry checksums and package VCS identities match. Host's Rust APIs
and source are unchanged from 0.12.9. Narrow Cargo updates and explicit locked
fetch prepare this graph; unrelated external selections and dependency edges are
preserved. Testkit remains 0.33.1 with server 16.1.0, so its CLI/server pairing
is unchanged. Native dependencies remain outside both libraries.

The owned release fixture has six textual line-count assertions that reject BSD
wc's padded counts. The actual fixture with padded output returns failure on
Bash 3.2 after one lost-reply upload, before later reconciliation dispatch.
Numeric comparisons fix this demonstrated defect; the fixture now supplies
padded real counts on every host. Its complete local/mocked release and
publication scenarios then pass on genuine Bash 3.2. This is controlled native
output reproduction on Linux, not a claim that macOS full CI now passes.
[Auth #19](https://github.com/dragginzgame/ic-auth/issues/19) owns the remaining
release-fixture native acceptance.

This is compatible tooling/qualification work, selected for the undated 0.3.7
draft. Public authentication APIs, signed bytes, formats, compiler floor and
manifest version remain unchanged at 0.3.6. The preceding 0.3.9 snapshot, Host
0.12.10 graph and count repair pass all 27 complete-gate targets on Linux with
genuine Bash 3.2 (`target/upstream-0.3.7/ci-bash32.log`, tracked diff SHA-256
`7ccd56016f0070454644a4d9dcadd55ef38d1bca4692d58979531900453a32eb`).
The release fixture also passes on Bash 5. The final newer snapshot passes all
27 complete-gate targets on Linux with genuine Bash 3.2, including actual Testkit
certification/upgrade/ingress, archive native/Wasm MSRV, release recovery and
publication reconciliation. The tracked diff remained unchanged during that gate:
SHA-256 `31c76ee5d6d126b6be8b883694f73eab424202c508ca43158e7da8dd432fede6`;
input hashes, before/after diffs and `ci-bash32.log` are retained in
`target/upstream-0.3.7/shared-0310/`. Only this qualification prose is finalized
afterward. Native macOS complete-gate acceptance remains outstanding under Auth
#19; earlier evidence remains bound to its original inputs.
No commit, push, tag, release, upload, deployment or sibling source/build effect
is performed. Canic's complete-verifier/cache and durable-session adoption remain
with Canic #491/#58/#459; this batch adds no runtime prerequisite.

## Earlier archive qualification (0.3.6, released)

The following records preparation before maintainer delivery. Source, dependencies
and hosted-CI observations are historical; the delivered Host 0.12.9 graph and
native failures above differ from this earlier local qualification.

The maintainer confirmed 0.3.5 is live. Remote main and the peeled tag match the
source above; annotated tag `25196a5e1f7dc67a85588ae0742cdf8998ea7d11` and both
unyanked official registry checksums match `.git/publication-state/0.3.5.json`:
protocol-types `368aa465723d9958cc02baf138e7b0f5d6e2411a134f1f22b9f5643674d530f8`,
Auth `0fef8711c7e0ac0aed54f086302e35d8d89378013a55147147a76e0e1d5e5219`.
The matching [0.3.5 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38069660311)
passes on Ubuntu; both native macOS jobs are running at this inspection.
Released 0.3.4's later CI run was cancelled; its unfinished hosts remain
unqualified. Released 0.3.3's three successful hosts retain their own input-bound
evidence. [Auth #20](https://github.com/dragginzgame/ic-auth/issues/20) tracks
native acceptance of the added certified-token session scenarios.

This continuation adopts Shared Tooling 0.3.8
`67285b28a98b7c4211ad32de726709d4e87edea4` through the verified 84-file canonical
snapshot exported from a clean isolated committed checkout. The delivered Auth
lock already selects all four Host packages at 0.12.8
`c810c299acd0c41001a046c8e8ad0ee6daceefc0`; official registry and package VCS
identities match. Host's Rust APIs are unchanged from 0.12.7. Testkit remains
0.33.1 with server 16.1.0. The incoming serde_json 1.0.152 lock selection is
preserved and its caches explicitly prepared. Sibling dirty future work is not
adopted.

The compatible 0.3.6 batch repairs existing archive MSRV qualification under
[Auth #21](https://github.com/dragginzgame/ic-auth/issues/21), following
[Shared #109](https://github.com/dragginzgame/shared-tooling/issues/109). Both
normalized archives and packaged manifests/locks remain untouched. An independent
consumer uses fixture-only registry patches to select their exact unpacked
payloads. Pre-compilation admission enforces their names/versions/paths, excludes
forbidden product/runtime/native edges and preserves selected external
version/source/checksum identities. Every supported library feature still checks
on native/Wasm at Rust 1.88.0; internal applications check separately. Final
hashes require unchanged archives, unpacked files and original/fixture inputs.
Ordinary registry publication admission stays intact. Public APIs, formats,
compiler floor and manifest version remain unchanged at 0.3.5.

Initial actual MSRV qualification and graph/lock refusal fixtures pass on Linux;
the latter also pass on genuine Bash 3.2. A disposable consumer with substituted
compiler effects refuses compiler failure, unpacked-source mutation, fixture
and original lock mutation, archive mutation and a no-follow hash failure on
Bash 3.2. This does not substitute for real compilation or native host acceptance.
Evidence, including the initial failed fixture setup attempts, remains under
`target/archives-0.3.6/`. Complete **`make -j4 ci` passes all 27 targets on Linux**,
including actual Testkit certification/upgrade/ingress, every library/client case,
Wasm, strict Clippy, ordinary registry package dry runs with uploads aborted, and
local/mocked release/recovery/evidence fixtures. Inputs remained unchanged through
the gate: tracked diff SHA-256
`cda64c48e1a31832ef6c353643b0f0877b0c3f72fdf1d6785a48b5d4a97ed12d`.
The new guard/test scripts and MSRV wrapper have separately recorded hashes in
`qualified-inputs.sha256`; the matching `after-gate-tracked.diff`, `ci.log` and
focused results bind their actual inputs. Final prose receives snapshot/link/diff
checks. Native macOS acceptance of this new source remains outstanding.

Read-only Canic inspection remains at dirty base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6`, selecting Auth 0.3.4, Testkit 0.33.1
and Host 0.12.7. Full-verifier/cache retirement and durable host adoption remain
Canic #491/#58/#459; existing published runtime APIs already support those
adapters. No sibling files or builds are changed. No package version, commit,
push, release, upload or deployment occurs.
The [Canic handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6100299824)
records published 0.3.5 qualification and the remaining consumer-owned verifier
and durable-storage obligations.

## Earlier integration qualification (0.3.5, released)

The following records preparation before maintainer delivery. Source,
dependency and hosted-CI observations are historical; the delivered Host 0.12.8
selection above differs from this earlier local qualification.

The maintainer confirmed 0.3.4 is live. Remote main and the peeled tag match the
source above; annotated tag `b9b53f90092c03f9dfa0860c779e0bf5f11cad11` and both
unyanked official registry checksums match `.git/publication-state/0.3.4.json`:
protocol-types `52fc5b3860a5a485c34226aa939864dca0208ff2bb917265682767dbb178a3d9`,
Auth `feebb28b3550a78b9bbb9f0687ecea907539c5393393d3be7940d76fa9c7c390`.
The matching [0.3.4 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38066840086)
passes on Ubuntu; both macOS jobs remain queued at the latest inspection.
Released 0.3.3's complete gate now passes on all three
native hosts; that evidence is separate from 0.3.4's added formatting/session
checks and its later dependency selections. The completed owned jobserver fix is
closed under [Auth #17](https://github.com/dragginzgame/ic-auth/issues/17). Complete
certified-token native acceptance closes Auth #18; the session extension is
tracked separately under [Auth #20](https://github.com/dragginzgame/ic-auth/issues/20).

The delivered lock already selects published Testkit 0.33.1
`b2beae568f930b1f23dac69abf27571baf9f95bf` and all four Host packages at 0.12.7
`7c31035ccf9922f914ba6c63096f2437a30e1e03`; official unyanked checksums and package
VCS identities match. Shared 0.3.7's existing 84-file snapshot remains current.
Host Rust APIs are unchanged from 0.12.6. Testkit now classifies scoped HTTP
connection resets as transport failure; Auth has no custom classifier or
automatic recovery policy. Its CLI still selects server 16.1.0. The later
Host 0.12.8 preparation in the sibling is separate from the reviewed release.

The compatible 0.3.5 draft extends actual certified-token session admission under
[Auth #20](https://github.com/dragginzgame/ic-auth/issues/20). A substituted host
refuses after the real library operation, before the reference backend commits.
Fresh admission must preserve generation, session/replay absence, quotas,
accounting and empty expiry indexes; the identical token must then admit.
Exact retry past proof expiry must propagate transaction refusal while preserving
the existing session/replay pair and lease. This does not implement or qualify
durable storage, restart recovery or a Canic endpoint. Library APIs, signed bytes,
formats, dependency selections and manifest version remain unchanged at 0.3.4.
Explicit setup, separate offline admission and all three actual Testkit scenarios
pass with this locked pairing. Complete **`make -j4 ci` passes all 27 targets on
Linux**, including actual formatting preservation, isolated public-package and
internal-app native/Wasm MSRV, every library/client case, Wasm, strict Clippy,
package dry runs with uploads aborted and local/mocked release/recovery/evidence.
Inputs remained unchanged through the gate: tracked diff SHA-256
`f09853b6968f2c3cfae742d0131b861b6a9575f068061cb3dcdbc737f3c1a5f1`.
`qualified-inputs.sha256`, the matching `after-gate-tracked.diff`, focused results
and `ci.log` retain evidence under `target/integration-0.3.5/`. Final prose receives
snapshot/link/diff checks. Native macOS acceptance of the pending source remains
outstanding. No commit, version change, push, release, upload or sibling effect
is performed.

Read-only Canic inspection remains at dirty base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6`. It now locks both Auth packages at
0.3.4, Testkit 0.33.1 and Host 0.12.7. The complete-token verifier snapshots remain
unchanged: full-verifier/cache retirement, certification and durable host
adoption still belong to Canic #491/#58/#459. Published runtime APIs already
support those adapters. No sibling files or builds are changed.

## Earlier integration qualification (0.3.4, released)

The following records preparation before maintainer delivery. Source,
dependency and hosted-CI observations in this section are historical. The
maintainer's delivered Testkit/Host selections above differ from this earlier
local qualification; its hashes and passing results retain their original inputs.

The compatible 0.3.4 draft extends actual certified-token qualification through
session admission and live authorization against the volatile reference store.
Library APIs, signed bytes and storage formats remain unchanged; the package
version remains 0.3.3. [Auth #18](https://github.com/dragginzgame/ic-auth/issues/18)
retains the integration qualification.

The latest refresh adopts Shared 0.3.7
`34e5ad7aac3599306c9572bb547f2239d09df1a3` from an isolated clean committed
checkout, retaining the 84-file selection. Mandatory portable assertions and
selected Cargo-tool activation now fail explicitly on Bash 3.2. Routine CI uses
the stable workflow/ref cancellation group required by Shared #108; all three
native hosts remain required, and cancelled unfinished checks do not qualify
their source. The incoming workflow and dependency changes were preserved.

All four published Host packages select 0.12.6 at
`5f356effea97fbc31dfcca5b9f1b2834f35325a9`, matching unyanked official sparse
registry checksums and package VCS identities. The applicable Host filesystem
fix refuses NUL-containing publication paths before creating parents; Auth
continues using its existing bounded no-follow reads and durable publication
APIs. Native dependencies remain outside both libraries. Testkit stays 0.32.2
with server 16.1.0; neither the CLI/server pairing nor package versions change.
Canonical Cargo-installer and failure-retention fixtures use substituted effects;
their Bash 5 and genuine Bash 3.2 results are recorded in
`target/shared-host-0.3.4/`. This is interpreter qualification on Linux, separate
from native macOS acceptance.

The updated **`make -j4 ci` passes all 27 targets on Linux** against these
selections, including actual formatting preservation, all three real Testkit
scenarios, native/Wasm MSRV, every library/client case, strict Clippy, package
dry runs with uploads aborted, and local/mocked release/recovery/evidence cases.
The selected canonical release-runner fixture also passes on genuine Bash 3.2.
`target/shared-host-0.3.4/ci.log` and the focused logs retain results. Inputs
remained unchanged through the gate: tracked diff SHA-256
`718ecdec6925fc656274bb9984229b9bbca004276193a731f05e101a58a0fd15`,
recorded in `qualified-inputs.sha256` with the matching
`after-gate-tracked.diff`; the session-module hash remains the value below.
Final prose receives snapshot, link and diff checks. Native macOS acceptance
for this pending source remains outstanding. No package version, commit, push,
release, upload or sibling source effect occurs.

The actual certified token, re-prepared after issuer upgrade, now passes through
both public session operations against the bounded volatile reference store.
Wrong network trust and ungranted narrowing reject without proof consumption;
the identical token then admits. The actual Testkit clock advances beyond proof
expiry: fresh verification rejects, while exact retry and session authorization
preserve the original lease and unchanged storage occupancy. Narrowed protected
scopes and advanced generation reject both authorization and retry; logout
preserves replay refusal. No production stable backend or Canic endpoint is added.

Earlier session qualification, before the Shared 0.3.6 refresh, passed
**all 26 `make -j4 ci` targets on Linux**, including all three
real Testkit scenarios, isolated package native/Wasm MSRV and internal application
checks, every library/client case, Wasm, strict Clippy, actual package dry runs
with uploads aborted, and release/recovery/evidence fixtures. Evidence is in
`target/integration-0.3.4/`. The initial sandbox loopback refusal is retained
separately from the passing permitted native run. Inputs remain unchanged through
the gate: tracked diff SHA-256
`5e2fa9e0c235ba1228e4441128e0cb62501b48abc99fb032503e786c7e68a195`
and new session qualification module SHA-256
`4253302a4aa8df544e981c09c3cf336cff72eef9ca0376070717d41e3560c9e0`.
`qualified-inputs.sha256`, the matching `after-gate-tracked.diff` and `ci.log`
retain that evidence. Intentional recipe overrides remain confined to mocked
release fixtures; no closed jobserver warnings occur. Final prose receives
snapshot, local-link and diff checks. Native macOS acceptance for this pending
source remains outstanding. No package version, commit, push, release, upload
or sibling effect is performed.

Shared 0.3.6 `0604bfd730ec7ec288cd2cfdad217a0d42bf256b` is verified at remote
main and canonically exported from an isolated clean committed checkout, with
the same 84-file selection. During the initial read-only inspection its changes
were uncommitted; delivery was rechecked before refresh. The adopted hook
preserves failed Git observations, and its checker requires explicit completion
before success or cleanup. Actual Auth formatting/preservation checks pass on
Bash 5 and genuine Bash 3.2.57 on Linux. Fourteen checker failure/completion
probes preserve status and evidence on both shells. The new
`make test-formatting-hooks` exports current qualification inputs and joins
all native CI hosts under [Auth #11](https://github.com/dragginzgame/ic-auth/issues/11).
Native macOS acceptance remains outstanding.

The Testkit caller delegates lockfile parsing and selected CLI admission to the
canonical `--lockfile` installer. It re-admits offline after the separate server
operation and refuses a changed selection even when the new CLI is already
installed. Explicit actual setup and separate offline admission pass. Owned
caller cases pass on both shells, including canonical malformed/ambiguous/source
refusal, original status retention, parallel ordering and retained server paths.
Canonical Git-hook and Cargo-installer fixtures pass on Bash 5 with substituted
effects; no sibling source is modified.

A disposable contradiction of the actual owned release fixture's first required
comparison reproduces Bash 3.2 continuing with status zero, while Bash 5 refuses.
[Auth #19](https://github.com/dragginzgame/ic-auth/issues/19) owns the consumer fix:
mandatory release and changed Testkit fixture comparisons now explicitly refuse.
The same contradicted release comparison and owned Testkit status check reject
on both shells before later dispatch. The wider canonical audit remains under
[Shared #107](https://github.com/dragginzgame/shared-tooling/issues/107); normal
fixture completion is not a substitute for enforced assertions. Earlier probes
and the initial caller diagnostic-matching failure remain in
`target/shared-0.3.6/`. Normal complete release/recovery fixtures pass on genuine
Bash 3.2 on Linux after the owned assertion fix.

Earlier Shared 0.3.6 **`make -j4 ci` passed all 27 targets on Linux**, including the actual
formatting-hook check, every earlier session/client/library case, real Testkit
scenarios, native/Wasm MSRV, Clippy, package dry runs with uploads aborted and
release/recovery/evidence fixtures. The tracked qualified diff SHA-256 is
`54f6257e0cbc524f72316726b48b107343ce86757b782c0d20729c06c3c52c8f`;
`qualified-inputs.sha256`, the identical `after-gate-tracked.diff` and `ci.log`
retain unchanged-input evidence. The session module hash remains the value above.
Final prose receives snapshot, link and diff checks. Native macOS acceptance
for this pending source remains outstanding; source remains uncommitted and
package versions remain 0.3.3. No release, push, upload or sibling source effect
occurs. The producer's exact-source
[Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38061078001)
is queued at inspection; it is not consumer acceptance.

Released 0.3.3's remote main and peeled tag match the source above; its annotated
tag is `4a23c8bb13a4ffdf71392fd0acc29f6aee0f80fe`. Both unyanked official
sparse-index checksums match the publication receipt: protocol-types
`0bc439c47138d1a44bbd1ca7340077cd3958932fc0f6b78b4507d2b4effd4644`
and Auth `b5367dec889c9477c979f3cb4c29596ac73d73fb5c1124175d899092198e52b7`.
Exact-source [0.3.3 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38057156297)
passes on Ubuntu and macOS Apple Silicon; macOS Intel remains in progress at the
latest inspection. These released-source results do not qualify pending 0.3.4.
[Auth #18](https://github.com/dragginzgame/ic-auth/issues/18) retains its native
acceptance requirement. [Auth #17](https://github.com/dragginzgame/ic-auth/issues/17)
likewise awaits final descriptor-fix native acceptance; previous-release results are not substituted.

Canic's read-only dirty base remains `ac55e50334dd6479ec36f404e89e60bcfe9184d6`.
Its lock selects Auth 0.3.2, Testkit 0.32.2 and Host 0.12.4. The batch caller now
uses Auth's bounded constructor directly, with protected 64-issuer budget,
principal ordering and duplicate rejection. Its owner records focused batch,
shared installation-verifier and graph acceptance in
[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6098306441);
source remains uncommitted and deployed adoption is not claimed. Full-token
callbacks/positive cache and durable sessions remain separate adoption slices.
No sibling files or builds are changed.

## Earlier integration qualification (0.3.3, released)

The following records preparation before maintainer delivery. Dependency,
source and hosted-CI observations in this section are historical.

The maintainer requested further integration and fixes after pushing 0.3.2.
The compatible **0.3.3** draft adds complete-token Testkit qualification under
[Auth #18](https://github.com/dragginzgame/ic-auth/issues/18); library APIs,
signed bytes, storage formats and package versions remain unchanged at 0.3.2.
The existing internal signature host is used without new issuance endpoints.
The public bounded Merkle constructor and complete verifier authenticate a
native test-key ECDSA root proof plus an actual certified issuer proof. The same
token rejects after caller/audience/scope, root-policy deadline, key/proof/registry
floors, network key or issuer-certificate freshness changes. Actual upgrade
invalidates pending retrieval, and authorized re-preparation of unchanged claims
verifies with preserved issuer identity and composed asset state. All three
real Testkit scenarios pass on Linux; this does not qualify management signing,
Canic deployment, browser reconciliation or durable session storage.

Canonically exported Shared 0.3.4 `169d77b8440568c5200eede971625126181f7bb2`
is verified at remote main, with the same 84-file selection from an isolated
clean committed checkout. It rejects malformed runner depth before dispatch and
requires explicit completion before runner success, including Bash 3.2.
Canonical runner fixtures pass on Bash 5 and genuine Bash 3.2.57; actual Auth
`make ci` with malformed depth returns 2 before any validation target dispatch.
Binaryen 133 is explicitly prepared and older bundles remain retained. Auth's
qualification Wasm uses direct Cargo builds, with no optimized product bundle.

Published Testkit 0.32.2 `9e697b6363441e3fa96477adae5b07820461c3c5` and all
four Host packages at 0.12.4 `5400f159474cebac1ec7ae7c8763abfd258bde03` match
unyanked official registry checksums, package VCS identities and remote source.
Host Rust source is unchanged from 0.12.2. Explicit setup prepares the locked
0.32.2 CLI and admits server 16.1.0; separate offline tool checks pass.
Native dependencies stay outside both authentication libraries.

Read-only Canic inspection remains at dirty base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6`; its lock now selects both Auth
packages at 0.3.2 and Testkit 0.32.1. The generic Merkle helper and two verifier
snapshots match the previously recorded hashes; adoption and retirement stay in
Canic #491/#58. No sibling files or builds are changed.

Evidence is in `target/integration-0.3.3/`. Initial compile/module-path failures
and sandbox loopback-startup refusal are retained separately from the passing
permitted native run. A source-prefix nounset injection into a disposable copy
of the unchanged canonical formatting-adoption checker returns 1 on Bash 5 but
0 on Bash 3.2, deleting its fixture before completion. Shared #103 owns that
remaining helper repair; canonical files are not patched here. Auth #11 retains
its native hook-preservation qualification.

Complete **`make -j4 ci` passes all 26 targets on Linux**, including all three
actual Testkit scenarios, every library/client case, isolated native/Wasm package
MSRV and internal application checks, Wasm, Clippy, actual package dry runs with
uploads aborted, and release/recovery/evidence fixtures. Inputs remained unchanged
through the gate: tracked diff SHA-256
`8baa158ef0592f85fb10a03fc526bfd612d32ab3c5d79975a1d9b184574e3850` and new
qualification module SHA-256
`6cc755b33704814fcbe8ab291f2326ec9743d48f47d9078574f27b7190b4f891` are retained
in `qualified-inputs.sha256`; `ci.log` records the complete result. Intentional
recipe overrides occur only in substituted release fixtures, with no closed
jobserver warnings. Final prose receives snapshot, link and diff checks.
Native macOS acceptance for this pending source remains outstanding; released
0.3.2 CI is still queued. No sibling source, package version, commit, push, release
or upload effect was performed.

## Earlier release reconciliation (0.3.2)

The following records delivery reconciliation before the current integration batch.

Remote main and peeled tag matched the 0.3.2 source at preparation; annotated tag
`cc896a2511ec9d74fd3e9a2b2df9c07414afd524` matches the retained publication
receipt. Official sparse-index entries are unyanked and match both receipt
checksums: `ic-auth-protocol-types`
`02974fddd268c9cb24c0f8126efd4d85e2510190be037fceaa8a948fe1a75713` and
`ic-auth` `655b30686bca3eddcbaeb9351a1609c47b6919ca6028ed4efa91d78aa35974f2`.
Exact-source [0.3.2 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38054843142)
is queued at inspection; no hosted native acceptance is claimed for this source.

Exact-source [0.3.0 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38046019077)
and [0.3.1 CI](https://github.com/dragginzgame/ic-auth/actions/runs/38051656374)
now pass on Ubuntu 24.04, macOS 15 Intel and macOS 15 Apple Silicon. Each host
passes selected setup, exact-ID artifact download, independent uploaded-byte
verification and the complete gate. This completes native common-toolset
adoption under [Auth #16](https://github.com/dragginzgame/ic-auth/issues/16).
It does not qualify the final 0.3.2 dependency-pin descriptor repair under
[Auth #17](https://github.com/dragginzgame/ic-auth/issues/17), which remains open
for that release's native results. [Auth #11](https://github.com/dragginzgame/ic-auth/issues/11)
retains the separately required native real hook/formatting preservation checks;
the hosted gate covers release routing and check-only formatting, but does not
run those focused hook cases.

Published authentication APIs and signed bytes remain unchanged from 0.2.11.
Canic's available full-token verifier and bounded batch constructor are described
in [the adoption contract](../canic-adoption.md); consumer qualification and
duplicate retirement remain owned by Canic #491/#58. Sibling trees stay read-only.
This reconciliation changes documentation and issue state only; no new numbered
release draft, source build, commit, push or publication is introduced. Earlier
local evidence paths below record preparation; `target/continuation-0.3.1-live/`
is absent from the current checkout, so those logs were not re-inspected here.

## Earlier upstream adoption (0.3.2, released)

The following records preparation before maintainer delivery. Dependency and
hosted CI observations are historical, not current qualification.

The maintainer confirmed 0.3.1 live. Remote main and peeled tag matched
`7ee9aa2a6f6fb19e94da096ce4149ff05c256476`; annotated tag
`ec7032f13a8ef8611a7318d0a7e681ea402cac32` and both unyanked
official registry checksums match retained publication intent. Exact-release
[CI](https://github.com/dragginzgame/ic-auth/actions/runs/38051656374) is in progress
at the latest inspection; 0.3.0 CI remains in progress. Neither qualifies this pending source.

Shared Tooling 0.3.2 `c16444bf006f17c5bb4dda5ad070a0f345da9623` is verified as
remote main and exported from a clean isolated committed checkout. The 84-file
snapshot includes the new advisory README review task; no schedule or gate is
introduced. Its formatter, Rust setup/check and LOC recipes now preserve the
jobserver, and tools-only includes load the existing execution guard. Shared
fixtures require explicit completion before success/cleanup, including Bash 3.2
early-exit failures. [Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99)
and [#103](https://github.com/dragginzgame/shared-tooling/issues/103) own these
producer corrections; canonical bytes and sibling trees are preserved.

The incoming root Testkit 0.32 requirement and lock are preserved. Published
Testkit 0.32.0 `1502f667bc6b7f54067dd57d1841cd0a150110a7` selects Host 0.12;
all four Host packages now resolve once at 0.12.2, at
`e1ef99e6a4c6d05f0b0d8364f8586c6cc358dadc`. Remote main/tags, official unyanked
checksums and package VCS identities verify. Host library source and the Testkit
runtime API are unchanged; the internal reexport graph cut does not change
authentication library APIs or formats. All native tooling stays outside both
libraries. Explicit parallel setup prepares the locked 0.32 CLI and admits the
same 16.1.0 server, retaining previous selections. Check commands never install.

Read-only Canic review remains at committed base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6`: its dirty root graph still selects
Auth 0.3.0 and Testkit 0.31.0, with Host 0.11.0 and 0.12.1 duplicates. Its local
batch helper and both verifier snapshots are unchanged from the review below;
the published full verifier and bounded batch constructor are already available.
The [adoption contract](../canic-adoption.md) now identifies published 0.3.1;
Canic consumer acceptance and retirement remain in #491/#58, with no sibling
source edits or runtime acceptance inferred here.

Evidence is retained in `target/continuation-0.3.1-live/`. Shared actual-include
descriptor/mode/preflight fixtures pass on Bash 5 and genuine Bash 3.2.57, and
canonical failure-injection/retention fixtures pass on both shells. Disposable
source-prefix nounset probes of Auth's owned tool-evidence fixture return 1;
they do not reproduce the upstream EXIT-handler zero-status bug. These owned
fixtures retain their evidence without that cleanup trap, so no speculative
local guard or additional framework is introduced. Initial fixture-directory
preparation failures and both probes remain retained.

Complete **`make -j4 ci` passes all 26 targets on Linux**, including real
certification/composed-root/protected-upgrade/fresh-key ingress on the unified
Host/Testkit graph, every library/client case, isolated native/Wasm MSRV, Wasm,
Clippy, actual package dry runs and release/recovery/evidence fixtures. Gate
inputs remained unchanged: captured tracked diff SHA-256
`f0426487d1d0eb8908b7bfc751022921757a9050039bed236109e5224520f776`; the
new README task is hashed separately in `qualified-inputs.sha256`.

The gate exposed one remaining owned boundary: dependency-pin validation calls
Cargo workspace discovery through a non-recursive recipe. Added descriptor
coverage fails on the unrepaired recipe with a closed reader, then passes after
recursive marking on Bash 5 and Bash 3.2. Unsafe modes still refuse before
substituted effects. Actual parallel dependency-pin, formatter and metadata
checks pass without descriptor warnings, and ShellCheck passes. Unchanged gate
evidence is reused; these affected checks qualify the final follow-up. The
earlier six pin-check warnings remain in the original full log rather than
being relabelled as a warning-free gate. [Auth #17](https://github.com/dragginzgame/ic-auth/issues/17)
retains this owned repair; the adopted Shared formatter/installer correction
now passes locally. Final handoff changes receive link, snapshot and diff checks.
Native macOS qualification for this pending source remains outstanding.
The compatible draft is **0.3.2**; package versions
remain 0.3.1. No commit, push, release, upload or sibling effect occurs here.

## Earlier tooling adoption and parallel Cargo repair (0.3.1, released)

The following records preparation before maintainer delivery. Dependency and
hosted CI observations are historical, not current qualification.

The maintainer confirmed 0.3.0 live. Remote main and peeled tag match
`6141442bdc7699fa697c9da69ef3299ce9b66a6e`; annotated tag
`375a33ab1aff897095ef2d305b9f52a480b3700d` and both unyanked
official sparse-registry checksums match retained publication intent.
Exact-release [CI](https://github.com/dragginzgame/ic-auth/actions/runs/38046019077)
is in progress at the latest inspection. Prior 0.2.11
[CI](https://github.com/dragginzgame/ic-auth/actions/runs/38043906989) now passes
all three supported hosts, including authenticated exact-artifact readback and
independent payload checks. That evidence does not qualify later source.

Shared Tooling 0.3.1 `fa452afaa5012866eb1c20820dfa8038c106e7ec` is verified
as remote main and exported canonically from an isolated clean checkout. All 83
selected files and modes verify. The aggregate now admits complete-set platform
and Rust/Cargo prerequisites before any installation; host diagnostics identify
the exact selected tool without executing unauthenticated bytes. Actual Auth
setup callers admit preflight separately from injected installation failures,
including rejection before Testkit or product setup under parallel Make.
[Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101) owns the
producer; [Auth #16](https://github.com/dragginzgame/ic-auth/issues/16) owns common
setup adoption. Exact-source Shared
[CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38049622600)
is queued at inspection; local substituted tests do not qualify native macOS.

The incoming lock selects published Host filesystem/artifact 0.12.1 at
`e5ecfa06c14d144cfeb85ea89d65906b1bf81636`, verified against remote main/tag,
official sparse-index checksums and package VCS identities. Its library source
is unchanged from 0.12.0; its owned release-tool, jobserver and Bash 3.2 repairs
do not change Auth's release policy or require cargo-set-version here. Root
requirements and Testkit 0.31.0 selection remain intact. Testkit retains its
four Host 0.11.0 packages and 16.1.0 server; do not misreport this as a unified
0.12 graph. These dependencies stay outside both authentication libraries.
The selected Testkit CLI/server passes actual offline admission.

Actual `make -j4 metadata` reproduced a closed inherited jobserver descriptor
with GNU Make 4.3 / Cargo 1.99. Root-owned Cargo recipes and wrappers now preserve
the descriptor handoff through recursive recipe marking. The globally included
execution guard still refuses unsafe Make modes before effects, including when
MAKEFLAGS is erased. No jobserver flags, separate job budget or vendored code is
introduced. The actual-Make descriptor fixture fails on the released Makefile and
passes on the pending source, on Bash 5 and genuine Bash 3.2.57. Actual parallel
metadata is warning-free. Shared formatter/installer boundaries remain owned by
[Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99); there is no
reviewed committed correction in Shared 0.3.1 to adopt yet. Auth's repair is
[Auth #17](https://github.com/dragginzgame/ic-auth/issues/17).

Read-only Canic review at committed base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6` finds its dirty root manifest and lock
selecting 0.3/0.3.0 with both published checksums. The unchanged local Merkle
builder and remaining complete-token proof callbacks are consumer-owned deletion
boundaries, not missing Auth APIs. The dirty `ops/auth/delegated/verify.rs` is
SHA-256 `0983dc4937c342ca2b666659fcd02339bff2c8c127d7fadf0b4b0d4ef836a509`;
the distinct `ops/auth/token/verification.rs` remains
`40220fc6b7bd899dc7889340e785b4ebca2c91d9bbdc17e7f0c91ae3577a73f7`.
The [adoption contract](../canic-adoption.md)
selects published 0.3.0 and retains Canic's qualification obligations. No consumer
build or runtime acceptance is inferred from dependency selection.

Earlier repair evidence is retained in `target/continuation-0.3.0-live/`; focused
descriptor, mode-refusal, ShellCheck and selected-tool checks passed. That complete
**`make -j4 ci` passed on Linux**, including both real certification/upgrade/fresh-key ingress
scenarios on the release-selected Host/Testkit graph, every library/client case,
native/Wasm MSRV, Clippy, package dry runs and release/recovery/evidence fixtures.
The earlier gate's tracked diff SHA-256 is
`80cac2c9aa673727b04151e5c3054760b1d3defafb7c141d9c4806d7821e1106`;
its new fixture hash is retained separately. Inputs stayed unchanged during the
gate. It exposed further owned release-fixture and read-only version-probe
boundaries, whose recursive marking and descriptor/refusal coverage are added
afterward. Affected descriptor fixtures on both shells, actual version read,
ShellCheck and parallel release/publication fixtures pass again; unchanged
gate evidence is retained. Final source/patch identities and logs remain beside
the earlier gate inputs. That gate precedes the new Shared 0.3.1/Host 0.12.1
adoption. New adoption evidence is retained in `target/upstream-0.3.1/`;
consumer setup/evidence fixtures and exact-source upstream command/host fixtures
pass on Bash 5 and genuine Bash 3.2.57. Actual parallel setup reuses the prepared
selections, and separate offline admission passes. A fresh complete **`make -j4
ci` passes all 26 targets on Linux** for Shared 0.3.1, Host fs/artifact 0.12.1 and
Testkit 0.31.0, including the final owned jobserver repair and all real canister,
native/Wasm MSRV, client, package-dry-run and release/recovery/evidence checks.
Gate inputs remained unchanged: captured tracked diff SHA-256
`c4457e1479113bac91a8a61bcb65f175b8946310fb4e270bcb8f77887be7a531`, with
the new descriptor fixture hashed separately in `qualified-inputs.sha256`.
Shared-owned descriptor warnings remain under #99; deliberate recipe overrides
belong only to substituted fixture Makefiles. ShellCheck passes for changed
owned scripts; final handoff edits receive link, snapshot and diff checks.
No native
macOS result exists for the pending source. The compatible draft is **0.3.1**;
public authentication APIs, signed
bytes, retained formats and package versions remain unchanged. No sibling source,
commit, push, release or publication effect occurs here.

## Earlier Shared Tooling adoption (0.3.0, released)

The following records implementation and qualification before maintainer delivery.
Pending-source, dependency selections and CI observations below are historical.

The maintainer confirmed 0.2.11 live. Remote main and peeled tag match the source
above; annotated tag `45c644dc752c66ffb4db3e3765a95c0459e8b765` and both unyanked
registry checksums match retained publication intent. The bounded Merkle batch
constructor is now available for Canic adoption; current consumer requirements
are in [the adoption contract](../canic-adoption.md) and Canic #491.
Exact-release [CI](https://github.com/dragginzgame/ic-auth/actions/runs/38043906989)
is queued at inspection; the pending adoption has no hosted result of its own.

Shared Tooling 0.3.0 `88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d` is exported from
an isolated clean committed checkout, excluding sibling edits. All 83 selected
files retain canonical bytes and modes. The new common aggregate owns host → IC
→ Rust ordering; IC Auth selects minimum compiler → locked Testkit as its ordered
product extensions. Direct host installation always requires all four tools;
optional ripgrep/cloc flags are removed. Existing installation layouts and
receipts are retained. The early CI and standalone qualification admission
boundaries remain intact. Actual caller fixtures substitute earlier prepared
sets and installation effects, preserving the adopted aggregate and collector.
They check parallel ordering, missing-compiler/CLI refusal before dispatch and
Rust failure evidence without product effects. They pass on Bash 5 and genuine
Bash 3.2, alongside upstream common-command and four-host installer fixtures.
Shared exact-source [CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38044218125)
is queued at inspection; local fixtures do not prove native macOS acceptance.

The incoming root Testkit 0.30 requirement and matching lock edit are preserved.
Published 0.30.0 at `6ac161b8ed689012bf8b0ce926946f94ea9f407d` has matching official
sparse-registry checksum and is unyanked. It bounds retained build diagnostics
and requires complete bounded identity/metadata probes; successful cache framing
and server selection are unchanged. It remains outside both libraries, selecting
the same four Host 0.11.0 packages. Explicit `make install-tools` prepares the
selected CLI/server through the new aggregate; offline admission is checked
separately. No direct process caller or compatibility path is introduced here.

Focused evidence is retained in `target/shared-0.3-adoption/`; complete **`make ci`
passes on Linux**, including isolated native/Wasm package MSRV, every library
and browser case, both real certification/upgrade/fresh-key signed-ingress
scenarios, Wasm, Clippy, package dry runs and release/recovery/evidence fixtures.
`ci.log` retains the complete result. Gate inputs are retained in
`qualified-inputs.diff`, SHA-256
`a56241c425141a7925d2a3918a6bd1f1ec273996a65af6d360ffd888fedb3010`;
those inputs remained unchanged throughout the gate. Only this prose outcome
is updated afterward. Matching native macOS consumer qualification remains
pending. The selected draft is **0.3.0** because the adopted installer CLI removes
flags and aggregate execution semantics change.
Authentication APIs, signed bytes and retained formats are unchanged. Package
versions remain 0.2.11; this continuation does not commit, push, publish or edit
sibling repositories. Owning adoption trackers remain
[Auth #11](https://github.com/dragginzgame/ic-auth/issues/11) and
[Shared #98](https://github.com/dragginzgame/shared-tooling/issues/98).

## Earlier Canic batch-construction extraction (0.2.11, released)

The following records implementation and qualification before maintainer delivery.
Pending-source and CI observations below are historical.

The maintainer confirmed 0.2.10 live. Remote main and peeled tag match the source
above; annotated tag `8fa4da48ca6e6be03bcabb72f6d40f5f1b876805` and both unyanked
registry checksums match retained publication intent. Exact-release
[CI](https://github.com/dragginzgame/ic-auth/actions/runs/38041557212) passes Linux;
Intel is running and Apple Silicon queued at inspection. The pending source has
no hosted result of its own.

Read-only Canic inspection at base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6` finds its changing root lock selecting
both Auth packages at 0.2.10. The local complete-token callbacks and positive
cache remain. Their replacement is already published and tracked in
[Canic #58](https://github.com/dragginzgame/canic/issues/58); no new endpoint or
wallet service is required. The Merkle construction helper matches immutable
committed source, SHA-256
`f2ea18223a396afb89f3c6ab3ce0c98f58e780f392e98e08996b354043e515ab`.

The pure library now implements a bounded root-and-witness constructor in its
default graph, preserving node bytes, leaf order and odd-node promotion. The
existing verifier shares its node hashing. Four construction cases cover the
frozen Canic root, witness directions, singleton/duplicate hashes, rejection and
all shapes through 64 leaves. All 26 token/proof cases pass, including a generated
witness against existing signed bytes and a wrong-position witness rejection.
[Auth #15](https://github.com/dragginzgame/ic-auth/issues/15) owns the addition;
[the adoption contract](../canic-adoption.md) retains Canic's protected issuer
policy, signing, persistence and qualification obligations.

The incoming root Testkit 0.29 requirement is preserved and reconciled to
published 0.29.0 at `e15cc2acfd9324f6877f854415005f91d169a031`. Its Host 0.11
selection removes the older filesystem/artifact graph and unifies all four Host
packages at 0.11.0. Explicit setup prepares the 0.29.0/debug CLI and admits its
unchanged 16.1.0 server, retaining prior installations. No CLI fallback or direct
PocketIC dependency is introduced. Complete `make ci` passes on Linux, including
isolated native/Wasm package MSRV qualification, all library/client cases,
both actual certification/upgrade/fresh-key ingress scenarios, Wasm, Clippy,
package dry runs and release/recovery fixtures. Matching macOS consumer
qualification remains pending. Evidence is retained in
`target/canic-adoption-0.2.10/`; production inputs are recorded before the gate
and remain unchanged afterward.

The next compatible draft is **0.2.11**: the constructor is additive, with no
changed signed encoding, wire/storage contract or existing public API. Package
versions remain 0.2.10; no sibling files, commits, pushes or release effects are
changed here. Consumer adoption remains in Canic #491.

## Earlier Host and tool preparation follow-through (0.2.10, released)

The following records implementation and qualification before maintainer delivery.
Pending-source and CI observations below are historical.

The maintainer confirmed the preceding work pushed. Remote main and peeled
`v0.2.9` match the current release above; annotated tag
`e69ad79f060a4a64278f67d6b2af869be9ba7aff` and both unyanked registry checksums
match retained publication intent. Published packages do not establish hosted
acceptance of the pending source.

The latest continuation preserves the incoming root `ic-host-fs` 0.11 requirement
and reconciles its lock to published filesystem/artifact 0.11.0. Official registry
checksums and downloaded VCS identities match Host
`1d768c80a5bb87e3330a6b7bacfdfc543063968f`. Both the native utility and
qualification runner now use that root-selected filesystem API directly.
Directory-required suffixes refuse without replacing a stripped file or creating
parents; staging cleanup failures remain secondary but appear in diagnostics.
The consumer regression fails against isolated Host 0.10.1 and passes with
0.10.2 and 0.11.0 across create/replace modes, saved/missing targets and three
suffixes. No duplicate pathname guard or compatibility shim is introduced.
Published Testkit 0.28.1 still requires Host 0.10, so its four 0.10.2 packages
remain intact. Only the two 0.11.0 packages are added to the incoming external
lock identities. Host's process hard cut belongs to
[Testkit #47](https://github.com/dragginzgame/ic-testkit/issues/47); Auth has no
direct process caller to migrate. Root manifest, Testkit selection and client
lock are preserved.

Shared Tooling committed 0.2.13
`5864f468d39f8f9d1bd26fca1afe0e20f25f1b5e` is exported from an isolated clean
checkout. The existing 83-file selection adds exact selected-CLI failure details
and the accepted preparation/early-admission contract, retaining 0.2.12's
independent Make-flag admission and raw/resolved directory rejection for LF/CR.
The five-tool bundle and CLI selection stay unchanged. Exact-source Shared CI
passes Linux, lint/security and Apple Silicon, with Intel still running. Host CI
passes Linux and MSRV, with Apple Silicon running and Intel queued at inspection.
These upstream results do not establish hosted consumer acceptance.

Normal release preflight now prepares the exact locked Testkit CLI/server through
the existing explicit target, then checks admission, after source/version/notes
admission and locked fetch. Cargo's explicit offline setting remains authoritative.
Saved-receipt recovery returns before setup against interrupted metadata.
Complete CI checks prepared tools immediately after snapshot integrity; standalone
qualification checks Testkit before dispatching its Wasm build, including parallel
Make. Ordinary checks remain offline. Auth's adoption is local;
[Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96) still owns
common prevention/diagnostics and other consumers' owning trackers retain their
adoption. No sibling source is modified.

Both real certification/composed-root/protected-upgrade/fresh-key ingress cases
pass on this mixed Host graph. Six native utility tests, strict native all-target
Clippy and Rust 1.88 all-target/all-feature checks of both native apps pass.
Early-refusal caller tests and the canonical Shared selected-installer fixture
pass on Bash 5 and genuine Bash 3.2. An actual missing-CLI consumer check names
package, locked version, target, profile, destination and explicit setup command,
with empty success output and no tool-directory creation. Release fixtures pass
with local bare remotes/substituted upload and setup effects, including failures
before the gate, dirty-source refusal and interrupted recovery without further
preparation. Actual tool-failure collection, prepared-tool admission, dependency
boundaries, formatting, ShellCheck, snapshot integrity and document links pass.
Current evidence is in `target/continuation-host-0.11/`. Earlier red regressions,
corrected fixture setup and sandbox/server-bind failures remain separately
retained in `target/continuation-0.2.9-live/`; the fleet inspection remains in
`target/selected-cli-fleet-scan/`.

The selected draft is compatible **0.2.10**: library APIs, signed bytes, stored
formats, package versions and browser client remain unchanged. The Canic adoption
contract now selects published 0.2.9; no new Rust API is needed for its existing
complete-token, durable-session or real-issuer adoption obligations. Full local
CI and native macOS qualification remain separate; no commit, push, release,
upload, deployment or sibling mutation occurs
in this continuation.

## Earlier Shared adoption and Canic handoff (0.2.9, released)

The following records implementation and qualification before maintainer delivery.
Pending-source and CI observations below are historical.

The maintainer confirmed 0.2.8 live. Remote main and peeled tag match the source
above; annotated tag `4b7e158cc72b2430ecfa5a1886eaa00007669f06` and both official
unyanked registry checksums match retained publication intent. Its
[CI](https://github.com/dragginzgame/ic-auth/actions/runs/38034022541) has passed
Ubuntu and macOS ARM, with Intel still running at final inspection; the pending changes
below have no hosted result yet.

Shared Tooling 0.2.10 at `43a0dc46cdc3c77e70a68e192561642ed50a3e0f` is reviewed
and exported from an isolated committed clone, excluding dirty sibling work.
The snapshot selects 83 files with its exact version annotation, adding the
formatting wrapper and registry helper/fixture explicitly. No fleet tools or
tool-pin changes are added. Formatting keeps its existing sort/fmt checks and
now retains failed command output through the shared collector. The Rust CI job
adds only Actions read permission and authenticated current-repository/run
artifact lookup, retaining the pinned action, uploader-returned ID and
independent byte verification. [#13](https://github.com/dragginzgame/ic-auth/issues/13)
retains hosted consumer acceptance. Shared's exact-source CI passes Linux and
macOS ARM; its Intel IC-tool setup failed on a Quill DNS resolution timeout,
separate from registry/formatting/adopted-source behavior.

Publication now delegates transport, HTTP classification and bounded identity/
schema admission to Shared's metadata mode, requiring curl 8.4.0+ and jq. Each
observation retains a fresh private directory; Auth still owns expected archive
checksums, source/tag intent, upload order and uncertain-dispatch refusal.
Matching yanked bytes reconcile earlier effects without another upload. The
expanded consumer fixture reproduces the old observer accepting wrong-crate
metadata with matching digests, then passes with the adopted observer. It covers
invalid identity/schema/stream/size, transport and old-curl refusal, unchanged
checksum conflict, preserved pre/post-dispatch 404 evidence, lost-reply refusal,
exact recovery and yanked reconciliation. No actual upload occurs in these local
bare-remote/mocked-transport checks. [#12](https://github.com/dragginzgame/ic-auth/issues/12)
retains native consumer acceptance separately from upstream
[Shared #94](https://github.com/dragginzgame/shared-tooling/issues/94).

The initial continuation preserved Host 0.10.1 and Testkit 0.28.0. The later
incoming Testkit 0.28.1 lock and its qualification are recorded below; the
earlier input hashes retain their original identities. The read-only Canic review at base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6` finds both Auth packages at 0.2.7 in
its dirty root lock and the standalone-proof call integrated. The local
complete-token verifier/proof-skipping cache and discarded installation
acknowledgement remain. The [adoption contract](../canic-adoption.md) selects
published 0.2.8 and supplies precise protected input/result projections for
[Canic #58](https://github.com/dragginzgame/canic/issues/58). Effective renewal
deadlines remain [#506](https://github.com/dragginzgame/canic/issues/506), and the
real browser issuer adapter still needs qualified non-issuing reconciliation
under [#507](https://github.com/dragginzgame/canic/issues/507). No new Auth Rust
API or wallet service is required for those consumer repairs.

Evidence is retained under `target/continuation-0.2.8-live/`, including red and
final publication runs, selected committed upstream source and reviewed Canic
hashes. Publication and registry fixtures pass on Linux Bash 5 and Bash 3.2.
The actual consumer formatting-hook/manifest perturbation checks, selected
upstream formatting/retention fixtures, actual tool-evidence caller, Actionlint,
ShellCheck, snapshot, dependency pins, local links and formatting pass. Incoming
manifest/lock/client hashes remain unchanged. Full local CI and native hosted
qualification of this pending tree remain separate.
The selected draft **0.2.9** is compatible: no library API, signed bytes,
stored format or package version changes. No local function, method or type is
removed; only duplicate transport/parsing inside the retained `observe` adapter
is replaced. No sibling source mutation, full gate, commit/push, publication,
npm release or deployment occurs here.

### Testkit 0.28.1 admission repair

At committed preparation source `1e11eeb`, the maintainer's incoming lock selects
published Testkit 0.28.1 while only the 0.28.0 CLI was prepared. The full gate
built the qualification Wasm, then refused offline admission with `selected Cargo
tool is not installed`; Cargo itself was available. The original failure remains
in `target/validation-failures/20261010T080312Z-2277050-17-test-qualification.log`.
Direct admission reproduces that exact prerequisite failure. Registry digest and
downloaded VCS metadata match released Testkit
`ebbea97c90161eb03834b01a9fb0159aa0d99d89`; no root dependency selection changed.

Explicit `make install-testkit-tools` prepares the selected 0.28.1 CLI and reuses
the existing admitted PocketIC 16.1.0 server. The former CLI/server and root
manifest/lock hashes remain unchanged. The consumer now reports the selected
CLI/version and explicit setup command when shared CLI admission fails, retaining
the original exit status and no success output. It never installs during a check
or selects a retained older CLI; the shared installer snapshot is untouched.
[#14](https://github.com/dragginzgame/ic-auth/issues/14) tracks this repair.

The new diagnostic fixture fails before the repair and passes afterward on
Linux Bash 5 and genuine Bash 3.2, alongside existing setup/check/changed-lock
failure and qualification/evidence cases. Both actual certification/composed-root,
protected-upgrade and fresh-key signed-ingress tests pass on the incoming graph.
The qualification runner's all-target/all-feature check passes on Rust 1.88;
ShellCheck, snapshot integrity, local links and diff checks also pass.
Evidence is retained under `target/testkit-qualification-repair/`. The existing
compatible 0.2.9 draft is extended without API/format/version changes or removed
symbols. No full gate, release/publication or sibling mutation occurs in this
repair; earlier 0.28.0 results remain historical evidence.

## Earlier client completion deadline repair (0.2.8, released)

The following records local preparation before maintainer delivery. Pending
versions and publication/CI observations in this section are historical.

The maintainer confirmed 0.2.7 live. Remote main and peeled tag match the source
above; annotated tag object `d993631e95c19149458406255745208cfe6f5b59` and both
official unyanked registry checksums match retained publication intent. Its
[main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37960389838) is
queued at inspection. No native macOS acceptance is inferred from publication.

The initial client continuation reviewed Shared Tooling 0.2.8 at
`b2646cde9abbc8861857a4379c683a0c19eba43e`, Host 0.9.7 at
`ca62e661918db2f4320743b9042a4993a5fff2aa` and Testkit 0.27.2 at
`1a8f2ff570ea1c3bd58b215e28af52e5d99870e8`. Dirty Shared sibling work is
excluded. The 80-file snapshot and Testkit/server selection remain unchanged;
the subsequent direct Host upgrade is recorded below.
[#12](https://github.com/dragginzgame/ic-auth/issues/12) still
awaits the payload-observation contract proposed in
[Shared #94](https://github.com/dragginzgame/shared-tooling/issues/94).

A new red regression shows successful ready-token storage returning completed
material after the original retrieval deadline. The client now rechecks the
exclusive deadline after the successful commit and generation check, before
returning to callers sharing the flight. At exact expiry and one nanosecond
later both callers receive `retrieval_expired`; committed bytes/revision remain
unchanged with no extra request ID, issuer call or session invalidation. A fresh
client can reuse the still-valid completed cache without replaying issuance or
retrieval. Completion one nanosecond before expiry succeeds; an obsolete login
still reports `stale_generation` first and preserves the new session.

All 46 client cases and both Rust/TypeScript Candid directions pass on Linux.
Red and final results and exact input hashes are retained in
`target/continuation-0.2.7-live/`. The selected next draft is **0.2.8**, a
compatible correction to the existing completion deadline. No public signature,
Candid format, stored profile or package version changes; no migration/reset or
removed functions, methods or types. Client qualification did not rerun unrelated
native/library product suites, the complete gate or publication. The requested
Host adoption below independently qualifies the native graph. No sibling edit,
commit, push, release, npm publication or deployment occurred. The private client
remains source-only; [#3](https://github.com/dragginzgame/ic-auth/issues/3) and
[Canic #507](https://github.com/dragginzgame/canic/issues/507) retain real adapter
and non-issuing reconciliation acceptance.

### Direct Host 0.10 adoption

The maintainer requested the new Host libraries. The incoming root catalog and
lock already selected `ic-host-fs`/`ic-host-artifacts` 0.10.0 for the native
utility. Reviewed source `98562bea26a98993d93b80ed908bea4876c32a91` matches
remote main/peeled tag and downloaded VCS metadata; both unyanked registry
checksums match the incoming lock. The sibling's later uncommitted filesystem
work is excluded.

The pre-adaptation check reproduced the removed `write_typed_with` import error.
The utility now uses the published three-argument `write_with`, preserving
explicit create/replace modes, 0o600 permissions, bounded no-follow reads and
typed publication errors. No fallback writer, retry policy or stored format was
introduced; no local function, method or type was removed. This extends the
compatible **0.2.8** draft without changing library APIs or protocol dependencies.

Initial Host 0.10 adoption used Testkit 0.27.2 with its admitted PocketIC 16.1.0
server. That package required all four Host packages at 0.9 and retained 0.9.7
in its separate graph. The later Testkit 0.28 qualification below removes that
constraint through the published owner adoption in
[Testkit #44](https://github.com/dragginzgame/ic-testkit/issues/44); IC Auth does
not patch or edit that sibling. Initial evidence is retained in
`target/host-0.10-adoption/`, including the initial compile refusal and exact
incoming manifest/lock/client hashes. Earlier client results retain their own
input identity.

On Linux, five native file-operation cases, both actual Testkit certification,
composed-root/upgrade and fresh-key ingress scenarios, Rust 1.88 all-target and
all-feature application checks, strict affected-package Clippy and library
dependency boundaries pass. The local/mocked release-publication fixture passes
with the current Host writer. Formatting, snapshot, dependency pins and local
documentation checks pass; the incoming manifest/lock/client bytes and complete
CI roster remain unchanged. Native macOS/full gate acceptance, actual release
and upload remain separate; none ran during this upgrade.

### Latest published Host/Testkit graph and Shared review

The upstream review finds published Host 0.10.1 at
`c7bdc3d4e1c658957202eebd76bff2c51e22f645` and Testkit 0.28.0 at
`48cfff270d5e228c0e09d6ee452786fb14c9a66e`. Both main/peeled tags, all five
unyanked registry checksums and downloaded VCS identities match the incoming
lock. That selection unifies all four Host packages at 0.10.1 and removes the
former 0.9 graph. Host completes missing-parent sync when a competing directory
creator wins; Testkit adopts the consolidated writer and retained publication
errors. No local adapter change is needed beyond the prior `write_with` adoption.

Offline admission first refused the unprepared 0.28 CLI. Explicit locked fetch
and `make install-testkit-tools` then prepared that exact selection. Offline
check admits its existing 16.1.0 server. The previous 0.27.2 CLI and server bytes
remain unchanged; no installation/reset or unknown state was discarded. Five
native utility cases, the Testkit caller setup/check/failure-evidence fixture,
Rust 1.88 application checks, strict affected-package Clippy and library
boundaries pass on the incoming graph. Evidence is retained in
`target/upstream-0.2.8/`, separately from the earlier mixed graph.
Both actual certification/composed-root/upgrade/fresh-key ingress cases and the
local/mocked release-publication fixture also pass. Formatting, snapshot, pins
and local documentation checks pass; the complete CI roster and incoming
manifest/lock/client bytes remain unchanged.

Shared Tooling 0.2.9 `f8a70ba348e9975a6eb5b337860b00bc8a0b36d1` is committed
and reviewed. It adds exact-source version annotations to snapshot export,
complete declared-file integrity to fleet reports, a combined issue/PR dashboard
and authenticated current-run artifact readback in its CI. The baseline and
Make admission source are unchanged. This inspection retains Auth's selected
0.2.8 snapshot; its next refresh should use the canonical exporter. The separate
CI adoption opportunity is tracked in
[#13](https://github.com/dragginzgame/ic-auth/issues/13), preserving exact artifact
IDs, job-scoped read permission and independent payload verification. No Auth
readback failure or hosted acceptance is inferred. Shared #94 remains a proposal
and does not supply #12's registry-payload contract.

Exact-source upstream CI for Shared, Host and Testkit is queued at inspection;
published payloads and local consumer results remain separate from native
upstream acceptance. The compatible 0.2.8 draft now covers this unified native
selection alongside the existing client fix, without public library API, stored
format or package version changes. No sibling edit, commit/push, release or
publication occurred.

## Earlier Shared Tooling, Host and client admission update (0.2.7, released)

The following records local preparation before maintainer delivery; pending
version, publication and CI observations in this section are historical.

The maintainer confirmed 0.2.6 live. Its remote main and peeled tag matched
`9bdfe5d843d63d90ab17582525881330942ceeb5`; annotated tag object `53d35e1802d86867dda9d8c5158c5be26ca6b157`
and both official unyanked registry checksums match retained publication intent.
Its [main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37956356512)
is queued at inspection; focused local results do not establish native macOS
acceptance. [#11](https://github.com/dragginzgame/ic-auth/issues/11) remains open.

The canonical exporter refreshes the 80-file snapshot from committed Shared
Tooling 0.2.8 `b2646cde9abbc8861857a4379c683a0c19eba43e`, verified against remote
main. The sibling's uncommitted distribution work is excluded. Make admission
now selects its companion probe relative to the included snapshot, and uses
`MAKE_COMMAND` for the isolated safety probe so recursive `MAKE` arguments do
not interfere. Ambient root substitution, missing companions, recursive extra
Makefiles/arguments and unsafe modes are covered by the upstream focused tests.
IC Auth's release policy, execution probe and complete CI roster are unchanged.

The initial qualification selected published Host 0.9.6 and preserved that lock
byte-for-byte.
Its reviewed source is `6aaa4229913bafc68a443e7e855468efcca8ee7a`; the official
unyanked `ic-host-fs` checksum matches the selection. This release folds bounded
text-hex decoding into the existing decoder with unchanged public response
contracts. No local API adaptation is required. IC Testkit remains 0.27.2 at
`1a8f2ff570ea1c3bd58b215e28af52e5d99870e8`, with its already prepared CLI and
PocketIC 16.1.0 server. The initial offline check refused uncached Host packages;
explicit locked fetch prepared them without changing the lock.

Five native Host utility cases and both actual certification/composed-root,
protected-upgrade and fresh-key ingress scenarios pass on Linux. Rust 1.88
all-target/all-feature application checks, strict affected-package Clippy and
library dependency boundaries pass. Under Bash 5 and genuine Bash 3.2, the
focused Shared release/format Make tests, consumer release/publication and tool
failure-evidence fixtures, and actual consumer formatting/hook preservation
checks pass. Evidence, including the initial cache refusal and exact input
hashes, is retained in `target/continuation-0.2.6-live/`.

The selected next draft is **0.2.7**, a compatible maintained-tooling and native
dependency update with the client admission repair below. Public Rust APIs,
generated contracts, stored profiles, client lock and package versions remain
unchanged. No migration or reset
is required; no local functions, methods or types were removed. No complete
local gate, native macOS acceptance, sibling mutation, commit, push, publication
or deployment occurred. [#12](https://github.com/dragginzgame/ic-auth/issues/12)
awaits the bounded payload-readback contract proposed in
[Shared #94](https://github.com/dragginzgame/shared-tooling/issues/94); the current
presence-only helper is insufficient and this revision does not implement it.

### Client storage admission and latest Host review

The subsequent upstream check still selects Shared Tooling 0.2.8 and Testkit
0.27.2. Host 0.9.7 is published and unyanked; its main/peeled tag match
`ca62e661918db2f4320743b9042a4993a5fff2aa`. The incoming lock already selected
its four packages and is preserved exactly. Runtime sources are unchanged from
0.9.6; the release adopts the same Shared Make fixes already selected here.
All four unyanked registry checksums match the lock, and downloaded VCS metadata
matches the reviewed release source.
Registry refresh perturbed six unrelated Windows dependency edges; those were
restored to the incoming bytes before final consumer qualification.

Three red regressions show a fresh request and expired prepare intent being
reserved after a delayed initial storage read, a lost CAS reservation or a
definitive TTL rejection's delayed storage clear. The client now checks the
original exclusive deadline before allocating a request ID or reserving fresh
intent. At exact expiry and one nanosecond later, it preserves the empty slot or
completed clear and revision without another ID, write, issuer call or session
invalidation. Storage completion one nanosecond before expiry can still reserve
and finish issuance. The repair preserves uncertain intent and does not cancel
transactions already begun before expiry. It changes no public signature, Candid
format or stored profile, and removes no function, method or type.

All 43 client cases and both Rust/TypeScript Candid directions pass on Linux.
Five native Host utility cases, both actual Testkit certification/upgrade/ingress
scenarios, Rust 1.88 application checks, strict affected-package Clippy, library
boundaries, formatting, snapshot, pins and local documentation checks pass on
the final incoming Host 0.9.7 lock. The actual consumer release/publication
fixture also passes with local bare remotes and mocked registry/upload effects.
Exact-source 0.2.6 main CI remains queued at
this inspection; native macOS acceptance is separate from these local results.

Evidence is retained in `target/continuation-0.2.7/`, separately from the earlier
0.9.6 qualification. [#3](https://github.com/dragginzgame/ic-auth/issues/3) retains
real issuer-adapter acceptance; injected client fixtures do not qualify Canic's
endpoints. [Canic #507](https://github.com/dragginzgame/canic/issues/507) continues
to own non-issuing reconciliation after replay receipt expiry. No sibling was
edited, and no complete gate, release or publication ran.

## Earlier client deadline repair and dependency qualification (0.2.6, released)

The following records local preparation before maintainer delivery; pending
version, publication and CI observations in this section are historical.

The maintainer confirmed 0.2.5 live and requested continued integration work.
The pushed main/peeled tag matched `6dec8b9463526536f3f9f888082ac26cc9a0a727`; annotated tag object
`a847f4131319aff66c05dabe22ed22c8a3e03d04` and both official unyanked registry
checksums match retained publication intent. Its
[main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37949883587)
is still queued at inspection. Issue #11 remains open for native acceptance.

Prepare and reconciliation previously processed replies after the saved operation
deadline, unlike retrieval. New regressions reproduce a late prepared reply
changing retained intent before expiry refusal. The client now checks its
exclusive deadline after the reply and current-generation check, before outcome
processing. At exact expiry and one nanosecond later, prepared/no-effect TTL/session
replies preserve original bytes/revision across reloads, allocate no new ID or
retrieval and do not invalidate the session. Replies one nanosecond before expiry
still complete retrieval. All 39 client cases and both Rust/TypeScript Candid
directions pass. This uses injected transport/shared memory, not Canic endpoints.

Earlier qualification used an incoming Host 0.9.4 lock, preserved byte-for-byte.
The latest selections are qualified below. Published
Host source `4e3daebd5df07c6449279535668436024a45c02b` matches remote main/tag;
crate sources are unchanged from 0.9.3, with version metadata and shared Make
adoption at their owner. Five native file-helper cases and both actual Testkit
certification/composed-root/protected-upgrade/fresh-key ingress scenarios pass.
The sandbox first refused localhost binding; that failure is retained, and the
same offline qualification passes with binding enabled. Testkit remains 0.27.1
with its admitted 16.1.0 server. Rust 1.88 all-target/all-feature application
checks and strict affected-package Clippy pass.

Evidence is retained in `target/continuation-0.2.5-live/`, including failing
client reproductions, final acceptance and the unchanged incoming lock hash.
The next draft is **0.2.6**: a compatible client deadline correction, native
dependency qualification and the Shared Tooling fixes below. Public Rust APIs,
generated contracts, storage profile
and package versions are unchanged; no migration or retained-state reset is
required. No functions, methods or types were removed. No full local gate,
sibling edit/build, commit, push, publication or deployment occurred. Canic's
existing #58/#506 verifier/deadline adoption and #507 non-issuing reconciliation
remain at their owning issues; the private client remains unpublished on npm.

### Shared Tooling 0.2.7 adoption

The maintainer requested the new shared revision. Remote main matches reviewed
`47d6ae6488b8007323fa7c2e22a6efa11d77ae63` (0.2.7 heading). The canonical exporter
refreshes 80 selected files, adding `make/execution.mk` explicitly; the baseline
itself is unchanged. The release and tool-evidence consumer fixtures select the
new companion and its existing execution probe. Shared entrypoints reject unsafe Make modes
before dispatch; release qualification stays bound to its disposable root and
formatting qualification preserves newline-ending paths and Git object custody.

On Linux, Bash 5 and genuine Bash 3.2 pass the consumer release/publication and
failure-evidence fixtures, actual consumer sorting/formatting/hook preservation
checks, and focused Shared owner release/format/hook suites. The latter include
direct/inherited unsafe-mode refusal, quoted variables/parallel Make, external
root confinement and real Cargo formatting in a newline-ending fixture. They are
distinguished from native macOS and real release acceptance. No source hooks
were activated or new tools installed. Evidence is retained in
`target/shared-tooling-0.2.7/`; the first export refused a differently spelled
clone origin URL, then passed after matching the recorded source exactly.

The client and incoming lock hashes were unchanged during that adoption, and the
complete CI roster matched its pre-adoption bytes. Issue #11 remains open for delivered native
acceptance. No Rust product suite, complete gate, commit/push, publication or
sibling mutation ran for this adoption. Shared #93 is a separate
artifact-readback proposal and is not part of this reviewed revision.

### Latest Host and Testkit qualification

The maintainer requested the latest upstream review. The new incoming lock
selects Host **0.9.5** and IC Testkit **0.27.2**; it is preserved byte-for-byte.
Host source `0f61811c88a6b6b4b026b04ca16e428409be877f` and Testkit source
`1a8f2ff570ea1c3bd58b215e28af52e5d99870e8` match their remote main/peeled tags
and the downloaded packages' VCS metadata. Official unyanked registry checksums
match the lock selections. Both releases change shared tooling; Host crate
sources and Testkit runtime/CLI sources are unchanged from the prior selections.
No IC Auth API adaptation or new dependency is required.

The initial offline metadata check refused uncached packages; selected CLI
admission also refused the unprepared version. Explicit locked fetch and
`make install-testkit-tools` then prepared the exact 0.27.2 CLI, admitted by its
selected-install receipt, and retained Testkit's existing 16.1.0 server. Prior
installations and refusal logs remain retained; ordinary checks stay offline.
The CLI exposes setup/check/run rather than a standalone version command.

Five native Host utility tests, the substituted Testkit caller failure/selection
fixture, both actual certification/composed-root/protected-upgrade/fresh-key
ingress scenarios, Rust 1.88 all-target/all-feature application checks, strict
affected-package Clippy and library dependency boundaries pass on Linux.
Logs and exact input hashes are in `target/host-testkit-latest/`. Pending client
and snapshot bytes remain unchanged. The earlier 0.9.4/0.27.1 evidence above is
retained separately; current qualification uses 0.9.5/0.27.2. Upstream native CI
is queued at inspection, separately from these consumer results. This extends
the existing compatible 0.2.6 draft without package version changes, full gate,
commit/push, release, sibling edits or deployment.

## Earlier client recovery and shared Make adoption (0.2.5, released)

The following records local preparation before maintainer delivery; pending
version, publication and CI observations in this section are historical.

The maintainer confirmed 0.2.4 live and requested continued issue work and help
for Canic integration. Remote main/peeled tag matched
`64e1d9b90bb27e50280e3845e86db336df4f04e3` during that continuation; the
annotated tag object is `32313542a51d6c08f18f78a6296163131d931063`. Both official
registry records are unyanked and match retained publication intent. Its
[main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37944459924)
is queued at inspection; publication does not establish native acceptance.

The client now has a regression for a possibly completed prepare whose reply
was lost. A new client instance reconciles just before the original stored
deadline; at that exclusive deadline, further reloads make no issuer calls,
allocate no request IDs, preserve exact intent/revision and do not invalidate
the authenticated session. Selecting a longer later operation lifetime cannot
restart the saved deadline. This qualifies existing behavior using injected
transport and a shared memory store, not an actual Canic endpoint or browser
reload. The [client contract](../browser-client.md) distinguishes bounded
automatic work from the owner's obligation to reconcile unknown effects.

Read-only review at Canic `ac55e50334dd6479ec36f404e89e60bcfe9184d6` confirms
its existing prepare replay authenticates caller/payload and recovers staged
responses. The reviewed replay/prepare files are committed and unchanged;
unrelated issuer-adoption work remains dirty. After committed receipt expiry,
another fresh prepare can prune it. Replaying the original ID then reserves a
new relative deadline and can prepare different claims. This is source-traced
evidence, not runtime reproduction. [Canic #507](https://github.com/dragginzgame/canic/issues/507)
owns non-issuing reconciliation/qualified replay and the actual pruning/delayed
request acceptance. IC Auth #3 remains open for a real adapter and coordinated
adoption. Existing complete-verifier/cache retirement (#58/#491) and effective
installation deadline propagation (#506) remain Canic-owned, with their existing
published library APIs available now.

`make test-client` passes all 36 lifecycle/storage/codec/example cases and both
Rust/TypeScript Candid directions using explicitly selected Node 24.21.0 and
npm 12.2.0. Source and evidence are retained in
`target/continuation-0.2.4-live/`, including `client-tests.log`, exact release/CI
observations and `canic-recovery-inputs.sha256`. Public APIs, runtime behavior,
wire/storage formats, dependencies and package versions are unchanged. The next
draft is **0.2.5**, compatible coverage, integration documentation and the shared
Make adoption below; no retained storage disposition or migration is required.
No functions, methods or types
were removed. No full local CI, sibling build/edit, commit, push, release,
publication, npm delivery or deployment occurred.

[0.2.1 main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37927487960)
now passes complete Linux, Intel macOS and Apple Silicon jobs at exact source
`9cd4cf09dde6ce0b739ed58af18612ed02202841`. This supplies the remaining native
acceptance for issue #9, now closed. It does not qualify later sources or this
new client regression. No queued run was cancelled, rerun or dispatched.

During continuation, [IC Auth #11](https://github.com/dragginzgame/ic-auth/issues/11)
identified the now-committed Shared Tooling Make adoption. The canonical exporter
refreshed the snapshot to 79 files at reviewed remote main
`ce13a5314916891fd239d9b199b4a91b04775054` (0.2.6 heading), using an isolated exact
source checkout. `make/release.mk` and `make/rust-format.mk` now own the existing
entrypoints; replaced local recipes/defaults and redundant phony declarations
are removed. The target names remain. Direct delivery, destinations, local build
output, explicit locked fetch/cache preparation, metadata adapters and the
complete CI roster are retained. The baseline itself is unchanged. No sibling
checkout was changed, tool installed or real hook configuration activated.

The existing release fixture now invokes the canonical checker against the
actual Makefile, selecting both includes explicitly and substituting only runner
effects. All increments, exact resume/version/destination arguments, conflicts
and failures pass on Bash 5 and genuine Bash 3.2. Actual consumer formatting
hook checks pass on both shells, including real sorting/rustfmt, partial staging,
formatter failure and unrelated-file/lock preservation. Both real formatting
targets reject missing/wrong formatter selection without source changes. A
separate owner include fixture covers substituted-Cargo command ordering and
failure propagation; it is distinguished from the real consumer hook checks.
The local/mocked release/publication reconciliation suite passes on both shells.
Default-goal/environment checks verify harmless help, direct policy, selected
destinations, checkout-local output and toolchain admission; before/after CI
roster bytes match. Formatting, ShellCheck, snapshot/declaration/documentation
guards and diff whitespace pass. Logs and retained source/export inputs are in
`target/continuation-0.2.4-live/make-adoption/`; an initial fixture launch lacked
its TMPDIR parent, was retained, and passed after explicit directory preparation.

Issue #11 remains open for delivered exact-source native macOS acceptance; these
are Linux/Bash portability and substituted-effect checks, not native macOS or
real release qualification. Client input hashes remain unchanged, and final
Make/snapshot identities are recorded separately in
`selected-inputs-after-shared.sha256`. No additional Rust/canister product suite
or complete local gate ran for the Make adoption.

### Tool failure-evidence fixture repair (pending 0.2.5)

The maintainer's release gate at committed source `2f7594d` failed in
`test-tools-evidence`. Retained fixture `tools-evidence.MqwAQ0` shows that Make
stopped during parsing: the fixture copied the actual Makefile but omitted its
new release/format includes. The consumer-owned fixture now copies both
includes. Its Cargo installation fault, missing-tool refusal, original aggregate
status, logging-failure precedence and exact four-file archive checksum checks
remain intact. No selected shared source or product contract changed.

`make test-tools-evidence` and the same script under genuine Bash 3.2 both pass;
ShellCheck passes. Logs are retained in `target/tools-evidence-make-fix/`, with
passing fixtures `tools-evidence.6bqluV` and `tools-evidence.JDoEKk`. The original
failed fixture and validation logs are preserved. This compatible fix extends
the existing 0.2.5 draft and [issue #11](https://github.com/dragginzgame/ic-auth/issues/11).
The complete gate/release was not rerun; native macOS acceptance remains pending.

## Earlier compatible improvements for 0.2.4 (released)

The following records local preparation before maintainer delivery; pending
version, publication and CI observations in this section are historical.

The maintainer requested new Host/Shared Tooling review, improvements and help
for Canic integration. The incoming lock edit selecting Host 0.9.3 and Testkit
0.27.1 was preserved and explicitly prepared. Host remote main is
`545e7236b91d84e190c80931b784f72cc4fafb11`; its 0.9.3 release changes Shared
Tooling adoption, not Rust library APIs. Canonical Testkit CLI installation and
offline admission retain its selected PocketIC 16.1.0 server.

Shared Tooling remote main remains the already adopted
`04e07b4bf54e7aeb03eb7804a845cee27b7305df`, with 77 selected files. The sibling's
new release/format Make includes are uncommitted owner work, tracked in
[Shared Tooling #91](https://github.com/dragginzgame/shared-tooling/issues/91) and
[#92](https://github.com/dragginzgame/shared-tooling/issues/92). No dirty source
was imported or sibling checkout changed; adoption requires a reviewed commit.

Two real signed-token regressions demonstrate that earlier success does not
extend issuer certificate freshness or pin the IC network trust anchor. Exact
age equality accepts; advancing the clock, tightening freshness or changing the
protected BLS key rejects the same untouched token. The
[token contract](../tokens.md#canic-verifier-integration) explains the protected
Canic adapter inputs and why the returned deadline is not a cache lease.
Public APIs, signed bytes, features and package versions remain unchanged.
The numbered next changelog is **0.2.4**, a compatible coverage/tooling batch.

At Canic committed base `ac55e50334dd6479ec36f404e89e60bcfe9184d6`, the owner's
[acceptance comment](https://github.com/dragginzgame/canic/issues/491#issuecomment-6082545028)
records 28 focused cases for the issuer's published standalone-proof adapter,
including policy-capped deadlines and native/Wasm checks. That evidence belongs
to the owner and its retained working inputs; it does not establish deployed
adoption or qualify IC Auth's current changes. Read-only coordinator inspection
found ROOT discarding installation responses and retaining declared deadlines.
[Canic #506](https://github.com/dragginzgame/canic/issues/506) owns the repair and
its rejection/renewal acceptance. Full-token verifier and proof-cache retirement
remain coordinated in [Canic #491](https://github.com/dragginzgame/canic/issues/491)
and [#58](https://github.com/dragginzgame/canic/issues/58); no new library API is
needed for either step.

Focused Linux checks pass: 25 token/proof cases, five native file-helper cases,
both real certification/composed-root/protected-upgrade/fresh-key ingress
scenarios, Rust 1.88 application checks and 39 token/proof/session cases, and
warning-denied Clippy of the affected library/native applications. Evidence and
input hashes are retained under `target/continuation-0.2.4/`; the actual runtime
state is retained at the path printed in `qualification.log`. This is local
Linux qualification of the incoming graph, not its native macOS acceptance.
Formatting, snapshot integrity, dependency declarations/boundaries, 256 local
documentation references and diff whitespace pass. Testkit setup/admission and
failure-retention caller fixtures pass on Bash 5 and genuine Bash 3.2; actual
offline owner CLI admission also passes. The recorded source/lock hashes still
match after qualification. Logs are `guards.log`, `testkit-check.log` and
`testkit-callers-bash32.log`, alongside the focused Rust/runtime logs.
No functions, methods or types were removed. No full local CI, sibling edit,
package version mutation, commit, push, publication or deployment occurred.

The completed [0.2.0 main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37923280901),
source `7291bf70426cfa9ca03403155ab249e2d978d88e`, now supplies complete Linux,
Intel macOS and Apple Silicon acceptance for the delivered failure collector,
Testkit setup hard cut and Candid transport coverage. All jobs passed the real
negative-path evidence upload/download/byte verification and complete gate;
IC Auth #2/#7/#8 are closed with that matching acceptance. Its failed tag run is
not relabelled as successful. This older source selects Host FS 0.9.1/Testkit
0.26.0 and does not qualify the current graph. The 0.2.1 workflow still awaits
matching native acceptance for #9; real browser issuer adoption remains #3.

## Earlier Shared Tooling preparation for 0.2.3 (released)

The following records preparation before maintainer delivery; pending-version
and manifest wording in this section is historical.

The maintainer requested the latest Shared Tooling and continued issue fixes.
The reviewed remote main revision is
`04e07b4bf54e7aeb03eb7804a845cee27b7305df` (owner's 0.2.5 heading).
The canonical exporter refreshed the existing 77-file selection from an isolated
immutable source checkout; no sibling files were changed or extra upstream test
roster added to this consumer. Local AGENTS.md records that exact revision.
The baseline itself is unchanged. Relevant executable fixes preserve final
unterminated IC matrix records and literal trailing newlines in Git hook paths,
including failed-read status; companion-selection guidance is also refreshed.
This is compatible developer tooling, so the undated next changelog is **0.2.3**.
Package manifests and lock remain 0.2.2.

Canonical hook/IC installer fixtures pass on Bash 5 and genuine Bash 3.2 using
an isolated owner source tree. Consumer formatting-hook adoption checks also
pass on both shells against this checkout's actual hook, installer and Make
formatting targets. Owner fixture results are distinguished from consumer
caller acceptance. Logs and the exact source archive/export evidence are retained
under `target/continuation-0.2.3/`. The local/mocked release/publication
reconciliation fixture passes, as do actual offline `make tools-check`, ShellCheck,
77-file snapshot integrity, dependency declarations, 255 local links and diff
whitespace. The effective hook path remains `.githooks`; no config activation
was changed. Logs are `shared-{hooks,ic-tools}-bash{5,32}.log`,
`consumer-hooks-bash{5,32}.log`, `release-tools.log`, `tools-check.log`,
`shellcheck.log` and `guards.log`. Selected input hashes and the working-tree diff
are retained. No authentication library code or dependency selection changed;
no product Rust/canister suite or complete CI gate ran for this setup adoption.
No functions, methods or types were removed.

## Released 0.2.2 and issue acceptance review

Remote main and peeled `v0.2.2` match the current release source above; the
annotated tag object is `89db3ccdec0c4424370dcd06939fdded42da3e11`.
Both official registry entries are unyanked and match retained publication
checksums. Its exact-source
[main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37936975452)
is queued at inspection. The standalone proof API is published and its consumer
acceptance remains owned by Canic #491, separately from library issue #10.
A final read-only review at Canic committed base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6` observes its changing working tree
selecting both 0.2.2 packages with matching registry checksums; this is selection
evidence, not consumer qualification.

The earlier accepted source `5210a34cc57d11301709af19877d1a5f0a100f36` has
successful complete Linux, Intel macOS and Apple Silicon jobs in
[0.1.14 main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37914751465).
This supplies the outstanding native acceptance for the delivered MSRV (#4),
fleet reporter (#5) and streaming file hash (#6) repairs. It does not qualify
later dependency selections or unrelated later source. Those fixes remain
present; their relevant MSRV wrapper/file helper source is unchanged. Issues
#4/#5/#6 are closed with this exact-source acceptance. Library API issue #10 is
closed after 0.2.2 publication and its retained native/Wasm/MSRV qualification;
Canic adoption and verifier retirement stay open in the consumer tracker.

The 0.2.0 tag's Apple Silicon job failed at exact evidence download before its
validation gate. `gh run view --log-failed` returned no log bytes; the job
annotation reports “None of the provided artifact IDs were found.”
Retained job/annotation observations identify the transport failure but not its
cause; no speculative retry or workflow repair is introduced.
Keep #2 and newer native acceptance (#7/#8/#9) open pending their own matching
results. The private browser client's real issuer adapter and adoption (#3)
remain consumer-coordinated work. No queued workflow was cancelled or rerun.
No commit, package version change, release, publication or sibling edit occurred.

## Earlier 0.2.2 preparation (released)

The following records local preparation before maintainer release/publication;
pending manifest, qualification and delivery wording in this section is historical.

The maintainer authorized continuing the standalone proof batch for **0.2.2**.
It is an additive public API under the existing `token-verification` feature,
with no wire, signed-byte, dependency, feature-selection or existing verifier
contract change. Manifests/lock remain 0.2.1 until explicit release preparation.

`token::verify_delegation_proof` accepts the existing proof directly with
protected expected issuer, root key policy, clock and finite proof bounds. It
returns private-field `VerifiedDelegationProof` evidence with a borrowed
certificate, canonical hash and policy-capped exclusive deadline. Certificate
rules and material accounting are shared with complete token verification; the
same root engine checks bindings, witness, low-s ECDSA and live authority.
The complete-token validation order and existing public error surface are
preserved. No token fabrication, verification callback or new host effect exists.

The root-signed issuer leaf excludes certificate issue-time metadata. The new
result validates its consistency but does not authenticate that metadata as a
root-signed timestamp; complete tokens bind the full certificate hash through
issuer-signed claims. This distinction is covered by a focused test and the
[contract](../tokens.md#standalone-root-delegation-verification).
Canic retains installation/issuance approval, protected network/key and
application policy, renewal, storage and certification composition.

[IC Auth #10](https://github.com/dragginzgame/ic-auth/issues/10) owns this batch,
coordinated under [Canic #491](https://github.com/dragginzgame/canic/issues/491).
Focused evidence is retained in `target/continuation-0.2.2/`; the source tests
cover real signed proofs, wrong root/issuer/key/path/seed/header/leaf/witness,
scalar/high-s failures, exact material/time/TTL bounds, canonical grants and
live authority invalidation. Existing complete-token and session tests run
alongside the new API.

Focused checks pass: 23 proof/token cases on Rust 1.99, 14 session cases,
13 canonical-vector cases and the library's existing unit case; Rust 1.88 runs
all 37 proof/token/session cases successfully. `make check-wasm` passes every
selected library feature. `make check-msrv` passes isolated Cargo package payloads
for all supported features on native/Wasm and the separately checked internal
applications. Warning-denied Clippy of all IC Auth library targets/features,
formatting, 77-file snapshot integrity, declarations, transitive dependency
boundaries, 254 local documentation references and diff whitespace pass.
Logs are `library-tests.log`, `msrv-tests.log`, `wasm-msrv.log`,
`library-clippy.log`, `fmt.log`, `guards.log` and `prose-final.log` beneath the
same evidence directory. The MSRV payload identities remain in the fixture
printed by `wasm-msrv.log`; a later module-comment wrap changes no code contract.
This is local Linux/native and cross-compilation evidence. The complete local CI
and release gate, native macOS, live IC effects and Canic adapter acceptance were
not run for this pure verifier batch. No functions, methods or types were removed;
existing private certificate blocks and budget accounting were extracted for reuse. No sibling edits, package version changes, commit, push,
release or publication occurred.

## Released 0.2.1 and Canic gap review

The annotated tag object is `8dc86ad90a094c476385977c1ff57e0fd16344d3`;
remote main and the peeled tag match the release source above. Both unyanked
registry entries match `.git/publication-state/0.2.1.json`.
The exact-source [main workflow](https://github.com/dragginzgame/ic-auth/actions/runs/37927487960)
is queued at inspection, with no duplicate automatic tag run. Keep IC Auth
#7/#8/#9 open pending their matching native acceptance; publication is not proof
of that qualification.

Canic's changing working tree at committed base
`c4c046f947b2b28f4342cbf6efe9221ba1ed5f70` now uses compatible `0.2` catalog
requirements and resolves both Auth packages at 0.2.1 with matching checksums.
This selection does not requalify its previously accepted input snapshots.
The complete runtime token verifier can be adopted with the existing API.

One concrete pre-token extraction gap is recorded in
[IC Auth #10](https://github.com/dragginzgame/ic-auth/issues/10), coordinated under
[Canic #491](https://github.com/dragginzgame/canic/issues/491): active issuer proof
installation verifies a root delegation before a token exists. The library's
root verifier is private and depends on complete-token prechecks. A standalone
bounded proof boundary must share the canonical verifier and independently
validate protected identity, key, time and material inputs. It must not invent a
token or expose an unchecked helper. This is proposed follow-up work, not an API
shipped in 0.2.1. Canic retains installation authority, live policy, renewal,
storage and certification composition.

Inspection hashes, registry/consumer selection evidence and issue bodies are
retained in `target/continuation-0.2.1/canic-gap-review/`. No sibling edits,
builds, package version changes or release effects occurred during this review.
Only publication/adoption documentation was aligned; local links, snapshot
integrity and diff whitespace are the relevant checks. No new release candidate
is selected for this governance-only review.

## Earlier 0.2.1 preparation (released)

The following records preparation before maintainer delivery; pending and
uncommitted wording in this section is historical.

The maintainer requested continued IC Auth work while Canic catches up. The
compatible pending release is **0.2.1**, covering
[IC Auth #9](https://github.com/dragginzgame/ic-auth/issues/9) and qualification of
the incoming native Host/Testkit selections. Auth library APIs, signed bytes,
feature surfaces and package versions remain unchanged at 0.2.0.

The CI workflow now selects `push.branches: [main]`, unchanged pull-request
activity and `workflow_dispatch`. An atomic main/release-tag push therefore
selects one automatic matrix for its source; feature branches use PR validation
or explicit dispatch. Distinct main commits retain their own complete runs.
No path filter, shared concurrency key or cancellation was introduced. All
workflow content except `on` is identical before/after, proven by sorted JSON
comparison: three hosts, browser/MSRV/setup/cache preparation, complete gate,
upload/download byte qualification and failure retention are preserved.
The release/publication scripts retain exact annotated-tag, pushed-tag and
validation-receipt admission. They previously had no CI tag-specific caller.

Actionlint, actual event/matrix assertions and case review pass. The
[developer instructions](../development.md#validation) document explicit dispatch
for supported branch/tag refs, its default-branch/selected-trigger requirement,
exact `headSha` acceptance and historical-tag limitations. No remote run was
triggered or cancelled. Actual native event qualification still requires
maintainer delivery; IC Auth #9 remains open. Review evidence is retained in
`target/continuation-0.2.1/{actionlint.log,event-check.log,event-review.md,ci-before.json,ci-after.json,jobs-before.json,jobs-after.json}`.

Concurrent owner work selected Host 0.9.2 and Testkit 0.27 in the manifest.
The lock initially retained Testkit 0.26.0, then another process completed its
0.27.0 resolution before the targeted Cargo update (which changed no selection).
Those incoming entries and tokio-util 0.7.20 were preserved. Official index entries
for Testkit 0.27.0/Host FS 0.9.2 are unyanked and match locked checksums.
Explicit preparation through the canonical installer and owner CLI succeeded;
older slots and server evidence remain retained. Testkit still admits PocketIC
16.1.0. No startup-error variant adapter was needed: the local runner already
consumes the bounded owner startup result without variant matching.

The current selected graph passes five native file-helper cases, both real
certification/composed-root/protected-upgrade/fresh-key signed-ingress scenarios,
Rust 1.88 application checks including tests, and warning-denied Clippy of both
native applications. Logs are `host-tests.log`, `qualification.log`,
`native-msrv.log` and `native-clippy.log` under the same evidence directory.
The real qualification state is retained beneath `target/portable-fixtures/`
at the exact path printed in `qualification.log`. Native startup ran with local
loopback networking admitted. The selected manifest/lock hashes in
`selected-inputs.sha256` still match after qualification. This is Linux acceptance,
not current native macOS, browser adapter, Canic or wallet-service acceptance.
Caller fixtures pass on Bash 5 and genuine Bash 3.2, including the actual unchanged
CI archive step. ShellCheck, formatting, snapshot/declaration/link guards and diff
whitespace pass. Aggregate checks and real offline CLI admission are recorded
alongside these logs.

The previously aligned release/consumer documentation remains part of the local
batch. No package version mutation, full local CI, commit, push, release,
publication or sibling source edit was performed. Canic continues independently
under its existing tracker; no wallet endpoint is introduced.

## Released 0.2.0 and consumer handoff

The maintainer delivered 0.2.0. Both official Cargo sparse-index records are
unyanked and match `.git/publication-state/0.2.0.json` for the exact source above
and annotated tag object `db9d1b2e42466136c870be1d989bb3e009b10be8`.
The crates.io API refused HTTP 403, so verification used the official index.
Exact-source [main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37923280901)
and [tag CI](https://github.com/dragginzgame/ic-auth/actions/runs/37923280939)
are queued at inspection. Keep IC Auth #7/#8 open pending their matching native
acceptance; publication does not establish that qualification. Evidence is in
`target/continuation-0.2.0-live/`.

The next adoption step remains Canic's runtime token verifier under
[Canic #491](https://github.com/dragginzgame/canic/issues/491). Its owner recorded
202 passing focused encoding, DTO, configuration and facade cases for the initial
0.1.14 slice, with immutable input snapshots. The current read-only review finds
Canic still at committed `c4c046f947b2b28f4342cbf6efe9221ba1ed5f70` with changing
uncommitted work: Auth 0.1.14 and Testkit 0.26.0 are selected. The earlier
Testkit catalog/lock mismatch is resolved at this inspection; the older acceptance
cannot be relabelled as qualification of the new graph. Canonical hashing and
shared proof declarations are adopted in source, but `AuthOps::verify_token`
still takes the local positive-cache/callback verification routes. Delegate those
callers to `ic_auth::token::verify_token` with protected caller, clock, fleet/role,
scope ceiling, key/network policy and finite limits. Qualify live authority changes
and retain typed endpoint errors/metrics before retiring the duplicate engine.
Durable session transactions and composed certification stores follow separately.
The published library already supplies this verifier; no new IC Auth endpoint,
wallet provider or library release is needed to begin that consumer work.

During the preceding release-status review, an incoming lock edit advanced
`tokio-util` 0.7.19 to 0.7.20; it was then preserved without compilation. The
current graph qualification is recorded above and is separate from 0.2.0 release
acceptance. No sibling
source, manifest version, release effect or compilation was performed by this
release-status review. Publication status and consumer documentation were aligned;
local links, snapshot integrity and diff whitespace are the relevant checks.

## Earlier hard-cut preparation (released)

The following records preparation before maintainer delivery; pending and
uncommitted wording in this section is historical.

The maintainer requested the latest Shared Tooling and its hard cut. The reviewed
snapshot now contains 77 canonical files from Shared Tooling 0.2.0 main revision
`8140e3dd1b44409d682c721889ab702f438c6a17`. Its five-tool IC bundle no longer
owns PocketIC. IC Auth removes `ci/ic-auth-tools.tsv`,
`scripts/ci/check-pocketic-alignment.sh` and
`scripts/ci/check-pocketic-binary.sh`, including the former alignment checker's
`usage()` function. Testkit owns server selection, asset authentication and
offline admission; there is no replacement consumer server catalog or fallback.

`make install-tools` explicitly prepares the locked `ic-testkit-server` through
the shared Rust installer and then invokes owner setup.
`make install-testkit-tools` prepares that capability alone;
`make testkit-tools-check` checks the selected executable receipt and consumes
Testkit's admitted absolute server path without installation. Qualification uses
only that path. The caller rejects a Testkit lock selection changed during its
effects. CI invokes the same setup/check surface on Linux and both macOS
architectures and archives retained Testkit setup evidence through the shared
archiver and existing failure collector.

The complete pending batch is now **0.2.0**, because this developer setup contract
is breaking. It carries the compatible proof-transport qualification below;
the manifests remain 0.1.14 and no package version or release effect was executed.
The current shared snapshot also adopts checkout-local formatter discovery and
single-document dependency exception admission. Native owner/consumer acceptance
must remain distinct: Testkit 0.25.4's exact-source three-host provisioning and
managed launch passed in
[owner CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37901828971);
new IC Auth native macOS acceptance still requires its own exact-source CI.

Another process selected Host Tooling 0.9.0 and IC Testkit 0.26.0 in the root
manifest/lock during setup. Those incoming changes were preserved. The initial
0.25.5 CLI setup completed, but qualification refused its receipt after the lock
selected 0.26.0. Explicit preparation of the matching 0.26.0 CLI then succeeded.
The new five-tool bundle is active locally; the previous six-tool bundle
`ic-set.M4eBQ5`, its original pins/receipt and every receipt-covered file are
unchanged. Older Testkit executable slots remain retained. Both selected clients
continue using PocketIC 16.1.0, admitted by the owner's store.

Focused caller checks pass on Bash 5 and genuine Bash 3.2: missing/ambiguous/
non-registry CLI selection, installation/setup/admission failures, concurrent
selection changes, qualification-path substitution and actual CI archive bytes.
Real setup and offline admission return the same path under both shells. Aggregate tool
checking and failure-evidence qualification pass. The canonical five-tool IC
installer and dependency-pinning fixtures pass; formatter-hook adoption,
ShellCheck and Actionlint pass. Evidence is retained under
`target/shared-hard-cut/`. The first real lifecycle invocation with the current
graph built successfully but the sandbox refused the local server bind;
`qualification-final.log` retains that failure. With loopback networking admitted,
both real certification/composition/upgrade and fresh-key signed-ingress scenarios
pass in `qualification-native.log`; owner runtime state is retained under
`target/portable-fixtures/qualification.HVJfZN`.

The preserved Host 0.9.0/Testkit 0.26.0 graph passes all five native file-helper
tests, strict Clippy of both native applications and Rust 1.88 checks including
their tests. See `incoming-host-tests.log`, `incoming-native-clippy.log` and
`incoming-native-msrv.log` in the same evidence directory. Formatting, dependency
boundaries/pins, all 77 snapshot files, 251 local links across 54 documents and
diff whitespace pass in `guards-final.log`. The selected manifest/lock hashes
are retained in `selected-inputs.sha256`; auth libraries acquire no native Host,
Testkit or direct PocketIC dependency.

This implements the consumer handoff in
[IC Auth #7](https://github.com/dragginzgame/ic-auth/issues/7), coordinated with
[Shared Tooling #76](https://github.com/dragginzgame/shared-tooling/issues/76).
The [consumer evidence update](https://github.com/dragginzgame/ic-auth/issues/7#issuecomment-6079297692)
keeps delivery and native macOS acceptance outstanding at that existing tracker.
Canic can adopt published IC Auth 0.1.14 now; this tooling cut does not require a
new authentication API or wallet endpoint. No sibling source edits, complete
local CI, commit, push or publication are authorized or claimed.

The maintainer subsequently committed the hard-cut implementation at
`f6e17b08be17cbb39a41fb526079b7a710bbd405` with the manifest still 0.1.14.
Their complete-gate invocation exposed a missed consumer release-fixture overlay:
`test-release-tools` still tried copying the retired `ci/ic-auth-tools.tsv`.
The local fix removes that entry and exports the current Testkit setup/check
and caller-test scripts alongside the qualification wrapper. The shared matrix
already comes from the verified snapshot export; no replacement matrix is added.
`make test-release-tools` now passes all local release/publication scenarios;
ShellCheck and diff whitespace pass. The failed fixture `release-tools.ZUIs3S`
and original validation logs remain retained. Successful evidence is in
`target/shared-hard-cut/release-fixture-fix/release-tools.log` and
`target/portable-fixtures/release-tools.o1qRdM`. A complete-gate rerun and native
macOS acceptance are still outstanding; this repair did not commit or release.

## Earlier compatible continuation (carried into 0.2.0)

The maintainer delivered 0.1.14 and requested more work to accelerate Canic
adoption. Both live packages are unyanked and their registry checksums match
`.git/publication-state/0.1.14.json` for the exact source above and annotated tag
object `083af85df9c6b3fcfe8fe1c2f081e2069060d990`. Exact-source
[main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37914751465) and
[tag CI](https://github.com/dragginzgame/ic-auth/actions/runs/37914751714) were
queued at review. Publication does not establish native acceptance.

The read-only review initially found Canic at the previously reviewed commit
`c4c046f947b2b28f4342cbf6efe9221ba1ed5f70` with unrelated dirty work and no root/Core
IC Auth dependency. During inspection, owner work began selecting published
0.1.14 in both manifests, re-exporting metadata and proof atoms, and delegating
canonical encoding through checked projections. Its registry/key-policy hashes
remain host-owned. These are changing uncommitted sources, not a consumer build
or completed A5 acceptance. No Canic source was edited or compiled here.

The [adoption contract](../canic-adoption.md#current-canic-adapter-boundary) now
maps that boundary and the next complete-verifier callers. The positive-cache
path and callback verifier still remain in the inspected source; next consumer
work should converge on `token::verify_token`, retain protected network/key-name
admission, typed endpoint errors and finite limits, and define an independent
local scope ceiling. Do not infer that ceiling or key enrollment from submitted
claims. Durable session adoption and certification composition remain separate
host obligations in [Canic #491](https://github.com/dragginzgame/canic/issues/491).
The [consumer progress handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6078838724)
records the observed source change and exact next callers; the
[qualification update](https://github.com/dragginzgame/ic-auth/issues/8#issuecomment-6078839767)
records delivery separately from pending fixture/native acceptance.
The current published library is sufficient to start this work; wallet login
and publication of the next qualification batch are not prerequisites.

The compatible batch initially selected 0.1.15 and is now carried into 0.2.0.
It extends
[IC Auth #8](https://github.com/dragginzgame/ic-auth/issues/8) with independent
fixtures for complete retrieved tokens, delegation certificates and chain-key
batch proofs at the immutable Canic DTO source above. Three additional tests
preserve exact bytes in both Candid directions, including both Merkle sibling
directions, seed bindings, opaque root/issuer signatures, nested key paths and
optional extensions. Malformed roles/identities in certificate/leaf material
reject within complete proof/token envelopes. Synthetic signature/key bytes
qualify transport only, not cryptographic acceptance or the consumer adapter.

All eleven wire tests pass on Rust 1.99 and 1.88, strict package Clippy passes,
and the test target checks for Wasm on 1.88. Evidence is retained in
`target/continuation-0.1.15/{types-final,types-clippy,types-msrv,types-wasm-msrv}.log`.
The initial module-discovery failure is retained in `types.log`; ordinary module
discovery under `tests/proofs/mod.rs` fixes it. The immutable DTO comparison
passed before concurrent consumer edits started. Public APIs, signed encoding,
manifests, browser selections and Shared Tooling snapshot were unchanged by this
qualification batch. A separate incoming lock update was recorded below, before
the current hard cut and newer incoming selections above.
Formatting, dependency boundaries/pins, all 79 snapshot files, 252 local links
across 54 documents and diff whitespace pass; the guards log retains the checks.
No full local CI, new canister lifecycle run, commit, push or publication occurred.

During final checks, another process selected all four Host packages at 0.8.10
in the lockfile. This incoming work was preserved; no package selection was
changed by this continuation. Registry checksums match the lock, and the four
cached library source trees are byte-identical to 0.8.9. Five native file-helper
tests, strict tooling Clippy and Rust 1.88 checks of both native applications
including tests pass. Evidence is retained in
`target/continuation-0.1.15/incoming-host-{tests,clippy,msrv}.log` and
`incoming-host-lock.patch`. Testkit 0.25.5 and its PocketIC 16.1.0 client are
already in released 0.1.14 and remain unchanged. The final lock checksum is
retained in `final-locked-input.sha256`; auth library dependency paths are
unchanged. This does not establish native macOS, new runtime or complete-gate
acceptance. Host's release changed its own Shared Tooling under
[Host #37](https://github.com/dragginzgame/ic-host-tooling/issues/37); it does not
update IC Auth's reviewed snapshot implicitly.

## Earlier 0.1.14 preparation (released)

The following records work before maintainer delivery; its draft and uncommitted
wording is historical.

The maintainer delivered 0.1.13. Both live packages are unyanked and their
registry checksums match `.git/publication-state/0.1.13.json` for exact source
`4421f991065a8bc9fc21d3d70776943b493dc740` and annotated tag object
`5a040475c50f444abd37b6f42699a0b4c0fbdf09`. The exact-source
[main CI](https://github.com/dragginzgame/ic-auth/actions/runs/37910262133) and
[tag CI](https://github.com/dragginzgame/ic-auth/actions/runs/37910261591) were
running Linux with both macOS jobs queued at initial review. The subsequent
exact-source main inspection passes its complete Linux gate; both macOS jobs
remain queued. Publication does not establish native acceptance.

The maintainer clarified that Canic should reuse as many applicable shared
libraries as possible to reduce code duplication. The published Rust libraries
are the first confirmed duplicate engine in this continuation's review.
The [Canic adoption contract](../canic-adoption.md) maps existing capability
families to published features/APIs and identifies protected authority, durable
transactions, retained-state disposition and certification composition owned by
the Canic adapter. A read-only source/declaration review at committed Canic
`c4c046f947b2b28f4342cbf6efe9221ba1ed5f70` confirms no root/Core IC Auth dependency
and still-active generic engines; unrelated dirty work is preserved. Source
review is not a consumer build, stable-host qualification or deployment claim.
Implementation and acceptance remain in
[the Canic #491 handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6078139476);
no sibling edit is requested or performed. Wallet login is independent of local
Rust adoption.

The wider source/declaration review and clarified owner map are recorded in
[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6078242947).
It records existing shared delegation separately from remaining adoption and
contract differences. The two Testkit URL-selection callers are traced in
[Canic #501](https://github.com/dragginzgame/canic/issues/501#issuecomment-6078229079);
[Canic #502](https://github.com/dragginzgame/canic/issues/502) owns the IC Jobs
assessment. The review used the adopted flow-convergence method, preserved dirty
owner work and ran no consumer builds. Library reuse must preserve one canonical
mechanism and the actual trust, atomicity and recovery contract; no code-size or
runtime savings are claimed from source inspection.

The maintainer then made Canic's IC Auth adoption the immediate priority. The
[priority handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6078380486)
selects published 0.1.13 for starting consumer work, independently of wallet
login, IC Jobs or 0.1.14 publication. The pending 0.1.14 batch now adds independent
wire fixtures for complete prepare requests, claims/prepare responses and
retrieval requests under
[IC Auth #8](https://github.com/dragginzgame/ic-auth/issues/8). The DTO source
files match committed Canic `c4c046f947b2b28f4342cbf6efe9221ba1ed5f70`; the
fixtures add no Canic dependency. Four additional tests cover both Candid
directions, exact bytes, request metadata, optional extension states, fixed
hashes/nonces, principals, unsigned boundaries and invalid nested identities/
roles. All eight type tests and strict package Clippy pass on Rust 1.99; the
eight wire tests pass on 1.88 and the test target checks for Wasm on 1.88.
Formatting, dependency boundaries/pins, snapshot/links and diff checks pass.
These fixtures are codec compatibility evidence, not actual Canic compilation,
proof authentication, generated endpoint or stable-host qualification. No broad
local CI, unrelated Rust suites or new canister lifecycle runs were needed.

README, agent publication status and developer release examples reflect the
delivered 0.1.13. The compatible pending version remains 0.1.14 for the maintained
adoption documentation and wire qualification; public library APIs, signed bytes,
manifests, lockfile and browser source are unchanged. Evidence is retained in
`target/continuation-0.1.14/`, including `types.log`, `types-clippy.log`,
`types-msrv.log`, `types-wasm-msrv.log` and `guards.log`.
No commit, push, publication or deployment was performed by this continuation.

## Earlier 0.1.13 preparation (released)

The following records work before maintainer delivery; its draft and uncommitted
wording is historical.

The maintainer delivered 0.1.12. Both live packages are unyanked, and their
registry checksums match the retained publication intent for the exact remote
source/tag. The [main release CI](https://github.com/dragginzgame/ic-auth/actions/runs/37903856227)
and [tag CI](https://github.com/dragginzgame/ic-auth/actions/runs/37903860241)
both pass their complete Linux gates, including the negative setup/check fixture
and exact artifact upload/download byte verification. Both macOS jobs remain
queued in each run. Delivery is distinct from native acceptance; the existing
qualification issues remain open until that evidence completes.

The current compatible draft is 0.1.13. Its reviewed Shared Tooling snapshot
selects `1af63d31942448a46ef42553285176b98dce9274` (0.1.36), verified against
remote main, with the same 79 canonical files and no sibling changes.
The initial 0.1.35 refresh at `be550afa57fe9e16872e5110b5cd69c24b4fa9e8`
introduced the canonical installer for consumer-selected registry
binaries/examples without replacing the existing formatter bundle.
Both fixed/selected installer fixtures passed on Bash 5
and genuine Bash 3.2 with substituted Cargo effects. Actual consumer tools-check
and negative failure-evidence collection pass on both shells. The real published
Testkit 0.25.4 CLI was installed in the versioned `debug` selection through that
canonical installer; offline receipt/byte admission returns the same CLI path
on both shells, and that CLI's server check returns the same previously qualified
server path. This qualifies that selection on Linux, not native macOS.

The adopted dependency-preparation rule already matches IC Auth's release
preflight: `cargo fetch --locked` runs after source/changelog admission and before
offline validation. No release-adapter change or new network switch was needed.
The local/mocked release/publication reconciliation fixture passes with the
selected snapshot. Evidence is retained in `target/continuation-0.1.13/`,
including `rust-tools-bash5.log`, `rust-tools-bash32.log`, `consumer-tools*.log`,
`testkit-selected-*` and `release-tools.log`. Snapshot/link checks, ShellCheck
and diff checks pass. That snapshot refresh changed no Rust manifests/lock,
public libraries or npm selections; no broad Rust/CI gate was run for it.

The subsequent 0.1.36 refresh adopts the canonical repair under
[Shared Tooling #65](https://github.com/dragginzgame/shared-tooling/issues/65):
both Cargo and local selection receipts require exactly one JSON document,
installation ancestors and the selection slot are checked again after Cargo
returns, and failed Cargo status is preserved with the retained attempt.
The exact-source installer fixtures pass on Bash 5 and genuine Bash 3.2,
covering conflicting receipt streams, redirected ancestors, original failed
status, prior installation preservation and locking with substituted Cargo.
Actual consumer tools-check and negative failure-evidence collection also pass
on both shells. The existing real Testkit debug CLI passes the updated offline
checks on both shells with the same path; before/after receipts and executable
checksums are unchanged. This is Linux artifact re-admission, not a new build
or native macOS qualification. ShellCheck, snapshot/link and diff checks pass;
evidence is retained in `target/shared-review-0.1.13/`. The exact-source
[Shared Tooling CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37908179516)
has passing lint/security, Linux portable regression in progress and both macOS
jobs queued at review. This refresh changes neither the incoming Host lock
selection nor Rust/browser sources, and does not rerun unrelated Rust or release
gates. The compatible pending version remains 0.1.13.

The subsequent Host review preserves the incoming lock selection of all four
published `ic-host-*` 0.8.9 packages. Their registry checksums and archive source
identities match `0464db5146be910a0f078447831fa2807072c75a`; their library sources
are byte-identical to 0.8.8. The release changes Host's own dependency preparation
under [Host #36](https://github.com/dragginzgame/ic-host-tooling/issues/36), already
implemented here. There is no new library API to substitute, and the direct
`ic-host-fs` minimum remains 0.8.7 for bounded no-follow hashing. Testkit 0.25.4
and its selected PocketIC 16.1.0 server are unchanged.

Explicit locked dependency preparation, five file-helper tests, dependency-pin
and library-boundary checks, strict native Clippy and Rust 1.88 checks for both
native applications pass with that lock. Both real certification/composed-root,
protected upgrade and fresh-key ingress scenarios pass, as does the local/mocked
release/publication fixture. Evidence is retained in
`target/host-review-0.1.13/`. The exact-source
[Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37905479815)
passes Linux and MSRV; both macOS jobs remain queued at review. These focused
checks do not establish native macOS acceptance or remote CI for the uncommitted
IC Auth batch. Public Rust libraries and browser selections remain unchanged.

[IC Auth #7](https://github.com/dragginzgame/ic-auth/issues/7) owns consumer
adoption of Testkit's published setup/check interface. An isolated probe of
published Testkit 0.25.4 `085756dd0e0e2304de7e4a0b6b887201918646b6` built its
actual CLI from the package payload and its own locked inputs. An empty workspace
table isolated that fixture from this repository; source code was unchanged.
Explicit official-asset setup and offline check with `PATH=/nonexistent` return
the same admitted executable path. Passing only that check-returned path to
IC Auth's real certification/composed-root, protected upgrade and fresh-key
ingress scenarios passes both tests on Linux. Probe evidence and retained server
state are under `target/continuation-0.1.13/testkit-owner/` and the named
`target/portable-fixtures/testkit-owner.*` directory.

The probe preserves the selected snapshot, consumer pins and existing IC bundle
link, verified before/after. No new Make command or installation route is
advertised. Owner macOS qualification and the reviewed Shared Tooling six-tool
retirement remain prerequisites in
[Testkit #38](https://github.com/dragginzgame/ic-testkit/issues/38) and
[Shared Tooling #76](https://github.com/dragginzgame/shared-tooling/issues/76).
Sibling repositories remain read-only; their coordination belongs in those
issues. No further package version change, commit, push or publication occurred.

## Earlier 0.1.12 qualification (released)

The following records preparation before the maintainer delivered 0.1.12;
its pending-version and uncommitted wording is historical.

The maintainer authorized work on [the MSRV wrapper](https://github.com/dragginzgame/ic-auth/issues/4),
[setup failure evidence](https://github.com/dragginzgame/ic-auth/issues/2),
[client adoption](https://github.com/dragginzgame/ic-auth/issues/3) and
[optional fleet tooling](https://github.com/dragginzgame/ic-auth/issues/5).
The final continuation also covers [streaming file identities](https://github.com/dragginzgame/ic-auth/issues/6)
and the reviewed Shared Tooling 0.1.34 refresh.
GitHub remains their status tracker. The prior release/publication tracker was
closed after verifying the delivered 0.1.11 source/tag and both live package
checksums against publication intent. Linux's complete release CI passes; both
macOS architectures fail at `target_args[@]: unbound variable` in `check-msrv.sh`.

The MSRV wrapper now expands its empty native-target array safely on Bash 3.2,
retaining strict shell settings, explicit Rust 1.88.0 and all existing isolated
package/native/Wasm/feature checks. The failure is reproduced on genuine Linux
Bash 3.2.57, and the repaired full package matrix passes under that shell.
The change affects argument routing, not compiler floors or dependencies.

The CI setup caller records real aggregate install/check output in the collector's
selected `target/rust-tools-install.log` and `rust-tools-check.log`. It preserves
the aggregate failure even if logging also fails, and rejects a logging failure
after a successful aggregate. The focused fixture copies the actual wrapper,
Makefile and installer, substitutes only Cargo's installation effect for the
main negative cases, checks offline refusal without a second dispatch, and runs
the adopted collector block directly. The original logs/build bytes survive the
archive; unrelated Rust cache data is excluded. Both Bash 5 and genuine Bash 3.2
pass. CI uploads through the ordinary action and downloads the exact artifact ID
for checksum verification on all three hosts; those hosted results remain pending.

The snapshot now selects clean committed Shared Tooling
`3d33cd250fcae7dbe5cabe44b2abd6b2c91a1822` (0.1.34), verified against remote main.
The unused `cloc-tooling.pl` and its manifest record were explicitly retired;
the remaining 79 files retain canonical hashes/modes. The upstream optional
target reports its owner instead of invoking a sibling. Real local `make cloc`
and offline `make tools-check` pass, and the omitted fleet report gives the
expected diagnostic. The removed reporter measures 279 code LOC with cloc 2.10.
Shared Tooling retains the canonical fleet implementation and tests.
The 0.1.34 alignment checker preserves literal paths under `CDPATH` and stops
before Cargo if the selected directory disappears. Its canonical fixture passes
on Bash 5 and genuine Bash 3.2; the adopted checker verifies this consumer's
actual locked pairing with and without `CDPATH` on those shells.

The incoming lock subsequently advanced to Host 0.8.8 and Testkit 0.25.4; those
selections are preserved. The hash-only helper now uses Host's bounded streaming
no-follow API, introduced in 0.8.7, instead of buffering the complete artifact.
Its CLI argument validation, exact digest stdout and rejection behavior remain
intact. The native utility and root no longer declare the unused direct
`ic-host-artifacts` dependency; it remains owned transitively by Host/Testkit.
The root selects a minimum `ic-host-fs` 0.8.7 for the new API. Five file-helper
tests include empty-file/zero-limit admission and nonempty zero-limit rejection.
The final selected stack passes strict native Clippy, the complete Bash 3.2
Rust 1.88 package/feature matrix and both real certification/upgrade/ingress
scenarios. Testkit 0.25.4's newly published setup/check CLI remains a separate
consumer adoption: this batch retains the existing server installer and
consumer-owned PocketIC 16.1.0 pins.

The maintainer selected issue-only coordination for other repositories: keep
siblings read-only and record findings/adoption requirements in their owning
GitHub issues. Production client adoption remains consumer-owned under
[Toko #1803](https://github.com/dragginzgame/toko/issues/1803) and
[Canic #491](https://github.com/dragginzgame/canic/issues/491), linked from
[IC Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
Read-only review found Toko unchanged at `44d4e2c6d41e3575b2503e79ffa369129ff3ecf2`
apart from its unrelated dirty lock; its TTL-only issuance wrappers do not expose
the client's exact request/reconciliation contract. Canic remains at committed
`c4c046f947b2b28f4342cbf6efe9221ba1ed5f70` with unrelated dirty work preserved. No sibling source was changed
and no production adapter or deployed adoption is claimed. No sibling-edit
approval is pending; adoption contracts and qualification belong in those issues.

Snapshot/dependency/link/format checks, ShellCheck and Actionlint pass. The positive
setup-check wrapper preserves exact captured output. The canonical shared Make
fixture passes on Bash 5 and 3.2 for minimal and explicit fleet selections,
including spaced paths. The later Host/Testkit qualification and direct native
dependency cleanup are recorded above; browser source, npm selections and both
authentication libraries remain unchanged. No broad local CI or unrelated
authentication suites were run.

Evidence is retained under `target/issue-work-0.1.12/` and its uniquely named
`target/portable-fixtures/` subdirectories. The compatible draft is 0.1.12;
Rust manifests and the lock remain 0.1.11. No commit, push, release or publication
was performed. The final release/publication fixture passes with the actual
streaming hash helper and current lock, using local bare remotes and mocked
registry/upload effects. Final evidence includes `final-focused.log`,
`final-msrv-bash32.log`, `final-native-clippy.log`,
`final-native-qualification.log` and `final-release-tools.log` in that directory.
The implementation batch is ready to commit and push for native CI. Hosted
macOS, exact artifact upload/download qualification and the complete CI gate
remain required before treating 0.1.12 as release/publication-qualified.

## Earlier 0.1.11 qualification (released)

The following records local preparation before the maintainer delivered 0.1.11;
its pending-version and uncommitted wording is historical.

The maintainer requested review of new Host Tooling and continuation of 0.1.11.
The incoming lock already selected all four Host packages at 0.8.5, replacing
the committed 0.8.4 selections. That change is preserved. Live registry metadata,
cached package archive digests and selected lock checksums agree for every package;
all are unyanked and declare Rust 1.88.0. Published package source and remote
main/tag resolve to `1cad3253096b6eb67be5187209e7fb606593c501`.
The clean Host sibling was read-only. The 0.8.4-to-0.8.5 crate sources are identical;
the release adopts the IC installer reuse fix already selected here. No further
native API substitution is needed: the consumer already delegates bounded
no-follow reads, SHA-256 and durable private publication to Host.

Shared Tooling subsequently advanced to committed
`635a39a9dd5f8d021fa9c9196b591e00521a7e02` (0.1.32), verified against remote main.
A new clean isolated clone supplied the canonical refresh of the same 80 files;
the sibling's further dirty dashboard/setup work was preserved and excluded.
The selected additions document PocketIC's intended Testkit ownership and correct
Cargo-install qualification guidance. No new workflow or installer was selected.
The consumer workflow has no concurrency cancellation, so it already preserves
each push's native jobs. The complete IC installer, consumer pins, receipts and
Testkit 0.25.3/PocketIC 16.1.0 pairing remain in place until the published replacement
setup/check contract is qualified. No direct PocketIC dependency was added.

Two additional storage rejection tests prove that unavailable strict durability
does not admit a relaxed fallback or alter retained intent, and that exhausting
nat64 revisions cannot wrap, clear an existing intent or allocate another key.
All 35 Node tests and both Rust/TypeScript Candid directions pass on Host 0.8.5.
The previous nine native Chromium observations remain evidence for the unchanged
storage implementation; the new injected cases are Node qualification.
Four native file tests, strict tooling/runner Clippy and actual Rust 1.88.0
tooling/runner checks pass. Both real IC Testkit scenarios pass with Host 0.8.5,
covering certification/composed roots, protected upgrades and delegated ingress.
Snapshot integrity, dependency declarations/inheritance, local links, locked
metadata, library graph boundaries and formatting pass. The release/recovery and
publication reconciliation fixture passes with the final selected snapshot and
Host lock, using local/mocked effects only. Full CI and native macOS remain pending.

Current evidence is retained under `target/continuation-0.1.11/host-0.8.5/`.
Rust manifests remain 0.1.10 and the compatible draft remains 0.1.11. No existing
function, method or type was removed. No commit, push, release, publication or
sibling source change was performed. Real issuer adoption, wallet login and native
macOS qualification remain separate from this local batch.

## Earlier preparation of the current 0.1.11 batch

The following records the earlier 0.1.31 refresh and initial storage qualification;
the current source selection and additional tests are recorded above.

The maintainer pushed/released 0.1.10 and authorized continued local development
and current Shared Tooling adoption. Remote main/tag resolve to the exact release
above. Both unyanked 0.1.10 registry checksums match retained publication intent:
`51162f12aeb7cf52a2249475d8c423e1dc9e33e18e4727ce53687f3696975c04`
(protocol types) and
`f962f1ad928c5aabff78891a09c883847503661d6a41c4b29c520b2e16b3cc98`
(auth). Rust manifest/lock versions remain 0.1.10. The next compatible draft is
0.1.11: opt-in private storage and installer reuse are additive, with no existing
Rust API or signed-byte change. No commit, push, release or publication was made.

Shared Tooling remote main and committed sibling HEAD are
`9af82393c620e486578febed74a648523725c234` (0.1.31). The sibling has further dirty
Cargo-install qualification changes; they were preserved and excluded. A clean
isolated clone at that exact commit supplied the canonical 80-file refresh.
The selected baseline adds fleet/report ownership guidance and the IC installer
compares complete validated records across all hosts instead of raw pin bytes.
No fleet dashboard, new Cargo installer or central qualification workflow was
added to consumer CI. The existing tool/version/checksum selections are unchanged.

The old installer rejects the same selected records with reversed rows and new
comments. The adopted installer passes offline against the existing real bundle,
with exact stdout, unchanged active link and unchanged installed-pin digest. The
clean upstream IC installer fixture also passes with mocked downloads/archives;
that evidence is distinct from real bundle reuse and native macOS qualification.
The current GitHub description still matches the independent authentication
library scope; no description write was necessary.

The private browser package now offers opt-in `IndexedDbTokenStore`, with an
explicit factory, dedicated owner-selected database and frozen bounded profile.
It atomically reserves capacity and revisions with the record CAS, waits for
strict transaction commit, retains tombstones/uncertain intent, validates exact
schema and refuses malformed state without deleting it. Closing/version-change
only closes the connection. No storage migration, implicit reset or alternate
reader was introduced. `fake-indexeddb 6.2.5` is test-only, prepared explicitly with
lifecycle scripts disabled; no existing npm selections or Rust dependencies moved.
See [browser storage](../browser-client.md) for the contract and owner obligations.

Native Linux Chromium 153.0.8010.12 passes the actual IndexedDB fixture, including
independent-connection CAS, detached bytes, reopening/exact bigint intent,
tombstone/capacity admission, profile/corrupt-state refusal and late transaction
abort. Node tests additionally exercise the client recovering a lost prepare
reply after reopening and transaction timeout/rollback. These are storage/lifecycle
checks, not a production issuer adapter, browser proof verifier, live consumer
adoption, device-crash recovery or native macOS/browser-engine qualification.
The standard DOM-dump attempt stalled, and a virtual-time attempt observed an
incomplete page; retained artifacts distinguish them from the passing real-time
CDP observation. The isolated test browser and local server were closed.

Focused checks pass: 33 Node lifecycle/codec/storage/example tests, both Candid
directions, and nine native Chromium storage observations against the final
compiled source. Snapshot integrity, tracked npm declarations/Cargo inheritance,
local links, formatting, locked metadata, auth graph boundaries and ShellCheck
pass. The local/mocked release/publication fixture also passes with the adopted
snapshot and complete working client tree. Source/input digests and the observed
local headless-browser binary digest use the existing Host utility. Existing
Rust sources, dependency selections and declared compiler floors are unchanged;
no broad local CI or unrelated Rust feature suites were run. Remote CI for the
uncommitted batch and native macOS/browser-engine qualification remain pending.

Evidence is retained under `target/continuation-0.1.11/`, with native fixture source,
results, selected tool/source identities and failed attempts. No existing function,
method or type was removed. Neither Shared Tooling nor Canic/Toko source was edited.

## Earlier browser client batch (released in 0.1.10)

The following records preparation before the maintainer's 0.1.10 release. Its
pending-version and untracked-file wording is historical.

The maintainer pushed/released 0.1.9, then authorized continuation on the browser
client in [IC Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
Live 0.1.9 registry checksums match retained publication intent:
`de44ab5b877f3df89674d273b3e38b2e398e25cc60ab5bf3caa61d8cc436b7ff`
(protocol types) and
`fa9b48d98aa8232eb264eb94f2ff0238a922a786e314884fc3e52e1cd978f101`
(auth). The Rust manifest remains 0.1.9. The next compatible draft is 0.1.10:
the private client/dev tooling is additive and existing Rust APIs and signed
bytes remain unchanged. No release, push or publication was performed here.

The private npm root uses Node 24.21.0/npm 12.2.0 and locked ICP SDK core 6.1.0,
bindgen 0.4.1 and TypeScript 7.0.2. Runtime contracts come from the Rust DTO owner;
no client service endpoints or hand-written Candid DTO copies were introduced.
A Rust development-only printer feature exports the contracts without changing
normal protocol dependency selections. Native Host packages remain outside the
browser and public protocol graphs. See [browser client](../browser-client.md)
for uncertainty, generation, cache, storage and consumer boundaries.

Explicit npm preparation used reviewed registry routing with lifecycle scripts
disabled. Subsequent checks were offline and preserved the lock. Local source
fixtures exercise login/renewal races, captured-generation invalidation,
unknown prepare/retrieval replies, typed TTL exhaustion, expiry/poll bounds,
cache isolation, material binding, storage admission and CAS behavior. The
Internet Identity example compiles with the actual SDK and checks authenticated
agent identity plus logout during pending login. These are transport/lifecycle
fixtures, not a live issuer adapter or browser cryptographic qualification.

The read-only Toko source review selected
`44d4e2c6d41e3575b2503e79ffa369129ff3ecf2` and preserved its dirty Cargo lock.
The adoption recipe names replacement boundaries but leaves user/shard/project
semantics in Toko and issuer authority in Canic. A real adapter must expose
qualified exact-ID reconciliation before old token helpers are removed.
Issue #3 remains open for that consumer work. Neither sibling was edited.

Focused validation passed: 23 Node lifecycle/codec/example tests and both Candid
directions; protocol type tests and the exporter on development Rust 1.99.0 and
minimum Rust 1.88.0; strict protocol Clippy; auth dependency boundaries, locked
metadata, formatting, ShellCheck, Actionlint, snapshot integrity and local links.
The release/publication fixture includes the client source/lock without build
outputs and passes its local/mocked release and uncertainty checks. These fixture
Make override warnings are deliberate substitute gate recipes, not production
Makefile duplicates. Evidence is retained under `target/browser-client/`, with
failed first attempts alongside the final logs. Generated-contract directories
record exact input hashes using the existing Host utility and tool versions.

The canonical dependency declaration check passes in an isolated source fixture
with the new npm manifest/lock tracked; the real index was unchanged. The real
checkout's npm check correctly requires those currently new files to be tracked
at commit. The Rust lock is unchanged. No broad local CI or native macOS run is
claimed; configured CI now prepares and checks the client on all three hosts.
No existing function, method or type was removed in this additive batch.

## Earlier Host 0.8.3 review and wrapper cleanup (released in 0.1.9)

The following records preparation against 0.1.8, before the maintainer's 0.1.9
release. Its pending-version wording is historical.

All four Host packages were already selected at 0.8.3 in the incoming dirty
lockfile. That update and the prior Shared Tooling batch were preserved.
Live registry metadata identifies 0.8.3 as the latest unyanked version of each;
all four checksums match the selected lock. Published package source and observed
remote main are `67d031222073f23ad437b45053156e229f86a016`. The clean sibling was
read-only. Evidence is retained in `target/host-0.8.3-review/`.

The committed 0.8.2-to-0.8.3 changes add live output observation to owned-child
communication and expose regular lock-file admission without acquiring a lock.
This consumer has no private pipe-reader loop or descriptor-opening lock
implementation to retire. IC Testkit owns server startup and retained output;
Shared Tooling owns release process orchestration. Directory locks and registry
uncertainty policy remain with the shell release/publication owner. The current
native CLI already delegates bounded no-follow reads, digest computation and
durable private publication to Host. Its explicit permissions and create-only
semantics are retained; the link-following `hash_file` API is not an equivalent
substitute. No new dependency, adapter, function or type was needed or removed.

A concrete wrapper defect was reproduced: inherited `CDPATH=.` emitted the
checkout path before the digest (`cdpath-cached-before.stdout`) and before the
NUL-delimited release metadata (`cdpath-metadata-before.nul`). Consumer wrapper,
release metadata, publication and fixture entrypoints now resolve an absolute
script path and use physical `cd`, preserving exact output. The release fixture
checks exact metadata bytes and keeps `CDPATH` set throughout its local release
and mocked publication flows. No authentication API or signed bytes changed.

An initial offline attempt lacked the selected Host cache entries and failed
without resolution changes (`cdpath-before.stderr`). Explicit `cargo fetch
--locked` prepared those four entries (`cache-preparation.log`); subsequent Rust
checks stayed offline. Four native file tests pass (`native-file-tests.log`),
including bounded hashing, no-follow admission, private replacement and saved
intent preservation. Exact wrapper stdout and release metadata pass after the
fix (`cdpath-after.stdout`, `cdpath-metadata-after.nul`). ShellCheck passes.

Both real IC Testkit scenarios pass with the selected 0.25.2 client and 16.1.0
server on Host 0.8.3 (`final-testkit-qualification.log`, retained state
`target/portable-fixtures/qualification.3kaA17`). The first run was refused by
the sandbox at loopback binding before either scenario could execute; its log
and `qualification.RK3eWS` state are retained separately. The successful run used
the same verified server outside that restriction. This exercises certification,
composed roots, protected upgrades and actual delegated ingress, not wallet login.

Focused native tooling/runner checks pass on actual Rust 1.88.0
(`native-msrv.log`, compiler identities in `compiler-identities.log`), as does
strict native Clippy (`native-clippy.log`). This checks the changed Host graph;
the earlier isolated public-package/native/Wasm minimum qualification remains
its own evidence. Consumer release/recovery/publication fixtures pass with
inherited `CDPATH` (`consumer-release-fixtures.log`); their local/mocked effects
are not live release or upload evidence. Snapshot, pinning/inheritance, links,
metadata, library boundaries and the diff pass (`final-governance.log`).
Final source/lock/snapshot/Wasm identities and the working diff are retained here.

The compatible pending batch remains 0.1.9; manifests remain 0.1.8.
No commit, push, release, publication or sibling change was performed.

## Earlier Shared Tooling 0.1.29 batch (released in 0.1.9)

The following records preparation before the maintainer's 0.1.9 release; its
pending-version wording is historical.

Observed remote Shared Tooling main is
`1a54fb625d6e47efa64c4384808ecbc87be84e7e` (0.1.29). A clean isolated checkout
exported its committed bytes; the sibling's untracked Cargo-install helper was
excluded and the sibling remained read-only. The canonical refresh passed for
80 files, explicitly adding `scripts/ci/check-release-source.sh` to the existing
selection. Evidence is retained in `target/baseline-0.1.29/`.

The local release adapter now uses that helper. Ordinary initial admission
allows only pending `CHANGELOG.md` notes; interrupted preparation retains the
four exact metadata paths and receipt-bound byte checks. Refusals name all
staged, unstaged and untracked paths, quote unusual names and distinguish failed
Git observations from successfully observed dirty source. The runner identifies
an initial preflight refusal before this attempt starts validation/preparation.
No files or index bytes are repaired merely to pass admission.

The refreshed baseline restricts standing issue-writing permission to verified
`dragginzgame/*` destinations, including the maintenance prompt. Its optional npm
checks remain unselected: this repository has no frontend publication root.
The optional sibling issue dashboard was not added to the snapshot. No task,
maintenance agent or schedule was activated.

The shared server defaults now select the same reviewed 16.1.0 archives as
`ci/ic-auth-tools.tsv`. The consumer-owned selection remains explicit because
its client/server pairing is qualified with IC Testkit. No Cargo dependency,
compiler floor, authentication API, signed bytes or runtime pairing changed.

Both unyanked 0.1.8 registry checksums match the retained publication receipt:
protocol `ef8d1f627a45eefbf5b3e831a61c4ee16b283976ae9bf155686fba8b64722300`,
auth `6e239af75a1eca605a0b9b0da602466de2f1f7685637213ad236d68de1bd1d2c`.
The remote annotated tag object is `4245aeec57a212e31d1c27e1077a401defd47d8b`
and dereferences to the release commit above. The repository description remains
consistent with implemented scope. Current status prose now reflects that release;
earlier batch records below retain their original evidence and limits.

Focused checks pass: canonical source-admission fixtures cover lock-only,
hidden staged, unstaged, untracked, unusual-name and rename cases, Git errors
and preservation (`shared-source-fixtures.log`). The adopted simulation-only
runner passes (`shared-release-runner.log`). Consumer fixtures additionally
confirm lock/staged diagnostics, refusal before the substitute gate, unchanged
source/index bytes, exact local tags/push, interrupted preparation and publication
reconciliation (`final-consumer-release-fixtures.log`, retained fixture
`target/portable-fixtures/release-tools.ZaZcZM`). Their mocked gate/transport and
local bare remote do not establish live delivery. Duplicate-recipe warnings are
deliberate fixture overrides; production `make help` has empty stderr.

The source-owned optional npm checker fixtures pass without executing npm, Node,
Cargo or network commands (`shared-npm-fixtures.log`); no npm root was selected
for this Rust repository. Snapshot integrity (80 files), Cargo pinning/inheritance,
documentation links and the diff pass (`final-governance.log`). ShellCheck passes
with the sourced PR companion included (`final-shellcheck.log`); the earlier
invocation omitted that companion and retained its SC1091 diagnostic separately.
Source identities and the working diff are retained alongside these logs.
No authentication or dependency graph changed, so no Rust library, full CI or
live canister qualification gate was run for this tooling batch.

The compatible pending changelog is 0.1.9; manifest versions remain 0.1.8.
No commit, push, release or publication was run.

## Earlier Shared Tooling/MSRV batch (released in 0.1.8)

The following records preparation against 0.1.7, before the maintainer's 0.1.8
release above. Its pending-version and qualification wording is historical.

Shared Tooling `1872ed2c20f6c70689bb2249050b1d673c60bfa0` (0.1.28) matched remote
main at observation. A clean isolated checkout exported committed bytes; an
initial attempt used an origin URL with a different `.git` spelling and was
refused before writing (`snapshot-refresh.log`). The corrected canonical URL
refresh passed (`final-snapshot-refresh.log`). The 79-file selection includes the
new complete governance/task catalog, malformed-link installer admission and
simulation-only release-runner fixture. `make tasks` reads that catalog;
no maintenance agent, timer or schedule was activated.

Rust 1.88.0 is now the common qualified package minimum, inherited by all five
members. Development Rust remains 1.99.0. The actual Rust 1.85.0 native probe
failed because selected `psm` 0.1.32 and `ar_archive_writer` 0.5.3 require 1.88.0
(`probe-1.85-default.log`); no dependency downgrade or ignore-version escape was
used. Native/Wasm 1.88 probes passed before declaring the floor. See
[the minimum compiler contract](../msrv.md).

`make check-msrv` prints real old-compiler identities and checks normalized
public package payloads outside the production workspace. Each library feature
and all features together pass on native and Wasm; internal tooling/runner
all-targets/all-features and the Wasm canister pass separately. Local protocol
payload staging preserves all external versions/sources/checksums; the script
checks that invariant against the workspace lock. The final successful run retained
`target/portable-fixtures/msrv.n5KtOq` (`qualified-msrv.log`); earlier successful
inputs remain separately retained. Setup explicitly
prepares 1.88.0 and its Wasm target without changing development selection or
updating rustup (`msrv-install.log`). The configured complete CI/release gate
now includes the minimum lane; local execution remained focused.

The initial Host 0.8.1 inspection was followed by a concurrent lock update to
all four Host 0.8.2 packages. That update was preserved. Live unyanked registry
checksums match the final lock; packaged source and observed remote main are
`92bd2fecc71124b562e227a32a67644e1e5e34b7`. The committed 0.8.1-to-0.8.2 diff
changes a portable child fixture from `/bin/true` to `/bin/sh -c 'exit 0'` and
release metadata, with no production API/runtime/dependency change. Dirty sibling
files were read-only and never imported. Auth graphs remain free of testkit,
native Host, Canic and wallet dependencies.

Released protocol/auth 0.1.7 checksums match the retained publication receipt:
`ccdbbc0217ee785e69d9c4d64f5e00a096e240ad4893a310f0cceecc4c9c0b5b` and
`130c5ef49bf8c9288acdf9ed9a78950a5daafd130234db4006ad897581b1e420`.
Both initially observed release CI runs were queued; the final observation has
one running and one queued (`final-release-ci.json`). No completed hosted
qualification is claimed:
[run 37800480770](https://github.com/dragginzgame/ic-auth/actions/runs/37800480770),
[run 37800481361](https://github.com/dragginzgame/ic-auth/actions/runs/37800481361).

Real testkit scenarios and strict native/Wasm Clippy pass on Host 0.8.2
(`host-0.8.2-qualification.log`, retained state
`target/portable-fixtures/qualification.u6K2B1`). Four native file tests, the
local/mocked consumer release fixture and the adopted simulation-only runner pass
(`focused-tooling.log`). The source-owned host/IC installer fixtures pass,
including malformed active links and actual collector cases
(`upstream-host-fixtures.log`, `upstream-ic-fixtures.log`); those synthetic native
host substitutions do not establish real macOS execution or footprint savings.
Both public package dry-runs build without uploading (`package-dry-run.log`).
The GitHub description remains consistent with implemented library scope.

Current evidence is retained separately under `target/baseline-0.1.28/`.
Final formatting, tool checks, snapshot, pinning/inheritance, links, metadata and
boundaries are recorded in `final-governance.log`; ShellCheck passes for the
consumer-owned new scripts. Final source/lock/snapshot/Wasm and normalized package
identities are recorded in `source-identities.sha256` in that same directory.
The selected next version is 0.1.8: support for a lower compiler, additive local
commands and shared tooling fixes preserve public auth APIs and signed bytes.
Manifest versions remain 0.1.7. No commit, push, release, publication, production
deployment or sibling adoption was performed. Wallet service policy and durable
session host adoption remain separate; older sections retain their original
pre-release selection/evidence wording.

## Earlier IC Testkit adoption (released in 0.1.7)

The following records preparation against 0.1.6, before the maintainer's 0.1.7
release above. Its pending-version and qualification wording is historical.

The maintainer required `ic-testkit` as the canister test infrastructure. The
working qualification package now selects published `ic-testkit` 0.25 and uses
its runtime re-export, typed Candid calls, IC Host file reader and bounded startup
with retained server stdout/stderr. The direct `pocket-ic` declaration and exact
pin exception were removed; no authentication library API or signed bytes changed.
Both complete certificate/lifecycle and signed-ingress scenarios are retained.

The command is now `make test-qualification`; its wrapper and documentation are
`scripts/dev/test-qualification.sh` and
[the IC Testkit contract](../ic-testkit-qualification.md). There is no alias for
the earlier uncommitted command. `ci/ic-auth-tools.tsv` is the consumer-owned
matrix: PocketIC 16.1.0 matches testkit's transitive client, while other tools keep
the shared defaults. Official release metadata verified all three archive hashes;
the immutable shared matrix/snapshot remains unchanged. Make setup, checks and
the offline qualification wrapper all select the consumer matrix. No test-time
download or alternative server discovery is enabled.

Published testkit 0.25.0 is unyanked, checksum
`1757127736e1d3bd4e6abe4d49017cfa75e2cdd7f9b07cb6a634a789c87e8c8f`.
Its package source revision and observed remote main are
`6b6d2cfe7f4c3e7a204a0c18beb6edb8895007b4`. The sibling's dirty files were
read-only and excluded. The concurrent Host fs/artifact update to 0.8.1 is preserved;
all four selected Host packages now use 0.8.1 from source
`973f00a029dd873c242a4d06e9f8d2d5172ff0df`. Qualification
dependencies remain confined to the unpublished native runner.

Both full scenarios pass through testkit with the checked 16.1.0 server
(`final-qualification.log`, retained state
`target/portable-fixtures/qualification.VRqUsc`), including CDPATH-sensitive wrapper
invocation. Strict native/Wasm Clippy passes (`qualification.log`). Local tool,
formatting, snapshot, pinning/inheritance, link, metadata and library-boundary
checks pass, as do the four native file-operation tests (`final-checks.log`).
ShellCheck and local/mocked release fixtures pass (`shellcheck.log`,
`final-release-fixtures.log`); the fixture copies the consumer matrix and new
wrapper along with the complete local packages. Both public package dry-runs
build successfully without uploads (`package-dry-run.log`). All four Host 0.8.1
registry checksums match the final lock and are unyanked (`ic-host-*-registry.json`).
Final source, lock, both matrices, snapshot, Wasm and package archive identities
are recorded in `target/testkit-adoption/source-identities.sha256`.
An initial formatting check
found the longer renamed command string, and a later link check found the old
document path; both were corrected before final checks. Server stdout/stderr
are retained separately per instance. Native macOS execution of these changed
sources awaits the configured CI jobs; no complete local CI gate was run.

Evidence from the earlier direct-PocketIC implementation below is historical;
its source identity list does not attest these changed files. Current adoption
evidence is retained separately under `target/testkit-adoption/`. Versions remain
0.1.6 with pending 0.1.7 notes; no commit, release or publication is authorized by
this correction.

## Earlier direct PocketIC qualification batch

`make test-pocketic` builds the unpublished qualification host and tests real
certification, root composition, expiry cleanup and protected metadata upgrades.
The host uses actual runtime caller/time/certificate values, controller-only
signing and one certification root containing assets, signatures and status.
The native runner uses the PocketIC instance's network key and the real library
verifier. Its independent resource canister checks an installation-protected
owner after ingress authentication. See the
[qualification contract](../ic-testkit-qualification.md).

The HTTP gateway scenario uses the IC agent's canonical delegation encoding and
real signed requests. Fresh Ed25519 session keys retain the same fixed fixture
principal across signer/resource upgrades. The replica rejects wrong targets,
changed signed expiration, wrong session keys and expired delegations. Valid
non-owner and anonymous requests reach the application guard and are denied,
separately from replica credential rejection. Previously issued, unexpired
delegations still authenticate after pending leaves are reset; clearing the
store is not revocation. Stable owner/branch metadata survives the fixture's
upgrade; this is not a stable session or signature-restoration adapter.

Both integration scenarios pass on Linux (`final-strict-qualification.log`), with retained
instance state under `target/portable-fixtures/pocketic.*`. The wrapper checks
the complete pinned IC bundle and locked client/server alignment before any
server execution; both PocketIC selections are 16.0.0. Negative ingress assertions
require HTTP 400/403 and the corresponding target/signature/expiration refusal;
transport failures cannot satisfy them. An initial strict assertion expected 403
for target rejection; the server correctly returns 400 for that case
(`strict-rejection-qualification.log`). The final run checks both supported
authentication rejection codes and specific reasons, and retains state in
`target/portable-fixtures/pocketic.kJbtc7`.
The first attempt lacked
an NNS subnet and correctly had no network root (`initial-pocketic.log`). Adding
that required trust-root subnet fixed the fixture, without changing library
verification. Intermediate and final successful logs remain separate.

The IC agent 0.49.2/CDK 0.20.3/PocketIC dependencies are confined to unpublished
applications. Bounded no-follow Wasm reading uses `ic-host-fs`. PocketIC owns its
server protocol/process behavior. Its 16.0.0 package pins `thiserror` 2.0.18, so
workspace resolution necessarily selects that patch instead of 2.0.21. Existing
type/encoding/signature/token/session and native tooling focused tests pass with
the resulting lock, as do every selected Wasm graph, metadata, dependency guards
and strict native/Wasm Clippy (`focused-rust.log`, `final-clippy.log`). No full
local CI/workspace gate was run. Final fixture-only source cleanup removed an
unused direct upstream signing dependency; metadata, both PocketIC scenarios and
strict native/Wasm Clippy passed again afterward (`final-strict-qualification.log`).

The initial user-changed Host 0.7.2 lock entries were preserved, followed by a
concurrent root/lock selection of Host 0.8.0. Live registry records match those
final fs/artifact entries; their packaged source revision is
`fc74f679c7503ef9dd2db8fbd893c5c5907c72c4`. Focused Clippy/native file tests and
the final PocketIC execution use 0.8.0. Its process cleanup API break is outside
this repository's selected fs/artifact usage; no local public contract changed.
The initial qualification executions used 0.7.2 and are not relabelled.

Shared Tooling source `db039347d2372b877c1c46dcdd2b5c3aa9412009` (VERSION 0.1.27)
matches remote main. An isolated clean copy exported committed bytes, excluding
the sibling's uncommitted edits. The canonical 65-file snapshot includes the
failure-evidence selector and PocketIC alignment/binary helpers. Default tool
retention remains full; no CI compaction option was selected. The source-owned
host evidence fixture passed against an isolated copied tool bundle, including
the actual collector (`selector-fixtures.log`). An earlier invocation omitted
required arguments and was refused (`upstream-evidence-selector.log`). This is
producer/helper evidence, not hosted GitHub artifact-roundtrip qualification.

Release fixtures now copy the actual adopted snapshot and discover every local
workspace package before synthetic version rewriting. Local/mocked publication,
runner and archive fixtures pass (`portable-fixtures.log`); warnings there come
from deliberate substitute Make recipes, not duplicate production recipes.
They perform no real repository release or registry upload.

Formatting, the 65-file snapshot, dependency pinning/root inheritance and local
links pass (`governance.log`). The exact SDK pin was initially refused by the
dependency guard; its client/server compatibility requirement is now documented
in `AGENTS.md` and scoped in `ci/dependency-pinning-exceptions.json`.
ShellCheck passes for the new wrapper and changed consumer release fixture
(`shellcheck.log`); that fixture passes again with the exception overlay
(`final-release-fixtures.log`). Both public package dry-runs build successfully
using live registry metadata, without upload (`package-dry-run.log`). The GitHub
description remains consistent with the independent authentication libraries
(`repository-description.json`). Final source, lock, snapshot and Wasm identities
are retained in `target/pocketic-qualification/source-identities.sha256`.

Live 0.1.6 registry checksums match its receipt: protocol types
`fad377ac63ecda737ec1636d79690729fc07b6b5f2d749262ed294b412645bec`, auth
`7eb9c82e1657c68bc9127ba02142a9d04c0c9e2e691a53fd837ef4a701521ddf`.
Both maintainer-triggered 0.1.6
[CI](https://github.com/dragginzgame/ic-auth/actions/runs/37782819316)
[runs](https://github.com/dragginzgame/ic-auth/actions/runs/37782819222)
completed successfully. This qualifies that released source on the configured
hosts, not the new uncommitted fixture on macOS.

The selected next version is 0.1.7: new commands/internal qualification and
shared tooling adoption are compatible; public auth APIs, signed bytes and
default dependency boundaries are unchanged. Manifest versions stay 0.1.6.
No commit, push, release, publication, production deployment or sibling adoption
was performed. Wallet service policies, wallet proofs/browser login and durable
session adoption remain separate. Older evidence below retains its original
source/selection wording and absent target files are historical references.

## Earlier signature preparation/retrieval batch (released in 0.1.6)

The following records preparation against 0.1.5 before the maintainer's 0.1.6
release/publication above; pending-version and qualification statements below
describe that earlier evidence.

Optional `canister-signature-preparation` exposes a bounded volatile
`SignatureStore`, independently of verification/token/session features. It
retains the existing Canic domains, empty signature paths, upstream DER keys and
CBOR proof DTO. The host authorizes preparation and supplies actual signer,
protected limits, clock, query certificate and ordered composition siblings.
There are no IC runtime calls, implicit root writes or native Host dependencies.
See the [contract](../signature-preparation.md) for input, lifecycle and
certificate-source obligations.

Physical capacity includes expired records. Each leaf has one expiry index
entry; live retries preserve their deadline, while expired-key repreparation
replaces that entry. Explicit prune removes at most 128 earliest expired records;
remove/prune preserve other leaves and never clone or scan the complete store.
All fallible preparation checks precede mutation. Retrieval enforces the
exclusive deadline and requires the supplied host certificate's canister root
to match the current composed witness digest. Its structural certificate check
does not substitute for consumer BLS/freshness verification. Deleting leaves
cannot revoke proofs already delivered.

Read-only extraction input remains Canic
`4c51a87c6a32397196bb3f65d064641194df10a5`; its dirty sibling changes were excluded.
The selected upstream signing-map source was reviewed rather than wrapped:
its internal clock and query-certificate calls are incompatible with this pure
boundary. DFINITY's certified tree, key and hash primitives are reused. Endpoint
eligibility, operation/caller records, renewal and other certified state remain
host-owned. Stable root publication/restore and real IC ingress delegation are
not qualified by this pure mechanism batch.

Eleven new tests pass, including 1999 live retries without expiry-index growth,
expiry/repreparation, failure atomicity, overflow/clock bounds, multi-key removal,
bounded pruning, certificate/output/depth limits and real BLS verification of
both Canic domains composed with two other certified branches. All 70 workspace
tests pass, including existing token/session and native file operations.
Strict all-feature Clippy, default and each independently selected Wasm graph,
locked metadata and dependency guards pass. Logs are retained under
`target/signature-preparation/`: `store-tests-fixed.log`, `workspace-tests.log`
and `compile-and-boundaries.log`. The first test compile rejected a borrowed
temporary (`initial-tests.log`); the following fixture run correctly rejected
an obsolete certificate and an undersized output limit (`store-tests.log`). The
fixtures were corrected without weakening production checks; failures remain.

The Cargo publication dry run built both archives and explicitly aborted uploads
(`package-dry-run.log`). The unpacked auth archive passed the preparation unit
and ten integration tests against checksum-verified published protocol types
0.1.5, and its preparation-only feature compiled for Wasm. Evidence is
`packaged-tests.log` and `packaged-wasm.log`. The isolated fixture adds an empty
workspace table and replaces Cargo's staged protocol checksum with the observed
published checksum; archive/source bytes are unchanged. This checks the package
boundary, not a live upload or a durable host.

Formatting, the 62-file snapshot, dependency pins and local links pass
(`governance.log`), along with ShellCheck and the final diff review.
`source-identities.sha256`, produced through the native `ic-host-*` utility,
binds source/tests, metadata, selected tooling/guards and the original archives.

The concurrently changed lockfile selection of `ic-host-fs` and
`ic-host-artifacts` 0.7.1 was preserved. Live registry checksums match the selected
entries and packaged source revision `410fee7c309e781edf6a361f0e480d71b7c11e5a`,
also observed as remote Host main. Native tests and Clippy use that selection.
Shared Tooling remote main still matches the adopted 0.1.26 revision
`75a8a60f49cec11d3f6aecab5c977029c42cc549`; no snapshot refresh is needed.

Live 0.1.5 registry checksums match the retained publication receipt: protocol
types `07832762f8124cfdc79d3c897e1ee74671b60e993cd8aa027be03c28643751bf`, auth
`40e6ed17becd4bd06fef97ef9d2d1ec9da09f06fd2a800a4613b96500487570c`.
The maintainer's 0.1.5 CI runs were still queued at the latest observation;
the preceding [0.1.4 run](https://github.com/dragginzgame/ic-auth/actions/runs/37772047548)
completed successfully on all configured hosts. Neither result qualifies this
uncommitted preparation batch on macOS.

The pending version is `0.1.6`: preparation is a compatible optional addition;
existing APIs, signed bytes and default graphs are unchanged. Manifest versions
remain 0.1.5. No commit, push, release, upload, full local CI, sibling adoption
or stable/PocketIC lifecycle was performed. Earlier target logs named below were
not present when this turn started and remain historical references.

## Earlier session/replay batch (released in 0.1.5)

The following records preparation against 0.1.4 before the maintainer's 0.1.5
release/publication above; its observations and pending-version wording are
historical evidence.

`ic-auth` now has optional `sessions`, with complete token verification before
new admission, canonical scope narrowing and exact Canic request hashes. The
engine resolves retry and consumption inside the host's synchronous transaction,
then stages only the touched session/replay pair. Exact retry after proof expiry
returns the unchanged still-authorized session. Replacement and logout retain
replay history; authority generation changes do not erase it. Live checks include
actual caller/subject, audience, role, current scopes, generation, enrolled root
key identity, epoch/version floors and narrowed root acceptance deadlines.

`MemorySessionStore` supplies a bounded volatile backend with explicit physical
record/encoded-byte quotas, indexed expiry cleanup capped at 128 records, checked
generation advancement and no whole-store clone/encoding. Per-record CBOR decode
is bounded and validates structural invariants; it is for trusted host storage,
not an ingress credential. The [session contract](../sessions.md) specifies
transaction rollback, monotonic cleanup time, durable restore and protected
authority-generation obligations. It distinguishes local session lifetime from
the original proof, IC ingress delegation and resource ownership. No CDK, stable
memory allocation, globals, certification publication or wallet graph was added.

The read-only extraction input remains Canic
`4c51a87c6a32397196bb3f65d064641194df10a5`. Its workflow, pure application policy,
scope model and storage operations were reviewed from committed source; the
sibling's staged/dirty work was excluded. Canic still owns its protected fleet
context, endpoint guards and durable backend. Consumer adoption must settle its
retained representation and remove the superseded owner in a separately
authorized change. This batch does not claim that ownership convergence.

Fourteen session tests pass with real ECDSA/BLS chains, including forged-proof
rejection, strict proof starts, TTL overflow at the maximum clock, exact retry,
generation/root-key/scope invalidation, logout/replacement, byte/capacity failures,
rollback after staging, corrupt record rejection and indexed bounded cleanup.
The request hash has an independently computed byte vector. Existing protocol,
token, IC signature and type tests pass too, along with strict all-feature
Clippy, default/signature/token/session Wasm compilation, locked metadata,
dependency boundaries and native tooling tests. Focused logs are retained under
`target/session-engine/`: `focused-checks.log`, `final-focused-checks.log` and
`session-final-checks.log` distinguish successive source/host selections.

The final Cargo package dry run built both archives and aborted uploads
(`package-dry-run.log`). The unpacked auth payload passed all 14 session tests
offline against checksum-verified, published protocol-types 0.1.4 and compiled
for Wasm with `sessions`. Logs are `packaged-native.log` and `packaged-wasm.log`.
The isolated fixture added an empty workspace table and replaced Cargo's staged
protocol checksum with the observed published checksum; library source and
original archives were untouched. This verifies the package boundary, not
downstream application or stable host adoption. Formatting, snapshot integrity,
dependency pins, local links, ShellCheck with sourced scripts and the diff pass;
evidence includes `governance-checks.log` and `shellcheck.log`.
`source-identities.sha256`, produced with the native `ic-host-*` utility, binds
the final source/tests, manifests/lock, Makefile, toolchain, shared snapshot and
selected guards/runner. Original validation logs and failed attempts are preserved.

The initial session build rejected the fixture's module path before execution;
`initial-tests.log` records it. The next run failed one expected strict-start
case because its IC certificate was signed ahead of the test clock. The fixture
time was corrected without weakening production freshness checks; the failed
run remains in `session-tests.log`. Later checks stopped correctly when the root
manifest concurrently selected Host 0.7 while the lock still selected 0.6;
`locked-host-selection-failure.log` records that refusal.

The concurrent root selection was preserved. Published `ic-host-fs` and
`ic-host-artifacts` 0.7.0 availability and checksums were verified, and their
packaged source revision `491fc0e231b9650526f5f57b9ab7b1f62f02218c` matched remote
main. Explicit offline workspace resolution changed only those two lock entries;
explicit locked fetch prepared them before offline validation. Native bounded
hash/no-follow/durable-create/replacement tests and Clippy pass on 0.7.0.
The earlier focused checks used 0.6.0 and are not relabelled. Both native owners
remain outside every auth graph; no Candid/process dependency was needed.

Shared Tooling 0.1.26 at `75a8a60f49cec11d3f6aecab5c977029c42cc549` was verified
as remote main, reviewed and adopted through its canonical exporter from the
clean sibling source. Its 62-file snapshot preserves the source-owned runner's
locked Git-ref transaction and concurrent symbolic-ref handling. Rules/tool pins
are unchanged. The adopted runner regression tests and local bare-remote release
adapters with mocked registry/upload transport pass; they do not run a live
release or the full local CI gate.

Live sparse registry records match the retained 0.1.4 intent: protocol types
`f3b8439d22f13f7bfd5fb382d5563f213c308a5951dbddb2a6014421fe926e56`, auth
`baad1f7e93287cfc77ccb863b2fad5760b650749ef9597fd0d347b65ca2fa4ee`.
The public repository description was corrected to the released token-verifier
scope and read back. The maintainer's 0.1.4
[CI run](https://github.com/dragginzgame/ic-auth/actions/runs/37772047548)
passed Linux and ARM macOS; Intel macOS was still running at the final observed
read. This qualifies the pushed 0.1.4 source on those completed hosts, including
the fixture-isolation fix. Native macOS qualification is not inferred for this
new session/Host 0.7/Shared Tooling 0.1.26 batch.

The pending version is 0.1.5: session APIs are optional compatible additions,
existing token/signature APIs, signed bytes and default graphs are unchanged,
and the native Host/shared-runner updates preserve maintained command contracts.
Manifest versions remain 0.1.4. No commit, source push, release, upload, full local
CI, stable backend/PocketIC lifecycle or downstream adoption was performed.
Current evidence is under `target/session-engine/`; the earlier target logs listed
below were not present when this turn started and remain historical references.

## Earlier application-token verification batch (released in 0.1.4)

The following evidence records preparation against 0.1.3 before the maintainer's
subsequent 0.1.4 release/publication recorded above.

The optional `token-verification` feature verifies the complete existing root
ECDSA chain-key/Merkle proof and issuer IC BLS proof. Actual authenticated caller,
clock, root-key enrollment, audience, local role, allowed/required scopes, finite
input limits and live epoch/version/acceptance policy are explicit host inputs.
Both cryptographic verifiers are concrete; no accepting callbacks or retained
identity cache can bypass a proof. Verified claims are borrowed, cannot be
deserialized into verification evidence, and expire no later than the host's
current root-key acceptance deadline. The [API contract](../tokens.md) explains
the remaining host enrollment, decoding, replay, session and resource-policy
obligations.

The read-only source remains committed Canic
`4c51a87c6a32397196bb3f65d064641194df10a5`. Its root chain-key header, delegation
leaf, Merkle directions, key/path bindings and issuer domain are preserved.
The canonical derivation-path hash now has an independent fixed byte vector.
`k256` owns SEC1 parsing, ECDSA scalar validation, high-s detection and prehash
verification. No custom cryptography, Canic authority registry, network-key-name
policy, storage, runtime clock, certification-root publication or Solana graph
was imported. Native file/artifact operations continue to use `ic-host-fs` and
`ic-host-artifacts` `0.5.2`; they remain excluded from both auth libraries.

Focused Linux checks passed: 41 auth tests across the selected APIs, strict
all-feature Clippy, default/signature/token Wasm builds, locked metadata and
library dependency boundaries. The 12 token tests use real ECDSA and BLS proofs
and cover identity, grant narrowing, authority invalidation, exact time bounds,
forged signed bytes, malformed/high-s signatures, witness tampering and input
budgets. Logs are under `target/token-verification/`, including
`focused-checks.log` and `initial-tests.log`. `source-identities.sha256`, produced
by the native `ic-host-*` utility, binds the final library/tooling source and
tests, manifests, lockfile, Makefile, toolchain, snapshot and boundary guard;
documentation-only source comments were completed before final packaging.
No existing locked external version
changed; 12 selections were added for optional `k256` verification.

The pushed release's native CI exposed an intermittent parallel fixture
collision: in [run 37764724605](https://github.com/dragginzgame/ic-auth/actions/runs/37764724605),
both macOS jobs failed the tooling intent test after reading the hash test's
`abc` input. Linux passed. The old directory name used PID and clock precision,
and `create_dir_all` could share a directory between simultaneous tests.
The local helper now adds a process-wide atomic sequence and exclusively creates
each leaf directory. Four host-tooling tests and strict Clippy pass locally;
native macOS qualification of this fix awaits CI for the new source. The failed
job log is retained in `previous-ci-failure.log`, and local checks in
`fixture-checks.log`. This changes test isolation, not production intent writes.

The final real Cargo package dry run built both distributable archives and
aborted uploads (`package-dry-run-final.log`); the earlier dry run remains in
`package-dry-run.log`. The unpacked auth payload also compiled offline for native
and Wasm with `token-verification` against the checksum-verified, published
protocol-types `0.1.3`. This isolated fixture copied the normalized manifest,
added an empty workspace table, and replaced Cargo's staged-dependency checksum
with the observed published checksum; it did not change the source or archives.
Logs are `packaged-native.log` and `packaged-wasm.log`. The first attempt refused
the enclosing workspace before compilation; that result remains in
`packaged-native-workspace-failure.log`. This is package-boundary compilation,
not downstream application adoption.

Formatting, dependency pins, snapshot integrity, local links, boundary-script
ShellCheck and the diff passed. Local bare-remote release adapters and mocked
publication/recovery passed with a substituted complete gate, not live effects;
logs are `governance-checks.log` and `release-adapter-checks.log`. Recipe override
warnings occur only in that intentionally substituted fixture Makefile. The
maintained Makefile has one recipe per target. Existing-version dry-run warnings
are expected because manifests remain `0.1.3`.
Sparse registry records independently matched the
retained `0.1.3` publication checksums: protocol types
`b667ca6a00543bc8a3e36d2d6d8a8e3cee2528e986a894a28025c86311aec273`, auth
`56f767225901ae8e618ddddb266d5391cb755bd13e7caba9dafed24da2c52e25`.
The crates.io API returned HTTP 403 during read-only checking; the public sparse
index supplied the live registry observation instead.

Shared Tooling `0.1.25` at `eeb72e741199bd8574280eacb3542d8379b912f6`
was reverified as remote `main`; the reviewed 62-file snapshot is unchanged.
The repository description was read and still accurately describes the released
scope. The next undated changelog entry is `0.1.4`: optional token verification
and fixture isolation are compatible additions/fixes, with existing encoding,
wire contracts and default graphs preserved. No manifest bump, commit, push,
release, publication, full local CI, native macOS run, live IC/PocketIC execution
or consumer adoption was performed by this batch. Session/replay atomicity,
signature preparation/retention and wallet service contracts remain separate
unfinished extraction work in the existing tracker.

## Earlier signature verification batch (released in 0.1.3)

The following evidence records preparation against `0.1.2`, before the
maintainer's subsequent `0.1.3` release and publication recorded above.

The maintainer requested continuation of the accepted extraction. The working
tree now adds the optional `canister-signature-verification` feature to `ic-auth`.
It uses DFINITY's public-key parser and IC verifier with protected signer, seed
hash, raw network root key, clock and finite byte/freshness limits supplied by
the host. It guards the upstream short-input DER slice, requires exact short-form
DER re-encoding, and checks authenticated signing-certificate time after signature
verification. The [API contract](../signatures.md) distinguishes message
authentication from application-token/session admission.

The read-only Canic input was committed
`4c51a87c6a32397196bb3f65d064641194df10a5`; sibling dirty changes were excluded.
Issuer and root-role signature domains are preserved byte-for-byte. No Canic
authority, global signature maps, metrics, clock calls or certification-root writes
were imported. Preparation, retrieval, bounded signature retention, host-owned
certification composition, the complete token/session engine and wallet service
remain unimplemented. The existing `0.1.2` registry packages still contain only
the earlier contracts/encoding; this addition is local and unpublished.

Focused Linux checks passed: signature tests using real BLS-signed fixture
certificates, the existing 17 protocol/type tests, all-feature strict Clippy,
default and verification-selected Wasm compilation, locked metadata and both
selected dependency graphs. Evidence is retained in
`target/signature-verification/validation.log`. Coverage includes exact Canic
domain bytes, protected authority changes, message/certificate/tree tampering,
subnet delegation range and nesting rejection, DER truncation/integrity, input
limits and missing/truncated/overflow/trailing time with exact age/skew boundaries.
The initial run failed one fixture expectation: changing the encoded principal
length can form a structurally valid key with the wrong signer. The test now
separates malformed lengths from the maintained `CanisterMismatch` contract;
production verification was unchanged. The failed run remains in the session
transcript and its build artifacts were preserved.

No existing locked registry package version changed during this addition; the
23 added selections support optional verification and its fixture cryptography.
`ic-host-fs`/`ic-host-artifacts` were already selected at `0.5.2` when this turn
started and remain in the native app only. The earlier host-tooling evidence below
describes its original `0.5.1` adoption. The published Host `0.5.2` source at
`c7014995bf0890c1df9cd9b9a6ec14ea70f98c6f` was verified as remote `main` and
inspected read-only; no unrelated package update was performed.

Shared Tooling advanced to `0.1.25` at
`eeb72e741199bd8574280eacb3542d8379b912f6`, verified as remote `main`. The reviewed
delta hardens archive paths, including inherited `CDPATH`, option-like names,
newlines and occupied pipes. The canonical exporter initially refused to replace
the earlier uncommitted snapshot files. After verifying the old snapshot, its
exact source files were preserved under `target/shared-tooling-evidence/0.1.24/`;
a clean source and isolated canonical export supplied the replacement bytes.
The 62-file snapshot includes the upstream archive regression fixture. No shared
script was patched locally. Baseline rules and tool pins are unchanged.

The adopted archive regression fixture passed; formatting, dependency pins,
snapshot integrity, local links, ShellCheck and substituted release Make routing
passed. Logs are under `target/signature-verification/`. The live registry-aware
package dry run built both distributable archives and aborted upload; its log is
`target/signature-verification/package-dry-run.log`. Existing-version warnings
reflect unchanged manifest version `0.1.2`, not a new publication attempt. The
public description was read and remains accurate for the released encoding APIs
and complete token verification still in development.

The pending release remains `0.1.3`: the optional verifier is a compatible
addition, existing encoding APIs/wire bytes are unchanged, and no manifest version
was bumped. No full local CI, native macOS, live network signature, PocketIC,
commit, source push, release or upload is claimed by this batch.

The earlier host-tooling snapshot contained 61 committed Shared Tooling files from
`e9bfdc54c0daefc3dbbdfe091e5665dca5468eb3` (0.1.24), verified as remote `main`.
It was exported by the canonical refresh helper from a clean temporary clone,
preserving the sibling's dirty archive/evidence work. The snapshot verifier
passed. The snapshot supplies shared
Make/tool setup, formatter hook and focused governance helpers. Pinned host,
Rust and IC tools are installed in this checkout and `make tools-check` passed
on Linux x86_64. Tool installation output is retained at
`/tmp/ic-auth-install-tools.log`; installation build output remains under
`.tools/rust/build/`. The repository-local formatting hook is enabled.
The refresh adds the direct-runner regression fixture, evidence archiver and
failure-retention action. Latest final-hook payload/index/tag checks, completed
resume reconciliation and conditional remote-tracking refresh are adopted.
Tool pins and the toolchain selection are unchanged; no tool upgrade was needed.

## Earlier host-tooling adoption

The maintainer requested continued work using `ic-host-*` where applicable and
the latest committed Shared Tooling. That batch originally selected
published `ic-host-fs`/`ic-host-artifacts` 0.5.1 only in `apps/tooling/`. Registry
availability/digests were checked; the sibling's dirty process changes were not
adopted. Existing locked package selections were preserved; only the host app,
its two owners and their required dependencies were added. The dependency guard
now rejects native host dependencies in either auth library graph.

The native CLI delegates bounded SHA-256 identities, no-follow reads and durable
private-file publication to those owners. Release validation receipts use durable
replacement; publication intent and dispatch markers use durable create-only
writes before upload dispatch. Limits and caller ownership are documented in
[development](../development.md). Registry policy, uncertain-effect recovery,
locking and Git orchestration remain here or in their adopted canonical scripts.
No current Candid/process adapter calls justify adding the other two Host crates.

Focused Linux validation passed: four native CLI tests, strict Clippy, locked
metadata, Wasm compilation, auth graph boundaries, release Make routing,
the latest upstream runner fixture and local/mocked consumer release/publication
fixtures. The exact already-published `0.1.2` commit also passed the updated late
metadata check: local package transformation is derived from its saved source,
so a release predating the host app remains verifiable. ShellCheck, Actionlint,
formatting, installed tool checks, pins, snapshot integrity and local links pass.
The evidence archiver was exercised against real retained host fixtures. The live
Cargo publication dry run built both distributable libraries and aborted uploads;
its log is `/tmp/ic-auth-host-publish-dry-run.log`. Existing registry selections
were not upgraded and native host packages are absent from the library graphs.

The first recovery fixture failed because locked Cargo could not rebuild the
utility while the interrupted manifest/lock pair was inconsistent. Receipt
reconciliation now uses the already-adopted portable checksum helper at that
boundary; normal validation/publication use the Rust utility. The corrected
fixtures pass. The failure remains in `/tmp/ic-auth-host-release-tools.log` and
`target/portable-fixtures/release-tools.HjHcgk/`; final evidence is in
`/tmp/ic-auth-host-release-tools-final3.log` and its reported fixture directory.
The publication fixture also changes source during dispatch-marker creation and
proves refusal before either upload; source is rechecked after the durable write.
The shared runner log is `/tmp/ic-auth-latest-release-runner.log`.

CI now runs the complete gate on Linux and macOS 15 Intel/Apple Silicon and
archives failed validation/fixture evidence from `target/portable-fixtures/`.
No native macOS, full local CI, PocketIC, new release, source push or upload was
performed by that batch. Its pending `0.1.3` notes selected a compatible patch:
library APIs, signed bytes and registry upload/retry semantics were unchanged.
The repository description was reviewed for that batch. At that time, the local
signature capability was not yet included in the published package contract.

## Earlier extraction and release evidence

This starts A1 and the encoding portion of A3 without claiming either complete.
Protocol `Fleet` labels and signed domains remain unchanged. Auth role parsing
now validates at construction/deserialization; the host still owns trusted
identity projection and all authority. Remaining Canic consumer adaptation,
generated-Candid/feature coverage and canonical ownership convergence stay in
the tracker. Preserve Canic's current release boundary and keep its reinstall-only
policy out of wallet identity storage.

Focused validation passed: 17 tests across both packages, the three imported
Canic hash vectors, Candid identity/role byte compatibility, Wasm compile checks,
Clippy with warnings denied, locked metadata, transitive dependency boundaries,
formatting, snapshot integrity and local links. Initial Clippy validation found a
constant-size iterator lint; it was corrected and the check passed. No full CI,
PocketIC, live verification, service or downstream-adoption evidence is claimed.

The implemented release/publication commands are tracked by
[IC Auth #1](https://github.com/dragginzgame/ic-auth/issues/1).
The shared runner now has working direct-delivery metadata and validation adapters
for patch/minor/major/resume. It retains complete-gate receipts and checks exact
selected commits, narrowly owned metadata, annotated tags and atomic branch/tag
delivery. Publication requires a clean, tagged, pushed release and verifies
crates.io archive checksums with retained intent before each upload.

Earlier focused checks cover real metadata transforms and local bare-remote
releases with a substitute complete gate, plus mocked registry/upload transport.
They exercise gate failure, staged source rejection, candidate conflicts, all
three increments, interrupted preparation, exact resume, dirty/untagged publish,
unknown registry state, checksum conflicts and lost-upload-reply reconciliation.
Fixtures and failed attempts remain under `target/release-tools.*/`. Package dry
runs use real Cargo and registry metadata, build both distributable crates and
abort upload. No live release or registry write is claimed by those checks.
Dependency pinning/root inheritance now pass against the tracked real lockfile;
formatting, locked metadata, shell/Perl checks, license copies, snapshot integrity
and local links are checked for this batch. The previous 17 auth tests remain
bootstrap evidence; no full CI or PocketIC run was added to this local tooling task.

Strictly offline attempts failed for unavailable registry metadata and Cargo's
staged-dependency checksum error; the first metadata fixture exposed yq's inability
to emit complete TOML. The implementation uses an explicit registry-aware dry
run and narrowly checked byte-preserving manifest edits. The interrupted-write
fixture also exposed an expected-receipt path error, which was corrected before
the recovery checks passed. Raw attempts are retained under `target/` and in
`/tmp/ic-auth-*-dry-run*.log` and `/tmp/ic-auth-release-tools.log`.

The maintainer's subsequent `make publish` attempt packaged `0.1.1` and stopped
at a registry checksum conflict for `ic-auth-types`, before either upload was
dispatched. The retained `.git/publication-state/0.1.1.json` binds that attempted
source, tag and archives; there are no dispatch markers for that attempt.
The existing package is actually `ic_auth_types`, from
[ldclabs/ic-auth](https://docs.rs/crate/ic_auth_types/0.1.1); crates.io's
[name restrictions](https://doc.rust-lang.org/cargo/reference/registry-index.html#name-restrictions)
prevent distinguishing it by hyphens versus underscores. Its delegation/CBOR
types do not replace our Canic-compatible application-token/proof contracts.

The delivered `0.1.2` repair renamed our package and directory to
`ic-auth-protocol-types`, with Rust import `ic_auth_protocol_types`, throughout
the workspace, dependency graph, publication intent, release transforms and
fixtures. `ic-auth` and the repository name are unchanged. Candid layouts,
signed domains and hash contracts are unchanged. Neither spelling of the new
name appeared in the registry index on 2026-10-08; that does not reserve it.
The old `0.1.1` tag, release receipt, publication intent and artifacts remain
intact. Dependency/import names are a breaking public Rust contract; the
maintainer explicitly selected version `0.1.2` as an exception scoped
to this rename, instead of the normally required `0.2.0` minor increment.
The rename remains marked breaking and requires consumer dependency/import
updates. The original repair did not execute a release or upload; subsequent
maintainer delivery/publication is recorded above. See
[development](../development.md) for current command effects.

At the maintainer's request, the changelog now separates bootstrap/extraction
notes under the original `0.1.0` entry from the release/publication tooling added
in `0.1.1`. The original undated `0.1.0` heading is retained, with its bootstrap
commit/date and explicit untagged/unpublished status; the finalized `0.1.1` date
is unchanged. These historical corrections are based on commits `7a03102`,
`15a7fe4` and `61b1139`, rather than a fabricated earlier tag or publication.
Local links, snapshot integrity and the documentation diff were checked; no
Rust builds or portable test suites were run for this documentation correction.

Rename validation passed: all 17 existing Rust tests, locked offline metadata,
dependency boundaries, Wasm compilation, formatting, license checks, snapshot
integrity, local links and shell/Perl checks. The release/publication fixture
passed with the renamed package, real transforms/local bare remotes and mocked
uploads; its evidence is retained in `/tmp/ic-auth-rename-release-tools.log` and
the named `target/release-tools.*` directory. The live registry-aware Cargo dry
run built both package payloads and aborted both uploads; its log is
`/tmp/ic-auth-rename-publish-dry-run.log`. External locked dependency selections
are unchanged. No full CI or macOS execution is claimed for this rename.

The dependency-pinning checker enumerates tracked paths as well as untracked
ones, so it rejected the unstaged directory move's missing old manifest. It
passed with the move admitted into an isolated temporary index, leaving the real
index untouched during that repair. The move was subsequently committed by the
maintainer. Failed attempts and previous publication evidence remain retained.

Documentation closeout identified the wallet-service contract decisions explicitly
in the design: application identity scoping, credential linking/recovery, distinct
revocation semantics and exact public protocol limits. They do not block initial
Canic library extraction, but must be settled before supported wallet login.
