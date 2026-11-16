# Current status

Volatile Auth is implemented in the library; complete consumer integration and
browser lifecycle acceptance remain open. Stable storage is deferred to
`ic-storage-new` orthogonal persistence; no interim adapter is planned.

The **0.47.0 unreleased** source checkpoint adds browser readiness and admission
coalescing with explicit clock mapping and a 64-operation ceiling. It also adopts
Platform 0.34.6, Types 0.21.2, Runtime 0.23.13, Wire 0.24.2 and Testkit 0.107.1.
The former Types/Wire generator mismatch is resolved. Both Cargo graphs contain
one Platform/Types cohort and no legacy Platform or Agent. Rust package versions remain Auth
**0.46.1** and protocol-types/Wire **0.42.0**. The preceding source checkpoint is
`4054a11`; guarded ordinary calls are included in this source delivery. 0.2.0 remains
the last tag. All packages remain unpublished. The repository is now `dragginzgame/ic-auth`;
historical provenance is retained for `dragginzgame/ic-auth-legacy`. Historical audit identities
retain their names and paths at the time of inspection.

| Capability | Current evidence | Remaining acceptance boundary |
| --- | --- | --- |
| Canonical claims, both proof mechanisms and four Wire contracts | Independent signed vectors, 37 independently assembled frames, 296 resource thresholds and generated browser interoperability | Complete current consumer delivery; no pre-recode compatibility requirement |
| Bounded memory receiver and issuer | Native lifecycle/model/fault checks; private policy and one operation owner each | Production host integration and accepted installed costs |
| Installed transactions, volatile loss and enrollment | Five single-subnet Wasm cases: rollback, real certificates, closed upgrade loss, bound authority replies and terminal key revocation | Canic registry integration, production challenge/refresh handling and late/overlapping replies |
| Browser journal, readiness, admission and scheduled cleanup | Strict IndexedDB, nineteen readiness/guarded-call cases, seven scheduler cases, two-tab/logout fencing, real SDK signing and independent Wire frames | Key disposal and renderer interruption |
| HTTP and installed browser journey | Five raw certificate cases plus four journey cases: lost issuance reply/reconciliation, real Wire admission, protected write, logout and credential replay refusal | Production Canic wiring, multi-receiver coordination and external signer recovery |

The browser package exposes lifecycle handles, a bounded session coordinator and
complete retrieval/admission/cleanup operations, including cancellable cleanup
scheduling through journal-owned leases and deadlines. Readiness receipts remain detached
scheduling hints. Guarded ordinary calls prepare readiness once and recheck the
deadline and logout generation around each signature without application replay.
Claim/guard/settlement stages and signing-attempt machinery
are internal. The [architecture review](../reports/audits/2026/11/16/auth-architecture/01/report.md)
found real ownership improvements but no demonstrated equivalent-feature LOC
saving. Its API and current-status findings are addressed in this working slice;
the report remains the immutable pre-change evidence.

[Compatibility and acceptance](../design/compatibility.md) owns the evidence
boundaries. [Browser integration](../../packages/client/README.md),
[installed qualification](../../qualification/ic-auth-host-conformance/README.md)
and [developer tooling](../development/tooling.md) describe usage and focused gates.
The authorized pre-recode hard cut removes the executable legacy oracle and
legacy-feature qualification lane; strict security checks and current browser
interoperability remain. No legacy checkout is required.
Platform owns crypto/SDK behavior, Canic owns application endpoint wiring, and
hosts own authenticated inputs, transactions and certification-root publication.

[Hosted native jobs for `3839e03`](https://github.com/dragginzgame/ic-auth/actions/runs/43858613569)
passed on Linux and both macOS architectures; that run's history scan failed on
two type examples in exception comments. The local correction and exact historical
exceptions pass the scan. Browser and installed gates remain explicit local checks,
not hosted CI claims. Rust remains 1.100.0; Toko Tooling 0.44.11 is now selected and authenticated.

The fixed combined issuer/receiver journey is now exercised on one subnet through
the supported browser API. Next integration work should use production Canic
boundaries and complete the remaining browser lifecycle with the same owners. The original
repository remains production owner. Publication, deployment and live-storage
migration are not authorized.
