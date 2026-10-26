# Current handoff — 2026-10-26

## Checkout and pending work

The maintainer completed release **0.16.2** at `9928476`; its saved release
plan is complete. Package versions remain 0.16.2 and notes target **0.16.3**.
This adoption has not committed, released, pushed, published or changed the
real checkout's release recovery state.

The current locked selection is Platform/macros **0.6.16**, Wire/derive
**0.7.12**, Host/Build/Testkit **0.68.5**, Memory **0.51.3**, File CLI **0.31.1**
and Shared Tooling **0.26.18** (`37652c087fff16f55f83d18d27ba17c83f34021c`,
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

## Release behavior and validation

Shared Tooling 0.26.18 makes normal patch/minor/major releases fresh attempts
from current clean committed metadata. Historical plans do not select or block
those attempts. Explicit `release-resume VERSION=X.Y.Z` retains exact-source,
version, payload and destination checks. Auth's fixtures cover interrupted exact
resume, dirty-source refusal, fresh validation from committed metadata and an
unrelated malformed historical plan. Build/validation evidence remains retained.

`target/shared02618/` records passing shared runner, Auth release/publication
fixtures (local/mocked effects), native/Wasm Clippy, snapshot integrity, dependency
pins, Make jobserver, ShellCheck, formatting and local links. The real manifests,
lockfile and completed 0.16.2 plan remain unchanged. Full CI, installed-canister
and native macOS qualification were not rerun for this script-only adoption.

The Wire generator API fix is already in release 0.16.2. Earlier evidence under
`target/wire-generator-fix/` includes 63 client tests, Rust/TypeScript vectors,
host/type tests, native/Wasm MSRV and dependency boundaries. That repair displays
bulk advice on stderr; Shared Tooling does not replace the required Wire API call.

[Historical handoff](history-0.16.1-adoption.md) retains previous adoption and
validation details; its release labels and tool availability are historical.
