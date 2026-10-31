# Current handoff — 2026-10-31

## Source and pending work

Released source is Auth **0.27.0**, tag `v0.27.0`, commit
`81ff5379f39128b2d7f11ea4217b4f2e7e674d1f`. Package versions remain unchanged;
working changes are unstaged and uncommitted. No release or publication was run.

Pending **0.28.0** includes checked session scopes, shared backend preflight checks,
reusable native conformance scenarios and publication-runner adoption. The minor
is required because `Session::scopes()` returns `&[ScopeLabel]`; use `as_str()` for
text. Valid stored bytes remain unchanged. Malformed stored scope grammar now
reports `InvalidPolicyOrRecord` during decoding, with the same public read refusal.

[#123](https://github.com/dragginzgame/ic-auth/issues/123) is implemented locally
and remains open pending delivery/acceptance. Auth keeps release receipt/tag and
checksum admission, private publication intent, uncertain-upload markers and exact
registry readback. Shared owns exact `NAME@VERSION` dispatch, fresh attempts and
interruption evidence. Its stable lock in publication state requires separate
explicit disposition before a new dispatch; registry confirmation does not clear
that lock. Existing intent schema and recovery authority are preserved.

## Selected suppliers and governance

The incoming lock already selected Memory/Storage **0.60.3** and Testkit
**0.75.1**. Root requirements now match; the lock is preserved byte-for-byte.
All 65 Memory, 12 Storage and 28 Testkit source files match the previous qualified
0.60.1/0.75.0 packages exactly. No record migration, reset or feature change.

Other selections: Platform/macros/runtime guard **0.15.3**, canonical Types
**0.15.1**, Wire/derive **0.13.1**, Build/Host/file CLI/editor **0.73.2**, Metrics
**0.24.1**, PocketIC **16.1.0**. The new Testkit CLI is explicitly prepared and
admits that server. Testkit's private arithmetic-only Metrics **0.23.2** exchanges
no values with Auth's selected Metrics.

Shared snapshot **0.27.23**, revision
`619d194875369e0f72b45477b66adb430acbf6b7`, was exported from an isolated clean
checkout and all 135 payloads verified against that independently selected commit.
The sibling's dirty work was preserved. This is committed-source adoption, not a
claim of a release tag. Auth keeps one Cargo-owned workspace/catalog/lock and
coordinated release; Shared's pending canonical `VERSION` automation is inapplicable.
No root VERSION file, prepare-work target or pending-file release mode was added.

## Public and consumer contracts

Wire 0.13 owns Principal/Account schema metadata through `platform-types`;
Platform reexports canonical Types 0.15.1. Consumers sharing those public types
align their suppliers. Passive packages select only Principal; hashing and external
conversions remain explicit. Public graphs exclude runtime, Candid, persistence,
Metrics and native tools. The reference Wasm retains one runtime guard/generator.
See [migration](../canic-adoption.md#platform-015-migration).

The private browser client supports qualified local archives. A clean **0.27.0**
archive and receipt remain at
`target/auth-integration-cleanup.4KahDd/released-browser/target/browser-client-packages/package.HgmuE3/`.
It is local evidence, not durable distribution or a pending-0.28 release. Current
Wire regeneration left all five files byte-identical and client checks passed.
Production issuer transport and final consumer distribution remain host work.
[Recorded Chromium acceptance](https://github.com/dragginzgame/ic-auth/issues/3#issuecomment-6464039789)
has its original Linux/browser/origin/fixture limits; no new Chromium run is claimed.

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns production
integration. Earlier inspection found Auth 0.26 adoption, denied access and a
test-gated adapter; this batch did not refresh or mutate Canic. Its persistent
receiver/enrollment PRs need consumer-owned integration and qualification.

## Latest evidence and limits

`target/persistence-testkit-shared.5p1kVf/` retains registry metadata, the reviewed
snapshot impact/export/provenance, source comparisons and tool preparation.
Ten storage tests pass, including old-ledger/format refusal without clearing.
Wasm host Clippy/build and Rust 1.91 pass; five live upgrade cases cover real
certificates, protected checkpoints, portable proof authority, signed ingress and
SDK admission/logout. Native runner Clippy, dependency pins and public/host graphs
pass. Auth's local/mocked release/publication caller qualification also passes,
including interrupted attempts and recovery (`release-tools.log`).
No full CI, native macOS or new real-browser qualification is claimed.

Earlier focused evidence remains in `target/publication-wire-adoption.bAvc09/`
(Wire, 69 client tests, stored bytes, #123 interruption/recovery),
`target/host-build-adoption.SvfUvK/` (Host/Build callers), and
`target/auth-integration-cleanup.4KahDd/` (66 core/three backend cases and live host
qualification). Initial failures and corrected reruns remain retained. Native
conformance stays outside public libraries and the Wasm host. The
[released handoff](https://github.com/dragginzgame/ic-auth/blob/81ff5379f39128b2d7f11ea4217b4f2e7e674d1f/docs/status/current.md)
retains older adoption facts; historical paths do not prove artifact availability.

## Retained-state boundaries

Current `AUTHNAT3`, session/login `0x415554485311`/`0x415554485313` and replay
`0x415554485302` formats remain. Earlier retirement consent grants no new reset
permission. Older Memory allocation-ledger backings require a reviewed transition;
never erase only a ledger, reinstall or discard backing to obtain admission.
Preserve retained failures, binaries, backings and release/recovery artifacts.
Hosts own clock, allocation, authority refresh, atomicity and certification.
Cryptographic checks, TEE/non-TEE trust and replay/charging rules are unchanged.
Reference qualification establishes no production wallet service or deployment.
