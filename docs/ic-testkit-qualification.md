# IC Testkit signature, token, session and ingress qualification

`make test-qualification` first admits the prepared Testkit CLI/server, then
builds the internal Wasm host in
`apps/qualification/canister/` and runs `apps/qualification/runner/` through
published `ic-testkit` against its admitted server. Both packages are
unpublished. They are
qualification fixtures, not a wallet login service or a durable session backend.

Setup is explicit through `make install-tools` and `make fetch`. Tests run offline:
the wrapper asks the locked Testkit CLI to check its prepared server and consumes
only the returned absolute path. There is no test-time setup, server pin catalog
or retained-bundle fallback. `make install-testkit-tools` explicitly prepares the
CLI and server; `make testkit-tools-check` performs offline admission alone.
The current lock selects `ic-testkit` 0.34.3, which owns the PocketIC
dependency; runtime types are imported through
its re-export, and typed Candid calls preserve application results separately
from encoding, decoding and replica errors. There is no direct `pocket-ic`
dependency or exact-version pin exception in this workspace.
The selected transitive client and server are both 16.1.0. The IC agent is 0.49.2
and the host CDK is 0.20.3; versions and dependencies are owned by the virtual root.
PocketIC 16.1.0 pins `thiserror` 2.0.18, so the shared lock selects that version;
the existing authentication capabilities are tested with that selection too.

Wasm bytes are read through published `ic-host-fs` with a 16 MiB limit and no
symlink following, through the root-selected filesystem crate. Testkit owns bounded
server startup and retained stdout/stderr; each fresh instance has a 30-second
startup budget, while upstream operation limits remain 30 seconds. Each invocation
prints a unique retained state directory beneath
`target/portable-fixtures/qualification.*`. Its instances use distinct child paths;
SDK temporary files also use that root. Test logs and state remain inspectable
after success or failure. This does not prescribe a production identity store.

The shared `ci/ic-tools.tsv` now selects five tools and has no PocketIC row.
The former consumer matrix and alignment/binary checkers are removed under
[IC Auth #7](https://github.com/dragginzgame/ic-auth/issues/7). The canonical Rust
installer prepares `ic-testkit-server` from the one locked registry Testkit
package, with an explicit `debug` profile and versioned executable/receipt slot.
Testkit setup/check uses `.tools/testkit-server`; its owner authenticates assets
and admits the server. Older executable slots and six-tool bundles remain
retained. A dependency update that changes the selected Testkit package requires
explicit setup and qualification, rather than a test-time download.
If the locked Testkit selection changes during setup/check, the caller refuses
to report the former selection as success and requires another explicit invocation.

## Host and trust boundaries

The host uses the actual runtime canister ID, caller, clock and query certificate.
Controllers alone may prepare signatures, prune leaves or change the other
certified branches. Signing messages has no wallet eligibility meaning here.
Installation supplies a non-anonymous resource owner as protected metadata.
An ingress request cannot select that owner.

The fixture composes `assets`, `sig` and `status` under one published root.
Branch updates preserve the signature contribution; signature cleanup preserves
the other owners' roots. The host publishes after every successful mutation.
Queries obtain an actual data certificate and pass protected sibling hashes to
the library's [retrieval API](signature-preparation.md). The native verifier uses
the PocketIC instance's independently obtained network root, approved signer,
protected expected seed and host clock. Tests configure NNS and application subnets
so the certificate's real subnet trust chain is checked.

The controller-only preparation call now selects explicit seed/domain/message
inputs in this unpublished fixture. A combined-profile scenario uses the existing
`canic-root-role-attestation` and `canic-issuer-delegated-token` identities with
the same payload in one store. Both actual proofs verify under their own protected
profiles; crossed seed/domain use rejects. The 16-leaf fixture budget is shared,
failed capacity admission preserves both profiles, and live duplicate preparation
retains the original deadline. Expiring/pruning the short-lived Root leaf preserves
the issuer proof and frees a slot for that profile. Asset-branch updates and
upgrades preserve host metadata; upgrade clears both pending profiles, whose
re-preparation returns the same public keys. These are signing-profile and composed
root checks, not certified HTTP serving or application authority policy.
See [Auth #35](https://github.com/dragginzgame/ic-auth/issues/35) and
[Canic #60](https://github.com/dragginzgame/canic/issues/60).

The fixture's single stable-memory owner preserves its resource owner and branch
metadata with CDK storage. Pending signatures intentionally remain volatile.
Upgrades restore protected metadata, create an empty signature store and republish
the composed root. This is an explicit fixture lifecycle contract, not a stable
codec for the reusable library or a wallet-service upgrade policy.

## Exercised scenarios

The certificate scenario rejects unauthorized preparation, verifies an actual
Canic-domain signature, preserves retry deadlines, changes another branch without
losing signatures, upgrades the host, rejects lost pending retrievals, and
expires/prunes/reprepares a leaf while retaining other certified state.
It checks the DER identity is unchanged after upgrade and new preparation.

The ingress scenario enables a local HTTP gateway. The IC agent's standard
`Delegation::signable()` owns request-ID encoding; the fixture signs that payload
under `ic-request-auth-delegation`. The agent verifies the canister-signed link
against the local root and submits real signed update envelopes. The replica
authenticates the caller before the independent resource canister checks its
stored owner. Tests distinguish application denial of an authenticated non-owner
or anonymous caller from replica rejection of invalid credentials.

Fresh Ed25519 session keys authenticate as the same fixture principal before and
after both signer and resource upgrades. Previously issued, unexpired delegations
still authenticate after pending leaves are cleared. Wrong target, altered signed
expiration, mismatched session key and expired delegation are rejected by the
replica. Each negative ingress assertion requires HTTP 400/403 with the expected
authentication refusal reason; timeouts, connection failures and application
denial cannot satisfy it. Negative tests deliberately use the IC agent's
unchecked constructor
to bypass its early validation and reach the replica; no library proof verifier
is replaced or bypassed in production code.

## Evidence and limits

The complete-token scenario joins the public bounded Merkle constructor,
native test-key ECDSA root signing and an actual Testkit-certified issuer proof
with `token::verify_token`. It verifies the narrowed local grant, then rejects
the same token after changing caller, audience, scope ceiling, root-policy
deadline, key/proof/registry floors or network trust anchor. Advancing the actual
Testkit clock makes the issuer certificate stale while the claims remain valid.
After actual upgrade, pending retrieval rejects; controller-authorized
re-preparation of the unchanged claims verifies again with the same issuer
identity and preserved composed asset state. See
[Auth #18](https://github.com/dragginzgame/ic-auth/issues/18).

Released 0.4.2 routes that scenario through public
[token preparation and finishing](token-preparation.md): explicit host authority,
presenter, operation, clock and bounds produce the canonical signing hash; the
actual retrieved certified issuer proof is then attached. Subsequent live-policy,
certificate freshness and upgrade assertions still use complete verification.
This exercises the producer API requested in
[Auth #25](https://github.com/dragginzgame/ic-auth/issues/25).

The same certified token, retrieved after issuer upgrade, also passes through
`session::establish_session` and `session::authorize_session` against the bounded
volatile reference store. A different network trust anchor or ungranted narrowing
request rejects without consuming the proof; admission of the identical token
then succeeds. Advancing the actual Testkit clock beyond claims expiry rejects
fresh token verification, while exact retry returns the still-live session with
its original deadline and unchanged storage occupancy. A narrowed protected
scope ceiling or advanced generation rejects authorization and retry. Logout
preserves replay refusal, including after generation invalidation.

The released 0.3.5 extension injects host transaction refusal after successful
verification/staging and before the reference backend's commit. Fresh admission
returns `StorageFailure` with no session/replay records, quota usage, accounting
or expiry-index residue; the identical certified token then admits. Exact retry
past proof expiry also propagates transaction refusal, preserving the committed
pair and original lease before a normal retry succeeds. These are substituted
host failures with real certification and an actual host clock, separate from
consumer durable commits and restart recovery.
[Auth #20](https://github.com/dragginzgame/ic-auth/issues/20) tracks this session
extension separately from completed certified-token acceptance under Auth #18.

The released 0.4.0 extension admits that actual certified token under a protected
root enrollment whose deadline precedes both signed claims expiry and the
requested session lease. Verification preserves signed claims expiry while
reporting the capped proof deadline; session admission caps the committed lease.
Actual host-clock advancement checks unchanged exact retry and authorization
before the cutoff, then rejection at the exclusive cutoff without changing
accounting. Bounded pruning removes only the expired session. Restoring a
currently eligible enrollment still cannot consume the same proof again; replay
remains until signed claims expiry, after which bounded pruning reclaims it with
no accounting residue. [Auth #22](https://github.com/dragginzgame/ic-auth/issues/22)
owns this qualification. Canic must still decide its authority semantics and
qualify its real durable backend; this test changes no library policy.

Root signing in this scenario uses a deterministic native fixture key. It does
not qualify management-canister signing, a production token issuance endpoint,
browser reconciliation, stable session storage or Canic's adapter. The canister
remains a controller-guarded signature fixture.

The focused command is part of the configured CI/release gate on Linux and both
supported macOS architectures. Released 0.4.0's complete gate passes on all three
hosts in [matching-source CI](https://github.com/dragginzgame/ic-auth/actions/runs/38128802347).
The released 0.4.2 preparation journey passes the complete local Linux gate and
strict native/Wasm Clippy. Its [matching CI](https://github.com/dragginzgame/ic-auth/actions/runs/38138739513)
passes Linux; both macOS jobs remain queued at inspection. Previous-release
results do not qualify the current browser-client repair.

Fresh session keys stand in for browser keys; no browser UI, Solana wallet proof,
challenge/replay service or wallet-to-principal mapping is exercised. The fixed
fixture seed does not choose the wallet service's identity namespace. No stable
session transaction backend, Canic adoption, production deployment or performance
improvement is established. Wallet identity/linking/recovery, allowed origins and
targets, and maximum credential lifetimes still require the explicit decisions
in the [accepted design](design/extraction.md). The existing
[Canic extraction tracker](https://github.com/dragginzgame/canic/issues/491)
remains the adoption owner. Exact local evidence belongs in the
[current handoff](status/current.md).
