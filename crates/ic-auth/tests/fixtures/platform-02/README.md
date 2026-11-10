# Pre-owned-Principal vectors

These exact signed claims and session/replay/login records were emitted from
released Auth 0.10.7 (`debe7f51e48415daa336a65ae168f335c01ccab0`) and its
locked Platform 0.2.1. The existing `sessions.rs` `Fixture::new(1)` admits once
at `NOW`; its subject is the one-byte identity `02`, not a Candid marker.

The signed-claims vector is now a schema-1 rejection fixture: pending Auth 0.38
adds signed device slots/generations in schema 2 and explicitly retires schema 1.
Auth 0.22 retired the CBOR storage records after explicit confirmation that no
deployed or retained Auth records required preservation. All these vectors now
prove refusal of retired formats, not restoration compatibility. No fallback reader
or implicit reset is supplied. Historical capture evidence was recorded under
`target/platform03-shared0253-20261020/`.
