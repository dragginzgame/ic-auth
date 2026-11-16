# Installed volatile Auth qualification

`make qualify-host` runs five explicit installed tests through IC Testkit/PocketIC.
The harness uses IC Build 0.87.3 to build the release Wasm fixture once per process
and install its owned, hash-checked bytes in every case. Build owns the bounded
offline/locked Cargo operation, private target directory and artifact capture;
there is no prebuilt Wasm path or stale-artifact fallback.
Start a prepared PocketIC 16.x server and supply its URL; the target never
downloads or launches a server. For example, run `pocket-ic --port 4943` in a
separate terminal and use `http://127.0.0.1:4943` in place of the URL below:

```sh
POCKET_IC_SERVER_URL=http://127.0.0.1:PORT make qualify-host
make lint-host
# To select only receiver cases:
POCKET_IC_SERVER_URL=http://127.0.0.1:PORT make qualify-host HOST_TEST_FILTER=receiver::
# To select the authenticated enrollment case:
POCKET_IC_SERVER_URL=http://127.0.0.1:PORT make qualify-host HOST_TEST_FILTER=enrollment::
```

`make qualify-client-http` is a separate opt-in gate using the same built Auth
fixture on one live PocketIC subnet. It also requires the prepared client npm
dependencies and an explicitly selected Chromium executable:

```sh
POCKET_IC_SERVER_URL=http://127.0.0.1:PORT \
IC_AUTH_BROWSER_EXECUTABLE=/absolute/path/to/chromium make qualify-client-http
```

The harness owns each Testkit instance and deletes it at completion; the caller
owns the prepared server. PocketIC exposes a loopback gateway
and advances real time. Host Process 0.80.2 owns the direct Node child, with a
90-second deadline and 64 KiB per output stream. Playwright owns Chromium's
separate group and closes it in the runner's `finally`; forced descendant cleanup
after a killed Node process is not claimed. Node reuses the existing
browser runner; its `installed-http` and `installed-flow` groups are excluded from
ordinary browser runs. Both harnesses share setup through ordinary Rust modules.
Gateway/instance closure precedes the final result, including browser-test failure.

Five native Chromium cases exercise the public Auth HTTP adapter and actual SDK
certificate verification. A signed fixture `Prepare` update returns the exact
native Wire reply under the simulator's protected root. The wrong root and a
modified certificate response refuse. Replaying the valid Prepare certificate
against a distinct Complete request keeps the new request unresolved until the
bounded attempt times out, with only one update submission. An untampered Complete
retry then verifies successfully. No certificate verifier or successful reply is
mocked. These five raw transport cases retain their existing commands.

The second harness test runs four browser journey cases against the same Wasm's
qualification-only combined issuer/receiver owner. It uses real MemoryIssuer and
MemoryReceiver operations, generated product Wire codecs, the public browser API
and SDK-authenticated HTTP replies. A fixed account signs initial issuance; its
separate browser key signs admission, protected application calls and cleanup.

- A lookup before issuance cannot create an operation; a missing session cannot write.
- Losing a committed issuer reply leaves sequence zero pending. Reconciliation
  retrieves its original credential without another signature. Wrong scope refuses;
  actual admission binds the browser caller and account subject.
- The account identity cannot use the browser session. One authorized browser write
  commits; exact admission retry retains its lease without another issuer call.
- Durable local logout blocks new admission. One real remote cleanup confirms;
  subsequent writes and captured credential replay refuse at the host even when a
  raw caller deliberately bypasses the local journal. Repeated cleanup dispatches nothing.

The native harness independently observes one signature, one write, one retained
issuer operation, one receiver login fence and zero sessions. It aligns installation
time with the live gateway before creating bounded enrollment. Enrollment is a
protected installation-local test authority, not an invented IC response. Platform
signs with the existing public test seed locally; no external signing effect escapes
before message commit. The fixed proof has 60 seconds and the adapter conservatively
keeps 30 seconds from before initial dispatch, never refreshing that deadline on lookup.
This does not establish a general production clock mapping or external signer recovery.

These are qualification endpoints and one combined issuer/receiver, not Canic mounts
or multi-receiver integration. No product Rust or browser runtime code was added.
Production Canic wiring, browser readiness/coalescing, guarded ordinary-call API,
cleanup scheduling, renderer recovery and key disposal remain separate acceptance gates. The separate five rollback/loss cases still
run through `make qualify-host` without requiring a browser.

The Rust toolchain must include `wasm32-unknown-unknown`; both workspaces' locked
dependencies must already be fetched. Node 24.21.0 generates the independent
Ed25519 host vectors before each installed run. The Make target supplies absolute
Cargo/rustc paths from rustup and the fixture's exact Cargo package identity.
Build scratch belongs under `qualification/target/host-build/`; Build removes
its private child before returning success. Host FS 0.80.2 opens the scratch
parent and vector file through held directories; Host Artifacts 0.80.2 bounds
the vector read to 32 KiB. Tests are ignored in ordinary Cargo runs because they
require the explicit build/server setup. They are not yet part of hosted CI.

The fixture uses Platform's supported standalone entrypoints and IC Wire 0.24.2.
Its control identities are test-only and do not replace Auth endpoint identities.
Canic still owns production application wiring; this is not full Canic integration.
All new source is test code, classified through Toko LOC metadata. Rust product source,
features and dependency lock are unchanged; the fixture enables the existing
receiver/Ed25519 features alongside its issuer feature.

The tests install actual Wasm and establish:

- Traps after host leaf mutation, system root publication and Auth completion
  restore the previous issuer phase, reservations, credit and certified root.
  A host publication error must become an enclosing trap to achieve this rollback.
- A successful completion verifies against the simulator's actual IC root key,
  actual query certificate and canonical Auth claims. Completed retries skip
  publication and retain the original claims and root.
- Actual ingress caller mismatch, elapsed IC time and account revocation refuse
  before publication. These ordinary refusals return without trapping.
- A real upgrade loses the volatile heap, clears its certification contribution
  and leaves issuance, completion and revocation closed. There is no reset route.

Receiver cases additionally establish:

- Admission, known/unknown logout and a protected application projection roll
  back with the enclosing IC trap, including retained clocks, work credit,
  replay rows and login fences. Faults occur after successful Auth operations;
  internal publication interruption remains covered by the native fault tests.
- Ordinary bad-proof refusal commits its work charge and no identity. Successful
  exact retry after proof expiry preserves the original lease and proof credit.
  Committed logout removes the session but keeps replay and known-login fences;
  subsequent authorization cannot invoke the application projection.
- Actual upgrades lose both known revocation and unknown denial state. Enrollment,
  admission, authorization and logout stay closed afterward. An intact companion
  admits the same valid proof; its issuer-ahead credential also shows why an
  expired temporary denial cannot establish conclusive logout.

Enrollment now uses a real IC call to the authority chosen at installation.
Empty fixture install arguments select self; the enrollment case installs a
separate authority and encodes its principal with Wire's bounded payload codec.
Ingress carries neither the authority destination nor trusted reply metadata.
The receiver captures the destination before awaiting; the runtime owns callback
provenance, and the authority binds its reply to its actual IC caller.

The single-key enrollment case checks wrong receiver/challenge, expired response,
substituted key, unavailable authority and revision regression. After a successful
refresh, exact retry keeps its old session deadline. Traps after metadata or key
publication restore both within the callback transaction; the separately committed
request clock/challenge are not rolled back. The old session remains usable after
those traps. Committed key revocation disables admission/authorization, preserves
safe logout and refuses reactivation at a higher revision/generation. Upgrade
loss still prevents dispatch or reconstruction. Only the authority's actual IC
controller can configure these test failures.

This is a bounded single-key host fixture with fixed issuer/domain/grants and a
public test key, not Canic's complete enrollment registry or production boot flow.
Its host digests cover the test reply and key/revocation entry through Wire and
Platform. Challenges use a deterministic test sequence; production unpredictable
entropy, production refresh scheduling, complete multi-entry validation and
recovery remain separate obligations. No state borrow crosses the await, and no
new executor or cryptography is introduced.

All fixture canisters share one simulated subnet. The harness uses PocketIC's
single NNS/root subnet because its public `root_key()` API needs that subnet for
certificate verification. No application canister calls across a subnet boundary;
this is not a recommendation to deploy Auth on NNS. The enrollment test asserts
both canisters share the same subnet and the topology contains exactly one.

Installed late/overlapping-reply checks remain outstanding for the supported
single-subnet Canic flow. The semantic requirements still apply; native enrollment
checks remain intact. No alternate topology or request-order instrumentation is
retained to manufacture timing coverage.

The independent Node/OpenSSL memory-fixture packer has a `--host` mode: original
scenario timestamps become seconds relative to Unix nanoseconds
`2000000000000000000`. This permits forward-only simulator time changes while
preserving the field layout, grants, caller derivation and public test key.
It signs and verifies each generated digest outside Auth. The unchanged default
mode still reproduces all nine retained vectors exactly; generated host bytes
stay in `qualification/target/host-vectors/`. No production credential is created.

Initial installation deliberately creates fresh issuer state with protected fixture
constants and deterministic nonce/approval. It does not qualify production account
identity providers, entropy, complete authority enrollment or external signing services.
The fixture has one signature leaf and no application certification tree. Complete
authenticated receiver boot, browser lifecycle, Canic wiring, composed certification owners and durable
storage remain separate acceptance work. Closed loss containment is not recovery.

The Platform migration was exercised with the prepared PocketIC 16.1.0 server,
Rust 1.100.0 and Chromium 141.0.7390.37. All five installed tests and both HTTP
harness tests (five certificate cases and four journey cases) pass.

The qualification graph selects Testkit 0.107.1 and one Platform 0.34.5 / Types
0.21.1 cohort. The Wasm fixture selects Runtime/macros/guard 0.23.12 and explicit
bounded decoder/encoder hooks. Queries are synchronous; updates use the supplied
executor. Upgrade cases stop the canister before upgrade and restart afterward.
The qualification unsafe-code lint is `deny` so the Runtime macro can narrowly
expect its generated Wasm export attribute; no handwritten unsafe code is added.
PocketIC still owns its thiserror 2.0.18 constraint. No legacy Platform, Agent or
second Runtime is selected. Simulator/server provisioning remains host-owned.
