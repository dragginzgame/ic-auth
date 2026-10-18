# Independent authentication ownership

Accepted extraction direction: 2026-10-08; current boundary reconciled on
2026-10-18. [Canic #491 — Extract reusable authentication, token and session machinery](https://github.com/dragginzgame/canic/issues/491)
owns consumer implementation and acceptance. Earlier source checkpoints are
historical evidence, not alternate current APIs.

## Purpose and scope

IC Auth owns reusable IC signature/proof support, portable credential contracts,
certificate rules and caller-bound local session/replay/revocation machinery.
Released 0.9 also supplies bounded Wire transport and a private admission/logout
client. The separate Rust application-token lifecycle and old browser token
prepare/retrieve client are retired after explicit deployment/uncertainty review.

A wallet-auth service remains planned. No wallet provider, accepting wallet
endpoint or standard IC ingress-delegation service is implemented here. Its
future identity/nonce/storage lifecycle belongs here, independently of Canic's
fleet lifecycle. Applications retain users, business permissions and assets;
Toko NFTs remain in Toko. Auth requires no Solana RPC, minting or asset custody.

## Dependency direction and entry points

| Package | Owned implementation and public surface |
| --- | --- |
| `ic-auth-protocol-types` | Checked identifiers, passive claims/certificates/proofs and typed logout results. No CDK, storage, timers or certificate-tree state. |
| `ic-auth` | Canonical signed bytes, certificate/proof verification and issuance rules, signature preparation and transaction-based local sessions. Verification/host effects are explicit capabilities; the crate installs no endpoints or trust. |
| `ic-auth-wire` | Bounded Auth method/data layouts and structural domain projections over IC Wire. Generated browser codecs come from that dependency's generator. |
| `packages/client` | Private portable admission coordinator, guarded SDK identity and durable logout journal/transport. Identity, issuer/admission operations and storage are injected. |
| `apps/tooling` | Native file operations through IC Host and generic lock admission through IC Build. Auth owns release identity, exact package policy and upload/retry decisions. |
| `apps/qualification` | Private CDK/Testkit hosts, bounded Wire controls and an IC Memory fixture checkpoint. Reference integration evidence, not production service or storage. |
| `apps/wallet-auth` (planned) | Future wallet endpoints, protected configuration, identity mappings, nonce state, storage, certification and lifecycle. No service exists yet. |

The Rust public entry points are the library modules under `crates/`; the private
browser export is `packages/client/src/index.ts`. Auth Wire methods carry layouts
and limits, never automatically installed canister exports. Native/tooling/test
applications are unpublished and outside all public library graphs.

```text
ic-auth -------------------> ic-auth-protocol-types
ic-auth-wire --------------> ic-auth-protocol-types + ic-wire codec/generator
private browser client ----> generated Auth Wire contracts + injected SDK boundaries
Canic/local host ----------> selected Auth verification/session capabilities
future wallet-auth service -> Auth primitives (no Canic/Toko dependency)
```

Use one virtual workspace/lock and root dependency catalog. Generic native file,
process, build and canister-test mechanics belong to Host, Build and Testkit.
Auth supplies its policies and keeps focused consumer integration assertions.
Do not extract a generic abstraction merely because two trust boundaries validate
similar values: signed claims, transport frames and stored records have different
obligations. Private controller contracts and canonical application Wire projections are
necessary transport isolation, not competing authentication engines.

## One application authentication flow

The [portable authentication design](0.6-portable-authentication.md) and
[protocol](0.6-auth-protocol.md) select proof admission, then caller-bound local
session checks. Certified/BLS and enrolled Ed25519 proofs use the same engine.
Both verifiers are implemented; production issuer/enrollment and optional TEE
assurance remain open. Plain Ed25519 verification proves no hardware custody.
Receiver-to-issuer calls are not the selected multi-subnet admission architecture.

A directly authenticated fresh browser principal identifies one login/device;
`(issuer, subject)` identifies its account. Stable-account ingress delegation is
a separate capability, not a substitute for that browser principal. Each permitted
receiver independently verifies admission and commits its session/replay/login
state; one credential does not imply one global verification or automatic tree
access. Ordinary calls carry no proof and use protected local scope checks.

Canic owns fleet/member/role authority, issuer approval and renewal, enrollment,
real caller/time acquisition, endpoint guards, certification publication and
atomic production storage. Toko owns account linking and resource permission.
Auth must not discover trusted authority from client claims or depend on either
product. Root proof verification, signature-store prepare/retrieve and certificate
rules remain reusable primitives; they do not restore the retired token lifecycle.
See [current Canic integration](../canic-adoption.md) and
[session storage/reset contracts](../sessions.md).

## Historical extraction input and retained owner boundary

The inspected Canic source was at
`95038e1a381ce88eb19415f5bb6d8439457b75f1`. Recheck the selected source before
extracting: this design does not freeze another contributor's working tree.
Paths below are relative to `crates/canic-core/src/`.

| Original extraction input | IC Auth ownership | Retained Canic ownership |
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

In the 0.6 application profile, account authentication from paths 1/2 may authorize
an issuer binding, but application calls in path 3 use the fresh browser key's
own principal. Fresh delegation keys under one stable account principal do not
provide independent per-device sessions in the caller-keyed session store.

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

The host selects memory or persistent storage at build time per canister
role. Both use that same engine and trait; neither IC Auth nor a standalone
consumer depends on Canic configuration. Memory mode omits `session-restore`,
loses sessions on upgrades and applies protected signed login-start and proof
issuance floors after each loss of replay history. Newly issued renewal of an old
login must still refuse. Persistent mode enables checked restoration and
retains replay/authority state atomically. The [storage and reset contract](../sessions.md)
defines the clock, cutoff, client recovery and mode-switch requirements. A local
generation increment alone cannot invalidate an old token for fresh admission.

Proof caches must not turn an earlier signature check into authority after a
policy window closes, an epoch changes or the caller/context differs. Separate
cached cryptographic work from live semantic checks.

## Certification ownership

A canister has one published certification root. The library must expose tree
contributions/witness construction through an explicit host-owned composition
boundary. It must not privately call `certified_data_set` behind an independent
subsystem's back.

The future standalone service owns its tree. A Canic host remains responsible for
composing any other certified branches. Specify preparation retention,
retrieval, pruning, root refresh and lifecycle restoration together. A library
move alone does not fix competing owners.

## Planned wallet service boundary

Wallet login uses an IC Auth-owned challenge format and adapters for Solana and
Ethereum, without SIWS or SIWE protocols or dependencies. Use established
signature primitives and wallet message-signing APIs, not custom cryptography.
The application login profile binds a fresh browser principal to an account;
the stable-principal ingress delegation service described below is a separate
capability, not an alternative application endpoint authentication flow. See the
[0.6 wallet boundary](0.6-portable-authentication.md#wallet-account-authentication).

The proposed public interaction is challenge preparation, proof submission and
certified delegation retrieval. Final method/type names and transport are fixed
with the implementation, not inherited from retired application Candid bindings.

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
