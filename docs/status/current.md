# Current handoff — 2026-10-27

## Checkout and selection

The maintainer completed release **0.17.1** at `a53a0da`. Package versions remain
0.17.1. The working tree prepares **0.18.0** because the maintainer explicitly
selected Platform **0.7.0** for Auth's public Principal, parse-error and
canister-signature key identities. Consumers sharing those Rust types must align
their direct Platform dependency to 0.7; passive callers select only `principal`
with defaults disabled. Claims, Wire frames, stored records and browser journal
formats are unchanged; no reset is required. See the
[consumer migration](../canic-adoption.md#platform-07-migration).

Host/Build/Testkit now select **0.69.0**, aligned with Auth's Platform/macros
**0.7.0**. Incoming Memory **0.51.9** already selects Platform 0.7, so the old
Platform 0.6 package leaves the lock. Wire/derive **0.7.18** remains selected.
Existing external Principal conversions and the stable-structures identity remain
compatible. Other incoming package identities were preserved, including
crypto-common **0.1.6**, generic-array **0.14.9** and rustix **1.1.6**.

Shared Tooling **0.26.22** is adopted at
`5fc904a935d82ad31f0fba14434b71ccb8c10345` (125 files), including the SDK boundary
checker. Its owned-type Candid prohibition and native error/source/input rules are
explicit; prose adoption alone installs no runtime guard. Root release notes keep
emoji categories. Generic Shared matrices remain opt-in; Auth's actual adapter,
evidence and product checks remain in CI. Both lock-selected CLIs and the existing
File CLI **0.31.1** were explicitly prepared; checks remain offline.
No staging, commit, tag, push, release, publication or deployment occurred here.

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
The issue remains open for delivery of this uncommitted work; Platform's supplying
export issue is complete. Native-browser/production acceptance remains separate.

Current Tooling 0.69 evidence is in `target/ic-tooling-069-20261027/`:
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
