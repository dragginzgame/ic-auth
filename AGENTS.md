# IC Auth Agent Instructions

Apply the installed [baseline](DRAGGINZGAME.md), adopted from Shared Tooling
0.29.4 at `628ccc75d813bfd9bae777bfbaef55838d25b1b2`, and its task-specific
references. The [snapshot manifest](.shared-tooling.snapshot) records the exact
policy/tooling selection. Read [the current handoff](docs/status/current.md)
before implementation. Product-specific safeguards and scoped permissions below
remain binding.

## Ownership

- IC Auth owns reusable proof/credential and session machinery, bounded Auth
  transport contracts and its private admission/logout client. A standalone
  wallet authentication service is planned, not implemented. Neither the library
  graph nor that future service may depend on Canic or Toko.
- Keep authentication, IC ingress identity delegation, application authorization
  and application resource ownership distinct. A wallet proof is not a resource
  grant. An application token is not an IC ingress delegation.
- Canic owns fleet topology, member/role authority, issuer approval and renewal
  orchestration, endpoint guards and deployment configuration. Toko owns its user
  records, account-linking policy and NFT ledger semantics.
- Keep one canonical owner during extraction; consumer adoption belongs to its
  own repository and qualification. A dependency blocker grants no sibling PR
  or deployment authority.
- GitHub issues are the work tracker. IC Auth-specific work belongs in
  [IC Auth issues](https://github.com/dragginzgame/ic-auth/issues).
  [Canic #491](https://github.com/dragginzgame/canic/issues/491) remains the existing
  extraction and Canic adoption tracker; design and handoff files describe
  contracts and evidence, not a parallel task queue.

## Implementation boundaries

- Follow [the accepted design](docs/design/extraction.md). Use a virtual root,
  one lockfile, root-owned dependency selections and reusable packages in `crates/`.
  Reserve `apps/wallet-auth/` for the future service; no deployed service exists here.
- Keep protocol/type dependencies independent of CDK and storage. Make platform
  signing and wallet functionality explicit capabilities. Canic consumers must
  not pull Solana dependencies into their selected graph.
- Pure verification takes time and protected authority as explicit inputs. Host
  adapters obtain those values from authenticated runtime/configuration state.
  Never infer trusted authority from client-supplied fields.
- The host owns the clock, stable-memory allocation, transaction boundary and
  certification-root publication. Library operations must not secretly write a
  global certification root or allocate conflicting stable-memory regions.
- The requested Storage consolidation moves allocation governance and
  checked leases from retired `ic-memory` into `ic-storage`. Use Storage-owned
  keys, namespace grants, backing and collection APIs; preserve the older Shared
  snapshot's governance invariants under that new owner. Only the private host
  selects Storage; its direct Wasm Runtime dependency resolves to the host's one
  executor/guard. Legacy backing is refused without reset or a compatibility reader.
- Use published `ic-host-*` for applicable local file, artifact and process
  mechanics and `ic-build` for generic Cargo lock admission. Native host tooling
  belongs in `apps/tooling/` and test infrastructure
  in unpublished `apps/qualification/`, outside the public library graphs.
  Release identity and upload/retry policy stay here.
- Wallet login must not use SIWS/ic-siws, require a Solana transaction or depend
  on RPC. Use established signature primitives; do not implement cryptography.
- Identity mappings, controller authority, delegated session lifetime and
  recovery policy are service contracts. Do not import Canic's reinstall-only
  lifecycle policy into this identity service.

## Platform boundary

Use published `ic-platform-types` directly for the protocol's canonical
Principal. Select Platform's `agent` capability for native ingress identities. Application protocols,
generated browser contracts and private qualification controls use IC Wire.
The fixture exposes only Wire methods, with no Candid projection or direct
declaration. Its old
Candid checkpoint is explicitly retired; production storage is unaffected.
Auth selects canonical Principal and parse errors from `ic-platform-types`.
The engine explicitly selects Types' `principal-self-authenticating` for
browser-key hashing; passive protocol and Wire packages select neither external
Principal nor SHA-2. Platform is optional and selected only by crypto capabilities.
Convert explicitly through `principal::ExternalPrincipal` at native
Agent/Testkit boundaries; keep `principal-external` selected there.
Platform canister-signature keys and certificate types use owned identities.
Keep application/runtime identities owned and preserve original bytes. It supplies
maintained canister-signature, certificate-tree and BLS code plus upstream
ECDSA and Ed25519 capabilities. All K256 and Ed25519 API imports use
Platform; Auth retains prehash/high-S policy, explicit normalization,
canonical-scalar admission and strict verification. The unpublished
feature-unification harness alone has a direct Dalek dev dependency
to enable permissive parsing adversarially, never in a deployed graph. Forward only the features required by each Auth
capability; BLS fixture signing is a dev dependency. Preserve imported crypto provenance; do not implement replacement cryptography.
Runtime-only consumers select the published independent Runtime owner directly;
public Auth libraries and passive contracts exclude it. The mixed crypto/runtime
host selects Runtime's `cdk` capability directly for runtime calls,
certificate acquisition/publication and entrypoint generation (Platform #21).
Keep explicit Wire codecs and the upstream controller/runtime semantics; the
retired no-op `hidden` macro option is not accepted.
Platform owns the maintained non-cryptographic runtime adapters and generator;
its maintained executor is the sole runtime executor. Reject the separate
registry executor and retain one Platform generator identity. Its `cdk` feature is Candid-free.
Platform's runtime singleton guard is selected only with the executor. Require
one coordinated guard in the host and exclude it from public library graphs.
All public library capability graphs and the reference Wasm refuse Candid.
Native Agent/Testkit still bring it transitively. Platform's raw APIs support
existing admission/profiling; only the actual host selects IC0/executor.
See [the handoff](docs/status/current.md), Platform #16/#17/#18 adoption evidence
and `ci/cargo-dependency-boundaries.json` for direct-dependency ownership.

## Qualification tooling

- Native canister qualification uses published `ic-testkit` for typed Wire calls,
  deliberate raw-call evidence and bounded startup. Import simulator clients and
  DTOs from `ic_testkit::pocket_ic` in the native runner only. Testkit owns that
  maintained client and its lifecycle helpers; do not add a separate Platform
  client or registry `pocket-ic` dependency. The locked Testkit package selects the setup/check CLI;
  Testkit owns server selection and admission outside the shared five-tool bundle.
  `make install-testkit-tools` explicitly prepares it, and
  `make testkit-tools-check` returns its admitted server path offline.
  Qualification also requires the explicitly prepared browser client dependencies
  for real SDK logout transport; see [client setup](docs/development.md#browser-client-setup).
  Changing that pairing requires successful certification, upgrade and ingress
  qualification. Testkit and native host dependencies stay outside all public libraries.

## Validation and delivery

- See [developer setup](docs/development.md) for the implemented command surface.
  Rust 1.99.1 is both the common package minimum and pinned development/release
  compiler. `make check-msrv` checks isolated Cargo package payloads for every
  supported library feature on native/Wasm; native and canister Wasm Clippy cover
  the internal applications using the same compiler.
  Setup explicitly prepares the declared compiler; checks never
  download one. See [minimum compiler qualification](docs/msrv.md).
  Choose the package, test and capability affected by the completed batch;
  [developer setup](docs/development.md#validation) distinguishes narrow commands
  from aggregate suites. `make ci` remains the complete repository gate. Releases
  select the exhaustive source-bound Protocol/Engine plans described in
  [Auth release validation](docs/auth-release-validation.md), with full coverage
  for shared or uncertain changes and retained per-stage recovery evidence.
  Local validation cadence follows
  [the shared baseline](DRAGGINZGAME.md#validation-and-evidence). The validation
  runner seals declared Rust/frontend preparation inputs
  between stages and refuses drift without fetching or restarting implicitly.
  `make test-formatting-hooks` checks actual staged preservation and is part of
  the complete gate. Auth's release/recovery and evidence caller fixtures remain
  in CI; generic Shared runner/archive matrices are opt-in for changed-contract
  adoption or a named missing consumer proof.
  Current publication and implementation status belongs in
  [the handoff](docs/status/current.md), not this instruction file. The internal
  IC Testkit fixture qualifies certification, composed roots, protected metadata
  upgrades, certified-token admission and signed ingress; it does not establish
  production wallet-login or stable session-backend adoption. Host stores must
  commit session/replay changes atomically and advance protected generation with
  authority changes.
- The private browser client is in `packages/client/`, with Rust-owned generated
  IC Wire contracts and injected SDK identity/issuer/storage boundaries.
  See [client contracts](docs/browser-client.md). `make install-client-dependencies`
  explicitly prepares its selected Node/npm dependencies; `make test-client`
  checks lifecycle behavior and Rust/TypeScript Wire interoperability offline. Do not claim a
  production issuer adapter, npm publication or consumer adoption from transport
  fixtures. Its logout journal uses a dedicated application-owned
  database, bounded records and strict transactions; do not erase unknown
  issuance or change its stored profile implicitly. The maintainer confirmed no
  deployed old token-store records: that former client/format is retired in 0.9.
- A locked IC Wire update also selects the browser runtime generator. In the
  same adoption batch, run `make generate-client-contracts`, review all five
  generated files and run `make test-client`, even when Auth DTOs are unchanged.
  Validation compares generated source and never repairs it automatically.
- Fresh release preflight prepares the lock-selected IC Build editor after
  source/notes admission. The adapter selects its absolute admitted path from
  the validated source lockfile; no shell export is required. Manual setup is
  `make install-release-tools`. Recovery and ordinary checks only admit the saved
  selection offline; they never install or rebuild from partially prepared
  metadata. Auth retains an 8 MiB input limit and recovery custody.
- Protocol (`ic-auth-protocol-types`, `ic-auth-wire`) and Engine (`ic-auth` plus
  private qualification/tooling) release independently. All member versions are
  explicit; `ci/cargo-release-groups.json` owns group membership and ledgers.
  Select `RELEASE_GROUP=protocol|engine` explicitly for release, resume and publication.
  `make publish-dry-run RELEASE_GROUP=…` checks the selected group's builds using
  live registry metadata without uploading; `make check-package-builds` verifies all
  three current local archives together and remains in the complete gate. It permits
  prepublication Protocol identity changes without selecting an older registry peer.
  `make test-release-tools` uses local bare remotes and mocked
  upload/registry transport. Release targets share the group-plan adapter and
  existing delivery runner; `make publish` requires a clean, exactly tagged and
  pushed release. Running those effectful commands still requires explicit
  release/publication authority.

- For design/governance work, check local links, ownership consistency, snapshot
  integrity and the diff. Do not run Rust or portable script suites for prose.
- Implementation batches add meaningful focused tests with their package. Cover
  rejection cases, replay/atomicity, authority invalidation and signed-byte
  contracts; use PocketIC for real canister lifecycle and ingress delegation.
- Check for active builds before compilation or source mutation. Keep build
  output in this repository and preserve other work.
- Make routine changes directly in the [normal checkout](docs/development.md#normal-checkout-workflow).
  Use topic branches and PRs when requested or required by the repository workflow;
  a coding or dependency-adoption request does not itself require a PR.
  Do not create contribution directories, clones or worktrees for ordinary work
  or release delivery. Coordinate active sessions before switching branches or
  changing release inputs; never disable input-drift admission or discard recovery
  state to retry. The redundant capacity/0.33 checkouts are retired; their
  integrated history has local `archive/` refs and required evidence is under
  `target/checkout-retirement/`. Keep build trees and bulk checkout archives out
  of `.git`; explicit `cargo clean` still owns the entire workspace `target/`.
- No package commands, runtime capabilities or qualification may be advertised
  before they exist and are checked. Do not create placeholder auth endpoints
  that accept requests without complete verification.
- Commit, PR, publication and deployment authority follows the shared baseline.
  Setup or inspection alone does not authorize publication or deployment.
