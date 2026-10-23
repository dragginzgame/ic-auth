# Current handoff — 2026-10-23

## Delivered source and pending adoption

Auth **0.15.0** is delivered on remote main at
`089d49d5fb92e46450638ce7fe6cfcfdafc53707` and available in the registry.
Package versions remain 0.15.0. Pending **0.15.1** notes cover published Host
FS/Process, Build and Testkit **0.66.0**, plus Shared Tooling **0.26.8**.
This is compatible at Auth's public boundary: APIs, signed material, method IDs,
transport bytes, storage and browser journal formats are unchanged from 0.15.0.

The current lock selects Wire/derive **0.6.10**; the initial adoption used 0.6.7.
The reference batch preserves incoming Memory **0.50.13** and file CLI **0.31.1**. The
bounded update changed only the five requested Host/Build/Testkit selections.
A separate late lock update selected Platform/macros **0.6.10** after initial
qualification with 0.6.9; it is preserved and affected checks now pass.
Both published source trees are identical to 0.6.9; the only dependency requirement
change raises the Wire metadata minimum to 0.6.7, satisfied by this lock. No sibling/path override
or storage migration was added. Commit `4d28772071cb5bc1b40b54938fec1c3c459399ef`
recorded the adoption externally during qualification. This agent performed no
commit, push, own version change, release or publication.

Shared Tooling is the **119-file 0.26.8** snapshot at
`0b07c2a3163a5a66328e100b553baf93afd8ed07`. This is reviewed committed main source;
its changelog still marks the version unreleased. The canonical exporter retained
the selected file closure and changed only the setup guide and two build helpers.
Its Cargo setup fix tolerates an admitted owner's lock withdrawal, and source
cache initialization no longer uses a global installation lock.

## Ownership and setup

Published Host FS/Process/Artifacts and Build source trees are unchanged from
0.65.0. Testkit 0.66 makes its PocketIC owner fields private. The reference runner
borrows the client during setup with `instance()`, then deliberately takes
`into_parts()` and retains client-before-server teardown in its existing fixture.
Setup panic paths retain the original owner. Auth adds no wrapper, runtime policy
or public capability. Native simulator clients and DTOs still come from Platform;
Testkit owns startup, lifecycle and typed Wire dispatch.

Wire 0.6.10 changes generated helper names, built-in TypeScript qualifications
and the shared runtime. Explicit regeneration updates all five browser files;
Auth's method identities, budgets and payload schemas remain unchanged. Auth
does not select the newer optional packed-field schema. See the repair below.

The user approved executable preparation. `make install-tools` and
`make install-release-tools` completed: existing common Rust/File CLI tools were re-prepared
under the new build policy, alongside Testkit and IC Build **0.66.0**. Offline
admission passes, including PocketIC **16.1.0**. Initial old-policy refusals and
an early host-tooling run before file-CLI preparation remain retained; their
focused reruns pass after setup, with no product-code workaround. Checks never
repair or fetch tools implicitly. Development Rust remains 1.99.1; MSRV is 1.91.

## Upstream-adoption qualification

These adoption results used Wire 0.6.7. The later 0.6.10 client checks are recorded
separately below; no earlier full-graph qualification is relabelled as 0.6.10.

Evidence: `target/shared-0268-host-066/`, including incoming bytes, source
comparisons, canonical snapshot preview, setup logs and selected-input seals.
Passing focused checks:

- Strict native tooling/runner and reference-Wasm Clippy.
- Protocol/type tests, all eight public Wire and two private contract cases,
  all **54** browser tests and all **11** host-tooling cases. Generated comparisons
  and three bidirectional Rust/TypeScript vectors pass.
- Isolated Rust 1.91 public-package native/Wasm feature selections and internal
  applications, with unchanged package archives, source and manifest/lock seals.
- All six native fixture tests and **11 live** Testkit cases: certification,
  composed roots, protected authority/upgrades/checkpoints, signed ingress and
  the SDK two-device/two-receiver journey on distinct application subnets.
- Direct dependency ownership and native/Wasm normal/build exclusions. Public
  libraries and reference Wasm remain Candid-free and exclude native host tools.
- Actual setup/check refusal, validation-seal and workflow-evidence callers;
  formatting, dependency sorting and staged preservation. Release metadata,
  failure admission, exact delivery, resume and mocked publication reconciliation
  pass with the selected editor in disposable repositories.
- Snapshot integrity, local documentation links and whitespace checks.

The late Platform selection passes refreshed Clippy, isolated MSRV, generated
contracts, protocol/Wire/browser tests, boundaries and all six native/11 live
fixture cases (`platform-final.log` and `platform-live.log`). Host-tooling,
setup/evidence, formatting and release/recovery results precede that late patch;
their Host/Build implementations, selected executables and build helpers are
unchanged, so that evidence is retained rather than replaying unaffected suites.

Earlier evidence under `target/shared-0267-host-065/` retains its original
selections. This batch establishes Linux reference-host evidence, not full CI,
native macOS, installed IC or TEE qualification. Clocks are controlled, issuer
operations are injected probes and IndexedDB is faked in Node. No production skew
bound, throughput or browser persistence guarantee follows.

## Scoped application-write reference

Pending 0.15.1 adds the private `session_set_asset` update and one installed
reference case, using the existing Auth 0.15 API. The host fixes the required
`write` scope and acquires runtime caller/time and protected authority; admission
proofs and claimed callers are absent from ordinary requests. Authorization and
the existing fixture asset mutation share one synchronous IC message. The read
endpoint uses the same authorization helper. No public API, store field or format
changes; Canic retains production guards, account/resource policy and enrollment.

Evidence: `target/canic-scoped-write/`. Strict native/Wasm Clippy, Rust 1.91
internal-package native/Wasm checks, both private contract cases, all six native
fixture tests and all **12 live** cases pass on the preserved selection. The new
case sends real signed Agent updates after admission-proof expiry, checks narrowed
read-only and unknown callers, rejects retained keys after logout, preserves the
peer subnet's independent session and refuses writes after authority invalidation.
Every refused write preserves the observable application value. The fixture's
single-asset scope policy is not a general account/resource ownership grant.
Memory 0.50.13's runtime source is unchanged from 0.50.12; its pairing is covered
here, without a new storage migration or full Memory-library qualification claim.
The [Canic adoption tracker](https://github.com/dragginzgame/canic/issues/491#issuecomment-6368395382)
records the reference and the remaining production boundary. No new Auth release
or API is required for Canic to use the equivalent shared-engine call.

## Client regeneration repair — Wire 0.6.10

The release gate at source `8cbd43c486defe8872c2593a860a54c16bcf507d` selected
Wire 0.6.10 while the committed browser files still matched 0.6.7. The retained
`contracts.zJLs5K` candidate correctly refused that stale output at `test-client`.
The original failure logs and release-validation state remain untouched.

`make generate-client-contracts` now refreshes all five files from the actual
locked owner, with no downstream runtime patch. The four generated registries
have identical method identity, budgets and payload schema before/after; the
runtime is byte-identical to published Wire 0.6.10. Frozen Rust frames and all
three Rust/TypeScript vectors pass. `make test-client test-types` passes all 54
client cases, eight public transport and two private contract cases. Evidence is
retained under `target/client-wire-0610/` with unchanged manifest/lock seals.
No DTO, signed-byte, browser journal or storage migration is needed. Full release
validation was not rerun and no release, recovery, commit or publication ran.

## Consumer acceptance

Auth 0.15's admission ID is `0x415554480012`; receivers and generated clients must
use that released contract together. Credential/read/logout identities and frozen
frames remain unchanged. Do not reset portable journals or replay unknown issuance.
The unpublished finite-Wire fixture checkpoint remains retired; production memory
and storage are independent. This patch adds no format cut or reset requirement.

Production enrollment, retained receiver runtime/budgets and scoped exports remain
[Canic #491](https://github.com/dragginzgame/canic/issues/491). Issuance, lookup-only
reconciliation and browser-key logout remain
[Canic #507](https://github.com/dragginzgame/canic/issues/507); future-start endpoint
acceptance remains [Canic #418](https://github.com/dragginzgame/canic/issues/418).
All three trackers remain open at inspection. These tooling updates require no
new Auth public API migration. Siblings were not edited; the private browser
package remains unpublished. The repository description still matches this scope.

[Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58) and
[#59](https://github.com/dragginzgame/ic-auth/issues/59) retain production issuer,
persistence/reset and abuse/workload acceptance. Skew activation remains closed;
TEE evidence and production acceptance are separate.
