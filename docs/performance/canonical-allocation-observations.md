# Canonical hashing allocation observations

Current canonical hashing streams the shared field encoder; the older allocation
tables below describe earlier implementations, not the current allocation cost.
The latest matched footprint comparison is recorded at the end of this document.
An earlier streaming experiment was rejected after its Wasm/cost comparison;
see the [Auth #113 experiment record](https://github.com/dragginzgame/ic-auth/issues/113#issuecomment-6503797232).

## Reproduction and scope

Run `make profile-canonical-allocations` after ordinary explicit dependency/tool
preparation. The harness-free benchmark under `crates/ic-auth-tooling/benches/` uses
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

## Grant evidence and rejected expiry queue — 2026-11-08

Matched reference-canister builds use `6989a91` plus the pending private
Build 0.80 / Host 0.76 / Testkit 0.94 adoption, with Platform 0.27 and Storage 0.69.
Rust is `1.99.1 (58cf8f9c3)`. Each build uses the unchanged Cargo release profile,
`wasm32-unknown-unknown` and `.cargo/config.toml`'s `cpubits="32"`; there is no
post-link optimizer. Command:
`cargo build --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown --release`.
The baseline lock and modules, build logs, measurement script and retained patch
are under `target/complexity-cleanup/`. The only lock change during the experiment
removes three unused direct dependencies from the private feature-unification
harness; selected dependency versions and the canister graph are unchanged.

| Candidate | Total module bytes | Code-section payload bytes | Total delta | Code delta |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 1,869,714 | 1,438,143 | — | — |
| Single typed session/replay expiry queue, rejected | 1,881,623 | 1,448,144 | +11,909 | +10,001 |
| Retained grant-validation evidence | 1,869,146 | 1,437,689 | -568 | -454 |

Code size is the unsigned-LEB128 payload length of section 10 in the complete
uncompressed module. The rejected queue was reverted; the two original indexes
retain their pruning merge and atomic commit behavior. Grant encoding instead
uses a borrowed checked value, one field writer and the existing standalone
canonical checks. Independent signed-byte vectors and proof/session rejection
suites pass. No runtime speedup or production Canic footprint is inferred.

The proposed state-specific login retention record was also left unchanged:
preserving its frozen stored representation would require another codec model,
adding machinery for a small internal redundancy. Browser unknown issuance was
narrowed through types rather than derived from sequence presence, because the
current profile permits durable uncertainty without a recoverable sequence.

## Wire 0.18 and Storage 0.70 adoption — 2026-11-08

Matched builds use `6989a91` plus the pending cleanup, private-package relocation
and upstream/tooling adoptions. Both use Rust 1.99.1, the unchanged Cargo release
profile, `wasm32-unknown-unknown` and repository `cpubits="32"`; there is no
post-link optimizer. Command:
`cargo build --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown --release`.
The compared dependency change is Wire/derive 0.17.14 to 0.18.0 together with
Storage 0.69.1 to 0.70.0. Other selected versions and Auth Wasm source are fixed.
The native-only Testkit budget projection is absent from both Wasm graphs.

| Reference fixture | Total module bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Before Wire/Storage adoption | 1,869,187 | 1,437,689 |
| Wire 0.18.0 and Storage 0.70.0 | 1,870,184 | 1,437,987 |
| Change | +997 | +298 |

Code size is the unsigned-LEB128 payload length of section 10 in the complete
uncompressed module. Before SHA-256:
`c9f0e2e97a8f100580f6323e9e114f72652457bd8cd9c3e08d4d06fc95ac8624`;
after SHA-256:
`10ee087b8f84482d7be8fc3d82ffe143c273eb324ab6c223d35aa6b11addd384`.
Manifests, locks, modules, build logs and measurements are retained under
`target/wire-adoption/`. This is a combined dependency-adoption measurement,
not attribution to either upstream alone, a runtime performance result or
Canic's deployed footprint. Generated browser bytes and stored profiles remain
unchanged; focused Rust/client and installed certification/session/ingress
qualification pass.

The later incoming Runtime/macros/guard 0.20.3 selection is recorded separately:
1,869,903 total bytes and the same 1,437,987 code bytes, SHA-256
`47fa1806e4531efa0465c835d4291c0b020e86e059e8542cc2dcf93afd4cfe20`.
Its lock, module and measurements have the `incoming-runtime` prefix in the same
evidence directory. Both sides of the matched table above use Runtime 0.20.2;
this later selection is not mixed into that comparison.

## Reference-store replacement — 2026-11-09

Session and login replacement now use the old value returned by `BTreeMap::insert`
to remove the previous expiry/pool metadata. Deletion keeps the same accounting;
all publication remains after the host's successful commit callback. No record,
codec, proof, trust or cleanup-order contract changes.

The baseline uses `e22cebd`'s public Engine code plus the pending Storage 0.71
adoption. It is the module built during that adoption, copied before these store
edits. Both sides select the identical Cargo.lock, Storage 0.71.0, Rust 1.99.1,
existing Cargo release profile, `wasm32-unknown-unknown` and repository
`cpubits="32"`; there is no post-link optimizer. The only deployed-source changes
in this comparison are the two reference-store implementations. Build command:
`cargo build --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown --release`.

| Reference fixture | Total module bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Before replacement cleanup | 1,831,907 | 1,411,604 |
| After replacement cleanup | 1,822,844 | 1,404,357 |
| Change | -9,063 | -7,247 |

Code size is the unsigned-LEB128 payload length of section 10 in the complete
uncompressed module. Before SHA-256:
`b1fe288b519ea6677ecec643fade08b129e3676a071699da9dc2ff1ddaa06482`;
after SHA-256:
`32367cea91ddddc5072ca1164b3622e80080c85430f29fbc103b28de5d1a8def`.
The baseline build log is `target/storage-071/wasm-build.log`. Copied modules,
original store sources, the selected lock/compiler, new build logs and
`wasm-comparison.json` are retained under `target/store-cleanup/`.
This measures the reference fixture's footprint, not an installed Canic binary
or a runtime speedup. Storage adoption is present on both sides and receives
none of this size attribution.

## Signature-store seed lookup — 2026-11-09

Signature preparation now retains the seed-presence result from its deadline
lookup, rather than searching the outer tree again before insertion. Validation,
capacity, retention, certified leaf bytes and root publication are unchanged.

The baseline is the retained reference-store cleanup above. Both modules use its
same sealed Storage 0.71 lock, Rust 1.99.1, release profile, Wasm target and
`cpubits="32"`, without a post-link optimizer. Only `signature_store.rs` differs
in the final deployed source. The build command remains the one above.

| Reference fixture | Total module bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Before seed-lookup reuse | 1,822,844 | 1,404,357 |
| After seed-lookup reuse | 1,822,484 | 1,404,136 |
| Change | -360 | -221 |

Code size remains the unsigned-LEB128 section-10 payload length. Before SHA-256:
`32367cea91ddddc5072ca1164b3622e80080c85430f29fbc103b28de5d1a8def`;
after SHA-256:
`b8df0c12a2c9e42a9c8e4d8c707debbcb690e44e9472055f544da734890d3c85`.
Modules, source backups, unchanged lock, build/test logs and comparison JSON
remain under `target/signature-cleanup/`. No runtime speedup or Canic footprint
is inferred.

A combined trial also replaced the qualification host's temporary byte vectors
with borrowed hash-tree leaves. It produced 1,826,429 total / 1,406,558 code bytes,
growing 3,585 total / 2,201 code bytes against this baseline. That host change was
reverted, preserving its original source bytes. The rejected module and logs
remain separately named in the evidence directory; it is not the final result.

## Borrowed login predicates — 2026-11-09

Private login-record queries now borrow their record; pool membership inspects
the enum directly instead of extracting a full binding to test its presence.
The private session-deletion helper also drops its unused boolean result. Public
interfaces, serialized records, admission/denial policy and transaction ordering
remain unchanged.

The baseline is the retained seed-lookup result above. Both sides use the same
sealed lock, Storage 0.71, Rust 1.99.1, Cargo release profile, Wasm target and
`cpubits="32"`, without a post-link optimizer. Only the session-memory and login
revocation implementations differ in deployed source. The build command remains
the one above.

| Reference fixture | Total module bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Before borrowed predicates/deletion cleanup | 1,822,484 | 1,404,136 |
| After cleanup | 1,820,985 | 1,402,758 |
| Change | -1,499 | -1,378 |

Code size is the unsigned-LEB128 payload length of section 10. Before SHA-256:
`b8df0c12a2c9e42a9c8e4d8c707debbcb690e44e9472055f544da734890d3c85`;
after SHA-256:
`4a06259db848f67c6e31ec81a2fbf724f212a6e4f054c846b13f9654ae94e6d8`.
Modules, original source copies, the sealed lock, build/test logs and comparison
JSON remain under `target/record-cleanup/`. This is reference-fixture size
evidence; no runtime speedup or Canic footprint is inferred.

## Narrow pool-capacity inputs — 2026-11-09

The private capacity helper receives the requested pool rather than a complete
login record. Logout borrows the retained record while reading its deadline and
binding. No capacity, clock, budget, retention or callback check changes.

The baseline is the borrowed-predicate result above. Both modules use its same
sealed Storage 0.71 lock, Rust 1.99.1, release profile, Wasm target and
`cpubits="32"`, without a post-link optimizer. Only `portable/revocation.rs`
differs in deployed source; the build command remains the one above.

| Reference fixture | Total module bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Before narrower inputs | 1,820,985 | 1,402,758 |
| After narrower inputs | 1,821,026 | 1,402,675 |
| Change | +41 | -83 |

Code size is the unsigned-LEB128 section-10 payload length. Before SHA-256:
`4a06259db848f67c6e31ec81a2fbf724f212a6e4f054c846b13f9654ae94e6d8`;
after SHA-256:
`e11ceca931b79b2707ec2bca410e0e39feb4f38683cb03958c192d8f403e8e19`.
Modules, original source, the sealed lock, build/test logs and comparison JSON
remain under `target/pool-cleanup/`. The size effect is mixed and small; this is
retained for narrower inputs and borrowing, without a runtime speedup or Canic
footprint claim.

## Borrowed pruning candidates — 2026-11-09

The two ordered expiry cursors keep their candidates borrowed while comparing
deadlines. Only the selected caller or fingerprint is copied into the staged
deletion. The separate indexes, session-first tie ordering, 128-record cap and
commit-before-mutation contract remain unchanged.

The baseline is the narrow-capacity result above. Both modules use its identical
sealed Storage 0.71 lock, Rust 1.99.1, release profile, Wasm target and
`cpubits="32"`, without a post-link optimizer. Only `session/memory.rs` differs
in deployed source; the build command remains the one above.

| Reference fixture | Total module bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Before borrowed candidates | 1,821,026 | 1,402,675 |
| After borrowed candidates | 1,821,027 | 1,402,675 |
| Change | +1 | 0 |

Code size is the unsigned-LEB128 section-10 payload length. Every non-custom
section is byte-identical; the only difference is the `name` custom section.
This is a source simplification with no executable size or speedup claim.
Before SHA-256:
`e11ceca931b79b2707ec2bca410e0e39feb4f38683cb03958c192d8f403e8e19`;
after SHA-256:
`3d2f2c443d409992ca002bcd13d5b09526e961596247d90393d3c2173af2fe06`.
Modules, original source, the sealed lock, build/test logs, comparison JSON and
`section-equality.json` remain under `target/prune-cleanup/`.

A further replay-counter map-entry trial grew the module to 1,822,380 total /
1,403,885 code bytes: +1,354 total / +1,210 code bytes against the same baseline.
That trial was reverted; its separately named module, measurements and passing
focused tests remain retained. The final rebuild matches the original borrowed-
candidate module exactly.

## Direct SHA-2 0.11 adoption — 2026-11-09

Auth's root direct requirement changes from `sha2` 0.10 to 0.11.1, already
selected by Platform's RustCrypto/Principal helpers. Upstream-only older SHA-2
versions remain; no cryptographic body, canonical input, domain or signature
policy changes. Two native artifact diagnostics project the new digest array to
the existing hexadecimal display type; deployed Rust source is unchanged.

Both reference modules use Rust 1.99.1, `wasm32-unknown-unknown`, the existing
release profile and `cpubits="32"`, with no post-link optimizer. Both run
`cargo build --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown --release`.
The old lock selects Tooling 0.37.2, the new lock 0.37.4; Tooling is a dev-only
selection and its installed executables/build settings are unchanged. The only
normal-graph change is the direct SHA-2 selection. No unrelated package version
changes. The two native diagnostic edits are excluded from this Wasm.

| Reference fixture | Total module bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Direct SHA-2 0.10.9 | 1,821,027 | 1,402,675 |
| Direct SHA-2 0.11.1 | 1,823,778 | 1,405,535 |
| Change | +2,751 | +2,860 |

Code size is the unsigned-LEB128 section-10 payload length. This is a dependency
freshness update with a small measured footprint increase, not an optimization
or runtime speedup claim. Before module SHA-256:
`3d2f2c443d409992ca002bcd13d5b09526e961596247d90393d3c2173af2fe06`;
after: `aba7ae48b250ec67a43a64ef5a8af1443c73abb71156d4f99b69eee20ba181b3`.
Sealed lock copies/identities, modules, comparison JSON, focused test/build/lint
logs and installed qualification output remain in `target/tooling-0374-sha2/`.
The existing frozen canonical, Merkle, request, portable-claims and independent
signature vectors pass. These measurements do not establish Canic adoption or
its deployed size.
