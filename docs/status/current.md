# Current handoff

## Source and delivery

Auth 0.11.0 is released at `5cfc065`; own package versions remain 0.11.0.
The pending [0.11.1 notes](../../CHANGELOG.md) cover the native Host file-CLI
cutover. Staging, release, publication and deployment remain separately owned.
Sibling checkouts are read-only; this contribution uses an isolated PR checkout.
GitHub issues own follow-up tracking.

Shared 0.25.4 remains adopted at `db555ab6cf4a93af9a183f18d921d3b0ee97c725`.
Platform 0.4, Wire 0.5, Host 0.31.1, Memory 0.49.1, Build 0.22 and Testkit 0.57
selections are preserved. No sibling/path override or library dependency change
is delivered by this file-tool cutover. The prior implementation and qualification
handoff is retained in committed source `5cfc065:docs/status/current.md`.

## Ownership

Auth owns proof/certificate semantics, checked application types, session/replay
and bounded Wire transport. Platform owns runtime/crypto integration; Wire owns
codecs; Memory owns memory primitives. Canic owns fleet authority and production
issuer/store integration; Toko owns application accounts. Host, Build and Testkit
remain native-only dependencies outside deployed Auth graphs.

Released 0.11 preserves frozen 0.10.7 signed/frame and session/replay bytes while
exposing owned Platform Principal and Wire 0.5 Rust identities. Keep explicit
external DTO conversions and all protected root/signer/seed/freshness admission.
No storage reset, journal rewrite or cryptographic replacement is authorized.
The private browser client retains injected identity/issuer/storage boundaries.
Production acceptance remains with Auth #3/#58/#59 and Canic #491/#507; the real
SDK fixture does not establish production issuer or browser-store deployment.

## Pending Host file cutover — Auth #78

`ci/host-file.env` explicitly pins published `ic-host-file` 0.31.1. Cargo.lock
contains multiple Host FS versions; this executable selection is deliberately
independent, exact and unchanged during workspace metadata preparation.
`make install-host-file` prepares `--features cli` through the canonical Shared
store. `make host-file-check` and the thin `run-host-file.sh` only admit it offline.
Auxiliary tool identity uses the declared development compiler, including inside
`cargo +1.91.0` test processes. No implicit setup or fallback remains.

Auth's generic Rust file branches and local NUL-batch implementation are removed.
The private Rust utility retains browser contract generation and Build-owned
external-lock admission. Shell consumers delegate hashing/private publication to
Host, explicitly passing the existing 4194304-byte/4096-file batch allowances.
Auth retains source/input seals, intent/marker schemas, uncertain upload recovery,
byte budgets and before/after-publication decisions. CLI callers use standard
`--` when a positional path begins with a dash; NUL batches preserve raw bytes.

Fresh admitted release preflight includes selected file-tool setup. Ordinary
checks, publication and interrupted metadata recovery only admit prepared tools;
the independent pin does not depend on partially rewritten Cargo metadata.
CI cache identity includes that pin. No public library, browser contract or
canister source changes as part of this cutover.

## Qualification

Focused evidence is retained outside this checkout under
`/tmp/host-followups.oBBZP6/`: real published CLI preparation was cache-only.
Eleven Rust 1.91 tooling cases, strict development Clippy, actual file/batch
contracts and local/mocked release/publication interruption/reconciliation pass.
Tool setup/check failure evidence, formatting, snapshot/pins/links, ShellCheck and
Actionlint pass. Default/all-feature public Wasm graphs exclude Host/Build/Testkit.
Tooling tests, Clippy and the release fixture also pass after integrating the
published 0.11.0 release metadata; root manifests/lock and release receipt match main.
The hosted Linux failure at `61af6cc` exposed the Testkit caller fixture's old
aggregate-tool roster. Its updated fixture now passes locally, including drift
refusal for the file-tool pin and delegation scripts sealed by CI.
Native macOS, full local CI, browser runtime and installed-canister checks were
not run for this native-only patch. Required hosted checks and owner review remain
before PR integration. Auth #78 owns the remaining default-branch consumer
acceptance; Host's published producer does not require another implementation.
