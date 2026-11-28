# Current status

Auth 0.54.0 and Auth Wire 0.52.0 require the IC Wire 0.33 sparse format. Upgrade the browser and host together. Previously issued transport frames, stored browser credential journals, and earlier issuer and receiver checkpoints are refused. A populated 0.53 installation cannot upgrade in place. Provision fresh Auth state and have users authenticate again. The sparse renewal request also produces a new operation ID.

Auth **0.54.0** supports 100,000 sessions, 400,000 replays and 125,000 device slots, with 64 MiB of logical row capacity. Synchronous authorization borrows the signed extension.

Auth **0.52.0** reclaims expired device slots and permits a host-configured increase in the receiver slot ceiling during upgrade.
Host integration is tracked in [Canic #89](https://github.com/dragginzgame/canic/issues/89) for receiver row budgets and [Toko #23](https://github.com/dragginzgame/toko-new/issues/23) for browser epoch discovery.
Auth Wire **0.52.0** includes the extension in session admission and read replies. Protocol-types is **0.44.2**
and the private browser client is **0.2.0**. The checkout uses authenticated
Toko Tooling 0.46.9, IC Platform 0.35.1, IC Platform Types 0.21.6 and IC Wire 0.33.0. Production consumer acceptance remains open. The stable-only API uses
`receiver-stable` and `issuer-stable` with published `ic-storage` 0.80.6. Stable installations require fresh Storage 0.80 backing. Both owners use v1 checkpoints and require adjacent canister versions. Pre-cut images are refused. Historical cost records
identify the exact cohorts and artifacts measured.

| Capability | Current evidence | Remaining acceptance boundary |
| --- | --- | --- |
| Canonical claims, both proof mechanisms and four Wire contracts | Independent signed vectors, 37 independently assembled frames, 296 resource thresholds, generated browser interoperability; 27 focused crypto tests pass with Platform 0.34.11 | Complete current consumer delivery |
| Bounded memory receiver and issuer | Lifecycle/model/fault checks; browser-bound issuer revocation; private policy and operation owners | Production host integration and agreed resource budgets |
| Stable upgrade checkpoints | Native receiver and issuer reopen, consumed/corrupt image refusal, installed session continuity, logout and failed-upgrade rollback. [Canic 0.238.0 implements the host hooks](https://github.com/dragginzgame/canic/issues/50) | Upgrade budgets at selected capacity |
| Installed transactions, volatile loss and enrollment | Five lifecycle cases qualify rollback, real certificates, closed upgrade loss, bound authority replies and terminal key revocation | Production enrollment/refresh policy; pinned Canic witness recorded below |
| Browser lifecycle | Readiness/guarded calls, seven scheduler cases, twelve renderer-loss cases, seven disposal cases, ten external-signer boundary cases and three mixed multi-receiver cleanup cases | Real wallet/device recovery, production key custody and deployed multi-canister coordination |
| HTTP and installed browser journey | Certified response checks and issuance recovery → admission → protected write → logout → replay refusal | Production cutover, external signers and certified consumer integration |
| Installed costs | Fourteen measurement scenarios covering sixty-three observations covering independent capacities, 1/8/32 populations, mixed expiry and maximum scope dimensions | Consumer-selected workloads, production host overhead and accepted budgets |

Browser recovery uses native IndexedDB, real SDK signing and disposable Chromium
renderers. External signers and multi-receiver transports use host test doubles;
those tests do not establish real wallet or remote receiver behavior. Reopening
preserves issuance sequence, cleanup inventory and uncertainty. Explicit retry
reuses evidence; application calls never replay automatically. Terminal disposal
uses the public login binding and delegates conditional deletion to host custody.

Canic 0.219.0 now has generated issuer codecs, its browser adapter and an installed
Ed25519 profile witness with two independently enrolled receivers. The
[pinned consumer handoff](../design/compatibility.md#pinned-canic-consumer-evidence)
records source inspection and Canic-reported qualification separately from checks
run here. Production custody/cutover, external signers, certified consumer
integration and operating budgets remain open. Pursue concrete consumer gaps;
do not duplicate Canic's adapters or expand synthetic fixtures by default.

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
