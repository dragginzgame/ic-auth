# Current handoff — 2026-10-30

## Base, scope and delivery

The cleanup is committed in `9879db386c3cda06cbff66f488b1bae1c08d1576`, atop Auth
**0.23.0**, `95dbb9011a90b7119fa14e423f3112f1956f3c41` (`Release 0.23.0`).
[#115 — Reduce admission ownership and remaining protocol/client complexity](https://github.com/dragginzgame/ic-auth/issues/115)
owns this ten-candidate cleanup and measurements. Pending changelog is **0.24.0**;
own versions remain **0.23.0**. The client-tool fixture repair is local, unstaged
and uncommitted. No Git delivery, release or publication ran for this repair.
Retain release/recovery state. [#113](https://github.com/dragginzgame/ic-auth/issues/113) and
[#114](https://github.com/dragginzgame/ic-auth/issues/114) own prior cleanup and
Memory/Storage adoption; their release checkpoint is in [Git history](https://github.com/dragginzgame/ic-auth/blob/95dbb9011a90b7119fa14e423f3112f1956f3c41/docs/status/current.md).

Shared Tooling remains **0.27.6**, revision
`5a72359b5be19d5221334cf2c9b3204621d3f3f6`. Platform/macros **0.8.2**,
Tooling/Testkit **0.70.2**, Wire/derive **0.7.23**, Memory/Storage **0.56.0** and
Metrics **0.17.4** are unchanged. The lock adds only the private qualification
contracts' dependency on Auth protocol types. Rust **1.91** remains the minimum;
development uses **1.99.1**.

## Pending consumer migration and cleanup

Admission consumes `SessionRequest` and moves its scopes into the session. Retain
raw inputs or explicitly clone when the host needs a checked request afterward.
`PortableVerificationContext::Ed25519` borrows immutable `PreparedIssuerKey`.
Admit protected issuer bytes once with Platform's existing parser/weak-key checks;
retain original bytes and rebuild after configuration decoding. Live mechanism,
binding, grants, clock, revocation, budget, strict signature and canonical-scalar
checks remain. This caches key shape only, never proof validity or TEE assurance.
Standalone raw-key verification retains its API and rejection order.

Read endpoints project `SessionError::read_refusal()` into finite
`SessionReadRefusal` categories. Wire `ReadSession` moves to **0x415554480013**;
old **0x415554480003** frames are refused. Update readers/receivers together;
keep detailed errors host-side. All five browser contracts were regenerated and
reviewed; only `read.ts` changes. Rust/TypeScript/Rust vectors include read refusals.

The remaining cleanup uses a zero-size certified-leaf value, owner-local codec
error mapping, singleton/pre-sized Merkle witnesses and inline Principal fields
in private controls. Logout cleanup projects loop state instead of cloning whole
journals; returned snapshots remain detached. Tooling delegates help/usage to Clap:
usage errors exit 2, operational errors 1, help/version 0. Current adoption notes
are reconciled. Root ECDSA preparation was measured and deliberately not adopted.
See [sessions](../sessions.md), [Wire migration](../../crates/ic-auth-wire/README.md)
and [consumer migration](../canic-adoption.md).

Released 0.23 APIs still require owned Session/Replay commits, compact receipts,
checked `RequiredScopes` and immutable `SessionLimits`. Callbacks commit atomically;
receipts authorize nothing. Preserve current authority/generation and store checks.
No signed bytes, session/replay/login record, `AUTHNAT2` checkpoint or browser
journal format changed in this batch. The TEE/non-TEE trust model is unchanged;
there is no verification-result cache or replacement cryptography.

## Matched evidence and limits

Evidence is retained in `target/cleanup-024-20261030/`: archived base source,
separate build outputs, before/final/trial artifacts, lock identities, patches,
logs and PocketIC state paths. Fresh same-lock reference Wasm is **2,038,026 →
2,039,331 bytes**, **+1,305 bytes (0.064%)**. This is a small overall size increase.
The three fresh small Ed25519 admission samples have median **2,573,594 →
2,395,538 instructions (6.9% lower)**. Ordinary authorization remains **5,381**;
small/wide records remain **364/479 bytes**. Small Ed25519 record decoding is
**28,362 → 27,801**. These are fixture measurements, not production estimates.

The isolated Root-key prototype added **1,325 bytes** for roughly **158,000**
instructions saved within a **440-million**-instruction certified admission
(about **0.036%**). Keep the existing Root API/verifier. BLS dominates that path;
no BLS algorithm, validation rule or certificate trust change was attempted.

Focused engine, strict/adversarial signature, revocation, refusal, Wire, private
host, metrics and tooling tests pass. Native/all-feature and Wasm host Clippy,
Rust 1.91 native/Wasm checks, dependency boundaries and compile-fail docs pass.
Client qualification passes the packed consumer, all **69** lifecycle/codec tests
and four cross-language vectors. Installed final-Wasm cases pass live authority
and certification, composed roots, protected checkpoint upgrade and SDK signed
ingress/admission/logout. PocketIC remains admitted **16.1.0** with Testkit 0.70.2.
The first final lifecycle run exposed an invalid dummy enrollment key in a test;
the corrected valid-but-mismatched key still proves controller mismatch refusal.
Retain that failure and its passing retry. No full CI, macOS, real Chromium or
production adoption is claimed.

The user's complete gate stopped at `test-client-tools`: its fake generator
omitted the new `read-vector.hex`. The fixture now emits/checks all four vectors,
including preservation on input drift. Focused `make test-client-tools` passes;
evidence is in `client-tools-fix.log` and fixture `client-tools.G4GAVw` under
`target/portable-fixtures/`. Complete CI has not been rerun for this repair.

## Memory custody and consumer ownership

Only the unpublished host selects Memory/Storage. Released 0.56 changes Memory's
allocation-ledger/Cell-value encoding to bounded Wire (`ICML`/`ICMWIRE1`). Older
Memory backing remains incompatible even with `AUTHNAT2` application payloads.
The host refuses it without clearing; retain older binaries/states. Any retained
installation needs a reviewed transition before upgrade. Never clear only the
ledger, reinstall or discard application records to get past admission. Auth's
earlier CBOR retirement authorizes no additional data retirement.

Hosts own protected grants/clock/generation, allocation, atomic commits and
certification. Canic owns endpoint/issuer/lifecycle acceptance in
[Canic #491](https://github.com/dragginzgame/canic/issues/491) and browser issuance
in [#507](https://github.com/dragginzgame/canic/issues/507); its shared metrics
adapter consumes Auth outcomes. No production wallet service, issuer adapter,
Canic acceptance or browser publication is established here. User delivery and
separately authorized complete CI remain separate from this local cleanup.
