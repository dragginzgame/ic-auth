# Changelog

## [0.46.2] - Unreleased

### ✨ Added

- Start the private browser client with one bounded signing-attempt owner around
  the published Platform SDK identity. Capture principal/destination, recheck the
  journal owner before and after signing/completion, and close the capability on
  every exit. No transport retry or application replay is introduced. Ten real SDK
  cases cover suspended signing, mutation, cancellation, timeout and read-state
  polling through `make test-client`.
- Add a strict IndexedDB login journal with bounded destination inventory and
  atomic local logout. Preserve the existing private v1 profile, pending sequence
  and unknown issuance, including when its sequence is missing. Keep mutable
  records and transactions private. Eight native Chromium cases cover two tabs,
  suspended SDK signing, aborted writes, strict durability, reopening and corrupt
  profiles through `make test-client-journal`.
- Extend the same journal with durable issuance reservation, lookup-only recovery,
  exact completion and sequential renewal. Preserve missing-sequence uncertainty,
  reject conflicting/stale replies and retain evidence on corruption or overflow.
  Adopt the Rust-generated credential codec and supplier runtime; add generation
  drift checks and independent frame checks. Seven more native browser cases
  qualify concurrency, recovery and atomic evidence writes.
- Connect journal decisions to an injected issuer adapter through one bounded SDK
  attempt. Bind signing to the original issuer, browser login and current sequence;
  select account/browser roles, retain lost-response uncertainty and commit exact
  evidence before returning it. Eight native browser cases cover adapter dispatch,
  cancellation/timeout, late replies/signatures and failed completion recovery.
  Add focused journal/issuance/transport selections.
- Add explicit receiver admission through the Rust-generated EstablishSession
  contract and an injected authenticated-update adapter. Commit destination
  inventory before dispatch; bind browser signing to current proof expiry/sequence
  and reject late or mismatched receipts. Preserve every refusal without automatic
  retry. Seven native browser cases and two independent codec cases qualify this
  operation; add the focused admission group. Actual HTTP authentication, readiness
  coordination and remote cleanup remain unimplemented.

### 🧪 Testing

- Add installed Wasm issuer qualification for transaction rollback, real certified
  signatures, caller/time/revocation refusal and terminal closure after volatile
  upgrade loss. `make qualify-host` uses an explicitly provided PocketIC server;
  `make lint-host` checks the native harness and Wasm fixture. Product code and
  package versions are unchanged. Canic integration remains separate.
- Extend the same fixture to receiver admission/logout and authorized application
  transactions. Check ordinary bad-proof charging, proof-expired exact retry,
  retained revocation and closed upgrade loss with an intact-receiver control.
  Reuse the independent Node/OpenSSL packer with simulator-scale timestamps;
  production Canic integration and browser lifecycle remain unqualified.
- Replace injected receiver enrollment metadata with an installation-bound IC call.
  Qualify reply binding, freshness/regression refusal, single-key terminal revocation
  and atomic callback publication with a separate installed authority. Complete
  production registry validation and challenge/refresh scheduling remain separate.
- Keep all installed fixtures on one simulated subnet. Late/overlapping-reply
  integration checks remain outstanding for the supported Canic flow; retain the
  native semantic checks without cross-subnet timing scaffolding.

## [0.46.1] - Source checkpoint 2026-11-16

Source checkpoint: `ic-auth` 0.46.1. Protocol-types/Wire remain 0.42.0;
all packages remain `publish = false`.

### ✨ Added

- Add optional `issuer-memory` Ed25519/certified issuance and browser-authenticated renewal with one private operation
  phase enum and one retained login binding per account/login identity. Reserve
  completed claims/signature bytes before entropy, approval or signing, and retain
  exact callbacks, original deadlines and full-start fresh-intent budget charges.
- Record external dispatch before returning signing input. Lost responses remain
  lookup-only; authority changes, revocation and epoch changes refuse late
  completion. Completed evidence remains recoverable without new signing authority.
- Preserve browser binding, sequential renewal watermarks and the original login
  window. Read-only preflight reserves nothing; preparation rechecks after awaits.
  One lookup index maps retained renewal sequences to the shared operations.
  Missing evidence never recreates a consumed sequence, and exact results remain
  retrievable after expiry, revocation or restart without reissuing.
- Add indexed issuer pruning, capped at 32 operation/login removals per call.
  Capture positive reconciliation retention at reservation, keep referenced logins
  and renewal watermarks, and release exact result-byte reservations only on removal.
  Close consumed-ID epochs independently of restart fences so unrelated live
  callbacks survive maintenance. Reclaimed slots receive higher generations.
- Complete certified issuance through synchronous host leaf/root publication.
  Retain claims in the same operation model; retries never republish a leaf.
  Host publication failure closes the issuer and requires enclosing rollback.
  Installed-host recovery and stable custody remain unqualified.

### 🧪 Testing

- Add independent issuer lifecycle, failure-containment and capacity coverage,
  with Node/OpenSSL signed bytes and existing renewal operation identities.
  Real Platform BLS certificates verify actual publication inputs and fresh
  witnesses with unchanged claims. Wire and cryptographic schemes are preserved.

## [0.46.0] - Source checkpoint 2026-11-16

Source checkpoint: `ic-auth` 0.46.0. Protocol-types/Wire remain 0.42.0;
all packages remain `publish = false`.

### ✨ Added

- Make receiver policy snapshots, decisions and assessment functions private.
  Hosts use `receiver_memory` for operation inputs, configuration, errors and
  inspection/usage results. Move all nineteen policy tests beside their private
  owner; replace two synthetic proof-to-facts drivers with mapped real receiver
  coverage, including certified replacement of a denied slot. Compile-fail cases
  guard the narrower API. This breaks the unpublished Rust policy API only.
- Extend volatile receiver qualification to two certified issuers, two devices
  on one account and two independent receivers, using seven independently packed
  claims and a second OpenSSL-signed root authorization. Exercise account quota
  isolation, failed root/issuer proofs without displacement, replacement at full
  login capacity, proof-expired retry, renewal and receiver-local logout.
- Check certified enrollment refresh without lease extension, generation
  invalidation with retained fences, logout during an enrollment outage and
  unknown logout before admission. Five native certified receiver cases now run
  through `make test-memory`; these do not qualify installed-host recovery.
- Verify the new fixture bytes through the normal focused and hosted checks.
  Existing protocol bytes, cryptographic primitives and product behavior are unchanged.

## [0.45.0] - Source checkpoint 2026-11-15

Source checkpoint: `ic-auth` 0.45.0. Unchanged protocol-types/Wire remain 0.42.0.
All packages remain `publish = false`.

### ✨ Added

- Add synchronous `MemoryReceiver::authorize` application projection after current
  session checks, with borrowed receipts that cannot escape the invocation. Keep
  application results separate from Auth refusal; recheck before effects after an
  await. Qualify refusal without callback execution and both proof mechanisms.
- Add independent signed multi-device and second-issuer fixtures. Two receivers
  demonstrate issuer-qualified quota isolation, replacement with an unaffected
  second device, proof-expired retry, local logout and original-deadline renewal.
- Inject test-only interruptions between memory publication mutations for admission,
  renewal/replacement, unknown-to-known transfer, logout and pruning. Every injected
  interruption must leave all authentication and mutation entrypoints closed;
  successful completion restores readiness. No fault controls ship in deployed builds.
- Make volatile Auth the current milestone and defer stable storage until its
  supplier contract is ready. Remove the interim `ic-storage` adapter plan;
  `ic-storage-new` remains the intended future durable destination.
- Qualify memory operation sequences with an independent fixed-scenario ledger:
  353 traces and 7,307 operations across two receivers compare admission/retry,
  renewal/replacement, logout, authority changes, pruning, byte occupancy and
  retained work charges. A deliberately broken write-then-error adapter is
  rejected by the model; actual host rollback remains a separate integration gate.
- Add passive bounded memory-session inspection with exclusive Principal cursors,
  at most 32 results and 128 scanned caller rows plus one lookahead. Empty pages
  advance across retained fences; expired/stale sessions remain diagnostic facts.
  Instrumented work counts and signed receiver cases establish limits, cursor
  behavior across pruning and absence of clock/budget/storage mutation.
- Replace bare memory-receiver activation with bound boot/refresh requests and
  admission of host-authenticated enrollment metadata. Reject mismatched response
  source, receiver/domain/challenge, late responses, expired/overlong freshness,
  revision/generation regression and conflicting authority content. Preserve exact
  response retry and prior accepted state on refusal.
- Cap session leases at accepted enrollment freshness. Refresh cannot extend an
  existing lease; generation changes invalidate sessions without resetting safety
  records or credit. Expired enrollment blocks admission/read while safe logout
  remains available. Closed/lost instances cannot reopen through enrollment.
  Full snapshot authentication, entry/revocation validation and atomic installation
  remain host obligations; installed recovery and stable storage are still open.

## [0.44.0] - Source checkpoint 2026-11-15

Pushed as [1e50f32](https://github.com/dragginzgame/ic-auth-new/commit/1e50f32469bdba8ce1673ea4543cd0b6782720e7).
Package versions: `ic-auth` 0.44.0, protocol-types/Wire 0.42.0.
All packages remain `publish = false`.

### ✨ Added

- Add hosted CI for Linux and both macOS architectures, pinned tool preparation,
  snapshot/pin admission, native contracts and independent signing fixtures.
  Linux also checks formatting, strict lint, Rustdoc and Wasm compilation; one
  separate job validates the workflow and scans full history with redacted output.
  Reuse authenticated Toko 0.44.4 setup/cache/evidence helpers for
  [issue #1](https://github.com/dragginzgame/ic-auth-new/issues/1).
- Replace known-only revocation with caller-authenticated local logout. Unknown
  callers use a separate bounded denial pool and initially empty operation budget;
  retries retain their deadline and remain unconfirmed. Unknown traffic cannot
  consume known-login capacity or proof credit. Expired denials transfer to known
  logins only after successful proof verification, or are explicitly pruned.
- Share retained refill arithmetic between independently owned proof and logout
  budgets. Add signed future-credential and admission/logout ordering cases,
  clock/overflow refusal, pool isolation and exact Wire result projection evidence.
  Canic still owns endpoint wiring; host boot/recovery and stable storage remain open.

## [0.43.0] - Source checkpoint 2026-11-15

Pushed as [285ac17](https://github.com/dragginzgame/ic-auth-new/commit/285ac178cf8df9f10467368d51c117aa7853d5a3).
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
