# Historical Canic receiver Wasm attribution — 2026-11-01

The optimized diagnostic probe measured **2,375,276 bytes**. It did not establish
another large removable Auth component: directly named Auth/protocol functions
accounted for 5,667 bytes and Auth-shaped generic glue for 7,347 bytes in a
separate named companion. Inlining, LTO and shared implementations prevent
interpreting these figures as savings from removing Auth.

This is a historical source-specific observation, not current Canic adoption or
production admission evidence. No optimization, cryptographic verification or
TEE/non-TEE trust change resulted from this study.

## Inputs and reproduction boundary

- Canic source: `e753785953e6805461ac337868c32163f65c3209` (0.114.0), plus
  a captured working patch with SHA-256
  `58e3399bad05febe573c8530d826b85f165833469ff0507e372733031c4e57df`.
- Application: `canisters/audit/local_authorization_probe`, role `app`, network
  `local`, using its `canic.toml` and Host `build_artifact` example. The normal
  Canic checkout remained read-only; the study used isolated files under Auth's
  `target/`. This is not a recommendation to create another checkout.
- Dependencies: Auth/Protocol 0.28.0, Platform 0.15.3, Types 0.15.1, Wire 0.13.2,
  Memory/Storage 0.60.4 and Metrics 0.24.3. These are retired measurement inputs.
- Compiler: Rust 1.99.1, Linux x86_64, `opt-level=z`, full LTO, one codegen unit,
  aborting panic, no debug/assertions/overflow checks/incremental compilation.
- Build route: Host's supported `fast` path with those release-matching settings,
  followed by explicit admitted Binaryen 133 `wasm-opt -Oz`; Twiggy 0.8.0 for
  shallow symbol inspection. Canonical Release was blocked by the executable
  selection defect recorded in [Canic #705](https://github.com/dragginzgame/canic/issues/705).
  This substitute route was diagnostic, not canonical Release qualification.

## Selected results and limitations

| Complete uncompressed module | Bytes |
| --- | ---: |
| Raw runtime with names | 7,491,252 |
| Host fast output, stripped, before Binaryen | 2,696,617 |
| Explicitly optimized, stripped | 2,375,276 |
| Explicitly optimized, named companion | 3,287,956 |

Code-section lengths were **not measured**. Twiggy's aggregate of 2,156,804 bytes
across 5,058 function bodies excludes section framing and is not a code-section
measurement. Future comparisons must report both module and code-section sizes.
The 321,341-byte reduction from the optimizer is the normal optimization step,
not a new Auth saving.

The named and stripped companions did not become byte-identical after both were
stripped. Their symbols support investigation, not exact attribution of release
bytes. Large Platform entrypoint bodies contain inlined Canic handlers; they
are not evidence that removing Platform wrappers would save those bytes.
The historical classifier's non-code attribution defect is recorded in
[Canic #724](https://github.com/dragginzgame/canic/issues/724); its ownership totals
were excluded from these selected results. No runtime instruction, latency or
successful production receiver benchmark was established.

The original local scratch path, `target/canic-wasm-attribution.LEJv31/`, is no
longer present. Its patch, raw outputs and artifacts cannot be independently
verified from this summary or their recorded hashes. The longer narrative remains
[in Git history](https://github.com/dragginzgame/ic-auth/blob/a11f7558bd2187c06fedc739c7a0c36b470ba9ca/docs/wasm-attribution.md).
Current integration obligations belong to
[Canic #491](https://github.com/dragginzgame/canic/issues/491), not this report.
