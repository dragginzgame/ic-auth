# Current handoff — 2026-11-04

## Current local work

Pending Engine improvements now include bounded signature-output serialization
and eight reusable backend scenarios. Consumer test drivers supply new prune/
clock hooks and a fallible generation update. All 15 backend tests pass, including
deliberately broken cleanup bounds, partial writes, generation changes and lost
clock floors. Signature checks pass eight library tests, ten retrieval tests and
the focused real-BLS composed-witness test. Native and Wasm Clippy, dependency
ownership, formatting and local documentation checks pass. Evidence:
`target/auth-focused-improvements/`. These checks do not establish actual Canic
backing performance, crash atomicity, clock provenance or installed acceptance.

The unchanged exact-retry path is profiled natively: 100 calls each show five
allocations/116 bytes for small claims and 545 allocations/16,736 bytes for wide
claims, matching standalone decode in both profiles. Setup/signing/request
construction are excluded. No retry optimization, Wasm saving or latency result
is claimed. Input-sealed evidence: `target/portable-fixtures/retry-allocations.x76x4g/`;
[scope and decision](../retry-allocation-observations.md).

Shared Tooling 0.29.4 is adopted at
`628ccc75d813bfd9bae777bfbaef55838d25b1b2` through the canonical pinned-revision
exporter using the existing supplier repository. All 150 payloads match the
independently selected commit. Auth's direct metadata/publication adapters now
use Shared's retained-state guard before/after work, preserving original failure
statuses. New source extracts and gate logs use `target/release-metadata/` and
`target/release-validation/`; saved receipts still admit their original paths.

The focused Auth caller fixture passes runner/direct refusal, evidence
preservation, target-owned extraction and original failure precedence. ShellCheck
passes with source following. Existing Git-side retained state passes the default
guard. Recovery review cleared both historical metadata extracts for removal:
`metadata.FSuknc` exactly reproduces `f8d693c`, and `metadata.9Xe8vq` reproduces
`4d74611`, including all 406 source files and executable modes in each. Both
commits are ancestors of main; outer manifests/locks/notes duplicate source and
both generated plans reproduce byte-for-byte. No unique work or required
evidence was found. No process uses the extracts, no release/publication lock
exists, and 18,723 retained/source files contain no external recovery reference
to them. The later Engine 0.33.1 attempt is complete with publication custody.
The maintainer then authorized removal. After rechecking every file against the
review, active use and recovery locks, both directories were deleted: 820 files,
8,588,761 logical bytes and 10,559,488 unshared allocated file bytes reclaimed.
Maintained source and Git refs were fingerprinted unchanged through removal;
all other recovery/publication state remains intact. No full archive or release
was created. The compact deletion receipt is `removal.json` under:
`target/shared-tooling-0294/recovery-review/`.

Eleven inline Rust unit-test modules and their private checkpoint helper now
use ordinary adjacent `tests.rs` discovery. All 24 existing tests pass across
Engine, Protocol Types and the two qualification libraries; focused library
Clippy passes. Reviewed production files retain their exact non-test code.
Moved bodies match after formatting and the explicit fixture-relative include
and helper-scope adjustments. The internal revocation module is now named
`tests`; no production visibility was widened. API doctests and public
qualification support remain with their existing owners. Cargo payload lists
contain the relocated public-package modules. Evidence:
`target/shared-tooling-0293/`.

Metrics 0.29.0 now supplies qualification admission counters. The fixture no
longer derives Serde for recorders or qualifies a private report codec; Canic's
metrics adapter owns reporting. All five outcome tests pass, including overflow
leaving committed admission and exact retries intact. Focused native test Clippy
and canister Wasm Clippy pass. The latest layout scan finds all 263 Rust test
attributes under `tests.rs` or `tests/`, with only module wiring in production
files. Evidence: `target/shared-metrics-adoption/`.

Publication is authorized for Protocol 0.33.1 followed by Engine 0.33.2 through
the normal checkout on `main`, direct delivery to `origin/main` and crates.io.
Group ledgers own the pending notes; incoming TOML-only lockfile updates were
preserved. Release versions, full-gate results and publication are not yet final.

The first release source is `eec726e`. Its Protocol gate passed the first 29
stages and completed all release-tools fixture cases, but final source admission
refused incoming lock drift to Metrics 0.29.1 and Storage 0.66.2. No real version,
tag or upload was created. Original evidence is retained at
`target/release-validation/protocol-gate.z5fVNr/logs/validation.pPPrEH`; it is not
reused as admission for the incoming lock. The two compatible patch selections
are preserved for fresh source commitment and complete qualification.

## Retired sibling checkouts

The maintainer authorized consolidation/deletion of
`/home/adam/projects/ic-auth-capacity` and
`/home/adam/projects/ic-auth-release-033`. Both directories are removed. Neither
had unique committed or reflog-only work missing from the normal repository.
Their HEADs remain as local branches `archive/ic-auth-capacity` (`f8d693c`) and
`archive/ic-auth-release-033` (`fa6823a`), both ancestors of current main.

Capacity's implementation, regression tests and contract documentation were
already integrated by `173ab46` and published in Engine 0.33.1; the three working
files were byte-identical before deletion. Its unique qualification context and
38-case native evidence were retained. No older manifest or draft overwrote
current source or notes.

Required historical capacity/release/publication records, logs, final package
archives, browser receipts and qualification Wasm are checksum-verified at
`target/checkout-retirement/20261104-siblings/manifest.json`: 424 evidence files,
4,177,949 payload bytes, within a 16 MiB selection budget. There is no full
checkout/source/build archive in `.git`. Original paths inside historical
records remain unchanged; these completed records are not relocated live
recovery admission. Compiler caches and redundant source trees were removed.
Explicit `cargo clean` can remove the entire target, including this evidence.
Main-checkout source was fingerprinted unchanged before removal; subsequent
edits only update these instructions, history and pending governance notes.

## Delivered baseline

Engine 0.33.1 (`ic-auth`) is published at release commit
`f9105b938aa76de80f1e83c635e1ab445133911c`; `main` and annotated
`engine-v0.33.1` were pushed. Registry checksum:
`44ffe15b94526c3ad7c79194b10d0c4a7efc21643a54a85ad904c6a01ea61dd2`.
Protocol stays 0.33.0. Platform 0.21, Testkit 0.84.1, Types 0.18, Runtime/guard
0.19, Wire/derive 0.16.1 and Shared 0.29.2 were the released selection.

All 30 fresh release stages passed against source `173ab46`, including native/
Wasm checks, package/MSRV consumers, live canister and recovery/publication cases.
Original current-repo proof remains at
`.git/release-state/engine-gate.Uy6QIN/logs/validation.4GXM4g`; the completed plan
is `.git/release-state/engine-0.33.1.plan`. Earlier failed/interrupted attempts
retain their own meaning; their evidence is not substituted for this receipt.

Clean-release backend/browser bundles and checksums:
`target/platform-021-delivery/delivery/manifest.json`, source `f9105b9`, dirty
false. Nine backend scenarios and installed-archive Chromium/real-IndexedDB
checks pass. Private bundle tests use reference adapters/injected transports;
actual Canic storage, issuer, browser and production acceptance remain Canic-owned.

`SessionLimits` accepts positive protected host budgets such as 100,000 sessions;
it allocates no backing. Count/byte enforcement, checked arithmetic, restoration
refusal, individual record limits, verification and TEE/non-TEE trust remain.
The separately requested Canic cfg warning fix passed default Wasm, receiver-
enabled Wasm and native Clippy under `canic/target/auth-capacity-cfg/`; no Canic
commit or deployment was performed here.

## References and limits

[Canic adoption](../canic-adoption.md) owns consumer contracts; GitHub issues own
tasks. Canic #491 owns production acceptance, and Auth #125 owns native group-CI
acceptance. No fresh hosted result or issue closure is inferred from local work.
[Earlier delivery/qualification observations](history-0.33-delivery-and-qualification.md)
retain the complete previous handoff. They are history, not an active work queue
or instructions to recreate retired checkouts. Releases use the normal checkout.
