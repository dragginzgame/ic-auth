# Current handoff — 2026-10-26

## Checkout and pending work

The checkout is at maintainer commit `262c500` (candidate 0.16.2). Package
versions remain 0.16.1 and notes target **0.16.2**; the latest reported release
attempt stopped at Clippy before version preparation. This repair has not
committed, released, pushed or published anything, or altered release state.

The current locked selection is Platform/macros **0.6.16**, Wire/derive
**0.7.12**, Host/Build/Testkit **0.68.5**, Memory **0.51.3**, File CLI **0.31.1**
and Shared Tooling **0.26.16** (`2077c934efb82b181bea58fe50078dbad289c1aa`,
122 files). The incoming lockfile is preserved. SHA-2 0.10 and Dalek 2 stay on
the Platform-compatible lines. No application/storage format or runtime
capability changes.

Wire 0.7.11 replaced the silent host generator with required acknowledgement
and diagnostic arguments. Auth now passes an empty acknowledgement map and
prints nonempty advice to stderr. The workspace declares the matching 0.7.11
minimum while retaining the selected 0.7.12 source. Regeneration preserves all
five browser files byte-for-byte. Browser cap values remain Rust-owned, with
structured configuration and byte/node/depth drift tests.
Native allocation policy is not a JavaScript heap bound. Full native/browser
boundary equivalence and real-browser execution remain qualification gaps under
[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).

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

Current repair evidence is under `target/wire-generator-fix/`. Native and Wasm
Clippy, all 63 client tests, Rust/TypeScript interoperability, host-tooling tests,
transport/type tests, isolated Rust 1.91 native/Wasm package checks and dependency
boundaries pass. Dependency pins, formatting, local links and diff checks pass.
The generator produced no bulk advice for the current Auth schemas; no paths
are acknowledged or silently suppressed.

Earlier cleanup/adoption evidence remains under `target/cleanup-seven/`,
`target/deps-latest/` and `target/upstream-next/`. The 12 live-canister and
release/recovery results for older tool selections are historical. This host
API repair did not rerun the complete CI gate, live-canister or release/recovery
qualification of 0.68.5, or native macOS/real-browser qualification.
No saved release or validation state was removed or reused for changed inputs.

[Historical handoff](history-0.16.1-adoption.md) retains previous adoption and
validation details; its release labels and tool availability are historical.
