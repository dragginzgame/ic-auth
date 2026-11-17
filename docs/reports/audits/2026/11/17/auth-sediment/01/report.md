# Auth architectural sediment audit

Verdict: **PASS WITH FINDINGS** for bounded ownership and flow inspection.
The concrete documentation and helper-ownership findings were repaired locally.
No core redesign is justified by this review. This is not a security audit,
whole-program correctness proof or production-readiness certification.

## Scope and identity

Requested by the maintainer: audit retained architectural sediment, clean up and
sanity-check direction. Method: [flow convergence and duplication](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/audits/flow-convergence-and-duplication.md),
with its common audit contract and authorized module-cleanup procedure, fixed at
Toko revision `626da7cd9626cc2e152594862a2d82ea82ef51da` (installed 0.44.11).
Provisional local scope uses AGENTS.md, README, DESIGN, authentication,
compatibility and current status; no recurring audit overlay is adopted.

Source: `a05b5ddd4099a3f64b95deb173a0c16746d98ac5`, plus pre-existing pending
supplier locks and installed qualification work: renewal/certified admission,
independent capacities, populations, mixed expiry and wide scopes, fixture/vector
support and corresponding documentation. Those changes were preserved. This audit
changes design/status/evidence guidance, the changelog and native harness helper
ownership; it changes no product source, fixture Wasm source, manifest or lock.
[Evidence identities](evidence.json) identify reviewed files and selected graphs.

The preceding [architecture review](../../../16/auth-architecture/01/report.md)
is historical evidence, not a fresh test pass or an equivalent measurement baseline.
No LOC/performance trend is inferred. Scope covers product package ownership,
receiver admission/read convergence, issuer phase ownership, browser admission
handoff and the new installed test scaffolding. Crypto internals, exhaustive
browser scheduling, production Canic implementation and future durable storage
are outside this structural review; their domain obligations remain required.

## Owners and traces inspected

| Behavior | Owner and input | Contract and consumer |
| --- | --- | --- |
| Credential admission | `MemoryReceiver::admit`; protected host inputs plus untrusted credential | Private retry/login/capacity rules and complete verification precede record publication; Ed25519/certified inputs share the receiver path |
| Protected application effect | `MemoryReceiver::authorize` calls `read` | Current enrollment, caller, time and scopes are rechecked before synchronous projection; copied receipts are not continuing authority |
| Issuance/recovery | `issuer_memory::Operation` and `Phase` | Reserved/challenged/approved/dispatched/completed state carries retained evidence; custody and certification publication remain host-owned |
| Browser admission | `establishSession` and coordinator converge on `admitCredential` | Journal inventory precedes transport; shared signing attempt guards asynchronous boundaries; generated codecs project bounded requests/replies |
| Installed measurement | Fixture control owns execution; native support owns setup/measurement transport | Scenarios retain independent input, capacity, lease, accounting and refusal assertions |

Standalone issuer and Ed25519 receiver normal-dependency graphs contain neither
Canic, Host, Build, Testkit nor Toko. Issuer state is feature-separated without a
fourth crate. There is no implemented generic storage backend protocol.

## Findings and repairs

1. **MEDIUM — stale architectural authority (Auth design owner).** DESIGN still
   prescribed an issuer package, generic `Receiver<B>`, universal fact/change
   values and an interface justified by two backends despite only one implemented
   backend. Its early-stage status and pending custom callback decision also
   contradicted current implementation and hard-cut scope. Consequence: future
   work could recreate abstractions deliberately avoided by the implementation.
   Reconciled package/API ownership, private facts and future storage prerequisites;
   removed the obsolete generic API sketch. No public API changed.
2. **LOW — scenario-owned shared infrastructure (host qualification owner).**
   Population/mixed scenarios imported setup from `independent_capacity`, and
   mixed scenarios imported measurement transport from `population_costs`.
   This coupled otherwise independent scenarios and mislabeled mixed/wide output
   as population measurements. Moved setup and transport into
   `installed/receiver_support.rs`, changed the common log prefix to
   `receiver-cost`, and retained scenario assertions and instruction intervals.
   No new product states or qualification modes were introduced.
3. **LOW — historical cost guidance presented as current work (documentation
   owner).** The design called for legacy differential measurements and carried
   a historical persistent-workload ceiling into current gates; status emphasized
   further synthetic dimensions without a consumer-selected decision. Clarified
   historical ceilings' workload scope, added an evidence-reading guide and
   made consumer integration/agreed budgets the next decision. Historical numbers,
   cohorts and immutable reports remain intact. This does not waive any accepted
   budget for a matching workload.

## Intentional retention and direction

- Keep three product crates, private policy owners and separate issuer/receiver
  features. Keep generated Wire projections and host-owned authority/effects.
- Keep repeated boundary validation where untrusted decoding, asynchronous use or
  changed authority requires it. A receipt or journal entry cannot replace that
  validation. No protective guards were deleted as superficial duplication.
- Keep independent capacity controls, mixed expiry and signed fixtures: they test
  different failure guarantees. The fixture's fixed profiles are qualification
  inputs, not an application service or proposed generic framework.
- Keep historical measurement cohorts with their original identities. They cannot
  be merged into one comparable cost table without matching artifact/workload
  evidence. The new reading guide distinguishes these from the latest run.
- Stop expanding synthetic measurements by default. Next work should resolve the
  existing Canic consumer contracts and operational workload/budget decisions.
  Existing ownership remains in Canic #10/#11/#16/#17 as linked by current status;
  this run did not query or change their remote status. Real signer custody and
  deployed coordination remain acceptance gaps, not proof of Auth defects.
- Durable storage remains deferred. A second backend must earn any shared storage
  interface through demonstrated operations, corruption admission and rollback.

## Verification and limits

Fresh checks: `make fmt lint-host` passed native harness and Wasm fixture strict
Clippy; `make fmt-check` and `git diff --check` passed. Standalone normal dependency
graphs were inspected with offline locked `cargo tree` for `issuer-memory` and
`receiver-memory,ed25519-verification`.

With a prepared local PocketIC 16.1.0 server:
`HOST_TEST_FILTER='independent_capacity population_costs mixed_costs --nocapture' make qualify-host`
passed all **seven affected installed tests**, with twelve unrelated cases filtered
out. Independent host vectors were regenerated by that target. Rust 1.100.0;
Platform 0.34.13, Types 0.21.2, Runtime/macros/guard 0.23.16, Wire 0.24.3,
Testkit 0.108.0, Host Artifacts/FS 0.80.3 and Process 0.80.4. Release fixture Wasm
remained 1,529,882 bytes with SHA-256
`4cd55ec86d2c5326370ca3b0942288ee9bcd3e106963f048b34c5060877670c5`.
The server was stopped after verification.

No product, full browser or full release suite was rerun: product and browser
source were unchanged by these repairs. Existing nineteen-test/HTTP results in
installed-cost documentation are prior evidence, not new execution in this audit.
No commits, pushes, publication, deployments or sibling source edits were made.
