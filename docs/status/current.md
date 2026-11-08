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

## Completed frozen delivery

Batch `batch-rovUq5Bd` captured source
`96dbfd0c2c2b1428d6325031dd6e741553cc23ea` for Engine alone. It completed publication
and registry checksum readback, then was reconciled without deferred normal edits.
No upload remains unresolved. Protocol manifests and its ledger were preserved.

- Durable intent, Git and recovery: `.git/release-state/frozen/batch-rovUq5Bd/`.
- Registry receipt: `.git/publication-state/engine-0.35.9.json`; the original also
  remains in that batch's `repository.git/publication-state/`.
- Managed delivery checkout: `target/release-frozen/batch-rovUq5Bd/`.
- Publication evidence: its `target/publication-engine-0.35.9.4viiS5/`.
- Delivery/resume/reconciliation logs: `target/delivery-0359/`.

An initial gate stopped on shared tool-install contention before compilation.
The resumed fresh gate passed all **30 stages**. A second transient editor lock
stopped pre-push admission after the release commit; exact saved-release recovery
retained the valid gate and completed push/publication without rerunning it.
Both interruptions and their evidence remain retained.

The successful Linux gate is
`target/release-frozen/batch-rovUq5Bd/target/release-validation/engine-gate.LbbY8l/`,
with logs in `logs/validation.W4fCAE/`. It covers native/Wasm Clippy, isolated
package/MSRV qualification, dependency boundaries, all 95 client cases and Wire
interoperability, proof/session suites, all 11 ordinary installed qualification
cases, package archives and release/recovery fixtures. Chromium remains opt-in
and ignored. The final fixture stage took 1,047 seconds. Hosted CI runs
asynchronously; no hosted completion is claimed here.

## Selected dependencies and footprint

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
