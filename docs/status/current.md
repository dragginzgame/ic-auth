# Current handoff — 2026-10-26

## Checkout and selection

The maintainer completed release **0.16.3** at `a51fa31`. Package versions
remain 0.16.3; pending notes target **0.16.4**. This adoption has not committed,
pushed, released, published or changed the real checkout's recovery state.

The incoming lockfile already selected published Platform/macros **0.6.18**,
Wire/derive **0.7.16** and Host/Build/Testkit **0.68.6**. Registry metadata was
verified and that lockfile was preserved byte-for-byte. Memory **0.51.3** and
the separate File CLI **0.31.1** selection remain unchanged. SHA-2 0.10 and
Dalek 2 stay on Platform's compatible lines; no new capability is enabled.

Shared Tooling **0.26.19** is adopted at committed source
`c830c33447f0a84e015bda7fa14b6c096a58368e` (122 files), exported from a clean
copy without using the sibling's pending edits. Its changelog finalizer accepts
archived `Draft` labels below historical releases while retaining active
candidate/draft refusals.

The locked Wire generator was run and all five browser files remain identical.
Auth still supplies explicit bulk-diagnostic handling. Numeric browser policy
stays Rust-owned; native allocation policy is not a JavaScript heap bound.
No application protocol, signed bytes, stored format or public API changed.

## Ownership and acceptance

Auth owns reusable proof/credential verification, one session engine, bounded
Wire contracts and the private browser admission/logout coordinator. Canic owns
topology, authority approval/renewal, endpoint guards, production enrollment,
deployment configuration and its durable host adapter. No standalone wallet
service is implemented. See the [Canic adoption guide](../canic-adoption.md).

[Canic #491](https://github.com/dragginzgame/canic/issues/491) retains runtime,
store, reset and load acceptance; [Canic #507](https://github.com/dragginzgame/canic/issues/507)
retains issuance policy, abuse, reconciliation and browser integration.
[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3) owns real-browser
journal qualification. Local canister/SDK fixtures do not establish production
adoption, clock-skew qualification or TEE custody. Host stores still own atomic
session/replay/revocation commits and protected authority generations.

`AuthRequestMetadata`, empty Rust `LogoutRequest` and older session error
conversions remain supported during Canic integration. The browser retains its
direct SDK dependency pending the published-export prerequisite in
[Auth #87](https://github.com/dragginzgame/ic-auth/issues/87) /
[Platform #59](https://github.com/dragginzgame/ic-platform/issues/59).
Certificate-time API adoption remains [Auth #86](https://github.com/dragginzgame/ic-auth/issues/86);
the crypto advisory remains [Auth #85](https://github.com/dragginzgame/ic-auth/issues/85)
through [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).

## Validation and release behavior

Evidence is retained under `target/upstream-20261026/`: native/Wasm Clippy,
locked metadata, direct/transitive dependency boundaries, dependency pins,
Make jobserver, formatting/links, 63 client tests and Rust/TypeScript vectors,
type/tooling and signature tests, isolated Rust 1.91 package consumers on
native/Wasm, and the Wasm capability checks pass. All five generated outputs
were reviewed unchanged. Both locked 0.68.6 executables were explicitly prepared.

Local Testkit qualification passes six fixture tests and all 12 integration
cases, including real certification, upgrades, ingress, issuer-scoped accounts,
multiple receivers/devices and SDK admission/logout. The initial sandboxed run
could not bind loopback sockets; the host-permitted rerun passed. This is Linux
local-canister evidence, not native macOS or actual-browser qualification.

Shared changelog/runner and Auth release/publication fixtures pass with
substituted/local effects. No full CI or registry upload was run. Normal release
targets start fresh from clean committed metadata; explicit
`release-resume VERSION=X.Y.Z` retains exact-source recovery checks. Preserve old
plans and evidence. Detailed prior validation remains in the
[committed 0.16.3 handoff](https://github.com/dragginzgame/ic-auth/blob/a51fa3113914ab8a8a23040a35e7728b450207b1/docs/status/current.md)
and [historical handoff](history-0.16.1-adoption.md).
