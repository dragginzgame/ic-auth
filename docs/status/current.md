# Current handoff

## Delivery and current work

The maintainer confirms Auth **0.25.0** is live. Published tag `v0.25.0` resolves
remotely and locally to `d3f2277c7a6c9dd778d80399a99025b99c3035c4`.
The checked-scope MSRV fixture repair is included in that release.
The complete local release gate passed all 28 stages at source `0fde575`;
its sealed evidence remains in `.git/release-state/validation-logs/validation.RIYT7k/`
and `.git/release-state/validation-0fde57548dcfda23db830e10d6b83d2ca6e4007d-0.25.0.json`.
Preserve all release/recovery files and failures.

The maintainer requested issue reconciliation and remaining repairs. The full
three-host [CI matrix](https://github.com/dragginzgame/ic-auth/actions/runs/41054237679)
for `d3f2277`, attempt 1, now passes Ubuntu, macOS Intel and macOS ARM.
The nine delivered trackers were closed as completed with this acceptance
evidence. Rejected experiments remain documented outcomes; old-backing custody
and consumer deployment obligations are not relaxed by closure. GitHub issues
remain the tracker; local #116/#119 changes are not qualified by release CI.

[#116](https://github.com/dragginzgame/ic-auth/issues/116) now has a local cleanup:
qualification certificate envelopes and signature fixtures use `ic_wire::ByteBuf`,
the exact re-export of `serde_bytes::ByteBuf`. The engine declares Wire explicitly
as a dev dependency so minimal signature-test features do not enable sessions.
Other Serde byte adapters remain; no crypto, certificate bytes, public capability
selection, runtime policy or record identity changes. Pending notes target
**0.25.1**; package metadata stays **0.25.0**. No staging/commit/push/release,
publication, deployment or sibling source edits were performed here.

The requested [Metrics adoption](https://github.com/dragginzgame/ic-auth/issues/119)
selects published **0.18.0**, with a root-owned requirement and only that external
lock identity changed from the captured incoming selection. The new release adds
Summary/Histogram Serde/Wire projections; Auth uses the unchanged counter schema
and arithmetic. No caller migration, public library Metrics dependency, endpoint,
registry, stable metrics store or Auth record migration is introduced.

## Dependency selection and evidence

Shared Tooling remains **0.27.6**, revision
`5a72359b5be19d5221334cf2c9b3204621d3f3f6`. The release selects Platform/macros
**0.8.3**, Tooling/Build/Host/Testkit **0.70.4**, Wire/derive **0.7.24**,
Memory/Storage **0.56.1**, Metrics **0.17.7**. An incoming local lock update to
Memory/Storage **0.56.2** was preserved; #116 checks use that captured lock and
are not a complete dependency-adoption qualification. Rust minimum remains
**1.91**, development **1.99.1**, and prepared PocketIC **16.1.0**.
Current local Metrics is **0.18.0**; the released **0.17.7** selection above is
historical. Adoption evidence is in `target/metrics-adoption-20261030/`.
All five admission-counter tests and eleven native host tests pass, including
wide totals/overflow, malformed projections, budget/reset independence and custody.
Affected native/Wasm Clippy, focused Rust 1.91 checks and library graph exclusions
pass. Installed admission/retry/logout/signed-ingress and checkpoint-upgrade cases
pass with the accepted artifact. No full CI or macOS result is claimed for this
uncommitted adoption; the earlier release matrix is a separate source identity.

Current #116 evidence is in `target/bytebuf-116-20261030/`: captured lock,
focused certificate/signature tests, Clippy, Wasm and installed certification logs.
Fourteen signature and 29 certified cases pass, plus the installed certified
admission/live-authority/upgrade and exact Rust/TypeScript proof-byte roundtrip.
The first narrow Clippy attempt exposed an existing sessions-only helper warning
when sessions are disabled; the maintained all-feature lint selection passes.
Do not claim a new full CI or native macOS result from these local checks.

The released #117/#118 measurements and migration decisions are preserved in
[release-source history](https://github.com/dragginzgame/ic-auth/blob/d3f2277c7a6c9dd778d80399a99025b99c3035c4/docs/status/current.md)
and their owning issue discussions. Earlier `target/cleanup-025-20261030/` and
`target/hard-cuts-025-20261030/` evidence must be retained where still present.
Rejected boxed-grant/borrowed-checkpoint/per-byte-array trials remain rejected;
no new size or performance saving is claimed for import consolidation.

## Persistent safety and ownership

Released session/login records use fixed 32-byte audience IDs and identities
`0x415554485311`/`0x415554485313`; replay remains `0x415554485302`.
The private checkpoint is `AUTHNAT3`. The maintainer separately confirmed no
retained current Auth session/login records. Retired identities, wrong lengths,
relabeled text bodies and `AUTHNAT2` checkpoints refuse without clearing.
See [storage/custody](../sessions.md) and [consumer migration](../canic-adoption.md).
Older Memory allocation-ledger backing remains incompatible even with `AUTHNAT2`;
retain old binaries/backings and require an explicitly reviewed transition.
Never erase only a ledger, reinstall or discard backing to pass admission.

No cryptographic algorithm, signed preimage, certificate/delegation check,
TEE/non-TEE trust assumption or verification-result cache changed. Raw proof
grants retain deferred validation for exact retries. Hosts own clock/protected
authority/generation, allocation, atomic commits and certification publication.
Canic owns endpoint/issuer/lifecycle acceptance in
[Canic #491](https://github.com/dragginzgame/canic/issues/491) and browser issuance
in [#507](https://github.com/dragginzgame/canic/issues/507). No production wallet
service, issuer adapter or consumer adoption is established by these fixtures.
