# Current handoff — 2026-11-12

## Delivery

Engine **0.39.4** is published; Protocol remains **0.39.2**. The downloaded
registry archive matches the retained archive/schema-2 receipt. Engine tag
`engine-v0.39.4` and commit `8495375964647d00306b067abfaee0cde41d4604` are
pushed. Source: `39aa37b660cd941dc73fdf075e096a18586e20dd`.
Batch `batch-kCAsAk34` passed all 30 stages and reconciled into the normal checkout.
Evidence: `target/auth-release-0394/`, including timestamped delivery output,
registry archive/readback and reconciliation. Fifteen installed cases passed;
Chromium remains unprepared/ignored.
Hosted CI is asynchronous. No Auth service, Canic source, production backing or
sibling deployment changed. Canic can now adopt the three missing APIs below.

## Delivered changes and measurement follow-up

The #134 source delivery adds optional `persistent` conformance in private
bundle **0.36.2**, at `e9e831598d2b02db5e8f7fea883616953c7950ee`.
Its `ReceiverHost` uses checked leases,
actual backing reopen and host pre-commit rejection; no private ledger export.
Six persistent cases (including five deliberate broken-commit drivers), strict
Clippy and the archived registry consumer pass. The existing 26 bundle tests
also pass against registry Engine 0.39.4 and Protocol 0.39.2. Clean archive and
`delivery.json`: `target/portable-fixtures/backend-package.u5EztD/`; the archive
is under `target/package/ic-auth-backend-conformance-0.36.2.crate` there.
SHA-256: `764d3708796384d789413baf3c910f79f6f48026157646d2495744fa635d31f6`.
Canic #764 owns adoption; no sibling files or production code changed. This is
Git/source-bundle delivery only: no publishable product changed, so no Engine
bump, crates.io upload or frozen product gate was needed. #133 remains pending.

Engine 0.39.4 preserves the incoming Storage **0.73.6** locks and requires its discard
APIs. Prepared Auth commits avoid materializing unused old values. Eighteen native
tests, strict focused Clippy, the two 100,000-record installed probes and both real
certified/Ed25519 lifecycle tests pass. Formats and trust checks are unchanged.
Matched evidence: `target/auth-132-discard/` (before/after Wasm, locks and logs).
The qualification Wasm is 420 bytes smaller; lookup locality remains unchanged.
Detailed matched timings and the initial failed sandbox bind are retained in the
evidence above and the delivered handoff at `4938916`.

[Release selection](../development/release-validation.md#deployment-path-observations--2026-11-12)
now excludes only the unchanged selector regression for reviewed product/data
changes. Mapping/refusal, ordinary publication/recovery and frozen two-group/native
archive callers pass; snapshot integrity passes. Evidence: `target/auth-133-release/`.
The corrected release command took **25m52s** to verified registry availability:
about 87s preparation, 1,367s validation, 67s metadata/Git delivery and 30s
publication. All stages executed; infrastructure changes required full scope.
[#133](https://github.com/dragginzgame/ic-auth/issues/133) remains open for an
eligible deployment exercising the omission; no deployment saving is claimed.

## Indexed persistence — Auth implementation complete

The optional `persistent` capability owns receiver session/replay/fence records
and issuer intent/login/renewal/result bookkeeping. Hosts retain checked memory
leases, trusted caller/time/authority, atomic transactions and certification
composition. Caller sessions have a separate point table; maintenance uses
incremental quotas, indexes and bounded reference-counted expiry cleanup. No
routine operation scans or serializes unrelated login history. See
[persistent integration](../integration/persistent-auth.md).

The receiver schema is `0x41555448525302`; earlier local prototypes refuse.
Signed claims and existing public session/replay/login payloads are unchanged.
Canic's reinstall approval does not retire other consumers' records. Storage
0.73's backing/cell formats refuse older images without reset or a reader; no
production backing is implicitly retired. Library persistence selects Storage's
Wasm Runtime IC0, never an executor/guard/CDK; the host owns its one runtime.

The maintainer accepts roughly **1.223M** instructions for cold ordinary
receiver authentication at 100,000 records for now. The regression ceiling is
**1.3M**, including after same-release upgrade. The published Auth 0.39.3 APIs
do not require Storage locality work or a sub-1M result for Canic adoption.
This changes no verification checks or trust assumptions. No verification cache,
inter-canister call or cryptographic replacement was added.

The 0.39.4 gate measured cold authentication **1,223,850**, post-upgrade
**1,223,195**, renewal preflight **507,729** and recovery **643,861** instructions
at 100,000 records. Issuer prepare was **3,712,362** and renewal **2,503,057**.
Evidence: `target/auth-release-0394/delivery-corrected.log`, gate `engine-gate.mXTSu4`
in the captured checkout's `target/release-validation/`.
Bookkeeping probes exclude initial proof verification, Root approval, signing
and endpoint overhead. The 100,001st issuer login refuses without changing occupancy.
Keep the earlier `target/auth-132-installed/accepted-cost/` evidence. Warm samples in
`target/auth-132-installed/scaling.log` are not cold-call acceptance.

### Delivered #132 APIs and consumer handoff

`IssuerStore::retrieve_renewal` now provides browser-authenticated lookup-only
recovery, including expired/interrupted originals. `preflight_renewal` projects
the authenticated account, exact request and original policy before application
authorization. Both share the private renewal index/binding checks with `renew`;
neither reserves work or advances time. `ReceiverStore::inspect_sessions` returns
passive expired/stale metadata in caller order, with an exclusive cursor and a
bounded scan across fences. Empty pages can have continuations. See the
[consumer contract](../integration/persistent-auth.md). Stored formats, signed
bytes and verification/trust rules are unchanged. Canic retains host keys,
Root enrollment/approval and application policy; no additional Auth key store
or consumer renewal index/session mirror is required.

Keep focused evidence and input identities in `target/auth-132-apis/`;
the complete release gate qualifies the incoming Wire 0.21.5.

[Auth #132](https://github.com/dragginzgame/ic-auth/issues/132) owns the completed
Auth implementation and published discard follow-through.
[Canic #764](https://github.com/dragginzgame/canic/issues/764)
owns adoption, duplicate-engine deletion and actual endpoint qualification.
The APIs are published in Engine 0.39.3; this does not establish Canic adoption.

## Tooling and suppliers

Both workspaces select Toko Tooling **0.39.18**, with a verified 110-file snapshot.
Adoption and delivery evidence is under `target/auth-release-0394/`.
Keep `target/auth-release-0392/tooling/` and `target/tooling-adoption-20261112/`.

The release preserves incoming Storage **0.73.6**, qualification Metrics
**0.29.12**, Entropy **0.1.10**, and Wire **0.21.5** in both locks. All five browser
artifacts were regenerated; four Auth bindings are unchanged and Wire's runtime
reduces varint/record allocation work. All 98 client cases passed. Other selections
remain Build 0.83.0, Host FS/Process 0.76.5, Platform 0.31.3,
Runtime/macros/guard 0.21.3, Types 0.19.5 and Testkit 0.101.0. The independent
file CLI pin stays 0.76.4. Keep earlier supplier observations under
`target/auth-release-0392/tooling/`; they are not the current release lock identity.

Testkit retains managed startup and its native PocketIC 16.1.0 re-export; one
Wire identity is required. Principal boundaries copy validated bytes. Awaited
instance deletion remains best-effort, with teardown logs retained. Protocol
and default/session graphs exclude native tooling and persistent Storage.

## Qualification and recovery custody

`batch-qaK3IdsJ` stopped before delivery because the exact-pin exception still
named Tooling 0.39.17. Snapshot/tool admission and Clippy passed. The correction
updates both exception values to 0.39.18 and shipped in the corrected batch.
Keep its failed gate and `target/auth-release-0394/delivery.log`; do not resume it.

Keep completed `batch-kCAsAk34`, `batch-IAAjfLaF` and `batch-6Eztyvd6`
checkouts/durable state, publication receipts and their release evidence.

- Preserve stopped `batch-XoEWGnOX`, `batch-5x63tbsF` and `batch-H6iT55MB`;
  their package-boundary, fixture-closure and debug seeding limitations were
  corrected before 0.39.2 delivery. Logs and exact-candidate seeding proof remain
  under `target/auth-release-0392/`. Details are retained at `fc1ee1e8`.

Preserve every candidate under `target/release-frozen/` and durable
`.git/release-state/frozen/`, including publication-state. Preserve prior
`batch-BiswIN7d`, `batch-t5q4t2DE`, `batch-1JDi3jNF`, `batch-sat4DSEt`,
`batch-4hebGSFP`, `batch-yiX0dEoV` and `target/publication-recovery/`.
Do not resume old attempts against versions already published.

Earlier evidence inventories, device-contract decisions and supplier qualification
remain in Git at `0bcf7ac667f693e874a00c6550b3b965359d7ffc` and
[history](history.md). Keep their referenced `target/` evidence; a prior external
cleanup removed some older raw logs, which narrative measurements cannot replace.
The [session contract](../protocol/sessions.md) and
[Canic adoption guide](../integration/canic-adoption.md) own bounded device,
ninety-day renewal, retained fences and lost-state refusal behavior.
