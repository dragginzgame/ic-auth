# Current handoff — 2026-11-10

## Toko Tooling 0.39.7 and documentation layout

Both workspace pins and Cargo locks now select published `toko-tooling` 0.39.7
(`fd795f6ef878496ac27a990a9208b87fa786a9148c18580338ef8ac79df501d7`).
Auth-owned docs now live under protocol, development, integration and performance
topics, alongside the existing design/status directories. [The index](../README.md)
links the entrypoints. Relative links and consumer-owned command references were
updated. The canonical root [LICENSE.md](../../LICENSE.md) and
[CONTRIBUTING.md](../../CONTRIBUTING.md) match the selected producer byte-for-byte;
both workspace catalogs inherit the root `license-file`. Required source notices
remain in the packaged READMEs and generated private client archive.
Only that package changed relative to the incoming lock selections; pending Auth
device-slot work and the incoming Platform selection are preserved. Payloads came
from authenticated crates.io archives located through Cargo metadata, installed by
the new producer helper. The canonical exporter retired managed reference docs,
rules, audits and task catalogs under the maintainer's explicit no-docs selection.
Unused maintenance runner/schedule templates were also retired; no schedule was
enabled. Auth's required `scripts/ci/release-receipts.sh` is now an explicit root,
since its former documentation root no longer selects it. No snapshot-owned files
were edited manually. Auth-owned documentation is retained.

Auth explicitly opts into the maintained release surface with
`TOKO_TOOLING_RELEASE := 1` before `make/consumer.mk`. Consumer links use upstream
URLs pinned to reviewed source revisions. The task catalog now selects
`fcbbcd147678fdfe5ab22c36710e017fb2c018e4`; Auth's forwarding `make tasks` target
was removed because that command belongs only to the producer. Do not create `docs/toko-tooling/`;
0.39.5 replaces that earlier policy with online references. Existing Protocol/Engine
release requirements remain. Current tooling evidence is under `target/tooling-adoption-0397/`;
the previous package/client and documentation-move evidence stays under 0396;
the prior documentation-retirement evidence stays under its original 0395 directory.
The preceding 0.39.3 release-plan regression remains under its original evidence
directory; it is not relabeled as a new execution. No sibling edits or delivery.
Installer/tool admission and the final `make toko-tooling-check` passed: 108
installed files, with no managed `docs/`, `rules/`, `audits/` or `tasks/` payloads.
The 0396 canonical-license/archive checks and 97 client tests retain their original
evidence identity. This adoption preserves those files and package mechanics.
The earlier Make jobserver regression passed after marking the metadata recipe as a
recursive Cargo caller and extending its substituted metadata response. All 247
local documentation references resolved in that batch. Saved release-validation records retain
their original paths and evidence; they are not rewritten for these source moves.
For 0397, installer/tool admission, snapshot and both-workspace metadata checks,
canonical-file byte comparisons, all 247 documentation links, release dry-run and
the final Make jobserver/frozen-routing fixture passed. The fixture now asserts
the producer-only `tasks` target is absent. No reference docs were reinstalled;
no unrelated package selections changed.
Only Tooling changed in either lock relative to the incoming selections.

## Pending bounded multi-device contract

Local Protocol/Engine **0.38.0** work replaces the rejected one-device restriction.
Nothing in this batch is committed, pushed or published. Canic remains unchanged.
Do not claim downstream adoption or a release gate from focused qualification.

- Portable claims use canonical schema 2 with signed `device_slot` (0–7) and
  nonzero `device_generation`. Schema 1 refuses. Issuers allocate slots under
  account authentication, preserve the original binding/window on renewal, and
  advance generations with fresh browser keys for replacement. Auth does not
  implement Canic issuance, asynchronous callback fencing or fleet orchestration.
- Receivers retain one known record/session per `(issuer, subject, device_slot)`.
  Eight is the protocol ceiling; `MemoryLoginRevocations::with_device_limit`
  admits a tighter host limit before admission/restore. Persist that policy.
  Higher generations require complete verification before atomic replacement;
  lower/equal-generation rebinding and generation wrap refuse. Failed commits
  preserve the original device. A late logout by a displaced key cannot remove
  the replacement or another device.
- Known slot fences survive logout and expiry. Pruning removes bounded expired
  session/replay records and unknown denials, never known slot fences. Global
  retained-slot capacity bounds accounts; safe account retirement/reset remains
  explicit host policy. There are no hidden timers or new inter-canister calls.
- Durable admission must delete `login.displaced_caller()` from old session/login
  rows in the same commit as new records/indexes/totals, retaining replay history.
  That accessor is transaction-only and is absent from encoded checkpoint data.
  Both reference and independent encoded backend qualification implement it.
- Replay rows retain `(issuer, subject)` independently of sessions. Account-wide
  replay quotas survive logout/key changes/restore. `SessionReader::occupancy`
  accepts the account tuple; projections are `max_replays_per_account` and
  `replays_for_account`. Corrupt session/replay ownership refuses restore.
- The maintainer authorized retiring the old Auth format on 2026-11-10. Session,
  replay and login identities are now `0x415554485341`, `0x415554485342` and
  `0x415554485343`; the fixture checkpoint is `AUTHSLOT` / `0x41555449f102`.
  Old records/checkpoints refuse without deletion or a fallback reader. This
  does not authorize deleting Canic data. Public Wire envelopes and upstream
  cryptographic algorithms/validation remain unchanged; signed claims change.

Qualification evidence is under `target/device-contract-*`. The original
installed run passed four cases and failed the newly extended upgrade scenario
because the second receiver had not enabled checkpointing. After explicit setup,
that case passed, including replacement/old-key refusal across upgrade. The final
rebuilt artifact then passed all five installed cases together (34.38 seconds).
Actual Chromium remains unprepared/ignored. The browser suite passed 97 tests; all five
regenerated browser contract files remain byte-identical because claims are an
opaque blob. Final focused native checks passed 152 Auth cases, 21 backend cases
and 12 adversarial feature-unification cases. Native/Wasm Auth Clippy, all-target
qualification Clippy, tooling bench compilation, documentation links and diff
checks passed. Both Cargo locks stayed unchanged. Full CI/release was not run.

The retained pre-change fixture is 1,859,049 bytes. The first device-slot artifact
was 1,886,557 bytes; the final artifact is 1,886,577 (+27,528 / 1.48%), SHA-256
`6ee1f0cafc5efca36f09105fa8c0213a171c90f1bfeac829220f61f4ac222b7a`.
Installed authorization measured 4,676
instructions, stored session decode 21,768, record size 267 bytes. These are
fixture observations, not production fees or a matched published-release comparison.
The change includes account-aware replay records. Preserve failed logs as evidence.

Canic read-only inspection found Engine 0.36.1 / Protocol 0.36 and caller-based
replay accounting. Its local issuer ceiling increased to 4,096 logins / 2,048
issuances, but admission remains globally bounded by caller+epoch+login ID, without
account slot allocation. Adoption needs coordinated supplier identities, issuer
slot/generation custody with retry/callback fences, receiver adapter changes,
explicit retained-state disposition and installed consumer qualification.

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
Platform 0.29.2**, made outside the captured batch, plus incoming CRC32fast 1.5.5.
The Platform update was preserved on local
branch `deferred/auth-0370-platform-0292` at `3a7e24f`, then reapplied after
reconciliation without altering the released package versions. It is **not**
part of 0.37.0; the focused pending-code checks do not establish complete supplier
adoption or release qualification. The qualification lock remains
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
