# Current handoff — 2026-10-20

## Source and selections

Local main is fast-forwarded to `d36f61ebb1c9f3e5ea7331f4f18e9ee62146da72`,
which merges PR #80 (Host file CLI) and PR #79 (caller-owned ingress tests).
All three host checks passed for both exact heads; main has the same combined
source tree. Its post-merge CI remains separate evidence. Auth #78 is closed.
Own package versions remain 0.11.0; compatible pending notes select **0.11.1**.
No commit, release or publication of the pending adoption is authorized.

The incoming dependency selections and lock are preserved: published Build
0.23.0, Host FS/Process/Artifacts 0.31.2, Metrics 0.14.10, Platform/macros 0.4.1,
Testkit 0.59.0, Memory 0.49.2 and Wire 0.5.0. Nine supplying archives are
checksum-matched to the lock and verified published/unyanked. No sibling/path
override is used. Root retains the merged ingress tests' Tokio sync/time features.
Build's Auth-used external-lock interface still compiles unchanged; its new
command-observation APIs require no consumer wrapper or new local implementation.

Shared Tooling **0.25.6** is adopted through the canonical 118-file snapshot at
`84a8742c7c2e3f5611647f9c0f67fed049960327`. The reviewed source includes focused
fixture-export qualification and retained Cargo metadata failure diagnostics.
The selected payload delta is the metadata helper and its release documentation.
Metadata failures retain physical root inputs, stdout/stderr and separate Cargo/
logger statuses without retrying or repairing. Existing installer/build policy
is unchanged; no common-tool reinstall is inferred from this snapshot refresh.

The native file CLI stays independently pinned at **0.31.1** in `ci/host-file.env`;
Host libraries select 0.31.2. Shared owns setup/admission, Host owns bounded file
mechanics, Auth owns byte/count limits, publication intent and recovery decisions.
No local file implementation or runtime fallback remains. Fresh release preflight
prepares selected tools; ordinary checks and recovery admit them offline.
The locked Build 0.23 editor and Host CLI currently need explicit setup. The
maintainer's Testkit 0.59/server 16.1.0 setup authority remains in effect.

## Ownership and remaining acceptance

Auth owns reusable proof/certificate semantics, checked types, local session/
replay/denial machinery and bounded Wire transport. The private client coordinates
admission and durable logout through injected identity, issuer and storage
boundaries. Application authentication has one flow: proof admission, then
caller-bound session checks. Platform owns runtime/crypto integration, Wire
codecs/generation, Memory primitives, and Host/Build/Testkit native mechanics.
Native dependencies stay outside deployed Auth graphs. Public capabilities and
the reference Wasm refuse Candid; native Agent/Testkit still retain it.

Canic owns protected fleet authority/enrollment, production scoped guards,
runtime caller/clock, root publication and atomic stores. Neither Canic nor Toko
is an Auth dependency. Frozen 0.10.7 signed/frame and session/replay bytes remain
unchanged; the released 0.11 Rust identity migration is documented in
[consumer migrations](../canic-adoption.md#platform-04-migration).
Production acceptance stays in [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58),
[#59](https://github.com/dragginzgame/ic-auth/issues/59) and
[Canic #491](https://github.com/dragginzgame/canic/issues/491)/
[#507](https://github.com/dragginzgame/canic/issues/507). Reference fixtures do not
establish production issuer receipts, browser persistence or durable backends.

## Qualification and evidence

Rust 1.91 all-target tooling/runner checking passes with the new Build selection.
Protocol/types/Wire, chain-key, portable/adversarial, canister signature/store,
certified proof and session/restore tests, strict native/Wasm Clippy and Wasm
capability checks pass. Actual native/Wasm deployment exclusions and negative
fixtures, grouped formatting and staged preservation, snapshot/pins/links, tool
failure evidence, Testkit caller, Make jobserver and substituted release runner
checks pass. Executable-dependent file/release/MSRV/live qualification remains
pending setup; do not substitute the earlier pairing's results for it.

Evidence and incoming before-images are retained under
`target/adoption-0256-20261020/`; earlier Testkit 0.59/Platform 0.4.1 pairing
results remain separately under `target/pr-followup-20261020/`. Reconciliation
preserved the incoming lock byte-for-byte, the empty index and unrelated work.
No full local CI, pending-source native macOS, production deployment, release or
publication is claimed. Historical handoffs remain in Git; issues own follow-up.
