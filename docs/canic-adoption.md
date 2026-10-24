# Canic library adoption contract

Auth owns reusable authentication semantics; Canic owns the product host that
uses them. [Canic #491 — Extract reusable authentication, token and session machinery](https://github.com/dragginzgame/canic/issues/491)
is the implementation/acceptance tracker. This document is the current interface
boundary, not evidence that Canic or Toko have deployed it. Earlier extraction
checkpoints are retained separately in [adoption history](canic-adoption-history.md).

## Released API and dependency direction

Released Auth 0.15 contains `ic-auth-protocol-types`, `ic-auth` and
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

Auth 0.15 selects Platform 0.6 and enables only `principal` for passive types.
Consumers sharing that owned Rust type must align to Platform 0.6; external
Agent/Testkit/crypto APIs keep explicit `principal::ExternalPrincipal`
conversions. Do not select the older upstream Principal identity or downgrade
Auth to follow a historical example. No Candid or runtime capability is selected
by the passive edge. [The published-type acceptance](https://github.com/dragginzgame/ic-auth/issues/82#issuecomment-6312237287)
qualifies direct exchange with Platform 0.6.1 on Rust 1.91, native and Wasm.

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

Source inspection on 2026-10-24 used Canic's working tree based on
`cf2d3d1fba84a00a76cdbc541ae7e68fe36e9bb5`, including pending 0.112 Wire work.
These are source findings, not published Canic or installed-receiver acceptance.
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
still returns a denial, and the Auth-backed `application_sessions/shared` store
is compiled only for tests. Existing delegated issuer/verifier switches and
proof install/status APIs do not supply portable issuance or reconciliation.

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

Canic's draft `canic_raw_update(..., wire(id = ..., request = ..., response = ...))`
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

Auth 0.15 transport exposes IC Wire 0.6. Earlier Auth 0.14 uses
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

Native Testkit 0.65 and 0.66 take a `WireCallSpec` with the contract's `NAME`, `ID`,
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
Use the current Platform 0.6 boundary below for new consumers.

## Platform 0.6 migration

Released Auth 0.13 adopts published Platform/macros 0.6.0. Consumers sharing Auth's
owned Principal, parse errors or imported canister-signature key type must align
their direct Platform dependency to 0.6. Type-only consumers must select
`features = ["principal"]` with defaults disabled. Auth's protocol-types package
selects this explicitly; `runtime`, `cdk`, native clients and signature creation
also select it upstream. No runtime feature is needed for passive protocol types.

The independently published Testkit 0.66 selects Platform 0.6 too. Native
qualification imports simulator clients and DTOs directly from Platform;
Testkit retains startup, lifecycle extensions and Wire calls.
Auth keeps explicit external Principal conversions; no override or compatibility
facade is needed. The qualification host and all
public Auth capabilities select Platform 0.6. Signature creation drops unused
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
