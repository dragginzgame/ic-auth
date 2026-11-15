# Architecture

Status: proposed replacement architecture, 15 November 2026. This selects an implementation direction and acceptance obligations. It does not authorize consumer cutover, data deletion, a new issuer service or a storage-format change. The companion qualification plan below defines the implementation sequence.

## 1 Decision

Build a receiver-centered authentication library with one admission and session policy, two independently implemented storage backends, a separate issuer state owner and the private browser admission/logout coordinator.

The storage work is staged alongside the Auth rewrite. Volatile memory and published `ic-storage` supply the initial two backends; `ic-storage` is an interim implementation dependency. The intended durable destination is `ic-storage-new` with orthogonal persistence, once its required capabilities and Auth integration are qualified. The initial stable backend does not freeze the final persistence architecture or require maintaining three production backends indefinitely.

The application flow remains:

**Portable credential → receiver-local admission → caller-bound session → current authorization on each ordinary call.**

A receiver does not call an issuer or repeat signature verification on every application operation. It checks its retained session against current protected authority, time, revocation, device generation and required scopes. Application account linking and resource permissions remain outside Auth.

The architectural change is to make a complete receiver operation the unit of ownership. Session, replay, login denial, device replacement and accounting become one bounded transition. A stable backend should not have to reconstruct a temporary memory ledger to reuse policy. Both backends supply bounded operation facts to the same decision functions, then apply a validated change using their own storage mechanics.

This is a fresh implementation of selected responsibilities, not a port of the old module tree. The existing engine already shares important checks; a proposed consolidation earns its place by removing a transformation or independent invariant without losing refusal, recovery or cost guarantees.

## 2 Verified reference and scope

Reference: [dragginzgame/ic-auth at e52a40be75dd9809d0528189d84b43fe96952ad9](https://github.com/dragginzgame/ic-auth/tree/e52a40be75dd9809d0528189d84b43fe96952ad9). Public main resolved to that same commit during this review. Its commit message is “on hold.”

| Verified source fact | Design consequence |
| - | - |
| Workspace selects Engine 0.41.1, Protocol types/Wire 0.40.1 and Rust 1.100.0 | Preserve the reference identities and freeze a replacement supplier graph before qualification |
| Complete Ed25519 and certified verifiers feed one session engine | Keep one admission policy; do not introduce mechanism-specific sessions |
| Stable receiver reconstructs at most two selected login records into a memory ledger | Replace this translation with bounded receiver facts, subject to differential qualification |
| Stable caller records already combine an active session or retained fence | Preserve this useful locality; do not create a duplicate session mirror |
| Persistent issuer owns intents, renewal indexes and completed results | Replace its lifecycle deliberately, not with a stateless signing callback |
| Browser package is private and includes durable admission/logout coordination | Browser uncertainty handling remains product scope |
| Custom session-store commit/restore APIs are supported alongside the stable backend | Their removal requires actual consumer review, not an assumption that stable storage displaced them |

Sources: [manifest](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/Cargo.toml), [engine](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/session/mod.rs), [stable receiver](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/persistent/receiver/mod.rs), [persistent integration](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/docs/integration/persistent-auth.md), [browser contract](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/docs/protocol/browser-client.md).

These are inspected source contracts. Historical test and performance reports remain historical evidence; no replacement implementation or qualification run is established by this design. Some documentation describes older release cohorts. Pinned executable definitions and independently retained cases resolve those details before implementation.

## 3 Owners and package boundaries

| Owner | Owns | Must not own |
| - | - | - |
| Protocol types | Validated identifiers, passive credentials, grants and typed outcomes | Storage, runtime, signing effects |
| Wire adapter | Exact methods, DTO projections, limits and browser generation | Trust selection or handwritten alternative codecs |
| Canonical/proof code | Signed bytes, request identity, bounded complete verification | Session admission, enrollment provenance or resource policy |
| Receiver | Admission, authorization, logout, replacement, replay and maintenance rules | Transport, endpoint installation or automatic recovery from lost state |
| Receiver backends | Indexed facts, checked records, prepared mutations and accounting | Different authentication policy |
| Issuer | Login bindings, operation identity, renewal, effect intent and reconciliation | Wallet account authentication, private-key custody or a remote service |
| Browser coordinator | Login generation, issuance sequence, exact evidence, receiver inventory and logout uncertainty | Private-key storage or application-operation replay |
| Host | Authenticated caller/time, enrollment, transaction isolation, lifecycle and certification root | Reimplementation of Auth transition rules |
| Application | Account linking, eligibility, roles, resources and business effects | Treating a receipt as enduring authorization |

Retain three existing public boundaries: ic-auth-protocol-types, ic-auth-wire and ic-auth. Introduce an issuer package only for the real dependency boundary between receiving credentials and signing/issuance custody; proposed name ic-auth-issuer. Its state code may use the signer-free canonical base of ic-auth with receiver/backend features disabled. The receiver must never depend on the issuer package. Qualify the resulting feature-unified graphs.

Keep receiver memory/persistence implementations as modules behind explicit capabilities, not one package per table or operation. Isolate interim `ic-storage` leases, record encodings and collection APIs within their backend; do not expose them through protocol types, shared Auth decisions or mandatory receiver API bounds. Platform supplies cryptographic primitives and canonical Principal identity. Keep metadata generation, Host, Build, Testkit and release tooling out of deployed receiver dependencies.

Public signatures are proposals until the consumer inventory is complete. Package identity, Rust source compatibility, Wire compatibility and stored-record compatibility are separate decisions.

## 4 Frozen protocol contracts

### Identity and authority

A fresh Ed25519 browser key identifies one login. Derive its self-authenticating Principal from the standard DER public key and compare it with the authenticated ingress caller. A delegated signature whose sender remains a different account Principal does not satisfy that binding. [IC authentication](https://docs.internetcomputer.org/references/ic-interface-spec/https-interface/#authentication).

Accounts are qualified by (issuer, subject) inside one protected AudienceId. Device slots are issuer-assigned 0–7, with nonzero monotonically increasing u64 generations. Renewal preserves browser key, account, slot, generation, login start and absolute deadline. A higher generation can replace a slot only after complete proof verification. Overflow refuses.

A request may name an issuer/key as a lookup hint; it cannot install authority, choose a public key, relax a verifier or supply trusted time. One protected role selects one signed grant. Never combine scopes from different roles.

### Canonical bytes and request identity

Preserve claims schema 2 and the exact domain IC-AUTH-PORTABLE-CREDENTIAL followed by a zero byte. Preserve field order: schema, mechanism, network, application, issuer, key binding, browser key, account subject, slot, device generation, login start, login deadline, issuance, admission expiry, nonce, grants. Fixed-width integers use big-endian encoding; existing length/count prefixes and principal/label encodings remain unchanged.

Roles and signed scopes must arrive strictly ordered and unique. Reject noncanonical signed inputs; do not sort them into validity. Preserve all other current signed domains used by delegation/certificate helpers.

Unsigned session requests are different: their existing constructor sorts unique requested scopes. Preserve that normalization, the exact canic-application-session-request-v1 hash framing and the distinction between absent TTL and an explicitly requested default TTL. The historical domain text is a byte contract, not ownership of Canic policy.

The replay fingerprint is SHA-256 of exact claims bytes. Alternate proof bytes do not create another credential identity. Ed25519 verifies that digest using ordinary strict Ed25519, not Ed25519ph. Preserve small-order rejection and canonical scalar checking even under dependency feature unification. Certified verification preserves its separate ic-auth-portable-credential framing, complete delegation/issuer checks, trusted IC root and zero future-skew allowance.

Sources: [signed protocol](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/docs/design/0.6-auth-protocol.md), [request identity](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/session_request.rs), [strict verifier](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/portable_signature.rs).

### Limits and time

| Contract | Reference bound |
| - | - |
| Claims | 65,536 bytes |
| Fresh certified proof material | Aggregate variable-material cap at most 131,072 bytes, plus protected witness/signature/time bounds |
| Fresh credential grants | At most 16 roles and 32 role–scope entries in total; a repeated scope under another role counts again |
| Labels | 1–64 bytes under their existing grammars |
| Requested session scopes | 1–16 unique scopes |
| Grant allocation admission | 4,096-byte defensive cap; maximum valid fresh grant list is 3,332 canonical bytes |
| Login / proof / session ceilings | 90 days / 60 seconds / 1,800 seconds; host may tighten |
| Time | Checked u64 nanoseconds; exclusive deadlines; no positive grace |

Fresh claims require login_start ≤ issued_at ≤ now < admission_expiry ≤ login_deadline, with all original duration and enrollment checks. A session lease is capped by requested/protected TTL, login deadline, key validity and the captured enrollment freshness deadline.

Proof expiry stops fresh admission, not an already live session. Fresh enrollment cannot extend a stored lease. NotYetValid is a refusal/retry hint, never evidence of validity. No safety-critical finite clock-skew bound is assumed.

Completed issuer evidence retains its original accepted structural recovery rules. Do not turn the newer fresh-grant ceiling into permission to discard already completed results.

## 5 Exact Wire surface

Preserve these method identities, direction checks, DTO declaration order, nested tags and error projections. Names are reference bindings; consumers own actual exports.

| Contract | Identity | Request | Response |
| - | - | - | - |
| CredentialEnvelope | 0x415554480001 | PortableCredential | PortableCredential |
| EstablishSession | 0x415554480012 | credential, scopes, optional ttl_secs | Result of SessionView or AdmissionRefusal |
| ReadSession | 0x415554480013 | unit | Result of SessionView or SessionReadRefusal |
| Logout | 0x415554480014 | unit | LogoutResult |

SessionView fields remain caller, issuer, subject, expires_at_ns. Credential proof variants remain Certified then Ed25519. Admission refusal order remains NotYetValid, Expired, InvalidCredential, AuthorityUnavailable, BeforeReset, ScopeRejected, ReplayConflict, Revoked, Capacity, Throttled, Unavailable. Preserve the complete nested model rather than regenerating an “equivalent” enum order.

| Limit set | Bytes | Allocation | Nodes | Depth |
| - | -: | -: | -: | -: |
| Credential and admission requests | 524,288 | 2,097,152 | 65,536 | 32 |
| Ordinary replies | 4,096 | 16,384 | 128 | 8 |
| Unit requests | 11 | 128 | 8 | 4 |

Credential-envelope replies use the credential limit set. Browser credential retention defaults to 131,072 bytes, with configurable maximum 1,048,576 that never overrides Wire's lower bound. Browser logout replies retain their additional 1,024-byte cap.

Wire bounds, proof-domain bounds and browser retention limits are separate. For example, a bounded 64-byte signature transport field still requires exact signature length at verification. Host raw-argument admission precedes copying/decoding. Every boundary rejects truncation and trailing input according to its own contract.

Generate browser codecs from the Rust-owned schema through Wire. No local runtime fork or handwritten parallel codec. Canonical signed claims are opaque bytes in this transport. [Wire definitions](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth-wire/src/lib.rs), [nested models](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth-wire/src/model.rs).

## 6 Receiver state and operation model

Use these logical records; this is not a chosen storage encoding.

| State | Key and invariant |
| - | - |
| Caller state | Browser Principal → active session or retained known/unknown denial/fence |
| Device binding | (issuer, subject, slot) → current generation and caller; survives ordinary expiry/logout |
| Replay | Claims fingerprint → consuming caller, issuer-qualified account and retention deadline |
| Account counters | Physical replay occupancy across all devices, including after logout |
| Expiry indexes | Deadline plus stable tie-breaker → bounded maintenance work |
| Control state | Domain/configuration, authority generation, trusted clock observations, budgets and physical totals |

Keep active-session login binding in one authoritative logical record. Derived indexes must update atomically with it. Retain existing logical session/replay byte ceilings and accounting semantics even if an in-memory representation is smaller; separately bound actual heap/index/backing costs.

Admission loads only the caller, selected slot owner, relevant replay and account occupancy. No global scan or complete heap clone. Memory can borrow records; stable storage validates and decodes selected records. Never translate stable state through a temporary MemoryLoginRevocations object merely to invoke a policy rule.

One internal ReceiverFacts value contains these bounded facts and their transaction revision. Pure functions check liveness, narrowing, exact retry, replacement and capacity. A non-public ReceiverChange contains only affected records and resulting accounting. It cannot be serialized as an authorization token or applied after its originating transaction.

A private operation-level backend interface is justified by the two implementations. It exposes selected reads and prepared writes, not a public universal key/value store, repository framework or arbitrary transaction language. Storage-specific corruption checks and indexes remain with that backend. Keep the interface closed until actual custom consumers demonstrate what extension is needed.

## 7 Admission, authorization and logout

### Fresh admission and exact retry

The synchronous receiver operation has this order:

1. Establish a ready host snapshot, enabled policy and trusted caller/time; preserve required clock observation semantics.

2. Bound and canonically decode claims once. Reject browser/caller mismatch before stable caller/slot lookups.

3. Load bounded facts. If an unchanged live session has the claims fingerprint and request hash, perform current liveness/revocation checks and return its original receipt.

4. Otherwise reject consumed replay, invalid context, disallowed narrowing, login denial, device conflict and capacity before expensive verification where the reference permits.

5. Check the selected proof's shape and protected mechanism; charge the shared weighted budget at the reference charging boundary.

6. Execute the complete verifier. No caller-supplied boolean verification callback.

7. Prepare session/replay/login/displacement/index/accounting changes and the reply. Commit atomically; do no fallible post-commit read.

The exact-retry identity is canonical claims fingerprint plus canonical narrowing request and a still-live current session. It does not require another signature verification or identical certificate-witness bytes. Changed scopes or TTL conflict; reordered unsigned scopes keep their established normalization. Proof-expired exact retries can succeed; lease-expired, revoked or stale-authority retries cannot revive state.

Cheap preflight may reject untrusted claims but must never reserve an account binding, advance a device fence or acquire authority from them.

### Ordinary authorization

Read the caller record once where the backend permits. Validate current authority/key/enrollment generation, lease, login binding/revocation, device fencing, eligibility and required scopes. Run the application projection only after success.

Returned account metadata and admission receipts are facts, not transferable permissions. Recheck after every await before a new privileged mutation or outbound effect. Already dispatched external work follows application reconciliation policy. [IC callback and rollback boundaries](https://docs.internetcomputer.org/guides/security/inter-canister-calls/).

### Logout

Only the authenticated nonanonymous self-authenticating caller selects the target. The request supplies no issuer, subject, credential or deadline. Expired proof/key/enrollment or disabled new admission does not prevent self-revocation when the retained safety state is usable.

Known login: remove the session and retain denial through its original authenticated login deadline, preserving any longer live denial. Return Revoked only after commit. Unknown login: use the separately bounded denial pool and unknown-login budget; return Unconfirmed even on a retry. Its temporary deadline is not proof that all outstanding credentials are excluded.

Logout preserves replay ownership and device fences. Expired leases never free a slot's generation history. Time pruning cannot retire known fences. Any account retirement needs a separately authenticated policy proving old authority cannot return.

## 8 Proposed API and transaction semantics

The following signatures describe responsibilities; they are not a compiled or promised source-compatible API.

```rust
Receiver<B>::admit(host, credential, request) -> AuthResult<AdmissionOutcome>
Receiver<B>::authorize(host, required_scopes, project) -> AuthResult<R>
Receiver<B>::logout(host) -> AuthResult<LogoutResult>
Receiver<B>::prune(host, budget) -> AuthResult<PruneReport>
Receiver<B>::inspect(cursor, limit, scan_budget, observed_time) -> Result<SessionPage, ReadFault>
Receiver<B>::install_authority(host, authenticated_snapshot) -> AuthResult<()>
```

B selects the memory or stable implementation. Host inputs carry actual caller, trusted clock, current protected enrollment, role/eligibility and a serialized transaction context. These inputs are host assertions, not authentication created by a public constructor. Use separate host projections: admission/authorization require current protected authority; logout requires authenticated caller, trusted clock and intact safety state without fresh proof/enrollment; maintenance requires host administrative authority. The library performs no await and cannot choose its own system caller or clock.

AuthResult separates a normal outcome, including typed refusal, from a fatal host-abort condition. A normal refusal can still contain required control-state changes. Endpoint adapters must not convert every refusal into a trap.

| Event | Session/replay/login state | Control state |
| - | - | - |
| Cheap refusal | Unchanged | Retain clock/refill observation where the reference requires |
| Invalid proof after charged verification begins | Unchanged | Retain attempted-work charge |
| Exact live retry | Unchanged | No new verification charge; same-time stable retry need not write |
| Recoverable authorization-record commit refusal | Unchanged | Preserve required attempted-work charges and clock |
| Successful admission/replacement/logout | Atomic affected-record commit | Consistent metadata/budget publication |
| Fatal collection/IO fault or enclosing host transaction abort | Entire transaction rolls back | Same rollback boundary; do not claim charges survive an IC trap |

Malformed signatures and noncanonical scalars can refuse before a charge in the reference. Preserve an explicit failure-stage matrix; “all invalid signatures are charged” is too broad.

Prepare validation, bounded encodings and capacities before publication. Memory applies validated changes without a fallible publication tail. Stable code uses Storage prepared operations within the host atomic boundary. Native failure injection restores the complete backing and discards/reopens poisoned runtime caches.

A projection cannot leak a borrowed guard across await. A copied result does not become later authority. Read-only session access does not imply that a required trusted clock observation is absent; passive inspection, by contrast, must not advance it.

Custom commit callbacks remain an acceptance surface. Inventory consumers before deciding whether to retain an adapter, redesign their API or perform a coordinated Rust cut. Preserve typed callback failures, exact deltas, post-commit counts, no writes on rejected changes and no success before durable completion.

## 9 Backends, budgets and retention

Both volatile and interim stable receivers are required in the first vertical slice. A memory implementation is a real bounded indexed backend, not a full-store clone around every operation. The interim `ic-storage` backend uses host-provided exclusive checked leases and Storage-owned collections. Auth owns neither a memory manager nor an executor. Stable-specific requirements below describe that backend; they do not prescribe the successor's physical representation.

Keep a separate caller lookup from maintenance/replay indexes where the measured workload justifies it. The reference uses three receiver leases and two issuer leases. A different physical layout needs a measured justification and a separately selected format decision.

Open validates schema, protected configuration, counts and required metadata; operations validate selected persisted records and index associations. Do not force memory records through encode/decode to imitate corruption checks. Do not trust stable bytes because a frame decoded structurally.

Receiver proof verification uses one persistent weighted bucket across mechanisms, callers and endpoints. Unknown logout has a separate operation bucket. Both begin empty in the reference. Issuer fresh-intent credit begins at the configured burst. Reopen/restart does not refill any bucket. No partial weighted charge occurs when credit is insufficient.

Physical expired rows count until explicit pruning. Replay quota belongs to the issuer-qualified account, not the changing browser key. Preserve deterministic tie ordering and bounded removal. Retain current public maintenance bounds during compatibility qualification: memory session/replay batches up to 128, persistent issuer pruning up to 32, and inspection up to 32 sessions over 128 visited caller rows plus one lookahead. Extract other backend-specific ceilings from the pinned implementations before claiming full equivalence.

Pruning cannot erase consumed operation watermarks, live replay, unexpired denial or known device generations. Empty inspection pages may still advance their exclusive cursor; callers must not loop internally until a full page appears.

Issuer transitions also need bounded memory and stable implementations under the same intent/recovery contract; volatile loss follows the closed recovery rules, not an automatic fresh start.

### Intended orthogonal-persistence destination

`ic-storage-new` is the selected long-term storage direction, developed concurrently with this rewrite. Its current design selects retained native Rust records/collections, explicit schema conversion and a portable recovery path for incompatible runtime layouts. This is a direction, not an available production API or a dependency pin. The reviewed sibling design is `docs/DESIGN.md` at commit `c86832b405b25dc449af99571c8f25457e21fd80`; its existing M0 implementation and probes do not establish the selected native persistence contract.

Keep logical caller/device/replay/issuer records, bounded decisions and independent conformance cases usable across this transition. Do not turn temporary stable codecs, lease counts or collection operations into permanent Auth abstractions. Build the private operation interface from demonstrated backend needs; do not invent a generic storage framework or speculative OP adapter while the supplier contract is unsettled.

Storage owns retained allocation custody, representation admission and persistence lifecycle mechanics. Auth owns session/replay/revocation invariants, issuer uncertainty and application-state conversion requirements. The host owns transaction rollback, readiness and certification-root publication. An OP access scope is not a transaction: ordinary `Err` can retain earlier writes. Complete expected-refusal checks before identity mutation and qualify fatal partial mutation through the actual host rollback boundary; preserve the distinct clock and attempted-work behavior in section 8.

Adopting OP requires the same semantic suite plus installed evidence for compatible retained-state upgrades, incompatible-layout refusal, explicit schema conversion, checkpoint recovery and failed publication/rollback. All boot/readiness, replay, device-fence and issuer recovery rules remain in force. Retained bytes never establish fresh enrollment or repair missing revocation evidence; execution handles, guards and pending futures are not durable Auth state.

The interim stable backend is an implementation and comparison step, not a commitment to deploy it or migrate live users through it. Select exact supplier artifacts and the required state-transfer or legacy-refusal disposition at the eventual consumer cut. This design authorizes neither a production migration nor reinterpreting old stable records as retained native objects.

## 10 Issuer intent and recovery

Keep one issuer operation owner, with states for reserved intent, retained nonce/approval, authorized signing input, pending external outcome and completed evidence. Exact internal encoding is new design work; preserve all current operation identities, deadlines and recovery behavior.

Proposed operations are prepare_issue, select_renewal, prepare_renewal, record_nonce, record_approval, signing_input, complete, reconcile, revoke and prune. Their arguments distinguish authenticated account, authenticated browser, protected signing authority and untrusted request data.

Initial issuance requires account authentication. Renewal uses the retained browser binding and preserves its original window, slot and generation. Application grant approval remains host-owned and is rechecked after any await. Read-only renewal selection reserves neither capacity nor signing authority.

Reserve completed-result capacity before randomness, approval or signing. Commit the exact intent and effect identity before external dispatch. A lost response remains pending/unknown; reconciliation is lookup-only unless a qualified effect adapter proves that resuming the same operation is safe. Missing/pruned evidence cannot authorize replacement issuance.

Before accepting a signing completion, recheck boot/operation epoch, authority, revocation, original binding and deadlines. A stale callback cannot publish. Repeated completion/retrieval returns retained evidence, including expired evidence where the contract allows; it does not reauthorize a new credential.

Exactness has three owners:

- Issuer retains exact canonical claims, nonce, original deadlines and Ed25519 signature where applicable.

- Certified completion retains claims and commits the signature leaf with the host's certification owner; current witness/certificate assembly remains host-owned.

- Browser commits the complete credential frame it received as the exact evidence for its sequence.

Do not promise that every reconstructed certified witness is byte-identical. Do not use a changed witness to change claims or credential identity. [Issuer result representation](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/persistent/issuer/model.rs).

Renewal sequence watermarks and operation epochs prevent recreation after result pruning. Bound operations globally and per account, reserve bytes for completion and prevent login reclamation while operations reference it. On binding-state loss, disable old bindings and require new account authentication and a fresh browser key.

No production wallet issuer, signing service or secret-key store is selected. Existing signature/delegation/batch helper APIs receive an individual consumer/disposition review; a receiver-focused rewrite does not authorize deleting independently used primitives.

## 11 Browser lifecycle

Keep one coordinator over one strict journal owner. Inject Platform SDK identities, issuer transport, receiver transport and clock mapping. Do not recreate a second SDK identity facade or implement application retries.

Retain distinct account and browser signer roles. Initial issuance/reconciliation uses the account identity; renewal and subsequent reconciliation use the browser binding. Receiver calls use only the browser identity. Capture destination and principal bytes before asynchronous work and bind temporary signers to their target, generation and attempt.

Persist issuance intent before dispatch. Pending sequences reconcile; completed sequences reuse retained evidence. Store completion before receiver admission. Record each receiver in the logout inventory before contacting it.

Preserve the current private journal profile ic-auth-login-cleanup-journal-v1 unless a separately authorized browser migration changes it. Metadata and exact credential evidence commit atomically, while ordinary guards load bounded metadata only. Unknown issuance remains authoritative even when its recoverable sequence is absent.

Logout commits durable local invalidation before waiting for any admission flight or sending cleanup. Fence late signatures, completions and replies; already returned signatures or accepted requests cannot be recalled. A storage failure means durable local logout was not established.

Preserve the reference limits: 64 destinations including issuer, eight attempts per destination, 64 outstanding admission/readiness operations per coordinator, five-second network attempts and a fixed cleanup deadline. This bounds cooperating coordinator work, not all tabs or hostile scripts.

Only authenticated Revoked replies confirm a destination. Unconfirmed, capacity refusal, timeout and HTTP acceptance do not. Expiry of local cleanup is not evidence of remote expiry. Keep unknown issuance and unconfirmed destinations visible.

Readiness may perform the reference's bounded preparation/retry sequence, at most two attempts under its specified authenticated expiry conditions. Explicit establish remains forced admission without that expiry retry. Never replay an application mutation after an uncertain reply. Preserve injected conservative clock mapping; infer no issuer/receiver/browser offset.

Require real SDK transport and native browser IndexedDB qualification, including two tabs, strict durability, renderer interruption and suspended signing. Fake storage tests are a separate layer.

## 12 Boot, authority and lost state

Every install, upgrade and recovery starts authentication closed. Checked retained safety state and a fresh authenticated enrollment response bound to receiver and boot challenge are both required for readiness.

Snapshots retain monotonic revision/generation and terminal key revocation. Exact repeat revision/digest is idempotent; lower revision or same revision with changed content refuses. Authority generation changes invalidate prior sessions atomically. Freshness is at most the reference's 3,600 seconds and any tighter approval deadline; refreshing it cannot extend earlier leases.

Complete persistent reopen is distinct from volatile loss or stale backup restoration. Missing replay, denial, device or authority evidence cannot be replaced with an empty store, local timestamp, bumped local generation or fresh enrollment. Stay closed pending trusted complete recovery or a separately authorized and qualified fence that excludes every old credential.

No finite clock-skew assumption makes unknown logout conclusive. An issuer-ahead credential may remain capable of later admission. Preserve the reference's explicit Unconfirmed outcome and lost-state refusal.

Ordinary admission may be disabled while safe self-revocation remains available. Missing safety state must not report successful revocation. Authority refresh scheduling belongs to the host; Auth supplies validity rules, not another timer/executor.

## 13 Deletions, reuse and compatibility policy

| Candidate | Disposition |
| - | - |
| Stable-to-memory mini-ledger reconstruction | Replace with bounded operation facts and shared pure decisions |
| Parallel receiver orchestration and redundant record projection | Remove only after the common operation trace covers both backends |
| Full-store cloning, mirrors or scans introduced for convenience | Prohibited on ordinary calls |
| Duplicate canonical bytes, request hashes or browser codecs | One authoritative owner; retain independent vectors |
| Separate policy for Ed25519/certified or volatile/stable | Prohibited |
| Retired token/session flows and historical compatibility readers | Do not reintroduce |
| Existing independently used verifier/signature helpers | Retain or relocate only after consumer inventory |
| Custom commit/restore API | Decision pending actual downstream review |
| Release scripts, npm publication and deployment framework | Do not transplant into the implementation scaffold |
| Tests for uniquely required guarantees | Retain/re-express independently; delete only redundant coverage with a recorded mapping |

Reuse of a sound primitive is permitted with a named invariant, caller, dependency review and evidence. Copying an existing package is not a completed design milestone.

Wire and signed bytes remain fixed. Stable records do not become compatible merely because wire vectors pass. Before opening existing backing, choose either exact current-record compatibility or a new format that refuses legacy backing. A hard cut requires explicit consumer authority and a recovery/data disposition plan; this design supplies none.

Do not reuse the current schema identifiers for different records. Do not erase browser journals or issuer pending work to simplify the cut. Old production state remains owned by its existing host until an authorized transition is qualified.

The first deliverable is one complete receiver slice on both backends, with independent proof vectors and failure traces. Expansion follows the qualification gates, not a target line count.

# Implementation and qualification

Status: implementation sequence, 15 November 2026. Companion to the architecture above. Passing a stage authorizes the next implementation slice, not consumer deployment. The [scope/request slice](../development/request-boundary.md) and [canonical claims codec](../development/claims-boundary.md) have focused evidence; no complete stage or replacement gate is claimed from these slices.

## 1 Freeze the reference and contract inventory

Use commit e52a40be75dd9809d0528189d84b43fe96952ad9 of dragginzgame/ic-auth. Resolve files from committed objects, not a sibling working tree. The repository's reference.sha256 records selected reference-file bytes, including the initial request implementation and retained vector source. This is an integrity inventory, not test results or a complete repository archive; retained fixture provenance lives beside the tests.

Before implementation, retain:

- Canonical claims, signing-domain and session-request hash vectors with independent expected bytes.

- Wire method IDs, complete nested field/tag order, all four resource limits and generated TypeScript output.

- Receiver/session/replay/login and issuer stored-format identifiers and selected record fixtures.

- Browser journal profile, completion identity, refusal codes and recovery traces.

- A named obligation for every unique reference contract test, including negative backend drivers.

- Actual downstream call sites for custom SessionStore/commit/restore APIs, standalone proof/signature helpers and browser exports.

The current source's receiver schema is 0x41555448525304; issuer schema is 0x41555448495303. These identifiers are reference facts, not names available for a different replacement encoding.

The source identifies Canic integration and a private browser package, but those documents do not establish all current downstream callers. Do not infer “unused” from an absent local reference. Record each consumer's package/source identity, feature set, API use, retained state, pending issuer work and browser journals. Unavailable consumers leave an API-removal or cutover gate open, not a reason to stop independent implementation.

Produce a disposition table: preserved contract, newly implemented owner, independent acceptance case, proposed API change, required downstream decision. Do not port old engine code merely to make this table green.

## 2 Build the protocol boundary first

Implement passive domain types, the signed codec and exact Wire adapter before storage orchestration.

Required evidence:

- Retained golden claims and digest values, independently constructed without the replacement encoder.

- Every signed field affects the digest; strict role/scope ordering, duplicates, malformed principals, wrong schema/mechanism, nonminimal/trailing input and length arithmetic refuse.

- Unsigned session-scope reordering retains the old request hash; duplicates refuse; None TTL and explicit default differ.

- Old/new cross-decoding and exact bytes for every method, response/error variant and nested proof variant.

- Separate below/at/above byte, allocation, node and depth cases. A byte-only test does not cover traversal or allocation admission.

- Browser full-frame caps remain independent of native budgets; buffers/subarrays detach; u64 remains bigint.

- Regeneration matches all four canonical TypeScript codecs and their selected Wire runtime, with no local patch.

- Retired method identities and token/schema profiles refuse without fallback.

Keep the old implementation only in independent qualification consumers/processes. No product crate may depend on it. A round trip through one new encoder/decoder is not independent evidence.

## 3 Prove complete verification

Qualify both enrolled Ed25519 and certified proofs before calling either a replacement.

Ed25519 cases include valid signatures, wrong digest/key, malformed length, noncanonical S, small-order key/R, cross-domain replay, wrong caller and feature-unified dependencies that otherwise enable compatibility behavior.

Certified cases include complete Root/delegation and issuer-canister verification, wrong canister/seed/key binding, wrong network root, audience or grants, malformed CBOR/tree/path material, stale/future certificates and independently exhausted proof-work bounds.

The fresh certified aggregate variable-material cap is at most 131,072 bytes and may be tightened by protected configuration. Also preserve separate issuer signature size, witness-step, certificate age and lifetime limits. The outer 524,288-byte frame limit does not waive these.

For each failure, record its stage, public projection, whether cryptography ran, whether credit changed and whether any identity state changed. In particular, scalar/shape rejection before charging differs from failed charged verification.

Test protected verifier selection: request-supplied algorithm/key material cannot choose enrollment, and standalone VerifiedClaims cannot be constructed from decoded claims or used as a session. Preflight must reserve no unverified binding.

## 4 First complete receiver slice

Implement the same narrow application scenario on both memory and actual stable collections:

1. Two issuer-qualified accounts, two browser devices and two independent receivers.

2. Valid Ed25519 admission, proof-free authorization and a compact receipt.

3. Exact retry after proof expiry but before lease expiry, returning the unchanged receipt.

4. Request conflict, invalid signature and wrong caller with their expected charging/write behavior.

5. Known logout, unknown logout and admission/logout orderings.

6. Higher-generation replacement of one device while the other stays authorized.

7. Fresh renewal clamped to the original absolute login deadline.

8. Stable reopen with fresh authority readiness and preserved replay/device fences.

Use a private operation-level storage interface only after implementing the real needs of both backends. Compare facts and transitions directly; do not make the stable implementation emulate a complete memory store.

Exit gate: one admission pipeline, no temporary reconstructed login ledger, no global read/clone, one current authorization rule and equivalent accepted/refused behavior on both backends. A consolidation that increases transformations or materially worsens ordinary lookup costs must be revised.

This gate qualifies the interim memory/`ic-storage` slice, not completion of the intended OP transition. Reuse its independent behavioral cases against `ic-storage-new` when that supplier is ready, adding the lifecycle and transaction proofs specified in architecture section 9. Protocol, proof, receiver-policy and issuer/browser work can proceed without inventing the successor's API.

Then repeat complete receiver scenarios using certified credentials. Neither mechanism is optional for replacement acceptance.

## 5 Failure and atomicity matrix

Inject failures at preparation, each staged mutation, backend publication and enclosing host commit.

| Boundary | Required observation |
| - | - |
| Shape/context/replay/capacity refusal | No unauthorized session, replay consumption, slot replacement or denial reservation |
| Charged invalid proof | No identity changes; attempted-work credit remains spent on normal return |
| Recoverable record commit refusal | All identity records/indexes/account totals unchanged; expected operational metadata retained |
| Failure after partial backing mutation | Host rollback restores the entire backing; stale runtime caches are discarded |
| Exact retry at saturated capacity | Original live receipt succeeds without new proof credit or record capacity |
| Replacement commit fails | Displaced device still works; new device has no installed authority |
| Logout commit fails | No Revoked acknowledgement; prior session/replay/denial state remains consistent |
| Pruning/generation commit fails | Original records, counters, authority association and fences survive |
| Clock rollback/overflow | Typed refusal; no wrapped deadline, generation or fresh allowance |
| Same-time exhaustion or exact retry | Preserve the reference's zero additional stable-write behavior where applicable |

Distinguish normal Auth refusal from failure of the enclosing transaction. An IC trap restores message state, including attempted-work charges made in that message; do not manufacture a stronger guarantee. A host that traps on ordinary invalid proofs defeats the intended charging policy.

Run conformance against an independently implemented encoded-record adapter and a deliberately broken write-then-error adapter. The negative adapter must fail. Native snapshot restoration is test machinery, not a product journal or proof of installed IC rollback.

Maintain a small independent state model for sequences of admission, retry, renewal, logout, replacement, authority changes and prune. Compare observable outcomes, retained fences and quota ownership; do not merely mirror production function structure.

## 6 Bounded state, corruption and loss

Populate the selected capacities and exercise account-wide replay saturation across device churn, logout, reopen and failed pruning. Unknown callers must not consume admitted-login logout reservations. Every retained count and encoded-byte total must agree with its changes.

Stable corruption cases cover header/configuration, selected caller records, replay owner, slot association, counts and expiry indexes. Opening/reading malformed or foreign state must refuse without resetting or fabricating missing reservations. Native memory invariants need direct structural tests, not serialized corruption machinery.

Exercise empty/full maintenance batches and equal-deadline ordering. Prove bounded visited work, not only bounded returned results. Known device fences survive session expiry and cleanup. Inspection advances its exclusive cursor over fences, may return empty pages and does not claim a cross-call snapshot.

Installed lifecycle cases must cover:

- Complete stable reopen followed by fresh authenticated boot enrollment.

- Volatile state loss plus issuer-ahead credentials and fresh enrollment: remaining closed.

- Stale/incomplete backup or storage mode switch: remaining closed.

- Old authority after key revocation, revision regression, equal revision with changed bytes and stale boot challenge.

- Enrollment outage through expiry and bounded host refresh attempts.

- Snapshot restoration as a separate recovery boundary, without reviving old authority.

These cases require actual host integration. A native map test cannot establish caller/time provenance, durable custody or rollback protection.

## 7 Issuer and external-effect recovery

Implement issuer intent rules independently of a production signing service. Use a deterministic test effect driver and qualified Platform primitives.

Required cases:

- Account-authenticated initial binding and browser-authenticated renewal with immutable login window.

- Device exhaustion, higher-generation replacement and overflow refusal.

- Completed-result byte capacity reserved before effects.

- Receiver and issuer budget initialization differences; persistent reopen never refills implicitly.

- Per-account/global intent limits, including fresh IDs from one account.

- Pause before and after nonce, approval, external dispatch and completion; restart and reconcile the original operation.

- Logout, authority change or boot change while signing is pending: late publication refuses.

- Lost signing response: unknown remains unknown; missing evidence cannot trigger a replacement effect.

- Exact completed retrieval after expiry/revocation where retained, without renewing authority.

- Pruned result plus retained sequence/epoch watermark: consumed work cannot be recreated.

- Certified completion and signature-leaf publication in one host transaction, with current witness assembly kept distinct from exact retained claims.

Qualify issuer policies over bounded memory state and stable state. Volatile loss disables old bindings; it is not a recovery implementation. No browser-key-only re-creation of a login after loss.

Before a real signer is admitted, specify its operation identity, response authentication, reconciliation capability and behavior after an unknown result. A transport retry policy is not an external-effect recovery proof.

## 8 Browser qualification

Retain public browser behavior through the implementation cut. Test the coordinator through its package exports with injected issuer boundaries, generated codecs and real SDK identities.

Then run actual browser/native IndexedDB scenarios:

- Strict atomic journal/evidence commit, aborted queued writes and refused insufficient durability.

- Two tabs contending for one issuance sequence, receiver admission and logout generation.

- Renderer interruption with pending issuance, credential completion, admission or cleanup lease.

- Reopen with an expired completed credential versus unresolved issuance; only the former can progress under the specified renewal rules.

- Local invalidation before waiting for network/admission; late signatures and completions fenced.

- Captured destination/principal mutation attempts cannot redirect a signer.

- All destination acknowledgements authenticated; Unconfirmed/timeout/capacity never reported as Revoked.

- Unknown issuance remains after cleanup deadline; original deadlines and attempt counts do not restart.

- 64-operation coordinator bound, 64-destination inventory and eight-attempt limits.

- At most two readiness preparation attempts under the exact allowed expiry conditions; no application mutation replay.

- Byte-for-byte Rust/TypeScript interoperability for both proof variants and every refusal.

Keep fake IndexedDB/Node, real browser, SDK-certified receiver and production issuer/consumer evidence separately identified. A passing subset cannot stand in for the others.

## 9 Dependency and performance gates

Qualify standalone passive types, Wire codec-only, memory receiver, stable receiver, each verifier, both verifiers together and issuer consumers. Inspect Cargo feature unification and actual native/Wasm graphs. Preserve one Platform Types identity and one host executor/guard. Receiver consumers must not pull issuer signing custody, native tooling or TypeScript metadata generation.

Record exact source, locks, rustc, target, profiles, optimizer, final Wasm and simulator/server identities. Keep original and replacement measurements under matched conditions in isolated targets.

Measure:

- Total uncompressed Wasm and code-section bytes.

- Fresh admission and ordinary authorization separately.

- Decode, proof verification, bookkeeping and complete endpoint costs.

- Cold/populated/post-upgrade lookup, memory, allocations and stable IO.

- Exact retry, invalid proof, wrong caller, exhaustion and unknown logout floods.

- Prune/inspection work and issuer recovery/renewal, including refusal costs.

The reference handoff reports 1,224,808 instructions for authentication at 100,000 receiver records and 1,223,895 after upgrade. Retain the accepted 1,300,000-instruction regression ceiling for the matched installed scenario. These are published source reports, not measurements rerun for this review. [Pinned handoff](https://github.com/dragginzgame/ic-auth/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/docs/status/current.md).

Extract and retain the other accepted verifier/fixture ceilings before comparison. Historical named-symbol attribution is not removable Wasm size. The handoff explicitly leaves populated persistent failure-cost and production fleet budget qualification incomplete; set and justify those missing operational limits rather than declaring historical success.

No line-count, size or speed target can waive replay, revocation, rollback or uncertainty guarantees.

## 10 Consumer cut and completion criteria

Select a consumer only after its relevant gates pass. Its review must decide:

1. Rust API changes, including custom commit callbacks and independent signature helpers.

2. Exact supplier/package identities and coordinated receiver/browser artifacts.

3. The selected persistence supplier and existing-state disposition: exact stable-format preservation, an explicitly authorized conversion to OP, or a separately authorized hard cut with legacy refusal.

4. Browser journal/key custody, pending issuer work and recovery evidence.

5. Authority refresh/readiness, host rollback, certification and application rechecks.

6. Operational rollback and external-effect reconciliation after the cut.

Keep shadow comparison observational: no double signing, double replay consumption or duplicated business effects. Use isolated fixtures for destructive failures.

A hard cut does not mean reinstall by default. No local document authorizes erasing host data. An incompatible backing must refuse; any importer/reset/reinstall needs the consumer's explicit disposition and installed recovery proof. Restoring an old binary alone does not restore compatible data or undo external effects.

Completion requires a consumer contract inventory, independent vectors, both proof mechanisms, both backends, issuer/browser interruption evidence, actual Wasm lifecycle results and accepted costs. Each claimed simplification lists the surviving owner, deleted path and invariant coverage.

Report interim Auth qualification and OP adoption separately. Completion of the long-term persistence direction additionally requires qualified `ic-storage-new` integration and the selected consumer state/recovery disposition; successful `ic-storage` qualification alone does not establish that outcome.

Only after a qualified replacement and consumer decision exist should release/delivery machinery be integrated through the shared owners. Do not reproduce the old release system as an initialization task.
