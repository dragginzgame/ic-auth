# Installed volatile Auth qualification

`make qualify-host` runs five explicit installed tests through IC Testkit/PocketIC.
The harness uses IC Build 0.87.3 to build the release Wasm fixture once per process
and install its owned, hash-checked bytes in every case. Build owns the bounded
offline/locked Cargo operation, private target directory and artifact capture;
there is no prebuilt Wasm path or stale-artifact fallback.
Provide a prepared PocketIC 16.x executable; the target never downloads a server:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic make qualify-host
make lint-host
# To select only receiver cases:
POCKET_IC_BIN=/absolute/path/to/pocket-ic make qualify-host HOST_TEST_FILTER=receiver::
# To select the authenticated enrollment case:
POCKET_IC_BIN=/absolute/path/to/pocket-ic make qualify-host HOST_TEST_FILTER=enrollment::
```

The Rust toolchain must include `wasm32-unknown-unknown`; both workspaces' locked
dependencies must already be fetched. Node 24.21.0 generates the independent
Ed25519 host vectors before each installed run. The Make target supplies absolute
Cargo/rustc paths from rustup and the fixture's exact Cargo package identity.
Build scratch belongs under `qualification/target/host-build/`; Build removes
its private child before returning success. Host FS 0.80.1 opens the scratch
parent and vector file through held directories; Host Artifacts 0.80.1 bounds
the vector read to 32 KiB. Tests are ignored in ordinary Cargo runs because they
require the explicit build/server setup. They are not yet part of hosted CI.

The fixture uses Platform's supported standalone entrypoints and IC Wire 0.24.1.
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

Local evidence on 2026-11-16: the five single-subnet installed cases
pass, with strict native/Wasm Clippy, on Linux with Rust 1.100.0, IC Testkit 0.106.0 and PocketIC client 16.1.0.
After adopting Build 0.87.3 and Host FS/Artifacts 0.80.1, all five installed
cases and strict native harness Clippy pass again; the Wasm source is unchanged.
The existing local server reports 16.0.0, within Testkit's supported 16.x range;
its observed SHA-256 is
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
This records the exercised binary's identity, not independently authenticated
provisioning. Provisioning remains owned by Testkit/the host environment.

The qualification lock owns the native simulator/build graph. Testkit 0.106.0
still selects Host 0.79 transitively; its objects do not cross the new Build/Host
API boundary. PocketIC pins `thiserror = 2.0.18`. The Wasm fixture selects one
Platform runtime/guard pair, 0.22.5. It does not introduce another executor.
