# Current handoff — 2026-10-22

## Source and pending adoption

Delivered baseline is Auth **0.14.3** at
`835ccb3fd9124a6b9b5249314da1c31b593ba9db`; its
[three-host CI matrix](https://github.com/dragginzgame/ic-auth/actions/runs/39746230974)
passes. Package versions remain 0.14.3. Compatible pending **0.14.4** notes cover
published IC Build/Host/Testkit **0.64.0** and Wire/derive **0.5.20** adoption.
Auth public APIs, signed material, Wire frames and stored formats are unchanged.

The incoming root catalog and lock already selected these releases and remain
byte-for-byte unchanged. Registry searches confirmed published Host/Build/Testkit
0.64.0. Platform/macros remain **0.6.7**, Memory **0.50.10** and Tokio **1.54.1**.
No sibling/path overrides or storage migrations were added. Shared Tooling remains
the 118-file **0.25.9** snapshot at
`85aded191d85ac4a8c0e5deab207f6be18895862`. Development Rust remains 1.99.1;
the supported package minimum is 1.91.

## Ownership migration

Testkit 0.64 removes its upstream PocketIC re-exports. The native runner now
imports `PocketIc`, `PocketIcBuilder` and `Time` directly from Platform and
explicitly selects its `pocket-ic` capability in the runner's dev dependency.
Testkit retains bounded managed-server startup and typed Wire query/update calls.
No facade or new public capability was introduced. Public Auth and deployed
reference-Wasm graphs remain independent of the native simulator feature.

Published Host FS, Process, Artifacts and Build source trees are unchanged from
0.63.3. Other 0.64 removals are unused by Auth. Testkit's changed precedence for
invalid response schemas does not change Auth's valid finite method contracts.
The guides and agent overlay now describe the direct Platform import boundary.

Both lock-selected **0.64.0** executables were explicitly approved, prepared and
admitted: Testkit CLI with PocketIC **16.1.0**, and the IC Build release editor.
The separately pinned file CLI remains 0.31.1. Ordinary checks remain offline;
no release, publication, deployment, own version change or Git delivery ran.

## Current qualification

Evidence: `target/host-064-0144/`, including incoming bytes, published source
comparisons, preparation logs and selected-input seals. Passing checks:

- Strict native tooling/runner Clippy and reference-canister Wasm Clippy.
- Protocol/Wire tests, all **54** browser tests and all **11** host-tooling tests.
  Explicit regeneration reviewed all five browser files; they remain unchanged,
  and exact comparisons plus three bidirectional Wire vectors pass.
- Shared dependency ownership and Auth native/Wasm normal/build capability
  checks; public libraries and reference Wasm retain their Candid exclusion.
- `make check-msrv`: all 18 isolated public native/Wasm feature selections and
  internal applications, with unchanged package/archive/input seals.
- All five native fixture tests and **11 live** Testkit cases with the prepared
  0.64.0/16.1.0 pairing, including certification, upgrades, signed ingress and
  the two-device/two-receiver Ed25519 SDK journey on distinct application subnets.
- Testkit caller setup/check failure and input-seal fixtures, plus release
  metadata, failure admission, exact delivery, resume and mocked publication
  reconciliation using the actual selected editor and disposable repositories.
- Formatting, local documentation links, snapshot integrity and whitespace.

This is Linux reference-host evidence. Full CI, native macOS, real installed IC
and TEE qualification were not run for the pending batch. Issuer operations are
injected/read-only probes, clocks are controlled and IndexedDB is faked in Node;
these tests do not establish production issuer, browser persistence, throughput
or a deployment clock-skew bound. Earlier evidence remains at
`target/upstream-0143/` and `target/platform-066-0142/` with its original selections.

## Consumer acceptance

Auth #83/#84 are closed for delivered 0.14.2 signer-lifetime and immutable-target
fixes; native-client dependency adoption does not install the private browser
client into Canic. The browser package remains unpublished.

Production Root enrollment, retained receiver runtime/budgets and successful
scoped exports remain [Canic #491](https://github.com/dragginzgame/canic/issues/491).
Actual issuer issuance, lookup-only reconciliation and browser-key logout remain
[Canic #507](https://github.com/dragginzgame/canic/issues/507); future-start endpoint
acceptance remains [Canic #418](https://github.com/dragginzgame/canic/issues/418).
No missing producer API was found in this adoption; siblings were not edited.

[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58) and
[#59](https://github.com/dragginzgame/ic-auth/issues/59) retain production issuer,
persistence/reset and abuse/workload acceptance. Production skew activation
remains closed and TEE evidence is separate; no test value grants approval.
