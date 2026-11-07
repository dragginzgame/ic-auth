# Canonical hashing allocation observations

The current encoder remains unchanged. These observations justify keeping the
existing implementation until an actual host profile demonstrates a material
cost. An earlier streaming experiment was rejected after its Wasm/cost comparison;
see the [Auth #113 experiment record](https://github.com/dragginzgame/ic-auth/issues/113#issuecomment-6503797232).

## Reproduction and scope

Run `make profile-canonical-allocations` after ordinary explicit dependency/tool
preparation. The harness-free benchmark under `apps/tooling/benches/` uses
DHAT 0.3.3 in Cargo's optimized bench profile. DHAT is a dev
dependency of unpublished tooling only; its allocator never enters production
Auth or the canister fixture. The benchmark refuses debug mode, accepts a nonzero
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
The canonical and retry commands share the capture driver and DHAT observation
helper. Both files are included in their input seals; report fields are unchanged.

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

## Portable claims and current reference Wasm — 2026-11-07

Source: Auth `0bbb351` plus the pending IC Build adoption, backend qualification
and profiler changes. Rust 1.99.1, Linux x86_64, optimized native bench profile,
100 calls per row; DHAT 0.3.3. Evidence:
`target/portable-fixtures/canonical-allocations.AQor3L/`. The five older delegation
profiles reproduce the allocation counts/bytes in the table above. The added
portable inputs use the same small/wide grant counts, with shape-valid synthetic
claims rather than authenticated issuer material.

| Portable claims hash | Blocks/call | Cumulative bytes/call | Peak scratch bytes |
| --- | ---: | ---: | ---: |
| Small | 5 | 868 | 448 |
| Wide | 9 | 14,308 | 7,168 |

These figures expose allocation/reallocation in the temporary encoded blob;
they do not establish a material production cost or justify a new writer/sizing
abstraction. The earlier rejected streaming experiment remains relevant.
Production encoders and hash implementations were left unchanged.

A separate fresh build uses the existing qualification canister's complete
capability graph, including BLS fixture signing, the published Runtime/CDK and
Storage host. It is not Canic's production artifact or a minimal session-only
consumer. Command: `cargo build --locked --offline --release --target
wasm32-unknown-unknown -p ic-auth-qualification-canister`, with the repository's
existing profile/target flags and no optimizer change. Raw output is 1,903,528
bytes, including a 362,294-byte function-name subsection. This is a baseline,
not a before/after size improvement or regression.

Twiggy 0.8.0 reports one retained instantiation of `establish_session` (5,982
shallow bytes) and one of `authorize_session` (1,485 shallow bytes). Neither shows
duplicate retained instantiations in this fixture. Shallow symbol sizes exclude
shared callees and code inlined elsewhere; they cannot establish Canic's generic
fan-out or predict savings from outlining helpers. No generic-code refactor was
adopted from this observation.

Artifact, profiler versions/hashes, compiler, source/lock seals, raw symbol data
and build logs remain under `target/evidence/auth-next-035/`. Reproduce attribution
with `twiggy top` and `twiggy monos -a --regex <artifact>
'.*(establish_session|authorize_session).*'`. Future comparisons must keep source
scope, selected dependencies, features, compiler and optimization settings matched.
