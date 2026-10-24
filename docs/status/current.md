# Current handoff — 2026-10-24

## Delivered source and local work

The checkout is based on release **0.15.2**, commit
`140e6285cb4df64635b42af1dabb104c5a64fde2`; package versions remain 0.15.2.
Pending **0.15.3** notes cover the preserved documentation/issue cleanup and
published upstream adoption. No commit, push, own version change or release ran.

The lock selects Platform/macros **0.6.13**, Wire/derive **0.6.12**, Host FS/Process,
Build and Testkit **0.67.0**, Memory **0.50.15** and smallvec 1.16.4. Incoming
Platform/Memory selections were preserved. The independently pinned file CLI
remains **0.31.1**. Public Auth APIs, signed bytes and storage formats are unchanged.
Shared Tooling is the 121-file snapshot at
`4c0f198e56d4c6d27b12a5c2d515e0e01070151e`. Its committed VERSION and canonical
snapshot label are **0.26.10**, while that commit's changelog names the new setup
diagnostics **0.26.11**. The dirty producer VERSION was excluded; no sibling
working files or unpublished package were adopted.

The shared external IC inventory is selected explicitly by `check-boundaries`;
Auth retains only its additional restrictions/Platform ownership exceptions in
local policy. Validation seals both policy files. The updated fixture checks that
a changed common inventory stops subsequent validation before Cargo dispatch.
The [released cleanup handoff](https://github.com/dragginzgame/ic-auth/blob/140e628/docs/status/current.md)
retains the prior focused checks and evidence under `target/cleanup-0151/`.

## Implementation and consumer boundary

Auth delivers both portable proof verifiers, one session engine, host-selected
storage/reset interfaces, bounded Wire contracts and the private browser
admission/logout coordinator. Canic owns topology, authority approval/renewal,
endpoint guards, production enrollment, deployment configuration and its durable
host adapter. The future standalone wallet service is not implemented.

The [Canic adoption guide](../canic-adoption.md) describes existing config and
Wire endpoint wiring, feature profiles, synchronous application authorization,
and the distinction between `(issuer, subject)` and browser ingress caller.
The private fixture demonstrates software behavior; it does not establish
production Canic adoption, real-browser storage qualification or TEE custody.

[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382815988)
retains runtime/store/reset/load acceptance, including atomic session/replay/login
writes, retained rejection charges, protected clock/authority, persistent original
expiry and volatile reset floors. [Canic #507](https://github.com/dragginzgame/canic/issues/507#issuecomment-6382816186)
retains issuer policy, bounded valid-issuance abuse, exact original-credential
reconciliation, browser integration/distribution and matched complete workloads.
Auth #58/#59 are closed as superseded coordination tickets; their historical
implementation and cost evidence remains available. No production acceptance was
closed with them. Auth [#3](https://github.com/dragginzgame/ic-auth/issues/3)
now owns the concrete remaining real-browser client/journal qualification.

Auth 0.15 admission uses method ID `0x415554480012`; receiver and client must
adopt it together. The four Auth schemas remain unchanged in 0.15.2; its generated
runtime matches locked Wire 0.6.12. Do not reset portable journals or replay
unknown issuance. The retired unpublished fixture checkpoint formats remain
retired; production storage is independent. The browser package stays private.

## Retained interfaces and blockers

`AuthRequestMetadata`, Rust's empty `LogoutRequest` and the older session error
conversions remain supported public surfaces during Canic integration. See the
[consumer review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382404160).
They need an intentional API cut, not an uncoordinated cleanup.

[Platform #51](https://github.com/dragginzgame/ic-platform/issues/51) requests
an authenticated-time verifier result; Auth retains certificate decoding and
freshness checks until a compatible published API exists.
[Auth #85](https://github.com/dragginzgame/ic-auth/issues/85) retains the selected
crypto-bigint advisory correction/exemption and consumer qualification, supplied
through [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).
Production skew qualification, optional TEE evidence and real consumer deployment
remain separate gates; no clock allowance is established by software fixtures.

## Validation and evidence

Prior hosted source evidence is in [0.15.2 CI](https://github.com/dragginzgame/ic-auth/actions/runs/40082453794)
and the complete [0.15.1 matrix](https://github.com/dragginzgame/ic-auth/actions/runs/39985894914).
Those runs do not qualify this pending dependency adoption.
Adoption evidence is retained in `target/upstream-02610-067/`, including original
working-tree bytes, registry versions/checksums, snapshot impact/adoption and
focused validation logs. The approved Testkit/Build 0.67 executables are prepared;
offline admission selects PocketIC 16.1.0 and the exact Build editor. Platform
0.6.13's published `src/` tree is identical to 0.6.12. The 0.67 archive API and
error-struct changes affect no Auth call site; the selected native/Wasm APIs
compile without an Auth Rust-source or public contract migration.

Passing local checks include type/protocol/session and both crypto-profile tests,
public/native/Wasm dependency boundaries, the common inventory's refusal cases,
isolated package MSRV, strict native/reference-Wasm Clippy, all 54 browser tests
and three Rust/TypeScript vectors. All five generated files were refreshed and
are byte-identical to the incoming files. Tool-evidence, formatting/hooks, release
runner, jobserver, package dry-run and client/Testkit caller fixtures pass. An
initial Testkit fixture read overlapped its harness edit; its frozen-source rerun
passes, including the new common-inventory drift case. No input drift was waived.
The initial 0.50.14 pass has 12 live/six native cases. Cargo.lock then changed to
Memory 0.50.15; its published `src/` tree and original manifest are identical.
The preserved new selection passes fresh MSRV, strict Clippy, all 12 live/six
native cases and boundary/contract checks under its own unchanged input seal.
Release/recovery fixtures pass with their exported 0.50.14 lock and the unchanged
0.67 tools; they use local remotes and mocked registry effects. Earlier evidence
is retained separately. No full CI/release gate, native macOS or installed-IC
result is claimed by local Linux validation.
