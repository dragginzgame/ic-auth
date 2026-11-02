# Current handoff — 2026-11-02

## Delegated 0.32 delivery

The user requested deployment of this batch: prepare source, run the standard
Protocol then Engine minor releases, push their exact commits/tags to origin/main
and publish the three 0.32.0 crates to crates.io. Use the dedicated checkout
`/tmp/ic-auth-release.qoEtNy`; preserve its recovery state and all older attempts.
`target/complexity-032-delivery/` records the frozen source and delivery evidence.
Storage 0.65.2, Build 0.74.1 and ctutils 0.4.8 landed after focused qualification;
the release gates must qualify this frozen selection. Later development updates
stay outside that checkout. No service, sibling or npm deployment is included.
The initial attempt was interrupted to correct the scope-validator documentation;
its `validation.DpoFIl` logs remain retained. Restart fresh after the doc commit.
The next gate (`validation.tf4ZxL`) caught a nested-logging fixture assertion;
the fixture now verifies retained input checkpoints instead of console text.

## Delivered 0.31.0

Protocol **0.31.0** (`ic-auth-protocol-types`, `ic-auth-wire`) is published at
`ae8ae1dc20c26d70a026dc9481a465f4d39338fd`, annotated tag `protocol-v0.31.0`.
Engine **0.31.0** (`ic-auth`) is published at
`38ba68f6ad06735d94566e964c11169bf46413da`, annotated tag `engine-v0.31.0`.
Both release commits/tags were atomically pushed to `dragginzgame/ic-auth` main;
all three exact crates.io archive checksums were verified by the publisher.
Protocol was published before Engine. The user explicitly confirmed these
release, push and registry effects after the initial automatic-review refusal.

Both standard releases passed the complete local CI gate, including native/Wasm
lint, isolated archive/compiler consumers, 70 browser tests, crypto/session tests,
11 live PocketIC tests and the full release/recovery/publication fixture.
Protocol gate: `validation.Zw2P6z`; Engine gate: `validation.OxL1Ze`,
both under `/tmp/ic-auth-release.x4YbcZ/.git/release-state/validation-logs/`.

Keep `/tmp/ic-auth-release.x4YbcZ`, both completed group plans, validation receipts,
publication intents/dispatch observations, archives and raw logs. No resume or
uncertain upload remains. Its clean HEAD stays at the Engine release for recovery.
`target/upstream-delivery/` in development records delivery and original lock input.
Hosted 0.31 observations remain in `target/upstream-delivery/proposed-delivery.json`.

The previous handoff-only commit/push was refused as outside the exact 0.31
authorization. The new delegated 0.32 delivery includes current source and docs;
the completed 0.31 release state stays intact.

## Local canister dependency cleanup

The private canister drops unused direct Metrics and Serde-bytes dependencies;
fixtures own its metrics and Wire owns its certificate envelope bytes. Direct
Metrics policy permits only fixtures and the native runner. Both dependencies
remain transitive, so no Wasm reduction is claimed. Original focused evidence
is under `target/canister-dependency-cleanup/`; this local cleanup is included in
the pending Engine 0.32 notes below, without a package version change or delivery.

## Local complexity hard cut

Audit items 1–9 are implemented locally: one private codec-binding pattern with
canonical per-method guards, one login record representation, consuming checkpoint
restore, borrowed session profiling, owned role projections, checked grant scopes
and 64-byte roles, one private enrollment field roster, shared browser u64 checks
and Cargo-discovered CI manifest sealing. The maintainer confirmed no retained
roles above 64 bytes need preservation. Valid signed/Wire/storage bytes and all
cryptographic/trust checks remain unchanged; no record reset occurred. Malformed
portable scope text now fails as `InvalidShape` during decoding. Both groups have
pending 0.32.0 notes; package versions remain 0.31.0 until standard preparation.

Focused evidence is under `target/complexity-031/`: affected native and Wasm
Clippy, passive engine lint, contract/fixture and session/revocation tests, seven
compile-fail examples, 70 client tests, jobserver/input-seal caller proofs and six
live PocketIC session/certificate/upgrade/ingress/SDK cases pass. Chromium was not
run. Startup-only failures from an unfinished path and a reused create-only log
directory are retained beside their passing fresh-directory retries.
No matched Wasm/latency improvement is claimed. Group-scoped release validation
(audit item 10) stays unselected: this cross-group batch requires the full gate;
omissions need separate same-source, host and receipt qualification.

## Development checkout and frozen selections

Development main remains at the delivered Engine commit; all changes are local.
Confirmed incoming selections are Runtime/macros/guard **0.18.1**, Testkit
**0.80.1**, Wire/derive **0.15.1**, Storage **0.65.1**, Metrics **0.27.2** and Host
FS/Process/Artifacts **0.74.2**. The user confirmed leaving the lock stable for
qualification. The `incoming-*` logs and `incoming-final.lock` under
`target/complexity-031/` bind the passing Rust/lint/client/six PocketIC checks to
that selection; `qualified.lock` preserves the earlier checked selection.
Regeneration and review found all five browser files unchanged. Explicit setup
prepared Testkit 0.80.1 and admitted the same 16.1 server. The independent file
CLI stays pinned to 0.74.1. None of this is release/full-gate/macOS acceptance.
Original/base/released/merged locks remain in
`target/upstream-delivery/development-sync/`; local package versions stay 0.31.0.

The published lock selects Platform **0.18.0**, Types **0.17.0**, Runtime **0.18.0**,
Storage **0.65.0**, Wire/derive **0.15.0**, Metrics **0.27.0**, Testkit **0.80.0**,
Build **0.74.0**, Host FS/Process/Artifacts **0.74.1** and canonical PocketIC
**0.18.0**. The server is **16.1.0** and browser SDK **0.1.1**; the browser lock
and all five generated browser files are unchanged. Published Testkit internally
retains arithmetic-only Metrics 0.26.1; no counter crosses that boundary.
Storage imports Runtime directly on Wasm and Wire now implements the canonical
Types 0.17 schema identity, resolving both earlier supplier blockers. No aliases
or overrides are used. The host retains one Runtime/generator/guard; public
libraries remain runtime, native-SDK, Storage and Candid-free by capability.

Shared Tooling **0.27.28**, `8c7fae6605c69a930e3b1f14e1ff5c4c9dafbcb7`, is
canonically exported and independently source-verified (145 payloads). Optional
group validation/reuse remains unselected; both releases used the full gate.
Rust **1.99.1** is the pinned and minimum compiler. Reviewed README corrections,
cc 1.11.0 and crc32fast 1.5.4 are included. No sibling checkout was changed.

## Storage and authentication boundaries

Storage 0.65 replaces retired Memory/stable-structures in the private checkpoint
host. Its allocator, checksummed collections and B-tree backing require fresh
state. Old manager/ledger images are refused without a reader, fallback or reset;
no retained backing was cleared. A production installation needs its own reviewed
whole-image transition. No production service is deployed in this repository.

The host selects `StorageConfig`, namespace grants, checked `RuntimeMemory` leases
and `adapter::StableCell`. Load bounds cover the 128 KiB Auth payload plus magic
and verify checksums; create/set failures are checked. The host owns atomicity,
clock, protected generation and certification-root publication. The retired
Memory 0.63 header test preserves a separate application sentinel on refusal.
Source/qualification evidence remains under `target/storage-owner-adoption/`,
`target/all-upstream-adoption/` and `target/upstream-retry/`.

Auth's `AUTHNAT3` payload, Wire methods, record bytes and signed material are
unchanged. Preserve session/login IDs `0x415554485311` / `0x415554485313` and replay
`0x415554485302`. Cryptographic checks, protected authority and the TEE/non-TEE
trust model are unchanged; no verifier rewrite or verification-result cache exists.
Auth's public Principal/Wire Rust identities change, so consumers must align with
Protocol 0.31, Types 0.17, Wire 0.15 and Platform 0.18 at their selected boundaries.

## Historical acceptance and consumer ownership

[Auth #124](https://github.com/dragginzgame/ic-auth/issues/124) is closed against
`28ee29d12122cfbfe05b19a49460bd5ee79bf7c8` and the all-host passing
[CI 41434941116](https://github.com/dragginzgame/ic-auth/actions/runs/41434941116).
Retain all historical artifacts and evidence recorded in the preceding handoff
at `d08ae8916dca83b6583e1550173328b877ddd41f`; no cleanup is authorized. Its
older matched Wasm observation does not establish a saving for this release.

Protocol 0.30.0 remains at `ec0233f3c4d746bb9bf7009f7432683a57c0b616`,
Engine 0.30.0 at `80459e300cdb6fb67a94ff45d33f0774ebf0bd9f`, with their group
tags. Preserve historical journals/receipts and the recorded original checkout
`/tmp/ic-auth-release.EqLgnm` if present. Historical recovery uses its original
adapter, saved source/group/version and storage formats.

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns installed
adoption, issuer/renewal, guards, durable backend and browser acceptance; #720 owns
storage deduplication. No Canic source, production issuer/backend, npm publication
or live-IC deployment changed here. Consumer adoption/qualification stays there.
