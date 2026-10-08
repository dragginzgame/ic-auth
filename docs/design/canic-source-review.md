# Canic incorporation review — 2026-10-08

Source: clean Canic checkout at
`e286b3fd98460c98670336853f80658a920966e0`, inspected read-only. This supersedes the
older source reference in the accepted design for this implementation batch.
The imported source retains the Canic MIT notice in the root [license](../../LICENSE).
The [accepted design](extraction.md) remains authoritative;
[Canic #491](https://github.com/dragginzgame/canic/issues/491) tracks the larger
extraction and adoption. This is a source/contract record, not another task queue.

## Incorporated in this workspace

| Canic source under `crates/canic-core/src/` | IC Auth result |
| --- | --- |
| `dto/auth/common.rs`, `token.rs` | Passive request, grant, audience and application-token contracts in `ic-auth-protocol-types` |
| Protocol portions of `dto/auth/proof.rs` | Certificates, chain-key batch proofs/witnesses and issuer signature contracts in `ic-auth-protocol-types` |
| Canonical identity serialization in `ids/fleet/mod.rs`, `ids/network.rs` | Fixed-width `CanonicalId` and exact network-qualified `AudienceId`, preserving text serialization |
| Role grammar in `ops/auth/delegated/canonical.rs` | `AuthRole`, validated on construction and deserialization without deployment role constants |
| Token/proof portions of `ops/auth/delegated/canonical.rs` | Canonical bytes and hashes in `ic-auth::canonical`, preserving existing domains and tags |
| Scope grammar from `model/auth/application_authorization/scope.rs` | Existing lowercase segmented grammar and 64-byte bound enforced during canonical encoding |

No custom signature primitive was added. SHA-256 remains supplied by `sha2`.
Length prefixes reject overflow with a typed error rather than panicking. Hash
functions that previously returned an infallible array now return a `Result`
where an encoded vector can overflow. This is an explicit Rust API adaptation;
accepted canonical bytes remain unchanged. The three existing golden vectors for
batch header, issuer leaf and complete root proof pass unchanged.

`DelegationAudience::Fleet` and the `canonical_network_id` / `fleet_id` field names
are retained wire labels. They do not import fleet topology or authority into
IC Auth. `AudienceId` holds the exact two 32-byte protocol identities, not a
free-form label. A Canic adapter must project its protected network and fleet IDs
byte-for-byte and parse roles with `AuthRole`; it must never construct its expected
authority from the token being checked. Wire decoding validates syntax only.
The existing role grammar has no new length bound; hosts must retain their own
input-size limits. Passive proof vectors are not a bounded authenticated decoder.

## Source that needs a host boundary before reuse

| Source family | Reusable mechanism | Authority/effect retained by Canic or service host |
| --- | --- | --- |
| `ops/auth/canister_sig_key.rs`, `domain/auth.rs` | IC key parsing and signature support, preferably upstream-owned | Network trust enrollment and derivation policy |
| Root/issuer canister signatures | Prepare, retrieve and verify mechanisms | Attestation meaning, eligibility and one composed certification root |
| `delegated/verify.rs`, `audience.rs`, `cert_rules.rs`, `delegation_cert.rs`, `prepare.rs`, `chain_key*` | Proof binding, validity, narrowing, issuance and crypto | Exact protected fleet/role/network/issuer policy and approved signing effects |
| `delegated/cache.rs` | Bounded crypto-result cache semantics | Live policy/epoch checks and host-owned state; do not copy thread-local authority |
| `model/auth/application_authorization`, session workflow/storage | Scope/session/replay transitions and atomic keyed storage contract | Caller, clock, memory allocation, lifecycle and authority-generation transitions |
| Auth access/API and runtime renewal/provisioning | Thin calls into the future library | Endpoint guards, fleet authority, installation and renewal orchestration |

Controller key requests, registry/root-key policy snapshots, active-proof
installation/status envelopes and their canonical policy hashes remain in Canic.
They need an explicit checked projection when verification is extracted; they
were not pulled into the passive package to satisfy imports.

## Consumer surface inspected

The read-only caller search covered Core, facade/macros, Control Plane, Host and
CLI sources, including colocated tests. Core DTO/model/ops/policy, API/access,
workflow/runtime and stable-storage references make this more than an import
rename. Outside Core, the inspected surfaces include:

- Facade `src/prelude/mod.rs`, `src/api/mod.rs`, `src/macros/build.rs` and
  `src/macros/endpoints/{role,root,fleet_coordinator}.rs`.
- Macro `src/endpoint/validate/{mod,tests}.rs` and `src/endpoint/expand/tests.rs`.
- Host `src/frontend/workflow`, `src/observatory/workflow`, `src/release_set/config`,
  `src/release_set/tests/roles/capabilities.rs` and `src/canic_metadata`.
- CLI `src/auth/codec.rs` and Control Plane `src/api/lifecycle.rs`.

This is a textual dependency inventory, not proof of complete generated-Candid,
macro-expansion, feature-selected or downstream coverage. Canic adoption must
trace those generated contracts and producer/consumer fixtures coherently with
[Canic #354](https://github.com/dragginzgame/canic/issues/354). No Canic source was
removed, aliased or redirected. Until that adoption, Canic retains its active
implementation. The contracts/encoding are published independently at `0.1.2`;
the later signature addition below remains uncommitted and unpublished.
A1 is not claimed complete, and the larger extraction is still partial.

The qualification findings named by #491 remain unresolved by the encoding move. In
particular, hashing an issuer binding does not prove exact seed verification,
and hashing policy material does not enforce cache-policy expiry. No replay,
atomicity, revocation, wallet login or stable ingress identity claim is made.

## Subsequent signature verification boundary

The signature review used committed Canic
`4c51a87c6a32397196bb3f65d064641194df10a5`, reading
`ops/auth/{canister_sig_key,issuer_canister_sig,root_canister_sig}.rs` from Git,
not its dirty tooling/dependency changes. Existing issuer/root domain bytes and
explicit signer/seed checks inform the new reusable verifier. Canic's fixed seed,
fleet trust policy, metrics, thread-local signature maps and direct
`certified_data_set` calls were not copied.

The [signature API](../signatures.md) uses the upstream public-key type and IC
signature verifier, adds a bounded DER admission check and exact re-encoding,
and receives protected signer, seed hash, root key, clock and limits explicitly.
There is no custom DER field parser or cryptographic primitive. Certificate
freshness is checked after upstream cryptographic verification; Canic's reviewed
wrapper did not provide that clock boundary. Signed-message construction preserves
both inspected Canic domains exactly and checks the one-byte length conversion.

Real fixture signatures verify under deterministic BLS trust keys; changed
signer/seed/network/time policy rejects the same proof. This is verification
mechanism evidence, not live IC, complete token/session verification, service
certification composition or consumer adoption. Preparation/retrieval and their
bounded retention, host root composition and lifecycle remain unimplemented;
no A2 completion claim is made. Canic still owns its active source until a
separately authorized adoption removes it coherently.
