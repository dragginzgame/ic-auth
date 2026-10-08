# IC Auth Agent Instructions

Apply [DRAGGINZGAME.md](DRAGGINZGAME.md), adopted from Shared Tooling revision
`2687f26317952c43c685f7f799ed09288dc10a67` and recorded in
[.shared-tooling.snapshot](.shared-tooling.snapshot). Read
[the current handoff](docs/status/current.md) before implementation.

## Ownership

- IC Auth owns reusable proof, token and session machinery and its standalone
  wallet authentication service. Neither its library graph nor reference service
  may depend on Canic or Toko.
- Keep authentication, IC ingress identity delegation, application authorization
  and application resource ownership distinct. A wallet proof is not a resource
  grant. An application token is not an IC ingress delegation.
- Canic owns fleet topology, member/role authority, issuer approval and renewal
  orchestration, endpoint guards and deployment configuration. Toko owns its user
  records, account-linking policy and NFT ledger semantics.
- Do not edit sibling repositories without explicit scope authorization. An
  extraction must coordinate consumer adoption and retain one canonical owner
  when that adoption is completed.
- GitHub issues are the work tracker. IC Auth-specific work belongs in
  [IC Auth issues](https://github.com/dragginzgame/ic-auth/issues).
  [Canic #491](https://github.com/dragginzgame/canic/issues/491) remains the existing
  extraction and Canic adoption tracker; design and handoff files describe
  contracts and evidence, not a parallel task queue.

## Implementation boundaries

- Follow [the accepted design](docs/design/extraction.md). Use a virtual root,
  one lockfile, root-owned dependency selections, reusable packages in `crates/`
  and the deployed service in `apps/wallet-auth/`.
- Keep protocol/type dependencies independent of CDK and storage. Make platform
  signing and wallet functionality explicit capabilities. Canic consumers must
  not pull Solana dependencies into their selected graph.
- Pure verification takes time and protected authority as explicit inputs. Host
  adapters obtain those values from authenticated runtime/configuration state.
  Never infer trusted authority from client-supplied fields.
- The host owns the clock, stable-memory allocation, transaction boundary and
  certification-root publication. Library operations must not secretly write a
  global certification root or allocate conflicting stable-memory regions.
- Wallet login must not use SIWS/ic-siws, require a Solana transaction or depend
  on RPC. Use established signature primitives; do not implement cryptography.
- Identity mappings, controller authority, delegated session lifetime and
  recovery policy are service contracts. Do not import Canic's reinstall-only
  lifecycle policy into this identity service.

## Validation and delivery

- See [developer setup](docs/development.md) for the implemented command surface.
  Focused Rust checks are `make test-types`, `make test-protocol`,
  `make check-wasm` and `make clippy`; `make ci` is the explicit complete gate.
  Both crates remain unpublished extraction candidates. Canonical hashing is
  not proof verification or authenticated admission.

- For design/governance work, check local links, ownership consistency, snapshot
  integrity and the diff. Do not run Rust or portable script suites for prose.
- Implementation batches add meaningful focused tests with their package. Cover
  rejection cases, replay/atomicity, authority invalidation and signed-byte
  contracts; use PocketIC for real canister lifecycle and ingress delegation.
- Check for active builds before compilation or source mutation. Keep build
  output in this repository and preserve other work.
- No package commands, runtime capabilities or qualification may be advertised
  before they exist and are checked. Do not create placeholder auth endpoints
  that accept requests without complete verification.
- Commit, PR, publication and deployment authority follows the shared baseline.
  This bootstrap does not authorize a GitHub repository, release or deployment.
