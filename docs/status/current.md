# Current handoff — 2026-11-09

## Published state

**Engine 0.37.0** (`ic-auth`) and **Protocol 0.37.0**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published, non-yanked and verified
against the exact Cargo archive SHA-256 values. Both release commits and annotated
tags are pushed to `origin/main`; Shared's reconciliation fast-forwarded the
normal checkout to the delivered source.

- Protocol commit: `c5dc307bb46919e8fb0020b9a9b64ae90d5c4aee`;
  tag `protocol-v0.37.0`, object `48ed4fb8af60d94c54ee0a415ac5872f708e857e`.
- Engine commit: `a97afcfcd2040603ccc9ca129f83e2341e339d06`;
  tag `engine-v0.37.0`, object `2feedb1c1d72e8be3bdebcb029c7b8994f36c848`.

| Published package | Registry SHA-256 |
| --- | --- |
| `ic-auth-protocol-types` | `c8d89ab1a7d742a9a8df6a663a67a9e22c0bbb74d6ac8767dd57626d651d0adb` |
| `ic-auth-wire` | `8c8a2bf68b85bff966a31ef98f3ea4e9690326dae64da4f4c777c9fcfe69a1a2` |
| `ic-auth` | `f6588c9791d8233dd79ab97d2ff6c73c2ca75ce75b2e10bf42fb59d616c050e5` |

Both source-bound 30-stage release plans passed, including native/Wasm Clippy,
isolated compiler/package checks, both-lock preparation/rollback, source and
prepared-input drift refusals, all crypto/session/browser obligations, actual
installed certificates/composed roots/upgrades/signed ingress/SDK logout, and
release/recovery/publication caller fixtures. Independent crates.io and remote
ref readback agree with the publication receipts. Hosted CI runs asynchronously;
its result is separate from these completed local release gates.

## Delivered contracts and dependency cohort

Both public groups take the documented Rust identity cut to canonical Types 0.19
and Wire 0.19. Platform 0.29.1, Runtime/macros/guard 0.21.0, Testkit 0.97.0,
Storage 0.71.1, Metrics 0.29.9, Host 0.76.2, Build 0.81.1 and SHA-2 0.11.1 are
selected by the delivered locks. The host has one Runtime/executor/guard and
Testkit shares the same Wire/Types/Platform identities. The previous native Wire
mismatch is resolved. Agent/delegation boundaries explicitly copy validated
Principal bytes; redundant conversions at owned PocketIC boundaries are removed.
Signed bytes, method identities/budgets, verification policy and current stored
profiles are unchanged by this supplier/type cut. Four generated Auth browser
contracts were unchanged; Wire's shared runtime was regenerated and qualified.

Toko Tooling 0.38.2 is selected exactly in both workspace catalogs/locks; its
176-file authenticated snapshot retains 87 entrypoints. Four product packages
remain under `crates/`. Six private packages now own an independent
`qualification/` workspace/catalog/lock, outside both release groups. Private
versions remain 0.36.1; Shared owns atomic selected-product preparation of both
locks and preserves those versions. Product graphs have no reverse qualification
edge. Auth retains source-bound validation and publication/recovery custody.
The release includes reference-store/pruning/signature-tree cleanup and #130's
private host-entropy conformance work. Crypto remains upstream-owned.

## Deferred normal-checkout update

The normal checkout retains an **uncommitted product Cargo.lock update to
Platform 0.29.2**, made outside the captured batch. It was preserved on local
branch `deferred/auth-0370-platform-0292` at `3a7e24f`, then reapplied after
reconciliation without altering the released package versions. It is **not**
part of 0.37.0 and has no qualification claim. The qualification lock remains
on delivered Platform 0.29.1. Do not discard this deferred work.

## Persistence and consumer boundaries

The published host-entropy contract and five dev-only `ic-entropy` conformance
cases cover real signing/admission/replay, fail-closed providers, seed reuse and
zero-nonce policy. Auth #130 is delivered in this cohort. Canic #757 owns the
production entropy adapter; no composed Canic acceptance is claimed.

Storage 0.71 selects bucket backing only in the private host. Use fresh fixture
backing; retired allocation headers refuse without clearing. No production
backing or `AUTHCLK1` payload was migrated. The approved browser journal cut is
`ic-auth-login-cleanup-journal-v1`; unknown issuance remains durable. The prior
no-retained-record decisions never permit erasing Canic's protected safety state.
90-day login windows still use short proofs and 30-minute receiver leases;
renewal cannot extend the window, and lost protected state stays closed.

Canic's integration, configuration, deployment hold and final installed acceptance
remain owned by [Canic #491](https://github.com/dragginzgame/canic/issues/491).
The last read-only review was HEAD `3700c6e1bf78cb04a60dcfaf0f789d4dd992c9e0`
plus dirty inputs; it found no Auth API gap. This release changed no Canic source.
No standalone Auth service is deployed. The private backend helper keeps its
independent version; registry-consumer bundle qualification remains opt-in.
Do not treat the historical 0.35.8 bundle as a delivered 0.37 helper archive.

## Custody and cleanup incident

Completed `batch-yiX0dEoV` captures source `7aec8a1` and delivered both groups.
Keep `target/release-frozen/batch-yiX0dEoV/` and durable
`.git/release-state/frozen/batch-yiX0dEoV/`, including both schema-2 publication
receipts under `repository.git/publication-state/`. Gate evidence is:

- Protocol: `target/release-frozen/batch-yiX0dEoV/target/release-validation/protocol-gate.RO5uB9/`.
- Engine: `target/release-frozen/batch-yiX0dEoV/target/release-validation/engine-gate.jCdJo8/`.
- Recovery/readback: `target/publication-recovery/`.

An external cleanup removed the entire earlier Auth `target/` during validation,
after a disk-full failure. It also removed prior managed checkouts, archives and
raw observations. Cleanup is now paused. The durable Git store survived; the
same saved batch was restored without replacing its source or journals, tools
were explicitly prepared, and both complete gates ran afresh. Lost raw evidence
was never reused or relabelled as a pass. Historical observations and checksums
are retained in [history](history.md), but deleted raw bytes cannot be independently
verified. Keep older durable batch/publication custody; do not resume failed
`batch-Z6H_KgCz` or earlier attempts against delivered versions. No batch was
retired and no recovery state was deleted by this release.
