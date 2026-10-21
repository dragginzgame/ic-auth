# Current handoff — 2026-10-21

## Source and selections

Delivered baseline is Auth **0.13.3** at
`7a1183ba8556ab7c6e7880332a8695af2b86c44f`. Package versions remain 0.13.3.
Pending notes select **0.14.0** because the admission error reply is a breaking
Wire contract change. Update receiver encoders and generated clients together;
no old text-error reader or parallel admission flow is retained.
The private browser constructor also changes to explicit `{ browser, account }`
roles so Canic's issuer caller does not get replaced by a fresh receiver key.

The incoming lock selects Platform **0.6.4**, IC Tooling/Testkit **0.63.2**,
Memory **0.50.8** and Wire/derive **0.5.16**. This batch preserves those incoming
selections. Shared Tooling remains the reviewed 118-file **0.25.9** snapshot at
`85aded191d85ac4a8c0e5deab207f6be18895862`. There are no sibling/path overrides.
Development tools use Rust 1.99.1; the package minimum remains 1.91.

## Admission refusal boundary

Auth owns canonical credentials/proofs, session/replay/revocation machinery,
bounded Wire contracts and the private admission/logout client. Canic owns
protected enrollment, actual caller/clock, issuer orchestration, atomic production
storage, endpoint exports and scoped guards. No sibling files were edited.

The passive `AdmissionRefusal` contract and the core
`SessionError::admission_refusal()` projection preserve diagnostic categories
without parsing strings. Wire's admission reply carries its bounded enum; the
SDK adapter throws `AdmissionError` with the generated refusal tag. Detailed
backend/crypto errors stay host-side. There is no automatic retry or issuance.
Only `NotYetValid` is a future-start hint, not evidence of a valid proof; the
caller must retain the same credential, original deadline and active generation.
Expiry, invalid proofs, stale authority, reset and throttling remain distinct.

Admission requests, successful replies, method identities, read/logout contracts,
canonical signed bytes and session/replay/login/journal storage are unchanged.
Production stores and memory migration are outside this batch. The generator
owns the changed TypeScript schema; checked-in artifacts are never hand-patched.
See [Canic adoption](../canic-adoption.md) and [browser contracts](../browser-client.md).

The coordinator supplies a temporary account signer only to issuer issue/lookup
operations and a separate browser signer to receiver admission. Both are bounded
by durable generation/cancellation and principal/target checks; only the browser
identity is returned for ordinary calls. The issuer adapter's saved intent must
retain the original account caller, separate browser key and immutable request.
No second journal, account store, receipt policy or production issuer was added.

## Validation and limits

Focused checks pass: all three refusal-projection cases, six Wire transport
cases, 14 volatile and 25 restoration-enabled session cases, portable revocation
and certified cases, 50 browser tests with three bidirectional Rust/TypeScript
vectors, and 11 host-tooling cases. Strict native/Wasm Clippy passes. Isolated
Rust 1.91 package native/Wasm feature matrices and internal applications pass;
normalized package archives, sources, manifests and locks remain unchanged.
Dependency boundaries, exact generated-source comparison, documentation links,
shared snapshot and scoped formatting/diff checks pass.

Five native fixture cases and all eleven live Testkit cases pass, including
actual SDK-authenticated future-start, expiry, bad-signature and stale-authority
refusals. The same future credential succeeds later; exact retry preserves the
original session lease. Earlier session regressions separately prove no
session/replay mutation before the start and one verification across exact retry.
The SDK transport performs no automatic admission retry or issuer signing.
The account/browser follow-through also passes all eleven live cases with
distinct ingress signers, immutable lost-reply recovery and browser-key logout.
The fixture passes the original signed proof expiry explicitly instead of
rebasing it on the later gateway/client startup time.
Issuer callbacks use authenticated read-only probes in the reference fixture;
this is not a production issue/lookup implementation. Three added browser cases
cover issuer target confinement, logout blocking late account signing and account
principal changes during signing. Evidence: `target/canic-account-0140/`.

Evidence is retained in `target/typed-admission-0140/`. The first sandboxed
attempt could not bind localhost; the run with loopback access passes using the
same already prepared Testkit 0.63.2 CLI/server and a fresh reference Wasm.
Rust/frontend input seals match across final checks and the incoming lock stays
byte-for-byte unchanged. Full CI, production IC, native macOS and native-browser
qualification are not claimed. No executable installation, commit, release,
publication or deployment ran.

## Production acceptance

Auth [#3 — Production issuer/browser acceptance](https://github.com/dragginzgame/ic-auth/issues/3),
[#58 — Production session persistence/reset acceptance](https://github.com/dragginzgame/ic-auth/issues/58)
and [#59 — Production abuse/workload qualification](https://github.com/dragginzgame/ic-auth/issues/59)
remain open. Reference SDK/Testkit tests use protected fixture enrollment and
injected issuer operations; they do not establish a production issuer or backend.

Canic's active owner checkout now explicitly selects the complete certified
verifier and locally fixes effective renewal deadlines. It preserves issuer-scoped
account identity, protected cleanup time and shared transactional login pruning.
Those owner-reported native/backend checks are not production integration; the
authorization facade remains deliberately fail-closed. Protected runtime
admission, retained ledgers/budgets, Root enrollment and successful guards remain in
[Canic #491](https://github.com/dragginzgame/canic/issues/491). Actual non-issuing
issuer reconciliation remains [Canic #507](https://github.com/dragginzgame/canic/issues/507);
future-start endpoint/client acceptance remains [Canic #418](https://github.com/dragginzgame/canic/issues/418).
Local consumer work is distinct from its default branch and installed acceptance.
The maintainer's production skew bound remains unqualified and live activation
stays closed. TEE-required hardware/enrollment evidence is separately unavailable;
controlled-clock software tests do not supply either qualification.

Previous delivery, tooling/release recovery and source-bound 0.13.3 validation
records remain in [the delivered handoff](https://github.com/dragginzgame/ic-auth/blob/7a1183ba8556ab7c6e7880332a8695af2b86c44f/docs/status/current.md).
Their retained evidence is not deleted or relabelled by this handoff refresh.
