# IC Auth

Independent application authentication protocol libraries for the Internet
Computer, with a wallet-authentication service planned.

The accepted scope includes IC signatures, application tokens, local application
sessions and wallet-backed IC identities. Canic will consume the libraries;
applications such as Toko will integrate through a documented client/service
contract. The libraries and reference service must not depend on either project.

## Status

The Rust workspace implements passive token/proof contracts, validated protocol
identifiers, canonical signed encoding, optional IC signature/application-token
verification and atomic session/replay admission extracted from Canic. Both
libraries are published on crates.io at `0.1.6`; see the
[signature](docs/signatures.md), [token](docs/tokens.md) and
[session](docs/sessions.md) contracts. Durable canister adoption remains pending.
Bounded [signature preparation and retrieval](docs/signature-preparation.md)
includes explicit host certification composition.
The working tree adds [IC Testkit qualification](docs/ic-testkit-qualification.md)
for real certification, upgrades and signed ingress; this internal fixture
does not implement a wallet login provider.
**Wallet login remains unimplemented.** Tokens and local sessions do not grant
application resource ownership. The native utility in `apps/tooling/` is unpublished and reuses
`ic-host-fs`/`ic-host-artifacts` for release and publication file operations.

Canic still runs its existing implementation. Its adapter adoption and removal of
superseded code require a separately authorized Canic change.

- [Architecture and ownership](docs/design/extraction.md)
- [Current handoff](docs/status/current.md)
- [Developer setup and commands](docs/development.md)
- [Canic source review and incorporation boundary](docs/design/canic-source-review.md)
- [Extraction and Canic adoption tracker](https://github.com/dragginzgame/canic/issues/491)
- [Agent instructions](AGENTS.md)

## Workspace

```text
Cargo.toml                     # Virtual workspace and dependency catalog
Cargo.lock                     # One selected Rust dependency graph
crates/
  ic-auth-protocol-types/      # Passive auth contracts and validated identifiers
  ic-auth/                     # Encoding and optional signature/token/session machinery
apps/
  tooling/                     # Unpublished native release/publication file utility
  qualification/               # Internal Wasm host and PocketIC/IC-agent tests
```

The types package is `ic-auth-protocol-types` (Rust import
`ic_auth_protocol_types`). The unrelated [`ic_auth_types`](https://docs.rs/crate/ic_auth_types/0.1.1)
package belongs to another IC-Auth project and does not implement our application
token/proof contracts. Crates.io treats hyphens and underscores as colliding
names, so changing only the punctuation cannot resolve that conflict.

Future service and client packages belong in `apps/wallet-auth/` and
`packages/client/`. They are not created as empty or accepting placeholders.

With the [declared tools](docs/development.md) prepared, run the focused checks:

```sh
make test-types
make test-protocol
make test-signatures
make test-signature-store
make test-tokens
make test-sessions
make test-qualification
make check-wasm
```

Planned wallet authentication targets Solana message signing without SIWS or
`ic-siws`. NFTs remain in their application's ledger. This service does not mint,
index, bridge or custody Solana assets.

Canic integration uses local library calls. It must not add a remote authentication
request to every protected application call.
