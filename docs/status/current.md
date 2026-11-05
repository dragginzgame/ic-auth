# Current handoff — 2026-11-05

## Published state

Protocol 0.34.0 (`ic-auth-protocol-types`, `ic-auth-wire`) and Engine 0.34.0
(`ic-auth`) are published on crates.io. Both annotated tags and release commits
are pushed to `origin/main` from the normal Auth checkout. The publication
adapter verified each registry archive checksum against its uploaded package.

- Protocol: commit `106fda655ed55a3542e9383331132f9491904768`, tag
  `protocol-v0.34.0` (`08f41509062f490d4181b3c45aec2893f44b3745`).
- Engine: commit `5a19c0ca75966612e8d476d1d1ba4b10490aec99`, tag
  `engine-v0.34.0` (`cdba7001b7d0d3f23dddb4d2e18ecaa03db7f148`).
- Exact archive identities and publication custody:
  `.git/publication-state/{protocol,engine}-0.34.0.json`.
- Delivery and upload logs: `target/auth034-delivery/`.

The published cohort selects Platform 0.22.1, Wire/derive 0.17.1, Testkit 0.86.0,
Storage 0.67.0, Metrics 0.29.2, Types 0.18.1, Runtime/guard 0.19.1 and
Host/Build 0.74.3. Rust 1.99.1 remains the compiler and minimum.
Protocol and Engine retain independent release groups; use matching 0.34 public
Protocol identities when adopting Engine 0.34. Signed material, session records
and Auth verification policy are unchanged.

## Qualification and accepted size tradeoff

Both standard release gates passed all 30 stages, including native/Wasm lint,
package/MSRV consumers, browser interoperability, live canister qualification
and release/recovery/publication caller fixtures. Receipts and logs:

- Protocol: `target/release-validation/protocol-gate.bB4F0B/logs/validation.eKJOuo`.
- Engine: `target/release-validation/engine-gate.nJVopu/logs/validation.B6CvxW`.

All 11 ordinary live cases pass. The separate opt-in Chromium case was not run.
Wire regeneration changed only the generated browser runtime's shared error
translation; four application contracts stayed byte-identical. All 70 browser
tests and Rust/TypeScript interoperability passed. These are Auth qualification
results, not production Canic adoption or a new cryptographic security audit.

Platform 0.22.1 restores packed-map/sequence envelope admission and removes
redundant BLS key serialization/parsing without dropping verification checks.
The maintainer explicitly accepted the residual Wasm increase on 2026-11-05 and
lifted the publication hold. Platform's matched Canic report found approximately
29 KB residual overhead versus 0.21 after its correction; this does not establish
size parity or erase Auth's earlier reference-fixture growth. No Auth optimizer
profile change or shared decoder API was shipped. Earlier matched comparisons
remain under `target/testkit085-wasm/apples/`, `target/decoder-boundary-probe/`
and `target/canic-decoder-probe/`; their scope is recorded in the historical
[pre-release handoff](https://github.com/dragginzgame/ic-auth/blob/ed785d7/docs/status/current.md).

## Adopted repository practices

Shared Tooling's committed 0.30.0 snapshot is pinned to
`b37a882314d22bf7ab393b3fe561809377ffd22d`, with all 156 payloads verified.
This is committed-snapshot adoption; upstream's ledger still called it unreleased.
The LOC reporter includes the duplicate-TOTAL and exported-link corrections.
Private qualification packages declare their test role; allocation observations
now use harness-free `benches/` targets. Existing script entrypoints are preserved.
There are no Rust `allow` suppressions to convert; two narrow `expect` attributes
remain. The corrected LOC classification is accounting, not code removal.

Engine release plans now require `test-protocol`, which runs Engine default tests.
Failure propagation is covered by the release-selector fixture. Final-candidate
stage reuse remains unqualified; use the standard gate without bypasses.
Detailed audit evidence remains under `target/shared-tooling030-audit/`.

## Remaining ownership and recovery

This Auth delivery is complete; no release blocker remains.
[Canic adoption](../canic-adoption.md) and
[signature preparation](../signature-preparation.md) describe the consumer
contracts, including certified issuance's durable journal and update/query split.
Canic [#491](https://github.com/dragginzgame/canic/issues/491) owns production
clock/restore acceptance, certified issuance, key custody and installed integration.
Auth [#125](https://github.com/dragginzgame/ic-auth/issues/125) is closed after
verified historical Linux/Intel macOS/ARM macOS group-comparison evidence; that
hosted run is not a hosted qualification of this new release.

Completed release plans and publication custody remain under `.git/release-state/`
and `.git/publication-state/`. Earlier interrupted 0.33 attempts were superseded
by successful releases, without unresolved uploads. Their evidence remains intact.
Retired sibling checkout refs are `archive/ic-auth-capacity` and
`archive/ic-auth-release-033`; retirement evidence remains under
`target/checkout-retirement/20261104-siblings/`. No artifacts were deleted in this
release. Target-owned evidence can be lost to an explicit clean. Historical detail
is preserved in Git and [earlier delivery history](history-0.33-delivery-and-qualification.md).
Continue in the normal checkout; do not create new sibling directories.
