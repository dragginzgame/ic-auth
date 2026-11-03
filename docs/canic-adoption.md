# Canic adoption contract

Auth owns reusable authentication machinery. Canic owns its runtime composition,
issuer approval/renewal, fleet/member/role policy, endpoint guards and deployment.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) tracks production
integration; this guide supplies the contract, not evidence of deployment.
Older migrations and observations are preserved in [adoption history](canic-adoption-history.md).

## Current integration cohort

Published **Protocol 0.32.0** (`ic-auth-protocol-types`, `ic-auth-wire`) and
**Engine 0.32.0** (`ic-auth`) are the current adoption cohort. Protocol releases
independently from Engine. Keep the two Protocol packages aligned; use the
Engine package's declared Protocol requirement rather than forcing equal versions
across groups. An Engine-only update leaves Protocol's Rust type identity intact.
The published cohort uses canonical Types 0.17, Platform 0.18 and Wire 0.15.
The qualification host selects independent Runtime 0.18 and the native runner
uses `ic-platform-pocket-ic` 0.18 with Testkit 0.80. Rust 1.99.1 is the minimum.
[`Cargo.toml`](../Cargo.toml), the lockfile and the [current handoff](status/current.md)
distinguish this checkout's selections from delivered release evidence;
[group ledgers](../CHANGELOG.md) own pending changes. This checkout prepares
Protocol/Engine 0.33 with Platform/PocketIC 0.20, Runtime 0.19, Types 0.18,
Wire 0.16, Testkit 0.83, Metrics 0.28.2 and Storage 0.66.1. These resolve the former Principal/Wire and duplicate
Runtime blockers. Align Rust consumers when that cohort is released. Auth carries
no Account DTOs, so the upstream Account byte-adapter cut does not change Auth's
frames, method identities or stored records. Regenerated browser files are unchanged.

## Current Canic source review — 2026-11-03

Read-only review used Canic HEAD `0509ec870891515923f79d3932ece127b351cd15`
plus its existing uncommitted integration. No Canic build, edit or installed
acceptance is supplied by this review. Its authentication guide now records
controlled-local persistent admission/read/logout, Root refresh and scoped
guards, plus heap-only issuer key preparation/registration. The production clock
bound, snapshot restore fencing, credential issuance/reconciliation and browser
delivery remain Canic-owned acceptance under #491. The facade now calls the
portable authorizer when enabled; older reports of an unconditional denial are
historical. Production activation remains explicitly gated.

The working persistent adapter already calls `Session::check_admission_pair`
and `SessionLimits::check_capacity`, the two changes requested by
[Canic #720](https://github.com/dragginzgame/canic/issues/720). These changes are
uncommitted; the owner still needs their focused acceptance and closure review.
No new Auth API is indicated by this inspection. Adopt the matching backend
conformance bundle and private browser archive with source/integrity receipts
when selecting the published Auth cohort; production transports and durable
adapter qualification stay in Canic.

The registry currently offers Host Tools 0.80 (Platform 0.20/Types 0.18), which
can address the newer native Host identity family. Query 0.89 also uses the
matching Types 0.18 through Platform 0.19. Canic's complete latest family still
needs a Timers-compatible Runtime transition: published Timers 0.38.1 selects
Runtime 0.18.1, while Storage 0.66 and Auth's qualification host select Runtime
0.19 with a different singleton guard. Do not retain both executors/guards or
add identity aliases to force resolution. Metrics/Storage 0.67 matches Metrics
0.28 and Storage 0.66. Supplier alignment and publication of Auth's pending
0.33 cohort are separate from implementing a new Auth compatibility layer.

`Session::scopes()` returns checked `ScopeLabel` values; use `as_str()` for text.
Requests move their checked labels into sessions. `DelegatedRoleGrant::scopes`
also contains checked `ScopeLabel` values. Construct labels with `parse()` or
`ScopeLabel::try_from(String)`; use `ValidatedGrants` for grant cardinality and
strict ordering. `AuthRole` enforces 1–64 bytes during construction and decoding.
These shape checks grant no authority; never normalize or rewrite signed bytes.
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

## Auth 0.30 / Platform 0.17 / Wire 0.14 adoption

This historical cohort precedes the current 0.32 selection above. The published
Protocol and Engine 0.30.0 releases select canonical Types 0.16.0,
Platform 0.17.0, Wire/derive 0.14.2 and native Testkit 0.78.0. Existing Principal,
parse-error, codec and limits values have a new Rust package identity; align the
consumer's declarations with this cohort. This is a public Rust cut, with unchanged
signed bytes, transport/stored formats, verification checks and TEE/non-TEE policy.
Principal self-authentication no longer implies registry conversions: select
`principal-external` at native boundaries. Runtime-only consumers select
`ic-platform-runtime` directly; the qualification host uses Runtime/macros/guard
0.17.1, independently of the crypto facade. Keep one executor/generator/guard.
Native PocketIC clients and client DTOs come from `ic_platform::pocket_ic` 0.17;
use Testkit 0.78 to share Wire 0.14 limits. The server remains PocketIC 16.1.0.
All three Auth crates are published at 0.30.0. The backend source archive is
qualified against their registry identities; see the [current evidence](status/current.md).
This does not establish installed Canic acceptance.

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
identities when exchanging values. Passive packages select Types directly;
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

For a matched handoff, run `make test-backend-package` and
`make test-client-package` from the same fixed Auth source after preparing their
declared dependencies. These existing commands create and qualify the actual
backend `.crate` and browser `.tgz`; separate pack-only runs are unnecessary.
Retain both `delivery.json` receipts, source/input records and archive integrity.
The backend bundle targets the exact published Engine/Protocol versions recorded
in its receipt. The private browser package retains version 0.0.0: identify its
delivery by source revision, input hashes and npm integrity, not that version.
A receipt marked dirty identifies a local candidate, not a released source.
Copy the selected archives into durable consumer-owned test-support/dependency
locations and follow their installation instructions. Backend scenarios still
need to run against Canic's actual adapter, and the browser still needs Canic's
production issuer/reconciliation and receiver endpoints. Neither archive installs
those components or establishes production acceptance.

Current `AUTHNAT3` is the private qualification checkpoint. Current Auth session,
login and replay discriminators remain unchanged; no new reset or migration is
introduced. Never erase Memory backing or only its allocation ledger to obtain
admission. Cryptographic checks and the TEE/non-TEE trust model are unchanged.

<a id="platform-07-migration"></a>

Historical Platform 0.7 migration notes referenced by older release notes are
preserved in [adoption history](canic-adoption-history.md#platform-07-migration).
