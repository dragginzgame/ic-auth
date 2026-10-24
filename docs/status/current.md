# Current handoff — 2026-10-24

## Delivered source and local work

The checkout is based on release **0.15.4**, commit
`754ef9cb0dfd19653e70f594e86378d90ae76a13`; package versions remain 0.15.4.
Pending **0.16.0** notes cover published IC Wire **0.7.1** and Tooling **0.68.0** adoption. The minor
cut reflects the public `ic-auth-wire::wire` Rust type/error boundary, not a
change to encoder bytes, method IDs, signed claims or retained storage formats.
No commit, push, package version change or release ran for this adoption.

Public Auth contracts, native Testkit, private control contracts and the reference
Wasm now share Wire/derive **0.7.1**. The temporary Wire 0.6 selection is removed.
Published Wire source is `2978dee37ddaf62c09ce1df411a0b1b6c9748bcd`;
Host FS/Process, Build and Testkit **0.68.0** come from
`8f281177faaeee0adc51edc1031d4a7b268b0ce0`. Platform/macros **0.6.14** and
Memory **0.50.16** are unchanged. The independent file CLI remains **0.31.1**.
Shared Tooling remains the 121-file snapshot at
`4c0f198e56d4c6d27b12a5c2d515e0e01070151e`: canonical VERSION **0.26.10**,
although the producer changelog names **0.26.11**.

Wire 0.7 decodes once without reserialization. Auth transport has no normalizing
maps/sets/custom DTO visitors: it preserves lists and opaque signed bytes.
Semantic scope/grant checks remain in Auth. Primitive encodings remain strict;
`Error::Noncanonical` is removed and malformed primitives return `Malformed`.
Keep original frame bytes for exact retries, hashes and signatures.

The native runner passes canonical limits directly to Testkit's `WireCallSpec`.
The temporary alias/helper and response-only Serialize bounds are removed,
adopting [Tooling #248](https://github.com/dragginzgame/ic-tooling/issues/248).
All five generated browser files were refreshed. Only the shared runtime changes,
adding Packed<i8>/Int8Array support; Auth's four schemas remain byte-identical.
The optional Wire bulk-schema advice is not enabled and no fields are migrated.

## Implementation and consumer boundary

Auth delivers both portable proof verifiers, one session engine, host-selected
storage/reset interfaces, bounded Wire contracts and the private browser
admission/logout coordinator. Canic owns topology, authority approval/renewal,
endpoint guards, production enrollment, deployment configuration and its durable
host adapter. The standalone wallet service is not implemented.

The [Canic adoption guide](../canic-adoption.md) describes config, Wire endpoint
wiring, capability profiles and the distinction between `(issuer, subject)` and
browser ingress caller. Consumers sharing Rust Wire types must align to 0.7.
Auth 0.15's admission method ID remains `0x415554480012`. Do not reset portable
journals, replay unknown issuance or restore retired private checkpoint formats.

[Canic #491](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382815988)
retains runtime/store/reset/load acceptance. [Canic #507](https://github.com/dragginzgame/canic/issues/507#issuecomment-6382816186)
retains issuer policy, issuance abuse, reconciliation and browser integration.
Auth #58/#59 were superseded coordination, not production acceptance. Auth
[#3](https://github.com/dragginzgame/ic-auth/issues/3) owns real-browser journal
qualification. The browser package stays private; fixtures do not prove production
Canic adoption, clock skew or TEE custody.

## Retained interfaces and follow-up

`AuthRequestMetadata`, Rust's empty `LogoutRequest` and older session error
conversions remain supported during Canic integration. See the
[consumer review](https://github.com/dragginzgame/canic/issues/491#issuecomment-6382404160).
This adoption does not change certificate freshness handling for
[Platform #51](https://github.com/dragginzgame/ic-platform/issues/51).
[Auth #85](https://github.com/dragginzgame/ic-auth/issues/85) retains the
crypto-bigint advisory correction/exemption and qualification through
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).

## Validation and evidence

Current adoption evidence and incoming working-tree copies are retained under
`target/wire071-tooling068/`. Type/transport tests and all 54 browser tests pass,
including Rust/TypeScript vectors and the existing nonminimal-length/scope-order
regression. Host-tooling tests, native/reference-Wasm Clippy, dependency boundaries,
isolated native/Wasm MSRV and six fixture unit tests pass. Formatting, dependency
pins and local links pass; manifest/lock hashes remained unchanged. The selected
Testkit and Build 0.68 executables are missing; explicit preparation approval is
pending before live-canister and release/recovery qualification. The source
adoption evidence is posted on [Tooling #248](https://github.com/dragginzgame/ic-tooling/issues/248#issuecomment-6384815492).

Earlier Wire 0.7.0/Testkit 0.67 evidence is in `target/wire-070/`, including
Clippy, MSRV, boundaries, six fixture unit tests and all 12 runner cases. Those
results do not qualify the new Tooling selection. The earlier incoming edits
are preserved; published source, not dirty sibling work, supplies this adoption.

The [0.15.4 handoff](https://github.com/dragginzgame/ic-auth/blob/754ef9c/docs/status/current.md)
records earlier cleanup evidence in `target/cleanup-0154/`. No full CI/release
gate, native macOS, installed-IC or production-browser result is claimed here.
