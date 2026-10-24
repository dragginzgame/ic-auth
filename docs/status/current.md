# Current handoff — 2026-10-24

## Delivered source and local work

This checkout contains manually committed 0.16.0 metadata at `f22bab5`, but
0.16.0 has no remote tag or published packages at this review. Shared Tooling
0.26.14 and Auth's qualified recovery adapter are now adopted here. The original
saved plan and validation receipt remain untouched; no release was executed.

To target exactly 0.16.0 with fresh validation, reviewed base manifests are
prepared in `target/release-016-base/` for explicit maintainer application. They
restore only local package versions and sibling requirements to 0.15.4, retaining
all external selections (verified structurally). Commit that disposition together
with this adoption, then run `make release-minor`. The new runner archives the
old preparation plan without claiming delivery and runs a new complete gate.
Do not use exact resume after the source changes or manually tag the old commit.

Pending 0.16.0 adopts published Platform/macros **0.6.15**, Wire/derive **0.7.2**
and Shared Tooling **0.26.14** at
`e14050bcc0c627f9c8a9064051699cac44fd7ab3` (121-file snapshot).
Host/Build/Testkit **0.68.0** remain the latest published selection;
Tooling 0.68.1 is an unpublished draft. Memory and the File CLI are unchanged.

Shared Tooling #306 supplies explicit read-only admission waiting and busy status
75. Auth's saved-source release editor uses a 60-second wait and preserves busy
status without installing, repairing or selecting a partially prepared graph.
Ordinary offline checks remain immediate. The exact saved source continues to
own the lockfile, manifest and editor features.

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

The exact recovery scripts were qualified in `target/upstream-adoption/` before
adoption here. `target/upstream-adoption-*.log` retains that qualification output;
`target/shared02614-adoption/` retains current-checkout adoption checks.
Type/transport and all 54 browser tests, Host tooling, dependency boundaries,
isolated native/Wasm Rust 1.91.0 MSRV and Clippy pass. All five generated browser
files were refreshed and remain byte-identical. Published Platform 0.6.15's
`src` tree is identical to 0.6.14; its Wire dependency selection changes.

The Auth release/recovery fixture passes with explicit waiting, busy-status
propagation through the actual metadata adapter, and no installation or metadata
mutation on contention. It also qualifies Shared 0.26.13's fresh-release behavior
from manually committed metadata without inventing an earlier delivery. Initial
fixtures assumed `--check` was last and required earlier delivery; both were
updated to the adopted contracts.
The first PocketIC run hit Unix socket path length in this nested checkout;
all 12 cases pass with a short temporary state directory and the same compiled
Wasm, native runner and admitted server. These initial failures remain in their
logs and are not passing evidence. Testkit caller, failure-evidence, jobserver
and formatting-hook fixtures, dependency pins and source-following ShellCheck
also pass. Shared runner recovery tests, snapshot integrity, formatting and
517 local documentation references pass.

The saved 0.16.0 release plan and receipt are unchanged. Root package versions
remain 0.16.0 until the maintainer applies the prepared base files. This batch
needs fresh source validation; never reuse the old receipt for changed inputs.
No full CI gate, native macOS, installed-IC or production-browser result is
claimed. No commit, push, publication or real release was run.
