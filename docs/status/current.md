# Current handoff — 2026-10-28

## Delivered base and working selections

Auth HEAD is **0.19.5**, `100d9b2fe336446d090db4db33a000f35cf55dce`.
The [release-source handoff](https://github.com/dragginzgame/ic-auth/blob/100d9b2fe336446d090db4db33a000f35cf55dce/docs/status/current.md)
retains the preceding adoption and Canic contribution evidence. Current source
still has Rust **1.91** as its package minimum and development Rust **1.99.1**.
Shared Tooling remains **0.27.3** at exact
`cc81a58a5b2dd287c577cbffd2c576c178b6785c`, **127 files**.

Memory/Storage **0.53.0**, Platform/macros **0.7.3**, browser Platform **0.1.1**
over Core **6.1.1**, and PocketIC **16.1.0** remain selected. Incoming lock-only
Tooling **0.69.7** and Wire/derive **0.7.20** edits are preserved. The browser
generator was explicitly rerun/reviewed for Wire; all five outputs are unchanged.
This batch qualifies browser packaging and its generator caller, not all native
Tooling 0.69.7 callers or an installed fixture on that new graph.

## Browser client delivery preparation

The maintainer requested a head start on browser delivery for Canic.
[#104](https://github.com/dragginzgame/ic-auth/issues/104) owns the Auth changes.
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

The edits are unstaged/uncommitted; Auth/npm versions are unchanged. No upload,
publication, deployment, merge, sibling edit or full local CI was performed.
See [client delivery](../browser-client.md#local-archive-delivery).

## Prior dependency qualification

[#103](https://github.com/dragginzgame/ic-auth/issues/103) retains Memory/Storage
0.53.0 and Tooling 0.69.6 evidence: six checkpoint cases, four native tooling
cases, package/lock refusals, strict fixture Clippy, release/MSRV Wasm, graph
checks and four installed certificate/upgrade/ingress cases. The host uses one
Memory-admitted allocation and Storage's governed cell, retaining the 128 KiB
pre-load ceiling and exact bytes. Memory 0.53 trusts exclusive backing custody
between construction/lease-open/growth checks. Restore remains closed until
fresh protected authority; no raw aliases are used during collection access.
Evidence and the previous Wasm remain in `target/persistence-adoption-20261028/`.

[#102](https://github.com/dragginzgame/ic-auth/issues/102) retains earlier
Wire/Platform and Memory 0.52.1/Tooling 0.69.5 evidence, including the original
68 browser tests, transport/signature checks and four installed cases. Its
retained corrected evidence-directory failure is not a runtime defect.
Historical logs remain in `target/dependency-adoption-20261028/`.

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
