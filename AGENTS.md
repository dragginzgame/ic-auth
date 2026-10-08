# IC Auth Agent Instructions

Apply [DRAGGINZGAME.md](DRAGGINZGAME.md), adopted from Shared Tooling revision
`75a8a60f49cec11d3f6aecab5c977029c42cc549` and recorded in
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
- Use published `ic-host-*` for applicable local file, artifact and process
  mechanics. Keep these native host dependencies in `apps/tooling/`, outside
  both protocol libraries. Release identity and upload/retry policy stay here.
- Wallet login must not use SIWS/ic-siws, require a Solana transaction or depend
  on RPC. Use established signature primitives; do not implement cryptography.
- Identity mappings, controller authority, delegated session lifetime and
  recovery policy are service contracts. Do not import Canic's reinstall-only
  lifecycle policy into this identity service.

## Validation and delivery

- See [developer setup](docs/development.md) for the implemented command surface.
  Focused Rust checks are `make test-types`, `make test-protocol`, `make test-signatures`,
  `make test-tokens`, `make test-sessions`,
  `make test-host-tooling`, `make check-wasm` and `make clippy`; `make ci` is the
  explicit complete gate. `make test-release-runner` checks the adopted runner
  with substituted effects.
  Both libraries are published at `0.1.4` with contracts, encoding and optional
  IC signature/token verification. The working tree adds session/replay admission
  and a bounded volatile backend. Stable host adoption and wallet login remain
  unimplemented. Host stores must commit session/replay changes atomically and
  advance protected generation with authority changes.
- `make publish-dry-run` checks both package builds using live registry metadata
  without uploading. `make test-release-tools` uses local bare remotes and mocked
  upload/registry transport. The three release targets share the complete gate
  and direct-delivery runner; `make publish` requires a clean, exactly tagged and
  pushed release. Running those effectful commands still requires explicit
  release/publication authority.

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
