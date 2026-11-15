# Changelog

## [0.43.0] - Source checkpoint 2026-11-15

Package versions: `ic-auth` 0.43.0, protocol-types/Wire 0.42.0.
All packages remain `publish = false`.

### ✨ Added

- Add an opt-in bounded memory receiver with complete verified admission, exact
  retry, proof-free reads, known-login revocation and indexed pruning. Preserve
  replay quotas, device fences and original leases through renewal/replacement;
  retain failure charges without publishing authorization records. Both proof
  mechanisms share one canonical decode and the existing policy. Independent
  signed sequences and reference byte-size boundaries cover the operations.
  Stable storage, unknown-login denial and host boot/recovery remain open.
- Check physical session/replay capacity, issuer-qualified replay quotas and
  replacement byte accounting with bounded records and checked arithmetic.
  Projected totals remain separate from reservations or atomic publication.
- Dispatch credentials through one host-selected verifier profile. Submitted
  proof tags cannot choose authority; matching proofs preserve existing error
  and charging behavior through the Wire/domain qualification drivers.
- Assess login renewal and device replacement without reserving state. Preserve
  immutable browser bindings and expired/denied slot fences; higher generations
  need a different browser and complete proof verification before any commit.
  Renewals and replacements reuse the known-login pool at capacity.
- Share proof-free session assessment and exact-retry decisions over bounded
  retained facts. Preserve original leases, request conflicts, single-role scope
  checks and current authority/login/device fences. These pure helpers
  are also used by the optional memory receiver.
- Evaluate requested scopes against one protected role without combining grants
  or repairing signed order. Both proof mechanisms capture login/key/enrollment
  lease ceilings; later refresh cannot extend a previously calculated lease.
  These helpers establish no session, replay decision or storage transaction.

## [0.42.0] - Source checkpoint 2026-11-15

Pushed as [03d1a53](https://github.com/dragginzgame/ic-auth-new/commit/03d1a53).
Package versions: `ic-auth` 0.42.0, `ic-auth-protocol-types` 0.41.0 and
`ic-auth-wire` 0.41.0. Each is the next minor after its highest existing crates.io
version checked on 2026-11-15 (0.41.1, 0.40.1 and 0.40.1 respectively).
The earlier 0.1.0/0.2.0 tags are GitHub-only rewrite milestones. All packages
remain unpublished until the replacement is complete and publication is authorized.

### ✨ Added

- Connect Wire credentials to passive domain proof material through a consuming
  projection that moves opaque buffers and preserves signed order and distinct
  commitments. Add borrowed verifier views, five reference-projection cases and
  three real Wire-to-verifier host-driver cases. Endpoint wiring remains Canic's;
  transport gains no Auth/crypto dependency and Auth gains no Wire dependency.
- Compose complete certified proof verification from shared claims context,
  root authorization and issuer signatures. Admit aggregate proof material
  before decoding, preserve uncharged binding refusals and charge once before
  root/issuer work. Seven complete-proof cases and a real Ed25519/certified
  shared-budget case use independent OpenSSL and BLS fixtures. Successful
  evidence remains separate from session/replay/storage admission.
- Separate Ed25519 implementation from common context, budget and proof evidence;
  certified-only consumers select no Ed25519 verifier. Share grant-subset policy
  and root material accounting without duplicate certificate hashing.
- Add optional root-delegation verification using Platform ECDSA, protected
  enrollment and revocation floors, exact certificate/leaf binding, bounded
  Merkle inclusion and low-S refusal. Eight independent OpenSSL fixture cases
  and two immutable-evidence API cases cover the component, now composed by
  the complete certified credential verifier.
- Implement bounded root-delegation certificate and signing commitments, with
  shared grant encoding and six independently checked compatibility cases.
  Preserve certificate issue time's exclusion from the root leaf. These APIs
  encode structure only; the optional root verifier consumes their commitments.
- Add optional certified issuer-signature verification through Platform, with
  protected issuer/seed/network binding, bounded input and certificate freshness.
  Eight independent fixture cases cover real BLS certificates, both subnet-range
  layouts and adversarial refusals. This component cannot admit credentials;
  The complete verifier owns claims binding and aggregate proof-work bounds.
- Keep Ed25519 and certified feature graphs independent; fixture signing and
  certificate construction live in the separate qualification workspace.

### 🛠️ Tooling

- Adopt IC Wire 0.24.1 in qualification and remove 0.22.2 from its lock.
  Retain 296 independently measured reference resource thresholds alongside
  the 37 fixed frames; current code never supplies its own expected thresholds.
- Align pending crate versions with the existing registry names and add the
  local protocol-types dependency version needed for eventual packaging.

## [0.2.0] - 2026-11-15

GitHub-only source milestone for Wire contracts and Ed25519 verification.
All product crates are 0.2.0 in this tag and remain `publish = false`.
Registry-aligned versions begin with the following development increment.

### ✨ Added

- Implement all four Wire contracts, including nested certified/Ed25519 proof
  layouts and every reply variant, through published `ic-wire` 0.24.1. Preserve
  routing identities, opaque claims, unnormalized scope text and all four resource
  allowances. Transport supplies no verifier, session engine or callable endpoint.
- Generate browser codecs from Rust schemas using an optional native feature.
  Thirty-seven fixed frames agree across reference/current Rust and generated
  browser codecs, with optional TTLs, u64 extremes and detached buffer decoding.
- Add the independent `ic-auth-wire-conformance` qualification workspace package
  for reference/current codec comparison, including separate byte/allocation/node/
  depth boundaries and aggregate allocation refusal. The old supplier stays out
  of product dependencies. Add focused native/browser checks and extend protocol
  lint/docs/Wasm checks. All product package versions now start at 0.2.0.
- Add optional Ed25519 proof verification through published Platform 0.33.7,
  with protected caller/audience/key/grant/time checks, immutable proof results
  and retained receiver-wide work charging. Preserve preflight refusal order,
  strict signatures over exact claims digests, exclusive deadlines and no scope
  union across roles. Certified verification and session admission remain open.
- Retain and independently reproduce the reference OpenSSL acceptance vector.
  Add malformed-proof, context and failed-attempt tests plus a qualification-only
  legacy-scalar feature test whose upstream negative control accepts `S + L`
  while Auth refuses it without charging. Default codec consumers gain no
  verification dependency.

### ▶️ Try it

Run `make test-wire test-verification qualify-wire qualify-ed25519` after the
setup described in [developer tooling](docs/development/tooling.md).
`make qualify-wire-browser` additionally needs Node and the pinned reference
checkout. Certified verification, receiver/storage and browser lifecycle are
still unimplemented. Canic owns endpoint wiring; Auth installs no endpoints.

## [0.1.0] - 2026-11-15

Initial GitHub source milestone for the replacement protocol foundation.
This is not a complete authentication engine or a crates.io publication.

### ✨ Added

- Begin the replacement protocol boundary with validated scope labels, exact
  unsigned session-request hashes and bounded host TTL calculation. Fixed
  reference vectors preserve request identity, including absent versus explicit
  TTL; malformed inputs and deadline overflow refuse without normalization of
  labels. This does not implement credential verification or session admission.
- Add the bounded schema-2 claims codec with exact signed bytes and SHA-256,
  passive claims/roles and published Platform principal identity. Preserve the
  independent golden vector, strict signed-grant ordering and all count/byte
  limits; decoding remains separate from proof and authority verification.

### 🛠️ Tooling

- Establish the Rust 1.100.0 product workspace and product/qualification
  directories, with authenticated Toko Tooling 0.44.4, shared dependency commands
  and the standalone LOC reporter. The first real packages now inherit the
  tooling selection, produce a lockfile and have focused native/lint/docs/Wasm
  checks. Both packages remain unpublished.

### ▶️ Try it

From the repository root:

```sh
make install-tools
make tools-check
make toko-tooling-check
make deps-list
make test-request
make test-claims
```

Setup may download pinned tools. Prepare locked crate sources with
`cargo fetch --locked` on a fresh checkout before running offline checks.
The tooling check verifies Cargo selection and installed bytes; the request
target runs seven contract tests and one compile-fail doctest; the claims target
adds ten integration cases. Use `lint-protocol`, `doc-protocol` and
`check-protocol-wasm` for the now-expanded protocol boundary.
