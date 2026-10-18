# IC Auth

Independent application authentication protocol libraries for the Internet
Computer, with a wallet-authentication service planned.

The implementation owns IC signature/proof contracts, portable credential
admission and caller-bound local sessions. Canic consumes these libraries through
host adapters; applications such as Toko retain account and resource policy.
Wallet-backed identities and their service are planned, not current endpoints.
No Auth library or future service may depend on either project.

## Status

The latest local release is **0.9.1**. It implements portable certified and enrolled
Ed25519 verification, one local session engine, replay/revocation bookkeeping and
explicit protected authority/work budgets. See the [session contract](docs/sessions.md)
and [current handoff](docs/status/current.md) for implementation and validation.

The existing [signature](docs/signatures.md), [delegation](docs/delegation.md),
[signature preparation](docs/signature-preparation.md),
[certificate issuance](docs/certificate-issuance.md) and
[chain-key](docs/chain-key-signatures.md) helpers remain reusable primitives.
Application resource permissions and IC ingress delegation remain separate.

The private [browser client](docs/browser-client.md) supplies one portable
admission coordinator, a durable logout journal, guarded SDK identity and HTTP
logout transport. Its Rust-owned IC Wire contracts are in `crates/ic-auth-wire/`.
The former token prepare/retrieve client and stores are retired: the maintainer
confirmed no deployed old records or unresolved old issuance. New portable
journals still preserve uncertainty. The separate Rust delegated-token API is
also retired after explicit confirmation of no deployed users or unresolved issuance. Production issuer/enrollment and consumer
integration remain pending; reference qualification is separate from adoption.

**Production issuer/enrollment lifecycle, wallet login and the durable session
adapter remain unimplemented.** The reference receiver uses volatile state.
The [IC Testkit fixture](docs/ic-testkit-qualification.md) exercises actual
certification, portable sessions, upgrades and signed ingress; it is not a
production wallet service or evidence of completed consumer adoption.
The unpublished native utility in `apps/tooling/` delegates file operations to
IC Host and package-lock subset admission to IC Build; release identity and
publication/retry policy remain here.

Canic is adopting the shared libraries; its complete runtime verifier and durable
host integration remain consumer work. The
[Canic adoption contract](docs/canic-adoption.md) maps the published libraries
to its feature, authority, storage and certification boundaries; implementation
and acceptance belong in [Canic #491](https://github.com/dragginzgame/canic/issues/491).

- [Architecture and ownership](docs/design/extraction.md)
- [Current handoff](docs/status/current.md)
- [Developer setup and commands](docs/development.md)
- [Portable browser client and consumer boundary](docs/browser-client.md)
- [Minimum Rust compiler and package qualification](docs/msrv.md)
- [Shared maintenance task catalog](tasks/README.md)
- [Canic source review and incorporation boundary](docs/design/canic-source-review.md)
- [Extraction and Canic adoption tracker](https://github.com/dragginzgame/canic/issues/491)
- [Agent instructions](AGENTS.md)

## Application authentication

The accepted application flow is **portable proof admission → caller-bound local session
→ scoped session checks**. Each receiving canister admits the credential under its
protected fleet, role and scope policy. Ordinary application calls use the
authenticated ingress caller and the local session; they do not carry a token.
Portable credential verification and atomic admission share one session engine.
Application resource permissions and IC ingress signatures remain distinct.

This is the coordinated portable consumer contract, not a claim that Canic/Toko have
completed the cut. Their guards, Wire endpoints and frontend calls must change
together under the [adoption contract](docs/canic-adoption.md).

## Workspace

```text
Cargo.toml                     # Virtual workspace and dependency catalog
Cargo.lock                     # One selected Rust dependency graph
crates/
  ic-auth-protocol-types/      # Passive auth contracts and validated identifiers
  ic-auth-wire/                # Bounded transport and generated browser codecs
  ic-auth/                     # Encoding, proof verification and session machinery
apps/
  tooling/                     # Unpublished native release/publication file utility
  qualification/               # Internal Wasm host and IC Testkit/IC-agent tests
packages/
  client/                      # Private browser lifecycle client and generated contracts
```

The types package is `ic-auth-protocol-types` (Rust import
`ic_auth_protocol_types`). The unrelated [`ic_auth_types`](https://docs.rs/crate/ic_auth_types/0.1.1)
package belongs to another IC-Auth project and does not implement our application
token/proof contracts. Crates.io treats hyphens and underscores as colliding
names, so changing only the punctuation cannot resolve that conflict.

Released 0.8.0 removed Candid traits from the public domain types; `Principal`
now comes through feature-free IC Platform, retaining the upstream
`ic_principal::Principal` identity. Released 0.9.0 replaced application-facing
Candid with published IC Wire's bounded codec and generated TypeScript through
an Auth-owned transport adapter. Optional upstream certified-signature capabilities
still use Candid transitively. Private fixture controls, diagnostics, init and
upgrade checkpoints now use bounded Wire contracts too; the old unpublished
Candid checkpoint format is retired. Platform 0.1.12 supplies Principal, explicit certified-signature/tree/verifier
capabilities, raw admission/counters and native Agent identities. The reference
host now uses its `cdk` runtime and entrypoint boundary, with no direct CDK
declaration. See the [platform boundary](docs/development.md#ic-platform-boundary)
for runtime ownership and the remaining transitive Candid/CDK dependencies.
See the [consumer contract](docs/canic-adoption.md#wire-transport). The incoming
Wire 0.4 selection requires the next Auth minor release because Wire types are
publicly exposed; it does not change the already released 0.9 dependency contract.

The future wallet service belongs in `apps/wallet-auth/`. No accepting wallet
endpoint is created before its complete proof verification and service contracts.

With the [declared tools](docs/development.md) prepared, run the focused checks:

```sh
make test-types
make test-protocol
make test-chain-key-signatures
make test-signatures
make test-signature-store
make test-certified-proofs
make test-sessions
make test-qualification
make check-wasm
```

Planned wallet authentication uses custom Solana/Ethereum login challenges without
SIWS or SIWE, with established signature primitives and wallet message signing.
Wallet adapters remain unimplemented; see the [wallet design](docs/design/0.6-portable-authentication.md#wallet-account-authentication).
NFTs remain in their application's ledger. This service does not mint, index,
bridge or custody wallet assets.

Canic integration uses local library calls. It must not add a remote authentication
request to every protected application call.

## Ownership and dependencies

| Owner | Capability and integration boundary |
| --- | --- |
| IC Auth | Checked protocol types, signed byte contracts, proof/certificate semantics, session/replay/revocation transitions, bounded Auth transport layouts and the private admission/logout client. |
| IC Wire | Canonical transport codec, bounded schema/derive and TypeScript generator. Auth projects its own domain types and generates client artifacts through this dependency. |
| IC Host / IC Build | Native file/process operations and generic Cargo lock admission. Auth supplies paths, byte budgets, package isolation and release policy. |
| IC Testkit | Native canister transport, server selection and lifecycle support. Auth retains its authentication-specific integration assertions. |
| IC Memory / CDK | Memory-backed checkpoint and canister runtime support in the unpublished qualification fixture. Neither enters the published Auth library graph. |
| Canic | Fleet authority, issuer enrollment/renewal, endpoint guards, caller/clock, certification publication and atomic production storage. |
| Toko | User/account linking, project permissions, frontend custody and application assets. |

Signed claims, bounded transport and checked stored records serve different trust
boundaries; their independent validation is intentional. Production wallet/issuer
and durable host acceptance remain explicit unfinished work, not alternative
authentication flows. The [current handoff](docs/status/current.md) separates
current evidence from [historical extraction notes](docs/status/history-through-0.9.1.md).
