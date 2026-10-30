# Current handoff

## Delivered and selected scope

Auth **0.25.1** is live at `120c446d8c2a2187bb5383ef93ce13cfb449ec48`.
Its [CI matrix](https://github.com/dragginzgame/ic-auth/actions/runs/41065201923)
passes Ubuntu, macOS Intel and macOS ARM. Delivered #116/#119 and the earlier
nine trackers are closed with acceptance evidence retained in GitHub.
Preserve all release/recovery files, failures and retained binaries/backings.

[#122](https://github.com/dragginzgame/ic-auth/issues/122) now selects published
Build/Tooling/Host/Testkit **0.72.0**, Platform/macros/runtime guard **0.11.1**,
Metrics **0.21.0** and Wire/derive **0.10.0**. The independent Host file CLI pin
is **0.72.0**; it, Testkit and the Build release editor are explicitly prepared
and admitted. The latest batch adopts [#120 Memory/Storage 0.58.0](https://github.com/dragginzgame/ic-auth/issues/120)
and preserves Shared Tooling **0.27.11**. Own metadata stays **0.25.1**; notes remain
**0.26.0** for exposed Principal/parse-error and Wire Rust identity changes.
Root changes are unstaged/uncommitted; no full CI, release, publication or deployment ran.

## Published schema ownership and consumer contract

Wire **0.10** owns Principal/Account schema implementations through its explicit
`platform-types` feature. Platform **0.11** removes the reverse Wire dependency
and retired `wire-schema` feature, resolving the recurring trait-identity cycle.
Auth Wire's existing `typescript` feature now forwards `ic-wire/platform-types`;
its optional direct Platform dependency and that direct-dependency allowance are
removed. DTOs, schema shapes, byte limits and codecs remain unchanged. This
native metadata capability selects neither endpoint integration nor IC runtime.

Consumers exchanging Auth's Principal/parse-error types align to Platform **0.11**;
consumers exchanging public Wire types align to **0.10**. Passive types keep only
`principal`, with hashing/external conversion capabilities selected separately.
Agent/Testkit/crypto boundaries preserve original bytes. Testkit now shares
Platform **0.11.1** and Wire **0.10** with Auth. Memory/Storage **0.58** now share
those identities too, removing the older private Platform 0.10 and Wire 0.8
packages. No override or identity bridge is needed.
See [migration](../canic-adoption.md#platform-011-migration).

The host retains one coordinated runtime guard/generator and Platform's maintained
executor. Passive/public graphs exclude runtime guards, Candid, native tools and
fixtures. Existing missing/duplicate guard, feature-leak and generator refusal
cases pass. Platform's runtime/crypto/generator source, Build/Testkit source and
Metrics' Rust code are unchanged from the preceding selection. Wire's existing
codec/generator source is unchanged; only its schema integration moves ownership.
No signed/public frame, method ID, budget, retained layout or trust rule changes.

## Matched qualification

Latest Memory/Storage evidence is in `target/memory-storage-adoption-20261030.6Illbu/`:
before state, official indices, byte-identical 0.57-to-0.58 source comparisons,
metadata, dependency seal, logs and reference Wasm. Native/Wasm host Clippy, ten
host custody/reopen/bounds/corruption/old-ledger refusal cases, focused Rust 1.91
native/Wasm checks and dependency graphs pass. All eleven installed cases pass
certification, composed roots, protected checkpoint upgrades, live authority
refusal, signed ingress and SDK admission/retry/logout. Generated-contract
comparison passes with the unchanged Wire 0.10 selection. No storage framing,
application source, policy, setup selection or trust behavior changes.

The preceding shared-set evidence is in `target/latest-shared-adoption-20261030.dZ3Wzp/`: before state,
official indices, resolved/accepted metadata, dependency seals, logs and reference
Wasm. Native affected-package and Wasm-host Clippy, core library tests, 17 adversarial
feature-unification cases, ten host plus one fixture cases, five metrics cases and
native/Wasm dependency/refusal checks pass. Full `make check-msrv` passes isolated
package payloads and supported capabilities on Rust **1.91**, then internal apps;
its archives, unpacked source and locks stay unchanged through the check.

All five browser files were regenerated and reviewed byte-identical. Client/
packed-consumer and Rust/TypeScript interoperability checks pass, including all
69 client cases. Actual Host file/lock/bundle and Testkit setup/check/seal caller
fixtures pass. Eleven installed cases pass certification, composed roots, protected
checkpoint upgrade, live authority refusal, signed ingress and real SDK admission/
retry/logout through prepared Testkit **0.72.0** / PocketIC **16.1.0**. The Build
**0.72.0** editor is prepared/admitted; no real release metadata is rewritten.
The separate Chromium case remains ignored pending explicitly prepared browser
setup. No new full CI, release-recovery suite, macOS acceptance or comparative
size/performance result is claimed.

## Prior contributions and preserved work

Published suppliers supersede the earlier Wire 0.8/0.9 alignment proposals.
Our Platform PRs #107/#108 and Tooling PRs #272/#273 are closed without merging;
branches/evidence remain. Supplier trackers #105/#271 receive the new adoption
contract; #122 remains open through actual Auth delivery/acceptance. Normal sibling
working checkouts and their pending work are untouched.

Shared source remains `62edfbd43087a718bf0283e9040676a49211976d` (**0.27.11**),
with its previously verified **132-file** canonical export unchanged by this batch.
The generic publication adapter stays opt-in; Auth owns release identity,
upload/retry and recovery policy. The private qualification fixture library owns
checked configuration and volatile admission metrics through ordinary module
imports. Existing module/lint cleanup is preserved outside public library graphs.
Earlier evidence remains under `target/memory-metrics-storage-wire-20261030.gaQnXB/`,
`target/tooling-platform-wire-20261030.ATp3to/`, the earlier adoption directories and
owning issues. Failed Wire attempts describe past supplier gaps, not current admission.

## Retained state and trust

The #120 pair/floors are **0.58.0**. Earlier custody evidence remains in
`target/persistence-adoption-20261030/` and #120. Current `AUTHNAT3`, session/login
`0x415554485311`/`0x415554485313` and replay `0x415554485302` formats remain.
Earlier retirement consent adds no reset permission. Older Memory allocation-ledger
backing is incompatible even with `AUTHNAT2`: retain old binaries/states and
require a reviewed transition. Never erase only a ledger, reinstall or discard
backing to pass admission.

TEE/non-TEE trust, protected authority/clock, charging/replay, verification checks
and atomicity are unchanged. Hosts own allocation and certification. Canic owns
endpoint/issuer/lifecycle acceptance in [#491](https://github.com/dragginzgame/canic/issues/491)
and browser issuance in [#507](https://github.com/dragginzgame/canic/issues/507).
These fixtures establish no production wallet service, issuer adapter or deployment.
