# Canic adoption contract

Auth owns reusable authentication machinery. Canic owns its runtime composition,
issuer approval/renewal, fleet/member/role policy, endpoint guards and deployment.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) tracks production
integration; this guide supplies the contract, not evidence of deployment.
Superseded migration journals remain in Git history; use the current contracts below.

## Current integration cohort

Published **Protocol 0.35.1** (`ic-auth-protocol-types`, `ic-auth-wire`) and
**Engine 0.35.10** (`ic-auth`) form the released cohort. Keep both Protocol packages
aligned and satisfy Engine's declared Protocol requirement; the groups release
independently. Engine's qualification lock uses Platform 0.27.0, Types 0.18.2,
Wire 0.17.14, Testkit 0.93.2, Runtime 0.20.2, Storage 0.69.0 and Metrics 0.29.6.
Testkit retains Platform 0.26.1 only in native qualification; the Wasm host has
one facade and one Runtime executor/generator/guard. The complete Engine gate is
recorded in the [current handoff](status/current.md). Rust 1.99.1 remains the minimum.
[Cargo.toml](../Cargo.toml), the lockfile and [group ledgers](../CHANGELOG.md)
record development selections and release notes, respectively.

The released cohort also selects Build 0.79.1,
Host 0.75.1 and root-owned Toko Tooling 0.34.0. Existing Auth contracts are preserved;
no field selects Wire's opt-in packed 16-byte ID encoding. Both Protocol and
Engine passed complete release gates, recorded in the handoff.
Storage 0.69 preserves current collection images and returns ordinary operation
values directly. Historical image cuts apply only to Auth's private qualification
host here; retired images are refused without clearing. No production store is
migrated by this release.

The clean 0.35.8 backend helper archive passes registry identity checks and all
20 packaged tests. Its ten scenarios include immutable 90-day login deadlines,
late renewal and restore under fresh authority, with unchanged `Backend` hooks.
The handoff records its exact source, checksum and receipt. Retain that archive
and receipt in the consumer and qualify the actual adapter; earlier archives
retain their original scenario sets. Native Testkit 0.93 callers must select a
positive finite request `Duration`; Auth's runner retains 30-second request and
separate startup budgets. This native fixture migration changes no Auth protocol.

Host-selected `SessionLimits` support profiles such as 100,000 sessions without
providing a receiver default or allocating backing. Canic owns its configured
maximum, proportional retention/byte/index budgets and actual-store capacity and
performance qualification. Individual Auth record limits are unchanged. Auth
carries no Account DTOs; the upstream Account byte-adapter cut does not change
Auth frames, method identities or stored records. Qualify the complete consumer
graph with one Runtime executor/generator/guard.

## Clock-independent acceptance

Protocol and Engine 0.35.0 deliver the
[#127 contract](design/portable-clock-audit.md) as a minor hard cut. Keep production activation refused until the actual consumer qualifies it.
Consumers must update the receiver Wire codec and browser bindings together, handle
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

The consumer acceptance boundaries are independent:

| Boundary | Required result | Evidence that does not establish it |
| --- | --- | --- |
| Known-login logout | Atomically remove that caller's session and retain denial through its immutable signed login deadline, including after restore. Return `Revoked` only after commit; preserve state on failure. | A browser timeout, expired proof or fresh enrollment alone does not establish logout. |
| Unknown-login logout | Commit bounded temporary local denial and return `Unconfirmed`. Retries retain its original deadline and uncertainty; after expiry/pruning a delayed credential may admit. | Temporary denial is not confirmation of permanent revocation. |
| Complete durable restore | Restore sessions, replay/login records, authority, generation, clock and pool metadata consistently. Require fresh authenticated enrollment before readiness; preserve deadlines and denials. | Decoding session bytes alone does not qualify complete recovery. |
| Missing safety state | Stay closed pending trusted complete recovery, including when session/login rows happen to be empty. | A timestamp floor, generation increment or new enrollment cannot manufacture lost denial history. |
| Older internally valid snapshot | Qualify the host's protected authority/retirement fence against rollback of the whole accepted state. | Refusing missing metadata does not prove detection of an older valid snapshot; Auth cannot detect rollback when every trusted input is rolled back with it. |
| Ninety-day login | Preserve the original login window across renewal and restore. Keep proofs short and session leases bounded; refuse authorization exactly at the immutable deadline without relying on cleanup. | A 90-day login is not a 90-day session or permission to extend the deadline on renewal. |

The signed-deadline contract does not require a cross-canister clock-skew bound
for those logout guarantees. Hosts still supply trusted local time, preserve its
monotonic observations and qualify their recovery/authority lifecycle. Do not
remove production activation gates merely because Auth's #127 is closed.

## Read-only Canic review — 2026-11-07

Reviewed Canic HEAD `e6a061999dfae207bc20917ffefa2082e7ffe2e7` plus its
uncommitted source, authentication guide, current handoff and retained results.
The reviewed root selects Engine 0.35.5 and Protocol/Wire 0.35.0. The following
results qualify the existing consumer composition locally; this review did not
rerun those suites or establish a delivered release or mainnet deployment.
Earlier dated reviews remain in Git history.

| Boundary | Inspected consumer evidence |
| --- | --- |
| Receiver and storage | 179 native Auth cases and 51 actual stable-adapter cases pass, including the retained 0.35.1 conformance helper and shared pair/capacity checks. |
| Retention and capacity | Installed 4,096-login / 2,048 maximum-scope-session / 256 unknown-logout-denial profile passes exact retry, full-pool known logout, reconstruction, bounded expiry cleanup and fresh admission. |
| Issuer and authority | Installed Root membership, permission changes across awaits, certified approval/retirement, query-proof recovery, maximum-grant byte refusal and issuer-history reclamation pass. |
| Browser | Real Internet Identity with virtual WebAuthn passes issuance/recovery, renewal/reopening, two receivers/devices, logout and sign-out refusal; consumer deletion and supplier shutdown are recorded. |
| Lifetime and restore | Installed startup, refresh and snapshot cases preserve original deadlines and refuse missing safety metadata. Canic's selected full-snapshot contract is described below. |

Evidence is retained in Canic's `target/evidence/auth-patch-acceptance/` and
`target/evidence/auth-authority-capacity/`. Each phase retains its own lock:
the browser/initial installed/capacity results use Wire 0.17.9; later issuer
cases use Wire 0.17.10. They are not one new full-gate result on the final graph.
The inspected records support functional completion of that existing composition;
no missing Auth API or additional Auth implementation defect was identified.

Canic now documents removing its temporary clock-skew activation gate under the
signed-deadline contract, after its own local acceptance. Its accepted full
same-release snapshot restores captured sessions, logout state and original
deadlines; changes made after capture, including logout, roll back with that
snapshot. This does not establish the independent whole-state rollback protection
in the matrix above. Missing metadata refusal remains distinct. No new Auth trust
mechanism, remote authorization check or extension of signed deadlines is implied.

### Adapter extraction now in progress

After the functional closeout, Canic's current handoff records a newly accepted
extraction into a real `canic-auth` integration package with an explicit mount.
Application authorization and restoration boundaries have begun moving, but the
package, mount, storage/memory/lifecycle composition and installed acceptance of
the extracted arrangement are incomplete. The proposed mount is not yet an
available, qualified API. Prior tests qualify the earlier composition, not this
new arrangement. Core retains Fleet/Root authority; reusable Auth engine ownership
stays here, with no dependency from Auth back into Canic.

Canic's [production integration tracker](https://github.com/dragginzgame/canic/issues/491)
remains the work owner. Its supplier alignment, selected release gate, delivery
and deployment are separate consumer work. No Canic source, test, issue or release
state was changed by this Auth review.

### Backend bundle handoff

Canic still consumes the clean 0.35.1 helper under
`integrations/auth-backend-conformance`, as an excluded development-only package.
The already-qualified 0.35.8 helper adds the 90-day renewal/reopen case and expanded
authority refusals without changing the `Backend` hooks. Its exact source,
archive SHA-256 and receipt are in the
[clean delivery record](status/current.md#clean-backend-helper-delivery).
Retain that archive and receipt in Canic custody, preserve its normalized package
metadata, and run the existing complete conformance entrypoint against the actual
driver after relocation. See the [consumer refresh instructions](../crates/ic-auth-backend-conformance/README.md#refreshing-an-existing-consumer).
This is ready Auth-side support; it does not claim Canic adopted or passed the
new helper.

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
activation cutoffs and authority metadata from one protected generation. Obtain fresh
receiver-bound enrollment before readiness; restoration alone never opens access.
Volatile upgrade/state loss stays closed pending trusted complete recovery;
local timestamp floors and fresh enrollment alone cannot reopen it. Bounded
cleanup reclaims expired rows without evicting live replay/denial state.
See [storage and transaction details](sessions.md#custom-host-transaction-contract).

The released-source [backend conformance bundle](../crates/ic-auth-backend-conformance/README.md)
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

The [backend conformance package](../crates/ic-auth-backend-conformance/README.md)
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

Current `AUTHCLK1` is the private qualification checkpoint. The 0.35 hard cut
selects login record identity `0x415554485323` and logout Wire identity
`0x415554480014`; session/replay identities are unchanged. Old affected bytes
refuse without clearing. Auth's no-retained-record confirmation does not authorize
resetting a consumer's stores. Never erase Storage backing or only its allocation
ledger to obtain admission. Cryptographic checks and the TEE/non-TEE trust model
are unchanged.

<a id="platform-07-migration"></a>

Historical Platform 0.7 migration notes referenced by older release notes are
preserved in [the migration journal in Git](https://github.com/dragginzgame/ic-auth/blob/a11f7558bd2187c06fedc739c7a0c36b470ba9ca/docs/canic-adoption-history.md#platform-07-migration).
