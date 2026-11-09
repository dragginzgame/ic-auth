# Current handoff — 2026-11-09

## Published state

**Engine 0.36.1** (`ic-auth`) and **Protocol 0.36.0**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
Both release commits are on `origin/main` and both annotated tags are pushed.
Standard batch reconciliation fast-forwarded the normal checkout to that source.

- Protocol commit: `3c30f425bbc8864526514db0dcfa9d810591a2cc`;
  tag `protocol-v0.36.0`, object `4d187d269d247d3bb9be0caccc361cab2089e5a4`.
- Engine commit: `f5c380f72119627fb33250fc449215a6615a5a5a`;
  tag `engine-v0.36.1`, object `49e9b243e2f20a9b27a2ed2d9dd411903adfed37`.

| Published package | Registry SHA-256 |
| --- | --- |
| `ic-auth-protocol-types` | `12ae95d99ec8d6efef08819986166914bbe201758da98fec84239fb0e3f5d8be` |
| `ic-auth-wire` | `45dbcd7c83b877806c7e64b8cf49033248fe738bb66c6c169679e65f3e7d6340` |
| `ic-auth` | `09f6a947a0ea8ddfadd1b95949dde176e1906865b278e0923eac3fcb875547a3` |

## Pending local work

The 0.37.0 drafts include Platform/Wire adoption, the Storage 0.71 update,
reference-store/pruning and signature-store cleanups, current Tooling/SHA-2
adoption and #130 host nonce qualification. Product versions remain unchanged.
Implementation is committed locally for release capture; no new publication
has occurred. Prior source/test edits and delivery custody are preserved.

Publication is authorized for Protocol 0.37.0 followed by Engine 0.37.0, only in
Auth. Testkit 0.97 is now published with matching Wire 0.19 / Platform 0.29;
its required Host 0.76.2 cohort also qualifies the earlier incoming product lock.
The native limits blocker is cleared. Batch `batch-Z6H_KgCz` passed 29/30
Protocol stages, including installed certification, upgrade, ingress and SDK
logout. Its last release-caller fixture failed on an obsolete dependency-pair
assumption. The fixture now accounts for the current catalog and shared
qualification metadata; a fresh source capture will rerun the required gates.
The failed batch/evidence are preserved. No new product version or upload occurred.

## Current Tooling and qualification placement

Both workspace catalogs select published `toko-tooling = "=0.38.2"`, SHA-256
`45c12cc2aa598d14e137dd7db50f12e943470b4ce351dfcaffeccde6821c1876`.
Canonical installation preserves 87 entrypoints and installs 176 owned files.
The four product packages remain under `crates/`; six private packages now live
in the independent `qualification/` workspace, with their own catalog/lock and
unchanged effective versions. Product release groups exclude those private
packages. Shared owns atomic two-lock preparation through the release policy
opt-in; Auth retains its source-bound receipt and delivery/recovery safeguards.

Focused checks pass for both group file plans and preparation, private-version
preservation, rollback after failed qualification, product resolution with an
incompatible qualification supplier, and manifest/lock input drift. Native fixture,
adversarial, metrics, Wasm lint/build and an installed SDK/session/logout/upgrade/
ingress case pass from the new workspace. Formatting-hook, jobserver and Testkit
caller checks exercise both roots. All three public archives build; the private
backend archive passes registry-consumer and packaged-scenario checks. CI/direct
publication refuse missing or drifted qualification tooling before effects.
Both-workspace formatting, pin, ownership, LOC-role and local-link checks pass.
The two-lock union preserves every incoming registry identity except the requested
Tooling update. All 106 original Rust files match their pre-move bytes. Evidence
is under `target/tooling-0382/`. The complete CI/release gate was not run.
No product API, signed bytes, Rust implementation or stored format changed in
this relocation. No release or sibling source changed.

At the tooling split, incoming Storage 0.71.1 introduced Runtime 0.21 beside
0.20.3. The maintainer preserved it and the failed singleton evidence. The
Platform/Wire batch below now resolves that conflict with one Runtime 0.21.

The preceding `sha2 = "0.11.1"` adoption retains its 105 Auth / 12 adversarial
cases and two installed SDK/session/certificate/upgrade/ingress passes under
Tooling 0.37.4. Its checksum, source and limits remain in
`target/tooling-0374-sha2/` and [the retained history](history.md).

The matched SHA-2 update grew reference Wasm from 1,821,027 total / 1,402,675
code-section bytes to 1,823,778 / 1,405,535 (+2,751 / +2,860 bytes) under unchanged
Rust 1.99.1 release settings and `cpubits="32"`, without an external optimizer.
See [the measured scope](../canonical-allocation-observations.md#direct-sha-2-011-adoption--2026-11-09).
No new Wasm measurement, runtime speedup or Canic footprint claim is made.

## Host entropy contract — #130

Published `ic-entropy` 0.1.0 (SHA-256
`d5a7386a4f8a63fc339bd18b773c49c078427a0273f66b708466852336045297`)
is selected only as a private backend-conformance dev-dependency. The
[host contract](../host-entropy.md) distinguishes portable nonces, fresh Canic
challenges, seeds and proofs; it preserves existing signed/replay/lifetime rules
and documents fail-closed acquisition, restart/restore and TEE/non-TEE limits.
Five conformance cases exercise real signing/session admission, exact retry,
replay, unavailable/short/exhausted providers, seeded resets and zero-nonce policy.
All five plus the 20 existing backend cases, focused Clippy, injected boundary
refusals, ownership/pin/snapshot/link/format checks pass. Selected library and host
Wasm normal/build graphs match the pre-change graphs exactly. Evidence is under
`target/issue-130/`; no new Wasm size or production entropy-source claim is made.
Canic #757 remains the adapter owner; no composed Canic acceptance is claimed.
Auth [#130](https://github.com/dragginzgame/ic-auth/issues/130) is closed as locally
implemented and qualified; the changes remain uncommitted and unpublished.

## Platform and Wire adoption

Both catalogs retain the compatible Platform 0.29 minimum. Both locks select
the incoming published **0.29.1**; its extra native Candid service exporter is
unselected by Auth. The frozen release gate will qualify this incoming patch.
Types 0.19.0, Wire 0.19.0, Runtime/macros/guard 0.21.0, Storage 0.71.1 and all
unrelated selections are preserved. Native Testkit 0.97 shares Platform 0.29; the actual Wasm host has exactly one Platform 0.29 /
Runtime 0.21 cohort. The changed upstream files are solely the owned Candid
decoder and its tests. Public Auth and the Wasm host do not select that codec;
Testkit selects it only in the native runner.

Fresh full locked metadata, declaration and selected graph checks, affected
product/private-core/Wasm Clippy, 68 crypto tests and 25 backend tests pass.
The release Wasm builds at 1,831,434 total / 1,411,686 code-section bytes
(+133 / +9 against the retained pre-update artifact, unchanged release settings).
Generated browser sources still match. Evidence and preserved inputs are under
`target/platform-adoption/`. No Auth implementation or product version changed.

The preceding Platform 0.28 / Wire 0.19 adoption retains 127 Auth, 15 Protocol,
26 private, 12 adversarial and 97 client test passes, browser interoperability
and three archive builds under `target/platform-wire-adoption/`; those broader
checks were not replayed for this facade-only update. Four Auth browser contracts
were unchanged; only Wire's shared runtime changed. The 0.37.0 draft ledgers
retain the prior breaking Rust identity cut, with no signed-byte/stored-format
migration. Agent boundaries copy validated Principal bytes explicitly.
The full CI/release gate was not run.

Testkit 0.97 removes the old Wire 0.18 / Platform 0.28 edges and requires Host
0.76.2. Metadata and graph checks admit the aligned cohort. Native Principal
identity conversions were removed where the supplier now takes the owned type;
Agent/delegation byte conversions remain. The CLI/server is explicitly prepared.
Earlier failure logs remain under `target/publication-readiness/`; the required
complete release gate will provide fresh installed and publication evidence.
Registry backend-bundle delivery requires the new Auth product cohort.

## Persistence and consumer boundaries

Storage 0.71 selects bucket backing only in the private qualification host.
Use fresh fixture backing; retired allocation headers refuse without clearing.
No production adapter, retained backing or `AUTHCLK1` checkpoint was migrated.
The [prior Storage qualification](history.md#local-storage-071-adoption) applies
to 0.71.0. The 0.71.1 host now passes the aligned Runtime graph check. Native
installed qualification will be recorded by the new complete release gate.

The browser journal is `ic-auth-login-cleanup-journal-v1`; the maintainer approved
retiring old journals on 2026-11-08. Unknown issuance, renewal sequence and expiry
remain durable. That approval never permits erasing Canic's protected safety state.
The [browser contract](../browser-client.md#durable-logout-and-guarded-identities)
and [clock-independent design](../design/portable-clock-audit.md) retain 90-day
login windows, short proofs and 30-minute receiver leases. Renewal cannot extend
the login window; lost protected state remains closed pending trusted recovery.

The last [read-only Canic review](../canic-adoption.md#read-only-canic-review--2026-11-09)
covered HEAD `3700c6e1bf78cb04a60dcfaf0f789d4dd992c9e0` plus dirty inputs and found
no Auth API gap. Canic's deployment hold, Engine 0.36.1 adoption and final installed
qualification remain consumer-owned under [Canic #491](https://github.com/dragginzgame/canic/issues/491).
This dependency batch did not re-review or change Canic. No standalone Auth service
is deployed here. GitHub issues remain the task tracker; the release preparation is recorded above.

## Retained delivery custody

Keep completed Engine `batch-QTi7wJc_`, its managed checkout, original receipts and
`.git/release-state/frozen/batch-QTi7wJc_/` plus
`.git/publication-state/engine-0.36.1.json`. Its 30-stage release proof and publication
artifacts remain under `target/release-frozen/batch-QTi7wJc_/`; independent readback
is under `target/delivery-0361/`. Do not resume failed `batch-8jjqjTO1` against the
delivered version. Protocol 0.36.0 and older delivery custody remain protected.
[Historical notes](history.md) preserve earlier sources, locations, checksums,
qualification limits and rejected trials. No artifacts were deleted while trimming
this startup summary. Earlier raw files lost in the cleanup incident still cannot
be independently verified. Keep build trees and bulk archives out of `.git`.
