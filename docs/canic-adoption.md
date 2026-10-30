# Canic library adoption contract

Auth owns reusable authentication semantics; Canic owns the product host that
uses them. [Canic #491 — Extract reusable authentication, token and session machinery](https://github.com/dragginzgame/canic/issues/491)
is the implementation/acceptance tracker. This document is the current interface
boundary, not evidence that Canic or Toko have deployed it. Earlier extraction
checkpoints are retained separately in [adoption history](canic-adoption-history.md).

## Session commit contract

Pending Auth **0.25** replaces `CanonicalAuthError::{EmptyScope, InvalidScope}`
with `Scope(ScopeLabelError)`. Update Rust error matches to the typed cause;
invalid labels are no longer retained in errors. Scope grammar, bounds, first
refusal order, public refusal categories and signed/public Wire bytes remain.
Import `ScopeLabelError` and checked `ScopeLabel` from the protocol package;
request/operation getters expose checked labels with `as_str()` for text.
`ScopeLabel::validate` replaces the engine's detailed grammar helper. Keep raw
proof-grant validation at its existing stage, including exact-retry behavior.
Match `delegation::DelegationField`/`DelegationTarget` variants instead of string
fields/targets. Logout uses the existing Wire unit request; the empty Rust
`LogoutRequest` is removed.

Private session/login record identities change to `0x415554485311` and
`0x415554485313`, with fixed 32-byte audience IDs; replay remains
`0x415554485302`. The maintainer confirmed no retained current Auth records.
Adopt one coherent checkpoint, including byte totals, before serving; old formats
are refused without fallback or clearing. This is separate from the earlier
CBOR retirement and grants no permission to erase Memory backing or its ledger.
See [record format and custody](sessions.md).
`SessionEstablished::receipt()` projects the common receipt while callers retain
the outcome distinctions where needed. Receipts still authorize nothing.

Auth **0.24** consumes `SessionRequest` so its scopes move into a fresh
session. Clone explicitly when a host also needs the checked request afterward.
`PortableVerificationContext::Ed25519` borrows `PreparedIssuerKey`, constructed
from protected issuer bytes and retained with enrollment. Rebuild after decoding;
keep current authority, revocation, budget and strict verification checks. This
key shape is neither enrollment proof nor TEE assurance. Root verification and
standalone raw-key APIs retain their current contracts.

Read endpoints project `SessionError::read_refusal()` and return typed
`SessionReadRefusal` categories. Update the reader/receiver pair to Wire identity
`0x415554480013`; former `0x415554480003` frames are refused. Signed claims,
current session/replay/login storage and browser journals are unchanged.

Auth **0.22** hard-cuts private session/replay/login storage to bounded
native Wire records. The maintainer confirmed no deployed or retained Auth records
need preservation. Old records are refused, with no fallback or implicit reset.
Adopt one coherent new checkpoint before admission or reads; preserve atomic
commit, replay/denial retention and generation rules. The unpublished reference
checkpoint used `AUTHNAT2`, replaced by `AUTHNAT3` in pending 0.25; Canic owns its
own durable framing and lifecycle.
Signed claims, public Wire transport and the TEE/non-TEE trust model are unchanged.
See [record bounds and restoration](sessions.md#storage-and-reset).

Auth **0.23** changes all `SessionEstablished` payloads to
`SessionReceipt` (caller, issuer, subject, expiry), and consumes owned
`Session`/`Replay` in `SessionStore::admit` and `MemorySessionStore::admit_with_commit`.
Update backend signatures and move the validated pair into storage. Read full
records through the store; authorize later operations against current protected
state. Receipts are never grants. Atomic commit obligations and stored formats
are unchanged; no new reset is authorized or needed. Wire DTO Principal fields
also become Platform-owned values; native Wire bytes and generated browser
contracts remain unchanged. See [Wire migration](../crates/ic-auth-wire/README.md).

The same 0.23 cut requires `SessionContext::required_scopes: &RequiredScopes`.
Validate and retain operation requirements once with `RequiredScopes::new`, or
use `RequiredScopes::NONE` for empty requirements; live authorization remains
mandatory. Build immutable limits with `SessionLimits::new` at protected backend
configuration admission, reconstruct after decoding and return the retained value
from `SessionReader::limits`. Their getters replace public fields.
`MemorySessionStore::new` now returns a store directly; remove construction-only
error handling while retaining fallible record restoration. See
[session configuration](sessions.md#admission-and-ordinary-calls).

Auth **0.21+** requires `PortableAuthority::grants` to borrow a
retained `ValidatedGrants`. Construct it from protected enrollment configuration
once and reconstruct it on storage decoding/restoration. Replace it atomically
with authority and generation updates; keep all live authorization checks.
`as_slice()` supplies raw grant projections. This is structural evidence, not
proof verification or trust enrollment, and public Wire/checkpoint bytes remain.
No sibling adoption is claimed by the Auth implementation.


Auth 0.19 changes the Rust admission, context, lifetime and storage APIs.
Pass a complete credential and `SessionAdmissionContext` to admission; migrate
custom backends from mutable `SessionTransaction` staging to
`SessionReader` and `SessionStore::{read, admit, logout}`. The Auth 0.20
API returns `Option<Cow<'_, Session>>` from the reader and requires a projection
callback on `authorize_session`; borrow a retained memory record or return
`Cow::Owned` for a bounded persistent decode. Project the account/view inside the
snapshot, before any await. Public Wire and checkpoint bytes are unchanged. Match typed ledger
state and the explicit `SessionCommit::write` payload, and use checked TTL policy
values. See the [migration](sessions.md#session-commit-write-migration).
Historical consumer checkpoint: Auth 0.19.4 was published and Canic 0.112.0
used the commit API in its test-only backend;
[Canic PR #701 — Retain persistent Auth receiver state](https://github.com/dragginzgame/canic/pull/701)
proposes the retained persistent host component and makes that backend available
under the existing receiver feature. Its 40 native storage tests, normal
native/Wasm graphs and package/test Clippy pass. This is proposed storage
integration, not installed upgrade or production endpoint acceptance; #491
retains those requirements.

[Canic PR #703 — Bounded Root enrollment and receiver gate](https://github.com/dragginzgame/canic/pull/703)
is stacked on #701. Its internal producer and receiver installer bind snapshots
to protected Root/Fleet/receiver identities and a fresh boot challenge, retain
revision/generation high-water marks and terminal retirements, and atomically
advance session authority generation. The opaque prepare/fetch/finish pipeline
holds no mutable receiver borrow during the Root call. Its method identity is
reserved, not mounted in the Root bundle. Native storage/Wire and compile proofs
do not establish actual IC authentication, installed upgrade/outage acceptance or
production activation. Registration/custody approval, refresh scheduling and
user-facing endpoint wiring remain Canic-owned requirements in #491/#507.

## Released API and dependency direction

The workspace contains `ic-auth-protocol-types`, `ic-auth` and
`ic-auth-wire`. Select matching package releases in the consumer's root catalog;
no sibling source/path override or copied codec is needed. Auth has no Canic or
Toko dependency. Rust 1.91 is the common minimum.

- `ic-auth-protocol-types` owns checked identifiers, portable claims, certificate/
  proof contracts and typed logout outcomes. It exposes Platform-owned `Principal`,
  with no Candid traits, CDK, stable-memory owner or production endpoints.
- `ic-auth` owns canonical signed bytes, proof verification, certificate rules,
  bounded signature preparation and the transaction-based session engine. Pure
  verification takes trusted authority/time explicitly; it never installs trust.
- `ic-auth-wire` owns bounded transport projections and method layouts over IC
  Wire, plus canonical generated TypeScript. Structural decoding is not proof
  verification, enrollment or session admission.
- `packages/client` is a private, unpublished browser coordinator with injected
  SDK identity, issuer/admission transport and durable logout storage. It does
  not implement wallet login or a production issuer/reconciliation service.
  Local [archive delivery](browser-client.md#local-archive-delivery) now supplies
  a source/integrity-bound candidate and isolated consumer proof. Canic retains
  final archive distribution and actual endpoint/browser acceptance.

Auth selects Platform 0.11 and enables only `principal` for passive types.
Consumers sharing that owned Rust type must align to Platform 0.11; external
Agent/Testkit/crypto APIs keep explicit `principal::ExternalPrincipal`
conversions with `principal-external` selected. The engine explicitly selects
`principal-self-authenticating` for browser-key hashing; passive types do not
pull external Principal or SHA-2. Do not select the older upstream Principal identity or downgrade
Auth to follow a historical example. No Candid or runtime capability is selected
by the passive edge. [The published-type acceptance](https://github.com/dragginzgame/ic-auth/issues/82#issuecomment-6312237287)
records the earlier Platform 0.6 acceptance. Current Platform 0.11 qualification
and its limits are recorded in [the handoff](status/current.md).

Forward only required capabilities. `portable-signature-verification` selects
Ed25519, `chain-key-verification` supplies standalone Root verification, and
`certified-proof-verification` supplies complete certified portable verification.
`sessions` enables the engine without a verifier; `session-restore` adds checked
restoration. Signature-store preparation and canister-signature verification
remain separate optional primitives. Native Testkit/Host/Build, CDK and fixture
contracts stay out of the public Auth graphs. All public library capabilities, including certified verification, now reject
Candid and raw IC calls. Native qualification still brings Candid through Agent/Testkit.

### Receiver feature selection

Select the complete verifier at the receiver's normal dependency edge, rather
than relying on a test dependency to enable it:

| Receiver capability | `ic-auth` features, with defaults disabled |
| --- | --- |
| Volatile sessions with Ed25519 admission | `sessions`, `portable-signature-verification` |
| Volatile sessions with certified admission | `sessions`, `certified-proof-verification` |
| Volatile sessions accepting both enrolled mechanisms | `sessions`, `portable-signature-verification`, `certified-proof-verification` |
| Checked persistent restoration | Replace `sessions` above with `session-restore`; retain the selected verifier features. |

`certified-proof-verification` enables the complete portable certified verifier
and its Root/canister-signature prerequisites. Enabling only those lower-level
primitives does not enable `portable_certified` or
`PortableVerificationContext::Certified`. Ed25519-only receivers need no certified
verifier. The protected enrollment selects the mechanism; the submitted proof
cannot enable another one.

Qualify the receiver without dev dependencies on native and Wasm. A test build
can unify features from a dev dependency and hide a missing production feature.
Use the shared Auth Wire methods through `ic-auth-wire`, or the explicitly
qualified consumer transport described below; neither transport supplies a
verifier, enrollment or endpoint automatically.

### Canic configuration and endpoint wiring

Source inspection on 2026-10-28 used released Canic 0.112.0 at
`5c236ab130c0d63892ef281b45bdc3fa353e012f` in an isolated checkout.
These are source findings, not installed-receiver acceptance.
The existing schema and its parse test accept this stanza for an existing
`default` component spec:

```toml
[component_specs.default.auth.local_application_authorization]
allowed_scopes = ["my_app:read", "my_app:write"]
default_session_ttl_secs = 900
maximum_session_ttl_secs = 1800
```

Canic canonicalizes the scopes and rejects invalid/duplicate scopes and invalid
TTL bounds; its present maximum is 1,800 seconds. This config declares local
policy, but does not activate portable login: `access/auth::authorize_local_application`
still returns a denial. On that base the Auth-backed
`application_sessions/shared` store is compiled only for tests; PR #701 promotes
the internal component without opening the facade. Existing delegated
issuer/verifier switches and proof install/status APIs do not supply portable
issuance or reconciliation.

Keep local scope/TTL policy separate from Root-authenticated enrollment. The
latter supplies issuer/key/mechanism, validity and authority revision; a request
must never supply its own trusted authority. Receiver persistence needs an
explicit host choice, independent of issuer custody: volatile state loses logins
on reset and requires protected reset floors; persistent state must commit and
restore sessions, replay, login reservations/denials and authority/clock metadata
consistently. The current schema has no implemented persistence or TEE selector;
do not add example keys that its strict parser rejects. Production activation
still requires the qualified skew/reset and enrollment contracts.

Canic's current `auth-local-application-authorization` feature enables both proof
verifiers, restoration and older delegated verification prerequisites. Forward
the minimal receiver feature profile from the table above for each actual build;
runtime configuration alone cannot remove those compiled capabilities. Ed25519
verification does not imply TEE custody, and neither determines session persistence.

Canic's `canic_raw_update(..., wire(id = ..., request = ..., response = ...))`
path, already used by its Blob adapter, can mount Auth's owner codecs. Select
matching `ic-auth-wire` alongside the two existing Auth dependencies, and use
each marker's `NAME`, `ID`, `REQUEST_LIMITS`, `RESPONSE_LIMITS` and concrete codecs.
Preserve Canic's normal lifecycle/deployment fences; admission and logout cannot
require an already established session. The reference uses updates for all three:

| Export / owner marker | Synchronous host work |
| --- | --- |
| `establish_portable_session` / `EstablishSession` | Authenticate runtime caller; verify decoded claims against protected enrollment with retained budgets; atomically establish session/replay/login reservation. |
| `portable_session_read` / `ReadSession` | Check the caller's live local session and return bounded metadata without extending its lease. |
| `logout_portable_session` / `Logout` | Revoke the runtime caller and remove its session atomically; acknowledge only after commit, including the bounded absent-session path. |

Decode no client-supplied caller. Preserve normal refusal charges; trap on reply
encoding failure after mutation. Host schema registration must describe these
same owner DTOs, IDs, modes and budgets; enable TypeScript metadata only in the
native generator. Do not copy the fixture's literal `read` business scope into
a generic status check: define that endpoint's required scopes explicitly.

For ordinary application updates, authorize inside the synchronous workflow
before effects and accept only business arguments. Canic's current `AccessCheck`
contract requires side-effect-free predicates, whereas Auth's `authorize_session`
observes retained clock state through the revocation ledger. Do not disguise it
as a pure custom guard or discard that bookkeeping. A dedicated mutable auth
guard would need an explicit Canic contract; query authorization also needs its
own qualification. The tested update path below is usable without that redesign.

The browser supplies separate account and fresh browser identities to
`PortableAdmissionClient`: account identity authorizes issuance/lookup, browser
identity admits and calls each receiver. `establish(receiver)` journals that
destination before dispatch and returns a guarded identity for business calls.
Canic still owns the production issuer and lookup-only reconciliation adapter,
plus enrollment/renewal and browser logout integration. Those requirements remain
in [#491](https://github.com/dragginzgame/canic/issues/491) and
[#507](https://github.com/dragginzgame/canic/issues/507); the private client is not
an installed Canic service or published npm package.

### Synchronous protected application writes

The [reference receiver](../apps/qualification/canister/src/sessions.rs) now
shows a complete ordinary application update: `session_set_asset` accepts only
the new value, obtains the actual runtime caller, clock and protected authority,
and calls `ic_auth::session::authorize_session` with its server-selected `write`
scope. It uses the same retained store/ledger and authorization helper as reads.
No credential or claimed caller is submitted, and no await separates the decision
from the fixture's state mutation and certification publication.

Canic can use the existing Auth 0.15 API for this path. Keep its production
authority, retained ledger/store and endpoint error projection local. Resolve the
returned session's `(issuer, subject)` through application account/resource policy
before effects; the fixture deliberately gives its one asset to any admitted
`write` session. A scope is not a general resource-ownership grant. If application
work awaits another canister, recheck current authorization before subsequent
effects rather than carrying the pre-await decision forward unchecked.

The [installed test](../apps/qualification/runner/tests/signatures/sessions/application.rs)
uses real Agent-signed updates. It checks narrowed read-only sessions, unknown
callers, writes after admission-proof expiry, retained keys after logout,
independent receiver sessions on two subnets and authority-generation changes.
Each refusal leaves the observable application value unchanged. This is volatile,
enrolled-Ed25519 reference evidence, not Canic's production adapter, resource
policy, issuer or TEE qualification. Production acceptance remains in #491.

<a id="pending-09-wire-transport"></a>

## Wire transport

Use `CredentialEnvelope` for credential data, `EstablishSession` for admission,
`ReadSession` for a proof-free local read, and `Logout` for caller-authenticated
local revocation. Their contract markers own identity, concrete request/reply
codecs and limits. Canic binds them to actual exports, admits raw argument size before
copying, decodes the exact method and authenticates ingress/replies. Reply-encoding
failure after mutation must trap so that local state rolls back.

Pending Auth 0.26 transport exposes IC Wire 0.10; Auth 0.16 used 0.7 and Auth 0.15 used 0.6. Earlier Auth 0.14 uses
Wire 0.5; update the Rust transport and generated clients together. Consumers sharing its Rust
Wire types must align that dependency and select matching Auth package releases.
Auth 0.10 and 0.9 used Wire 0.4 and 0.3 respectively; those are historical
dependency contracts, not current selection instructions. Generated files must
come from the selected owner, never downstream patches.

Applications may bind their own command envelope through an explicit qualified
client transport adapter. Preserve the shared domain/signed contract and bounded
translation; do not reconstruct an independent Auth codec or restore a retired
application-token flow. The bounded Wire controls under `apps/qualification/contracts/`
serve private controller/certificate qualification only and are not production
schemas. Linking any Auth crate installs no endpoint or authority.

## Wire 0.6 migration

The subsequent Wire 0.7 adoption preserves these encoder bytes and IDs, but
removes decode-time reserialization and `Error::Noncanonical`. Invalid primitive
encodings return `Malformed`; custom Serde normalization is consumer-owned.
Auth's DTOs retain ordered lists and opaque signed bytes. Keep original frames
for exact retries and do not substitute decoding for Auth's semantic validation.
Align consumers of the re-exported Rust Wire types to 0.10. Testkit 0.72 uses
the same Wire 0.10 types directly, retaining the typed-call contract from
[Tooling #248](https://github.com/dragginzgame/ic-tooling/issues/248).
The public transport, native qualification and reference canister use Wire 0.10.
Metrics and Memory/Storage 0.58 share Wire 0.10 with Auth. No conversion or
compatibility reader is needed; current record bytes stay unchanged.

The finite `Method` trait, static shape admission and typed retained frames are
retired. DTOs use ordinary Serde; the contract marker's `encode_request`,
`decode_request`, `encode_response` and `decode_response` functions bind
its concrete types, ID and limits to Wire's native codec. Results are ordinary
owned byte vectors. Keep exact bytes for any authorized uncertain-effect retry.

Admission changes its ID to `0x415554480012` because refusal tags now follow
zero-based Serde order. Retired admission frames are refused by identity.
Credential/read/logout frames retain their frozen bytes and IDs. Signed claims,
session/replay/login records and the browser journal profile are unchanged.
No storage reset or replay of unknown issuance is authorized.

Native Testkit 0.72 takes a `WireCallSpec` with the contract's `NAME`, `ID`,
`REQUEST_LIMITS` and `RESPONSE_LIMITS`, plus explicit request/response Serde
types. Canister adapters use the same native codecs and pre-copy byte limits;
normal errors must retain admission charges, while reply-encoding failure after
mutation must trap for rollback. Requalify native traversal allowances as well
as field and frame caps; the removed static `SchemaLimit` is not a native error.

Wire metadata derives belong only in the host generator. Regenerate all five
browser files with the selected runtime and qualify Rust/TypeScript frames.
The private fixture's controls/checkpoint have new identities and require fresh
disposable installations; this changes no production memory allocation or layout.


Earlier identity cuts are historical evidence in the
[Platform 0.4/0.5 migration record](canic-adoption-history.md#platform-04-migration).
Use the current Platform 0.11 boundary below for new consumers.

## Platform 0.11 migration

Pending Auth 0.26 selects Platform/macros 0.11.1 and Wire 0.10.0. Align direct
Platform dependencies to `version = "0.11.1"` when exchanging Auth's owned
Principal or parse errors. Passive consumers retain only `principal`; hashing
and explicit upstream conversions retain their separate capabilities and
original bytes. Imported cryptographic primitives and strict verification rules
are unchanged. No signed/public Wire/Auth record migration or new reset is needed.

The qualification host now uses Platform's maintained executor and IC0 bindings;
no separate registry executor is selected. Native Testkit/Agent calls retain
their upstream external Principal boundary. Published Memory/Storage 0.58.0 and
Testkit 0.72.0 share Platform 0.11 with Auth. Neither needs a dependency override
or identity bridge.
Keep hosted endpoint, authority, storage and issuer acceptance with Canic.

Platform 0.11 selects its Cargo singleton guard only with the executor. Passive
Auth/protocol/Wire graphs exclude the guard, so using pure Auth functionality
does not reserve a runtime supplier. The qualification host selects the exact
coordinated guard; it contains no executor API. Metrics 0.21.0 keeps the same
counter/serialization code and uses Wire 0.10. Testkit's public limits align with
Auth's Wire 0.10; no schema or limit bridge is needed. The existing schema, frames,
signed bytes and retained record layouts are unchanged.

For native Principal/Account schema generation, replace Platform's retired
`wire-schema` feature with Wire's `platform-types` feature. Auth Wire's public
`typescript` feature forwards that selection and no longer declares Platform
directly. Wire owns the schema implementations for Platform's owned types,
removing the reverse dependency and its repeated release-alignment cycle.
This metadata capability selects neither runtime calls nor the endpoint feature.

## Platform 0.8 migration

Historical Auth 0.22 selected published Platform/macros 0.8.1. Its direct Platform
dependencies used `version = "0.8.1"` when exchanging Auth's owned Principal,
parse errors or imported key types. Type-only consumers select `principal` with
defaults disabled. Explicit upstream conversions additionally need
`principal-external`; direct `Principal::self_authenticating` calls need
`principal-self-authenticating`. Required client/signature features select the
conversion capability, and the Auth engine always selects public-key hashing.
Keep application identities owned and preserve original bytes at every boundary.

This dependency adoption preserves signed claims, current Wire storage records,
public transport, browser journals, verification checks and TEE trust. It supplies
no additional reset or retirement authority; the separately approved Auth 0.22
CBOR-record hard cut is described above. Auth selects none of Platform's changed
ICRC ledger DTOs or native profiler product code.

Published Memory/Storage 0.55.2 and native Testkit 0.70.0 also use Platform 0.8.1,
removing the earlier private Platform 0.7 branch. Storage/client edges retain
upstream stable-structures and external Principal identities; no dependency patch
or new bridge was introduced. That qualification used Platform 0.8.1 throughout.

## Platform 0.7 migration

Historical Auth 0.18 selects published Platform/macros 0.7.0. Align direct Platform
dependencies to `version = "0.7"` when exchanging Auth's owned Principal,
parse errors or canister-signature key types. Type-only consumers select only
`principal` with defaults disabled. Principal bytes, signed claims, Wire frames,
stored records and browser journal formats remain unchanged; no data reset is
required. Platform's removed Candid frontend compiler exports were not selected
by Auth.

Published Testkit 0.69.0 and qualification Memory 0.51.9 select Platform 0.7
alongside Auth; no Platform 0.6 package remains in the workspace lock. Existing
explicit conversions through `principal::ExternalPrincipal` preserve the native
Agent/Testkit/crypto boundary; the stable-structures backend identity is unchanged.
No dependency patch or second Auth Principal facade is needed.

The private browser client separately selects `@tokostudios/ic-platform` 0.1.1
over SDK Core 6.1.1. Use its explicit `/principal`, `/identity` and `/agent`
exports; the SDK request/expiry identities, trust roots, authorization, signed
bytes and retry/recovery policy remain with their existing owners.

## Platform 0.6 migration

Earlier Auth 0.13 adopts published Platform/macros 0.6.0. Consumers of that release sharing Auth's
owned Principal, parse errors or imported canister-signature key type must align
their direct Platform dependency to 0.6. Type-only consumers must select
`features = ["principal"]` with defaults disabled. Auth's protocol-types package
selects this explicitly; `runtime`, `cdk`, native clients and signature creation
also select it upstream. No runtime feature is needed for passive protocol types.

The independently published Testkit 0.68 selects Platform 0.6 too. Native
qualification imports simulator clients and DTOs directly from Platform;
Testkit retains startup, lifecycle extensions and Wire calls.
Auth keeps explicit external Principal conversions; no override or compatibility
facade is needed. The qualification host and all
public Auth capabilities at that release select Platform 0.6. Signature creation drops unused
upstream CBOR/delegation helpers; verification retains its required parser.
Signed claims, Wire frames, stored records, crypto policy and lifecycle semantics
remain unchanged; no storage or browser-journal migration is introduced.

## One application flow and host obligations

A fresh directly authenticated browser principal presents one portable credential
to each permitted receiver. Each receiver independently verifies admission and
commits session, replay and login reservation atomically. Subsequent operations
use the actual caller's local scoped session, with current protected authority,
clock and generation. A wallet/account proof, IC ingress delegation and application
resource grant are distinct. Account identity is `(issuer, subject)`; the browser
principal identifies this login/device.

Successful application authorization must preserve both `Session::issuer()` and
`Session::subject()` for account resolution. Returning the subject alone loses
its issuer namespace. Two approved issuers naming the same subject are distinct
accounts; qualify that case before enabling the successful facade path.

Canic owns fleet/member/role policy, issuer approval/renewal, protected Root/key
selection, enrollment freshness, real caller/time acquisition and scoped endpoint
guards. Its store implements Auth's transaction contract, including capacity,
failed-commit atomicity, replay and retained logout denials. Memory mode uses the
signed reset floors and starts closed; persistent mode restores complete checked
state and still requires fresh authority. See [sessions](sessions.md) and
[certification composition](signature-preparation.md). The fixture's IC Memory
checkpoint does not supply a production per-operation backend.

Preserve the typed `PortableClaimsError::NotYetValid` refusal through the host
endpoint and client adapter as a bounded retry hint, never authenticated success.
Auth 0.14 replaces `EstablishSession` error text with
`model::AdmissionRefusal`. Project the core error with
`SessionError::admission_refusal()` and convert it into the Wire type with
`.into()`; keep the original error in host diagnostics. The SDK adapter throws
`AdmissionError` with the generated refusal tag, rather than a generic transport
failure. Update receiver reply encoders and clients together; old text replies
are refused. Signed credentials, stored records and read/logout methods retain their existing
contracts. Auth 0.15 admission uses the new identity described above.
Keep strict admission starts and the original proof deadline; do not add clock
grace or automatically retry expiry, invalid signatures, stale authority or reset
floor failures. Consumer acceptance remains in
[Canic #418 — Future-start admission and retry diagnostics](https://github.com/dragginzgame/canic/issues/418).

The old Rust delegated-token issuance/verifier API and browser prepare/retrieve
client were retired in 0.9 after explicit no-deployed-user/no-unresolved-effect
confirmation. Use `ic_auth::delegation` for Root policy, proof verification and
errors, and `portable_certified::CertifiedProofLimits` for certified admission.
`token-verification` is retired. Generic signature-store prepare/retrieve,
certificate issuance rules and role-attestation primitives remain supported;
their similar names do not make them another application login flow.

Auth 0.17 generalizes ledger commit errors: `with_reservation<T, E>` and
`with_revocation<E>` require `E: From<LoginRevocationError>` and return callback
errors unchanged. Ledger-only callers with untyped `Ok(())` callbacks select
`with_revocation::<LoginRevocationError>` or
`with_reservation::<_, LoginRevocationError>` explicitly. Stored login records,
commit ordering and unknown-logout work charges keep the same contract.
Complete-verifier failures now use only `SessionError::Credential`, wrapping
`PortableCredentialError::Ed25519` / `Certified`; the direct session variants and
their `From` conversions are removed. Auth 0.19 admission owns the complete
verifier call; retain `admission_refusal()` projection. Consumer adoption belongs
to [Canic #491 — Extract authentication machinery into independent IC Auth](https://github.com/dragginzgame/canic/issues/491).

## Browser and production acceptance

Canic's selected issuer policy derives `account_subject` from the authenticated
issuance-call principal. It receives the fresh browser public key as a separate
request binding. Standard IC identity delegation retains the delegating caller;
that account principal must not become the receiver's browser identity. Issuing
for another account requires separate explicit authority. Receiver account lookup
still uses `(issuer, account_subject)`, not the browser caller alone.

Auth 0.14's coordinator therefore takes explicit `{ browser, account }`
identities. Its `issue`/`reconcile` callbacks receive a temporary issuer-bound
account signer; `admit` receives a receiver-bound browser signer. Both use the
same durable logout generation and bounded attempt. The issuer adapter binds the
original account caller, browser key, login, requested grants and exact request in
its saved intent before dispatch. Recovery cannot substitute the currently logged-in
account or infer a fresh request from missing receipts. The browser journal does
not duplicate that intent store or retain account signing keys.
Issuer logout authenticates the browser cleanup identity and resolves its retained
login binding; it must not reinterpret that browser caller as the account subject.
Losing account-provider access must not prevent browser-key logout. Losing the
browser cleanup key still has the documented residual revocation limits.

Use the portable admission coordinator and durable logout journal, or a consumer
adapter qualified against those contracts. Issuance intent precedes dispatch;
`reconcile` recovers the original work or refuses unresolved. Missing, pruned or
expired evidence cannot issue new credentials or extend the original deadline.
Logout durably invalidates the browser generation before fan-out, and only
authenticated acknowledgements mark receiver revocation. Preserve unknown effects.
See [browser contracts](browser-client.md),
[Auth #3 — Qualify portable browser admission/logout with a production issuer adapter](https://github.com/dragginzgame/ic-auth/issues/3)
and [Canic #507 — Portable issuance and non-issuing reconciliation](https://github.com/dragginzgame/canic/issues/507).

Toko owns account linking, frontend origin/key custody, project authorization and
its NFT ledger. Auth's native/Wasm tests and real-canister SDK fixture qualify the
library/reference host, not Canic's final facade, production issuer, native browser
or Toko deployment. Consumer acceptance must cover two receivers/devices, lost
replies, authority changes, logout, proof expiry and the selected upgrade/reset
profile. See [the current handoff](status/current.md) for source/evidence identity.

### Production adapter observables

The shared receiver transport is available as `httpAdmissionTransport`; Canic's
command envelope still needs its own qualified translation if it differs. The
issuer `issue`/`reconcile` endpoint contract remains Canic-owned. The producer's
injected issuer fixture is not a production endpoint or receipt-retention owner.
Use these observable outcomes when qualifying that actual adapter:

| Boundary | Required observation |
| --- | --- |
| Issuer/account and receiver/browser identities | The issuer authenticates the account caller and binds the separate browser key; successful receiver calls use the browser principal and preserve issuer-scoped account metadata. Logout/cancellation prevents late signing with either temporary capability. |
| Lost issuance reply and journal reopen | Recover only the original credential bytes and original admission deadline; issuance/signing counters stay at one. |
| Missing/pruned issuer evidence | Keep issuance unknown; no replacement signing, allocation or receiver submission. |
| Receiver admission and scoped operation | Bind actual caller/clock and protected authority to Auth's atomic session/replay/login store; successful scoped access must reach the real facade. |
| Logout, authority change and reset/restore | Retained keys cannot revive revoked sessions; peer/device state and original lease deadlines retain their defined independence. |

Auth's real SDK fixture now injects a lost issuer reply, reopens its journal,
retains unresolved lookup and recovers the original material before authenticating
to two actual local receivers. It proves coordinator/receiver composition, not
Canic's issuer lookup, production stable backend or native-browser persistence.
The SDK fixture also admits two browser devices for the same account to both
receivers. Each retains its own session and original lease; logging out one
device leaves the other active, and neither retained key can revive its revoked
login with a newly signed credential. The two devices use separate journals.
The fixture's authenticated account reads exercise distinct ingress signers;
they are read-only probes, not a portable issuer endpoint or account-login service.
Keep acceptance with [Canic #491 — Auth integration](https://github.com/dragginzgame/canic/issues/491)
and [Canic #507 — Non-issuing reconciliation](https://github.com/dragginzgame/canic/issues/507).

### Activation and software qualification

Canic's owner has kept live activation closed because no production issuer/receiver
clock-skew bound is qualified. Controlled clocks may qualify the software contract,
but fixture values, including zero skew, must not become production configuration
or evidence. Persist protected clock observations and reset/retention margins;
refresh receiver-bound Root authority before readiness. Keep admission starts
strict; a future-start hint supplies no grace or deadline extension.

TEE key/enrollment evidence is a separate requirement for TEE-required deployments.
Ordinary Ed25519 tests do not supply it. Continue software integration without
inventing TEE evidence, silently downgrading custody, enabling production or adding
a routine online issuer call to receiver admission. The actual runtime/store,
issuer receipt endpoints and browser distribution remain consumer/service work
under the existing trackers, not missing proof-verifier APIs.
