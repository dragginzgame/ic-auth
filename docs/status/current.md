# Current handoff — 2026-10-26

## Checkout and selection

The maintainer completed release **0.17.0** at `d4a5295`; package versions remain
0.17.0. It includes the certified-limit field removal and ledger/session-error
API changes. Consumers omit the zero-only
`CertifiedProofLimits::max_future_skew_ns`, select callback error types as needed,
and route complete proof errors through `SessionError::Credential`.
The authorized merges of [PR #88 — Reduce routine agent checks](https://github.com/dragginzgame/ic-auth/pull/88)
and [PR #89 — Latest root changelog sections](https://github.com/dragginzgame/ic-auth/pull/89)
are on main at `bd5cae3`; the checkout was fast-forwarded. Compatible workflow
and documentation notes collect under **0.17.1**. Shared adoption remains
uncommitted; the incoming openssl-probe 0.2.2 lock edit is preserved. No release
or publication was executed in this merge/adoption batch.

The upstream-adoption batch's incoming lock already selects Platform/macros
**0.6.19**, Host/Build/Testkit **0.68.8**, Wire/derive **0.7.17** and Memory
**0.51.4**, alongside crypto-common **0.1.6** and generic-array **0.14.9**.
It was preserved byte-for-byte; locked source preparation succeeds. The Build
editor and Testkit CLI/server were explicitly prepared and pass offline admission.
The separate File CLI remains **0.31.1**.
Shared Tooling **0.26.21** is adopted at
`66c91c4412856baa147482e648fc80428db8b179` (124 files), including the Node guide.
Root release notes use its emoji category headings. Narrow checks and aggregate
suites remain distinct; generic Shared runner/archive matrices are opt-in while
Auth's actual adapter, evidence and product proofs remain in CI.

## Auth-local simplification

Release 0.17.0 implements all ten Auth improvements; see the
[changelog](../../CHANGELOG.md) and [session contracts](../sessions.md).
Platform's authenticated certificate time replaces duplicate decoding; the
minimum is 0.6.14. Auth retains protected trust, bounds and freshness policy
([Auth #86 — Authenticated signing-certificate time](https://github.com/dragginzgame/ic-auth/issues/86)).
Explicit ledger/journal states preserve stored formats, atomicity, work charges,
uncertain issuance and recovery deadlines. Callback errors propagate directly;
logout still projects backend failures to storage while ledger capacity/clock
refusals remain distinct. Complete proof errors use `SessionError::Credential`.
Wire projections, publication inventory and private fixture/generator wiring
share their existing owners; generated browser bytes remain unchanged.

## Ownership and acceptance

Auth owns reusable proof/credential verification, one session engine, bounded
Wire contracts and the private browser admission/logout coordinator. Canic owns
topology, authority approval/renewal, endpoint guards, production enrollment,
deployment configuration and its durable host adapter. No standalone wallet
service is implemented. See the [Canic adoption guide](../canic-adoption.md).

[Canic #491 — Extraction and consumer adoption](https://github.com/dragginzgame/canic/issues/491)
retains runtime/store/reset/load acceptance. The
[0.17 API adoption handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6420966334)
records the two inspected ledger-only test callbacks needing explicit error types
and the session-error projection change. Canic source remains untouched.
`AuthRequestMetadata` and empty Rust `LogoutRequest` also remain supported.
[Canic #507 — Non-issuing reconciliation qualification](https://github.com/dragginzgame/canic/issues/507)
retains issuance policy, abuse, reconciliation and browser integration.
[Auth #3 — Real-browser portable admission/logout qualification](https://github.com/dragginzgame/ic-auth/issues/3)
remains open. Local fixtures do not establish production adoption, clock-skew
qualification or TEE custody. Hosts still own atomic storage and protected generations.

The browser retains its direct SDK dependency pending
[Auth #87 — Adopt Platform's npm SDK Core boundary](https://github.com/dragginzgame/ic-auth/issues/87)
and [Platform #59 — Required browser exports](https://github.com/dragginzgame/ic-platform/issues/59).
The crypto advisory remains in
[Auth #85 — Qualify the crypto-bigint advisory](https://github.com/dragginzgame/ic-auth/issues/85)
through [Platform #50 — Crypto capability qualification](https://github.com/dragginzgame/ic-platform/issues/50).

## Validation and release behavior

Merge/adoption evidence is under `target/pr-integration-20261026/`.
The reconciled PR #88 passes actual Make jobserver, tool-evidence and Testkit
caller fixtures with substituted effects in the isolated checkout
`/tmp/ic-auth-pr-integration-20261026/target/portable-fixtures/`. PR #89's exact
content proof preserves release text and older history. Snapshot/link and diff
checks cover Shared's documentation-only update; no new Rust suite is needed.
[Post-merge CI](https://github.com/dragginzgame/ic-auth/actions/runs/40363871947)
is pending; no full local gate or native macOS rerun was made for this batch.

Earlier upstream evidence is under `target/upstream-adoption-20261026/`.
Targeted native Clippy covers the tooling and qualification runner; Wasm Clippy
covers the fixture. `validation.7l21Ib` records passing snapshot/link checks,
four host-tool tests and native/Wasm dependency boundaries. Testkit 0.68.8 passes
six fixture tests and all 12 local qualification cases, including real
certification, upgrades, signed ingress and SDK admission/logout, in
`validation.BdD2ja`. Live state is retained in
`target/portable-fixtures/qualification.BvC8FB/`. Prepared input seals remain intact.

Earlier simplification evidence remains under `target/complexity-last-two-20261026/`:
session/proof tests, all 63 client tests, generated-source comparison, library
Wasm and isolated Rust 1.91 payload checks, including the permissive-parsing
harness. All five generated files were unchanged with Wire 0.7.17. Those suites
were not repeated for this upstream batch, which changes no Auth Rust/browser
source or supported minimum compiler. Earlier local/mocked release fixtures
remain under `target/complexity-20261026/`. No full CI, native macOS or
actual-browser qualification, registry upload or deployment was run.

Normal release targets start fresh from clean committed metadata; explicit
`release-resume VERSION=X.Y.Z` retains exact-source recovery checks. Preserve old
plans/evidence. Prior evidence remains in `target/upstream-20261026/` and the
[committed 0.16.4 handoff](https://github.com/dragginzgame/ic-auth/blob/ca70a9bbd63f6522fc2dc8188b970514a8041077/docs/status/current.md).
