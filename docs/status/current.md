# Current handoff — 2026-10-29

## Base, scope and delivery

The local base is Auth **0.22.0**, `34ca71b67e816d804a77f76d09bb210256862663`
(`Release 0.22.0`). See its [handoff](https://github.com/dragginzgame/ic-auth/blob/34ca71b67e816d804a77f76d09bb210256862663/docs/status/current.md)
and #109/#110/#111/#112 for earlier storage/dependency/harness evidence.

[#113 — Reduce transport allocations and admission duplication](https://github.com/dragginzgame/ic-auth/issues/113)
owns eight implemented cleanup candidates, two retained candidates and the two
subsequent configuration cuts. [#114 — Adopt Memory/Storage 0.56](https://github.com/dragginzgame/ic-auth/issues/114)
owns the latest dependency adoption. Pending changelog is **0.23.0**; own versions
remain **0.22.0**. Nothing was staged, committed or pushed. No full CI, release,
publication or sibling edits ran. Release/recovery evidence remains intact.

Shared Tooling remains **0.27.6**, revision
`5a72359b5be19d5221334cf2c9b3204621d3f3f6`, 127 files. Platform/macros **0.8.2**,
Tooling **0.70.2**, Wire/derive **0.7.23** and Metrics **0.17.4** remain selected.
Only Memory/Storage advanced in this adoption: **0.55.2 → 0.56.0**.
Root floors now declare 0.56.0; all unrelated external lock identities are
preserved. Rust **1.91** remains the minimum; development uses **1.99.1**.

## Pending Rust API migration

Wire DTO Principal fields hold Platform-owned values directly. Native Wire bytes,
bounds and generated browser sources remain unchanged; human-readable Serde uses
Principal text. Host-only `typescript` selects Platform `wire-schema`.

All `SessionEstablished` variants hold compact `SessionReceipt` values.
`SessionStore::admit` and `MemorySessionStore::admit_with_commit` consume owned
Session/Replay records. Update backend signatures and move validated records
into storage. Callbacks borrow staged records and complete atomically. The
receipt is prepared before commit without a fallible reread; it grants nothing.

`SessionContext::required_scopes` borrows immutable `RequiredScopes`, admitted
once from protected operation definitions and rebuilt after decoding.
`RequiredScopes::NONE` supplies empty requirements; duplicates/order semantics
and bounds remain. Every operation keeps live scope/authority/time/revocation checks.
`SessionLimits::new` admits immutable backend bounds once; getters replace raw
fields. Reconstruct after configuration decoding/changes and return the retained
value from `SessionReader::limits`. Occupancy, byte capacity, associations and
generation remain independently checked. `MemorySessionStore::new` is infallible;
restoration remains fallible. See [sessions](../sessions.md) and
[consumer migration](../canic-adoption.md).

Fresh admission reuses derived identity after the retry path. Merkle buffers,
ledger publication, journal status projection and generator policy are simplified.
Raw/prepared verification wrappers retain distinct refusal/retry ordering;
cryptography already converges. The streaming-hash trial was rejected for larger
Wasm and tiny savings. No proof algorithm, signed-byte, trust or cache change occurs.

## Memory/Storage 0.56 qualification boundary

Published source revision is `956b2b92728f82f75dd51d4d10ed52f325529e91`, checked
against live official Cargo index records and archive identities. Memory changes
its allocation-ledger/Cell-value CBOR to bounded Wire, with `ICML` framing and
`ICMWIRE1` marker. Storage reuses upstream retained-length bounds admission.
Underlying manager/substrate Cell layouts and Auth payload codecs are unchanged.

Only the unpublished qualification host selects Memory/Storage. Its production
adapter needs no source repair. A test-only Platform substrate capability builds
isolated old-ledger backing from a published 0.55.2 fixture; the new runtime gives
a typed ledger-record refusal, and Auth bootstrap leaves every backing byte and
application Cell intact. No legacy decoder or fallback is introduced.

Older Memory backing is incompatible even if its application payload is
`AUTHNAT2`. Installed qualification uses fresh backing and current-format
upgrades. Retain older binaries/states. Any retained installation needs its own
reviewed transition before upgrade; this task authorizes no reinstall, reset or
discarding records. Never clear only the ledger while leaving application data.
Auth's prior CBOR-record retirement does not authorize another data retirement.

## Evidence

Latest evidence: `target/memory-storage-20261029/`, including index/source
identities, before manifests/lock/patch, lock-delta admission, artifacts, source
seals, logs and fresh PocketIC state paths. Current reference Wasm is
**2,038,000 bytes**, down **159,335 bytes (7.25%)** from the same-code 0.55.2
artifact (**2,197,335 bytes**), with unrelated locked dependencies preserved.
No new runtime optimization or production workload estimate is claimed.

Ten native host tests pass, including current-cell bounds/reopen/custody and
old-ledger refusal without clearing. Native all-target and Wasm host Clippy pass;
Rust 1.91 native/all-target and Wasm checks pass. Host Wasm dependency boundaries
and all five generated browser contract comparisons pass. The four installed
final-Wasm cases pass certification/live authority, composed roots, checkpoint
upgrade and SDK signed ingress/logout. PocketIC remains the admitted **16.1.0**
pair selected by the prepared Testkit 0.70.2 CLI. No current full CI, macOS,
Chromium, complete packed-client rerun or production adoption is claimed.

Prior cleanup evidence remains in `target/cleanup-023-20261029/` and
`target/checked-config-023-20261029/`, with #113 recording historical matched
figures and failed trials. Scope probes reduced one/wide requirement authorization
from **5,830/51,736 → 5,554/45,655**, with construction outside the timed call.
Their 366-byte Wasm increase and slightly higher fresh-admission cost were accepted;
these historical measurements used Memory/Storage 0.55.2.

## Consumer ownership and next delivery

Native Auth session/replay/login records, `AUTHNAT2` application payload and
browser journals remain unchanged by Memory adoption. Hosts retain protected
grants/clock/generation, memory allocation, atomic commits and certification.
Canic owns issuer/endpoint/lifecycle acceptance in
[Canic #491](https://github.com/dragginzgame/canic/issues/491), and browser issuance
in [#507](https://github.com/dragginzgame/canic/issues/507). Its existing shared
metrics adapter records Auth outcomes. No production wallet service, issuer
adapter or browser publication is established. User delivery/version selection
and complete CI remain separate.
