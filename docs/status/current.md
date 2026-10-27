# Current handoff — 2026-10-27

## Checkout and selections

The maintainer completed source release **0.19.0** at
`98687a0071680e5db9fbbdd204e0eae61a47da12`; package versions remain 0.19.0.
The pending **0.19.1** batch adopts Shared tooling, optimizes Rust scope/retry work
and adds private certified-proof probes. Package versions, external selections,
public APIs and signed/stored/public Wire formats are unchanged; only fixture
dependencies/private Wire controls were extended for probes. No Auth staging,
commit, release, publication or deployment ran.

Auth selects Platform/macros **0.7.0**, Host/Build/Testkit **0.69.0**, Memory
**0.51.10** and Wire/derive **0.7.18**. Public Principal consumers align to
Platform 0.7; passive callers select only `principal` with defaults disabled.
Runtime adapters retain explicit upstream Principal conversions and checked Wire
codecs. Native qualification remains outside the public library graphs.
Rust **1.91** is the package minimum; development formatting/Clippy use **1.99.1**.

## Latest Shared Tooling adoption

The exact latest reviewed upstream revision is
`6b6bbd9c9e5798f0ecf6105637392a7a55cd450e` (**0.26.25**), exported from a clean
isolated checkout through canonical impact/refresh. The snapshot contains **126
files**. The initial reviewed source `16efc62` had an old `VERSION` label;
[Shared #316 — Reconcile VERSION with the 0.26.25 commit and changelog](https://github.com/dragginzgame/shared-tooling/issues/316)
records the producer correction adopted here. No consumer override was added.
[Auth #92 — Adopt latest Shared Tooling revision](https://github.com/dragginzgame/ic-auth/issues/92)
owns this adoption; evidence is in `target/shared-tooling-02625-20261027/`.

The hook now preserves opaque gitlinks, and the formatter supports declared
standalone sources. Auth has no tracked submodule, standalone formatter selection
or override; its grouped-sort/Rustfmt coverage is unchanged. Explicit module paths
continue to disable result reuse, and staged hooks always run formatters. Auth
already exports the Node prerequisite guide. The runtime panic scanner is not
selected or compiled here; its Clap/macOS fixes add no Auth compiler package or gate.
Snapshot integrity, actual formatting, staged/partial-stage preservation, local/
mocked release/recovery adapters and documentation links pass. Behavior evidence
uses `16efc62`; final revision `6b6bbd9` changes only producer `VERSION`, with no
payload changes. Final snapshot verification binds the corrected revision.
Generic producer and product/runtime suites, full CI and native macOS were not
run for this contributor-tooling update.

## Measured Rust optimization

[Auth #93 — Reduce repeated Rust admission work with measured Wasm qualification](https://github.com/dragginzgame/ic-auth/issues/93)
records shared scope lookup and removal of the extra exact-retry session clone.
Ordered lookup follows existing canonical validation; independent input, live
authority, cryptographic and storage checks remain. Fixed-clock before/after
PocketIC profiles retain identical keys, topology and signed timestamps. At 16
roles/32 scopes and 16 requested scopes, admission uses about **20%** fewer
instructions and authorization **12.8%** fewer. Exact retries improve **0.8–1.2%**;
small-scope authorization adds **22 instructions**. Reference Wasm is **1,058
bytes smaller** (2,304,928 to 2,303,870; code section down 319 bytes).
`target/rust-perf-20261027/` retains artifacts, comparison JSON and failed/candidate
evidence. The larger validation-split candidate was discarded. Focused 113 runtime
cases, session-only tests/doctests, native/Wasm Clippy and Rust 1.91 checks pass;
the matched live certified-upgrade/profile case passes. Testkit was explicitly
prepared for current compiler inputs. Full CI, macOS and production load were not run.

The measurement-only follow-up is [Auth #94 — Certified-proof admission measurements](https://github.com/dragginzgame/ic-auth/issues/94).
`target/certified-profile-20261027/` retains three component samples, artifact/input
identities and operational scenarios; [the report](../design/0.6-portable-authentication.md#certified-proof-operational-profile--2026-10-27)
explains overlap and scope. Two BLS checks consume about **98.8%** of the 440.25M
complete-verifier instructions. Fresh receiver admissions/renewal cost 440–454M;
valid retries avoid full verification. No production traffic target is set, so
practical impact is undetermined. The maintainer accepted retaining the existing
verifier and admission cost on 2026-10-27; keep the private probes, with no verifier
optimization, result caching or trust-model change. Public library digests are
unchanged within this task. Live profiling/refusal, fixture/Wire tests, Clippy, Wasm graph
boundaries and Rust 1.91 fixture checks pass. Measurement review is complete;
further performance work requires a concrete production workload requirement.

## Released Auth contract and prior evidence

[Auth #90 — Consolidate admission policy and simplify client and qualification state](https://github.com/dragginzgame/ic-auth/issues/90)
records the 0.19 Rust API cut: complete-credential admission owns verification;
`PortableAuthority` separates shared current authority from proof constraints;
immutable `SessionReader` and explicit admission/logout writes replace staging;
ledger state/bindings and TTL policies are typed. See the
[migration](../sessions.md#session-commit-write-migration) and
[Canic adoption contract](../canic-adoption.md). Signed claims, Wire frames,
stored records and browser journal layouts are unchanged and need no reset.

Earlier hard-cut evidence remains in `target/hard-cut-20261027/`: 105 runtime
cases, three invariant doctests, six fixture unit cases and six native lifecycle
cases, selected Clippy and Rust 1.91 native/Wasm checks. Earlier frontend and
release-adapter evidence remains in `target/complexity-20261027/`; Shared 0.26.23
adoption evidence is in `target/shared-tooling-02623-20261027/`. Each retains its
own inputs, failed attempts and qualification limits. The
[preceding handoff](https://github.com/dragginzgame/ic-auth/blob/98687a0071680e5db9fbbdd204e0eae61a47da12/docs/status/current.md)
retains the earlier Platform/Tooling selections and exact evidence paths.

## Ownership and remaining acceptance

Canic owns fleet approval/enrollment, endpoint guards, deployment and its durable
host adapter. [Canic #491 — Extract reusable authentication, token and session machinery](https://github.com/dragginzgame/canic/issues/491)
retains production receiver/store/reset/load and released-API adoption; the latest
[prospective adapter handoff](https://github.com/dragginzgame/canic/issues/491#issuecomment-6442148823)
does not claim deployment. [Canic #507 — Non-issuing reconciliation qualification](https://github.com/dragginzgame/canic/issues/507)
retains issuer/reconciliation policy and workload acceptance. No standalone wallet
service is implemented. Host session/replay/ledger writes and authority generations
must remain atomic; clock, certification publication and memory allocation stay host-owned.

The private browser client selects `@tokostudios/ic-platform` **0.1.1** over SDK
Core **6.1.1**. [Auth #3 — Real-browser portable admission/logout qualification](https://github.com/dragginzgame/ic-auth/issues/3)
retains actual-browser storage and signing-custody acceptance. Node/Testkit transport
fixtures establish neither production issuer adoption nor browser/TEE qualification.

The fixture still uses Testkit 0.69.0 client-before-server teardown fields.
[Tooling #257 — Expose mutable client access while retaining PocketIcInstance server custody](https://github.com/dragginzgame/ic-tooling/issues/257)
and [Tooling PR #258 — Allow mutable Testkit client access without splitting server ownership](https://github.com/dragginzgame/ic-tooling/pull/258)
identify the supplying accessor; wrapper adoption requires a separate published
selection containing it. Normal sibling checkouts remain untouched.

[Auth #85 — Qualify the crypto-bigint advisory](https://github.com/dragginzgame/ic-auth/issues/85)
and [Platform #50 — Crypto capability qualification](https://github.com/dragginzgame/ic-platform/issues/50)
retain the crypto advisory obligation. Preserve release recovery plans, original
artifacts, backups and earlier failed evidence. Normal releases start fresh;
exact-attempt `release-resume` uses its saved source without discarding state.
