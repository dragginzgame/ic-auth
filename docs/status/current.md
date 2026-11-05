# Current handoff — 2026-11-05

## Published state

**Engine 0.34.1** (`ic-auth`) and **Protocol 0.34.0**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published on crates.io.
Engine's release commit and annotated tag are pushed to `origin/main`:

- Engine: `e483aea9407d450f37c131de086ad648047176e0`, tag `engine-v0.34.1`
  (`0256f4d0fbdf2638311b680651af840f3e0ff48a`). The registry archive matches
  SHA-256 `60bddb026bd4bce91da677e982fc363d73a4047cfcf41ddc9befc4e5c601826d`.
- Protocol: `106fda655ed55a3542e9383331132f9491904768`, tag `protocol-v0.34.0`
  (`08f41509062f490d4181b3c45aec2893f44b3745`). Its identities are unchanged.
- Exact publication custody: `.git/publication-state/engine-0.34.1.json` and
  `.git/publication-state/protocol-0.34.0.json`. New upload/readback evidence:
  `target/publication-engine-0.34.1.IAcRHm/`.

The Engine lock selects Platform 0.23.0, Types 0.18.1, Wire/derive 0.17.2,
Testkit 0.87.0, Storage 0.67.1, Metrics 0.29.3, Runtime/guard 0.19.2 and
Host/Build 0.74.4. Rust 1.99.1 remains the compiler and minimum. Protocol and
Engine release independently; satisfy Engine's declared Protocol requirement.
There is no pending publication or implementation blocker from this batch.

## Qualified changes

Testkit's async-only PocketIC client replaces the old blocking path. Each live
test owns one two-worker Tokio runtime, awaits network/Wire/lifecycle operations
and explicitly closes its instance before stopping the managed server. Deletion
failures fail the test; managed-server Drop still stops/reaps on panic. No nested
runtimes or blocking facade remain. Native Testkit and Auth now share Platform
0.23; the normal Wasm graph also selects that single version.

Platform owns bounded certificate and standard canister-signature envelope
codecs. Auth retains budgets, witness construction, root binding and store policy;
its direct CBOR dependency is removed. The browser selects published
@tokostudios/ic-platform 0.2.0, with verified npm integrity and SDK Core 6.1.1.
Signed material, session records, verification checks and TEE/non-TEE trust
assumptions are unchanged. Testkit remains private qualification infrastructure.
Metrics 0.29.3 has identical production source to 0.29.2; Auth's five metrics
semantics tests pass against it.

The complete Engine release gate passed **all 30 stages**, including native/Wasm
lint, isolated package/MSRV consumers, 70 browser tests and interoperability,
all 11 ordinary live cases, package builds and release/recovery/publication
fixtures. The separate opt-in Chromium case was skipped. The saved-source recovery
fixture now uses current dependencies when perturbing manifest order, preserving
its old-commit, broken-current-lock and editor-contention assertions.
Gate: `target/release-validation/engine-gate.6W8p07/logs/validation.IMS6Ub`.
Focused adoption evidence: `target/evidence/testkit087/` and
`target/platform023-adoption/`. These are Auth qualification results, not
production Canic acceptance or a cryptographic security audit.

Shared Tooling 0.30.4 is pinned to `2aa41c7394300302fe802b69b31b855bdc89b671`;
all 158 canonical payloads match. LOC/formatting scratch fixes, numbered drafts
and compact evidence rules are adopted. `check-candid-boundary` scans 102 owned
Rust files with zero allowances through the existing boundary gate; this is
lexical enforcement, not macro/alias analysis. Historical journals are retired
from the working tree, with useful conclusions and Git pointers retained. Compact
allocation/Wasm reports, release records and maintained fixtures remain.

## Accepted Wasm tradeoff

The maintainer accepted Platform's residual Wasm increase on 2026-11-05 and lifted
the publication hold. Platform's matched Canic report found approximately 29 KB
overhead versus 0.21 after its correction; this does not establish size parity or
erase Auth's earlier fixture growth. No Auth optimizer profile change or shared
decoder API shipped. Earlier comparison locations include
`target/testkit085-wasm/apples/`, `target/decoder-boundary-probe/` and
`target/canic-decoder-probe/`; their
[historical scope](https://github.com/dragginzgame/ic-auth/blob/ed785d7/docs/status/current.md)
does not establish availability of every raw artifact today.

## Ownership and recovery

[Canic adoption](../canic-adoption.md) and [signature preparation](../signature-preparation.md)
own consumer contracts. [Canic #491](https://github.com/dragginzgame/canic/issues/491)
owns production clock/restore acceptance, certified issuance, custody and installed
integration. Canic remained read-only here. The independent Platform browser SDK
release is complete at source `d30e01c`, tag `browser-v0.2.0`; supplier custody
remains at Platform `target/browser020-delivery/`.

Preserve `.git/release-state/`, `.git/publication-state/` and retained target
artifacts. Earlier 0.34.1 gates stopped on external Metrics lock drift and the
stale dependency-pair fixture; their evidence remains under
`target/release-validation/engine-gate.rl6XlY/` and `engine-gate.vabiHR/`.
The successful fresh gate supersedes them; publication has no unresolved upload.
Earlier 0.33 attempts were also superseded by successful releases.
Retired checkout refs are `archive/ic-auth-capacity` and
`archive/ic-auth-release-033`; retirement evidence is under
`target/checkout-retirement/20261104-siblings/`. Ignored artifacts remain local
and removable by explicit clean. Historical receipts bind original source paths,
including retired docs; do not rewrite them. Continue in the normal checkout.
