# Canic adoption contract

Auth owns reusable authentication machinery. Canic owns its runtime composition,
issuer approval/renewal, fleet/member/role policy, endpoint guards and deployment.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) tracks production
integration; this guide supplies the contract, not evidence of deployment.
Older migrations and observations are preserved in [adoption history](canic-adoption-history.md).

## Current integration cohort

The current released Auth family is **0.29.1**, source
[`54d1d2a`](https://github.com/dragginzgame/ic-auth/tree/54d1d2aadfc67830f630612d5654f28815a64119).
Select matching `ic-auth`, `ic-auth-protocol-types` and `ic-auth-wire` packages.
[`Cargo.toml`](../Cargo.toml) and the [current handoff](status/current.md#selected-suppliers-and-governance)
own this checkout's exact upstream selections. Root package versions remain at
0.29.1 until an authorized release; pending changes are described in `CHANGELOG.md`.

`Session::scopes()` returns checked `ScopeLabel` values; use `as_str()` for text.
Requests move their checked labels into sessions. Signed protocol grants remain
raw DTOs checked by `ValidatedGrants`; never normalize or rewrite signed bytes.
The 0.29 API cut removed the unused top-level session budget variants and
conversion. Fresh proof budget failures retain their mechanism-specific credential
error; use `PortableCredentialError::budget_error()` for diagnostic classification.
Public refusal categories, verification ordering and charging are unchanged.

## Receiver feature selection

Select capabilities on the receiver's normal dependency edge with defaults disabled:

| Capability | `ic-auth` features |
| --- | --- |
| Volatile Ed25519 sessions | `sessions`, `portable-signature-verification` |
| Volatile certified sessions | `sessions`, `certified-proof-verification` |
| Both enrolled proof mechanisms | `sessions`, `portable-signature-verification`, `certified-proof-verification` |
| Persistent record restoration | Replace `sessions` with `session-restore`; retain the selected verifier features |

Protected enrollment selects the mechanism, issuer and key. Submitted proof
variants cannot select trusted authority or enable a capability. Qualify the
normal native/Wasm graph without dev dependencies; test feature unification can
hide a missing production capability. Auth libraries do not depend on Canic,
Toko, runtime, stable memory or Metrics.

## Session commit contract

The host obtains actual caller/time, current protected authority and generation,
then serializes each complete engine operation without await or reentrancy:

| Engine operation | Host inputs and obligations |
| --- | --- |
| `establish_session` | Exact credential, checked request, protected verifier/role/TTL, current policy, retained store/login ledger and receiver-wide verification budget |
| `authorize_session` | Actual caller, current authority/time/generation and checked operation scopes; callback runs only after all session checks pass |
| `logout_session` | Authenticated browser caller, trusted time and a separate unknown-logout budget; fresh enrollment or a new proof is not required |

Admission owns complete proof verification, replay consumption and reservation.
Ordinary operations authorize the existing caller-bound session instead of
re-verifying a proof. An admission receipt is not permission for later operations.
Application resource ownership remains separate; account identity is the pair
`(issuer, subject)`, not a subject alone. Recheck protected authority before effects
following awaits. Advance generation atomically with authority changes.

A persistent backend implements `SessionReader` and `SessionStore`. Reads expose
an isolated immutable snapshot. Writes validate before effects and atomically
persist the session/replay/login change, indexes and protected totals. Returned
errors preserve prior records; proof-verification charges and trusted clock
observations survive ordinary refusals. Do not trap to report those errors.

Reuse the reference commit callbacks or the shared checks:

- `Session::check_admission_pair` checks the reservation, pair association and
  current generation. Replay uniqueness and authentic storage provenance remain
  host obligations.
- `SessionLimits::check_capacity` checks physical record counts, including expired
  rows until pruning. Hosts independently enforce encoded-byte/backing/index
  budgets using their own record representation.
- `Session::check_login_change` checks association for logout; require a revoked
  change, remove only its caller's session and retain replay evidence. A missing
  session still requires committing the tombstone.
- `LoginRevocationChange::check_replacement` preserves a live binding/pool, denial
  and retention deadline at trusted time. The host retains clock/pool accounting
  and atomic persistence. A successful preflight is not proof verification.

Keep one ledger and the two independent budgets across calls. Restore complete
session/replay/login records with generation, clock high-water mark, pool counts,
reset floors and authority metadata from one protected generation. Obtain fresh
receiver-bound enrollment before readiness; restoration alone never opens access.
Volatile reset requires qualified reset floors and fresh enrollment. Bounded
cleanup reclaims expired rows without evicting live replay/denial state.
See [storage and transaction details](sessions.md#custom-host-transaction-contract).

## Wire transport

Use canonical method identities, limits and codecs from
[`ic-auth-wire`](../crates/ic-auth-wire/README.md). Hosts own export names and
runtime caller acquisition. Generated client contracts use the same Rust policy
and selected Wire runtime; application traffic has no Candid fallback.

Project original `SessionError` values with `admission_refusal()` or
`read_refusal()`, then convert to the Wire model. Keep the original diagnostic
locally. `NotYetValid` is a bounded retry hint, not proof validity; retain strict
start/deadline admission and the original credential. Do not retry invalid
signatures, expiry, reset-floor or stale-authority refusals automatically.

## Platform 0.16 adoption

Released Auth 0.29.1 adopts Platform 0.16; Auth 0.29.0 selected the earlier
Platform 0.15 cohort. Platform now maintains certificate/tree/BLS
identities and uses canonical owned Principal in canister-signature keys.
Keep external conversions at native Agent/Testkit boundaries. K256 0.14 and
Dalek 3 change concrete crypto identities; Auth preserves its byte-based API,
strict verification and explicit high-S policy. The upstream BLS identity-key
fix rejects forged certificates; native Agent's separate verifier is unchanged.
Canister build owners should select the qualified 32-bit RustCrypto backend;
this checkout declares it in [Cargo configuration](../.cargo/config.toml).
Current Wire and stored formats require no migration or reset.

## Platform 0.15 migration

Platform reexports Principal/parse errors from canonical Types 0.15; Wire 0.13
owns their schema implementations through `platform-types`. Align these public
identities when exchanging values. Passive packages select Principal only;
self-authentication and external conversions are explicit. Use
`principal::ExternalPrincipal` at upstream Agent/Testkit/crypto boundaries and
preserve original bytes. The reference host selects one Platform runtime,
generator and guard; public library graphs remain Candid/runtime-free.

## Browser and production acceptance

The private client supplies the admission coordinator, guarded identity and durable
logout journal. Use a [qualified archive from the intended release](browser-client.md#local-archive-delivery),
with source revision, clean/dirty status and integrity receipt. Keep distribution
at a durable consumer-owned location; an ignored producer directory is evidence,
not deployment. No npm publication or production issuer adapter is implied.

The consumer supplies production `issue`, lookup-only `reconcile`, admission and
logout transports. Authenticate the account separately from the browser signer.
Persist exact issuance intent before effects; after awaits recheck authority and
binding generation before committing. Reconciliation returns original material or
unresolved, never replacement issuance or extended deadlines. Local durable
logout invalidation precedes fan-out; only authenticated committed acknowledgements
resolve receiver logout. Unknown effects and journal records must survive reopen.

The [backend conformance package](../apps/qualification/backend-conformance/README.md)
can be packed as a self-contained native source archive using registry Auth types.
Run its scenarios against the actual host adapter, then qualify installed endpoint
behavior: both mechanisms, two receivers/devices, allowed scoped updates, issuer
namespacing, failure/replay/logout, fresh enrollment after upgrade, bounded cleanup
and before/exact/after expiry. Measure the actual host path at representative
occupancy; fixture costs do not establish production performance or crash atomicity.

Current `AUTHNAT3` is the private qualification checkpoint. Current Auth session,
login and replay discriminators remain unchanged; no new reset or migration is
introduced. Never erase Memory backing or only its allocation ledger to obtain
admission. Cryptographic checks and the TEE/non-TEE trust model are unchanged.

<a id="platform-07-migration"></a>

Historical Platform 0.7 migration notes referenced by older release notes are
preserved in [adoption history](canic-adoption-history.md#platform-07-migration).
