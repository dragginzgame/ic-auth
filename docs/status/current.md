# Current handoff — 2026-11-04

## Published cohort

Protocol 0.33.1 (`ic-auth-protocol-types`, `ic-auth-wire`) and Engine 0.33.2
(`ic-auth`) are published on crates.io. Both annotated tags and the release
commits are pushed to `origin/main` from the normal checkout. Registry identity
and archive checksums were observed by Auth's publication adapter:

| Package | Version | Registry SHA-256 |
| --- | --- | --- |
| `ic-auth-protocol-types` | 0.33.1 | `48da6380603cea3b451c43a9fc5fb580f487940f57571969ebae2266cb9cf000` |
| `ic-auth-wire` | 0.33.1 | `c57063776628aa608060aedeb95ac9246763652d053212e21eac1e2b6faa6cf8` |
| `ic-auth` | 0.33.2 | `8aca377ec4e9abeefbf03a363cf6007203f3a3a703614fb8ad898d02af1892fc` |

Protocol release commit: `4dec02e083181f37706df8046c6dd9cdbe2bfcb8`, tag
`protocol-v0.33.1` (`7597795af1e0ec38dca235af37677c9816151ef0`). Engine release
commit: `f9dc978505832ca3a32a068feee7cdaa317a272e`, tag `engine-v0.33.2`
(`5ee62f2f9c5d950b14ed825d458d0ff1a86ee644`). Completed plans and checksum-bound
publication custody remain under `.git/release-state/` and `.git/publication-state/`.

All 30 stages passed separately for both standard releases, including native/
Wasm Clippy, isolated package/MSRV consumers, browser lifecycle/interoperability,
real canister certification/upgrade/signed ingress, archive builds and Auth's
release/recovery/publication fixtures:

- Protocol source `9168a91`:
  `target/release-validation/protocol-gate.H39gaN/logs/validation.mqtPDn`.
- Engine source `f61f4ea`:
  `target/release-validation/engine-gate.fiypz8/logs/validation.VQ5F8b`.
- Delivery/upload logs: `target/auth-033-delivery/`.

The final lock selects Metrics 0.29.1, Storage 0.66.2, Host/Build 0.74.3,
Runtime/macros/guard 0.19.1, Platform 0.21, Types 0.18, Testkit 0.84.1 and
Wire/derive 0.16.1. Rust 1.99.1 remains the compiler/minimum. Shared Tooling
0.29.4 is adopted at `628ccc75d813bfd9bae777bfbaef55838d25b1b2`; all 150
snapshot payloads match the canonical pinned-revision export.

## Delivered behavior and adoption support

Signature-output serialization now refuses oversized writes before appending,
with unchanged CBOR envelope bytes and exact-boundary admission. Cryptographic
checks, signed material and the TEE/non-TEE trust model are unchanged. No Wasm
saving or latency improvement is claimed.

The backend bundle now has eight scenarios, adding bounded cleanup rollback and
retained clock/generation checks. Drivers supply prune/clock hooks and fallible
generation updates. Its 15 tests include deliberately broken bounds, partial
writes, generation updates and lost clock floors. The clean release archive was
qualified after publication against the exact registry cohort, with trait/error
identity admission, graph refusals and all 15 packaged tests passing:

- Receipt: `target/portable-fixtures/backend-package.WoSpAu/delivery.json`.
- Archive: `target/portable-fixtures/backend-package.WoSpAu/target/package/ic-auth-backend-conformance-0.33.2.crate`.
- Archive SHA-256: `aa0cb3efb9e23089610d420bc00ab79915bc464952f404f11dd911d9c9bbcbba`.
- Source `f9dc978`, dirty false. This unpublished source bundle is dev support;
  actual Canic backing, clock provenance and installed acceptance remain Canic-owned.

The unchanged exact-retry path was profiled natively: 100 calls each showed five
allocations/116 bytes for small claims and 545 allocations/16,736 bytes for wide
claims, matching standalone decode. Setup/signing/request construction were
excluded. No retry optimization was implemented. Evidence:
`target/portable-fixtures/retry-allocations.x76x4g/`;
[measurement scope](../retry-allocation-observations.md).

Rust unit tests/private helpers use ordinary adjacent `tests.rs` discovery.
All 263 Rust test attributes are under `tests.rs` or `tests/`; API doctests stay
with their declarations. Metrics recorders no longer own report serialization;
Canic's shared adapter owns reporting. Auth's release/publication callers apply
Shared's retained-state guard and put new source extracts/gate logs under
`target/`, preserving original failure status and historical recovery paths.

## Interrupted attempts and retained history

Incoming dependency updates invalidated three earlier attempts. Their passing
stages are historical observations, never substituted for the final receipts:

- Metrics/Storage drift: `target/release-validation/protocol-gate.z5fVNr/logs/validation.pPPrEH`.
- Host/Runtime drift: `target/release-validation/engine-gate.GYD3n3/logs/validation.ZdiDU0`.
- Build drift: `target/release-validation/engine-gate.rEEpuX/logs/validation.GxoZGh`.

The incoming selections were preserved and committed, then completely qualified
in the successful attempts above. No interrupted attempt created an Engine tag
or upload. The maintainer paused all Auth updaters for the final delivery.

The redundant `ic-auth-capacity` and `ic-auth-release-033` sibling checkouts were
removed with maintainer approval after confirming their work was integrated.
Their refs remain `archive/ic-auth-capacity` and `archive/ic-auth-release-033`.
Required compact historical evidence is at
`target/checkout-retirement/20261104-siblings/manifest.json`. Two verified,
reproducible old Git-side metadata extracts were also removed; receipt:
`target/shared-tooling-0294/recovery-review/removal.json`. No bulk checkout/build
archive was retained in `.git`. Other recovery/publication state is preserved.
Explicit `cargo clean` can remove all target-owned evidence.

## Remaining ownership

[Canic adoption](../canic-adoption.md) describes consumer contracts. Canic
[#491](https://github.com/dragginzgame/canic/issues/491) owns production acceptance;
Auth [#125](https://github.com/dragginzgame/ic-auth/issues/125) owns native group-CI
acceptance. No fresh hosted result, issue closure or production adoption is
inferred from local release qualification. The previous browser/backend handoff
at `target/platform-021-delivery/delivery/manifest.json` remains separate Engine
0.33.1/Protocol 0.33.0 evidence; it does not qualify the new backend hooks.
[Earlier delivery history](history-0.33-delivery-and-qualification.md) remains
historical. Continued implementation uses the normal checkout, without new
sibling directories.
