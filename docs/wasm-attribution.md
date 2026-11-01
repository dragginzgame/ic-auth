# Canic receiver Wasm attribution — 2026-11-01

This diagnostic does not justify another large Auth cut. The optimized Canic
probe is **2,375,276 bytes**. Its named companion contains 5,667 bytes of directly
named Auth functions and 7,347 bytes of Auth-shaped generic glue. These are
symbol ownership observations, **not the bytes removable by deleting Auth**:
LTO, inlining and shared implementations prevent that interpretation.

No optimization, verification change, cryptographic change or TEE/non-TEE trust
change was implemented. Successful admission cost cannot yet be measured through
this probe: the selected Canic public authorization facade deliberately denies,
and the shared session engine adapter is compiled only for tests. Production
integration remains with [Canic #491](https://github.com/dragginzgame/canic/issues/491).

## Source and build scope

The selected application is Canic's existing
`canisters/audit/local_authorization_probe`, role `app`, network `local`, using
its own `canic.toml` and the Host `build_artifact` example. This is qualification
infrastructure, not an installed production receiver. The snapshot is Canic
`e753785953e6805461ac337868c32163f65c3209` (Release 0.114.0) plus its existing
working dependency/docs/CI patch, captured separately with SHA-256
`58e3399bad05febe573c8530d826b85f165833469ff0507e372733031c4e57df`.
The normal Canic checkout was neither modified nor built. A local clone with
independent files reproduces that captured patch inside Auth's `target/`.

That source selects Auth/protocol **0.28.0**, Platform **0.15.3**, Types **0.15.1**,
Wire **0.13.2**, Memory/Storage **0.60.4** and Metrics **0.24.3**. It does not
measure the current Auth 0.29 checkout or its incoming Memory/Storage 0.60.5.
The selected normal Wasm graph contains no Solana package and one version each
of K256, Ed25519 Dalek, Curve25519 Dalek, BLS12-381, pairing and SHA-2. It offers
no obvious duplicate-crypto version reduction. Feature selection alone does not
prove that a verifier or generator remains in the final Wasm.

Canonical Release construction failed before the runtime build: Canic selects
the different `~/.local/bin/wasm-opt` before the admitted repository-local tool.
Both report Binaryen 133, but only the local tool has the required hash. Fresh
caller evidence is attached to the existing
[Canic #705](https://github.com/dragginzgame/canic/issues/705#issuecomment-6555941270).
No global executable, HOME or admission check was changed.

The supported `fast` Host path then passed with release-matching compiler
settings: Rust 1.99.1 on Linux x86_64, `opt-level=z`, full LTO, one codegen unit,
no debug information/assertions, aborting panic, no overflow checks/incremental
compilation, and retained names. The Host still owns role/protocol validation,
declarations, runtime construction, shrink and metadata. Explicit invocation of
the admitted Binaryen 133 with `-Oz` followed. This is a **diagnostic build, not
canonical Release qualification**. No runtime admission, latency or instruction
benchmark was run.

## Artifact sizes

Code-section byte lengths were not measured in this historical diagnostic.
Twiggy function-body totals below exclude section framing and must not be used
as code-section measurements or IC admission headroom. Shared Tooling 0.27.24
requires separate module/code-section measurements for future size comparisons.

| Artifact | Bytes | Interpretation |
| --- | ---: | --- |
| Raw runtime, names retained | 7,491,252 | Cargo output before Host shrink |
| Host fast output, stripped | 2,696,617 | Shrink and local Candid metadata, before Binaryen |
| Explicitly optimized, stripped | 2,375,276 | Diagnostic size observation |
| Explicitly optimized, names retained | 3,287,956 | Companion used for symbol attribution |

The existing optimizer pass removes 321,341 bytes (11.92%) from the stripped
fast output. **This is not a new Auth saving**; it is the normal release
optimization step applied explicitly for this diagnostic.

Removing debug/name information from both optimized variants through Binaryen
produces equal 2,378,547-byte sizes but different hashes. Therefore the named
companion is not byte-identical evidence for ownership of the stripped result.
It supports investigation, not exact release-byte allocation.

## Retained symbols

Twiggy 0.8.0 shallow items were classified after excluding data segments, names
subsections, custom sections and Wasm structure. Rust v0 crate disambiguators
were normalized, including variable-length hashes. All categories below are
disjoint and sum to the named companion's 3,287,956 bytes.

| Symbol category | Bytes |
| --- | ---: |
| Direct Auth/protocol symbols | 5,667 |
| Auth-shaped generic glue | 7,347 |
| Crypto provider symbols, shared ownership | 5,992 |
| Direct Canic symbols | 646,808 |
| Direct Platform symbols, including inlined Canic handlers | 154,674 |
| Direct probe symbols | 1,368 |
| Other upstream symbols and generic glue | 1,317,880 |
| Unnamed function bodies | 17,068 |
| Data, names and structure | 1,131,152 |

There are 2,156,804 code bytes across 5,058 function bodies, consistent with
Binaryen's 5,097-entry function map minus 39 imports. The largest directly named
Auth function is `Session::decode` (5,370 bytes); `ScopeLabel::validate` contributes
235 and `is_login_caller` 62. Other Auth code may be inlined or shared. Absence
of a standalone verifier symbol is not proof that its capability can be removed.

The largest named bodies are Platform entrypoint specializations for Canic's
runtime query (43,269 bytes), runtime-operation query (38,349) and runtime
configuration (23,069), plus Canic automatic ICP refill (24,999). The wrappers
contain inlined endpoint implementations. Twiggy's aggregate monomorphization
estimate for six `enter_current_method` instances is 128,232 bytes, with a
heuristic 84,963-byte duplication estimate. **That is not an attainable saving**:
the instances include different handlers, not simply repeated executor logic.

## One bounded experiment to consider

If a further small size experiment is wanted, target Canic's
`application_sessions/login_restore::validate_login_records`. It decodes sessions
and collects them into a `BTreeMap` through `Result`/iterator adapters. Retained
Auth-shaped stable quicksort and drift-sort specializations account for 1,543
and 1,368 bytes respectively. An explicit insertion loop could avoid the bulk
collection sort specialization. Other generic functions may remain shared, and
replacement code has a cost; these numbers are not a predicted net saving.

Before adopting such a change, preserve decoding/error order, duplicate-caller
semantics, audience/login consistency and protected occupancy checks; compare
the same restore rejection tests and matched optimized artifacts. This report
does not implement that experiment or claim a performance improvement. The
existing persistent-adapter deduplication owner remains
[Canic #720](https://github.com/dragginzgame/canic/issues/720).

## Report correctness and evidence

The actual Canic capability-report caller completed, but its revision-2
classifier mistakes real Twiggy `data segment` and names-subsection items for
named code. It counts 1,126,534 non-code bytes as code even while its total-byte
check passes. The distinct defect and real-producer examples are filed in
[Canic #724](https://github.com/dragginzgame/canic/issues/724). Its ownership totals
were not used for the table above. No Canic classifier source was modified.

Local evidence is retained at `target/canic-wasm-attribution.LEJv31/`:

- Source/status/manifest/lock receipts and the captured working patch; final
  comparisons confirm both the normal checkout and isolated source stayed stable.
- `native-build.log`, the failed `receiver-build.log`, successful
  `receiver-diagnostic-build.log`, and `diagnostic-environment.txt`.
- Tool/artifact hashes, normal graph and feature paths, `receiver-items.json`,
  `receiver-monomorphizations.json`, `function-map.txt`, `owner-summary.jq`,
  `owner-summary-v2.json` and `sizes-final.json`.
- The actual `canic-classifier-report.json`, normalization outputs and failed
  byte-equivalence comparison. Earlier draft summaries are superseded by the
  explicitly named final JSON files.

The initial Shared fixture export was cancelled before copying files; the
independent clone was used instead. The initial feature-discovery call without
Canic's enabled Wasm flags failed validation; the admitted feature-discovery
and optimization calls passed afterward. Those attempts remain preserved.
These ignored local artifacts are not durable distribution. No full CI,
PocketIC journey, deployment or successful production receiver is established.
