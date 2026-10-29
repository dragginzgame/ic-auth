# Current handoff — 2026-10-29

## Delivered base and local scope

Auth **0.21.0**, `353ed96bd77e9eeace3146d5b75df0523932ecb4`, is delivered on main.
Its [complete CI matrix](https://github.com/dragginzgame/ic-auth/actions/runs/40858572448)
passes Linux, macOS Intel and Apple Silicon. Issues #106/#107/#108 are closed.
The [delivered handoff](https://github.com/dragginzgame/ic-auth/blob/353ed96bd77e9eeace3146d5b75df0523932ecb4/docs/status/current.md)
retains earlier contracts and qualification history. Release/recovery state is untouched.
Rust **1.91** remains the minimum; development uses **1.99.1**. Shared Tooling
remains **0.27.6**, exact `5a72359b5be19d5221334cf2c9b3204621d3f3f6`, 127 files.

The maintainer authorized cleanup, confirmed no session/replay/login records need
preservation, then requested new Platform and latest Tooling/Wire/Memory/Storage.
Pending changelog is **0.22.0**; package versions remain 0.21.0. No staging,
commit, push, full CI, release or publication ran. Owning issues:
[#109 storage](https://github.com/dragginzgame/ic-auth/issues/109),
[#110 dependencies](https://github.com/dragginzgame/ic-auth/issues/110),
[#111 cleanup](https://github.com/dragginzgame/ic-auth/issues/111) and
[#112 Platform](https://github.com/dragginzgame/ic-auth/issues/112).
Delivery and required acceptance remain separate.

The maintainer subsequently committed the batch as `2d41d8a` (`0.22.0`); the
reported release verification failed all-features Clippy in the unpublished
feature-unification harness. Shared revocation fixtures had moved to Wire/Serde,
but its dev dependencies still supplied CBOR. The local repair declares Wire and
Serde, removes unused CBOR and makes the normal harness test selection use all
features. Focused all-target/all-features Clippy and all 31 signature/revocation
tests pass; logs are under `target/feature-unification-codec-20261029/`.
External lock selections and public library code are unchanged. Full CI was not
rerun, and the retained failed validation/release evidence is untouched.

## Current shared dependency adoption

Official sparse-index checks select Tooling **0.70.0**, Wire/derive **0.7.22** and
Memory/Storage **0.55.2**. Wire 0.7.23 was an unpublished draft at selection.
The incoming lock already selected Wire 0.7.22 and Memory/Storage 0.55.2;
requirements now declare those floors. Build/Host/Testkit advance from 0.69.14
to 0.70.0, and the independently pinned file CLI advances from 0.69.11 to 0.70.0.
All selected workspace Platform dependencies now use **0.8.1**; the old private
Platform 0.7 branch is removed. Metrics **0.17.3**, browser Platform **0.1.1** /
Core **6.1.1**, PocketIC **16.1.0**, and unrelated exact dependencies are preserved.

Published Wire/derive 0.7.22 and Memory/Storage 0.55.2 source trees are identical
to 0.7.21/0.55.1. Tooling 0.70's breaking ledger/Registry type identities are not
used by Auth; Testkit explicitly selects external Principal conversions, preserving
its existing API. Stable substrate, persisted formats and external identity bytes
remain unchanged. No Rust source repair or additional data retirement was needed.
The file, Build and Testkit CLIs were explicitly prepared/admitted; PocketIC
server selection stays Testkit-owned. Saved older tool selections remain intact.

Evidence is retained under `target/shared-070-20261029/`: official index snapshots,
before patch/manifests/lock, selected lock, source comparisons, generated-file
review, artifact, setup/check logs and focused qualification. All five browser
files were regenerated through the normal command and reviewed unchanged. Current
record/session/revocation tests (62), checkpoint tests (9), metrics tests (5),
native tooling cases (4), transport cases (13), 68 client/interoperability/packed
consumer cases, graph checks and native tooling/metrics Clippy pass. Rust 1.91
checks pass Wire Wasm, qualification host Wasm and native tooling. Testkit caller,
package/external-lock and evidence caller fixtures pass. Four installed final-Wasm
certification/composed-root/checkpoint-upgrade/SDK-ingress/logout cases pass.
Local/mocked release/recovery/publication caller fixtures also pass. No current
full CI, macOS, new Chromium or production consumer acceptance is claimed.

Current raw reference Wasm is **2,199,756 bytes**. Installed profiles remain small
Ed25519 admission **2,619,205**, retry **57,894**, ordinary authorization **6,835**,
wide authorization **29,588**, representative session **364 bytes** and decoding
**28,369 instructions**; certified admission remains about **440 million**.
These are fixture observations, not a production workload estimate.

## Platform boundary and retained cleanup

Protocol/Wire keep only Platform `principal`, without `ic_principal`, SHA-2,
Candid or runtime. The engine explicitly selects `principal-self-authenticating`
for `browser_principal`; verification/runtime capabilities stay separate.
Consumers sharing Principal, parse errors or imported key types align to Platform
0.8; explicit upstream conversions require `principal-external`. See
[migration](../canic-adoption.md#platform-08-migration). Earlier 0.8.1 qualification
is retained under `target/platform-081-20261029/` and #112. Auth selects neither
changed ICRC DTOs nor native profiler product code. Signed preimages, verification
rules, TEE trust, authority/atomicity and cache policy remain unchanged.

Private session/replay/login records use native Wire identities with framed
2048/512/1024-byte ceilings, 1024 nodes, depth 16 and a 65,536-byte native allocation
allowance. Typed login state replaces flags. The approved hard cut refuses old
CBOR without migration/reset; the fixture uses `AUTHNAT2` / `0x41555449f101`.
Hosts adopt one coherent checkpoint, preserving replay/denials. Transport
`PortableProof::Certified` holds `Box<CertifiedProof>`; direct Rust constructors
box it, while encoded/generated contracts retain their bytes.

The seven completed cleanup changes and three retained candidates are documented
in #111. Original matched savings were **299,159 bytes (12.0%)** smaller Wasm,
representative records **691–696 → 364 bytes**, decoding about 28% cheaper.
Those savings predate these dependency updates. Keep owned/pre-commit Session
copies, the canonical byte encoder and intentionally different native fixtures.
Earlier cleanup evidence directories were removed outside these tasks; their
original findings remain in #109/#110/#111.

## Consumer ownership

Retain `ValidatedGrants` from protected configuration; reconstruct after decoding
and replace atomically with authority/generation. The host owns clock, stable
allocation, complete atomic commits and certification publication. Canic records
Auth outcomes in its shared metrics store and existing endpoint; no separate
Auth metrics subsystem or per-producer endpoint is needed. The browser package
remains private/unpublished. No production issuer adapter or wallet service exists.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns receiver acceptance,
issuer approval, refresh, budgets, lifecycle and endpoint wiring;
[#507](https://github.com/dragginzgame/canic/issues/507) owns issuance/reconciliation
and browser integration. Earlier PR #701/#703 reviews were read-only; current
status was not rechecked. Sibling checkouts remain untouched. Authentication,
ingress delegation, application authorization and resource ownership stay distinct;
software proof does not establish TEE custody.
