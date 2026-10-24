# Current handoff — 2026-10-24

## Delivered source and local work

The checkout is based on release **0.15.3**, commit
`0ee62d9f030050c1d28c29840025bc5d66a9f0dd`; package versions remain 0.15.3.
Pending **0.15.4** notes cover local implementation and fixture simplification.
No commit, push, package version change or release ran for this cleanup.

The incoming lockfile selects Platform/macros **0.6.14** and Memory **0.50.16**;
those dependency edits were preserved, not made by the cleanup. Wire/derive
remain **0.6.12** and Host FS/Process, Build and Testkit **0.67.0**. The independent
file CLI remains **0.31.1**. Shared Tooling is the 121-file snapshot at
`4c0f198e56d4c6d27b12a5c2d515e0e01070151e`: its committed VERSION and canonical
snapshot say **0.26.10**, although the producer changelog names **0.26.11**.

Session transaction staging now permits one of unchanged, admission or removal.
The reference receiver reads generation from its store; checkpoint field order,
values and bounds are unchanged. Fixture capacities have one local definition.
Browser signing mechanics are shared while active-login and cleanup-lease policy
remain separate. Admission and logout share the network-call ceiling; storage
transaction timeouts remain independent. HTTP options and cleanup completion
predicates no longer have competing definitions.

The Testkit caller fixture exports maintained working sources using the existing
source exporter, then installs explicit substitutes. Generated client filenames
have one shell list and the tooling command remains typed through dispatch.
Public APIs, method identities, signed bytes and stored formats are unchanged.

## Implementation and consumer boundary

Auth delivers both portable proof verifiers, one session engine, host-selected
storage/reset interfaces, bounded Wire contracts and the private browser
admission/logout coordinator. Canic owns topology, authority approval/renewal,
endpoint guards, production enrollment, deployment configuration and its durable
host adapter. The future standalone wallet service is not implemented.

The [Canic adoption guide](../canic-adoption.md) describes config, Wire endpoint
wiring, capability profiles, synchronous application authorization and the
separation of `(issuer, subject)` from browser ingress caller. Private fixtures
are software evidence, not production Canic adoption or TEE custody evidence.

[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382815988)
retains runtime/store/reset/load acceptance. [Canic #507](https://github.com/dragginzgame/canic/issues/507#issuecomment-6382816186)
retains issuer policy, issuance abuse, exact credential reconciliation and browser
integration. Auth #58/#59 are closed as superseded coordination, not production
acceptance. Auth [#3](https://github.com/dragginzgame/ic-auth/issues/3) owns remaining
real-browser client/journal qualification.

Auth 0.15 admission uses method ID `0x415554480012`; receiver and client must
adopt it together. Do not reset portable journals or replay unknown issuance.
Retired private fixture checkpoint formats remain retired; production storage is
independent. The browser package stays private.

## Retained interfaces and follow-up

`AuthRequestMetadata`, Rust's empty `LogoutRequest` and the older session error
conversions remain supported public surfaces during Canic integration. See the
[consumer review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382404160).
They require an intentional API cut, not incidental cleanup.

[Platform #51](https://github.com/dragginzgame/ic-platform/issues/51) tracks the
authenticated-time verifier result. This cleanup does not migrate Auth's existing
certificate decoding/freshness checks to a different upstream API.
[Auth #85](https://github.com/dragginzgame/ic-auth/issues/85) retains the
crypto-bigint advisory correction/exemption and consumer qualification through
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).
Production skew, optional TEE evidence and consumer deployment remain separate.

## Validation and evidence

Cleanup evidence is retained under `target/cleanup-0154/`. The initial focused
pass covers sessions, host tooling, all 54 browser tests and Rust/TypeScript
vectors, Testkit caller admission/drift checks and jobserver propagation.
The second session pass adds all four double-write staging combinations,
checking refusal without invoking host commit or publishing records.
Native/reference-Wasm Clippy, isolated native/Wasm MSRV, six fixture unit tests
and all 12 qualification runner cases pass. The initial sandboxed run was unable
to bind localhost; the rerun with local networking passed. Formatting and local
documentation links pass. Manifest/lock input hashes remain unchanged.
The prepared Testkit/Build 0.67 tools are reused without installation or release.

Earlier upstream qualification and original working-source evidence remain in
`target/upstream-02610-067/`; the [0.15.3 handoff](https://github.com/dragginzgame/ic-auth/blob/0ee62d9/docs/status/current.md)
records that pass and its release-fixture lock distinction. No full CI/release
gate, native macOS, installed-IC or production-browser result is claimed here.
