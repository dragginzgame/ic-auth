# Auth release validation

Auth adopts Shared Tooling's [group-plan contract](release-group-validation.md).
`ci/release-validation.json` owns the complete ordered CI roster and the two
groups must match `ci/cargo-release-groups.json`. Ordinary `make ci` executes
every stage. Release verification selects a source-bound plan rather than invoking
that aggregate unconditionally. No prior whole-gate receipt or stage is reused.

## Selection

The baseline is the selected group's exact previous annotated release tag.
Missing baseline history selects the full roster; a foreign baseline refuses.
Shared observes the annotated baseline and changed paths; Auth additionally
requires its exact group release identity and applies the product mapping.
The selector considers every changed path since that baseline, including changes
already outstanding before the immediate implementation batch. It does not infer
scope from an unchanged package version.

Only Rust source/tests in Protocol's two public packages can select the Protocol
mapping. Engine's mapping admits Rust source/tests in `ic-auth` and its private
qualification packages. The selected release ledger and current handoff may
accompany those changes. Every other path selects the full roster: manifests,
locks, features, generators, browser files, tooling, scripts, Shared snapshots,
other owners and unknown inputs. Changes spanning both groups are full scope.

| Source-only group | Excluded obligations | Retained obligations |
| --- | --- | --- |
| Protocol (29 stages) | Unchanged release/publication caller fixtures. | Complete Clippy, MSRV, graph, browser, native tooling, crypto/session, installed canister/ingress and all local archive checks. |
| Engine (27 stages) | The above release fixtures, plus passive Protocol type/transport unit tests that have no engine dependency. | Complete Clippy, MSRV, graph, browser, native tooling, crypto/session, installed canister/ingress and all local archive checks. |
| Shared or uncertain | None. | Every ordinary CI stage, in the same order. |

`ci/release-validation-coverage.json` records a reason for each exclusion. Unknown
groups/coverage targets refuse. A new canonical stage is required by default;
there is no automatic omission for new checks. Public identity or format changes
retain their affected integration and archive tests; the mapping removes only
checks whose unchanged owners cannot consume that group's Rust source.

## Source custody and receipts

Shared admits canonical plan/policy JSON and seals the selection, recipes and
context. Auth's context records every tracked regular source file's exact bytes,
baseline identity, changed paths, compiler/Cargo, native host and build environment.
The stage wrapper re-observes that entire inventory before and after each command,
alongside whole-checkout Git admission. Added untracked source and persistent
drift refuse. The only permitted pending ledger is separately checkpointed.
This batches the source inventory without spawning a checksum process per file
at every stage. It neither projects Cargo versions nor weakens tool admission.

Schema-4 release receipts carry Auth's selection and original execution context,
bound to Shared's opaque admitted run proof. Shared owns the canonical schema,
stage/event/timing checks and log hashes; Auth has no parallel run parser.
Required stages must finish PASS; excluded stages must remain not-applicable.
Missing, changed, failed or interrupted evidence refuses
preparation. Recovery recomputes the original selection against exported saved
source and original notes, checks host/configuration and re-admits the retained
proof bytes using Shared's file reader and hashes. Prepared version metadata is
not projected into the original run or interpreted as a new execution. A failed
verify attempt keeps its gate; a retry creates a fresh one.
After the exact release is committed, publication can verify that self-contained
record against the original source/notes in a fresh clone. It records the original
host and build context, not a new execution on the publication host. Fresh release
verification and unfinished recovery still require their retained raw evidence.
Historical schema-2/3 releases keep their original adapters and frozen checkouts;
their receipts are never reinterpreted as the new evidence envelope.

CI manifest discovery projects the complete admitted manifest lists from Shared's
Cargo group plans. That owner admits actual workspace membership and unequal
group versions. Auth seals discovery and every returned member before dispatch;
it does not maintain another Cargo manifest inventory implementation.

Fresh source/group/index and editor admission, exact local archives, unselected
versions/ledgers, publication order and all delivery permissions remain required.
The mapping does not grant commit, push, publication or sibling authority.

## Qualification

`make test-release-validation` runs the actual Auth selector, Shared runner and
evidence admission with substituted stage bodies. It covers both mappings,
complete CI routing, shared expansion, required/excluded failures, source/input
drift, stale plans, unknown groups and tampered evidence. `make test-release-tools`
adds real metadata rendering and release/resume/publication custody using local
Git/bare remotes and substituted execution/upload effects. For changes limited
to validation receipts, `bash scripts/release/test-tools.sh --validation` selects
the actual metadata, interruption/resume and fresh-clone publication callers
without replaying unchanged setup and registry response matrices. The complete
CI target retains those matrices.

Explicit workflow dispatch with `qualify_release_groups=true` runs
`bash scripts/release/qualify-validation.sh` after each native job's complete
gate. Ordinary CI retains one full gate. The comparison executes both selected mappings
on that same clean source, using an explicit zero-delta qualification baseline.
Its context uses the root Makefile's checkout-local Cargo target from the start,
including when the helper is invoked directly without a target environment variable.
The source-selection cases above separately exercise actual owner/shared deltas.
This qualification baseline is never supplied by release verification, and its
context cannot satisfy release receipt re-admission. The full gate seals its
starting source and clean-worktree status; comparison refuses another HEAD,
dirty-source evidence or a changed attestation. The successful artifact
retains full and selected logs, timings, contexts and comparisons on Ubuntu,
macOS Intel and Apple Silicon. Failed comparisons retain normal failure evidence.

Qualification observations belong in the [current handoff](status/current.md).
Substituted dispatch tests are not actual Rust/canister or native-host proof.
The first adoption batch changes shared inputs and therefore selects full scope.
Matched same-source stage timings are recorded in the handoff.
