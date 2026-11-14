# Current handoff — 2026-11-14

## Current supplier adoption (unpublished)

Both workspaces now select Platform **0.32.2**, canonical Types **0.20.2**,
Storage **0.74.0** and Wire **0.22.1**. Qualification selects one Runtime
**0.22.2** executor/generator/guard and Testkit **0.104.0**. The published
`ic-testkit-wire` **0.1.0** adapter resolves the former Wire generation blocker:
Testkit owns the upstream PocketIC client and lifecycle; the adapter owns typed
Wire calls. Auth passes canonical method IDs and every directional limit directly.
No aliases, codec bridges, limit projections or custom transport were introduced.
The adapter is allowed only in the private runner and excluded from product graphs.

Native and Wasm compilation and strict Clippy pass. Declaration and selected
native/Wasm dependency boundaries pass, including the single host executor/guard.
All five generated browser files were regenerated and reviewed: no byte changes.
All 98 browser-client tests pass, including Rust/TypeScript Wire interoperability.
Client validation found and fixed a stale SDK-gate import in the consumer-owned
package checker after the tooling vendor migration. Evidence: `target/upstream-1114/`.
All 279 product tests, 32 backend-conformance tests and five installed
certification/upgrade/ingress cases pass. Three more installed tests pass:
ordinary persistent authentication through 100,000 records and both Ed25519
and certified-proof persistent lifecycles. Testkit setup/CI caller fixtures,
dependency pins, formatting, local links and diff checks pass; both locks stayed
stable through qualification. The Testkit 0.104 CLI and PocketIC 16.1.0 server
are explicitly prepared and the reference Wasm was rebuilt.
Public Principal/Wire Rust identities change; pending Protocol/Engine notes record
that migration. No product versions, stored formats, verification checks, signed
bytes, trust model, sibling source or release/publication effects were changed.

Toko Tooling **0.43.0** is installed as 105 canonical files, with executable
payloads under `vendor/toko-tooling/` and an explicitly selected registry-state
helper. Its vendor migration, license fix, CDPATH-safe hook installation,
formatting preservation, release dispatch/retention and setup/evidence callers
passed in `target/tooling-0421/`, `target/tooling-0422/`, `target/tooling-0425/`
and `target/tooling-0428/`. The local hook uses `vendor/toko-tooling/.githooks`.
The 0.43 adoption replaces the retired Make LOC interface with the installed
`toko-cloc` PATH launcher. Both workspace reports, snapshot/member admission,
license admission and release dispatch pass; only Toko Tooling changed in either
lock. Evidence: `target/tooling-0430/`. Deployment policy is unchanged.
The Host file CLI **0.77.1** is explicitly prepared and admitted. Build **0.84.1**
remains selected with its prepared release editor; host-tool tests and mocked
release/recovery qualification passed in `target/upstream-next/`.
Entropy **0.3.0** remains confined to private backend tests; its five cases passed
in `target/entropy-030/`. Incoming Metrics **0.30.5** passed five native tests.
Historical release receipts and prior failure evidence are retained.

## Unpublished compiler and warning cleanup — #135

Rust **1.100.0** is the package minimum and pinned compiler. Product, native
qualification and Wasm Clippy/Rustdoc pass with warnings denied and no
`missing_docs` allowance. Public APIs, integration tests, private fixture APIs
and generated Wire contracts are documented; projections link to domain owners.
Six Auth debt baselines are retired. Actual Rustdoc refusal replaces baseline
filtering; the shared parser remains installed but unused. Fresh/warm missing-doc,
broken-link, unsafe/must-use and unused-catalog rejection fixtures pass.
Evidence: `target/clippy-plain/`, `target/clippy-cleanup/`, `target/tooling-0410/`.

Earlier compiler qualification at `target/auth-135/` passed archive/MSRV features,
273 product tests, 52 native qualification tests and all 15 installed cases.
Those earlier results are not a full gate for the current incoming selections.
Chromium remains unprepared; macOS and the current full CI/release gate were not
run. Pending notes target Protocol 0.40.0 and Engine 0.41.0; versions are not bumped.

## Unpublished cycle-drain hardening

Hard cuts before 1.0 are the maintainer default. Receiver schema
`0x41555448525304` and issuer `0x41555448495303` refuse old backing without an
implicit migration/reset. Persistent receivers require protected verification
weights; volatile hosts use `SessionAdmissionBudget::for_verification`.
Operation-only buckets refuse proof verification. Issuer configuration requires
`issuance_burst` / `issuance_refill_interval_ns`, shared by new issuance and renewal.
Creation starts the issuer bucket full; reopen/restart preserve credit. Retained
exact recovery is uncharged; later ordinary errors retain attempted-work charges.

Same-clock exhausted admissions and exact retries perform zero backing writes.
Browser-caller mismatch refuses before persistent index reads. The 1,024-session
native probe observes zero backing reads/writes at the same clock; this is not an
installed instruction/cycle ceiling. Signed bytes, cryptographic checks, original
login deadlines and trust assumptions are unchanged. Canic must adopt the
[new configuration/refusal contract](../integration/canic-adoption.md#pending-auth-owned-persistent-store-adoption).

Earlier focused evidence: `target/auth-admission-followup/` (254 product cases,
then 34 unit/seven receiver follow-ups, 27 backend cases, five entropy/five metrics
cases and both installed persistent lifecycles). Preserve `target/auth-budget-hardening/`
and `target/auth-cycle-audit/`. The audit observed 2.36M instructions for invalid
Ed25519 and 453.17M for late certified failure; these are samples, not upper bounds.
Production/fleet budgets and populated persistent failure costs remain unqualified.
See the [session contract](../protocol/sessions.md) for limits and host obligations.

## Published delivery and Canic boundary

Engine **0.40.1** and Protocol **0.39.2** are published. Engine tag `engine-v0.40.1`,
commit `3ef463c11995e816fb69f25466e5cd6eb6fe8666`, batch `batch-jJwnO2R3` is reconciled.
The prior full/reduced deliveries took 27m15s / 13m17s; #133 is complete. Evidence:
`target/auth-release-0400/`, `target/auth-release-0401/` and retained batch receipts.
Hosted CI is asynchronous; the current work still needs its source-bound release gate.

Published credentials allow 32 total scope entries across at most 16 roles,
guaranteeing at most 3,332 canonical grant-list bytes; the defensive cap is 4,096.
Completed old results remain recoverable; new oversized issuance/admission refuses.
Engine 0.39.3 delivered browser-authenticated renewal recovery, renewal preflight
and bounded session inspection. [Auth #132](https://github.com/dragginzgame/ic-auth/issues/132)
is complete; [Canic #764](https://github.com/dragginzgame/canic/issues/764) owns consumer
adoption and duplicate-engine deletion. No Canic adoption is inferred here.

The accepted cold authentication sample is approximately 1.223M instructions at
100,000 records, with a 1.3M ceiling including same-release upgrade. Storage locality
work is not a prerequisite for Canic. Preserve `target/auth-132-installed/accepted-cost/`
and `target/auth-release-0394/delivery-corrected.log`. The private conformance bundle
0.36.2 targets published Engine 0.40.1 / Protocol 0.39.2; its receipt remains under
`target/release-frozen/batch-jJwnO2R3/target/portable-fixtures/backend-package.khXkRq/`.

## Recovery custody

Preserve every `target/release-frozen/` candidate, `.git/release-state/frozen/`
record and publication receipt. Do not resume attempts for versions already published.
This includes stopped `batch-qaK3IdsJ`, `batch-XoEWGnOX`, `batch-5x63tbsF`,
`batch-H6iT55MB`, `batch-HL6ZrkFz`; completed `batch-kCAsAk34`, `batch-IAAjfLaF`,
`batch-6Eztyvd6`; and prior `batch-BiswIN7d`, `batch-t5q4t2DE`, `batch-1JDi3jNF`,
`batch-sat4DSEt`, `batch-4hebGSFP`, `batch-yiX0dEoV`, `target/publication-recovery/`.
Earlier evidence inventories remain in [history](history.md) and Git at
`0bcf7ac667f693e874a00c6550b3b965359d7ffc`, `fc1ee1e8` and `20384f9`.
Keep referenced evidence; narrative measurements cannot replace logs lost to cleanup.
