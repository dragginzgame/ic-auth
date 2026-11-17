# IC Auth

A clean restart of IC Auth around its current application-authentication design.
Volatile authentication storage is the current milestone. Stable storage is a
future milestone, targeting `ic-storage-new` with orthogonal persistence once its
contract is ready. No interim `ic-storage` adapter is planned.
The pre-recode implementation imposes no Wire/API compatibility obligation.
One current contract is qualified by independent fixtures and Rust/browser checks;
historical sources remain provenance, not executable test dependencies.

**Status: volatile receiver/issuer libraries and partial browser lifecycle implemented.**
The three product crates own passive protocol types, exact Wire contracts and Auth
verification/state transitions. Platform owns cryptographic primitives; Canic owns
application endpoint wiring. Auth installs no endpoints or replacement executor.
The receiver retains bounded session/replay/device state; the issuer retains
issuance/renewal intents and exact-result reconciliation. Both preserve host-owned
transaction boundaries and refuse recovery by resetting lost volatile state.

The private browser package implements durable local invalidation, credential
retrieval, coalesced receiver readiness/admission, guarded ordinary calls, scheduled
cleanup and host-owned key disposal. Its shared HTTP transport uses Platform's
certified update path. Complete consumer integration and external signer recovery
remain open. See the
[current handoff](docs/status/current.md) for present capability and evidence limits.

Release versions are Auth 0.47.1 and protocol-types/Wire 0.43.0 (`v0.47.1`).
The three product crates are authorized for crates.io publication. Qualification
crates retain `publish = false`; the browser client 0.1.0 remains private.
Production deployment and consumer cutover require separate approval.
See the [publication scope](docs/development/publication.md) for dependency order
and package exclusions.

- [Architecture and qualification plan](docs/design/DESIGN.md)
- [Design and ownership overview](docs/design/authentication.md)
- [Compatibility and acceptance](docs/design/compatibility.md)
- [Current handoff](docs/status/current.md)
- [Developer tooling and focused checks](docs/development/tooling.md)

Run `make help`, `make toko-tooling-check`, and `make deps-list` from the root.
`make install-tools` prepares the pinned tools; `make tools-check` verifies them.
`make test-request`, `make test-claims` and `make test-wire` exercise the implemented contracts;
`make lint-protocol`, `make doc-protocol` and `make check-protocol-wasm`
cover their other focused checks.
The shared `make deps-update` maintains the lock; `toko-cloc -d 1` reports source
and test counts. See the [request boundary](docs/development/request-boundary.md)
and [claims boundary](docs/development/claims-boundary.md) for evidence and scope.
The [Wire boundary](docs/development/wire-boundary.md) records native budget and
current generated-browser checks. `make qualify-wire` runs native resource checks;
`make qualify-wire-browser` needs Node 24.21.0 and independently assembles all
37 current fixtures without a legacy checkout.
The [Ed25519 verification boundary](docs/development/verification-boundary.md)
records protected inputs, refusal/charging order and independent signatures.
Run `make test-verification` for its focused strict-verification checks.
The [certified issuer boundary](docs/development/certified-issuer-boundary.md)
records that component's scope and real certificate fixtures;
`make qualify-certified-issuer` runs those checks.
The [delegation encoding boundary](docs/development/delegation-encoding-boundary.md)
records root signing commitments and fixtures; run `make test-delegation-encoding`.
The [root verification boundary](docs/development/root-delegation-boundary.md)
records enrollment, grant binding and proof limits; run `make qualify-root-delegation`.
The [complete certified boundary](docs/development/certified-verification-boundary.md)
records the combined path and independent cryptographic evidence; run
`make qualify-certified qualify-shared-verification`.
`make test-wire-projection qualify-wire-admission` covers consuming Wire/domain
projection and real proofs through the verifier; Canic still owns endpoint wiring.
`make test-session-policy` checks role-specific narrowing and captured lease
ceilings shared by both verifiers; see the [request boundary](docs/development/request-boundary.md).
`make test-receiver-policy` covers [read/retry and login decisions](docs/development/receiver-policy-boundary.md)
over bounded facts, including independent proof-expired retry and replacement fixtures.
`make test-admission-capacity test-credential-profile` checks physical capacity
accounting and protected proof dispatch. `make test-memory` exercises the
[memory receiver](docs/development/memory-receiver-boundary.md) with real Ed25519
and certified proofs. Select `receiver-memory` together with at least one complete
verification feature. Host inputs and transaction/recovery custody remain explicit.
Import receiver host inputs, limits, errors and inspection results from
`ic_auth::receiver_memory`; policy fact assembly and decisions are private.
The certified qualification now covers independent issuers/devices/receivers,
replacement, renewal, logout and enrollment changes using independent signed fixtures.

`make test-issuer` checks the volatile issuer's capacity reservation,
single dispatch, lost-response reconciliation, stale-callback refusal, browser
renewal, sequence watermarks, retained results, bounded pruning and certified
publication failure containment. `issuer-memory` builds
independently of receiver/verifier features; the
host owns account authentication, entropy, approval, signing and transactions.
For browser-authenticated issuer logout, call
`MemoryIssuer::revoke_browser(caller, account, login_epoch, login_id, now_ns)`.
The host supplies the actual caller/time; account/login fields only select the
retained binding. Auth checks the browser key and revokes that login synchronously,
fencing pending completion and renewal while retaining exact result evidence.
The returned original login deadline is metadata, not acknowledgement from another
receiver. Account-authorized revocation remains `revoke`. Missing/pruned bindings
refuse; constructing fresh volatile state cannot establish logout.
Canic owns the issuer endpoint, codec and browser adapter in
[Canic #16](https://github.com/dragginzgame/canic/issues/16); this adds no Canic
dependency or issuer endpoint to Auth Wire.
Expired rows remain charged until explicit pruning after their captured retention
deadline. Pruning preserves referenced logins and closes consumed-ID epochs;
certified witnesses remain host-owned and are refreshed without republishing
completed claims. `make qualify-host` checks actual installed issuer rollback,
certificates and upgrade-loss closure with a prepared PocketIC server. See
[host evidence and limits](qualification/ic-auth-host-conformance/README.md).
Receiver checks also cover admission/logout/application rollback, normal refusal
charging and closed upgrade loss. A real installation-bound authority call now
checks reply binding, terminal single-key revocation and atomic callback publication.
All installed checks use one simulated subnet. Late/overlapping-reply integration
checks remain outstanding for the supported Canic flow.
Full Canic registry integration and production challenge/refresh handling remain
unqualified.

The private [browser client](packages/client/README.md) exposes a lifecycle-only
`LoginJournal` handle plus `retrieveCredential`, `establishSession`, `cleanupOnce`
and `httpUpdateTransport`. Journal stages and the bounded signing helper remain
internal. `make test-client` checks types, SDK signing, codecs, HTTP binding and
API boundaries; `make test-client-journal` checks native IndexedDB and cross-tab
behavior. `make check-client-generated` checks adopted Wire output.
`make qualify-client-http` checks certified replies, wrong-root/tamper/replay
refusal and a complete fixed-fixture journey from issuance reconciliation through
admission, a protected write and logout. All run on one subnet. Production Canic
wiring, multi-receiver coordination and browser lifecycle acceptance remain open.

The objective is less maintained state and clearer ownership, not a promised
zero-debt result or a line-count quota. Keep rejection, atomicity and recovery
proofs when replacing implementation. Do not copy the old release system, task
history or compatibility layers merely because they exist.

Source provenance: `dragginzgame/ic-auth-legacy`, commit
`e52a40be75dd9809d0528189d84b43fe96952ad9`. See `reference.sha256` for the selected
reference files. Licensing is retained in [LICENSE.md](LICENSE.md).
