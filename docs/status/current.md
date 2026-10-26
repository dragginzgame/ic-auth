# Current handoff — 2026-10-26

## Checkout and selection

The maintainer completed release **0.16.4** at `ca70a9b`. Package versions remain
0.16.4. The working tree prepares **0.17.0** for the certified-limit field removal
and ledger/session-error API changes. Consumers omit the zero-only
`CertifiedProofLimits::max_future_skew_ns`, select callback error types as needed,
and route complete proof errors through `SessionError::Credential`.
No real checkout staging, commit, tag, push, release or publication occurred.

The upstream-adoption batch's incoming lock already selects Platform/macros
**0.6.19**, Host/Build/Testkit **0.68.8**, Wire/derive **0.7.17** and Memory
**0.51.4**, alongside crypto-common **0.1.6** and generic-array **0.14.9**.
It was preserved byte-for-byte; locked source preparation succeeds. The Build
editor and Testkit CLI/server were explicitly prepared and pass offline admission.
The separate File CLI remains **0.31.1**.
Shared Tooling **0.26.20** is adopted at
`a7f85d45d4212d4c39225f58597580560a333550` (124 files). The exporter also selects
the Node guide newly referenced by dependency policy. Local guidance distinguishes
narrow checks from aggregate suites; detailed change accounting is opt-in.

## Auth-local simplification

Canister verification now uses Platform's authenticated signing-certificate time
result. The dependency minimum is 0.6.14; Auth retains DER/bounds, protected trust,
freshness and future-skew policy. Ambiguous time leaves are rejected, and complete
nonminimal LEB128 times remain accepted. This implements the local adoption in
[Auth #86 — Adopt authenticated signing-certificate time without duplicate CBOR decoding](https://github.com/dragginzgame/ic-auth/issues/86).

The login ledger represents reservations and revocations explicitly and derives
pool ownership from retained binding. One frozen storage projection preserves
current record bytes. The browser journal carries phase-specific types and commits
complete transitions while retaining its existing flat profile, uncertain issuance,
strict transactions, lease checks and original recovery deadlines.

Wire unit-enum projections share their declarations; publication arguments and
intent derive from one ordered package list. Browser policy accompanies its output
rather than being recovered from filenames. All five generated browser files are
unchanged. The private fixture groups protected context inputs separately from
mutable stores/budgets and no longer needs a field-borrowing context macro.

All ten Auth improvements are now implemented. Ledger callbacks return their
host error `E: From<LoginRevocationError>` directly; admission no longer captures
errors outside the callback. Logout still maps backend commit failures to storage
unavailability, keeping ledger capacity/clock refusals distinct. Complete proof
failures use only `SessionError::Credential`; direct `Signature` / `Certified`
variants and their `From` conversions are removed. Atomicity, work charges and
public refusal tags are preserved. See [session contracts](../sessions.md).

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

Current upstream evidence is under `target/upstream-adoption-20261026/`.
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
