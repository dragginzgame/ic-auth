# Current handoff — 2026-10-24

## Delivered source and pending cleanup

The local checkout is based on release **0.15.1**, commit
`9f42245`. Package versions remain 0.15.1; pending **0.15.2** notes describe
compatible cleanup. This pass performed no commit, push, release or publication.
Prior release qualification is retained in the
[0.15.1 handoff](https://github.com/dragginzgame/ic-auth/blob/9f42245/docs/status/current.md);
those results are not qualification of the incoming dependency selection.

Incoming work was preserved: the lock now selects Platform/macros and Wire/derive
**0.6.12**, Host FS/Process, Build and Testkit **0.66.3**, and smallvec 1.16.4.
Memory remains **0.50.13**, the independently prepared file CLI remains **0.31.1**,
and Shared Tooling remains the 119-file **0.26.8** snapshot
`0b07c2a3163a5a66328e100b553baf93afd8ed07`. No lock or manifest selection was
changed by this cleanup. Incoming Canic configuration/adaptor documentation was
preserved. Sibling checkouts remain read-only.

## Ownership and changes

Auth owns proof/credential and session machinery, bounded transport contracts,
and its private admission/logout client. Canic owns topology, authority approval,
renewal orchestration, endpoint guards and deployment configuration. The future
wallet service is not implemented here. Native tooling and the private fixture
are not public library capabilities.

- Protocol-types owns the shared structural grant/scope/label/claims ceilings.
  Validation and Wire projections consume those constants. Operational policy,
  transport allocation budgets and storage limits remain separate. No limit or
  wire/storage encoding changed; `portable::MAX_CLAIMS_BYTES` remains available.
- Contract generation infers request/reply types from the real codec functions.
  Generation/comparison needs Rust and the admitted file tool, not Node/npm.
  Actual browser tests and live qualification admit browser dependencies first.
- Auth's manifest supplies individual feature selections to boundary, packaged
  MSRV and Wasm checks, plus the explicit session/portable combination and all
  features. A synthetic new capability regression checks both targets.
- The jobserver fixture uses the maintained source exporter and derives recursive
  targets from the Makefile. Portable signatures are covered. Effects remain
  substituted; this does not execute releases or install tools.
- Host FS owns exhaustive file-CLI semantics. Auth retains one real shell-adapter
  test for working directory, argument forwarding, limits and refusal, plus its
  own CLI, external-lock and contract-vector cases.
- Completed Platform 0.4/0.5 migrations are in the existing
  [history](../canic-adoption-history.md#platform-04-migration). The current
  [Canic guide](../canic-adoption.md) keeps the 0.6 boundary and current wiring.

## Retained public contracts and upstream dependency

`AuthRequestMetadata` remains re-exported by Canic contracts. Rust's empty
`LogoutRequest` remains public even though current Wire logout uses unit; the
browser type aliases the generated schema. `SessionError::Signature` and
`Certified` remain supported conversions for verifier callbacks; the ordinary
shared verifier uses `Credential`. Deleting these surfaces needs an intentional
API cut, with little immediate benefit to current integration. The
[Canic tracker note](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382404160)
records the verified consumers and recommendation to retain them for now.

Platform 0.6.12 returns only verification success, so Auth still decodes the
verified signature envelope/certificate to enforce freshness. An additive
verified-time result is requested in
[Platform #51](https://github.com/dragginzgame/ic-platform/issues/51).
No timestamp check, bound or cryptographic verification was removed.

## Qualification of this cleanup

Evidence is retained in `target/cleanup-0151/`, with the original working-tree
bytes (including incoming work), focused logs and manifest/lock identities.
Passing focused checks on native Linux:

- `test-host-tooling` (four Auth/adapter cases), `test-types`, `test-protocol`
  and `test-sessions`, including retained replay, rejection and restore coverage.
- `test-client-tools`, `test-testkit-tools`, `test-make-jobserver` and
  `check-boundaries`. The first new boundary fixture attempted unsupported yq
  TOML output; the corrected fixture and complete focused rerun pass.
- `generate-client-contracts` and `test-client`: 54 client cases and three
  bidirectional Rust/TypeScript vectors.
- `check-wasm` and `check-msrv`: default, individual, session/portable combined
  and all-feature package selections on native/Wasm at Rust 1.91, plus the
  existing internal application MSRV checks.
- Strict all-target/all-feature Clippy for protocol-types, Wire, Auth and tooling;
  formatting, local documentation links and snapshot integrity.

Manifest/lock identities stayed unchanged. No complete CI/release gate,
installed-IC qualification or native macOS run was performed. The Testkit runner
admission ordering was exercised with substituted effects; this is not a live
canister result for the incoming 0.66.3 CLI. Earlier 0.66.0 setup/qualification
must not be relabelled as 0.66.3 evidence.

The locked Wire 0.6.12 generator refreshed all five outputs. Only the shared
runtime changed, to its canonical published bytes; all four Auth registries are
unchanged. Auth selects none of the new generic packed schemas. Public method
identities, bounds, signed material and persisted profiles remain unchanged.

## Consumer acceptance

Auth 0.15 admission uses method ID `0x415554480012`; receivers and clients must
use that contract together. Do not reset journals or replay unknown issuance.
The unpublished finite-Wire fixture checkpoint remains retired; production memory
and storage are independent. The private browser package remains unpublished.

Production enrollment, retained receiver runtime/budgets and scoped exports belong
to [Canic #491](https://github.com/dragginzgame/canic/issues/491). Issuance,
lookup-only reconciliation and browser-key logout belong to
[Canic #507](https://github.com/dragginzgame/canic/issues/507); future-start endpoint
acceptance remains [Canic #418](https://github.com/dragginzgame/canic/issues/418).
This cleanup does not demonstrate production Canic adoption. Auth
[#3](https://github.com/dragginzgame/ic-auth/issues/3),
[#58](https://github.com/dragginzgame/ic-auth/issues/58) and
[#59](https://github.com/dragginzgame/ic-auth/issues/59) retain production issuer,
persistence/reset and abuse/workload acceptance.
