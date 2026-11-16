# Installed Auth cost observations

This focused lane measures the existing synchronous receiver fixture on PocketIC
16.1.0, using independent signed Ed25519 vectors. It is a baseline observation,
not an accepted production budget, maximum-load claim or cycle-price estimate.
Product crates and product Wire contracts are unchanged.

Run with a prepared PocketIC server and the normal prepared build tools:

```sh
POCKET_IC_SERVER_URL=http://127.0.0.1:PORT \
  HOST_TEST_FILTER='cost_observations --nocapture' make qualify-host
```

The ignored `installed::costs::receiver_cost_observations` test uses the shared
owned fixture builder, installs fresh state and enrolls through the real authority
call. It then executes one ordered scenario: bad proof, initial admission, exact
retry, protected write, logout and revoked authorization. Every result is checked;
retry must retain the first lease, response size is independently re-encoded by
the native harness, and final state must contain one committed application write,
no session and one retained replay fence. The harness prints the built Wasm digest.
The preparation/build and measurement path are explicit; ordinary native tests do
not start a server or run these observations.

## Measurement boundary

The qualification-only `receiver_fixture_measure` endpoint wraps the existing
control function. Runtime's opt-in `system-raw` instruction counter zero measures
the synchronous function body, including fixture policy/host projection and Auth
work. Request decoding, outer endpoint/executor overhead, measurement encoding and
ordinary response encoding are outside the instruction interval. Enrollment is
rejected by this endpoint because its cross-message await would invalidate the
counter comparison. No replacement executor, counter implementation or product
instrumentation is introduced.

Heap values are total linear-memory pages before/after the operation, including
fixture/runtime/static memory and allocator reservation. They are not live Auth
allocations; unchanged pages do not imply zero allocation. Each page is 65,536
bytes. Response bytes describe the ordinary fixture control result, excluding the
measurement envelope, ingress signatures and certificates. Request bytes include
the measurement Wire frame. These are fixture sizes, not product endpoint budgets.

## Observed baseline, 2026-11-16

Source baseline: `6d5c970`, plus the local qualification measurement changes.
Rust 1.100.0, release-profile Wasm, PocketIC 16.1.0 NNS subnet; Platform 0.34.9,
Types 0.21.2, Runtime 0.23.14, Wire 0.24.3 and Testkit 0.107.4. The Platform and
Testkit lock updates were separately present in the working tree; these measurements
do not qualify all supplier behavior. The receiver baseline was remeasured with
the issuer lane in the same fixture. Fixture Wasm: 707,762 bytes, SHA-256
`881850a834562aa4a074313b143f1cfd4ae7aab1c60403f7a57eaa3cb1dbdb8a`.

| Operation | Instructions | Heap pages before → after | Request bytes | Response bytes |
| --- | ---: | --- | ---: | ---: |
| Bad Ed25519 proof | 415,905 | 18 → 18 | 330 | 13 |
| First admission | 2,558,769 | 18 → 18 | 330 | 23 |
| Exact admission retry | 246,280 | 18 → 18 | 330 | 23 |
| Protected authorization/write | 19,205 | 18 → 18 | 13 | 13 |
| Known-login logout | 6,088 | 18 → 18 | 13 | 22 |
| Revoked authorization | 16,472 | 18 → 18 | 13 | 13 |

## Certified issuer control observations

The `issuer_cost_observations` filter installs fresh issuer state and measures
six existing synchronous control operations through `auth_fixture_measure`.
Preparation includes fixed fixture entropy and approval. Certified completion
includes the fixture's signature-tree leaf mutation, hashing and system root
publication; it does not include fetching a certificate or verifying BLS proof.
Observation queries outside the interval bind the root to an actual certificate
and check retained claims, quota reservations and work credit. The existing
installed signature verification lane remains separate.

Both completed retries inject a root-publication trap: success proves that the
publication callback was not invoked again. The second occurs after account
revocation and must retain the exact claims, root, usage and credit. This is
retrieval of existing evidence, not permission to finish pending issuance or renew.
Wrong-account completion must refuse without changing state.

| Issuer operation | Instructions | Heap pages before → after | Request bytes | Response bytes |
| --- | ---: | --- | ---: | ---: |
| Prepare approved intent | 48,618 | 18 → 18 | 12 | 12 |
| Wrong-account completion | 2,154 | 18 → 18 | 13 | 13 |
| Certified completion/root publication | 90,433 | 18 → 18 | 13 | 12 |
| Completed retry | 11,662 | 18 → 18 | 13 | 12 |
| Account revocation | 3,196 | 18 → 18 | 12 | 12 |
| Retained retry after revocation | 11,662 | 18 → 18 | 13 | 12 |

## Installed refusal limits

The `limit_cost_observations` filter uses the same artifact and dependency cohort.
Five independently invalid proofs consume the receiver's ten work credits. The
unchanged valid proof then refuses without allocating sessions, replays or login
rows. After one full second from the actual simulator time, the single replenished
credit remains unspent on refusal. After another second, the same credential
admits and consumes the two available credits. Absolute rounded timestamps are
not used as refill boundaries because PocketIC advances time between messages.
The fixture projects both exhausted budget and invalid signature as `Proof`;
credit/state observations and successful refill recovery distinguish this scenario.

A fresh issuer reserves/approves an operation and revokes its login before
completion. Completion must return `Denied`, keep pending evidence, claims,
reservations, credit and root unchanged, and never enter the root-publication
callback (which would trap). This differs from returning an already completed
result after revocation, measured above.

| Refusal | Instructions | Heap pages before → after | Request bytes | Response bytes |
| --- | ---: | --- | ---: | ---: |
| Valid proof with exhausted work credit | 266,868 | 18 → 18 | 330 | 13 |
| Revoked pending issuer completion | 11,143 | 18 → 18 | 13 | 13 |

## Known-login capacity refusal

The `capacity_cost_observations` filter fills the fixture's one-known-login
limit. A second independently signed device credential succeeds on an otherwise
identical empty receiver, proving that its refusal on the full receiver is not
caused by invalid proof or caller binding. The full receiver retains all usage,
login counts and work credit. The original credential still retries with its exact
lease and authorizes a write; the refused caller cannot authorize. After original
logout releases the session, the retained login/replay fences still consume their
capacity and the second device remains refused without eviction.

| Refusal | Instructions | Heap pages before → after | Request bytes | Response bytes |
| --- | ---: | --- | ---: | ---: |
| Known-login capacity full | 238,634 | 18 → 18 | 330 | 13 |

This uses the same artifact and cohort as above. The known-login limit is reached
before the separate session/replay/byte limits; this is not independent saturation
coverage for those later limits or a population-scaling benchmark.

## Replay-capacity refusal, 2026-11-17

The `replay_capacity_cost_observations` filter admits the initial and renewed
credentials for one login, occupying one session and both replay slots. At time 35,
a later independently valid proof succeeds on an empty receiver but refuses on the
full receiver without work charge or state changes. Expired proof windows do not
implicitly prune physically retained replay rows.

Exact retry of the latest admitted credential still returns its original session
lease after that credential's proof expires, with no work charge or replay eviction.
Ordinary authorization succeeds. Logout releases the session, retains both replay
rows and blocks that same retry. The global and per-account replay caps are both
two in this fixture; their independent saturation is not established here.

| Refusal | Instructions | Heap pages before → after | Request bytes | Response bytes |
| --- | ---: | --- | ---: | ---: |
| Replay slots full | 264,829 | 18 → 18 | 330 | 13 |

The Wasm artifact/digest and runtime cohort match the baseline above. The native
harness also used the separately present Host Process 0.80.3 update (previous
measurements used 0.80.2); this lane is not a general qualification of that update.

These are single-operation observations in the stated scenario. The fixture has
one session slot and two replay slots, not a representative production population.
There are no invented acceptance ceilings. Issuer renewal and witness delivery, certified-proof admission,
capacity scaling, production host overhead and agreed operating budgets remain
unmeasured here. Future comparisons must retain scenario, toolchain, supplier graph
and measurement boundary, and record the new artifact digest.

## Source-delivery revalidation

The 0.47.0 source-delivery check reran all ten installed cases (five lifecycle and
five measurement scenarios) with Platform 0.34.10, Types 0.21.2, Runtime 0.23.14,
Wire 0.24.3, Testkit 0.107.4 and Host Process 0.80.3. All sixteen instruction counts,
heap-page observations and frame sizes above matched. The rebuilt fixture was
707,902 bytes with SHA-256
`045a89995066851db2b88486e7b48be58d899dc5150c1784bfa137e4bd93e97b`.
The separate installed HTTP certificate and browser-journey tests also passed.
This records a new artifact rather than attributing the earlier digest to the
updated supplier graph; acceptance limits remain unchanged.
