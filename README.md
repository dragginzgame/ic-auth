# IC Auth

Independent application authentication protocol libraries for the Internet
Computer, with a wallet-authentication service planned.

The accepted scope includes IC signatures, application tokens, local application
sessions and wallet-backed IC identities. Canic will consume the libraries;
applications such as Toko will integrate through a documented client/service
contract. The libraries and reference service must not depend on either project.

## Status

The working tree prepares the **0.6 library hard cut**; package manifests remain
at `0.5.10` until release preparation. It implements portable certified and enrolled
Ed25519 verification, one local session engine, replay/revocation bookkeeping and
explicit protected authority/work budgets. See the [session contract](docs/sessions.md)
and [current handoff](docs/status/current.md) for implementation and validation.

The existing [signature](docs/signatures.md), [token](docs/tokens.md),
[signature preparation](docs/signature-preparation.md),
[certificate issuance](docs/certificate-issuance.md) and
[chain-key](docs/chain-key-signatures.md) helpers remain reusable primitives.
Application resource permissions and IC ingress delegation remain separate.

The private [browser client](docs/browser-client.md) includes the retained token
lifecycle plus a separate portable logout journal, guarded SDK identity and HTTP
logout transport. It is unpublished; existing unknown issuance intents are not
migrated or erased. Production portable login/admission, key/clock integration
and successful deployed logout qualification remain pending.

**Production issuer/enrollment lifecycle, wallet login and the durable session
adapter remain unimplemented.** The reference receiver uses volatile state.
The [IC Testkit fixture](docs/ic-testkit-qualification.md) exercises actual
certification, portable sessions, upgrades and signed ingress; it is not a
production wallet service or evidence of completed consumer adoption.
The unpublished native utility in `apps/tooling/` uses `ic-host-fs` for release
and publication file operations.

Canic is adopting the shared libraries; its complete runtime verifier and durable
host integration remain consumer work. The
[Canic adoption contract](docs/canic-adoption.md) maps the published libraries
to its feature, authority, storage and certification boundaries; implementation
and acceptance belong in [Canic #491](https://github.com/dragginzgame/canic/issues/491).

- [Architecture and ownership](docs/design/extraction.md)
- [Current handoff](docs/status/current.md)
- [Developer setup and commands](docs/development.md)
- [Browser token client and consumer boundary](docs/browser-client.md)
- [Minimum Rust compiler and package qualification](docs/msrv.md)
- [Shared maintenance task catalog](tasks/README.md)
- [Canic source review and incorporation boundary](docs/design/canic-source-review.md)
- [Extraction and Canic adoption tracker](https://github.com/dragginzgame/canic/issues/491)
- [Agent instructions](AGENTS.md)

## Application authentication for the 0.6 cut

The accepted application flow is **portable proof admission → caller-bound local session
→ scoped session checks**. Each receiving canister admits the credential under its
protected fleet, role and scope policy. Ordinary application calls use the
authenticated ingress caller and the local session; they do not carry a token.
Token preparation and proof verification remain the admission machinery.
Application resource permissions and IC ingress signatures remain distinct.

This is the coordinated 0.6 consumer contract, not a claim that Canic/Toko have
completed the cut. Their guards, Candid interfaces and frontend calls must change
together under the [adoption contract](docs/canic-adoption.md).

## Workspace

```text
Cargo.toml                     # Virtual workspace and dependency catalog
Cargo.lock                     # One selected Rust dependency graph
crates/
  ic-auth-protocol-types/      # Passive auth contracts and validated identifiers
  ic-auth/                     # Encoding and optional signature/token/session machinery
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

The future wallet service belongs in `apps/wallet-auth/`. No accepting wallet
endpoint is created before its complete proof verification and service contracts.

With the [declared tools](docs/development.md) prepared, run the focused checks:

```sh
make test-types
make test-protocol
make test-chain-key-signatures
make test-signatures
make test-signature-store
make test-tokens
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
