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

## Published dependency and feature selection

Both `ic-auth` and `ic-auth-protocol-types` 0.5.8 are published and unyanked.
Their registry checksums match IC Auth release
`12bae9fa3d16209b427f0e9f2f558b9bdd452969`. Rust 1.88 is the supported minimum.
The pure preparation API requires at least 0.4.2; 0.5.7 preserves its Rust contract.
Select compatible registry requirements in Canic's
root catalog and record the resolved versions in its own lockfiles:

```toml
[workspace.dependencies]
ic-auth = { version = "0.5.8", default-features = false }
ic-auth-protocol-types = "0.5.8"
```

Child packages inherit these dependencies with `workspace = true`; their
existing capability features forward only the needed library features. This is
the recommended published selection. The read-only review below distinguishes Canic's
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
acceptance. Compatible Auth 0.5.1 through 0.5.7 introduce no new API prerequisite.

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

The 2026-10-13 audit observes local Canic 0.112.0 plus held changes, while
remote main remains 0.111.0. It also reconfirms
[Canic #506](https://github.com/dragginzgame/canic/issues/506#issuecomment-6141222675):
ROOT must retain validated issuer-effective installation deadlines in renewal
state. The current coordinator discards the successful typed response and stores
declared deadlines. This remains consumer-owned alongside final facade, cleanup,
logout and browser acceptance; Auth 0.5.8 adds no required API for those repairs.

Producer 0.5.4's [complete native matrix](https://github.com/dragginzgame/ic-auth/actions/runs/38247803905)
qualifies its retained library and actual canister fixture on Linux, Intel macOS
and ARM macOS. Published 0.5.7's [own run](https://github.com/dragginzgame/ic-auth/actions/runs/38284363288)
passes all three native hosts; 0.5.6's prior run was cancelled after its Linux pass.
Neither proves Canic's changing endpoints or persistence. Compatible 0.5.7
adopts contributor tooling and adds no new library API prerequisite beyond
Canic's selected 0.5.7. Durable store, rollback/restart/upgrade and deployed
acceptance stay with the consumer owner.

| Existing Canic capability | IC Auth selection | Retained Canic responsibility |
| --- | --- | --- |
| Protocol DTOs and canonical encoding | Protocol-types and default `ic-auth::canonical` | Fleet/configuration envelopes and checked deployment-identity projections |
| Pure delegated-token issuance | Default `token_preparation` in published 0.4.2 | Installed authority, authenticated presenter, host bounds, operation custody and authorized signing |
| Root/issuer canister-signature verification | `canister-signature-verification` | Protected signer, seed, network trust anchor, time and limits |
| Root/issuer canister-signature creation | `canister-signature-preparation` | Authorized message construction, runtime certificate, certification-root composition and lifecycle |
| `auth-delegated-token-verify` | `token-verification` | Independent key enrollment, fleet/role policy, authenticated caller and endpoint requirements |
| `auth-local-application-authorization` | `sessions` | Durable transaction backend, memory IDs, generation transitions and endpoint guards |
| Root chain-key signing and renewal | Existing Canic effect adapter | Management-canister signing, key eligibility and orchestration; there is no library signing endpoint |

Token verification enables signature verification; sessions enable token
verification. Signature preparation is independently selectable. No feature
imports Canic, Toko, Solana, CDK, stable memory or native Host/Testkit tooling.
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
[token contract](tokens.md) specifies exact binding and live-policy checks.

Replace the generic proof callbacks and semantic/cache-only admission path with
the complete verifier. Its `VerifiedToken` cannot be publicly constructed or
deserialized. Do not retain a second positive proof cache that bypasses changed
policy, key windows or epochs. Fleet member/role decisions, issuer approval,
renewal orchestration and application resource authorization stay in Canic.

## Pure issuance preparation (released 0.4.2)

[Auth #25](https://github.com/dragginzgame/ic-auth/issues/25) adds the default-feature
`ic_auth::token_preparation` boundary described in the
[preparation contract](token-preparation.md). Canic can project its protocol DTOs
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

The target session engine calls `session::establish_session`; protected operations call
`session::authorize_session` with current context. Use `SessionRequest::new`
from `session_request` in released 0.5.0 for canonical scope/TTL request identity.
Do not wrap the old session engine
behind the library or use its public verified-authority constructor to bypass
proof verification.

Align authority and expiry contracts before replacing Canic's session engine.
The three deadlines have distinct meanings:

| Deadline | Existing library contract | Consumer projection |
| --- | --- | --- |
| `VerifiedToken::expires_at_ns()` | Current proof admission deadline, capped by accepted root policy | Use for proof eligibility; it is not the signed token's original expiry or a committed session lease |
| `Session::expires_at_ns()` | Requested/default lease capped by `RootKeyPolicy::accept_until_ns` | Direct session-engine adoption must accept this authority cap explicitly |
| `Replay::remove_at_ns()` | Original authenticated `claims.expires_at_ns` | Preserve the signed proof's replay retention independently of logout, lease expiry or a shorter current policy window |

Canic's existing workflow accepts an independently supplied committed-session
expiry and retains replay until its authority projection's proof expiry. The
owner has explicitly preserved independent lease semantics in its pending
cache-removal slice. Mapping the capped verified-token deadline into that
projection can silently shorten replay retention; applying the library session
engine can also shorten a lease at the root-policy boundary. Do not fabricate a
longer enrolled root window to conceal the difference. Canic must resolve that
authority contract under [#491](https://github.com/dragginzgame/canic/issues/491)
and [#459](https://github.com/dragginzgame/canic/issues/459) before direct engine
adoption. This is a consumer decision, not an Auth compatibility shim or a change
to the existing library's session policy.

The released [root-deadline qualification](ic-testkit-qualification.md) exercises
this distinction with an actual certified token: a capped session expires before
the signed proof, and a later eligible enrollment cannot reuse it after session
pruning. [Auth #22](https://github.com/dragginzgame/ic-auth/issues/22) owns producer
qualification; it does not select Canic's policy or qualify consumer persistence.

Canic implements `SessionStore` and `SessionTransaction` against its actual
durable host store. Reads, capacity checks and the staged session/replay pair
observe one isolated synchronous transaction. Closure or commit failure leaves
records, indexes, counters, generation and accounting unchanged. Preserve replay
tombstones through logout, replacement and authority invalidation; advance
protected generation atomically with invalidating configuration changes.
`MemorySessionStore` is a volatile reference and cannot replace durable replay
state in production. The [session contract](sessions.md) owns limits, exact retry
and restore obligations.

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

## Consumer acceptance boundary

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

## Current Canic adapter boundary

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
their existing contracts. See the [diagnostic migration](tokens.md#root-binding-diagnostics).

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
[standalone contract](tokens.md#standalone-root-delegation-verification).
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
