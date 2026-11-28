# Frozen Wire resource thresholds

`resource-thresholds.tsv` retains 296 native admission thresholds: 37 reference
frames × four allowances × encode/decode. Columns are frame name, allowance
index (`0` bytes, `1` allocation, `2` nodes, `3` depth), decode flag, and the
minimum accepted allowance with the other three held at production values.
Each case asserts refusal immediately below, acceptance at, and acceptance above
that fixed value. Encode and decode accounting are kept separate.

The sparse-format thresholds were measured on native x86_64 Linux with Rust 1.100.0 and the published IC Wire 0.33.0. For each independently assembled frame, a binary search varied a single supplier allowance from zero to its production ceiling. The checked-in test reads the frozen observations and checks both sides of each boundary. The previous positional measurements are recorded in the v0.53.0 source.
