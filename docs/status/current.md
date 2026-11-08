# Current handoff — 2026-11-08

## Published state

**Engine 0.35.10** (`ic-auth`) and **Protocol 0.35.1**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
The Engine branch/tag push, registry checksum readback and standard batch
reconciliation completed. The normal checkout now contains the delivered release.

- Engine release commit: `d447e1ff45a7a79725119d8b3d917217daacc7e6`.
- Engine tag: `engine-v0.35.10`, object `e743f49b3a725b7d67943ef80a04cbd2b8d238b2`.
- Registry checksum: `ea1a19077449d4110e1f3d0309c5e5578e89ca1c3ed333366e69663466fbf54d`.
- Protocol is unchanged: commit `39b6e2605ad336c3aadff94bfd8af0dcb07cf8b3`,
  tag `protocol-v0.35.1`.

The [Engine changelog](../../releases/engine/CHANGELOG.md) owns shipped changes:
narrower internal scope checks, the session-only getter, root-owned Tooling,
Platform 0.27.0 and Storage 0.69.0. Public APIs, signed bytes, cryptographic policy
and current checkpoint formats are preserved. No reset or migration is introduced.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) remains closed.

## Completed delivery and qualification

Batch **batch-tnVpLRob** captured source
`8e1a0a8037870c6a153c1834ca0026a5da41b658` for Engine alone. Its schema-2 intent
is complete at index 1; reconciliation fast-forwarded the clean normal checkout.
No upload remains unresolved. This delivery was explicitly scoped to Auth.

- Durable intent, Git and recovery: `.git/release-state/frozen/batch-tnVpLRob/`.
- Registry receipt: `.git/publication-state/engine-0.35.10.json`; identical bytes
  remain in the batch's `repository.git/publication-state/`.
- Managed delivery checkout: `target/release-frozen/batch-tnVpLRob/`.
- Publication evidence: its `target/publication-engine-0.35.10.G9nKGy/`.
- Delivery, independent registry/ref readback and reconciliation: `target/delivery-03510/`.

All **30 required Linux stages** passed. Gate:
`target/release-frozen/batch-tnVpLRob/target/release-validation/engine-gate.UhakLb/`,
logs `logs/validation.oazCDy/`. It covers native/Wasm Clippy, package/MSRV and
boundary checks, 95 client cases, proof/session suites, 11 installed lifecycle
and ingress cases, archives and full release/recovery fixtures. Chromium remains
opt-in and ignored. The final fixture stage took 1,097 seconds. Hosted CI remains
asynchronous; no hosted completion is claimed.

Earlier attempts are retained: `batch-8Pq1jYH0` was interrupted after a focused
check found the missing jobserver-fixture substitutions; `batch-VzhWRpEn` passed
15 stages before the Testkit caller fixture refused at its old admission stub.
Both fixture gaps were corrected and qualified before the successful capture.
Neither attempt prepared a version, tag, push or upload. Keep their journals,
checkouts and logs; do not resume them against the delivered 0.35.10 identity.
The completed 0.35.9 batch `batch-rovUq5Bd`, its two contention interruptions and
receipt remain retained. Earlier detail is preserved in Git and `target/delivery-0359/`.

## Selected tooling, dependencies and footprint

Root metadata selects **toko-tooling 0.34.0** independently of application Cargo
packages. `toko-tooling.lock` and the installed 172-file snapshot agree on SHA-256
`5ca8ad09ad9359bc4945a4afdaafce301dd5a4b4955cd5c9f5058d7a54ffcf10`.
All 84 entrypoints remain. No tooling Cargo dependency or selection-only package
remains; the real Auth tooling commands and benchmarks stay in `apps/tooling/`.
Edit the exact root version, then run `make shared-tooling-update`; install reuses
its existing lock. CI/publication admission and mid-run tooling-lock seals are
qualified. Migration evidence: `target/tooling-034-migration/`.

The release selects Platform 0.27.0, Types 0.18.2, Runtime/macros/guard 0.20.2,
Testkit 0.93.2, Storage 0.69.0, Wire/derive 0.17.14, Build 0.79.1, Host 0.75.1
and Metrics 0.29.6. Testkit retains Platform 0.26.1 only in native helpers; the
Wasm host keeps one facade and one Runtime executor/generator/guard. Rust remains
1.99.1. All five generated browser files retain their original bytes.
Focused adoption evidence: `target/platform-027-adoption/` and
`target/storage-069-adoption/`. No packed-ID codec is selected.

The gate's reference qualification Wasm, before Engine version preparation, is
**1,869,560 uncompressed bytes**, SHA-256
`f6e1584d38c86dbe8fdf4677b561f6ce89770c175201185c3d63bcb5ad9b8a8a`.
Its artifact identity is retained in `target/delivery-03510/gate-wasm.sha256`.
This fixture measurement belongs to the captured dependency cohort; no throughput
or matched implementation saving is inferred from it.

Earlier matched scope cleanup used Platform 0.26.1 and its own fixed lock/compiler:
1,798 total / 992 code bytes saved, evidence `target/evidence/scope-surface-20261108/`.
[Other allocation observations](../canonical-allocation-observations.md#signature-retention-and-shared-grants--2026-11-08)
retain their original source/dependency limits and evidence. No speedup is claimed.

## Storage and consumer boundaries

The private client uses `ic-auth-login-cleanup-journal-v1`. The maintainer confirmed
no retained journals on 2026-11-08; retired `ic-auth-local-logout-journal-v1` records
are refused without clearing. Credential expiry, renewal sequences and unknown
issuance remain durable. See [storage and retirement](../browser-client.md#durable-logout-and-guarded-identities).
That approval does not authorize erasing Canic or other protected safety state.

The [Canic contract](../canic-adoption.md) and
[clock-independent design](../design/portable-clock-audit.md) retain 90-day login
windows, short proofs, 30-minute receiver leases, bounded unknown denial and
lookup-only uncertain issuance. Renewal never extends the original login window;
ordinary authorization stays local. Lost protected safety state stays closed
pending trusted recovery. Complete durable restore plus fresh authority is supported.

The last [read-only Canic review](../canic-adoption.md#read-only-canic-review--2026-11-07)
found no Auth implementation gap. Canic composition and installed adoption belong
to [Canic #491](https://github.com/dragginzgame/canic/issues/491). No sibling source,
release or deployment changed here; no standalone Auth service is deployed.
Preserve delivery/recovery and profiling evidence. Earlier raw files lost in the
cleanup incident cannot be independently verified. Keep build trees out of `.git`.
