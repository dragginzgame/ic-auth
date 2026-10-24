# Historical Canic adoption evidence

Archived on 2026-10-18. Earlier source checkpoints, proposals and measurements
below are preserved verbatim. Retired features and pending-publication statements
are not current guidance; use [the adoption contract](canic-adoption.md).

## Completed Platform identity migrations

The following Auth 0.11/0.12 migration notes were moved from the current guide
on 2026-10-24. Their dependency versions describe those releases, not the current
selection. Current consumers use the [Platform 0.6 contract](canic-adoption.md#platform-06-migration).

<a id="platform-03-migration"></a>

## Platform 0.4 migration

Released Auth 0.11 adopts published Platform 0.4.0. Auth's `Principal` path
is unchanged, but the type and parse errors now belong to Platform. Rust consumers
must align their application types to Platform 0.4; mixing its owned Principal
with the former upstream identity requires explicit conversion. Application and
runtime state use the owned type. Agent, Testkit and required external DTOs
keep their original types at those boundaries:

```rust
let application = ic_auth_protocol_types::Principal::anonymous();
let external: ic_platform::principal::ExternalPrincipal = application.into();
assert_eq!(ic_auth_protocol_types::Principal::from(external), application);
```

Conversions preserve significant bytes. Signing/self-authentication algorithms,
method frames and persisted profiles do not change. Frozen Auth 0.10.7 signed
claims and session/replay/login records remain exact and decodable; no reset is
authorized. Text longer than 63 bytes now returns Platform's `TextTooLong` before
normalization. Required Candid DTOs must keep the external Principal; the owned
type intentionally has no Candid trait or marker handling. Do not add a Candid
feature, a local Principal alias facade or a replacement cryptographic parser.
Testkit 0.57 still selects Platform 0.2 natively; Auth's explicit call conversions
support that released pairing without a forced Testkit upgrade.

Platform 0.4 owns the imported canister-signature key and verifier implementations.
`CanisterSigPublicKey` changes crate identity; its Principal field remains the
external type, so existing constructor conversions stay required. DER/raw bytes,
short-DER pre-admission and protected signer/seed/root/freshness checks are
unchanged. All Auth capabilities and reference Wasm now reject Candid and the
retired registry signature-package edges. Runtime RejectCode/raw-call convenience
changes affect no Auth call sites; the qualified host keeps explicit Wire codecs.
Native Agent/Testkit still select their independently maintained Candid graph.

## Platform 0.5 migration

Auth 0.12 introduced published Platform 0.5.0. Auth exposes its owned
`Principal`, so Rust consumers sharing that type must align their direct Platform
dependency to 0.5. Import paths stay unchanged;
0.4 and 0.5 are distinct Rust crate/type identities. Required external DTOs and
Agent/Testkit parameters still use `principal::ExternalPrincipal` with explicit
byte-preserving conversions. Consume Testkit and Memory's independently
published selections without a local identity compatibility facade or override.
Testkit 0.62 paired with Platform 0.5; Testkit 0.63 through 0.66 align to Platform
0.6. Older Testkit releases retain their own Platform identity.

Signed claims, frozen Wire frames, session/replay/login encodings and generated
TypeScript retain their bytes. This is a Rust dependency-identity cut, with no
journal/checkpoint reset or storage migration. The private host removes the
retired no-op `hidden` macro option while retaining explicit Wire hooks and its
existing endpoints. It uses none of the removed checkpoint/reader APIs or
`Call::change_timeout`; there is no replacement storage abstraction or wait-policy
change in Auth. The host still requires `cdk` for entrypoint generation; public
proof/type/session capabilities select no runtime feature.

# Canic library adoption contract

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns implementation
and acceptance. This contract describes the published IC Auth boundary; it is not
a second task tracker or evidence that Canic has adopted it. The
[accepted design](design/extraction.md) and
[incorporation review](design/canic-source-review.md) retain ownership and signed
encoding requirements.

The maintainer's broader goal is to reduce duplication through all applicable
shared libraries. This document covers authentication; the
[wider Canic owner review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6078242947)
records other shared mechanisms and their contract boundaries. Library selection
must leave one canonical mechanism while preserving the consumer's authority,
transactions and recovery obligations.

## Pending 0.9 Wire transport

Auth 0.8.0 removed Candid traits from public domain types. Pending 0.9.0 adds
`ic-auth-wire`, an Auth-owned adapter over published IC Wire 0.3.4. The new package
is implemented locally, not yet published; do not select a sibling path or copy
its implementation into Canic. Root package versions remain 0.8.0 until release.

After publication, select the matching Auth/types/Wire packages. Use the generated
bounded credential, admission, proof-free read and logout contracts; Canonical
signed claims and session/replay/login record bytes remain unchanged. Protected
Root/issuer enrollment, actual caller/clock, the atomic session backend, scoped
guards, macro exports and delivery/retry policy remain Canic-owned.

Bind the Method request/response types to actual consumer exports. Admit raw
argument/reply sizes before copying, decode with the shared method identity before
authentication/dispatch and encode its bounded response. Encode failure after local
mutation must trap; normal proof refusals must retain admission-budget charges.
No Candid selector or fallback exists in the application client. Platform/third-party
Candid and IC certificate verification are independent obligations.

The maintainer confirmed no deployed old browser token-store records or unresolved
old issuance on 2026-10-17. That old prepare/retrieve client and format are retired.
New portable journals still preserve unknown issuance and generation invalidation;
production `issue`/non-issuing `reconcile` remain required under
[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3) and
[Canic #507](https://github.com/dragginzgame/canic/issues/507).

The reference host qualifies Wire admission/logout and both proof mechanisms,
not Canic's final facade or production issuer. `apps/qualification/contracts/`
now contains only private Candid controller/certificate projections and must not
be imported into production. Wire's host TypeScript feature stays outside the
canister selection; see [browser contracts](browser-client.md).

## Pending 0.9 Rust token retirement

The maintainer confirmed no deployed users or unresolved operations for the old
Rust delegated-token flow. Its issuance/verifier API, token-only DTOs and canonical
claim encoders are removed; there is no compatibility feature or alias. Select
`certified-proof-verification` in place of `token-verification`. Move Root policy,
Root proof verification and errors to `ic_auth::delegation`; use
`portable_certified::CertifiedProofLimits` for the complete certified adapter.
Both proof mechanisms continue to use the same session engine and unchanged
signed portable/certificate/proof bytes.

Canic's Root installation and error projections need the new imports/typed causes.
`AuthRequestMetadata`, proof hashes and role-attestation/signature APIs remain.
Toko's older calls are pinned to Canic `=0.110.48`; coordinate their application
adoption in the existing trackers before selecting the new API. This source cut
performs no deployment or retained-data reset. [Auth #69](https://github.com/dragginzgame/ic-auth/issues/69)
records the accepted retirement. Generic Wire test calls are requested in
[Testkit #119](https://github.com/dragginzgame/ic-testkit/issues/119); Auth keeps its
small runnable helper until that replacement is published and qualified.

## Released 0.8 public-type boundary

`ic-auth-protocol-types` exposes no Candid traits or optional Candid capability.
`Principal` remains the same `ic_principal::Principal`. Default, Ed25519,
chain-key-only, local-session and Wire normal/build graphs reject Candid. The
optional upstream certified-signature stack still uses Candid transitively.

## Historical dependency and adoption checkpoints

The following sections retain historical release/adoption evidence. Their retired
token APIs are superseded by the pending 0.9 contracts above.

### Historical 0.7.5 adoption checkpoint

Select published `ic-auth` and `ic-auth-protocol-types` 0.7.5 from the root
dependency catalog. Both packages require Rust 1.91.0; the portable API introduced
in 0.7.1 is unchanged. The reviewed Canic checkout at
`600a831e0` selects 0.7.5; its offline canonical-role crypto closure gate passes.
Production endpoint and storage acceptance remains with
the consumer. Older checkpoints below are historical
evidence, not alternative supported application-session contracts.

```toml
[workspace.dependencies]
ic-auth = { version = "0.7.5", default-features = false }
ic-auth-protocol-types = "0.7.5"
```

The published 0.7.4 selection does not satisfy Canic's deployed-Wasm dependency
policy: K256 0.14 and Dalek 3 introduce SHA-2 0.11 alongside the IC signature
stack's 0.10. The published 0.7.5 fix selects K256 0.13.4, Dalek 2.2 and
Curve25519 4.1.3, retaining the existing verification contract and no public
upstream types. The Wasm boundary check now rejects duplicate crypto support
packages with all Auth capabilities selected. Canic's actual gate passes all 12
canonical roles, including nine signature-free roles, at the reviewed selection.
These older dependency selections
satisfy that build policy; the authentication protocol itself does not require
these library versions.
[Auth #65 — Align crypto dependencies for deployed Wasm](https://github.com/dragginzgame/ic-auth/issues/65)
records producer alignment and the actual consumer graph acceptance. Neither
graph check qualifies the production endpoint/store or successful facade budget.
No credential, checkpoint or Candid regeneration/reset is required for this fix.

The [real SDK admission example](ic-testkit-qualification.md#sdk-admission-through-the-portable-coordinator)
now exercises two reference receivers, an exact retry, proof-free calls and local
logout through `PortableAdmissionClient`. Its private adapter maps the fixture's
endpoint contract; Canic must map its own endpoint and authenticate the successful
reply. Issuance is injected, and production renewal/reconciliation remains open.
This example adds no new backend API or version prerequisite beyond the current
portable contract; it does not qualify Canic's actual endpoint/storage path.

Child packages inherit the catalog selections and forward only their capabilities.
`sessions` selects the engine without a verifier; `portable-signature-verification`
selects Ed25519 and `token-verification` selects complete certified verification.
Add `session-restore` only to roles that need checked stored-record restoration.
The host selects protected authority and calls the matching verifier; enabling a
feature does not enroll a key or authorize a submitted credential.

Use the Rust-owned `PortableCredential { claims, proof }` and `PortableProof`
data contracts at the portable credential boundary. In the shared admission
callback, consume the supplied decoded claims with
`decoded.verify(budget, &credential.proof, &verification_context)`. The released
0.7 callback is `|budget, decoded|`; the prior one-argument callback must change.
The protected enrollment selects the complete verifier; the envelope supplies no
Ed25519 issuer key. Standalone `portable_credential::verify_credential` still
validates raw credentials independently, outside session admission. Browser descriptors and bounded `credentialBytes`/`credentialFromBytes`
are generated from that same owner. Endpoint names, argument wrappers, issuer
operations and browser session orchestration remain consumer-owned integration.


The [0.6 portable authentication design](design/0.6-portable-authentication.md)
selects certified or enrolled Ed25519 proofs through one session/guard path.
The proposed `enrolled_ed25519` mode separates signing custody from verification;
`auth-tee` is optional deployment assurance, not a prerequisite for Ed25519.
Local issuer keys live only in heap and require fresh Root enrollment on upgrade.
Canic owns protected deployment/key enrollment and explicit operator-trust or TEE
policy. These are pending capabilities, not current config/build options. Ordinary
subnets and local tests remain usable without claiming hardware confidentiality.
Browser logout contacts each receiver directly and revokes renewal at the issuer;
expiry bounds missed notifications. Preserve login-level tombstones against late
admission and already-issued renewal credentials. #491 owns consumer acceptance;
the earlier issuer-call experiment is superseded as a multi-subnet recommendation.

The [reviewed protocol](design/0.6-auth-protocol.md) now fixes the browser
principal/account distinction, canonical credential bytes, proofless caller logout,
issuance and post-await effect ordering, monotonic enrollment and exact reset
profiles. Memory reset uses a signed login-start cutoff, so renewed pre-reset
logins also refuse; persistent normal upgrades retain revocation. All boot paths
require fresh authenticated enrollment before serving. The
[issuer enrollment contract](design/0.6-tee-enrollment.md) names Root approval,
volatile-key lifecycle, optional TEE evidence, freshness and key retirement.
IC Auth supplies the portable engine, checked storage codecs and reset-floor
calculation. Canic must
implement protected enrollment, atomic durable storage or a qualified volatile
reset profile, and readiness; a feature toggle or trait implementation alone
does not establish those host guarantees.

The maintainer selected one application-authentication flow on 2026-10-13:
`establish_session` now admits portable claims through either complete verifier,
atomically consuming replay with the retained login reservation;
ordinary endpoints call `authorize_session` using actual ingress identity and
current protected authority. Use `logout_session` for caller-authenticated local
revocation; do not directly remove a stored session and acknowledge logout.
The portable record distinguishes browser caller from `(issuer, subject)` account
identity. Old token-session records are rejected. See [the session API](sessions.md)
for callback, transaction, reset and feature-selection requirements.
Do not call the standalone verifier first and then repeat verification during
fresh session admission.

Canic must retire its token-per-call `auth::authenticated()` application guard and
the raw/decoded token argument entry surfaces when its callers adopt session
guards. Regenerate the affected Candid and client contracts in that same consumer
cut. Toko must replace token-prepending application calls with session admission
on each selected receiver, then calls signed by its existing session identity.
Preserve registration, user resolution, resource ownership and role policy.
Remove unused parallel bootstrap/renewal wrappers after tracing their callers;
do not remove issuer preparation, Root authority or infrastructure attestations.

A missing, expired or invalidated session refuses the application operation.
An explicit admission/renewal step may restore eligible access; there is no
implicit token-per-call fallback or repeated wallet prompt for each receiver.
Admission still verifies each receiver's protected fleet/role/scope requirements.
Coordinate retained sessions and replay evidence under the owning lifecycle
contract; a hard cut is not permission to erase state.

The accepted 0.6 storage choice is per-role **build configuration**, not a new
IC Auth dependency on Canic. Map memory mode to `MemorySessionStore` with
`sessions`; map persistent mode to Canic's durable `SessionStore` with
`session-restore`. Do not unconditionally enable the restoration feature from
Core if memory-only roles must omit its Wasm. Reject unsupported build/config
combinations rather than silently selecting another backend. Runtime toggles
that retain both implementations cannot claim decoder exclusion.

Reset cutoffs are fields of `PortableClaimsContext`, not `SessionPolicy`.
After losing session/replay/revocation state, call
`PortableClaimsContext::advance_reset_floors(qualified_clock_skew_ns)` once using
the protected boot time, then retain both resulting floors before readiness.
The proof-issuance floor is at least `boot_time + 1`; the immutable login-start
floor also includes the qualified clock skew. Neither floor may decrease.
Retain previous stricter floors, reject overflow/clock rollback and require fresh
authenticated enrollment before serving. Never calculate these cutoffs per call.
A fresh proof for a pre-reset login remains ineligible: the browser needs a fresh
login principal and login window, not just re-signing old claims. Durable normal
upgrades retaining all three record sets preserve their existing floors. Follow
the [complete lifecycle contract](sessions.md#storage-selection-and-upgrade-contract-for-06).

Consumer acceptance must cover memory-mode upgrade logout, refusal of an old
consumed but unexpired token after upgrade, fresh signed re-admission, persistent
session/replay restoration, disabled-backend state not being resurrected, and
per-role resolved features/release Wasm. Toko recovery must start a new browser
login when the immutable login-start floor rejects the old one. Account authentication may reuse an eligible existing
wallet/identity login; it must not preserve the old browser caller as a new login.
Memory-mode receiver upgrades therefore log out that receiver's existing users.
Client orchestration must ensure admission per receiver/identity/scopes before
ordinary calls. One fleet-wide login flag is insufficient. Reuse the same valid
credential across permitted receivers. Proof expiry requires fresh issuance within
the immutable login window; a receiver reset floor can require a new browser login.
Neither event itself invalidates another receiver's retained session. Retry only
bounded admission/recovery and never blindly replay application mutations
with uncertain outcomes. See [multiple receivers](sessions.md#multiple-receivers).

Canic config syntax, endpoint wiring and actual host lifecycle qualification are
pending consumer work in #491. The Auth fixture exercises actual canister
upgrades with volatile reset or explicit checkpoint restoration; neither mode
qualifies Canic's production lifecycle or durable transaction backend.

The read-only Toko checkpoint `44d4e2c6d41e3575b2503e79ffa369129ff3ecf2` selects
Canic 0.110.48 and actively uses token-per-call guards. Its positive proof cache
is the historical performance baseline, not uncached BLS verification on every
call. The [Wasm comparison](https://github.com/dragginzgame/canic/issues/491#issuecomment-6149634805)
and [size attribution](https://github.com/dragginzgame/canic/issues/491#issuecomment-6149738317)
support session adoption for repeated calls but do not qualify the actual durable
Canic backend or predict Toko's final Wasm size. Endpoint, frontend, lifecycle
and performance acceptance remain in the owning issues.

`Session::decode` is not called by the session engine itself. A durable Canic
adapter needs it when restoring `shared_session` bytes. Keep checked
restoration on that path; moving it to an upgrade hook still retains the decoder
in Wasm. The 0.6 flow decision does not select a new codec or a heap-only lifecycle.

### Historical selections before the portable cut

Both `ic-auth` and `ic-auth-protocol-types` 0.5.9 are published and unyanked.
Their registry checksums match IC Auth release
`75d8dae2ac71ce12a628929be2d0cd5bad30fbfb`. Rust 1.88 is the supported minimum.
The pure preparation API requires at least 0.4.2; 0.5.9 preserves its Rust contract.
Select compatible registry requirements in Canic's
root catalog and record the resolved versions in its own lockfiles:

```toml
[workspace.dependencies]
ic-auth = { version = "0.5.9", default-features = false }
ic-auth-protocol-types = "0.5.9"
```

Child packages inherit these dependencies with `workspace = true`; their
existing capability features forward only the needed library features. This was
the pre-cut selection; use the current portable selection above for new session
adoption. The read-only review below distinguishes Canic's
in-progress source adoption from consumer qualification.

Releases 0.3.0 through 0.3.8 change developer tooling and qualification; their
authentication APIs and signed bytes are unchanged from 0.2.11. Release 0.4.0
changes typed Root rejection causes, requiring updated exhaustive matches and
diagnostic projection; signed bytes and successful acceptance are unchanged.
Canic's owner-reported preparation qualification selects 0.4.2 at its recorded
checkpoint below. A later read-only lock inspection resolves 0.4.5 without
establishing new consumer qualification. Selected versions alone do not
establish full runtime or deployment acceptance. The consumer now reports
Auth 0.5.0 adoption in its held integration batch for pure request hashing,
certificate issuance and enabled chain-key checks/normalization, with direct
production K256 retired from Core. Its [scoped native/Clippy/Wasm evidence](https://github.com/dragginzgame/ic-auth/issues/32#issuecomment-6120465762)
and [extraction handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6120497751)
do not establish consumer publication, deployment or complete session-engine
acceptance. Compatible Auth 0.5.1 through 0.5.9 introduce no new API prerequisite.

The [read-only verifier review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6126777217)
finds Canic's dirty root lock selecting published 0.5.4. Its complete-token
adapter now calls `token::verify_token` with protected `allowed_token_scopes`,
enrolled Root policy and explicit freshness/material bounds; its copied verifier
is removed. Root/issuer wrappers share one host-owned SignatureStore. The new
durable SessionStore backend is present. In a
[later source checkpoint](https://github.com/dragginzgame/canic/issues/491#issuecomment-6126878501),
the session endpoint reaches a workflow calling `session::establish_session`,
and a new authorization wrapper calls `session::authorize_session`. Final backend
and guard compilation and durable transaction/lifecycle acceptance remain
Canic-owned; changing partial source does not establish a coherent adapter.
The [subsequent guard review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6127256752)
found the facade calling that wrapper alongside a duplicate local precheck.
That checkpoint is historical. The
[current integration review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6141222338)
finds Canic's root and both Blob consumer locks on published Auth 0.5.7, with
the production guard running
the SDK before Canic subject/scope denial classification. Admission commits one
stable session/replay pair with incremental indexes. Logout and other full-state
operations still require their own resource/lifecycle proof under
[Canic #380](https://github.com/dragginzgame/canic/issues/380).

Retained owner storage, Clippy and managed recovery evidence has advanced.
The changing tree is not a released/deployed consumer qualification, and no
sibling build/edit is performed here. The measured maximum-state authorization
budget failure and actual-facade denial coverage are tracked in
[Canic #525](https://github.com/dragginzgame/canic/issues/525).
The existing pure denial probe does not exercise the production SDK branch.
The [consumer owner now reports](https://github.com/dragginzgame/canic/issues/525#issuecomment-6130059637)
989,121 maximum-state authorization instructions without raising its 1M limit,
plus one/median, cleanup and recovery passes; actual-facade single/combined denial
acceptance remains open. Producer
[Auth #43](https://github.com/dragginzgame/ic-auth/issues/43) adds an isolated
actual-Wasm SDK decode/live-check probe to help attribute future regressions.
One-scope records measure about 53k instructions, and the maximum-width scope
record about 211k in the final gate (focused runs reached 292k). This excludes
consumer context/stable-row projection and is not a complete Canic budget pass.
[Canic #419](https://github.com/dragginzgame/canic/issues/419) retains the host
cleanup-custody finding: physical expired rows count until bounded explicit
pruning, which must not be stranded by unrelated intent failure. Browser adapter
qualification independently requires non-issuing reconciliation in
[Canic #507](https://github.com/dragginzgame/canic/issues/507).

The earlier 2026-10-13 audit observed local Canic 0.112.0 plus held changes, while
remote main remained 0.111.0. It also reconfirmed
[Canic #506](https://github.com/dragginzgame/canic/issues/506#issuecomment-6141222675):
ROOT must retain validated issuer-effective installation deadlines in renewal
state. The current coordinator discards the successful typed response and stores
declared deadlines. This remains consumer-owned alongside final facade, cleanup,
logout and browser acceptance; Auth 0.5.9 adds no required API for those repairs.

The follow-up after Auth 0.5.9 publication reviews clean local Canic commit
`ce45935307e1f95a7f72ea3e338b19b509e17210`. Official remote main remains the
0.111.0 release at `25b73cd403b20b4e148c6e340f8c760a26d97f6e`. The root lock
selects Auth/protocol-types 0.5.8; both independent Blob consumer locks select
0.5.7. These are compatible selections, not proof of one qualified graph. The
same cleanup/renewal/logout code and pure denial probe remain present at this
committed checkpoint. Browser prepare still enters reserve-or-replay and fresh
issuance; it does not supply the required non-issuing reconciliation adapter.
The existing owning issues retain their acceptance requirements. Fourteen file
hashes and the three selected lock graphs are retained in
`target/integration-0.5.9-live/canic-source.json`; this review does not run Canic.

A later working-tree checkpoint at that HEAD selects Auth/protocol-types 0.5.9
in the root, with both Blob consumer locks still on 0.5.7. The
[production denial follow-through](https://github.com/dragginzgame/canic/issues/525#issuecomment-6143196133)
identifies a narrower acceptance gap: protected verifier/TTL resolution precedes
session lookup and projects every resolution error to StaleAuthority, even with
a missing or expired session. The pure policy has no Root-resolution input and
cannot qualify those combinations. Canic owns the normative facade precedence,
unavailable/stale cause projection and actual endpoint tests; Auth's engine
contract remains unchanged. All source-traced paths deny, with no runtime bypass
reproduction claimed. The issue contains the exact matrix and owner repair choices.

Published Auth 0.5.9 includes canonical-scope consolidation and live signature
retry cleanup, preserving APIs, errors and stored/signed bytes. Its
[exact-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38379505548)
is running on Linux with both macOS jobs queued at inspection. Prior 0.5.8 CI
was cancelled after its Linux pass; it does not complete #43–#45 native acceptance.

Producer 0.5.4's [complete native matrix](https://github.com/dragginzgame/ic-auth/actions/runs/38247803905)
qualifies its retained library and actual canister fixture on Linux, Intel macOS
and ARM macOS. Published 0.5.7's [own run](https://github.com/dragginzgame/ic-auth/actions/runs/38284363288)
passes all three native hosts; 0.5.6's prior run was cancelled after its Linux pass.
Neither proves Canic's changing endpoints or persistence. Compatible 0.5.7
adopts contributor tooling and adds no new library API prerequisite beyond
Canic's selected 0.5.7. Durable store, rollback/restart/upgrade and deployed
acceptance stay with the consumer owner.

[Canic #533](https://github.com/dragginzgame/canic/issues/533#issuecomment-6144678469)
now owns a distinct cross-build state-preservation audit. Released 0.111's auth
cell contains session/replay arrays; the current committed candidate separates
rows and requires SDK session bytes. Auth does not decode the old Canic aggregate
or establish missing historical Root bindings. Canic must qualify the exact
retained memory/codec transition, preserve consumed-proof history and choose an
authorized session disposition alongside application-state preservation. Neither
same-release recovery nor a compatible Auth package patch proves that transition.

| Existing Canic capability | IC Auth selection | Retained Canic responsibility |
| --- | --- | --- |
| Protocol DTOs and canonical encoding | Protocol-types and default `ic-auth::canonical` | Fleet/configuration envelopes and checked deployment-identity projections |
| Pure delegated-token issuance | Default `token_preparation` in published 0.4.2 | Installed authority, authenticated presenter, host bounds, operation custody and authorized signing |
| Root/issuer canister-signature verification | `canister-signature-verification` | Protected signer, seed, network trust anchor, time and limits |
| Root/issuer canister-signature creation | `canister-signature-preparation` | Authorized message construction, runtime certificate, certification-root composition and lifecycle |
| `auth-delegated-token-verify` | `token-verification` | Independent key enrollment, fleet/role policy, authenticated caller and endpoint requirements |
| `auth-local-application-authorization` | `sessions` | Durable transaction backend, memory IDs, generation transitions and endpoint guards |
| Root chain-key signing and renewal | Existing Canic effect adapter | Management-canister signing, key eligibility and orchestration; there is no library signing endpoint |

Token verification enables signature and chain-key verification. Since the
portable cut, sessions select neither verifier; select only the proof mechanisms
needed by the receiver. Signature preparation is independently selectable. No
feature imports Canic, Toko, Solana, CDK, stable memory or native Host/Testkit tooling.
Library adoption does not remove a cryptographic dependency from the transitive
graph merely by moving its direct declaration.

## Published chain-key checks and normalization

Published 0.4.7 implements [pure SEC1 admission and compact prehash verification](chain-key-signatures.md)
under `chain-key-verification`, without selecting certificate verification.
Canic can select 0.4.7,
forward its existing ECDSA capability, and replace the parsing/verification in
`ops/auth/delegated/chain_key.rs` used by `chain_key_signing.rs` and
`token/verifier_config.rs`. Keep the original signing hash, protected key
selection, unsupported-algorithm and feature-disabled refusals, and local error
projection. Complete Root verification retains its existing error surface.

[Auth #29](https://github.com/dragginzgame/ic-auth/issues/29) owns this extraction;
[Canic #491](https://github.com/dragginzgame/canic/issues/491) retains adoption.
Configuration schema key admission can use the same public helper. Released 0.4.7
under [Auth #31](https://github.com/dragginzgame/ic-auth/issues/31) adds explicit
`normalize_secp256k1_signature`, returning compact low-S bytes through K256.
The reviewed `normalize_chain_key_ecdsa_signature_enabled`
can replace its conversion with this function and project the returned array to
its response vector. Retain the producer's decision to normalize management
replies, then verify using its original hash and protected key. Strict submitted
proof verification still rejects high S. Normalization success is not signature
authentication. Audit remaining production/test uses before moving K256 to
test-only dependencies. This does not qualify management replies, enroll a key,
decide certificate freshness/session leases or establish consumer acceptance.

The larger Canic offloading release must resolve its exposed upstream type
families under [Canic #517](https://github.com/dragginzgame/canic/issues/517).
This additive producer patch does not classify the consumer's release boundary.

## Canonical declarations and protected inputs

Released 0.4.7 adds [pure certificate issuance validation](certificate-issuance.md)
under [Auth #30](https://github.com/dragginzgame/ic-auth/issues/30), with no optional
feature required. Replace `delegated/cert_rules.rs` mechanics
used by certificate construction and pure token-material validation with
`validate_certificate_issuance`, passing independently resolved Root and explicit
TTL/grant/scope bounds. Map distinct lifetime and grant causes to the existing
consumer refusals; validate Canic role labels during the passive DTO projection.
Keep shared canonical rejection order, signed bytes, protected policy resolution,
issuer approval, signing and clock checks. Shape success does not establish any
of those authorities. Eager raw DTO role projection precedes shared validation.
Canic's owner reports accepting that diagnostic order as a pre-1.0 hard cut,
retiring duplicate rules and qualifying an invalid-role/invalid-lifetime case
with zero proof callbacks. See the [producer feedback](https://github.com/dragginzgame/ic-auth/issues/30#issuecomment-6120497461).
This resolves the earlier raw DTO order choice in its held integration batch;
matching delivered consumer acceptance remains under Canic #491. No sibling
source was changed or built here.

Use the protocol package's token, proof, certificate, audience and grant types
as their canonical declarations. Remove superseded generic Canic declarations;
retain Canic's infrastructure/configuration DTOs and endpoint error mapping.
Coordinate exposed facade/macro/Candid surfaces with
[Canic #354](https://github.com/dragginzgame/canic/issues/354). A wire-compatible
data shape does not establish unchanged Rust imports or error semantics.

Project protected network and fleet IDs into `AudienceId` byte-for-byte through
`CanonicalId`; parse the protected role as `AuthRole`. Preserve the `Fleet`
Candid label and exact canonical domains/vectors. These validated identifiers
carry protocol identity, not fleet membership or authority. Fallible canonical
hashes propagate their errors; do not add infallible wrappers or duplicate
encoders to preserve the previous internal API.

Call `token::verify_token` with a `TokenVerificationContext` assembled from the
actual authenticated caller, protected local audience/role, scope ceiling,
endpoint requirements, independently enrolled `RootKeyPolicy`, IC network key,
clock and finite limits. Never populate expected authority from the token under
verification. Key name, SEC1 bytes and derivation path are protected enrollment
facts; a submitted chain-key proof is not their provenance. The
[token contract](delegation.md) specifies exact binding and live-policy checks.

Replace the generic proof callbacks and semantic/cache-only admission path with
the complete verifier. Its `VerifiedToken` cannot be publicly constructed or
deserialized. Do not retain a second positive proof cache that bypasses changed
policy, key windows or epochs. Fleet member/role decisions, issuer approval,
renewal orchestration and application resource authorization stay in Canic.

## Pure issuance preparation (released 0.4.2)

[Auth #25](https://github.com/dragginzgame/ic-auth/issues/25) adds the default-feature
`ic_auth::token_preparation` boundary described in the
[preparation contract](delegation.md#retired-delegated-token-lifecycle). Canic can project its protocol DTOs
and protected host inputs into `PrepareDelegatedTokenInput`, retain the immutable
result, sign `claims_hash()`, and call `finish_delegated_token` after authorized
proof retrieval. Canic now requires published 0.4.2 and its owner records
[204 focused Auth/workflow cases, five wire cases and enabled/disabled Clippy](https://github.com/dragginzgame/canic/issues/491#issuecomment-6109065011)
in its uncommitted 0.111.1 draft. The duplicate pure preparation module is removed.
This is owner-reported native adoption; managed/Wasm/endpoint acceptance and
deployment remain separate. IC Auth performs no consumer build or edit.

Canic's current grant/scope ceilings are 16/32. Pass those existing host policies
as `TokenPreparationLimits`; they are not Auth defaults. Project canonical role
order errors into the existing noncanonical-grants diagnostic and invalid scope
errors into its rejected-scope diagnostic. Other shape/deadline/subset failures
remain typed. Update field reads to the prepared value's shared accessors rather
than recreating a mutable claims/hash pair or a second issuance engine.

The helper preserves the exact nonce domain, claims bytes, proof attachment,
rejection order and requested TTL. The owner still resolves the installed proof
and independently enrolled authority deadline, authenticates the presenter,
retains original operation/clock inputs and authorizes signing and certification
publication. This extraction is independent of Canic's complete-token freshness,
protected scope ceiling and committed-session lease decisions. Local Auth tests
and its actual certified-token fixture do not establish Canic endpoint adoption.

## Pure session request identity (released 0.5.0)

[Auth #32](https://github.com/dragginzgame/ic-auth/issues/32) exposes
[`session_request::SessionRequest`](session-requests.md) without optional features.
The always-compiled adapter can replace
`ApplicationSessionWorkflow::establishment_request_hash` with the shared request
hash while retaining existing typed scope/TTL prechecks and diagnostic order.
The exact Canic V1 domain, framing and absent/explicit TTL distinction remain.
This requires no token verifier, enrolled key, clock or session store.

0.5.0 removes the former `session::SessionRequest` import and changes construction
errors to `SessionRequestError`; update imports and error projection. The optional
session engine uses this same owner without changing its stored format, hashes or
lease/replay policy. Canic retains protected generation, authenticated endpoints,
operation identity and committed lease policy. Consumer qualification belongs in
#491; its exposed upstream release boundary remains #517.

## Durable sessions and certification

Use `session::establish_session` for portable admission, `authorize_session` for
ordinary scoped calls and `logout_session` for local revocation. Construct the
canonical scope/TTL request with `session_request::SessionRequest::new`.
Proof expiry, the committed lease and login revocation have separate owners:

| Deadline | Current portable contract | Host obligation |
| --- | --- | --- |
| `PortableClaims::admission_expires_at_ns` | Exclusive fresh-admission deadline; retained as `Replay::remove_at_ns()` | Preserve consumed-proof history until its authenticated deadline; an exact retry may return the same live session afterward |
| `Session::expires_at_ns()` | Requested/default lease capped by immutable login deadline and protected authority/enrollment expiry | Return the unchanged lease on exact retry, restore and ordinary authorization; refreshed enrollment must not extend it |
| `LoginRevocationChange::until_ns()` | Login reservation/tombstone retention, including the qualified clock-skew margin | Persist it with session/replay changes; proof expiry or local session removal must not discard revocation |

`SessionPolicy` carries generation, enablement, subject eligibility and TTL
policy. `PortableClaimsContext` carries current enrollment, issuer/mechanism/key,
grants, caller/time, validity and reset floors. A proof's fields cannot supply
those protected expectations. After a stored session lookup, use its issuer,
`mechanism()` and `key_binding_id()` only to select the exact current protected
enrollment; then call `authorize_session`. Decoding or selecting a row is not
permission to grant access. Account identity remains `(issuer, subject)`.

Canic implements `SessionStore` and `SessionTransaction` against its actual
durable host store. Serialize the complete engine call, both store transactions,
the login ledger and authority without await/reentrancy. Persist the proposed
`LoginRevocationChange` atomically with the touched session/replay records,
indexes, counters and accounting. A `None` change permits reads only; admission
requires a matching reservation, and logout a matching revoked removal. Use
`Session::check_login_change` rather than copying private association checks.
Closure or commit failure leaves the durable state unchanged. Keep failed
verification credit charged even on subsequent storage failure; do not trap
ordinary refusals and roll back charges.

`MemorySessionStore` can serve an explicitly selected volatile reset profile.
Its `transaction_with_commit` hook can stage touched records for a durable host,
but supplies no stable allocation, transaction or protected metadata itself.
Preserve replay through logout/replacement and advance generation atomically
with invalidating authority changes. All boot paths refuse admission and ordinary
access until fresh authenticated enrollment. See the [session contract](sessions.md)
for bounded restoration/pruning and memory-mode reset requirements.

The [executable receiver reference](../apps/qualification/canister/src/sessions.rs)
uses both complete verifiers through one admission path. Its
[upgrade and multi-receiver regression](../apps/qualification/runner/tests/signatures/sessions.rs)
checks restored sessions and tombstones, an expired-proof exact retry with the
original lease, rejection of newly signed material for a revoked login, and an
unaffected peer receiver. Checkpoint persistence is fixture-only: production
per-operation durability, Root enrollment and final Canic facade costs still
require consumer acceptance.

IC Auth's `Session::encode`/`decode` is its current record format, not a reader
for existing Canic stable records. Canic owns the frozen-format inventory,
retained-state disposition and release boundary under
[Canic #459](https://github.com/dragginzgame/canic/issues/459). Resolve those
obligations before a storage cut; neither library adoption nor pre-1.0 status
authorizes resetting retained authority or replay evidence.

Signature preparation uses `SignatureStore`, whose `root_hash` is one labeled
`/sig` contribution. Canic composes and publishes the single certification root,
supplies real query certificates and matching `SignatureSibling` witnesses, and
owns pending retrieval behavior on upgrades. The store has no durable restore
codec. Its [preparation contract](signature-preparation.md) and the unpublished
[Testkit fixture](ic-testkit-qualification.md) do not qualify a Canic lifecycle
adapter or durable session backend.

The released 0.5.3 qualification under
[Auth #35](https://github.com/dragginzgame/ic-auth/issues/35) exercises existing
Root role-attestation and issuer-token seed/domain pairs together in one store,
with actual certificate verification, shared capacity, profile-isolated expiry,
composed branch updates and upgrade re-preparation. The required library API is
already published in 0.5.2; this adds evidence, not a new consumer prerequisite.
[Canic #60](https://github.com/dragginzgame/canic/issues/60) owns
shared-host adapter acceptance after replacing its separate Root/issuer
SignatureMaps and direct wrapper certification-root publication.
Keep one host-owned `/sig` tree containing both profiles, then compose that
contribution with application branches. Forking two identically labeled `/sig`
owners is not the qualified composition. Canic must independently choose fixed
profiles for each authorized operation, retain caller/operation metadata and
cleanup scheduling, and review the shared physical budget and original duplicate
deadline behavior. After upgrade, republish the composed root before queries and
apply the consumer's explicit pending-operation disposition. Library leaf cleanup
does not revoke already-issued proofs, enroll an issuer or confer application
resource authority. No Canic lifecycle or certified HTTP adapter is qualified here.

## Historical pre-portable consumer acceptance evidence

These checkpoints describe earlier token contracts. Use the current portable
contract and session APIs above for new integration; passing historical tests
does not establish acceptance of the current consumer graph.

The maintainer's immediate priority is Canic adoption from the already-published
libraries. Published 0.2.0 carries complete proof-transport qualification and
the developer-tooling hard cut; authentication APIs and signed bytes are unchanged.
The earlier read-only inspection at Canic committed base
`25b73cd403b20b4e148c6e340f8c760a26d97f6e` finds its changing working tree
selecting compatible `0.3` requirements and resolving both libraries at
0.3.8 with matching registry checksums. Complete token verification is already available. Consumer dependency
changes require its own qualified lockfiles and remain owned by Canic.

Released Auth 0.3.3 adds
[complete-token Testkit qualification](ic-testkit-qualification.md), joining the
published bounded Merkle builder and complete verifier with an actual certified
issuer. Live caller/audience/scope, root deadline/epoch, network-key and certificate
freshness rejection, followed by upgrade and re-preparation of the same claims,
pass locally. Root ECDSA signing is native test-only signing. This reinforces the
existing adoption boundary without a new runtime API or a production issuer
adapter; Canic's actual endpoint acceptance remains separate.

Released 0.3.4 extends that fixture through reference-store session
admission, exact retry beyond proof expiry and live scope/generation invalidation.
Rejected network and narrowing requests leave the proof reusable. This joins
the existing public verifier and session engine; it does not qualify Canic's
stable transaction backend, restart recovery or its actual endpoints.

Released 0.3.5 also checks a substituted host refusal after the
library operation succeeds but before reference-store commit. Fresh admission
leaves no records, quota/accounting usage or expiry indexes; identical-token
admission then succeeds. Exact retry propagates storage refusal while preserving
the committed session/replay pair and lease. Canic must reproduce these
obligations at its actual durable transaction boundary, including restart and
retained-state recovery. [Auth #20](https://github.com/dragginzgame/ic-auth/issues/20)
owns this session qualification. It adds no required runtime API.

Protocol-types tests now independently describe Canic's prepare request,
claims/prepare response and retrieval request at immutable Canic
`c4c046f947b2b28f4342cbf6efe9221ba1ed5f70`.
They check both Candid directions and identical encoded bytes for metadata,
nested grants/audiences, fixed-width hashes/nonces, principals, unsigned deadlines
and absent/empty/nonempty optional extensions. Invalid nested roles/identities
reject. Native tests pass on Rust 1.99 and 1.88, and the test target checks for
Wasm on 1.88. [IC Auth #8](https://github.com/dragginzgame/ic-auth/issues/8) owns
this fixture evidence; it does not authenticate tokens or qualify actual Canic
generated Candid, macro endpoints or production adapters.

Released 0.2.0 adds complete retrieved-token and delegation-proof
fixtures, including certificate/leaf audiences and grants, batch headers, both
Merkle sibling directions, seed bindings, key identifiers, nested derivation
paths and opaque root/issuer signatures. Empty and nonempty issuer material is
preserved as transport; synthetic keys/signatures are deliberately unauthenticated.
Malformed roles/identities in the certificate or issuer leaf reject even within
a complete token. All eleven wire tests pass on Rust 1.99 and 1.88; the test
target checks for Wasm on 1.88. These independent fixtures describe the immutable
Canic DTO source, not the concurrently edited consumer adapter.
Released 0.2.0 also cuts the developer PocketIC setup over to Testkit
under Shared Tooling 0.2.0; that tooling change does not require a new Rust
authentication API or delay Canic's runtime verifier adoption.

Canic acceptance requires its selected native/Wasm feature graphs, canonical
signed-byte vectors, both Candid directions, facade/macro callers and real host
certification/upgrade/ingress qualification. Durable host tests must cover failed
commit without partial replay consumption, exact retry after restart, logout and
replacement replay refusal, capacity/expiry boundaries and protected authority
changes. Report removed symbols and actual remaining dependency edges when the
superseded owners are retired.

The initial 2026-10-09 review found no IC Auth dependency in Canic's root/Core
manifests at committed `c4c046f947b2b28f4342cbf6efe9221ba1ed5f70`. During the next
read-only inspection, the consumer working tree began selecting 0.1.14 and
adapting its canonical/proof code. This is uncommitted source evidence, not
consumer qualification or deployment from this source inspection. The owner
subsequently recorded 202 passing focused encoding/configuration/facade cases in
[its adoption handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6079111719).
Those qualify its retained input snapshots, not later graph changes or complete
verifier/host adoption. The next boundary below remains in the same owning issue.
Wallet login is not a prerequisite for these local Rust calls. The private
[browser client](browser-client.md#consumer-adoption) has a separate real-issuer
adapter and publication boundary.

## Historical Canic adapter checkpoints

### Published batch construction extraction

Published IC Auth **0.2.11** provides the pure, default-feature
`chain_key_batch::merkle_root_and_witnesses(&leaf_hashes, max_leaves)` constructor;
see the [batch contract](chain-key-batches.md) and
[Auth #15](https://github.com/dragginzgame/ic-auth/issues/15).
The earlier read-only Canic review was at dirty committed base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6`. Its reviewed batch caller directly
uses the published constructor with its protected 64-issuer budget, preserving
principal-byte sorting and duplicate issuer rejection. The owner records
24 passing batch cases and 182 focused authentication cases, including a
three-issuer batch with real ECDSA signatures and the shared installation
verifier, in [Canic's adoption evidence](https://github.com/dragginzgame/canic/issues/491#issuecomment-6098306441).
The copied generic tree implementation is retired in that uncommitted consumer
slice. This is local producer/installation qualification, not delivered Canic
adoption or a real IC certificate journey.

Canic retains authorized leaf records, batch identity/header policy, management
signing, stable state and renewal. Its Auth 0.3.2 selection at the recorded batch
qualification already supplied this API; the lock at that review selected 0.3.4.
Complete-token verification, certified-signature retention/composition
and atomic durable sessions remain separate consumer adoption slices; no Auth
library release is required to start those adapters.

### Complete verifier adoption

The in-progress Canic `ops/auth/delegated/canonical/mod.rs` calls the library's
encoding/hash functions through checked role, audience, grant, certificate and
claims projections in `ops/auth/delegated/protocol/mod.rs`. Root policy and
delegated registry snapshot framing remain Canic-owned. The protocol DTO
re-exports include metadata, chain-key header/witness/signature atoms and issuer
signature types. Remaining local token/certificate declarations and projections
do not establish canonical type adoption complete. Preserve the owner release
boundary and retained-data obligations while converging these callers.

The earlier cache-removal review of Canic, based on committed 0.111.0
`25b73cd403b20b4e148c6e340f8c760a26d97f6e`, removes the positive proof cache and
its proof-skipping runtime path. Every presentation resolves verifier policy
and checks both proofs through the remaining local callbacks. The owner records
focused qualification in its [pending 0.111.1 evidence](https://github.com/dragginzgame/canic/issues/491#issuecomment-6106918805).
This is uncommitted consumer work, not full upstream-verifier adoption or deployed
acceptance. The inspected verifier and session-workflow SHA-256 values are
`dcc6133dca6d4cbfb14b14360550a932a71d2f13f3e88f141a33071d7b5d99f2`
and `6a4c293a9807c5e6eb3cfa0a24d319bf132b7bf4299a5ee90f71a144619f25fb`.
No Canic compilation or source mutation is performed by this review.

The subsequent read-only review at that same committed base resolves both Auth
packages to published 0.3.8. Active-proof installation and token presentation now
share `ops/auth/delegated/root_proof/mod.rs`, which projects protected context
into `token::verify_delegation_proof`. The duplicate Root-proof acceptance engine
is retired in the consumer's dirty tree; complete token semantics, issuer proof
verification, certification and session/storage adoption remain separate.
The owner records 103 passing focused Auth cases and warning-denied feature-on,
feature-off and adapter Clippy in its
[Root-proof evidence](https://github.com/dragginzgame/canic/issues/491#issuecomment-6107336083).
This is owner-reported local qualification, not a new consumer build or deployed
acceptance by this review. The reviewed root adapter hashes to
`2f61147a74503b5e4d9276bdbdd1f2e9d83a23b384af22816f3adc37075d456b`;
the token verifier hashes to
`a3f431ec64ba05f8a7f5ddcce923a26c858cf393134b6ba47a2da9e1365bf2a0`.

The remaining diagnostic adapter rechecks submitted key/path metadata only after
library rejection; it cannot accept a proof. [Auth #23](https://github.com/dragginzgame/ic-auth/issues/23)
owns distinguishing submitted signature/header mismatches from header/enrolled
policy mismatches. The published 0.4.0 producer cut implements
`RootSignatureBindingMismatch` and `RootKeyPolicyBindingMismatch` for algorithm,
key ID and derivation-path hash in both public verifier entry points. For each
field, inconsistent submitted signature/header material takes precedence over
an enrollment mismatch. Other binding causes and cryptographic checks retain
their existing contracts. See the [diagnostic migration](delegation.md#root-binding-diagnostics).

When adopting published 0.4, Canic must update its root dependency selections and
all error matches, project the two typed origins into its existing public codes,
and retire the post-rejection key/path comparisons and hashing in
`map_root_delegation_proof_error`. Qualify signature-only, policy-only and combined
mismatches through both installation and presentation. The producer implementation
is published; 0.3 requirements do not select this minor release automatically.

A subsequent read-only review at the same committed Canic base finds its dirty
root catalog selecting `0.4` and both locked packages resolving to 0.4.0 with
the matching unyanked producer checksums. The Root adapter now maps
`RootSignatureBindingMismatch` to invalid input and `RootKeyPolicyBindingMismatch`
to stale material. Token presentation delegates diagnostic projection to that
same adapter; its duplicate submitted key/path comparisons and hashing are gone.
The reviewed lock SHA-256 is
`7526b96f01ea66916f2880ddb15354a3321f902ab13d18bba0e20931ba27983a`,
Root adapter `a16983e06e7561b2f894c5f4a63602c47ee01603204a9f2f800cd38297121df8`
and token verifier `7f1e1e4c68c03ea67664d5a93ed09554344248e018d405d1524bac073d4ed819`.
This establishes source migration in the consumer's working tree, without
claiming consumer compilation, endpoint acceptance or deployment by this review.

The current runtime verifier delegates to `token::verify_token`; the following
inputs remain its host contract after that cut:

| Current Canic caller | Library boundary | Canic obligation |
| --- | --- | --- |
| `ops/auth/token/verification.rs::verify_with_embedded_proofs` | `token::verify_token` | Project the complete token once; supply independently enrolled key policy, actual caller, protected fleet/role and endpoint requirements |
| `ops/auth/token/verifier_config.rs` | `RootKeyPolicy` and `TokenVerificationLimits` | Keep build-network/key-name admission and all numeric limits in protected configuration |
| Current canister's `auth.allowed_token_scopes` | Protected `allowed_scopes` | Keep the configured token ceiling independent of submitted grants and endpoint requirements; review session policy separately |

Preserve endpoint error/metric mapping with typed library errors. Qualify changed
key windows, epochs and local ceilings using the same token on successive calls,
as well as caller/audience/seed/signature rejection. Complete verifier adoption
does not require a new library endpoint or wallet service. Stable session storage
and certification preparation retain their separately described host contracts.

The earlier 2026-10-10 read-only review of Canic at base
`ac55e50334dd6479ec36f404e89e60bcfe9184d6` finds both libraries resolved to 0.2.7
in its dirty root lock. Its installation adapter calls the standalone library
verifier, while the complete-token adapter still uses local callbacks and the
positive cache. This identifies the next consumer boundary, not qualification
of its uncommitted graph. [Canic #58](https://github.com/dragginzgame/canic/issues/58)
owns that repair; the later owner cache removal above supersedes that specific
source finding. The newer shared Root adapter above supersedes local Root-proof
acceptance; complete-token and issuer callbacks still required their own cut at
that historical checkpoint. The current shared verifier above supersedes it.

The complete adapter must assemble these fields before calling the library:

| Library input | Protected Canic source or decision |
| --- | --- |
| `caller`, `now_ns` | Actual authenticated ingress caller and host clock at this invocation |
| `audience`, `role` | Current fleet binding and runtime role, through checked protocol projections |
| `root_key` | Existing independently enrolled chain-key policy, retaining network/key-name admission and all live floors/windows |
| `ic_root_public_key_raw` | Network-selected BLS trust anchor, after existing build-network validation |
| `allowed_scopes` | Sorted, unique protected ceiling for this local role; the existing local application configuration supplies it where enabled |
| `required_scopes` | The operation's endpoint requirements, independently of the ceiling |
| `limits` | Existing host TTL/skew policy plus explicit certificate age, material, signature and witness bounds |

Token-only verification without local session configuration needs an explicit
protected ceiling owned by Canic; neither submitted grants nor endpoint-required
scopes define it. The library rejects any scope in the selected local grant
outside this ceiling, even when the operation only requires a smaller subset.
It does not silently intersect or normalize the grant. The existing issuer
proof callback has no certificate-age or signature-size inputs, so those finite
bounds must be selected deliberately rather than inferred from received proof
bytes. Consumer limits are not new IC Auth defaults.

Project the complete token once and pass it to `token::verify_token`. After
success, use `VerifiedToken` accessors for authenticated claims, selected role,
local scopes, claims hash and the policy-capped exclusive expiry. Resolve
Canic-owned resource permissions and session/replay admission separately.
Keep typed error/metric mapping at the adapter and qualify the real entry point
with identical signed token bytes while changing the protected ceiling, root
window/epoch, IC trust anchor and certificate age. Retiring the cache together
with local proof callbacks leaves one verification owner; keeping a cap on the
old cache alone does not enforce these live inputs.

The pre-token installation caller has a separate boundary:
`ops/auth/delegation/active.rs::install_active_delegation_proof` checks an issuer's
`DelegationProof` before a completed token exists. Released 0.2.2 adds
`token::verify_delegation_proof` with `DelegationProofVerificationContext` and
`DelegationProofVerificationLimits`; see the
[standalone contract](delegation.md#standalone-root-delegation-verification).
Supply the actual issuer from protected runtime/configuration, enrolled root
policy, host clock and finite bounds. The result is proof evidence, not issuance
approval. Its certificate hash includes checked issue metadata that is absent
from the signed root leaf; complete token verification still binds that exact
hash through signed issuer claims.

[IC Auth #10](https://github.com/dragginzgame/ic-auth/issues/10) records completed
library qualification; consumer acceptance stays under Canic #491. Keep Canic's installation authority,
protected key/network/audience/grant policy, renewal, storage and live authority
checks. Preserve its strict installation not-before rule even if configured
cryptographic future skew is nonzero. Verify before storage mutation and use
the policy-capped deadline when recording proof freshness. Retire its duplicate
pre-token root verifier after consumer acceptance. This boundary does not block
adoption of the existing complete token verifier. No Canic source changes or host acceptance are implied.

## Standalone shape APIs (released 0.5.11)

[Auth #50](https://github.com/dragginzgame/ic-auth/issues/50) exposes the
[scope and grant checks](certificate-issuance.md#standalone-scope-and-grant-shape-checks-pending-0511)
needed before Canic certificate construction. Adopt the detailed const label
validator for both static and runtime scope constructors, map its byte-offset
causes, and retain Canic's set cardinality, aggregate byte and namespace policy.
The public canonical scope-list validator can replace the ordering loop inside
Canic's registry/key-policy encoding without changing the envelope or encoded
bytes. Standalone grant validation takes existing passive grants and explicit
16/32 bounds; it needs no dummy certificate or token.

Consumer work belongs in [Canic #491](https://github.com/dragginzgame/canic/issues/491).
Retire the duplicated loops only after static constructors, raw-role projection,
mixed-invalid refusal precedence and construction/configuration callers are
qualified together. Configuration sorting and intentionally empty disabled policy
remain Canic-owned. No consumer edits, builds or adoption are claimed here.
