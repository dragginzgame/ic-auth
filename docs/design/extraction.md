# Independent authentication ownership

Accepted direction: 2026-10-08. Implementation and Canic adoption are tracked in
[Canic #491](https://github.com/dragginzgame/canic/issues/491).

## Purpose and scope

IC Auth is the independent owner of reusable IC signature support,
application-token protocol mechanics and local application-session machinery.
A separately deployed wallet-auth service uses that foundation to authenticate
wallet users and issue standard IC ingress delegations. The initial wallet
provider is Solana message signing without SIWS or ic-siws.

The larger extraction includes token contracts, canonical signed encoding,
certificate rules, verification and cache validity, scoped session admission,
replay rules and bounded state operations. It is not limited to a wrapper around
DFINITY signature crates.

The service authenticates identities. Applications continue to own users,
business permissions and assets. Toko NFTs remain in Toko; neither a Solana RPC
service nor Solana token/NFT infrastructure is needed for wallet login.

## Dependency direction

```text
ic-auth-types <--- ic-auth <--- wallet-auth service
                        <--- Canic adapter and local authorization host

wallet-auth public Candid <--- TypeScript client <--- application frontend
```

No IC Auth package may import Canic, Toko, their storage models or their build
scripts. Canic supplies its trusted authority context through explicit adapter
inputs. Its consumers select verification/session functionality without wallet
or issuer signing dependencies when those functions are not needed.

## Packages

| Package | Responsibility |
| --- | --- |
| `ic-auth-types` | Passive public protocol contracts, validated identifiers and errors. No CDK, storage, timers or certificate-tree state. |
| `ic-auth` | Protocol encoding, certificate/token verification and issuance rules, local session state transitions, reusable signature support and optional wallet proof verification. Pure logic and platform integration occupy separate modules. |
| `apps/wallet-auth` | Service endpoints, protected application configuration, identity mappings, nonce state, storage backend, certified signature tree, controller policy and lifecycle. |
| `packages/client` | Wallet discovery/signing, challenge exchange and construction of a delegated ICP client identity. |

Use a virtual root with one lockfile and root-owned package/dependency catalogs.
Create additional crates only for an established independent dependency boundary;
do not allocate one crate per wallet, signature algorithm or source module.

## Source ownership map

The inspected Canic source was at
`95038e1a381ce88eb19415f5bb6d8439457b75f1`. Recheck the selected source before
extracting: this design does not freeze another contributor's working tree.
Paths below are relative to `crates/canic-core/src/`.

| Current source | IC Auth ownership | Retained Canic ownership |
| --- | --- | --- |
| `ops/auth/canister_sig_key.rs`, IC-key portions of `domain/auth.rs` | Key encoding/parsing and reusable trust-key checks. Prefer suitable upstream implementations. | Protected network/root selection and fleet-specific derivation policy. |
| `ops/auth/root_canister_sig.rs`, `issuer_canister_sig.rs` | Common signature prepare/retrieve/verify machinery and explicit certification support. | Attestation payload meaning, issuer eligibility and role-specific domains/authority. |
| `dto/auth/{token,proof,common}.rs` | Owned token/certificate/proof contracts after closing dependencies. | Fleet command/configuration envelopes and infrastructure-role contracts. |
| `ops/auth/delegated/{canonical,verify,audience,cert_rules,delegation_cert,prepare}.rs` | Canonical bytes, proof binding, validity, narrowing and issuance rules. | Resolving exact fleet/role membership, approved policies and authorized signing effects. |
| `ops/auth/delegated/cache.rs`, pure crypto parts of `chain_key*.rs` | Bounded verification cache semantics and cryptographic proof checks. | Runtime metrics, management-call orchestration and fleet key/renewal authority. |
| `model/auth/application_authorization/`, pure application-authorization policy | Invariant-bearing authority/session models, scopes, admission, replay, expiry and generation checks. | Actual protected generation transitions on fleet configuration changes. |
| `workflow/auth/application_sessions.rs`, `ops/storage/auth/application_sessions.rs` | Reusable session engine and atomic storage contract. | Local memory allocation, runtime context acquisition and lifecycle hooks. |
| `access/auth/`, `api/auth/`, runtime auth provisioning/renewal workflows | Calls into the extracted library; no copied engine. | Guards, endpoint mapping, caller/time acquisition and fleet orchestration. |

Some files mix these responsibilities and must be split. Moving whole directories
would import Canic authority into the new library. In particular, the present
verifier, canonical encoder and session authority types directly reference
`FleetKey` and `CanisterRole`.

Define validated auth-domain audience/role identifiers and explicit checked Canic
projections. Do not weaken exact comparisons into arbitrary unvalidated strings.
Preserve existing signed-byte vectors and domains unless a protocol change is
explicitly accepted. A source-package relocation must not silently change token
meaning. Coordinate contract ownership with
[Canic #354](https://github.com/dragginzgame/canic/issues/354); generic auth types
belong to IC Auth, while Canic fleet envelopes remain Canic-owned.

## Three different authority paths

1. A wallet signs a bounded challenge proving control of its key.
2. The auth service uses an IC canister-signature identity to delegate to a
   browser session key. The IC verifies the delegation for ingress calls.
3. Application tokens and local sessions carry explicit application grants and
   are verified/authorized locally by the consuming application.

The same library may support these paths, but their messages, signing domains,
validation rules and authority must remain distinct. A valid wallet signature
does not authorize an application role. An application token cannot be used as
an IC ingress delegation. Fleet admission does not confer NFT ownership.

## Embedded session engine

Local application-session verification stays local. The new auth service is not
an RPC gate in front of every application request. The host supplies actual
caller, trusted time, protected issuer/audience/role policy and authority
generation; the library checks the requested operation's scope against that
context.

Session admission and replay consumption need one atomic commit boundary. A
failed capacity or authority check must not leave partial replay/session state.
Specify exact-retry behavior, expiry pruning, per-subject/global quotas and byte
limits. Use bounded record operations rather than cloning/encoding the entire
store for each login. IC Auth owns the storage contract and engine; an embedded
host owns stable-memory registration and supplies its backend.

Proof caches must not turn an earlier signature check into authority after a
policy window closes, an epoch changes or the caller/context differs. Separate
cached cryptographic work from live semantic checks.

## Certification ownership

A canister has one published certification root. The library must expose tree
contributions/witness construction through an explicit host-owned composition
boundary. It must not privately call `certified_data_set` behind an independent
subsystem's back.

The standalone service owns its tree. A Canic host remains responsible for
composing any other certified branches. Specify preparation retention,
retrieval, pruning, root refresh and lifecycle restoration together. A library
move alone does not fix competing owners.

## Wallet service boundary

The public interaction is challenge preparation, proof submission and certified
delegation retrieval. Final method/type names are fixed with the implementation
and generated Candid, not duplicated manually in consumer clients.

The challenge binds the configured application identity/domain, wallet key,
session public key, nonce, issue/expiry times and permitted delegation targets.
The service reconstructs the expected challenge from its own state, validates
the precise supported wallet message encoding and signature, and atomically
consumes it. Bound unauthenticated preparation, message size and retained state.
Use feature detection for supported wallet signing methods; do not silently
replace a login message with a transaction.

A durable mapping and stable per-identity seed define the IC principal. Fresh
browser keys must recover the same owner. Replacing the signing canister ID can
change principals; deployment and upgrades must treat identity continuity as an
explicit invariant. Canic's reinstall-only policy does not govern this service.

The service's code and controllers can issue delegations for these identities.
Document that trust boundary. Recovery and wallet linking require explicit proof
and policy, and cannot simply recreate an existing Internet Identity principal.
Consumer resource ownership remains independently checked.

## Contract decisions before wallet-service implementation

The extraction can begin before these service-specific choices are settled.
Freeze and review them before exposing the wallet service as a supported login
provider; implementation must not choose them implicitly:

- **Identity namespace:** decide whether a wallet receives an application-scoped
  principal or a shared principal across approved applications. Specify the
  immutable application identifier, seed derivation, environment separation and
  domain-change behavior. A display name or mutable web hostname must not
  accidentally become the sole durable identity key.
- **Linking and recovery:** define proof requirements for adding/removing a wallet,
  concurrent-link conflicts and lost-wallet recovery, including whether recovery
  is supported initially. Keep Toko account linking separate from service-owned
  credential linking and document the controller's authority.
- **Lifetime and revocation:** distinguish browser logout, removal of a linked
  wallet, application-token/session revocation and an already-issued IC ingress
  delegation. State which events invalidate each credential, the maximum remaining
  validity and which checks enforce that guarantee. Never equate clearing local
  browser state with invalidating every issued credential.
- **Exact public contract:** freeze canonical challenge bytes and supported wallet
  message formats, API types/errors, retry/concurrency behavior, target-policy
  selection and finite time/size/capacity limits. Protected application registration
  must supply allowed domains and targets; client-supplied domain text is not
  proof of the browser's actual origin.

Attach concrete choices and their evidence to
[the extraction tracker](https://github.com/dragginzgame/canic/issues/491), then
update the corresponding contract sections here. These are unresolved design
choices, not implemented or qualified service behavior.

## Extraction sequencing and consumer adoption

The tracker owns the sequenced implementation slices: close the type boundary;
extract signature support; extract token engine; extract session engine; adopt
and remove superseded Canic implementations; implement standalone wallet login
and the client. These are one accepted larger scope, not permission to describe
the first small move as the completed extraction.

Canic adoption updates direct imports, feature gates, generated/public Candid,
endpoint/configuration adapters, fixtures and docs in one coherent change. It
removes moved source and unnecessary direct dependencies. Do not retain
compatibility aliases or parallel implementations after adoption.

Canic's current release boundary remains governed by
[#459](https://github.com/dragginzgame/canic/issues/459). This design does not
assign a new Canic release or authorize reset/publication. IC Auth's initial
package version and manifests are established in its first implementation batch;
there is no published release history to infer from this bootstrap.

Toko maintainers receive a client/service contract and a minimal independent
consumer demonstration. Actual Toko integration and its acceptance remain their
work; extracting Canic must identify every affected downstream public surface
without editing Toko implicitly.

## Qualification

Revalidate the linked findings in #491 against the chosen extraction input.
Moving source does not resolve them. Prioritize certificate composition, exact
issuer seed binding, cache-policy expiry, anonymous caller rejection, bounded
retention/admission and session-storage work.

Require independent native and Wasm consumers, focused Canic feature-selected
checks, signed-byte fixtures and generated Candid checks. Cover wrong
caller/subject/issuer/seed/audience/role/scope/network, time boundaries, policy
epochs, replay, exact retry, atomic failure and authority invalidation.

Use PocketIC to prove real ingress delegations and stable caller identity across
fresh browser sessions and the service's supported upgrade path. Demonstrate
login and an owner-checked operation in an independent consumer before asking
Toko to adopt. Measure footprint or performance changes before claiming savings.

No test results, crate versions, deployment or downstream adoption are implied by
this design.
