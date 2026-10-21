# Canic library adoption contract

Auth owns reusable authentication semantics; Canic owns the product host that
uses them. [Canic #491 — Extract reusable authentication, token and session machinery](https://github.com/dragginzgame/canic/issues/491)
is the implementation/acceptance tracker. This document is the current interface
boundary, not evidence that Canic or Toko have deployed it. Earlier extraction
checkpoints are retained separately in [adoption history](canic-adoption-history.md).

## Released API and dependency direction

The released Auth 0.10.7 stack contains `ic-auth-protocol-types`, `ic-auth` and
`ic-auth-wire`. Select matching package releases in the consumer's root catalog;
no sibling source/path override or copied codec is needed. Auth has no Canic or
Toko dependency. Rust 1.91 is the common minimum.

- `ic-auth-protocol-types` owns checked identifiers, portable claims, certificate/
  proof contracts and typed logout outcomes. It exposes `ic_principal::Principal`,
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

The 0.10.7 release routes the protocol's Principal import through feature-free
IC Platform 0.2.1. Its upstream type identity and serde/text/byte representations
are unchanged; consumers can keep importing Auth's `Principal`. No Candid or
runtime capability is selected by that edge.

Forward only required capabilities. `portable-signature-verification` selects
Ed25519, `chain-key-verification` supplies standalone Root verification, and
`certified-proof-verification` supplies complete certified portable verification.
`sessions` enables the engine without a verifier; `session-restore` adds checked
restoration. Signature-store preparation and canister-signature verification
remain separate optional primitives. Native Testkit/Host/Build, CDK and fixture
contracts stay out of the public Auth graphs. All public library capabilities, including certified verification, now reject
Candid and raw IC calls. Native qualification still brings Candid through Agent/Testkit.

<a id="pending-09-wire-transport"></a>

## Wire transport

Use `CredentialEnvelope` for credential data, `EstablishSession` for admission,
`ReadSession` for a proof-free local read, and `Logout` for caller-authenticated
local revocation. Their `Method` types own identity, bounded request/reply types
and limits. Canic binds them to actual exports, admits raw argument size before
copying, decodes the exact method and authenticates ingress/replies. Reply-encoding
failure after mutation must trap so that local state rolls back.

The published 0.10.7 transport exposes IC Wire 0.4. Consumers sharing its Rust
Wire types must align that dependency and select matching Auth package releases.
The older 0.9.1 transport retains IC Wire 0.3; these are distinct published
dependency contracts. Generated files must come from the selected owner, never
downstream patches.

Applications may bind their own command envelope through an explicit qualified
client transport adapter. Preserve the shared domain/signed contract and bounded
translation; do not reconstruct an independent Auth codec or restore a retired
application-token flow. The bounded Wire controls under `apps/qualification/contracts/`
serve private controller/certificate qualification only and are not production
schemas. Linking any Auth crate installs no endpoint or authority.

## Wire 0.5 migration

Pending Auth 0.11 selects published Wire 0.5 and native Testkit 0.57. Package
versions remain 0.10.7 until the maintainer releases the minor. Consumers that
share `ic-auth-wire::wire` Rust types must select matching Auth 0.11 packages and
Wire 0.5 after publication. Consumers using only domain/session APIs need no new
codec or DTO translation merely for this dependency change.

Wire no longer implies owned deserialization. Add an explicit
`serde::de::DeserializeOwned` bound to `M::Request` in owned generic request
decoders, or to `M::Response` for owned reply decoding. Encoding-only functions need no decode
bound. Auth keeps concrete owned DTOs; this migration introduces no borrowed
public Auth lifetime or alternate browser codec. Frozen 0.10.7 frames and generated
TypeScript remain unchanged. No login/journal/checkpoint reset is authorized.


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
Testkit 0.62 paired with Platform 0.5; current Testkit 0.63 aligns to Platform
0.6. Older Testkit releases retain their own Platform identity.

Signed claims, frozen Wire frames, session/replay/login encodings and generated
TypeScript retain their bytes. This is a Rust dependency-identity cut, with no
journal/checkpoint reset or storage migration. The private host removes the
retired no-op `hidden` macro option while retaining explicit Wire hooks and its
existing endpoints. It uses none of the removed checkpoint/reader APIs or
`Call::change_timeout`; there is no replacement storage abstraction or wait-policy
change in Auth. The host still requires `cdk` for entrypoint generation; public
proof/type/session capabilities select no runtime feature.

## Platform 0.6 migration

Pending Auth 0.13 adopts published Platform/macros 0.6.0. Consumers sharing Auth's
owned Principal, parse errors or imported canister-signature key type must align
their direct Platform dependency to 0.6. Type-only consumers must select
`features = ["principal"]` with defaults disabled. Auth's protocol-types package
selects this explicitly; `runtime`, `cdk`, native clients and signature creation
also select it upstream. No runtime feature is needed for passive protocol types.

The independently published Testkit 0.63 now selects Platform 0.6 too.
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

Canic owns fleet/member/role policy, issuer approval/renewal, protected Root/key
selection, enrollment freshness, real caller/time acquisition and scoped endpoint
guards. Its store implements Auth's transaction contract, including capacity,
failed-commit atomicity, replay and retained logout denials. Memory mode uses the
signed reset floors and starts closed; persistent mode restores complete checked
state and still requires fresh authority. See [sessions](sessions.md) and
[certification composition](signature-preparation.md). The fixture's IC Memory
checkpoint does not supply a production per-operation backend.

The old Rust delegated-token issuance/verifier API and browser prepare/retrieve
client were retired in 0.9 after explicit no-deployed-user/no-unresolved-effect
confirmation. Use `ic_auth::delegation` for Root policy, proof verification and
errors, and `portable_certified::CertifiedProofLimits` for certified admission.
`token-verification` is retired. Generic signature-store prepare/retrieve,
certificate issuance rules and role-attestation primitives remain supported;
their similar names do not make them another application login flow.

## Browser and production acceptance

Use the portable admission coordinator and durable logout journal, or a consumer
adapter qualified against those contracts. Issuance intent precedes dispatch;
`reconcile` recovers the original work or refuses unresolved. Missing, pruned or
expired evidence cannot issue new credentials or extend the original deadline.
Logout durably invalidates the browser generation before fan-out, and only
authenticated acknowledgements mark receiver revocation. Preserve unknown effects.
See [browser contracts](browser-client.md),
[Auth #3 — Qualify portable browser admission/logout with a production issuer adapter](https://github.com/dragginzgame/ic-auth/issues/3)
and [Canic #507 — Qualify non-issuing token prepare reconciliation after replay receipt expiry](https://github.com/dragginzgame/canic/issues/507).

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
| Lost issuance reply and journal reopen | Recover only the original credential bytes and original admission deadline; issuance/signing counters stay at one. |
| Missing/pruned issuer evidence | Keep issuance unknown; no replacement signing, allocation or receiver submission. |
| Receiver admission and scoped operation | Bind actual caller/clock and protected authority to Auth's atomic session/replay/login store; successful scoped access must reach the real facade. |
| Logout, authority change and reset/restore | Retained keys cannot revive revoked sessions; peer/device state and original lease deadlines retain their defined independence. |

Auth's real SDK fixture now injects a lost issuer reply, reopens its journal,
retains unresolved lookup and recovers the original material before authenticating
to two actual local receivers. It proves coordinator/receiver composition, not
Canic's issuer lookup, production stable backend or native-browser persistence.
Keep acceptance with [Canic #491 — Auth integration](https://github.com/dragginzgame/canic/issues/491)
and [Canic #507 — Non-issuing reconciliation](https://github.com/dragginzgame/canic/issues/507).
