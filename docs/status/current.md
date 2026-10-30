# Current handoff

## Scope and delivery

Base: released Auth **0.24.0**, `8f407d75c2027dc4a315c143ec3cc349f6e8d3f2`.
[#117](https://github.com/dragginzgame/ic-auth/issues/117) owns the preceding
ten-candidate cleanup (eight accepted, two rejected).
[#118](https://github.com/dragginzgame/ic-auth/issues/118) owns four further hard
cuts, now implemented and qualified locally. Pending notes target **0.25.0**;
own versions remain **0.24.0**. Work is unstaged/uncommitted. No push, full CI,
release, publication, deployment or sibling edits ran. Preserve release/recovery state.
Prior adoption checkpoints remain in [base history](https://github.com/dragginzgame/ic-auth/blob/8f407d75c2027dc4a315c143ec3cc349f6e8d3f2/docs/status/current.md).

Shared Tooling remains **0.27.6**, revision
`5a72359b5be19d5221334cf2c9b3204621d3f3f6`. Incoming lock changes were preserved:
Platform/macros **0.8.3**, Tooling/Build/Host/Testkit **0.70.4**, Wire/derive
**0.7.24**, Memory/Storage **0.56.1**, Metrics **0.17.7**. Manifest floors stay.
Rust minimum remains **1.91**; development uses **1.99.1**. The only additional
local lock change adds the already-selected `serde_bytes` dev edge to the
feature-unification harness, whose shared corruption tests now require it.

## Implemented contracts

The protocol package owns checked `ScopeLabel` and `ScopeLabelError`.
Request/operation getters expose checked labels; use `as_str()` for text.
Constructors still accept strings, preserve bytes/count-before-grammar ordering,
and share admission without repeated grammar implementations. Error projection
is `CanonicalAuthError::Scope(cause)`, replacing copied invalid-label errors.
`ScopeLabel::validate` replaces the engine's detailed grammar helper.
Raw proof grants remain strings: exact retries retain deferred proof verification.
Delegation error fields/targets use `DelegationField`/`DelegationTarget`, retaining
display labels and verification order. The redundant Rust `LogoutRequest` is
removed; public Wire logout remains unit, including the meaningful generated TS alias.

Private session/login records store audience IDs as fixed 32-byte strings under
new identities `0x415554485311`/`0x415554485313`. Replay remains `0x415554485302`.
Public identifier text, RPC contracts and signing encoders are unchanged.
The maintainer explicitly confirmed **no retained current records; hard cut is OK**.
Old identities, wrong lengths and relabeled text bodies are refused. The private
checkpoint is `AUTHNAT3`; former `AUTHNAT2` refuses without clearing. Adopt the
whole checkpoint and its byte totals coherently. See [sessions](../sessions.md)
and [consumer migration](../canic-adoption.md). No fallback or implicit reset exists.

The preceding batch derives receiver bindings from admitted authority, shares
receiver assembly/typed readiness and receipt projection, stages pruning keys
with pool counts, and retains prepared fixture signing material. Installation and
restoration keep independent authority/lifecycle checks. The Rust generator owns
the admitted artifact inventory; independent codec vectors remain. All five
generated browser source files are unchanged after Wire 0.7.24 generation.

## Matched measurements

`target/hard-cuts-025-20261030/` retains actual dirty baseline/current source,
locks, trial artifacts, logs, states and `measurements.json`. Both final artifacts
use the same current external dependency selections and runner. Baseline is the
already-cleaned #117 source, not the released 0.24 tree. Raw Wasm is
**2,030,943 → 2,031,570 bytes**: **627 bytes larger (0.031%)** for these four cuts.
Final SHA-256: `2ac1707d21e61e8ebae557fc7843a17af0a303f3b48e0493a92eab6712bba83d`.
Rebuilding after the harness-only lock edge produces identical bytes.

Small session records: **364 → 300 bytes**; wide: **479 → 415**. Small Ed25519
record decoding: **28,401 → 22,068 instructions** (22.3% fewer); certified:
**28,357 → 22,068**; wide: **49,206 → 42,902**. Small authorization is
**5,395 → 5,400**; wide **27,728/28,017 → 27,820/28,109**. Three fresh Ed25519
samples have median **2,394,974 → 2,356,357** (1.6% fewer); certified median
**440,194,623 → 440,156,103**, effectively unchanged. This is synthetic fixture
evidence, not a universal production speedup. The per-byte array prototype was
rejected because decoding worsened; fixed byte strings were adopted.

Earlier #117 evidence remains in `target/cleanup-025-20261030/`, with older
dependency selections. Its 9,340-byte saving is a separate matched comparison.
Boxed immutable grants and borrowed checkpoint decoding remain rejected trials;
do not combine their figures or reopen Root-key/BLS changes from these results.

## Qualification and custody

Current-lock focused engine, protocol/Wire, issuance, restoration/revocation and
feature-unification tests pass, including strict signatures, replay, atomicity,
old-format refusal and request hashes. Eleven native host tests and compile-fail
docs pass. Selected all-target/all-feature, default, sessions-only and Wasm host
Clippy pass; dependency boundaries and focused native/Wasm Rust 1.91 checks pass.
Packed client checks and all **69** lifecycle/codec tests pass, with four
cross-language vectors. Generated source comparison, formatting and local links pass.

Final artifact passes installed certification/live authority, protected checkpoint
upgrade, composed roots and SDK signed ingress/admission/logout. Testkit **0.70.4**
and PocketIC **16.1.0** are explicitly prepared. Retain failed attempts and all
trial evidence. Full CI, macOS, actual Chromium and production acceptance remain
unclaimed; delivery and the complete release gate are still required.

No crypto algorithm, signed preimage, certificate/delegation check, TEE/non-TEE
trust assumption or result cache changed. Hosts own clock/protected authority,
generation, memory allocation, atomic commits and certification publication.
Older Memory allocation-ledger backing remains incompatible even with `AUTHNAT2`:
refuse without clearing; retain older binaries/states and require a reviewed
transition. Record-retirement consent grants no permission to erase that ledger,
reinstall or discard application backing. Canic owns endpoint/issuer/lifecycle
acceptance in [Canic #491](https://github.com/dragginzgame/canic/issues/491) and
browser issuance in [#507](https://github.com/dragginzgame/canic/issues/507).
No production wallet service, issuer adapter or consumer adoption is established.
