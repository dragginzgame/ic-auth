# Current handoff — 2026-10-27

## Checkout and selection

The maintainer completed release **0.18.0** at `c164419`. Package versions remain
0.18.0. That release selects Platform **0.7.0** for Auth's public Principal,
parse-error and canister-signature key identities. Consumers sharing those Rust types must align
their direct Platform dependency to 0.7; passive callers select only `principal`
with defaults disabled. Claims, Wire frames, stored records and browser journal
formats are unchanged; no reset is required. See the
[consumer migration](../canic-adoption.md#platform-07-migration).

Host/Build/Testkit select **0.69.0**, aligned with Auth's Platform/macros
**0.7.0**. The incoming Memory **0.51.10** lock edit is preserved unchanged.
Wire/derive **0.7.18** remains selected.
Existing external Principal conversions and the stable-structures identity remain
compatible. Other incoming package identities were preserved, including
crypto-common **0.1.6**, generic-array **0.14.9** and rustix **1.1.6**.

Shared Tooling **0.26.23** is adopted at
`48ec9794892da6f8e2812653ec16c5e8ea23df47` (126 files), exported from a clean
isolated checkout of upstream main; sibling edits were not imported. The default
Rust formatting driver supplies per-pass progress/timing and checked input reuse.
Auth's explicit module paths disable reuse, and staged hooks always run formatting.
The actual hook fixture includes the new driver. Canonical Tooling gzip guidance
is adopted; Auth has no local application gzip encoder or compressed-byte change.
[Auth #91 — Adopt Shared Tooling 0.26.23 formatting orchestration](https://github.com/dragginzgame/ic-auth/issues/91)
records adoption and `target/shared-tooling-02623-20261027/` retains impact/refresh,
snapshot, normal/forced format and staged-hook evidence. Local/mocked release
metadata, exact tags/push, recovery and publication reconciliation pass with the
new formatter. Documentation links and diff checks pass; Rust/product suites
were not rerun for this tooling-only adoption. Generic Shared matrices remain opt-in; actual
adapter, evidence and product checks remain in CI. Both lock-selected CLIs and the existing
File CLI **0.31.1** were explicitly prepared; checks remain offline.
No Auth staging, commit, tag, push, release, publication or deployment occurred
during the current cleanup.

## Current complexity cleanup

[Auth #90 — Consolidate admission policy and simplify client and qualification state](https://github.com/dragginzgame/ic-auth/issues/90)
owns the earlier nine local cleanups and the five additional approved Rust API
simplifications. The pending **0.19.0** hard cut gives admission the complete
credential and protected verifier, shares `PortableAuthority` between admission
and liveness, separates immutable reads from explicit admission/logout commits,
exposes typed ledger state with named bindings, and checks TTL policy at
construction. No compatibility aliases or callback adapter remain; see the
[migration](../sessions.md#session-commit-write-migration).
Signed, Wire, stored and journal formats are unchanged and require no reset.
Canic's current 0.18 selection and normal sibling checkout remain untouched;
consumer adapter adoption follows an Auth release under Canic's own release policy.

Current hard-cut evidence is in `target/hard-cut-20261027/`: 105 focused Auth
runtime tests and three invariant compile-fail doctests pass. The new API retains
frozen record bytes, typed refusals, replay/atomicity, budget charging and current
TTL/authority checks. Native qualification callers, session-only and standalone
Ed25519/certified library profiles pass warnings-denied Clippy. The Wasm fixture
passes Clippy and six unit tests; its release Wasm is built. Selected library
features pass Rust **1.91** checks on native and Wasm. Fresh generator comparison
keeps all five browser files unchanged. Six filtered native session/config cases
pass, including upgrades, fresh authority, signed ingress and actual SDK logout
recovery. The prospective Canic adapter migration is recorded on
[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6442148823).
Full CI, the complete package/MSRV matrices,
actual-browser and native macOS runs were not performed.

The earlier local cleanup evidence remains in `target/complexity-20261027/`,
including 65 browser tests, Rust/TypeScript interoperability, native session/config
qualification and local/mocked release/retry/publication caller fixtures. Initial
missing browser vectors, formatting refusal and sandbox loopback failure remain
retained alongside corrected runs. Those prior results retain their own inputs;
unchanged frontend and release-caller behavior was not rerun wholesale here.

The remaining fixture-wrapper adoption needs mutable Testkit client access.
[Tooling #257 — Expose mutable client access while retaining PocketIcInstance server custody](https://github.com/dragginzgame/ic-tooling/issues/257)
owns the supplying accessor, ready in
[Tooling PR #258 — Allow mutable Testkit client access without splitting server ownership](https://github.com/dragginzgame/ic-tooling/pull/258)
from an isolated checkout. Auth still uses published Testkit 0.69.0 and its
client-before-server teardown fields until that API is published; four live
gateway callers prevent adopting the immutable-only wrapper.

## Browser adoption and qualification

The private client now selects published `@tokostudios/ic-platform` **0.1.1** over
its exact SDK Core **6.1.1** dependency. Registry integrity matches the npm lock;
there is one installed SDK identity. Product, test and qualification imports use
Platform's explicit exports, including signing-request and expiry APIs. The
browser fixture imports Platform and maps its transitive SDK module. Auth retains
trust roots, zero retries, signing authority checks, Wire bytes and recovery.

The normal npm test command runs the adopted declaration/literal-import guard;
`make check-client-boundary` exposes it separately. Normal client tests include
alias and nested direct-import refusals. All five generated files are unchanged
after explicitly running Wire 0.7.18 generation. See
[browser contracts](../browser-client.md#sdk-boundary) and the
[Auth #87 adoption evidence](https://github.com/dragginzgame/ic-auth/issues/87#issuecomment-6439885414).
That adoption is present in release 0.18.0; Platform's supplying export issue is
complete. Native-browser/production acceptance remains separate.

Earlier Tooling 0.69 evidence is in `target/ic-tooling-069-20261027/`:
- Targeted native Clippy covers tooling and the runner; Wasm Clippy covers the
  fixture. Both pass with warnings denied.
- `validation.GjFJX8` passes four host-tool tests, native/Wasm dependency
  boundaries and dependency pins.
- `validation.j3ri9R` passes **six** fixture tests and **12** local qualification
  cases: certification, composed roots, protected upgrades, signed ingress,
  scoped receivers/devices, SDK admission/logout, typed refusals and lost-reply
  recovery. Live state is in `target/portable-fixtures/qualification.Q5lAKe/`.
Offline generated-source comparison passes; the five browser files are unchanged.
Prepared input seals remain intact, and incoming work outside the Tooling batch
is preserved. No full CI, native macOS or actual-browser run was performed.

Earlier Platform/browser evidence is in `target/upstream-20261027/`:
`validation.05PteN` records **64** client tests and isolated Rust **1.91** package
payloads/internal apps on native/Wasm, including direct Platform Principal
exchange. `validation.Iv1DUH` records the previous live qualification. Those
suites were not rerun wholesale for this Tooling-only change.

## Ownership and acceptance

Auth owns reusable proof/credential verification, one session engine, bounded
Wire contracts and the private browser admission/logout coordinator. Canic owns
topology, authority approval/renewal, guards, enrollment, deployment and its
durable host adapter. No standalone wallet service is implemented.

[Canic #491 — Extraction and consumer adoption](https://github.com/dragginzgame/canic/issues/491)
retains runtime/store/reset/load acceptance. The
[Platform 0.7 consumer handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6439884953)
records the next public type alignment without claiming Auth publication or
Canic adoption. Sibling checkouts remain untouched. The prior 0.17 callback/error
migration and `AuthRequestMetadata`/empty Rust `LogoutRequest` contracts remain.
[Canic #507 — Non-issuing reconciliation qualification](https://github.com/dragginzgame/canic/issues/507)
retains issuance policy, abuse, reconciliation and browser integration.
[Auth #3 — Real-browser portable admission/logout qualification](https://github.com/dragginzgame/ic-auth/issues/3)
retains real-browser and production issuer acceptance. Local fixtures establish
neither production adoption, clock-skew qualification nor TEE custody. Hosts own
atomic session/replay storage and protected authority generations.

The crypto advisory remains tracked in
[Auth #85 — Qualify the crypto-bigint advisory](https://github.com/dragginzgame/ic-auth/issues/85)
through [Platform #50 — Crypto capability qualification](https://github.com/dragginzgame/ic-platform/issues/50).
Earlier evidence remains in `target/upstream-adoption-20261026/`,
`target/pr-integration-20261026/` and `target/complexity-last-two-20261026/`.
Normal releases start fresh; exact `release-resume VERSION=X.Y.Z` retains saved
source recovery. Preserve prior plans, build artifacts and evidence.
