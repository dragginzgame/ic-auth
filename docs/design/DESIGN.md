# Architecture

Status: stable checkpoint architecture. The receiver, issuer and browser owners below are implemented; the qualification sequence retains acceptance obligations, not a list of unimplemented features. See [current status](../status/current.md) for evidence and remaining consumer boundaries. This design does not authorize consumer cutover, data deletion, a new issuer service or a storage-format change.

Current compatibility decision (16 November 2026, Auth #2): the maintainer
authorized a hard cut from the pre-recode implementation and confirmed no existing
users/deployments need its Wire/API compatibility. Historical source links below
retain provenance, not executable-oracle or parity requirements. This repository
owns one current contract, independently qualified with its Rust/browser consumers.
The current layout is retained by this cleanup without promising historical
identities forever. No existing live data is migrated or deleted.

## 1 Decision

Every stateful Auth owner uses a stable upgrade checkpoint. The host selects the
issuer and receiver roles independently and provides a dedicated lease for each.
Ordinary operations use bounded heap indexes. The private browser coordinator
owns browser admission and logout.

The application flow remains:

**Portable credential → receiver-local admission → caller-bound session → current authorization on each ordinary call.**

A receiver does not call an issuer or repeat signature verification on every application operation. It checks its retained session against current protected authority, time, revocation, device generation and required scopes. Application account linking and resource permissions remain outside Auth.

The architectural change is to make a complete receiver operation the unit of ownership. Session, replay, login denial, device replacement and accounting become one bounded transition. A stable backend should not have to reconstruct a temporary memory ledger to reuse policy. The current memory owner assembles bounded facts for private decision functions and owns publication. A future durable implementation must demonstrate its needs before any shared backend interface is introduced.

This is a fresh implementation of selected responsibilities, not a port of the old module tree. The existing engine already shares important checks; a proposed consolidation earns its place by removing a transformation or independent invariant without losing refusal, recovery or cost guarantees.

## 2 Historical reference and current scope

Reference: [dragginzgame/ic-auth-legacy at e52a40be75dd9809d0528189d84b43fe96952ad9](https://github.com/dragginzgame/ic-auth-legacy/tree/e52a40be75dd9809d0528189d84b43fe96952ad9). Public main resolved to that same commit during this review. Its commit message is “on hold.”

| Verified source fact | Design consequence |
| - | - |
| Workspace selects Engine 0.41.1, Protocol types/Wire 0.40.1 and Rust 1.100.0 | Select the current supplier graph and qualify its actual boundaries |
| Complete Ed25519 and certified verifiers feed one session engine | Keep one admission policy; do not introduce mechanism-specific sessions |
| Stable receiver reconstructs at most two selected login records into a memory ledger | Keep bounded receiver facts; qualify durable behavior independently at the future milestone |
| Stable caller records already combine an active session or retained fence | Preserve this useful locality; do not create a duplicate session mirror |
| Persistent issuer owns intents, renewal indexes and completed results | Replace its lifecycle deliberately, not with a stateless signing callback |
| Browser package is private and includes durable admission/logout coordination | Browser uncertainty handling remains product scope |
| Custom session-store commit/restore APIs are supported alongside the stable backend | Historical APIs impose no retention requirement under the authorized hard cut |

Sources: [manifest](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/Cargo.toml), [engine](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/session/mod.rs), [stable receiver](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/persistent/receiver/mod.rs), [persistent integration](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/docs/integration/persistent-auth.md), [browser contract](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/docs/protocol/browser-client.md).

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

Retain three public packages: `ic-auth-protocol-types`, `ic-auth-wire` and `ic-auth`. The `issuer-stable` and `receiver-stable` features own separate modules in `ic-auth`; the issuer builds without receiver/verifier features, and the receiver does not depend on issuer state. Signing custody remains host-owned. No fourth issuer package is justified by current consumers. Qualify standalone and feature-unified graphs.

Keep receiver implementations as modules behind explicit capabilities, not one package per table or operation. Future persistence mechanics stay backend-local; do not expose them through protocol types, shared Auth decisions or mandatory receiver API bounds. Platform supplies cryptographic primitives and canonical Principal identity. Keep metadata generation, Host, Build, Testkit and release tooling out of deployed receiver dependencies.

Public signatures serve actual current consumers. Pre-recode package/API/Wire parity is not an acceptance gate; live-data custody and future delivery remain separate decisions.

## 4 Current protocol contracts

### Identity and authority

A fresh Ed25519 browser key identifies one login. Derive its self-authenticating Principal from the standard DER public key and compare it with the authenticated ingress caller. A delegated signature whose sender remains a different account Principal does not satisfy that binding. [IC authentication](https://docs.internetcomputer.org/references/ic-interface-spec/https-interface/#authentication).

Accounts are qualified by (issuer, subject) inside one protected AudienceId. Device slots are issuer-assigned 0–7, with nonzero monotonically increasing u64 generations. Renewal preserves browser key, account, slot, generation, login start and absolute deadline. A higher generation can replace a slot only after complete proof verification. Overflow refuses.

A request may name an issuer/key as a lookup hint; it cannot install authority, choose a public key, relax a verifier or supply trusted time. One protected role selects one signed grant. Never combine scopes from different roles.

### Canonical bytes and request identity

Preserve claims schema 2 and the exact domain IC-AUTH-PORTABLE-CREDENTIAL followed by a zero byte. Preserve field order: schema, mechanism, network, application, issuer, key binding, browser key, account subject, slot, device generation, login start, login deadline, issuance, admission expiry, nonce, grants. Fixed-width integers use big-endian encoding; existing length/count prefixes and principal/label encodings remain unchanged. Schema 3 adds a four-byte big-endian length and up to 4096 opaque extension bytes immediately after the account subject. Issuers use schema 2 for an absent extension and schema 3 for a present extension, including an empty one. Both proof mechanisms cover the complete record.

Roles and signed scopes must arrive strictly ordered and unique. Reject noncanonical signed inputs; do not sort them into validity. Preserve all other current signed domains used by delegation/certificate helpers.

Unsigned session requests are different: their existing constructor sorts unique requested scopes. Preserve that normalization, the exact canic-application-session-request-v1 hash framing and the distinction between absent TTL and an explicitly requested default TTL. The historical domain text is a byte contract, not ownership of Canic policy.

The replay fingerprint is SHA-256 of exact claims bytes. Alternate proof bytes do not create another credential identity. Ed25519 verifies that digest using ordinary strict Ed25519, not Ed25519ph. Preserve small-order rejection and canonical scalar checking even under dependency feature unification. Certified verification preserves its separate ic-auth-portable-credential framing, complete delegation/issuer checks, trusted IC root and zero future-skew allowance.

Sources: [signed protocol](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/docs/design/0.6-auth-protocol.md), [request identity](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/session_request.rs), [strict verifier](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/portable_signature.rs).

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

Fresh claims require login_start ≤ issued_at ≤ now < admission_expiry ≤ login_deadline, with all original duration and enrollment checks. A session lease is capped by requested/protected TTL, the login deadline and issuer authority expiry. Current enrollment freshness and generation are checked independently at admission and authorization.

Proof expiry stops fresh admission. Enrollment expiry suspends authorization. A same-generation refresh restores access for an otherwise live session through its original deadline. NotYetValid is a refusal/retry hint, never evidence of validity. No safety-critical finite clock-skew bound is assumed.

Completed issuer evidence retains its original accepted structural recovery rules. Do not turn the newer fresh-grant ceiling into permission to discard already completed results.

## 5 Exact Wire surface

These are the current method identities, DTO order, tags and error projections, qualified together with the browser codecs. They are not immutable historical bindings. Hosts own actual exports; direction checks and bounded admission remain required.

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

Generate browser codecs from the Rust-owned schema through Wire. No local runtime fork or handwritten parallel codec. Canonical signed claims are opaque bytes in this transport. [Wire definitions](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth-wire/src/lib.rs), [nested models](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth-wire/src/model.rs).

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

Private receiver policy consumes bounded session/login snapshots and admission footprints. `MemoryReceiver` owns indexed records, prepared session/replay values and publication. No universal `ReceiverFacts`/`ReceiverChange` protocol is required. Decisions and receipts cannot establish authority outside the host transaction; reads must recheck current host inputs before protected effects.

Both storage options use the same indexed heap between upgrades. The stable owners add admission of a complete checkpoint in the upgrade transaction. Keep semantic rules reusable without a public key/value store, repository framework or arbitrary transaction language.

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

Logout preserves replay ownership and device fences. A known slot remains protected through the maximum login lifetime from its generation’s start and any longer retained denial or predecessor deadline. Bounded pruning then releases it.

## 8 API ownership and transaction semantics

`MemoryReceiver` owns admission, `read`/`authorize`, logout, bounded pruning and
`inspect_sessions`. Enrollment uses `request_enrollment`/`install_enrollment`.
The public definitions in `crates/ic-auth/src/receiver_memory` own exact signatures
and typed errors; there is no generic `Receiver<B>` or common backend result API.

Host inputs carry actual caller, trusted clock, current protected enrollment, role/eligibility and a serialized transaction context. These inputs are host assertions, not authentication created by a public constructor. Use separate host projections: admission/authorization require current protected authority; logout requires authenticated caller, trusted clock and intact safety state without fresh proof/enrollment; maintenance requires host administrative authority. The library performs no await and cannot choose its own system caller or clock.

Typed operation results describe normal success/refusal; host traps are a separate transaction boundary. A normal refusal can still contain required control-state changes. Endpoint adapters must not convert every refusal into a trap.

| Event | Session/replay/login state | Control state |
| - | - | - |
| Cheap refusal | Unchanged | Retain clock/refill observation where the reference requires |
| Invalid proof after charged verification begins | Unchanged | Retain attempted-work charge |
| Exact live retry | Unchanged | No new verification charge; same-time stable retry need not write |
| Recoverable authorization-record commit refusal | Unchanged | Preserve required attempted-work charges and clock |
| Successful admission/replacement/logout | Atomic affected-record commit | Consistent metadata/budget publication |
| Fatal collection/IO fault or enclosing host transaction abort | Entire transaction rolls back | Same rollback boundary; do not claim charges survive an IC trap |

Malformed signatures and noncanonical scalars can refuse before a charge in the reference. Preserve an explicit failure-stage matrix; “all invalid signatures are charged” is too broad.

Prepare validation, bounded encodings and capacities before publication. Memory publishes prepared changes with closure on interrupted publication; the host owns transaction rollback. Future stable code must qualify prepared storage operations within that boundary. Native failure injection alone cannot establish installed rollback or durable cache recovery.

A projection cannot leak a borrowed guard across await. A copied result does not become later authority. Read-only session access does not imply that a required trusted clock observation is absent; passive inspection, by contrast, must not advance it.

The pre-recode custom commit-callback API need not be retained. Current host transaction boundaries must independently prove their behavior. Preserve typed callback failures, exact deltas, post-commit counts, no writes on rejected changes and no success before durable completion.

## 9 Backends, budgets and retention

The first vertical slice requires a real bounded indexed memory backend, not a full-store clone around every operation. Stable checkpoint owners use host-governed leases from published `ic-storage`. Stable-specific requirements below retain the reference's safety obligations without selecting the future supplier's physical representation.

Keep a separate caller lookup from maintenance/replay indexes where the measured workload justifies it. The reference uses three receiver leases and two issuer leases; this is reference evidence, not a required layout for the future backend. Checkpoint mode uses one allocation per Auth owner.

Open validates schema, protected configuration, counts and required metadata; operations validate selected persisted records and index associations. Do not force memory records through encode/decode to imitate corruption checks. Do not trust stable bytes because a frame decoded structurally.

Receiver proof verification uses one persistent weighted bucket across mechanisms, callers and endpoints. Unknown logout has a separate operation bucket. Both begin empty in the reference. Issuer fresh-intent credit begins at the configured burst and is charged once when a new intent is retained. Reopen/restart does not refill any bucket. No partial weighted charge occurs when credit is insufficient.

Physical expired rows count until explicit pruning. Replay quota belongs to the issuer-qualified account, not the changing browser key. Preserve deterministic tie ordering and bounded removal. Qualify current public maintenance bounds: memory session/replay batches up to 128, persistent issuer pruning up to 32, and inspection up to 32 sessions over 128 visited caller rows plus one lookahead. Select future backend-specific ceilings from its measured requirements; historical equivalence is not required.

The receiver groups replay counts and device-slot owners in one row per issuer-qualified account. Each row holds up to eight slot–caller pairs and survives replay pruning while any device slot is retained. Reopen reconstructs these indexes from the caller and replay records. The issuer keeps each account’s logins in an identity-sorted vector of up to eight entries and encodes them as the v1 identity-to-login map.

Pruning cannot erase consumed operation watermarks, live replay, unexpired denial or known device generations. Empty inspection pages may still advance their exclusive cursor; callers must not loop internally until a full page appears.

Issuer transitions retain bounded heap state between upgrades. A missing checkpoint follows the closed recovery rules.

### Stable upgrade checkpoints

Select `receiver-stable` with a verifier or `issuer-stable` independently.
`StableReceiver` and `StableIssuer` store separate v1 checkpoints in an exclusive
named `ic_storage::RuntimeMemory` lease. Ordinary operations on `state()` use
the indexed heap records and the host’s IC message transaction.

1. On a genuinely fresh installation, call `new_fresh(lease, config, now_ns)`
   and perform normal enrollment. Creation requires a zero-page allocation.
2. In pre-upgrade, call `seal(canister_version)` on each stable owner. Seal writes a bounded
   image and closes that instance. Commit the host's challenge generator,
   authority entries and terminal revocations, pending effect identities and
   certification state in the same upgrade transaction. Trap on any failure.
3. In post-upgrade, call `reopen(lease, config, now_ns, canister_version)` with the original
   protected configuration, optionally raising the receiver’s global session, replay, encoded-byte or retained-device-slot ceilings within its checkpoint budget. Reopen checks the version-specific SHA-256 digest,
   decodes the complete image with Wire resource limits, checks indexes and
   accounting and consumes the checkpoint. A skipped pre-upgrade hook, malformed
   image, foreign configuration or regressing clock cannot create fresh state.
   Trap on refusal so the IC restores the old module, heap and stable memory.
4. Restore host-owned state and republish the certification root. Obtain a
   fresh authenticated enrollment with a new challenge and revision before
   serving receiver calls. A still-valid session can then resume under the
   current authority. Issuer restart advances the boot and epoch fences.
   Completed results and unresolved effect identities remain available for
   reconciliation.

Read `ic0.canister_version` separately inside each upgrade hook. Seal binds the
checkpoint digest to the next version using checked addition. Reopen uses the
actual post-upgrade version. An intervening install fences both owners even if
its module never calls Auth. Any version gap refuses the upgrade. Pre-cut
checkpoint images are refused. Use `new_fresh` only during a genuinely fresh install. A missing image in post-upgrade must leave Auth closed.

Sealing also commits the issuer's per-login renewal states and the receiver's
caller/fingerprint/application-extension associations before encoding. Each owner
adds one 32-byte commitment to its payload. Reopen verifies those commitments
during record validation, checks issuer operation deadlines, renewal indexes and
unique login generations, and remeasures each receiver session. These checks
detect disagreement between independently stored fields under exclusive host custody.
The issuer uses an 8 MiB image. The receiver uses a 128 MiB image with up to 256
caller or replay rows per Wire frame. It stores the authoritative rows and
rebuilds the slot, expiry and account indexes during reopen. The receiver checks
configuration against the image ceiling, reserving for caller rows and framing
on top of the session and replay budget. It recomputes row charges on reopen. Independent chunk
limits bound Wire traversal and allocation admission. Qualify the expected
workload against the whole upgrade's instructions, heap use and encoded size.
The digest detects accidental corruption under exclusive host custody. Hosts
must also protect against restoring an obsolete whole-canister backup.

The first stable installation requires intact authority and safety state. An
already-lost volatile receiver cannot recover its sessions or fences. Images
from the pre-recode implementation have no reader in this contract.

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

Do not promise that every reconstructed certified witness is byte-identical. Do not use a changed witness to change claims or credential identity. [Issuer result representation](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/crates/ic-auth/src/persistent/issuer/model.rs).

Renewal sequence watermarks and operation epochs prevent recreation after result pruning. Bound operations globally and per account, reserve bytes for completion and prevent login reclamation while operations reference it. On binding-state loss, disable old bindings and require new account authentication and a fresh browser key.

No production wallet issuer, signing service or secret-key store is selected. Retain only signature/delegation/batch helpers needed by the current contract and its consumers.

### First volatile issuer slice

The initial implementation boundary is one bounded issuer operation owner, separate
from receiver dependencies. The reference `persistent/issuer/model.rs::Operation`
retains nonce, approval and result as separate optional fields. The replacement
encodes the permitted progress explicitly in a private phase enum with one
shared immutable operation/login binding. This removes invalid combinations from
the in-memory representation; it does not remove the required recovery states.
Do not expose phase fields or a generic transition engine as a public fact API.

| Private phase | Retained evidence and permitted progress |
| --- | --- |
| Reserved | Exact request, original deadlines, protected authority/epoch and capacity for completion; accept the first nonce or an exact retry |
| Challenged | Original nonce; accept host-authenticated approval for that exact intent, with conflicting callbacks refused |
| Approved | Original nonce/approval and fixed canonical signing input; recheck current authority before Ed25519 dispatch or synchronous certified publication |
| Dispatched | Ed25519 only: exact effect identity and signing input; a lost response stays unresolved and reconciliation is lookup-only |
| Completed | Exact retained claims and mechanism-specific completion evidence; repeated completion/retrieval returns the original result without another signing effect |

Capacity reservation, caller/account authentication, immutable device binding,
renewal watermarks, original deadlines, stale-callback refusal and result retention
are preserved capabilities. Nonce acquisition, application approval, actual signing
and certification-root publication remain host effects. Memory publication must
finish before effects are dispatched; a failed or interrupted response never
authorizes an additional effect. Closing lost state is containment, not recovery.

Initial issue and authenticated renewal enter the same operation owner after
their distinct caller/binding checks. They do not create separate pending-result
stores. The issuer checkpoint has explicit reopen and incompatible-format refusal;
current consumers and helper APIs are qualified at their actual boundaries.
The first implementation must exercise exact retries, conflicting callbacks,
capacity refusal before dispatch, authority/epoch changes and response loss using
independent effect traces. No empty issuer package or accepting endpoint represents
completion of this slice. These constraints do not establish a complete issuer or
a claim of LOC savings.

The local implementation after 0.46.0 supplies Ed25519/certified issuance and renewal through
`ic_auth::issuer_memory`, independently of receiver/verifier features. It retains
one account owner with login and operation maps; each account has at most eight
logins, so slot/browser selection needs no separate indexes. Operations refer to
the immutable login binding and retain their own original times, authority and
private progress. No initial/renewal split store or public transition/fact API is
introduced. Initial issuance may reuse an exact retained login under fresh account
authentication; this is not browser-authenticated renewal.

Browser-authenticated renewal compares the actual ingress principal with the
retained browser key using the same Platform-backed derivation as verification.
Its account/login fields are lookup hints; grants must match the original binding.
Read-only preflight returns inputs for current application permission without
observing time, consuming credit or reserving a sequence. Preparation repeats the
checks after any await. The first renewal requires completed issuance; each later
sequence requires completion of the previous renewal or pruning of that exact
renewal after both effect deadlines and the reconciliation interval. Only its
immediate successor is eligible. The original sequence stays consumed and an
unknown signing outcome stays unknown. Original start, deadline, browser,
slot, generation and policy remain unchanged.

One per-account lookup index maps `(login identity, sequence)` to the shared
operation, including its original epoch. It owns no nonce, approval or result.
The login retains its renewal sequence and disposition; an
old issuance completion cannot complete a newer renewal. Missing result evidence
does not reset this watermark or authorize recreating consumed work. Retained
browser-authenticated reconciliation remains readable after expiry, restart or
revocation; fresh preparation does not bypass revocation.

Renewal IDs preserve SHA-256 of the existing request frame at identity
`0x41555448495201`, with all four original encoding allowances. A private borrowed
projection uses published Wire; it is not a persistence codec or endpoint. Three
independently packed Node frame/hash vectors cover consecutive IDs and wide
integer/principal/grant fields. The issuer feature selects Wire, Serde and Platform
Types self-authenticating principal support without enabling either verifier.

Preserved in this slice: account/epoch/operation and account/login identities,
canonical Ed25519 bytes, immutable binding/grants/policy, device exhaustion,
monotonic generations, original deadlines, global/per-account bounds, full-start
attempt credit, stale-callback refusal and exact-result reconciliation. Dispatch
is now an explicit retained state: only its first transition returns signing input,
and an unknown result is lookup-only. Completion accepts a host-authenticated
64-byte signing result; Auth does not treat a remote callback as authenticated.

The byte limit reserves exact canonical claims for every operation, including
pending work, plus 64 signature bytes for Ed25519. Record counts bound remaining
memory; this is not the legacy serialized-record quota or a heap/allocator measurement. The host
must still commit dispatch before the effect and authenticate its outcome. Native
interruption closes partially published state; it does not roll back writes.

Bounded pruning now uses an operation-expiry index and an idle-login expiry index.
Each operation captures removal at `max(admission, receipt) + reconciliation_retention`;
pending/unknown/completed outcomes have the same retention obligation. The host's
positive retention interval is part of the immutable login policy. A stored exact
result-byte reservation avoids re-encoding records during maintenance. References
keep logins alive until every retained operation is gone; adding work to an idle
login removes its expiry entry before publication completes. Once the final
operation is pruned, a login that never reached signing is eligible for the next
prune call. A dispatched initial operation with unknown outcome keeps its login
until the login deadline unless explicitly revoked. Revocation releases the slot
on bounded pruning after the last operation’s retention ends. Hosts sizing for
rapid device replacement must allow for this retention interval and run pruning.

One call selects at most 32 records: operations first, then already-idle expired
logins with the remaining budget. Newly idle logins wait for the next call. Ties
preserve reference order: time, principal length/bytes, epoch and identity. No
account/history scan or full-state clone is used, and empty accounts are reclaimed.
Expired rows remain charged until explicit pruning. Maintenance observes trusted
time but neither charges nor refills fresh-intent credit.

Forgetting a current-epoch operation/login advances the operation epoch in the
same publication. A separate boot fence changes only on restart, so pruning does
not invalidate unrelated live callbacks. Renewal sequence watermarks survive
result pruning; expired unreferenced login reclamation is protected by closed
epochs, and newly authenticated slot reuse receives a higher global generation.
Neither pruning nor volatile state reconstruction establishes fresh authority.

Certified completion now uses the same operation/login owner and completion
watermark. Protected authority captures either Ed25519 or a certified seed hash;
request fields cannot switch the mechanism. Certified work moves directly from
approved to completed through one synchronous host callback, without an external
signing dispatch. Auth supplies the exact claims and empty-leaf path
`/sig/SHA256(seed)/SHA256(domain-framed-claims-digest)`. The existing fixed-domain
framing is shared with verification, preserving all signed bytes.

The host preflights its leaf capacity, then publishes the leaf and composed root
in the same transaction as Auth completion. Ordinary Auth refusals invoke no
callback. A callback error or unwind closes the issuer because host writes may
already exist; an IC host must trap to roll back, and native hosts must discard
the failed enclosing state. This boundary adds no signature store, executor or
rollback framework. It retains necessary coordination rather than claiming a
removed legacy invariant. Exact completed retries skip the callback after expiry,
revocation, restart or leaf loss. Current witness/certificate bytes can change;
retained claims and their fingerprint cannot. Host leaf pruning is separate from
issuer result pruning and cannot authorize re-signing a consumed operation.

Native and installed evidence is recorded by boundary in
[compatibility and acceptance](compatibility.md), including actual leaf/root
rollback and volatile loss. Stable reopen, external signer recovery and complete
consumer integration remain separate obligations. Keep those evidence limits out
of the semantic publication contract above.

## 11 Browser lifecycle

The implemented package has one strict IndexedDB journal, one internal bounded
signing-attempt owner, credential retrieval, explicit admission and one-step cleanup.
The supported surface is a lifecycle-only `LoginJournal` handle and complete
operations. Claim/guard/settlement stages remain internal. The opening factory
returns the same state owner without a second store, public mutable-record API
or generic backend interface.

`retrieveCredential` commits an issuance claim before its single bounded issuer
call and exact evidence before returning. `establishSession` commits target inventory
before one browser-signed admission call. `cleanupOnce` commits a lease/count before
one generated Wire update and confirms only a matching authenticated revocation.
`cleanupUntilFinished` schedules that operation using journal-derived wait delays;
it preserves the fixed deadline and attempt counts. A failed destination backs off.
Other due destinations proceed in order of fewest attempts. Cancellation and local
identity, configuration, storage or clock errors stop the run with progress retained.
The internal signing owner captures principal/destination and rechecks the journal
around asynchronous signing and reply acceptance; every exit closes that attempt.
These are retained safety obligations, not a claimed equivalent-feature LOC saving.

One HTTP adapter serves admission and cleanup directly. Platform owns certificate
verification/polling; Auth owns captured gateway/root/endpoint configuration,
single submission, cancellation and fresh journal-guarded signatures. No replacement
crypto, runtime or automatic application replay is introduced.

[Browser usage](../../packages/client/README.md) defines the supported API;
[compatibility and acceptance](compatibility.md#browser-evidence-and-remaining-acceptance)
records native and installed evidence. Raw HTTP reply authentication and a fixed combined issuer/receiver journey are
qualified on one subnet, including reconciliation, a protected write and logout.
The per-login SessionCoordinator shares readiness/admission and credential flights,
checks ephemeral hints against journal time, and bounds outstanding callers at 64.
Only typed expiry or completed expired evidence permits its second preparation
attempt; explicit admission never uses that retry. It returns scheduling metadata,
not an SDK signer or application authority. Its ordinary-call wrapper then invokes
one application callback with the existing bounded temporary signer. Every signature
and final result rechecks the captured readiness deadline and durable generation.
Once the callback starts, expiry refuses without renewal or replay.
Terminal cleanup now permits retryable host-owned key disposal using the retained
public binding. The host owns conditional deletion and replacement protection;
Auth retains the journal and unresolved evidence. Production Canic integration,
complete multi-receiver acceptance, host key custody and broader renderer recovery
remain open. The lifecycle requirements below still apply to that remaining work.

Keep one coordinator over one strict journal owner. Inject Platform SDK identities, issuer transport, receiver transport and clock mapping. Do not recreate a second SDK identity facade or implement application retries.

Retain distinct account and browser signer roles. Initial issuance/reconciliation uses the account identity; renewal and subsequent reconciliation use the browser binding. Receiver calls use only the browser identity. Capture destination and principal bytes before asynchronous work and bind temporary signers to their target, generation and attempt.

Persist issuance intent before dispatch. Pending sequences reconcile; completed sequences reuse retained evidence. Store completion before receiver admission. Record each receiver in the logout inventory before contacting it.

Preserve the current private journal profile ic-auth-login-cleanup-journal-v1 unless a separately authorized browser migration changes it. Metadata and exact credential evidence commit atomically, while ordinary guards load bounded metadata only. Unknown issuance remains authoritative even when its recoverable sequence is absent.

Logout commits durable local invalidation before waiting for any admission flight or sending cleanup. Fence late signatures, completions and replies; already returned signatures or accepted requests cannot be recalled. A storage failure means durable local logout was not established.

Preserve the reference limits: 64 destinations including issuer, eight attempts per destination, 64 outstanding admission/readiness/application operations per coordinator, five-second network attempts and a fixed cleanup deadline. This bounds cooperating coordinator work, not all tabs or hostile scripts.

Only authenticated Revoked replies confirm a destination. Unconfirmed, capacity refusal, timeout and HTTP acceptance do not. Expiry of local cleanup is not evidence of remote expiry. Keep unknown issuance and unconfirmed destinations visible.

Readiness may perform the reference's bounded preparation/retry sequence, at most two attempts under its specified authenticated expiry conditions. Explicit establish remains forced admission without that expiry retry. Never replay an application mutation after an uncertain reply. Preserve injected conservative clock mapping; infer no issuer/receiver/browser offset.

Require real SDK transport and native browser IndexedDB qualification, including two tabs, strict durability, renderer interruption and suspended signing. Fake storage tests are a separate layer.

Installed issuer transaction evidence is available through
[`make qualify-host`](../../qualification/ic-auth-host-conformance/README.md).
The standalone Platform/Wire fixture checks real IC rollback at leaf/root/local
completion boundaries and verifies actual certified claims. This qualifies the
synchronous transaction contract, not Canic application endpoint integration.

## 12 Boot, authority and lost state

Every install, upgrade and recovery starts authentication closed. Checked retained safety state and a fresh authenticated enrollment response bound to receiver and boot challenge are both required for readiness.

Snapshots retain monotonic revision/generation and terminal key revocation. Exact repeat revision/digest is idempotent; lower revision or same revision with changed content refuses. Authority generation changes invalidate prior sessions atomically. Freshness is at most the reference's 3,600 seconds and any tighter approval deadline; refreshing it cannot extend earlier leases.

Complete persistent reopen is distinct from volatile loss or stale backup restoration. Missing replay, denial, device or authority evidence cannot be replaced with an empty store, local timestamp, bumped local generation or fresh enrollment. Complete volatile loss permanently closes that installation. Stable installations reopen only from a valid retained checkpoint.

No finite clock-skew assumption makes unknown logout conclusive. An issuer-ahead credential may remain capable of later admission. Preserve the reference's explicit Unconfirmed outcome and lost-state refusal.

Ordinary admission may be disabled while safe self-revocation remains available. Missing safety state must not report successful revocation. Authority refresh scheduling belongs to the host; Auth supplies validity rules, not another timer/executor.

The installed issuer fixture exercises an actual upgrade with no retained heap:
its post-upgrade hook clears the signature contribution and leaves the issuer
absent. Completion, preparation and revocation remain closed. This establishes
volatile issuer loss containment only. The receiver fixture now also checks actual
upgrade loss after known revocation or unknown denial: enrollment requests cannot
recreate missing safety state. An intact receiver accepts the same valid credential,
including an issuer-ahead proof after temporary denial expiry. Receiver/application
writes roll back together on a trap; ordinary invalid proofs retain their charge.
The fixture now fetches a single-key enrollment reply from an installation-bound
IC destination, preserving callback source provenance across the await. It checks
receiver/challenge/freshness/revision and key identity, retains terminal revocation,
and commits host entry/receiver metadata in one callback transaction. Callback
traps do not undo the earlier request submission. All installed Auth checks
use a single simulated subnet. Late/overlapping-reply integration checks remain
outstanding for the supported single-subnet Canic flow; the response freshness and
challenge-fencing requirements remain unchanged. Fixed fixture domain/grants and
deterministic challenges do not qualify the complete Canic registry, production
entropy/refresh scheduling, trusted recovery or application endpoint wiring.

## 13 Deletions, reuse and compatibility policy

| Candidate | Disposition |
| - | - |
| Stable-to-memory mini-ledger reconstruction | Replace with bounded operation facts and shared pure decisions |
| Parallel receiver orchestration and redundant record projection | Remove only after the common operation trace covers both backends |
| Full-store cloning, mirrors or scans introduced for convenience | Prohibited on ordinary calls |
| Duplicate canonical bytes, request hashes or browser codecs | One authoritative owner; retain independent vectors |
| Separate policy for Ed25519/certified or volatile/stable | Prohibited |
| Retired token/session flows and historical compatibility readers | Do not reintroduce |
| Historical verifier/signature helpers | Retain only capabilities required by current consumers |
| Historical custom commit/restore API | Retired by the authorized hard cut; preserve current host atomicity obligations |
| Release scripts, npm publication and deployment framework | Do not transplant into the implementation scaffold |
| Tests for uniquely required guarantees | Retain/re-express independently; delete only redundant coverage with a recorded mapping |

Reuse of a sound primitive is permitted with a named invariant, caller, dependency review and evidence. Copying an existing package is not a completed design milestone.

Current Wire and signed-byte expectations are explicit and independently tested. Historical parity is not required. Passing Wire checks does not establish storage compatibility; any future opening of retained backing requires separate admission and recovery evidence. The source hard cut authorizes no live-data reset or deletion.

Do not reuse the current schema identifiers for different records. Do not erase browser journals or issuer pending work to simplify the cut. Old production state remains owned by its existing host until an authorized transition is qualified.

The first deliverable is one complete volatile receiver slice with independent proof vectors and failure traces. Durable storage and cross-backend comparison follow in a separate milestone. Expansion follows the qualification gates, not a target line count.

# Implementation and qualification

Status: acceptance sequence, reviewed 17 November 2026. The [current handoff](../status/current.md) records implemented slices and their proof limits. These requirements are not a parallel backlog. Passing a stage does not authorize consumer deployment; durable requirements apply only at the future storage milestone.

## Simplification constraints before further expansion

The [receiver complexity comparison](../reports/audits/2026/11/15/receiver-complexity/01/report.md)
establishes narrower volatile state ownership, not a demonstrated whole-product
LOC saving. Before adding issuer or browser state, record its canonical owner,
allowed transitions and external-effect/recovery boundaries, and map the relevant
legacy capabilities to retained, removed or deferred behavior. Name which duplicate
state or coordination layer disappears. If none disappears, identify the design as
retained necessary complexity rather than claiming simplification.

The browser package exposes lifecycle handles plus retrieval, admission and one-step
cleanup operations. Claim/guard/settlement stages and the signing-attempt helper
remain internal; qualification may import their source modules directly. The
opening factory returns the same journal owner with a narrowed consumer type,
without a second store or generic facade. This is API ownership, not a security
boundary against code running in the same JavaScript realm.

Keep one issuer operation owner and one browser journal owner. Reuse Platform
identities, generated Wire contracts and host transaction boundaries. Introduce
no public backend framework, compatibility mode or identity facade for hypothetical
consumers. Internal fact assembly and policy decisions stay internal unless an
actual external consumer earns that boundary; preserve public host inputs, errors,
configuration and useful result projections. Audit the current public receiver
policy surface before first downstream adoption or its next API change.

The authorized pre-recode cut retires historical custom commit/restore API
obligations; current host atomicity and recovery requirements remain. Defer durable
abstractions until the supplier and second implementation demonstrate their needs.
Track production Rust, Rust tests/qualification and browser TypeScript separately,
and compare equivalent capabilities. Retain independent safety tests; reduce tests
only with a concrete mapping showing redundant coverage. Smaller files, renamed
layers and missing functionality alone do not establish reduced complexity.

## 1 Maintain the current contract and provenance inventory

Use commit e52a40be75dd9809d0528189d84b43fe96952ad9 of dragginzgame/ic-auth-legacy. Resolve files from committed objects, not a sibling working tree. The repository's reference.sha256 records selected reference-file bytes, including the initial request implementation and retained vector source. This is an integrity inventory, not test results or a complete repository archive; retained fixture provenance lives beside the tests.

Maintain independent expectations for the current implementation:

- Canonical claims, signing-domain and session-request hash vectors with independent expected bytes.

- Wire method IDs, complete nested field/tag order, all four resource limits and generated TypeScript output.

- Receiver/session/replay/login and issuer stored-format identifiers and selected record fixtures.

- Browser journal profile, completion identity, refusal codes and recovery traces.

- Safety and refusal cases selected for current behavior, including negative host drivers.

- Actual current downstream call sites and browser exports.

The current source's receiver schema is 0x41555448525304; issuer schema is 0x41555448495303. These identifiers are reference facts, not names available for a different replacement encoding.

The maintainer's hard-cut decision removes pre-recode consumer inventory as an API-removal gate. Record actual current Canic/browser consumers, selected features and recovery obligations when integrating them; do not infer production adoption from library tests.

Produce a disposition table: preserved contract, newly implemented owner, independent acceptance case, proposed API change, required downstream decision. Do not port old engine code merely to make this table green.

## 2 Build the protocol boundary first

Implement passive domain types, the signed codec and exact Wire adapter before storage orchestration.

Required evidence:

- Retained golden claims and digest values, independently constructed without the replacement encoder.

- Every signed field affects the digest; strict role/scope ordering, duplicates, malformed principals, wrong schema/mechanism, nonminimal/trailing input and length arithmetic refuse.

- Unsigned session-scope reordering retains the old request hash; duplicates refuse; None TTL and explicit default differ.

- Independent current bytes and Rust/browser decoding for every method, response/error variant and nested proof variant.

- Separate below/at/above byte, allocation, node and depth cases. A byte-only test does not cover traversal or allocation admission.

- Browser full-frame caps remain independent of native budgets; buffers/subarrays detach; u64 remains bigint.

- Regeneration matches all four canonical TypeScript codecs and their selected Wire runtime, with no local patch.

- Retired method identities and token/schema profiles refuse without fallback.

Do not execute the old implementation in product or qualification checks. Independently assemble fixtures without the codec under test. A round trip through one encoder/decoder is not independent evidence.

## 3 Prove complete verification

Qualify both enrolled Ed25519 and certified proofs before calling either a replacement.

Ed25519 cases include valid signatures, wrong digest/key, malformed length, noncanonical S, small-order key/R, cross-domain replay, wrong caller under the supported dependency graph.

Certified cases include complete Root/delegation and issuer-canister verification, wrong canister/seed/key binding, wrong network root, audience or grants, malformed CBOR/tree/path material, stale/future certificates and independently exhausted proof-work bounds.

The fresh certified aggregate variable-material cap is at most 131,072 bytes and may be tightened by protected configuration. Also preserve separate issuer signature size, witness-step, certificate age and lifetime limits. The outer 524,288-byte frame limit does not waive these.

For each failure, record its stage, public projection, whether cryptography ran, whether credit changed and whether any identity state changed. In particular, scalar/shape rejection before charging differs from failed charged verification.

Test protected verifier selection: request-supplied algorithm/key material cannot choose enrollment, and standalone VerifiedClaims cannot be constructed from decoded claims or used as a session. Preflight must reserve no unverified binding.

## 4 First complete receiver slice

Implement this narrow application scenario on memory first; reuse it for the future durable backend:

1. Two issuer-qualified accounts, two browser devices and two independent receivers.

2. Valid Ed25519 admission, proof-free authorization and a compact receipt.

3. Exact retry after proof expiry but before lease expiry, returning the unchanged receipt.

4. Request conflict, invalid signature and wrong caller with their expected charging/write behavior.

5. Known logout, unknown logout and admission/logout orderings.

6. Higher-generation replacement of one device while the other stays authorized.

7. Fresh renewal clamped to the original absolute login deadline.

8. Volatile loss stays closed; fresh enrollment or a local generation cannot restore missing replay/device fences. Separately qualify stable reopen with fresh boot enrollment and complete retained state.

Use a private operation-level storage interface only after implementing the real needs of both backends. Compare facts and transitions directly; do not make the stable implementation emulate a complete memory store.

Volatile exit gate: one admission pipeline, no temporary reconstructed login ledger, no global read/clone, one current authorization rule and independent accepted/refused behavior evidence. Stable mode additionally requires equivalent cross-backend behavior. A consolidation that increases transformations or materially worsens ordinary lookup costs must be revised.

This gate qualifies the volatile receiver, not durable storage. Reuse its independent behavioral cases against `ic-storage` when that supplier is ready, adding the lifecycle and transaction proofs specified in architecture section 9. Protocol, proof, receiver-policy and issuer/browser work proceed without an interim adapter or an invented persistence API.

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

Qualify standalone passive types, Wire codec-only, memory receiver, each verifier, both verifiers together and issuer consumers. Qualify receiver-stable and issuer-stable independently. Inspect Cargo feature unification and actual native/Wasm graphs. Preserve one Platform Types identity and one host executor/guard. Receiver consumers must not pull issuer signing custody, native tooling or TypeScript metadata generation.

Record exact source, locks, rustc, target, profiles, optimizer, final Wasm and simulator/server identities. Compare current implementations only under matched conditions in isolated targets; the historical implementation is not an executable acceptance dependency.

Measure:

- Total uncompressed Wasm and code-section bytes.

- Fresh admission and ordinary authorization separately.

- Decode, proof verification, bookkeeping and complete endpoint costs.

- Cold/populated/post-upgrade lookup, memory, allocations and stable IO.

- Exact retry, invalid proof, wrong caller, exhaustion and unknown logout floods.

- Prune/inspection work and issuer recovery/renewal, including refusal costs.

The reference handoff reports 1,224,808 instructions for authentication at 100,000 receiver records and 1,223,895 after upgrade. Its 1,300,000-instruction ceiling belongs to that historical scenario, not the current volatile fixture. Adopt a current operational ceiling only with a matched workload and consumer decision. These are published source reports, not measurements rerun for this review. [Pinned handoff](https://github.com/dragginzgame/ic-auth-legacy/blob/e52a40be75dd9809d0528189d84b43fe96952ad9/docs/status/current.md).

Retain provenance for historical ceilings without treating unmatched observations as current gates. Historical named-symbol attribution is not removable Wasm size. The handoff explicitly leaves populated persistent failure-cost and production fleet budget qualification incomplete; set and justify those missing operational limits rather than declaring historical success.

No line-count, size or speed target can waive replay, revocation, rollback or uncertainty guarantees.

## 10 Consumer cut and completion criteria

Select a consumer only after its relevant gates pass. Its review must decide:

1. Rust API changes, including custom commit callbacks and independent signature helpers.

2. Exact supplier/package identities and coordinated receiver/browser artifacts.

3. The selected persistence supplier and existing-state disposition: exact stable-format preservation, an explicitly authorized state conversion, or a separately authorized hard cut with legacy refusal.

4. Browser journal/key custody, pending issuer work and recovery evidence.

5. Authority refresh/readiness, host rollback, certification and application rechecks.

6. Operational rollback and external-effect reconciliation after the cut.

Keep shadow comparison observational: no double signing, double replay consumption or duplicated business effects. Use isolated fixtures for destructive failures.

A hard cut does not mean reinstall by default. No local document authorizes erasing host data. An incompatible backing must refuse; any importer/reset/reinstall needs the consumer's explicit disposition and installed recovery proof. Restoring an old binary alone does not restore compatible data or undo external effects.

Completion of the volatile milestone requires a consumer contract inventory, independent vectors, both proof mechanisms, bounded memory state and loss refusal, issuer/browser interruption evidence, actual Wasm lifecycle results and accepted costs. Each claimed simplification lists the surviving owner, deleted path and invariant coverage. Stable storage, retained upgrades and cross-backend equivalence require separate qualification.

Report volatile Auth qualification and stable-storage adoption separately. Completion of the durable milestone additionally requires qualified `ic-storage` integration and the selected consumer state/recovery disposition; successful volatile qualification does not establish that outcome.

Only after a qualified replacement and consumer decision exist should release/delivery machinery be integrated through the shared owners. Do not reproduce the old release system as an initialization task.
