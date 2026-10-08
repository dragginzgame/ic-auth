# IC Auth

Independent application authentication protocol libraries for the Internet
Computer, with a wallet-authentication service planned.

The accepted scope includes IC signatures, application tokens, local application
sessions and wallet-backed IC identities. Canic will consume the libraries;
applications such as Toko will integrate through a documented client/service
contract. The libraries and reference service must not depend on either project.

## Status

The Rust workspace implements passive token/proof contracts, validated protocol
identifiers and canonical signed encoding extracted from Canic. **It does not yet
verify tokens, admit sessions or provide wallet login.** Hashing a token is not
authentication. Both packages are unpublished; their manifests and the guarded
publication command now support crates.io delivery of these implemented APIs.

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
  ic-auth-types/               # Passive auth contracts and validated identifiers
  ic-auth/                     # Canonical token, certificate and proof encoding
```

Future service and client packages belong in `apps/wallet-auth/` and
`packages/client/`. They are not created as empty or accepting placeholders.

With the [declared tools](docs/development.md) prepared, run the focused checks:

```sh
make test-types
make test-protocol
make check-wasm
```

Planned wallet authentication targets Solana message signing without SIWS or
`ic-siws`. NFTs remain in their application's ledger. This service does not mint,
index, bridge or custody Solana assets.

Canic integration uses local library calls. It must not add a remote authentication
request to every protected application call.
