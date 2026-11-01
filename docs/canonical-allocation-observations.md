# Canonical hashing allocation observations

The current encoder remains unchanged. These observations justify keeping the
existing implementation until an actual host profile demonstrates a material
cost. An earlier streaming experiment was rejected after its Wasm/cost comparison;
see the [Auth #113 experiment record](https://github.com/dragginzgame/ic-auth/issues/113#issuecomment-6503797232).

## Reproduction and scope

Run `make profile-canonical-allocations` after ordinary explicit dependency/tool
preparation. The native example uses DHAT 0.3.3 in release mode. DHAT is a dev
dependency of unpublished tooling only; its allocator never enters production
Auth or the canister fixture. The example refuses debug mode, accepts a nonzero
`--iterations` argument and defaults to 100 calls per observation.

Inputs are constructed before measurement, with one unmeasured warmup per
operation. Each profiled call must return the same digest and release all its
temporary heap before returning. Fixture construction, reporting and profiler
setup are excluded. Small inputs have one role/two scopes; wide inputs have
16 roles/32 scopes each. Both synthetic root proofs have four witness steps.
They are canonical hashing inputs, not cryptographically valid credentials.

The wrapper retains the compiler identity, source revision/dirty state, exact
source/lock hashes before and after execution, command log and JSON results in
the printed `target/portable-fixtures/canonical-allocations.*` directory.

## Observed native allocations — 2026-11-01

Source: released Auth 0.29.0 `6c0411b` plus the uncommitted observation example.
Compiler: Rust 1.99.1, Linux x86_64, release mode; 100 calls per row.
Evidence: `target/portable-fixtures/canonical-allocations.uJihop/`.

| Operation | Small blocks/call | Small bytes/call | Small peak bytes | Wide blocks/call | Wide bytes/call | Wide peak bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Certificate hash | 2 | 384 | 256 | 7 | 16,256 | 8,192 |
| Complete proof hash | 4 | 1,920 | 1,024 | 8 | 32,640 | 16,384 |
| Batch header hash | 1 | 256 | 256 | 1 | 256 | 256 |
| Issuer leaf hash | 1 | 256 | 256 | 6 | 16,128 | 8,192 |
| Derivation path hash | 2 | 150 | 100 | 2 | 150 | 100 |

DHAT totals include allocation/reallocation accounting; cumulative requested
bytes are distinct from peak simultaneous live scratch memory. These values
depend on the native allocator, compiler and exact inputs. The instrumented run
does not measure latency, Wasm size/instructions, full proof verification,
production occupancy or login frequency. It does not establish the fraction of
admission cost attributable to canonical hashing or BLS.

No optimization, signed-byte change, verification-result cache or trust-model
change follows from this measurement. Any future buffer experiment needs frozen
digest vectors, matched installed-host measurements and Wasm-size comparison
before adoption.
