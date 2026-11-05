# Current handoff — 2026-11-05

## Pending Engine 0.34.1

Engine 0.34.1 is drafted but unreleased; Protocol remains 0.34.0. Standard release
gate and publication remain separate work. The working tree also adopts committed
Shared Tooling 0.30.4 and compacts superseded historical documentation.

Testkit 0.87.0 is selected for native qualification. Its async-only client uses
one two-worker runtime per test, awaited Wire/lifecycle calls and explicit
instance close before server shutdown. The lock now selects one Platform 0.23
for both Auth and Testkit. All 11 ordinary live cases pass, including certification,
upgrade, signed ingress and SDK transport; explicit close succeeds. The optional
Chromium case is skipped. Evidence: `target/evidence/testkit087/`. The standard
release gate remains pending. No public Auth API changes.

The first release attempt stopped on external lock drift to Metrics 0.29.3;
its source matches 0.29.2 and all five Auth metrics tests pass. Preserve that
incoming selection and the stopped attempt under
`target/release-validation/engine-gate.rl6XlY/`. Delivery requires a fresh gate.

Auth requires published Rust Platform 0.23.0 and the npm browser SDK
@tokostudios/ic-platform 0.2.0, with verified archive integrity and unchanged SDK
Core 6.1.1. No sibling path overrides are selected. Platform owns bounded
certificate and standard signature-envelope codecs; Auth retains host budgets,
witness construction, root binding and store policy. Auth's direct CBOR dependency
is removed; public Protocol identities remain 0.34.0.

Qualification passes: 55 affected integration tests, the signature-store unit
test, strict Auth/canister lint, standalone preparation Wasm and release canister
build. Preparation/session graph checks pass. That Platform-only qualification used
Testkit 0.86 with native Platform 0.22; the subsequent 0.87 adoption above removes
that older native selection. Wasm selects Platform 0.23. All 70 browser tests, packed-client
consumer and Rust/TypeScript interoperability pass. Four focused live cases pass:
certificate/store composition, Root/issuer profile composition, portable certified
proof/authority/upgrade, and SDK two-receiver/device admission/logout with lost
replies and durable reopen. Evidence: `target/platform023-adoption/`.
No new complete release gate is claimed. Older 0.22.3 qualification retains its
scope under `target/platform0223-qualification/`.

The independent Platform npm release is complete: source `d30e01c`, tag
`browser-v0.2.0`, registry integrity matched to the qualified archive. Its
21 tests, packed compiler/provider graphs and Chromium page/module worker passed;
supplier custody is at Platform `target/browser020-delivery/`. Auth consumer
qualification is separate. Canic remains read-only here.

## Published state and accepted size tradeoff

Protocol 0.34.0 (`ic-auth-protocol-types`, `ic-auth-wire`) and Engine 0.34.0
(`ic-auth`) are published on crates.io, with tags/commits pushed to `origin/main`.
The adapter verified registry archive checksums against uploaded packages.

- Protocol: commit `106fda655ed55a3542e9383331132f9491904768`, tag
  `protocol-v0.34.0` (`08f41509062f490d4181b3c45aec2893f44b3745`).
- Engine: commit `5a19c0ca75966612e8d476d1d1ba4b10490aec99`, tag
  `engine-v0.34.0` (`cdba7001b7d0d3f23dddb4d2e18ecaa03db7f148`).
- Exact identities/custody: `.git/publication-state/{protocol,engine}-0.34.0.json`;
  delivery/upload logs: `target/auth034-delivery/`.

The published lock selects Platform 0.22.1, Wire/derive 0.17.1, Testkit 0.86.0,
Storage 0.67.0, Metrics 0.29.2, Types 0.18.1, Runtime/guard 0.19.1 and Host/Build
0.74.3. Rust 1.99.1 remains the compiler and minimum. Match Engine's declared
Protocol identities; the groups release independently. Signed material, session
records and verification policy are unchanged.

Both standard release gates passed all 30 stages, including all 11 ordinary live
cases; the separate opt-in Chromium case was not run. Gate logs are under
`target/release-validation/protocol-gate.bB4F0B/logs/validation.eKJOuo` and
`target/release-validation/engine-gate.nJVopu/logs/validation.B6CvxW`.
These results do not establish production Canic adoption or a crypto security audit.

The maintainer accepted Platform's residual Wasm increase on 2026-11-05 and lifted
the publication hold. Platform's matched Canic report found approximately 29 KB
overhead versus 0.21 after its correction; this does not establish parity or erase
Auth's earlier fixture growth. No Auth optimizer profile change or shared decoder
API shipped. Earlier matched comparisons remain at `target/testkit085-wasm/apples/`,
`target/decoder-boundary-probe/` and `target/canic-decoder-probe/`; see their
[pre-release scope](https://github.com/dragginzgame/ic-auth/blob/ed785d7/docs/status/current.md).

## Adopted repository practices

Shared Tooling 0.30.4 is pinned to `2aa41c7394300302fe802b69b31b855bdc89b671`;
all 158 payloads match the canonical export. Upstream labels this series unreleased.
LOC/formatting scratch fixes, numbered-draft handling and compact evidence rules
are adopted. Historical handoff/adoption journals are removed from the working
tree; Git retains them. Compact allocation/Wasm conclusions remain in docs.
Release receipts, active recovery records, fixtures and local artifacts are untouched.

`check-candid-boundary` scans `crates/` and `apps/` with zero allowances through
`check-boundaries`. The actual Make caller rejects an owned Candid declaration
before dependency dispatch; its Wire-only fixture proceeds, and 102 real source
files pass. This is lexical enforcement, not macro/alias analysis. Formatting/hook
preservation and unsorted-manifest refusal pass with the repaired dependency pair;
LOC has zero unclassified source. Dated/undated notes agree; wrong candidates refuse.
Snapshot/link/diff checks pass. Caller evidence: `target/evidence/shared-tooling/`.
No product tests were repeated for the tooling and documentation cleanup.

## Ownership and recovery

[Canic adoption](../canic-adoption.md) and [signature preparation](../signature-preparation.md)
own consumer contracts. [Canic #491](https://github.com/dragginzgame/canic/issues/491)
owns production clock/restore acceptance, certified issuance, custody and installed
integration. Auth #125 is closed on historical hosted group-comparison evidence,
not a new hosted qualification of this release.

Preserve `.git/release-state/`, `.git/publication-state/` and target evidence.
Interrupted 0.33 attempts were superseded by successful releases with no unresolved
uploads. Retired checkout refs are `archive/ic-auth-capacity` and
`archive/ic-auth-release-033`; retirement evidence is under
`target/checkout-retirement/20261104-siblings/`. Ignored target artifacts are local
and removable by explicit clean. Historical release receipts bind their original
source paths, including retired docs; do not rewrite them. Continue in the normal
checkout without new sibling directories.
