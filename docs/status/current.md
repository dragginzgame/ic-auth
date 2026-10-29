# Current handoff — 2026-10-29

## Delivered base and working selections

Auth HEAD is **0.19.6**, `6562624f198262fa5772f9b6a1ddf48d2c42ccd6`.
The [release-source handoff](https://github.com/dragginzgame/ic-auth/blob/6562624f198262fa5772f9b6a1ddf48d2c42ccd6/docs/status/current.md)
retains the preceding adoption and Canic contribution evidence. Current source
still has Rust **1.91** as its package minimum and development Rust **1.99.1**.
Shared Tooling remains **0.27.3** at exact
`cc81a58a5b2dd287c577cbffd2c576c178b6785c`, **127 files**.

Current selection: Memory/Storage **0.53.1**, Platform/macros **0.7.4**,
Host/Build/Testkit **0.69.8**, Wire/derive **0.7.21**, browser Platform **0.1.1**
over Core **6.1.1**, and PocketIC **16.1.0**. The incoming lock already supplied
the requested selections; targeted resolution and fetching leave it byte-identical.
Incoming native TLS AWS LC **1.19.0** / sys **0.47.0** updates are also preserved.
Incoming Wire 0.7.21 is retained and qualified below; no unpublished override is used.

## Complexity reduction batch

Pending notes select **0.20.0** because the public session read API changes:
`SessionReader::session` returns `Option<Cow<'_, Session>>`, and
`authorize_session` takes a projection callback inside the checked read snapshot.
Memory hosts borrow; durable readers can return owned decoded records. Canic
must migrate those Rust calls on adoption; no sibling source was edited.
See [session API](../sessions.md) and [consumer contract](../canic-adoption.md).

Role conversions reuse owned labels. Admission carries canonical/context checks
and requested expiration through verification; authorization observes login
binding/denial once. Final mutable reservation checks, atomicity, rejection order,
crypto checks and signed/public Wire/stored bytes remain. Platform's declared
minimum is 0.7.4, supplying canonical DER admission in place of Auth's re-encoding.
The private certified profiler returns typed stage/refusal codes. Valid lifecycle
test builders and recursive generator-input lists now have shared owners.

Linux release fixture: **2,517,111 → 2,499,590 bytes** (17,521 fewer, ~0.70%).
Matched ordinary authorization: **8,745 → 6,324 instructions** (~28% fewer).
The shared CBOR sink alone saves **14,852 bytes**, with **3,309 more instructions**
(~0.13%) for fresh Ed25519 admission and unchanged authorization. Counting remains
allocation-free. These are fixture samples, not universal production savings.
Baseline/current Wasm, focused command logs, failed sandbox startup/subprocess
attempts and native PocketIC state are retained in `target/complexity-20261029/`.
A second-test refusal from reusing a PocketIC state root is retained; its isolated
fresh-state rerun passes. Certified admission/components and real session/browser
admission/logout/ingress/upgrade and retained checkpoint cases pass. Focused Rust rejection, atomicity,
record/vector, adversarial feature, contract/doc tests and strict library/fixture/
runner Clippy pass. The 68 client tests, interoperability and packed consumer pass;
Wire 0.7.21 regenerates all five browser contracts unchanged. No full CI or release
was run; source/notes are unstaged, package versions and incoming lock are unchanged.

## Current shared-library adoption

[#105](https://github.com/dragginzgame/ic-auth/issues/105) owns this batch.
Memory 0.53.1 releases identity claims before destroying their backing allocation,
including refusal/unwinding, while detached handles retain custody. The 0.53
validation-boundary contract and stored bytes remain. Platform 0.7.4's new opt-in
external Candid endpoints/codecs stay disabled on owned public/fixture graphs.

Passing Linux evidence: six checkpoint and four native tooling cases,
strict host/tooling Clippy, release Wasm, Rust 1.91 host/Wasm check, dependency
boundaries/pins, package/external-lock refusals, all 68 client tests, Rust/TypeScript
roundtrips and the packed offline consumer. Its five generated contracts compare
unchanged. Exact Testkit/release helpers were explicitly prepared and admitted.
Four installed PocketIC cases pass: certificates/composed roots, certified proof
authority after upgrade, retained session/denial checkpoints and actual signed
ingress after upgrade. Evidence is retained in `target/shared-adoption-20261028c/`.
This is internal fixture qualification, not Canic production or native macOS
acceptance; the incoming TLS update is not remote TLS endpoint qualification.

The adoption notes/docs remain unstaged and uncommitted; package versions and
incoming lock bytes are unchanged. No full local CI, source push, publication,
deployment or sibling source edit was performed.

## Browser client delivery preparation

Auth 0.19.6 delivers the browser preparation from
[#104](https://github.com/dragginzgame/ic-auth/issues/104).
`make pack-client` produces a local npm archive; `make test-client-package`
also installs and qualifies a disposable offline consumer. The package check is
part of `make test-client`. Explicit client/file-tool preparation is required;
checks never fetch or install producer tools. Cargo retains the shell's inherited
Make jobserver; Node handles npm packaging/consumer commands only.

The private package remains **0.0.0** and unpublished. Archives expose compiled
ESM and public declarations, canonical generated codecs, README and license.
Fresh staging excludes stale outputs, tests, source tooling and dependency trees.
Source inputs are sealed across contract admission, compilation and packing;
the receipt records their hashes, Git identity/dirty observation, actual tools and
npm archive integrity. Dirty candidates must be rebuilt from the intended
delivered commit. Consumer-owned durable archive storage/distribution is still
required; ignored producer output is not that service.

Local Linux package evidence is under `target/client-delivery-20261028/` and
`target/browser-client-packages/`. It covers offline installation preserving
reviewed external selections, strict public declarations without Node globals,
blocked private imports, browser bundling and injected admission/logout with
separate account/browser identities and fake IndexedDB. All **68** existing
client tests and Rust/TypeScript roundtrips pass. The parallel-Make path,
archive-only command and Make/client-setup caller fixtures pass. An earlier
sandbox subprocess refusal is retained;
native execution passes. These are package/client proofs, not real browser,
production issuer, installed Canic or native macOS acceptance.

Current shared-library qualification produces a new dirty-source candidate under
`target/browser-client-packages/package.P15W7k/`; no npm upload occurred.
See [client delivery](../browser-client.md#local-archive-delivery).

## Prior dependency qualification

[#103](https://github.com/dragginzgame/ic-auth/issues/103) retains Memory/Storage
0.53.0 and Tooling 0.69.6 checkpoint/lock/graph/MSRV/installed evidence in
`target/persistence-adoption-20261028/`. [#102](https://github.com/dragginzgame/ic-auth/issues/102)
retains the earlier Wire/Platform and Memory 0.52.1/Tooling 0.69.5 evidence in
`target/dependency-adoption-20261028/`. Prior logs/Wasm and corrected-attempt
failures remain; the release-source handoff above records their boundaries.

## Canic integration boundary

The read-only Canic review found Auth 0.19.5 selected in its working tree, a
reserved unpublished `canic-auth` scaffold, a test-gated shared store adapter and
an application authorization facade that still denies. No missing shared Rust
engine API was established. Canic still needs the `ic-auth-wire` dependency and
actual admission/read/logout bindings; Auth owns the canonical contracts.

[PR #701](https://github.com/dragginzgame/canic/pull/701), source `3c559fd6e`,
supplies retained persistent receiver ownership, ledger/budgets and CI fixture
repairs. Stacked [PR #703](https://github.com/dragginzgame/canic/pull/703), source
`0ab0d1739`, supplies bounded enrollment producer/installer components; its Root
method is reserved, not mounted. Both were open at the last review. Their exact
native/codec/graph evidence is in the PRs and `target/canic-integration-20261028/`.
The isolated clone is there; normal sibling working checkouts remain untouched.

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns integration
and installed persistent receiver acceptance. Root RPC mounting, approved issuer
registration, refresh scheduling, clock/skew/budget policy and endpoint/lifecycle
wiring remain. [#507](https://github.com/dragginzgame/canic/issues/507) owns
production issuance, lookup-only reconciliation and browser integration. Account
authentication, browser ingress, application authorization and resource ownership
remain distinct; software signatures do not establish TEE custody.

Keep the accepted certified verifier/admission cost; no verification cache or
crypto/trust optimization follows. Preserve all build/release recovery evidence.
The [operational profile](../design/0.6-portable-authentication.md#certified-proof-operational-profile--2026-10-27)
and [adoption contract](../canic-adoption.md) retain the governing boundaries.
