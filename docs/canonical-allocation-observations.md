# Canonical hashing allocation observations

Current canonical hashing streams the shared field encoder; the older allocation
tables below describe earlier implementations, not the current allocation cost.
The latest matched footprint comparison is recorded at the end of this document.
An earlier streaming experiment was rejected after its Wasm/cost comparison;
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

## Signature retention and shared grants — 2026-11-08

Source: `291fbf1` plus the local cleanup batch. All three builds use the same
incoming Wire/derive 0.17.14 lock (SHA-256
`1f75b914f373c25e1231e2744b3ae2f75238f5a0638ba40c74dbb45e4aeea66d`),
Rust 1.99.1 (`58cf8f9c3`), target `wasm32-unknown-unknown`, Cargo's existing
release defaults and repository `--cfg=cpubits="32"`. There is no post-optimizer
or profile change. Command: `cargo build --locked --offline -p
ic-auth-qualification-canister --target wasm32-unknown-unknown --release`.

| Reference fixture | Total bytes | Code-section bytes |
| --- | ---: | ---: |
| Before this batch | 1,899,580 | 1,462,831 |
| Deadlines in existing certified-tree leaves; separate map removed | 1,885,348 | 1,448,864 |
| Shared runtime-selected grant encoder | 1,884,640 | 1,448,619 |

The retained changes save **14,940 total bytes and 14,212 code bytes** against
this matched baseline. Code size is the Wasm section-10 payload length, parsed
from the uncompressed module. Final artifact SHA-256:
`feac794568fe7a97d4bccdfc76ca239852d19d3a40953c0539a5732a32406e31`.
Raw artifacts, compiler identity, selected lock, source snapshots and build logs:
`target/evidence/signature-index-20261108/`.

Retention deadlines remain uncertified local metadata: the leaf still hashes
and serializes as the same empty certified value. The expiry index still owns
bounded cleanup and record counts. Exact-retry deadlines, expiry replacement,
cross-key retention, unchanged witness bytes and real BLS verification are
covered by focused tests. Grant admission and canonical bytes are unchanged.

This is the complete Auth qualification fixture, not Canic's deployed Wasm.
It establishes footprint reduction, not lower instruction counts or latency.
Expired re-preparation now updates its existing tree leaf, while live retries
still avoid tree mutation. No whole-verifier throughput claim is made. Browser
destination and journal cleanup are separate JavaScript/storage changes and do
not contribute to these Wasm deltas.

### Prepared key and receiver-state follow-up

The subsequent local batch preserves incoming native Host 0.75.1 and Wire 0.17.14
with lock SHA-256 `c6c4725ad731fb485422c1d7e5e834f21adcf24a6facbf832c3e6996bd5f5db8`.
The same compiler, target flags, release defaults and build command above apply;
no optimizer changed. Its fresh baseline reproduces the previous final artifact.

| Follow-up fixture | Total bytes | Code-section bytes |
| --- | ---: | ---: |
| Matched baseline | 1,884,640 | 1,448,619 |
| Key/private-branch cleanup; original receiver presence | 1,884,465 | 1,448,924 |
| Combined cleanup with inline receiver state (discarded) | 1,884,725 | 1,449,218 |
| Combined cleanup with boxed configured receiver (retained) | 1,884,402 | 1,449,460 |

This batch saves **238 total bytes but adds 841 code bytes**. It is retained for
single-owner receiver state and less duplicated key data, not a code-size win.
The private fixture allocates its receiver once per installation/restoration;
ordinary authorization introduces no new allocation from that representation.
Vector tooling is native-only. Public APIs, signature checks, signed bytes and
stored/checkpoint formats remain unchanged. Final artifact SHA-256:
`94aa95c3d28fdc3cdd023320574cd726c26e6b1f593191f5a5327ee87ffda44d`.
Raw inputs, alternative sources, measurements and checks are retained under
`target/evidence/prepared-state-20261108/`. Runtime speed was not measured.
