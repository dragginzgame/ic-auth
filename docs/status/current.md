# Current handoff — 2026-11-12

## Delivered state

Protocol and Engine **0.39.1** are published: `ic-auth-protocol-types`,
`ic-auth-wire` and `ic-auth` are non-yanked, with registry checksums matching the
retained archives and schema-2 receipts. Both tags and release commits are pushed;
Shared reconciliation fast-forwarded the normal checkout without discarding work.

- Protocol: `fe978d58840ed97ae6e5937932e5f8b9bf227986`, `protocol-v0.39.1`.
- Engine: `f825be33b99cc9a326fb3bd256c053a5ebfcc8cd`, `engine-v0.39.1`.
- Batch: `batch-BiswIN7d`, source `b0de07f468ca8362ea6430f72295e8a98e5522a3`.
- Registry readback: `target/auth-release-0391/registry/verified-receipts.json`.

That delivered source passed both complete 30-stage gates. The local changes
below have focused qualification, not a new release gate or publication. Product
versions remain 0.39.1; private qualification versions remain 0.36.1. Hosted CI is
asynchronous. Actual Chromium remains unprepared/ignored. No Auth service or Canic
deployment occurred. The maintainer separately authorized a scoped Storage
locality experiment; no Canic source or sibling delivery is included.

## Auth-owned indexed storage — #132 in progress

The optional `persistent` capability now owns durable receiver records and
issuer login/intent/renewal/result bookkeeping. Hosts supply checked memory leases,
clock/caller/authority, transactions and certification composition. Caller records
have their own table; each active binding is stored once, becoming a retained
fence when its session is removed. Issuer operations use point indexes, incremental
global/account/byte quotas and bounded reference-counted expiry cleanup. No routine
operation scans or serializes unrelated login history. See
[persistent integration](../integration/persistent-auth.md).

This is an uncommitted implementation batch, not completed #132 acceptance or
Canic adoption. Existing signed claims and public session/replay/login payloads
are unchanged. The new receiver backing schema is `0x41555448525302`; earlier
local prototypes refuse. Canic's reinstall-only approval does not retire another
consumer's retained records. The maintainer has removed the Storage-performance
hold: the measured 1.223M ordinary-call cost is acceptable for now. Publication
still requires the applicable correctness qualification and Engine release gate.

Current installed ordinary-authentication measurements (small records, first
stable-memory access charged to authentication) are:

| Retained receiver records | Instructions |
| --- | ---: |
| 1 | 291,773 |
| 1,000 | 645,904 |
| 10,000 | 982,875 |
| 100,000 | 1,223,407 |

The original 1M ordinary-call budget failed. The maintainer explicitly accepts
the measured cost provisionally; the regression now retains a 1.3M ceiling,
including after same-release upgrade. Published Storage 0.73.4 remains selected;
neither a Storage release nor a sub-1M result blocks this Auth batch. Storage's
16-byte inline-key limit and cold-page locality remain a separate optimization
investigation. The scoped allocation-order experiment is in Storage's checkout,
not an Auth dependency patch. This acceptance changes no authentication checks.

The focused installed rerun passes both 100,000-record scenarios on the published
Storage selection. Receiver authentication after same-release upgrade is
1,222,307 instructions, also below the provisional ceiling. The run retains the
longest nonce/approval completion-reservation fix. Both locks and the measured
Wasm hash remained unchanged through execution. Evidence:
`target/auth-132-installed/accepted-cost/` (build, scaling, hashes and retained
state). Formatting, diff checks and 254 local documentation references pass.

Installed issuer bookkeeping reached 100,000 logins: selected prepare 3,704,176,
renewal 2,570,094, maximum individual fill prepare 6,779,989 instructions. Its
10M bookkeeping regression passed. These probes exclude certificate verification,
Root approval, signing and production endpoint overhead. They are not matched
comparisons with Canic's older certified prepare measurements.

Evidence: `target/auth-132-installed/cold-scaling.log` (issuer pass, receiver
original-budget failure), `compact-scaling.log` (original-budget failure), retained
state/transport paths and artifact/lock identities in that directory. The earlier
`scaling.log` receiver samples ran after a diagnostic read and were warm; they
are explicitly unsuitable as cold-call acceptance. No crypto caching or relaxed
verification followed from these measurements.

Native checks passed ten persistence cases including 100,000 issuer logins,
account limits, malformed result refusal and reopening; the session/revocation
regressions passed, including real Ed25519, device replacement, logout and byte
refusal. Native library and fixture Wasm Clippy, graph policy and its caller
fixtures passed. The private checkpoint tests passed 13 cases. All five browser
files regenerate unchanged with incoming Wire 0.21.3; all 98 client cases passed.
Evidence is under `target/auth-132-*`. No complete release gate ran.

#132 also retains full issuer key/enrollment/effect custody convergence and the
installed certified/Ed25519 production-adapter lifecycle, corruption/interruption
and consumer handoff acceptance. Neither this fixture nor opaque signature-custody
samples establish those outcomes. Canic #764 owns its later adoption/deletion.

## Pending tooling adoption

Both workspaces select published Toko Tooling **0.39.17**. The authenticated
110-file snapshot preserves prior roots and adds Shared's `deps-update` helper.
It forwards scoped Cargo updates sequentially to both workspaces; no automatic
audit, CI prerequisite or manifest synchronization is added. The 0.39.11 LOC fixes
and 0.39.12 freshness-review rules remain adopted. Tool admission, both metadata
graphs and the actual scoped caller passed; dry-run and missing-workspace refusals
preserved both locks. Evidence: `target/tooling-adoption-20261112/`, with earlier
LOC/adoption proof under `target/tooling-adoption-03911/` and `target/tooling-adoption-03912/`.

The 0.39.17 adoption moves 16 module roots with children to `mod.rs`, preserving
public module paths. An empty session-test directory is removed; its standalone
integration test and fixture paths stay unchanged. Both lock updates
change only Toko relative to the incoming locks at adoption start. Installation evidence is
under `target/auth-release-0392/tooling/`. Protocol and Engine delivery is now
authorized for this batch; #132 remains open for its remaining acceptance scope.

Pre-freeze registry review confirms current direct first-party selections:
Build 0.83.0, Entropy 0.1.9, Host FS/Process 0.76.5, Metrics 0.29.11, Platform
0.31.3, Runtime 0.21.3, Types 0.19.5, Storage 0.73.4, Testkit 0.101.0, Wire
0.21.4 and Tooling 0.39.17. The newer Platform/Host/Wire selections were already
in the incoming locks; the accepted-cost measurements above predate them. The
release gate must qualify the captured graph. Wire 0.21.4 regenerates all five
browser files: four Auth bindings are unchanged; the shared runtime optimizes
u32 length decoding and packed booleans. The independently pinned Host file CLI
remains 0.76.4; defer its optional 0.76.5 refresh to the next Host/tool maintenance
batch. Evidence: `target/auth-release-0392/tooling/freshness.json` and retained
registry indexes. These observations do not replace release qualification.

## Pending qualification suppliers

Qualification selects Storage **0.73.4**, Testkit **0.101.0**, Entropy 0.1.9,
Metrics 0.29.11, Platform 0.31.2, Types 0.19.4, Wire 0.21.3 and one Runtime/
macros/guard 0.21.2. Host FS/Process remain current at 0.76.4. Compatible minimums
remain where the API contract is unchanged; Storage/Testkit requirements advance.
The product lock was preserved. Native Testkit selects Platform's management
codec and retains the upstream PocketIC re-export; public graphs exclude it.

Before incoming Wire 0.21.3 / Entropy 0.1.9, both metadata graphs, dependency boundaries and native/Wasm lint passed. All five
browser files regenerate unchanged; 98 browser cases and packed-client/Wire checks
passed. Native checkpoint/metrics/entropy/backend scenarios, installed qualification
(11 passed; optional Chromium ignored) and 12 feature-unification cases passed.
The final 13 checkpoint cases include a complete old SCL frame and foreign backing
headers, with unchanged bytes after refusal. Evidence: `target/qualification-upstream-20261112/`.

Storage 0.73's STORNAM1/STORCELL images replace earlier backing/cell formats. The
private fixture starts fresh and tests current-format upgrades; old images refuse
without reset or a compatibility reader. Auth's AUTHSLOT payload, session records,
signed bytes and trust model are unchanged. No production backing is retired here.
PocketIC pins qualification thiserror 2.0.18; the independent product lock selects
2.0.21. Cargo retains other transitive constraints; no product downgrade is imposed.
Testkit's new cached-snapshot path is not used by Auth's fresh-instance journey.
[#131 — Use Testkit's PocketIC re-export](https://github.com/dragginzgame/ic-auth/issues/131)
remains delivered/closed; earlier proof is under `target/testkit-0101-adoption/`.

## Bounded multi-device contract

Schema 2, introduced in 0.38, signs device slots 0–7 and nonzero generations;
schema 1 refuses. Eight devices can coexist per issuer-qualified account. Hosts
may choose a tighter persisted limit. Renewal preserves the binding and original
window; verified higher generations with fresh keys replace a slot atomically.
Stale rebinding and wrap refuse. See [sessions](../protocol/sessions.md) and
[Canic adoption](../integration/canic-adoption.md) for the complete contracts.

- Known slot fences survive logout and expiry; pruning removes expired sessions,
  replays and unknown denials, never known fences. Account retirement is host-owned.
- Durable admission deletes `login.displaced_caller()` from old session/login rows
  atomically with new rows/indexes/totals, retaining replay history. Failed commits
  preserve the old device; old-key logout cannot remove its replacement.
- Replay rows retain `(issuer, subject)` independently of sessions. Account quotas
  survive logout/key changes/restore; corrupt associations refuse restoration.
- The maintainer authorized retiring older Auth records. Current session/replay/
  login IDs are `0x415554485341`, `0x415554485342`, `0x415554485343`; checkpoint
  identity is `AUTHSLOT` / `0x41555449f102`. Old profiles refuse without clearing
  bytes. This never authorizes deleting Canic's protected safety state.
- Ninety-day login windows use short proofs and 30-minute receiver leases. Renewal
  cannot extend the original window; lost protected state stays closed.

## Canic and native boundaries

The last read-only Canic inspection observed Engine 0.36.1 / Protocol 0.36;
this is historical evidence, not current adoption status. Coordinated suppliers,
protected issuer slot/generation custody and callback fences, durable receiver
adapters, retained-state decisions and installed consumer acceptance remain owned
by [Canic #491](https://github.com/dragginzgame/canic/issues/491). Auth #130's entropy
contract was delivered; Canic #757 owns its adapter. No production issuer or
stable session-backend adoption is established by this fixture.

Native Testkit retains managed startup, Wire helpers and PocketIC 16.1.0.
Principal boundaries copy validated bytes. Upstream panics propagate; its 30,000 ms
polling setting is not an absolute HTTP/body deadline. Awaited deletion remains
best-effort with retained logs. Cryptography stays supplier-owned; Auth's checking
policy is unchanged. Build remains 0.83.0 and the file CLI remains 0.76.4.

## Evidence and recovery custody

The first 0.39.2 candidate `batch-XoEWGnOX` stopped before publication at the
package-admission fixture's obsolete all-features Storage refusal. Native/Wasm
Clippy, actual dependency graphs and the first ten gate stages passed. The
fixture now admits the optional backend while explicitly refusing executor
guard/macros; its focused identity/isolation/lock cases pass. Preserve the failed
candidate and `target/auth-release-0392/delivery.log`; corrected source needs a
fresh candidate, not mutation of that captured source.

Keep `target/release-frozen/batch-BiswIN7d/` and durable
`.git/release-state/frozen/batch-BiswIN7d/`, including `repository.git/publication-state/`.
Its gate roots are `target/release-validation/protocol-gate.pe0whn/` and
`target/release-validation/engine-gate.APrYjl/`; log: `target/auth-release-0391/delivery.log`.
Both delivered gates passed all 30 stages. No batch or recovery evidence was retired.

Keep prior `batch-t5q4t2DE`, preflight-only `batch-1JDi3jNF`, older `batch-sat4DSEt`,
failed `batch-4hebGSFP` / `batch-yiX0dEoV` custody and `target/publication-recovery/`.
The preflight-only batch stopped before validation/version preparation/delivery;
corrected committed source shipped through batch-BiswIN7d. Do not resume earlier
failed attempts against already published versions.

Earlier identities, scoped measurements and qualification narratives remain in Git
at `8fd2cbc1f0e073c01b81db9756d4fbae0298cc65` and under `target/upstream-followup-20261111/`.
Keep earlier `target/device-contract-*`, `target/upstream-20261111/`,
`target/platform-0300-adoption/`, `target/wire-0210-adoption/` and `target/tooling-adoption-0398/`.
An external cleanup deleted some old raw evidence; historical measurements cannot
replace those bytes. [History](history.md) retains older observations with their
original scope. [Setup](../development/setup.md) and [the docs index](../README.md)
own current command/topic entrypoints.
