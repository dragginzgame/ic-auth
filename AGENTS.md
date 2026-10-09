# IC Auth Agent Instructions

Apply [DRAGGINZGAME.md](DRAGGINZGAME.md), adopted from Shared Tooling revision
`b2646cde9abbc8861857a4379c683a0c19eba43e` and recorded in
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
- Keep sibling repositories read-only. Record findings and consumer adoption
  requirements in the owning GitHub issues, using existing trackers where they
  match; do not request sibling edits as part of IC Auth work. An extraction must
  coordinate consumer adoption and retain one canonical owner when completed.
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
  mechanics. Native host tooling belongs in `apps/tooling/` and test infrastructure
  in unpublished `apps/qualification/`, outside both protocol libraries.
  Release identity and upload/retry policy stay here.
- Wallet login must not use SIWS/ic-siws, require a Solana transaction or depend
  on RPC. Use established signature primitives; do not implement cryptography.
- Identity mappings, controller authority, delegated session lifetime and
  recovery policy are service contracts. Do not import Canic's reinstall-only
  lifecycle policy into this identity service.

## Qualification tooling

- Native canister qualification uses published `ic-testkit` for typed calls,
  bounded startup and its upstream runtime re-export. Do not add a direct
  `pocket-ic` dependency. The locked Testkit package selects the setup/check CLI;
  Testkit owns server selection and admission outside the shared five-tool bundle.
  `make install-testkit-tools` explicitly prepares it, and
  `make testkit-tools-check` returns its admitted server path offline.
  Changing that pairing requires successful certification, upgrade and ingress
  qualification. Testkit and native host dependencies stay outside both libraries.

## Validation and delivery

- See [developer setup](docs/development.md) for the implemented command surface.
  Rust 1.88.0 is the qualified common package minimum; development formatting
  and Clippy stay on 1.99.0. `make check-msrv` checks isolated Cargo package
  payloads for every supported library feature on native/Wasm, then the internal
  applications separately. Setup explicitly prepares that compiler; checks never
  download one. See [minimum compiler qualification](docs/msrv.md).
  Focused Rust checks are `make test-types`, `make test-protocol`, `make test-signatures`,
  `make test-signature-store`, `make test-tokens`, `make test-sessions`,
  `make test-host-tooling`, `make test-qualification`, `make check-wasm` and `make clippy`; `make ci` is the
  explicit complete gate. `make test-release-runner` checks the adopted runner
  with substituted effects.
  Both libraries are published at `0.2.6` with contracts, encoding and optional
  IC signature/token/session machinery, including bounded signature preparation.
  The internal IC Testkit fixture exercises actual certification, composed roots,
  protected metadata upgrades and signed ingress with fresh session keys.
  It is not a wallet login provider or stable session backend.
  Stable library host adoption and wallet login remain
  unimplemented. Host stores must commit session/replay changes atomically and
  advance protected generation with authority changes.
- The private browser client is in `packages/client/`, with Rust-owned generated
  Candid data contracts and injected SDK identity/issuer/storage boundaries.
  See [client contracts](docs/browser-client.md). `make install-client-dependencies`
  explicitly prepares its selected Node/npm dependencies; `make test-client`
  checks lifecycle behavior and both Candid directions offline. Do not claim a
  production issuer adapter, npm publication or consumer adoption from transport
  fixtures. Its IndexedDB store is opt-in with a dedicated application-owned
  database, bounded records and atomic strict-commit CAS; do not erase unknown
  issuance intent or change its stored profile/format implicitly.
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
