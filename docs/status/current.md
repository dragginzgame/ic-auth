# Current handoff — 2026-11-08

## Published state

**Engine 0.35.9** (`ic-auth`) and **Protocol 0.35.1**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
Engine's release commit and annotated tag are pushed to `origin/main`; the normal
checkout is fast-forward reconciled through the shared batch controller.

- Engine release commit: `06028c6454930ab4afdf9d77b19259f73a6cfb0c`.
- Engine tag: `engine-v0.35.9`, object `2847053a1e70f04ad9be7bf1134d919ccadbe881`.
- Registry checksum: `1aeb32a16468cd09e65c22e6085e327408f944f4d105abc8505f369c089cafd0`.
- Protocol is unchanged: commit `39b6e2605ad336c3aadff94bfd8af0dcb07cf8b3`,
  tag `protocol-v0.35.1`.

The published Engine includes both cleanup batches, Shared Tooling 0.31.11 and
incoming Build 0.79.1, Host 0.75.1, Storage 0.68.11 and Wire/derive 0.17.14.
The [Engine changelog](../../releases/engine/CHANGELOG.md) owns the shipped changes.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) remains closed.

## Delivery custody

Engine 0.35.10 delivery is scoped to Auth. Initial batch `batch-8Pq1jYH0`, source
`d8ffaa01c444cf8bc6cf5cbe0e2e76e612e44e27`, was interrupted during validation after
a focused check found missing substitutions for Tooling 0.34's admission/cleanup
in Auth's synthetic jobserver fixture. Replacement `batch-VzhWRpEn`, source
`a33fe182ed7b8a4ae9cc1aa26424264d77fb19ef`, passed 15 gate stages before its Testkit
caller fixture refused at the old admission stub. Both corrected callers and the
client pipeline now pass focused checks; fresh capture is pending. No version
preparation, tag, push or upload occurred. Keep both `.git/release-state/frozen/`
journals and `target/release-frozen/` checkouts.
Logs and correction evidence: `target/delivery-03510/`.

The completed 0.35.9 batch `batch-rovUq5Bd` and its two contention interruptions
remain retained in the same custody roots. Its 30-stage Linux gate passed at
`target/release-frozen/batch-rovUq5Bd/target/release-validation/engine-gate.LbbY8l/`,
logs `logs/validation.W4fCAE/`; publication evidence is its
`target/publication-engine-0.35.9.4viiS5/`. The verified receipt is
`.git/publication-state/engine-0.35.9.json`, also in that batch's Git store.
`target/delivery-0359/` retains delivery/readback/reconciliation logs. No upload is
unresolved. Chromium remains opt-in; hosted CI remains asynchronous.

## Selected dependencies and footprint

Shared Tooling now selects crates.io **toko-tooling 0.34.0** through root metadata
and the separate `toko-tooling.lock`. Its archive SHA-256 and installed 172-file
snapshot agree at
`5ca8ad09ad9359bc4945a4afdaafce301dd5a4b4955cd5c9f5058d7a54ffcf10`.
All Cargo tooling dependencies/policy entries are removed; the real Auth tooling
member and its commands/benchmarks remain. Other application lock selections,
all 84 entrypoints, local Rust edits and product release groups/roster are preserved.
Installation/tool admission, offline Make checks, membership and release routing
pass. CI and direct publication refuse missing/drifted tooling before effects;
the validation input seal also detects mid-run tooling-lock drift. No full gate ran.
Edit the exact root version, then use `make shared-tooling-update`; install
reuses its existing lock. Evidence: `target/tooling-034-migration/`.
The earlier registry migration evidence remains in `target/tooling-registry-migration/`.
The earlier LOC display checks remain in `target/shared-tooling-adoption-03112/`.

Pending Engine 0.35.10 narrows the private scope helper to `ScopeLabel`,
inlines its single-use request wrapper and gates the private browser-caller getter
to sessions. Public APIs, checks and stored/signed bytes are unchanged. Engine
0.35.10 delivery is authorized; package versions remain 0.35.9 before capture.
The complete source-bound gate and registry delivery are pending. Protocol stays
at 0.35.1. Delivery evidence belongs in `target/delivery-03510/`.

Auth now requires published **Platform 0.27.0**. Testkit 0.93.2 retains Platform
0.26.1 for its native helpers; Types 0.18.2 and Runtime/macros/guard 0.20.2 remain
shared and unchanged. The removed entropy API has no Auth callers. Locked fetch,
declaration/graph boundaries, affected native consumers and the Wasm host's
Clippy (`-D warnings`) pass, as do six selected crypto/proof cases. Existing
package versions/checksums, Wire generation and local Rust edits are preserved.
Evidence: `target/platform-027-adoption/`. No full gate ran; the later Storage
adoption below adds one installed checkpoint case. Earlier scope checks and the matched
measurement below used Platform 0.26.1, before this adoption.

The private checkpoint host now selects **Storage 0.69.0**, using direct cell
write results. Constructor errors, bounds and stored framing remain unchanged;
no reset or record migration is introduced. All other lock entries are preserved.
Graph checks, native/Wasm Clippy, 12 checkpoint tests and one installed PocketIC
upgrade/session-denial/fresh-authority case pass. Evidence:
`target/storage-069-adoption/`. This is focused adoption, not a full release gate.

Matched normal-checkout builds at `02c5d98` plus the local patch use the same lock
(`c2d9f5d8fd022d20ca5f200925ac737b91c1c72be9d4ee88f1f62759be411e42`),
Rust 1.99.1, Cargo release, existing target flags and no post-optimizer. They move
from **1,878,589 total / 1,444,351 code bytes** to **1,876,791 / 1,443,359 bytes**:
**1,798 total / 992 code bytes saved**. Final SHA-256:
`fb24bb0002bff2ec92e905c9a18f5a451161208772d73a92706647341d7d86cb`.
Evidence and input snapshots: `target/evidence/scope-surface-20261108/`.
This is reference fixture footprint, not throughput or a comparison with the
published cohort. Earlier qualification and measurements below retain their scope.

The released cohort selects Platform 0.26.0 (native Testkit alone retains 0.25.1),
Types 0.18.2, Runtime/macros/guard 0.20.1, Testkit 0.93.1, Build 0.79.1,
Host 0.75.1, Metrics 0.29.6, Storage 0.68.11 and Wire/derive 0.17.14.
Rust remains 1.99.1. All five generated browser files retain their original bytes.
No packed-ID codec is selected. Shared Tooling is 0.31.11 at
`6c529e6940a97226cca826726491e03173b08cd2`, with its 165-payload closure.

The gate's reference qualification Wasm, before Engine version preparation, is
**1,877,333 total bytes / 1,443,345 code-section bytes**, SHA-256
`44f9285b023deab3ea0cbbb3d33712504ce659303614cead475eb61e94ed3d3c`.
Measurement: uncompressed Wasm, section-10 payload length, Cargo release with the
existing target flags and no post-optimizer. This is not Canic's deployed Wasm
or a matched comparison with the earlier normal-checkout experiments.
`target/delivery-0359/gate-wasm.json` records the artifact and source identity.

Earlier matched cleanup measurements retain their own source/dependency scope:
[methods, results and limits](../canonical-allocation-observations.md#signature-retention-and-shared-grants--2026-11-08).
Raw evidence remains in `target/evidence/signature-index-20261108/` and
`target/evidence/prepared-state-20261108/`. The second batch saved 238 total bytes
but added 841 code bytes; its benefit is state ownership and less duplicate key
storage. No speed improvement is claimed.

## Storage and consumer boundaries

Public Rust APIs, signed bytes, cryptographic checks and Rust checkpoint formats
remain unchanged. The private client uses `ic-auth-login-cleanup-journal-v1`.
The maintainer confirmed no retained journals on 2026-11-08; the retired
`ic-auth-local-logout-journal-v1` profile is refused without clearing records.
Credential expiry, renewal sequences and unknown issuance remain durable. See
[client storage and retirement](../browser-client.md#durable-logout-and-guarded-identities).
That approval does not authorize erasing Canic or other protected safety state.

The [Canic contract](../canic-adoption.md) and
[clock-independent design](../design/portable-clock-audit.md) retain 90-day login
windows, short proofs, 30-minute receiver leases, bounded unknown denial and
lookup-only uncertain issuance. Renewal never extends the original login window;
ordinary authorization stays local. Lost protected safety state stays closed
pending trusted recovery. Complete durable restore plus fresh authority is supported.

The last [read-only Canic review](../canic-adoption.md#read-only-canic-review--2026-11-07)
found no Auth implementation gap. Canic's extraction into `canic-auth`, mounting,
persistence composition and installed acceptance belong to
[Canic #491](https://github.com/dragginzgame/canic/issues/491). No sibling source,
release or deployment changed here.

Older handoff details remain in Git history. Retain completed delivery/recovery
and profiling evidence; earlier raw files lost in the cleanup incident cannot be
independently verified. Keep build trees out of `.git` and do not erase journals.
