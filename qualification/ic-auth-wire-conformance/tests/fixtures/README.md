# Frozen Wire resource thresholds

`resource-thresholds.tsv` retains 296 native admission thresholds: 37 reference
frames × four allowances × encode/decode. Columns are frame name, allowance
index (`0` bytes, `1` allocation, `2` nodes, `3` depth), decode flag, and the
minimum accepted allowance with the other three held at production values.
Each case asserts refusal immediately below, acceptance at, and acceptance above
that fixed value. Encode and decode accounting are kept separate.

Captured before removing Wire 0.22.2, using the binary-search oracle in
`qualification/ic-auth-wire-conformance/tests/codecs.rs` from this repository's
v0.2.0 commit `821eaab45b21a3977cadf4c3339df0643e1974ca`. The capture added only
frame names and output of each computed minimum; the old/new comparison still
passed. Toolchain: Rust 1.100.0, native x86_64 Linux. That oracle used the fixed
browser frames under `crates/ic-auth-wire/tests/fixtures/` and the current Auth
DTOs, without the old Auth engine. Frame provenance remains with those files.

Reference supplier: crates.io `ic-wire` 0.22.2, selected by the pinned original
Auth Cargo.lock; archive checksum
`be2bdd6a6d430182fa2ce49ea7e53461ea20d4790c5f299f6b0d04860f8bbd84`.
Captured TSV SHA-256:
`12e0ff89fa1278a5fb46974ca3c6720da5b90f4bbedf0825ebb35a9f99239c7c`.

Qualification now uses published IC Wire 0.24.1 through its current API. The old
crate is absent from the lock and dependency graph. Do not replace these values
with measurements of the codec under test: intentional accounting changes need
explicit review of the current resource contract and independent boundary evidence.
These retained expectations do not require historical accounting parity forever.
This corpus is not an exhaustive current-contract proof or a host cost benchmark.
