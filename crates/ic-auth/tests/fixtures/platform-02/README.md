# Pre-owned-Principal compatibility vectors

These exact signed claims and session/replay/login records were emitted from
released Auth 0.10.7 (`debe7f51e48415daa336a65ae168f335c01ccab0`) and its
locked Platform 0.2.1. The existing `sessions.rs` `Fixture::new(1)` admits once
at `NOW`; its subject is the one-byte identity `02`, not a Candid marker.

The migration test compares both new encoding and checked record decoding with
these frozen bytes. It does not reinterpret a persisted profile or authorize a
storage migration. Capture evidence is retained under
`target/platform03-shared0253-20261020/legacy-source/` and `legacy-vectors.log`.
