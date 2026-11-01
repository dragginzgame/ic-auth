# Current handoff — 2026-11-01

## Delivered source

Protocol **0.30.0** (`ic-auth-protocol-types`, `ic-auth-wire`) is published at
`ec0233f3c4d746bb9bf7009f7432683a57c0b616`, tag `protocol-v0.30.0`.
Engine **0.30.0** (`ic-auth` plus private members) is published at
`80459e300cdb6fb67a94ff45d33f0774ebf0bd9f`, tag `engine-v0.30.0`.
Both annotated tags and release commits are pushed to GitHub main. All three
public crates are checksum-verified on crates.io. This library repository has
no production service or deployment target; no fixture was deployed to production.

The maintainer delegated adoption and delivery. Source preparation commits are
`f8130a8` and `8fe0785`. Protocol shipped first, then Engine against the published
supplier. Each group owns literal versions and `releases/<group>/CHANGELOG.md`;
Engine-only releases preserve Protocol identities and notes. Root notes index
these ledgers and the retained coordinated history through 0.29.2.

Development main is synchronized to the releases. A separate incoming lock edit
selecting Metrics **0.25.1** was preserved and restored, with a retained stash
identified in `target/persistence-wire-delivery/incoming-metrics-stash`. It is
uncommitted and outside the released lock and release qualification below.

## Adopted suppliers and boundaries

The released lock selects Memory/Storage **0.62.1**, Wire/derive **0.14.2**,
Platform/PocketIC client **0.17.0**, Types **0.16.0**, Runtime/macros/guard
**0.17.1**, Testkit **0.78.0**, PocketIC server **16.1.0**, Metrics **0.25.0**,
and Build/Host/file CLI/editor **0.73.6**. Compatible patch minima remain in the
catalog. One Wire provider serves persistence and RPCs. Shared Tooling remains
**0.27.25**, `982a99706e038810ee8108d5356a8637a7d42f67` (136 payloads).

Passive protocol and engine identity operations select Types directly. Crypto
capabilities explicitly select Platform; default/session-only graphs exclude the
facade. Align consumers on Types 0.16 and Wire 0.14; select external Principal
conversions explicitly at native boundaries. The private host selects Runtime
directly, with one executor/generator/guard. Public graphs exclude runtime,
Memory, Storage, Metrics, Candid and native tooling. DHAT stays dev-only.

Signed bytes, schemas, stored formats, cryptographic checks and TEE/non-TEE trust
policy are unchanged. BLS identity-key rejection remains covered. No local schema,
client or crypto shim is used. IcyDB and Canic checkouts were not changed.

## Qualification and delivery evidence

Both standard release commands ran the complete `make ci` gate in the isolated
checkout `/tmp/ic-auth-release.EqLgnm`, before version preparation. Passing logs:

- Protocol: `.git/release-state/validation-logs/validation.OQxMG0` in that checkout.
- Engine: `.git/release-state/validation-logs/validation.P7atRk` in that checkout.
- Adoption, release and publication summaries: development checkout
  `target/persistence-wire-delivery/`.

Each gate passed Clippy, graph/refusal policies, isolated Rust 1.91 native/Wasm
package and private-app checks, 69 browser tests, native authentication/session
suites, storage/checkpoint and metrics cases, 11 live PocketIC tests, Wasm and
normal three-archive verification, and release/publication recovery fixtures.
One expensive profiling test remains intentionally ignored. All five generated
browser files remain unchanged after locked Wire regeneration. macOS, live IC,
production issuer/backend and installed Canic acceptance are not established.

The first Protocol gate passed product checks but caught inherited Make group
controls in the missing-group release fixture. Commit `8fe0785` isolates the
fixture's Make environment; both later complete gates pass. Retain the failed
`validation.YInWyN` directory and `protocol-release-first-gate.log` as original
failure evidence. No real version preparation or remote effect began in that run.

The clean 0.30.0 backend archive at Engine release commit `80459e3` passed actual
registry-consumer identity and packaged-scenario qualification (seven tests).
Evidence is `target/portable-fixtures/backend-package.3Z8M7v/delivery.json` in the
release checkout; its SHA-256 is
`b812d874a8280d332bd324a1a7e6ee62e75f9996a6a3bd94c272ac5f2b60a69f`.
The unpublished bundle requires all four scenarios for full backend conformance;
reference and encoded-record drivers do not establish a production stable backend.

## Recovery custody and ownership

Keep the original release checkout, journals, receipts, publication intents,
archives and validation evidence. Both group attempts completed; no resume is
pending. Future fresh releases select a group explicitly. Recover any historical
attempt with its original source/adapter and exact saved group/version; never
reinterpret legacy journals or bypass retained publication/helper locks.

Earlier cohort, group and Types evidence remains under `target/cohort-adoption/`,
`target/memory-storage-adoption/`, `target/release-groups-work/` and
`target/types-boundary.BvpfC1/`; observations retain their original scope.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns adoption,
issuer/renewal, guards, durable backend and browser acceptance;
[#720](https://github.com/dragginzgame/canic/issues/720) owns storage deduplication.
No current Canic source inspection or installed adoption is claimed by this batch.
The [allocation observations](../canonical-allocation-observations.md) and
[Wasm attribution](../wasm-attribution.md) justify no further verifier/encoder
optimization or verification-result cache. The private browser client has no npm
publication or production issuer.

Memory/Storage remains private qualification support. Keep checked `StableKey`,
fresh-only `StableCell::create`, bounded loading and occupied-backing refusals.
Preserve `AUTHNAT3`, session/login `0x415554485311`/`0x415554485313` and replay
`0x415554485302`. The older Memory 0.56 ledger retirement is separate; never erase
backing or its ledger for admission. Hosts own clock, allocation, authority refresh,
atomicity and certification-root publication. Wasm keeps the qualified 32-bit
backend; compiler wrappers and cross-group validation reuse remain opt-in.
