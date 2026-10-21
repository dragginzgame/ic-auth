# Current handoff — 2026-10-21

## Source and selections

Delivered baseline is Auth **0.14.0** at
`39a572404374388117b01eb1518321526fe8cee5`; package versions remain 0.14.0.
Pending compatible notes select **0.14.1** for expanded SDK integration coverage
and corrected adoption guidance. No public API, Wire, credential, session or
journal format changes are pending in this task.

The incoming, already dirty lock selects Platform **0.6.5**, Wire/derive
**0.5.18**, IC Tooling/Testkit **0.63.2** and Memory **0.50.8**. Its bytes and
unrelated dependency changes are preserved. Shared Tooling remains the 118-file
**0.25.9** snapshot at `85aded191d85ac4a8c0e5deab207f6be18895862`.
There are no sibling/path overrides. Development tools use Rust 1.99.1 and the
package minimum remains 1.91.

## Current Canic alignment

Auth owns credentials/proofs, session/replay/revocation semantics, bounded Wire
contracts and the private admission/logout client. Canic owns protected enrollment,
actual caller/clock, issuer orchestration, atomic production stores, exports and
scoped guards. Sibling source remains read-only.

Canic's active owner checkout has adopted published Auth 0.14.0 and reports
passing shared-engine typed refusal and native/Wasm consumer checks. This is
local owner work, not default-main delivery or installed production acceptance.
It selects both complete verifiers, retains issuer-scoped account identity and
locally repairs effective renewal deadlines, protected cleanup time and pruning.
The actual receiver store remains test-gated and successful authorization is
still fail-closed; no missing producer API was established.

Released 0.14 separates explicit account/browser signers. Issue/reconcile receive
an issuer-bound account capability; receiver admission and ordinary calls use the
fresh browser principal. The original account, browser key/login, immutable
request/grants and admission deadline belong to the issuer adapter's saved intent.
Reconciliation recovers original evidence or remains unresolved, never reissues.
Issuer logout must resolve the browser key to that retained login binding without
requiring the account provider. No second intent store or production issuer exists
here. Typed admission refusals distinguish future starts from other failures;
only bounded explicit retries of the same credential are permitted.

## Pending SDK acceptance coverage

The existing SDK/Testkit case now runs two browser logins for one authenticated
account, each with a separate journal, across two actual local receivers.
Both devices retain their own issuer/subject metadata and original sessions.
Logging out the first preserves the second's exact session and permits its exact
admission retry without reissuance. Both logouts receive authenticated replies;
retained raw keys and newly signed renewal credentials cannot revive either login.
Durable guards refuse post-logout signing before network dispatch.

Issuer results are still injected and account calls are authenticated read-only
probes, not issue/lookup endpoints. Fake IndexedDB in Node does not qualify
native-browser persistence. The fixture's issuer also acts as a reference receiver;
its local logout export does not implement Canic's issuer receipt lifecycle.

## Validation and limits

Current evidence: `target/canic-014-followthrough/`. All 50 client tests pass with
three Rust/TypeScript vectors and exact comparison of all five generated files
against incoming Wire 0.5.18; no generated changes were needed. Strict runner
Clippy, Rust 1.91 runner checking and TypeScript compilation pass. Five native fixture cases and all eleven
live Testkit cases pass with a freshly built reference Wasm using incoming
Platform 0.6.5/Wire 0.5.18 and already prepared Testkit 0.63.2/server 16.1.0.
This is controlled Linux reference-host evidence, not a production skew bound,
TEE qualification, native macOS, installed IC or native-browser acceptance.
Local links, snapshot integrity, scoped formatting and whitespace checks pass.
The incoming lock remains byte-for-byte unchanged. No executable installation,
release, commit, publication or deployment ran.

Full CI and the full isolated public-package MSRV matrix were not rerun for this
test/documentation-only batch. Earlier delivered 0.14 projection/session/feature
matrix evidence remains at its [source-bound handoff](https://github.com/dragginzgame/ic-auth/blob/39a572404374388117b01eb1518321526fe8cee5/docs/status/current.md).

## Remaining production acceptance

[Auth #3 — Production issuer/browser acceptance](https://github.com/dragginzgame/ic-auth/issues/3),
[#58 — Production persistence/reset acceptance](https://github.com/dragginzgame/ic-auth/issues/58)
and [#59 — Production abuse/workload qualification](https://github.com/dragginzgame/ic-auth/issues/59)
stay open for actual consumer acceptance, not additional competing auth flows.
Protected Root enrollment, retained receiver ledgers/budgets, actual exports and
successful scoped authorization remain in
[Canic #491](https://github.com/dragginzgame/canic/issues/491).
Portable issuance, non-issuing reconciliation and browser-key issuer logout remain
[Canic #507](https://github.com/dragginzgame/canic/issues/507); actual future-start
endpoint/client acceptance remains [Canic #418](https://github.com/dragginzgame/canic/issues/418).
The private client's production distribution/integration is still consumer work.
Production clock-skew activation stays closed by maintainer direction; controlled
clocks cannot qualify it. TEE-required evidence remains a separate activation gate.
