# Current handoff — 2026-10-25

## Checkout and pending work

The checkout is at release **0.16.1**, commit `964a55a`; its saved release plan
is complete. Package versions remain 0.16.1. Source notes target **0.16.2**;
this cleanup has not committed, released, pushed or published anything.

The published selection is Platform/macros **0.6.16**, Wire/derive **0.7.10**,
Host/Build/Testkit **0.68.4**, Memory **0.51.1**, File CLI **0.31.1** and Shared
Tooling **0.26.16** (`2077c934efb82b181bea58fe50078dbad289c1aa`, 122 files).
Registry metadata and packaged VCS identities were inspected; source evidence is
under `target/deps-latest/`. The lockfile selections were incoming and remain
preserved. PocketIC pins thiserror 2.0.18; SHA-2 0.10 and Dalek 2 remain aligned
with Platform. No application/storage format or runtime capability changes.

The five browser files were regenerated with Wire 0.7.10. Existing browser cap
values now come from Rust's Auth transport policy; protocol budgets use Wire's
`method_limits` projection. The generated codec configuration is observed and
compared to structured Rust policy, with request/reply byte/node/depth mutation
refusals. Intentional browser byte asymmetry retains its former values and
behavioral coverage. Native allocation policy is not a JavaScript heap bound.
Full native/browser boundary equivalence and real-browser execution remain
qualification gaps under [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).

## Current cleanup

Session restoration and successful admission share infallible index installation;
validation and durable commit still precede publication. Regression coverage
checks restored replacement, old expiry removal, replay counts and pruning.
Browser login metadata and HTTP destinations are captured before asynchronous
work, and guarded identities return detached Principal values. The journal
schema, retry limits, expiry policy and signing checks are unchanged.

Qualification uses shared context field selection and empty budget construction,
while installation/reset/restoration retain their own readiness decisions. The
context macro preserves disjoint mutable storage borrows. Browser generation
selects all metadata/codecs through one private contract binding macro.

## Ownership and consumer acceptance

Auth owns reusable proof/credential verification, one session engine, bounded
Wire contracts and the private browser admission/logout coordinator. Canic owns
topology, authority approval/renewal, endpoint guards, production enrollment,
deployment configuration and its durable host adapter. No standalone wallet
service is implemented. See the [Canic adoption guide](../canic-adoption.md).

[Canic #491](https://github.com/dragginzgame/canic/issues/491) retains runtime,
store, reset and load acceptance; [Canic #507](https://github.com/dragginzgame/canic/issues/507)
retains issuance policy, abuse, reconciliation and browser integration.
[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3) owns real-browser
journal qualification. Local fixtures do not establish production adoption,
clock-skew qualification or TEE custody.

`AuthRequestMetadata`, empty Rust `LogoutRequest` and older session error
conversions remain supported during Canic integration. Signature vectors and
independent crypto/rule/revocation fixtures remain separate. Current follow-ups
include certificate freshness [Platform #51](https://github.com/dragginzgame/ic-platform/issues/51)
and advisory tracking [Auth #85](https://github.com/dragginzgame/ic-auth/issues/85)
through [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).

## Validation

The earlier cleanup evidence remains under `target/cleanup-seven/`, including
62 browser tests, six fixture unit tests and all 12 installed local Testkit
0.68.1 canister tests. It does not qualify the incoming 0.68.3 executable pairing.

The previous dependency evidence is under `target/deps-latest/`: regeneration and all
63 browser tests, Rust/TypeScript interoperability and client tool fixtures pass.
The approved Testkit/Build 0.68.3 executables are prepared. All six fixture unit
tests and all 12 live-canister tests pass, including certification, upgrades,
signed ingress and browser SDK admission/logout. Native/Wasm Rust 1.91 package
checks, Clippy, dependency boundaries and focused crypto/session/host suites pass.
Release/publication recovery fixtures and the command-stub runner pass with
local/mocked effects. Snapshot, pins, formatting, ShellCheck and doc links pass.

The next incoming update selects published Host/Build/Testkit 0.68.4 from
Tooling source `41f5d1688bbf4321be89d352974f17957a0ef0bc` (packaged identities
are recorded under `target/upstream-next/`). It raises its Platform/Wire minimums
to the already selected versions and simplifies CLI/build/filesystem mechanics.
Auth uses the same APIs and features; no migration is required. All 63 client
tests, transport/type and host tests, Clippy, isolated native/Wasm MSRV and
boundary checks pass under `target/upstream-next/`. Regeneration preserves all
five browser files byte-for-byte. Snapshot, pins, formatting and doc links pass.
The 0.68.4 Testkit/Build executables are missing; explicit setup approval is
pending before live-canister and release/recovery qualification. The 0.68.3
runtime/editor results above remain historical and do not qualify this pairing.

The previous lockfile remained unchanged during qualification. No commit, release, push,
publication or deployment was run. Full CI, native macOS and real-browser
execution were not run; traversal-boundary equivalence remains explicitly open.

[Historical handoff](history-0.16.1-adoption.md) retains previous adoption and
validation details; its release labels and tool availability are historical.
