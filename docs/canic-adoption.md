# Canic adoption contract

Auth owns reusable authentication machinery. Canic owns its runtime composition,
issuer approval/renewal, fleet/member/role policy, endpoint guards and deployment.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) tracks production
integration; this guide supplies the contract, not evidence of deployment.
Superseded migration journals remain in Git history; use the current contracts below.

## Current integration cohort

Published **Protocol 0.35.0** (`ic-auth-protocol-types`, `ic-auth-wire`) and
**Engine 0.35.2** (`ic-auth`) form the released cohort. Keep both Protocol packages
aligned and satisfy Engine's declared Protocol requirement; the groups release
independently. Engine's qualification lock uses Platform 0.23.2, Types 0.18.2,
Wire 0.17.4, Testkit 0.89.0, Runtime 0.19.3, Storage 0.68.0 and Metrics 0.29.3.
Platform 0.23/browser SDK 0.2 adoption and the complete Engine gate are recorded
in the [current handoff](status/current.md). Rust 1.99.1 remains the minimum.
[Cargo.toml](../Cargo.toml), the lockfile and [group ledgers](../CHANGELOG.md)
record development selections and release notes, respectively.

The released cohort also selects Build 0.75.0,
Host 0.74.4 and Shared Tooling 0.31.0.
Storage's collection image cut applies only to
Auth's private qualification host here; fresh fixture allocation is required and
old images are refused without clearing. This is not an in-place production-store
upgrade. Newer dependency selections remain deferred as recorded in the handoff.

Host-selected `SessionLimits` support profiles such as 100,000 sessions without
providing a receiver default or allocating backing. Canic owns its configured
maximum, proportional retention/byte/index budgets and actual-store capacity and
performance qualification. Individual Auth record limits are unchanged. Auth
carries no Account DTOs; the upstream Account byte-adapter cut does not change
Auth frames, method identities or stored records. Qualify the complete consumer
graph with one Runtime executor/generator/guard.

## Pending clock-independent adoption

Protocol and Engine 0.35.0 deliver the
[#127 contract](design/portable-clock-audit.md) as a minor hard cut. Keep production activation refused until the actual consumer qualifies it.
Canic must update the receiver Wire codec and browser bindings together, handle
`Unconfirmed` without reporting logout success, and select the explicit protected
login ceiling in ledger construction. Known logout retains denial through its
signed immutable deadline; unknown logout commits only bounded temporary denial.

Preserve complete session/replay/login and protected authority metadata on normal
upgrade, then obtain current enrollment before readiness. Remove timestamp-based
lost-state reopening: incomplete state remains closed pending trusted recovery.
Old login records and fixture payloads are refused under fresh identities; Auth's
no-retained-record confirmation does not authorize clearing Canic's actual stores.
Qualify disposition of its installations, failure atomicity, retained denial,
unknown outcomes and browser recovery under #491. No Canic source was changed.

## Read-only Canic source review — 2026-11-04

This historical review used Canic HEAD `e6a061999` plus its then-uncommitted
work. Its lock selected Engine 0.33.2, Protocol 0.33.1, Platform 0.22, Types 0.18.1,
Wire 0.16.2 and Testkit 0.85.1. These are consumer selections, distinct from
Auth's released qualification lock above. No Canic edits, builds or fresh
installed acceptance were performed by this review.

Canic's authentication guide and current handoff record the following evidence:

| Boundary | Recorded consumer progress | Remaining production obligation |
| --- | --- | --- |
| Receiver and storage | Persistent admission/read/logout, scoped guards, Root refresh, and Auth 0.33.1 backend conformance against the actual stable adapter | Complete denial/recovery acceptance, storage profiles and combined-state costs; the newer Auth backend maintenance hooks need their own matched receipt |
| Issuer | Controlled-local Ed25519 issuance, renewal and reconciliation | Certified issuance and the complete production approval/custody lifecycle |
| Browser | Controlled-local Chromium journey with two receivers/devices, lost replies and durable logout across reopen | Production account-login UI, key custody and activation |
| Authority and restore | Scoped Root/issuer snapshot retirement checks; new installed unavailable-enrollment, refresh-expiry and retirement checks | Clock-independent contract adoption and arbitrary unsealed restoration remain consumer-owned |
| Operator inspection | Protected receiver inspection and controller/Root/CLI paths | Final consumer acceptance and documentation reconciliation |

The persistent adapter uses `Session::check_admission_pair` and
`SessionLimits::check_capacity`, as requested by
[Canic #720](https://github.com/dragginzgame/canic/issues/720). Production
activation remains refused; controlled-local proofs do not establish production
clock/storage provenance or adoption of the pending clock-independent contract.
The latest focused cases are not a fresh complete passing integration target,
and earlier browser evidence retains its original dependency graph. Canic
[#491](https://github.com/dragginzgame/canic/issues/491) remains the acceptance
owner. No missing Auth API was identified.

Certified issuance is the next concrete consumer gap. Use the existing
[certified issuance handoff](signature-preparation.md#certified-portable-issuance-handoff)
to compose Auth's preparation/retrieval APIs with Canic's durable journal.
The Platform Wasm publication hold was subsequently lifted with the maintainer's
accepted residual size tradeoff. The [current handoff](status/current.md) records
that decision and released qualification; it does not update this historical
Canic acceptance review.

## Current Rust contract

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
Volatile upgrade/state loss stays closed pending trusted complete recovery;
local timestamp floors and fresh enrollment alone cannot reopen it. Bounded
cleanup reclaims expired rows without evicting live replay/denial state.
See [storage and transaction details](sessions.md#custom-host-transaction-contract).

The released-source [backend conformance bundle](../apps/qualification/backend-conformance/README.md)
now includes cleanup rollback/work-count bounds and clock/generation reopening
scenarios, in addition to admission/logout checks. Consumer test drivers supply
the new maintenance/clock hooks and fallible generation update. This is Auth-side
qualification support; actual backing costs, clock provenance, authority changes
and installed Canic acceptance remain with the consumer.

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

The following migration sections describe historical cohorts, preceding 0.35. The published
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

## Matched qualification handoff

For a matched handoff, run `make test-backend-package` and
`make test-client-package` from the same fixed Auth source after preparing their
declared dependencies. These existing commands create and qualify the actual
backend `.crate` and browser `.tgz`; separate pack-only runs are unnecessary.
Retain both `delivery.json` receipts, source/input records and archive integrity.
To include real Chromium/IndexedDB archive qualification, select a prepared
`IC_AUTH_BROWSER_EXECUTABLE` and use `make test-client-package-browser` for the
browser bundle. It includes the ordinary package checks and injected transports;
the live receiver/production endpoint qualification remains separate.
The backend bundle targets the exact published Engine/Protocol versions recorded
in its receipt. The private browser package retains version 0.0.0: identify its
delivery by source revision, input hashes and npm integrity, not that version.
A receipt marked dirty identifies a local candidate, not a released source.
Copy the selected archives into durable consumer-owned test-support/dependency
locations and follow their installation instructions. Backend scenarios still
need to run against Canic's actual adapter, and the browser still needs Canic's
production issuer/reconciliation and receiver endpoints. Neither archive installs
those components or establishes production acceptance.

The previous assembled local handoff is
`target/platform-021-delivery/delivery/manifest.json`, built from clean release
`f9105b9` with Engine 0.33.1 and Protocol 0.33.0. Earlier
`target/adoption-boundaries/delivery/` and `target/canic-adoption-033/delivery/`
remain separate candidate evidence.
It identifies the two exact archives, their receipts/input records and the source
and published Engine/Protocol versions used for qualification. Run
`sha256sum --check SHA256SUMS` from that delivery directory before copying it.
The browser's npm `integrity` is retained separately from its SHA-256.

The historical Engine 0.33.2/Protocol 0.33.1 backend archive has its own clean-source
registry-consumer qualification receipt at
`target/portable-fixtures/backend-package.WoSpAu/delivery.json`; the
[handoff at that source revision](https://github.com/dragginzgame/ic-auth/blob/a11f7558bd2187c06fedc739c7a0c36b470ba9ca/docs/status/history-0.33-delivery-and-qualification.md) records its delivery. The older
assembled pair does not qualify the new backend maintenance/clock hooks.

Extract the backend archive into the consumer's excluded `test-support/`
directory and use its normalized manifest as a development dependency; do not
point Cargo at Auth's working workspace. Implement its `Backend` driver for the
actual store and run `assert_backend_contract`, including authorization and
maximum-shape admission/reopen checks. Shape conformance is not a measurement
of the actual host's cost or capacity at representative occupancy.
Install the browser archive through an exact `file:` dependency, retain the
archive/integrity and resulting consumer lock, and qualify the selected SDK
graph and real issuer/receiver transports. The private package's `0.0.0` version
cannot distinguish these artifacts.

Those release receipts mark their exact source as clean. Earlier dirty-source
receipts identify development candidates and must not replace that source
selection. Clean archive qualification still requires the actual consumer's
store and production transports to pass its acceptance.

Current `AUTHNAT3` is the private qualification checkpoint. Current Auth session,
login and replay discriminators remain unchanged; no new reset or migration is
introduced. Never erase Memory backing or only its allocation ledger to obtain
admission. Cryptographic checks and the TEE/non-TEE trust model are unchanged.

<a id="platform-07-migration"></a>

Historical Platform 0.7 migration notes referenced by older release notes are
preserved in [the migration journal in Git](https://github.com/dragginzgame/ic-auth/blob/a11f7558bd2187c06fedc739c7a0c36b470ba9ca/docs/canic-adoption-history.md#platform-07-migration).
