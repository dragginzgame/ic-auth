# Canic library adoption contract

Auth owns reusable authentication semantics; Canic owns the product host that
uses them. [Canic #491 — Extract reusable authentication, token and session machinery](https://github.com/dragginzgame/canic/issues/491)
is the implementation/acceptance tracker. This document is the current interface
boundary, not evidence that Canic or Toko have deployed it. Earlier extraction
checkpoints are retained separately in [adoption history](canic-adoption-history.md).

## Released API and dependency direction

The released Auth 0.9.1 stack contains `ic-auth-protocol-types`, `ic-auth` and
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

The pending release routes the protocol's Principal import through feature-free
IC Platform 0.1.9. Its upstream type identity and serde/text/byte representations
are unchanged; consumers can keep importing Auth's `Principal`. No Candid or
runtime capability is selected by that edge.

Forward only required capabilities. `portable-signature-verification` selects
Ed25519, `chain-key-verification` supplies standalone Root verification, and
`certified-proof-verification` supplies complete certified portable verification.
`sessions` enables the engine without a verifier; `session-restore` adds checked
restoration. Signature-store preparation and canister-signature verification
remain separate optional primitives. Native Testkit/Host/Build, CDK and fixture
contracts stay out of the public Auth graphs. The optional upstream certified
signature stack still uses Candid internally; portable/session-only graphs do not.

<a id="pending-09-wire-transport"></a>

## Wire transport

Use `CredentialEnvelope` for credential data, `EstablishSession` for admission,
`ReadSession` for a proof-free local read, and `Logout` for caller-authenticated
local revocation. Their `Method` types own identity, bounded request/reply types
and limits. Canic binds them to actual exports, admits raw argument size before
copying, decodes the exact method and authenticates ingress/replies. Reply-encoding
failure after mutation must trap so that local state rolls back.

The published 0.9.1 transport exposes IC Wire 0.3. The current Auth working tree
contains a maintainer-selected Wire 0.4 update. That exposed Rust dependency
identity requires the next Auth minor release and coordinated consumer selection;
it is not silently part of the already released 0.9 contract. Generated files
must come from the selected owner, never downstream patches.

Applications may bind their own command envelope through an explicit qualified
client transport adapter. Preserve the shared domain/signed contract and bounded
translation; do not reconstruct an independent Auth codec or restore a retired
application-token flow. The bounded Wire controls under `apps/qualification/contracts/`
serve private controller/certificate qualification only and are not production
schemas. Linking any Auth crate installs no endpoint or authority.

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
