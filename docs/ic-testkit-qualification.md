# IC Testkit signature, token and ingress qualification

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
`ic-testkit` 0.32.2 owns the PocketIC dependency; runtime types are imported through
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
fixed fixture seed and host clock. Tests configure NNS and application subnets
so the certificate's real subnet trust chain is checked.

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

Root signing in this scenario uses a deterministic native fixture key. It does
not qualify management-canister signing, a production token issuance endpoint,
browser reconciliation, stable session storage or Canic's adapter. The canister
remains a controller-guarded signature fixture.

The focused command is part of the configured CI/release gate on Linux and both
supported macOS architectures. Local Linux execution and strict native/Wasm
Clippy pass. Native macOS execution of this new fixture awaits its own CI jobs;
previous-release CI does not qualify uncommitted changes.

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
