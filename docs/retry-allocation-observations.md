# Exact-retry allocation observations

Session admission still uses its existing decode-first exact-retry path. This
profile measures that cost without changing verification, refusal ordering or
the TEE/non-TEE trust model. It is separate from the rejected canonical streaming
experiment described in [canonical allocation observations](canonical-allocation-observations.md).

## Reproduction and scope

Run `bash scripts/dev/profile-retry-allocations.sh --iterations 100` after normal
explicit tool/dependency preparation. The wrapper runs a release-mode native
example with DHAT 0.3.3 and seals the source/lock/compiler inputs before and after
execution. Evidence, source/dirty identity, raw command output and JSON results
remain beneath `target/portable-fixtures/retry-allocations.*`.

Each fixture first completes a real Ed25519 admission into the volatile reference
store. It then submits exact retries after proof expiry, with a malformed
signature and exhausted verification credit. Every retry must return the original
receipt. Current session, authority, generation, request and revocation checks
still run. The comparison independently decodes the identical claims blob.

Fixture construction, initial signing/admission, owned request construction and
one warmup per operation are outside measurement. Both profiles request one scope;
the wide profile uses more grants with short labels, not maximum-length labels.
Every observed operation releases its temporary tracked heap before returning.
DHAT requested byte totals are distinct from physical allocator consumption.

## Observations — 2026-11-04

Rust 1.99.1, Linux x86_64, release mode, 100 calls per observation. Source is
`95e1f72` plus the pending local improvements; exact input digests and dirty state
are sealed in `target/portable-fixtures/retry-allocations.x76x4g/`.

| Profile | Roles / scopes per role | Claims bytes | Retry allocations/call | Retry requested bytes/call | Decode allocations/call | Decode requested bytes/call |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Small | 1 / 2 | 257 | 5 | 116 | 5 | 116 |
| Wide | 16 / 32 | 6,077 | 545 | 16,736 | 545 | 16,736 |

Peak simultaneously tracked bytes were 116 and 16,736 respectively for both
operations. In these fixtures the complete retry and standalone claims decode
have identical allocation totals. This identifies decoding as the allocation
source; it does not measure its fraction of CPU or instruction cost.

## Decision and limits

Keep retry behavior unchanged in this batch. Wide-input decoding is a concrete
candidate for a later measured experiment, but native allocation counts alone
do not establish a practical production problem or justify a new admission path.
Small credentials already use little scratch. No latency, Wasm instructions,
module/code-section size, deployed occupancy or production retry frequency was
measured here.

Any fingerprint-first experiment must preserve current refusals and their
ordering, exact request identity, live authority/generation/revocation checks,
original leases and proof-budget semantics. It must retain one owner for fresh
claims decoding and must not become a verification-result cache. Qualify matched
host cost and Wasm size before adopting an optimization.
