# Current handoff — 2026-10-20

## Source and pending release

Released baseline is Auth 0.11.0
`5cfc065df474e8dec0d4672130ed963df4b7e23a`. Own package versions remain
0.11.0; compatible pending notes select **0.11.1**.
Preserve incoming Wire 0.5.0, Testkit 0.57.0, Platform 0.4.0, Host FS/Process
0.31.1, Memory 0.49.1 and Build 0.22 selections. No sibling/path override is used.

Platform/macros **0.4.0** and Host FS/Process/Artifacts **0.31.1** are verified
non-yanked published releases with checksum-matched archives. Platform's source
is `26a07c510ab74413ec30779cd69af84db13cff11`. Host 0.31.1 published during
qualification and the lock advanced concurrently. All three new archives are
verified; their source matches 0.31.0 exactly. Final Host 0.31.1 checks pass;
earlier 0.31.0 results remain separate evidence. Empty selected features
still exclude capability/tempfile backends. Older native transitive Host versions
remain with their owning Build/Testkit packages.

Platform packages the original signature creation/verifier implementations, with
provenance preserved and unused store/Candid imports removed. Its signature-key
identity is new; its Principal field remains external, so existing conversions
stay necessary. Auth uses none of the retired runtime Candid conveniences or
RejectCode conversions. DER/CBOR/signed bytes, admission bounds, protected
signer/seed/root/freshness and runtime/controller behavior remain unchanged.
Every public capability and isolated package graph now forbids Candid, raw IC
calls and retired registry signature packages. Reference Wasm also forbids
Candid; its runtime retains one executor and IC0. Native Agent/Testkit still bring
Candid and independent Platform versions. Do not claim their removal from Cargo.lock.

Owned Principal, generic decode bounds and the Wire 0.5 dependency cut remain
part of released 0.11.0. Frozen Auth 0.10.7 Wire frames, signed claims and
session/replay/login records keep exact bytes; no journal/checkpoint reset or
storage migration is authorized. See the
[consumer migrations](../canic-adoption.md#platform-04-migration).

Shared Tooling **0.25.4** is adopted through the canonical 118-file snapshot at
`db555ab6cf4a93af9a183f18d921d3b0ee97c725`, verified against remote main.
It clarifies canonical failure-evidence collection and redirected-parent handling;
selected scripts/build-helper identity are unchanged from 0.25.3. Existing
prepared common Rust tools, Testkit 0.57/server 16.1.0 and Build 0.22 stay selected.
Checks/recovery remain offline; fresh admitted release preflight owns preparation.
No release effect is authorized by adoption.

## Ownership and consumer acceptance

Auth owns proof/certificate semantics, checked types, local session/replay/denial
machinery and bounded Wire transport. The private client coordinates admission
and durable logout through injected identity, issuer and storage boundaries.
There is one application flow: proof admission, then caller-bound session checks.
Platform owns runtime/crypto integration; Wire owns codecs/generation; Memory
owns memory primitives; Host/Build/Testkit own native mechanics and qualification.
Host/Build/Testkit remain outside deployed library graphs.

Canic owns fleet/member/role authority, protected issuer enrollment/renewal,
actual guards, caller/clock acquisition, root publication and atomic host stores.
Toko owns account linking and application assets; neither is an Auth dependency.
See the [ownership design](../design/extraction.md) and
[consumer contract](../canic-adoption.md). Canic can finish against published
0.11.0 without waiting for qualification-only PRs.

The real SDK two-receiver journey injects lost issuer replies, reopens fake
IndexedDB and retains uncertain issuance until original credentials/deadlines
can be recovered without reissuance. It covers exact retry, proof-free reads and
logout through local canisters. Issuer lookup is injected; production receipts,
browser storage and durable backends remain outside that evidence.
Production acceptance belongs to [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58),
[#59](https://github.com/dragginzgame/ic-auth/issues/59) and
[Canic #491](https://github.com/dragginzgame/canic/issues/491)/
[#507](https://github.com/dragginzgame/canic/issues/507).
[Auth #78](https://github.com/dragginzgame/ic-auth/issues/78) retains Host CLI
cutover; Shared's feature-aware installer prerequisite is available, but this
adoption does not change runnable file commands.

## Qualification and retained evidence

Final Platform 0.4 / Host 0.31.1 checks pass: isolated Rust 1.91 native/Wasm
package matrix and apps;
Wasm/native Clippy; protocol/type/Wire, signature/certified/store, session/restore
and frozen record tests; all 11 Host-tooling and 47 browser cases. Five native
fixture tests, release Wasm and all seven installed Testkit 0.57/server 16.1.0
cases pass, including certification, upgrade, signed ingress and SDK admission/
logout. Actual graph exclusions and negative fixtures, grouped formatting and
staged preservation, snapshot/pins/links, Make jobserver, tool failure evidence
and ShellCheck pass. Offline tool admission and the complete local/mocked
release/recovery fixture pass without reinstallation. The release artifact digest
is in `qualification-0311-wasm.sha256` under the evidence root below. No new API gap
or unresolved in-repo exclusion was found; external deployments retain their own
source/lock/target/artifact admission.
Current before-images, verified archives, graph observations, clean Shared source
and logs are under `target/upstream-040-031-0254-20261020/`. The `before/` tree
retains all incoming tracked and untracked work. The concurrent Host 0.31.1 lock
is retained as `incoming-host-0311.lock`; final `*-0311*` logs qualify that selection.
Cargo.lock remains byte-identical to it. No evidence artifacts are deleted.

[PR #79](https://github.com/dragginzgame/ic-auth/pull/79) merges released
0.11.0 into its existing contributor history. It retains typed Wire calls and
Platform-owned application Principals, converting only at the upstream Agent
boundary. Additional cases check wrong-root refusal, protocol versus application
failure and cancellation after the complete update was dispatched. Local
cancellation does not establish that a remote update did not execute. Refreshed
Platform 0.4/Testkit 0.57 Linux qualification passes all three ingress cases
against a fresh release Wasm, Rust 1.91 all-target checking, strict Clippy, actual
native/Wasm exclusions, formatting, pins, links and snapshot. Hosted native macOS
qualification remains pending. Evidence is under `target/pr-followup-20261020/`.
Historical narratives remain at `debe7f5:docs/status/current.md`, in
[handoff history](history-through-0.9.1.md) and
[adoption history](../canic-adoption-history.md); these are evidence, not alternate
APIs or task queues. No full CI, native macOS, production issuer/deployment,
release/publication or main-branch Git effects are claimed by this adoption.
