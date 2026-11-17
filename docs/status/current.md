# Current status

Auth **0.47.1** is the current release (`v0.47.1`), with protocol-types/Wire
**0.43.0** and private browser client **0.1.0**. Volatile Auth is implemented in the
library; production consumer acceptance remains open. Stable storage is deferred
to `ic-storage-new` orthogonal persistence, with no interim adapter.

The 0.47.1 dependency cohort is Platform 0.34.11, Types 0.21.2, Runtime 0.23.15,
Wire 0.24.3, Testkit 0.107.4 and qualification-only Host Process 0.80.3. Both Cargo
graphs have one Platform/Types cohort and no legacy Platform or Agent. Toko Tooling
0.44.11 remains authenticated; Rust is 1.100.0. Product features and Wire layouts
are unchanged by the latest supplier adoption.

The maintainer authorized crates.io publication of all three product crates.
Qualification packages and the browser client remain private. The release includes
six module layout corrections. Historical installed cost observations retain the
exact dependency cohorts and artifact identities they measured.

| Capability | Current evidence | Remaining acceptance boundary |
| --- | --- | --- |
| Canonical claims, both proof mechanisms and four Wire contracts | Independent signed vectors, 37 independently assembled frames, 296 resource thresholds, generated browser interoperability; 27 focused crypto tests pass with Platform 0.34.11 | Complete current consumer delivery |
| Bounded memory receiver and issuer | Lifecycle/model/fault checks; browser-bound issuer revocation; private policy and operation owners | Production host integration and agreed resource budgets |
| Installed transactions, volatile loss and enrollment | Five lifecycle cases qualify rollback, real certificates, closed upgrade loss, bound authority replies and terminal key revocation | Canic enrollment integration and production challenge/refresh handling |
| Browser lifecycle | Readiness/guarded calls, seven scheduler cases, twelve renderer-loss cases, seven disposal cases, ten external-signer boundary cases and three mixed multi-receiver cleanup cases | Real wallet/device recovery, production key custody and deployed multi-canister coordination |
| HTTP and installed browser journey | Certified response checks and issuance recovery → admission → protected write → logout → replay refusal | Production Canic wiring and external signer integration |
| Installed costs | Five measurement scenarios covering sixteen operation/refusal paths; full installed suite passes ten tests | Issuer renewal/witness delivery, certified receiver admission, independent session/byte/per-account limits, scaling and accepted budgets |

Browser recovery uses native IndexedDB, real SDK signing and disposable Chromium
renderers. External signers and multi-receiver transports use host test doubles;
those tests do not establish real wallet or remote receiver behavior. Reopening
preserves issuance sequence, cleanup inventory and uncertainty. Explicit retry
reuses evidence; application calls never replay automatically. Terminal disposal
uses the public login binding and delegates conditional deletion to host custody.

[Installed costs](../development/installed-costs.md) records measurement intervals,
artifact identities and supplier cohorts. The release check reruns all installed
cases with the selected graph; observations are fixture baselines, not production
budgets. Refusal preserves work credit/state where appropriate, login/replay fences
remain charged after logout, and exact retry preserves its retained session lease.

[Canic #16](https://github.com/dragginzgame/canic/issues/16) owns browser-bound issuer
logout wiring and [Canic #17](https://github.com/dragginzgame/canic/issues/17) owns
generated issuer codecs and the browser adapter. Broader integration remains in
[Canic #10](https://github.com/dragginzgame/canic/issues/10) and
[Canic #11](https://github.com/dragginzgame/canic/issues/11). No Canic source changed.

[Compatibility and acceptance](../design/compatibility.md),
[browser integration](../../packages/client/README.md),
[installed qualification](../../qualification/ic-auth-host-conformance/README.md)
and [developer tooling](../development/tooling.md) own detailed evidence and commands.
The [architecture review](../reports/audits/2026/11/16/auth-architecture/01/report.md)
remains immutable pre-change evidence; it established ownership improvements, not
an equivalent-feature LOC saving. Hosted CI and local installed checks are separate
claims. Historical sources retain provenance; no legacy executable oracle is used.

All three product crates are included in the authorized crates.io release under
[publication scope](../development/publication.md). Qualification crates remain
nonpublishable and the browser package remains private. Production ownership,
consumer migration and live-storage changes remain outside this release.
