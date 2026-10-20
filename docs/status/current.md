# Current handoff — 2026-10-20

## Source and pending release

Baseline is published Auth 0.11.1 at
`0e64aa852e5f284b8d36dbc32df5702373f48149`, plus the preserved incoming Shared
0.25.7/early-Clippy work, Platform 0.5 selection and lock-only updates. Own
package versions remain 0.11.1. Pending notes select **0.12.0**, because Auth
exposes the new Platform Principal crate identity. No release/version mutation,
commit, push, publication or deployment is authorized by dependency adoption.

Published Platform/macros **0.5.0** archives are verified unyanked and
checksum-matched to Cargo.lock; Platform's source is
`b840423928f0ca7f14d7fc860508730c87356640`. The incoming root already selected
Platform 0.5. The manifest and lock are preserved byte-for-byte, including Memory
0.49.3, Wire 0.5.1, Metrics 0.14.11, Host FS/Process 0.31.2, Build 0.23.0 and
Testkit 0.59.0. No sibling/path override or broad resolver update is used.

## Platform 0.5 adoption

Auth's public Principal now has Platform 0.5's Rust identity. Consumers sharing
it must align their direct Platform selection; see the
[migration](../canic-adoption.md#platform-05-migration). External Agent/Testkit/
cryptographic parameters retain byte-preserving conversions. The independently
published Testkit/Memory graphs still select Platform 0.4.1; these are not forced
onto the new Rust identity. Crypto implementations, DER/CBOR/signed bytes,
protected signer/root/seed/freshness policy and frozen record/frame bytes remain
unchanged. There is no storage, journal or checkpoint migration.

Only the unpublished host removes retired no-op `hidden` macro arguments. It
keeps explicit Wire codecs, endpoint identities, controller checks and lifecycle
hooks. It uses none of the removed storage/readers or `change_timeout` APIs.
`cdk` remains necessary for its entrypoints; public library capabilities remain
runtime-free. The runtime retains exactly the qualified registry executor 2.0.0
and ic0 1.2.0 identities. Public capabilities and reference Wasm exclude Candid;
native Agent/Testkit still retain Candid transitively. No direct Candid/CDK or
replacement crypto/runtime implementation is added.

## Shared Tooling 0.25.8

The canonical 118-file snapshot is pinned to
`0a2d2ee3b5e0ae77755c2915ac28836d4d5ba359`. Preserve the incoming complete-Clippy
ordering and its parallel-Make rejection coverage from 0.25.7. The selected
0.25.8 finalizer accepts numbered `- Unreleased` headings and dates them before
commit/tag/push. Auth's preflight now admits both supported spellings. The local
release fixture accepts the marker through a complete synthetic release and
checks the saved UTC date on the finalized heading; unsupported suffixes and
conflicting dates still refuse before setup/validation. Recovery retains its
source-bound original notes and date. No new installer/build policy or tool
selection is introduced by this snapshot refresh.

## Qualification and ownership

Full native/Wasm Clippy, protocol/type/Wire, chain-key, portable/adversarial,
canister signature/store, certified proof and session/restore suites pass.
The isolated Rust 1.91 public-package native/Wasm capability matrix and internal
apps pass, with unchanged archives/inputs. Five native fixture tests, fresh
release Wasm and all nine local Testkit 0.59/server 16.1.0 scenarios pass:
certification/composed roots, upgrades/checkpoint recovery, explicit-root and
signed ingress, cancellation and actual SDK two-receiver admission/logout.
The issuer lookup and IndexedDB are injected reference boundaries, not
production issuer or browser-persistence acceptance.

All 11 Host-tooling tests, Wasm capability builds, actual dependency-boundary
exclusions/refusal fixtures, formatting, pins, links, snapshot, Make jobserver,
early-Clippy input-seal cases and the substituted release runner pass. Three
coordinated package dry runs pass without uploads. The actual local/mocked
release/recovery fixture passes with Unreleased finalization. No new Platform
API gap or in-repo exclusion blocker was found; native macOS and full CI remain
unrun for this pending source.

Auth owns proof/certificate semantics, checked types, local session/replay/denial
machinery and bounded Wire transport. Canic owns protected fleet authority,
production stores/guards and issuer orchestration. Remaining production acceptance
stays in Auth #3/#58/#59 and Canic #491/#507. No sibling files were edited.
Before-images, archive/graph observations, qualification artifact identity and
logs are retained under `target/platform050-shared0258-20261020/`. The saved
incoming Git patch plus baseline reconstruct all incoming tracked files. Earlier
evidence remains separate. The real index and release metadata are preserved.
