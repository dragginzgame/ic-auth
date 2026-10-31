# Current handoff — 2026-10-31

## Delivered release

Auth **0.26.0** is live at `bd5f3fc555376ebeda47c89ac4ec6903a2cdf496`.
[Required CI 41112459602](https://github.com/dragginzgame/ic-auth/actions/runs/41112459602)
is complete and successful on Linux, Apple Silicon macOS and Intel macOS.
The finalized changelog records the dependency, schema-ownership and ordinary-module
work. Adoption trackers [#120](https://github.com/dragginzgame/ic-auth/issues/120)
and [#122](https://github.com/dragginzgame/ic-auth/issues/122) are closed against
this exact delivered source and CI, retaining their earlier evidence and limits.

Released selections are Build/Host/Testkit **0.72.0**, Platform/macros/runtime guard
**0.11.1**, Metrics **0.21.0**, Memory/Storage **0.58.0**, Wire/derive **0.10.0**
and Shared Tooling **0.27.11** at
`62edfbd43087a718bf0283e9040676a49211976d`. Rust **1.91** remains the supported
minimum; development uses **1.99.1**. No release or publication was run by this closeout.

## Current local work

The authorized upstream adoption selects published Platform/macros/runtime guard
**0.15.2**, canonical Types **0.15.1**, Wire/derive **0.13.0**, Build/Host **0.73.1**,
Testkit **0.75.0**, Memory/Storage **0.60.0** and host Metrics **0.24.0**.
Shared revision `143a9abb9ae81feae9400ff3844b03b1ddfdd75a` is exported from a clean
isolated committed checkout and independently verified (135 payloads). Its
committed VERSION is **0.27.21**, carrying the **0.27.22** catalog guidance.
All nine workspace members have full version/path catalog entries, adding no
dependency edges. Auth retains one coordinated release, workspace and lockfile.
Pending notes target **0.27.0** for exposed Rust identities; own versions stay
**0.26.0**. Existing dirty work and unrelated lock selections are preserved.

File/editor **0.73.1** and Testkit CLI **0.75.0** were explicitly prepared and
admitted with PocketIC **16.1.0**. Common executable pins are unchanged.
Auth's Metrics defaults remain disabled (arithmetic/validated Serde only).
Published Testkit separately retains arithmetic-only Metrics **0.23.2**; no Metrics
values cross that boundary and neither version selects native metadata.

Current evidence is under `target/upstream-adoption.IDwJwA/`. Strict Clippy passes
for public libraries, native/qualification/adversarial callers and the Wasm host.
All dependency boundaries, pins, grouped formatting and snapshot checks pass.
Public type/transport tests pass (15), engine tests pass (197), adversarial
signature/revocation tests pass, host/fixture unit tests pass (11) and metrics
tests pass (5). Old Memory ledger refusal still preserves application backing.
All five browser sources were regenerated/reviewed and remain byte-identical;
`generated-review.json` records their hashes. Client qualification passes 69 tests,
packed-consumer admission and Rust interoperability. The initial sandbox Node
subprocess refusal is retained separately from the admitted pass.

`make check-msrv` passes the Rust 1.91 native/Wasm capability matrix against
the actual three package archives, plus internal applications; archived inputs,
manifests and locks stay unchanged. Optimized qualification Wasm builds.
Installed PocketIC qualification passes all 11 enabled cases: certification,
composed roots, upgrades, live authority, durable sessions, signed ingress and
actual SDK admission/logout transport. The explicit Chromium case stays ignored.
Auth's `make test-release-tools` passes against the selected editor and updated
catalog: patch/minor/major metadata, exact tags, failure admission, retries,
interrupted preparation, restored-checkout recovery and publication reconciliation
(local bare remotes/mocked registry effects). Its retained fixture is
`target/portable-fixtures/release-tools.42IFmf/`; installed state is
`target/portable-fixtures/qualification.H4CwpY/`.
No full CI, native macOS or real Chromium/IndexedDB run is claimed for this batch.
Release 0.26 CI is evidence for its own source, not this changed graph.

The previous published-supplier blockers are resolved by the new pairing:
[Wire #36 — Keep codec and schema identities stable across Platform integration releases](https://github.com/dragginzgame/ic-wire/issues/36)
and [Tooling #274 — Align Testkit typed Wire calls](https://github.com/dragginzgame/ic-tooling/issues/274).
No copied schemas, aliases, patches or replacement transports are needed.
Previous evidence retains its own identities under `target/adoption-20261031/`,
`target/shared-adoption-02720/` and `target/platform-adoption-20261031.306Irl/`.
No root staging, commit, push, version bump, normal sibling mutation, merge,
publication or artifact cleanup was performed.

[PR #121](https://github.com/dragginzgame/ic-auth/pull/121) is closed as superseded
by release 0.26, without merging its second private `qualification-support` owner.
Its intended behavior is already delivered through `qualification-fixtures`:
the metrics implementation matches, receiver configuration differs only in its
Principal import, the signatures root matches, and the metrics test uses the
maintained prepared-key API. Preserve the released canonical owner and its fixes;
do not add a duplicate support package or restore the obsolete 0.25.2 draft.
The isolated comparison is retained under `target/post-026-closeout-20261031.RuDibH/`.

## Published consumer contract

Wire **0.13** owns Principal/Account schema metadata through `platform-types`.
Platform **0.15** reexports the canonical definitions from Types **0.15.1**;
the reverse Wire dependency and retired `wire-schema` remain absent.
Auth Wire's public `typescript` feature forwards `ic-wire/platform-types`;
DTOs, schema shapes, byte limits and codec behavior are unchanged. Passive types
select only `principal`; hashing and external conversions are explicit. Consumers
sharing Principal/parse-error identities align canonical Types **0.15**, and those
sharing public Wire types align to **0.13**. Auth retains raw Platform codec hooks
and needs no Endpoint dependency. The host retains one runtime guard/
generator/executor; public graphs exclude runtime, Candid, native tools and storage.
See [migration](../canic-adoption.md#platform-015-migration).

The private browser client supports qualified local archive delivery. Production
issuer transport, durable consumer distribution and deployment remain Canic-owned.
An archive for adoption must be generated from the intended delivered commit;
a dirty candidate or ignored build directory is not final distribution.
[Actual Chromium/IndexedDB acceptance](https://github.com/dragginzgame/ic-auth/issues/3#issuecomment-6464039789)
was delivered in 0.19.2 with its recorded Linux/browser/origin/issuer-fixture limits.
Real-browser execution remains opt-in; do not infer a new run from ordinary CI.

## Canic integration status

The last inspected Canic coordinated checkout selected published Auth/protocol
**0.26.0** and the aligned Platform/Wire/Memory family. This is consumer dependency
adoption, not production portable login/session acceptance. Current application
access still denies; the shared session adapter remains test-gated.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns production
integration, including the former #507 issuer/reconciliation criteria and removal
of the empty `canic-auth` scaffold.

[Persistent receiver PR #701](https://github.com/dragginzgame/canic/pull/701) and
[enrollment PR #703](https://github.com/dragginzgame/canic/pull/703) remain open on
older dependency/source bases; their last inspected hosted runs failed. They need
current-source integration and qualification. Protected registration/enrollment,
refresh scheduling, real admission/read/logout and allowed scoped operations,
issuer issue/lookup reconciliation and full endpoint/browser acceptance remain
Canic work. No missing shared Auth engine API has been demonstrated.

## Evidence and retained-state boundaries

Release CI owns current delivery evidence. Earlier focused logs keep their exact
source/lock identities under the adoption directories named in #120/#122; preserve
all retained failures, binaries, backing states and release/recovery artifacts.
Our superseded Platform #107/#108 and Tooling #272/#273 proposals are closed without
merging; their branches/evidence remain. The private fixture library retains one
configuration/diagnostics owner outside public library graphs.

Current `AUTHNAT3`, session/login `0x415554485311`/`0x415554485313` and replay
`0x415554485302` formats remain. Earlier retirement consent adds no reset permission.
Older Memory allocation-ledger backings require a reviewed transition: never erase
only a ledger, reinstall or discard backing to obtain admission. Hosts own clock,
allocation, authority refresh, atomicity and certification. Cryptographic checks,
TEE/non-TEE trust and replay/charging rules stay unchanged. Library/reference
qualification establishes no production wallet service, issuer adapter or deployment.
