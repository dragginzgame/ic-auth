# Current handoff — 2026-11-08

## Published state and recovery

Engine **0.35.8** (`ic-auth`) and Protocol **0.35.1**
(`ic-auth-protocol-types`, `ic-auth-wire`) are published; release commits and
annotated tags are pushed to `origin/main`.

- Engine commit `9ed4afbd74c27f56e00aed2dea18fcb195ffb674`, tag `engine-v0.35.8`.
- Protocol commit `39b6e2605ad336c3aadff94bfd8af0dcb07cf8b3`, tag `protocol-v0.35.1`.

Frozen batch `batch-YyKgHPDM` is complete and fast-forward reconciled. Both groups
passed fresh 30-stage release gates; no upload is unresolved. Custody remains in
`.git/release-state/frozen/batch-YyKgHPDM/`, with registry receipts in
`.git/publication-state/engine-0.35.8.json` and `protocol-0.35.1.json`.
Raw release artifacts remain under `target/release-frozen/batch-YyKgHPDM/` and
`target/delivery-0358/`. No release, push or publication ran for the local work below.
[#127](https://github.com/dragginzgame/ic-auth/issues/127) remains closed.

## Authorized release

The maintainer authorized deployment of Engine **0.35.9** through the shared
frozen batch flow. Protocol remains 0.35.1. Before capture, the incoming lock
advanced to Build 0.79.1 and Storage 0.68.11; these are preserved for a fresh
complete release gate. Earlier focused checks and footprint observations retain
their recorded selections. No new publication is complete yet.

## Local cleanup batch

The normal checkout at `291fbf1` contains two accepted audit batches:

- Signature retention deadlines share existing tree leaves; the duplicate map
  is removed. Empty certified-leaf bytes and bounded expiry indexing remain.
- Canonical grants use a shared runtime-selected encoder; standalone checks and
  same-call portable admission evidence retain their existing behavior.
- Browser coordination captures one immutable destination before asynchronous
  work, reconstructing mutable SDK Principals only at boundaries.
- The private journal drops unused per-target admission deadlines. Exact
  credential expiry, renewal sequences and unknown issuance remain durable.
- Prepared issuer keys reuse their original encoding retained by Platform.
- The fixture owns receiver presence in one enum, preserving checkpoint bytes
  and closed/restored authority gates. Its configured receiver is boxed once.
- Rust/TypeScript interoperability vectors share generation/check declarations.
- Redundant same-call eligibility and known-absent binding branches are removed;
  entrypoint policy, live authority and transaction-boundary checks remain.

The maintainer confirmed **no retained journals; hard cut is OK** on 2026-11-08.
The current profile is `ic-auth-login-cleanup-journal-v1`; the retired
`ic-auth-local-logout-journal-v1` profile is refused without clearing records.
See [client storage and retirement](../browser-client.md#durable-logout-and-guarded-identities).
This permission does not authorize erasing Canic or other retained safety state.

Engine's next pending notes are **0.35.9**, a compatible public Rust cleanup;
private browser storage retirement is explicitly documented. Package versions
await the authorized release adapter. Both implementation batches and the policy
adoption are ready for source capture. No Protocol contract changed.

## Selected dependencies and Shared Tooling

The incoming Wire/derive **0.17.14** lock is preserved. Regeneration with that
locked generator leaves all five browser files byte-identical. Other selections
remain Platform 0.26.0 (native Testkit alone retains 0.25.1), Types 0.18.2,
Storage 0.68.10, Runtime/macros/guard 0.20.1, Testkit 0.93.1, Build 0.79.0,
Metrics 0.29.6. The subsequent batch preserves incoming native Host 0.75.1;
it was already selected in that batch's baseline. Compiler: Rust 1.99.1.
No packed-ID codec is selected.

Shared Tooling **0.31.11**, revision
`6c529e6940a97226cca826726491e03173b08cd2`, remains adopted locally through the
canonical exporter with the 165-payload closure. Its policy-only qualification
is under `target/shared-tooling-adoption-03111/`. Ordinary issue closure follows
implementation and focused proof; delivery status and issue-specific acceptance
remain explicit. This refresh did not change executable tooling or packages.

## Focused qualification and footprint

The earlier index/journal batch passed 31 focused Rust cases, all 95 client cases,
the packed-consumer and Wire proofs, native/Wasm Clippy, and two installed cases.
Its source/lock/measurement evidence remains in `target/evidence/signature-index-20261108/`.
Those results retain their original scope and dependency selection.

The follow-up passes 52 selected Rust cases: signature/revocation, adversarial
feature unification, eligibility/liveness, fixture state and vector admission.
Seven browser codec cases and Rust roundtrip admission pass; all five generated
files remain identical. Formatting, selected native/Wasm Clippy and both installed
receiver/SDK and checkpoint/fresh-authority regressions pass. No full repository
gate or Chromium run is claimed. See [measurement method and limits](../canonical-allocation-observations.md#signature-retention-and-shared-grants--2026-11-08).

The follow-up's final fixture is **1,884,402 total / 1,449,460 code bytes**:
**238 fewer total bytes, 841 more code bytes** than its matched baseline.
SHA-256 `94aa95c3d28fdc3cdd023320574cd726c26e6b1f593191f5a5327ee87ffda44d`.
This retains simpler state ownership and less key data, not a code-size win.
Evidence: `target/evidence/prepared-state-20261108/`; reference qualification Wasm
is not Canic's deployed binary, and no speed claim is made.

## Auth/Canic ownership and retained evidence

The [Canic contract](../canic-adoption.md) and
[clock-independent design](../design/portable-clock-audit.md) retain 90-day login
windows, short proofs, 30-minute receiver leases, bounded unknown denial and
lookup-only uncertain issuance. Renewal never extends the original login window;
ordinary authorization stays local. Lost protected safety state stays closed
pending trusted recovery. Complete durable restore plus fresh authority is supported.

The last [read-only Canic review](../canic-adoption.md#read-only-canic-review--2026-11-07)
found no Auth implementation gap. Canic's separately accepted extraction into
`canic-auth`, mounting, persistence composition and installed acceptance belong to
[Canic #491](https://github.com/dragginzgame/canic/issues/491). Earlier integration
proofs do not qualify that new arrangement. Canic remains read-only here.

Superseded handoff detail remains in
[Git history](https://github.com/dragginzgame/ic-auth/blob/291fbf1/docs/status/current.md).
Earlier cleanup/profiling evidence and release recovery records remain retained;
raw files lost in the earlier cleanup incident cannot be independently verified.
Do not clean active delivery inputs, erase journals or place build trees in `.git`.
