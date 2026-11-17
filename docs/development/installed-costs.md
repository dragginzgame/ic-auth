# Installed Auth cost observations

These focused lanes measure synchronous receiver/issuer operations and query
witness assembly on PocketIC 16.1.0, using independent signed inputs. It is a baseline observation,
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

## Reading the evidence

The sections below retain observations from distinct fixture artifacts and supplier
cohorts; they are not one comparable performance series. The
[latest complete run](#mixed-expiry-and-maximum-scope-dimensions-2026-11-17) covers
nineteen installed tests and sixty-three observations. Earlier tables preserve
provenance, not additional acceptance gates. Shared native receiver setup and
measurement transport live in `installed/receiver_support.rs`; each scenario owns
its inputs and semantic assertions. The shared transport checks nonzero work,
heap-page monotonicity and independently re-encoded response size for both success
and refusal results; it does not supply the expected Auth outcome.

Further measurements need a concrete consumer workload or failure hypothesis.
Production host overhead and agreed operating budgets are the next acceptance
boundary; a larger synthetic matrix alone does not establish readiness.

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
There are no invented acceptance ceilings. The later observations below add
issuer renewal, witness assembly and certified-proof admission. Capacity scaling,
production host overhead and agreed operating budgets remain unmeasured. Future comparisons must retain scenario, toolchain, supplier graph
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


## Renewal, witness and certified admission, 2026-11-17

Source: `a05b5dd` plus the local qualification changes described here. Platform
0.34.11, Types 0.21.2, Runtime 0.23.15, Wire 0.24.3, Testkit 0.107.4 and Host Process
0.80.3; Rust 1.100.0, release Wasm, PocketIC 16.1.0 NNS subnet. The fixture now
includes the complete certified verifier and two issuer operation reservations.
Its 1,496,713-byte Wasm has SHA-256
`a3b1fa3c7158fdbf869bb51e4d673519202db3b80d04e7e28238085c51fa59ed`.
This is a larger qualification artifact, not a product Wasm size comparison.

`renewal_witness_cost_observations` first completes an initial certified issuance.
A wrong browser cannot reserve renewal or change claims/root/quota/credit. The
actual retained browser then reserves and approves sequence one, and the fixture
account completes it. The renewed claims must retain the original login start and
absolute deadline. Completed retry uses a root-publication trap as a negative
control and preserves the exact result, accounting and root.

The query-only `auth_fixture_witness_measure` interval includes acquiring/copying
the actual IC data certificate, reconstructing the fixture signature tree, reading
retained claims and encoding the canister-signature proof. It excludes outer Wire
encoding, network delivery and the replica's certificate generation. Its ordinary
response is the full fixture observation, not a product credential. Native checks
verify the returned signature against PocketIC's root. A later query obtains a
fresh certificate over unchanged claims/root without extending the login window.
All three observations contain a 433-byte certificate and 538-byte signature proof.

| Operation | Instructions | Heap pages before → after | Request bytes | Response bytes |
| --- | ---: | --- | ---: | ---: |
| Initial witness query | 49,533 | 18 → 18 | 11 | 1,293 |
| Wrong-browser renewal | 17,006 | 18 → 18 | 12 | 13 |
| Prepare approved renewal | 79,739 | 18 → 18 | 12 | 12 |
| Certified renewal completion | 90,943 | 18 → 18 | 13 | 12 |
| Completed renewal retry | 11,963 | 18 → 18 | 13 | 12 |
| Renewed witness query | 49,676 | 18 → 18 | 11 | 1,293 |
| Later certificate/witness query | 49,676 | 18 → 18 | 11 | 1,293 |

`certified_receiver_cost_observations` installs an isolated receiver with the test
network BLS key supplied at installation, separately from ingress proof bytes.
Its protected root ECDSA key, audience, grants and authority windows come from
fixed fixture configuration. Independent Node/OpenSSL records supply the canonical
claims and root signature, using the established fixture generator's `--certified
--installed` mode. The native harness uses Platform's qualification-only BLS signer
to construct the network certificate. Signing is outside the Wasm interval; the
Wasm fixture contains no BLS fixture signer.

The measurement includes fixed proof-material reconstruction, complete root ECDSA
and issuer BLS verification on valid fresh admission, and receiver bookkeeping.
Requests carry only candidate certificate/witness bytes; fixed claims/delegation
are reconstructed inside the fixture. These request sizes therefore do not describe
a full product credential or measure product Wire decoding. The ordinary response
is the measured outcome encoded with this fixture's method identity.

The receiver's initially empty bucket refills before the scenario. A ten-second
refill interval and admission at fixture time 155 seconds isolate the two proof
attempts from refill boundaries. The wrong-message proof consumes three credits
without installing state; the valid proof consumes three more. Exact retry,
authorization and logout retain the remaining four credits. Wrong-message refusal
can stop before BLS verification; it is not a worst-case invalid-proof cost.

| Operation | Instructions | Heap pages before → after | Request bytes | Response bytes |
| --- | ---: | --- | ---: | ---: |
| Wrong-message certified proof | 42,116,105 | 18 → 18 | 285 | 13 |
| Fresh certified admission | 239,590,216 | 18 → 18 | 285 | 23 |
| Exact certified retry | 277,174 | 18 → 18 | 285 | 23 |
| Certified-session authorization | 23,175 | 18 → 18 | 12 | 13 |
| Certified-session logout | 18,895 | 18 → 18 | 12 | 22 |
| Revoked authorization | 20,855 | 18 → 18 | 12 | 13 |

Retry returns the original lease, clamped to the fixed login deadline. Logout
releases the session while retaining its replay fence; subsequent authorization
refuses. These are installed Wasm executions of real cryptographic verification,
but the certified credential uses an independently signed test network, not an
actual Canic-issued credential. The separate witness lane above uses PocketIC's
real certificate. Neither lane establishes production host integration or accepted
resource budgets. Population scaling and independent session/byte/per-account
capacity saturation remain the next local measurement work.


Validation of this final artifact passed all twelve installed tests (five lifecycle
and seven measurement scenarios), plus both HTTP host tests covering nine native
browser cases. Native/Wasm strict lint, formatting and independent fixture
verification passed. The older tables retain their historical artifact identities;
adding the certified verifier changes fixture code layout, so instruction counts
from different artifacts are not asserted identical.


### Host/Testkit revalidation

The pending Host Artifacts/FS 0.80.3, Host Process 0.80.4 and Testkit 0.108.0
adoption reran all twelve installed tests successfully. All twenty-nine measurement
observations and the fixture Wasm identity above remained unchanged. These suppliers
belong to the native qualification harness; the product dependency graph is unchanged.


### Runtime/guard 0.23.16 revalidation

The qualification Runtime, macros and executor guard now resolve together at
0.23.16, retaining Types 0.21.2 and the Host/Testkit cohort above. All twelve
installed tests and both HTTP tests (nine browser cases) pass; all twenty-nine
measured observations remain identical.
The rebuilt fixture is 1,496,599 bytes, SHA-256
`6448ccb96476e9fe1fae4f6b9b42117c8406b1bba72092fef112291975169bb7`. This records its actual identity even though supplier Rust sources
are unchanged; Runtime's published Types requirement now permits compatible
patch versions instead of pinning exactly 0.21.2.


## Independent capacity boundaries, 2026-11-17

Three `independent_capacity` cases add session, logical encoded-byte and per-account
replay saturation. Installation-only overrides select each limit; no admission
request can change it. Each case admits the same independently signed candidate
on an empty constrained receiver and on a populated receiver with only the selected
limit relaxed. The populated constrained receiver must refuse without changing
session/replay bytes, login fences, per-account counts, verification credit or
application writes. Existing exact retry retains its original deadline and credit;
the original session remains authorized while the refused caller has none.

- Session case: one occupied session with four login slots, eight global replay
  slots and eight per-account replay slots. Raising only sessions to two admits.
- Byte case: an unrestricted positive control observes 345 bytes for one admission
  and 690 for two. A 690-byte ceiling admits both; 689 accepts the first and refuses
  the second. Session/login/replay limits remain spare. These are Auth's logical
  session/replay bytes, not heap usage or a maximum-sized record claim.
- Account case: one retained replay exhausts that account's one-slot quota while
  global replay capacity is eight. Raising only the account ceiling to two admits.
  A different independently signed subject also admits on the constrained receiver,
  proving available global capacity and separate account ownership.

Logout frees the original session and its byte occupancy. A previously refused
second device can then admit in the session/byte cases, while the original login
stays revoked and both replay fences remain. Account replay quota remains charged
after logout, so that account's second device still refuses without eviction.
The additional host-only account vector uses the existing independent Node/OpenSSL
packer; the nine retained native vectors remain unchanged.

| Refusal | Instructions | Heap pages before → after | Request bytes | Response bytes |
| --- | ---: | --- | ---: | ---: |
| Independent session limit | 259,214 | 18 → 18 | 330 | 13 |
| One byte below two-record fit | 259,269 | 18 → 18 | 330 | 13 |
| Independent per-account replay limit | 259,219 | 18 → 18 | 330 | 13 |

Source is `a05b5dd` plus the pending qualification work. The working-tree locks
separately selected Platform 0.34.13 before this run; these measurements use that
selection, Types 0.21.2, Runtime/macros/guard 0.23.16, Host Artifacts/FS 0.80.3,
Host Process 0.80.4, Testkit 0.108.0 and Wire 0.24.3. Rust is 1.100.0; PocketIC is
16.1.0 with an NNS subnet. Fixture Wasm is 1,498,526 bytes, SHA-256
`b528c33ce5699f81e46d50d4108e69fe16ee14468bd26349f8c86900f2df5543`.
Historical observations above keep their original artifacts and dependency cohorts.
These are small-state boundary checks, not population scaling or accepted production
budgets. Global replay saturation independent of the per-account limit, populated
lookup/prune scaling and production host overhead remain unqualified here.


Final validation passed all fifteen installed tests (five lifecycle and ten
measurement scenarios), plus both HTTP host tests covering nine browser cases.
Strict native/Wasm lint, formatting and all independent fixture checks passed.


## Global replay and small-population pruning, 2026-11-17

The independent global replay case leaves eight per-account replay slots, four
sessions and four login slots available, but limits the receiver to one global
replay row. After one admission, both the same account's second device and a
different subject refuse without charge or state mutation. Empty controls accept
both candidates; increasing only the global limit to two accepts the second device
on a populated receiver. Original authorization and exact retry remain valid.
Logout frees the session but retains the replay reservation and global refusal.
Measured refusal: 259,213 instructions, 18 → 18 heap pages, 330 request bytes and
13 ordinary response bytes.

The population scenario uses 1, 8 and 32 independently signed credentials from the
existing Node/OpenSSL packer. Each has a distinct account/browser and a common
proof window. Host-only vectors total 31,707 bytes and remain within the existing
32,768-byte native fixture admission bound; retained native vectors are unchanged.
All three receivers use the same installation-owned limits: 32 sessions/logins,
64 global replays, two replays per account, 32,768 logical bytes and 128 verification
credits replenished every 100 ms. The initially empty bucket fills before admission.
This isolates population changes from configuration changes.

The scenario measures the final fresh admission and authorization of the first and
last admitted callers. At time 35 seconds, proofs have expired but sessions remain
live. Pruning removes at most four replay rows per call; observed usage, unchanged
credit and retained login counts independently check each report. Every session
then returns its original exact-retry lease despite proof expiry and replay pruning.
At time 110, bounded pruning removes expired sessions. All session/replay bytes
reach zero while every known login fence remains. Empty pruning removes nothing.

| Measured operation | 1 session | 8 sessions | 32 sessions |
| --- | ---: | ---: | ---: |
| Final fresh admission | 2,571,260 | 2,567,280 | 2,619,222 |
| Authorize first admitted caller | 18,846 | 20,898 | 22,127 |
| Authorize last admitted caller | 18,846 | 23,906 | 28,205 |
| First replay-prune batch | 6,906 | 37,724 | 55,702 |
| First session-prune batch | 4,646 | 14,022 | 16,727 |
| Empty prune batch | 2,484 | 2,484 | 2,484 |

Values are instruction counts. First pruning batches remove one row for the
one-session population and four for the larger populations; they are not equal
amounts of work across all three columns. Complete cleanup takes 1, 2 and 8 calls
per phase, respectively. All samples report 18 → 18 total linear-memory pages.
Admission frames are 330/23 request/reply bytes, authorization 13/13, pruning 13/16.
As above, intervals exclude outer codecs/runtime overhead; ordinary authorization
includes the fixture's application-write projection. These single-run observations
do not establish asymptotic complexity, allocation counts or production budgets.

Source remains `a05b5dd` plus pending qualification work. The supplier cohort is
Platform 0.34.13, Types 0.21.2, Runtime/macros/guard 0.23.16, Host Artifacts/FS 0.80.3,
Host Process 0.80.4, Testkit 0.108.0 and Wire 0.24.3. Rust 1.100.0, release Wasm,
PocketIC 16.1.0 NNS subnet. Fixture Wasm: 1,526,274 bytes, SHA-256
`a5ed17de7b512f7cc220d1f3c0b5b261a9c687aedba6405c56846fdff7aef259`.
All seventeen installed tests pass (five lifecycle and twelve measurement scenarios,
fifty-one observations), plus both HTTP host tests covering nine browser cases.
Strict native/Wasm lint, formatting and independent fixture checks pass.

Larger populations, maximum-sized records/grants, mixed-expiry maintenance,
production endpoint overhead and accepted operating budgets remain separate work.
Historical tables retain the artifacts and supplier cohorts they measured.


## Mixed expiry and maximum scope dimensions, 2026-11-17

The mixed-expiry scenario admits three independent credentials whose login/proof
windows end at different times, then records one valid unknown browser's logout
as Unconfirmed. Pruning at successive host times checks the selected expiration
indexes against actual retained counts and bytes. At time 31, a one-row batch removes
only the expired session; a separate two-row batch removes its replay and another
expired replay, leaving a later replay and two live sessions. Those sessions retain
exact-retry leases and authorization. Later batches separately remove the last
replay, the middle/late sessions and the unknown denial. Early/empty pruning does
nothing. All known login fences remain, and cleanup never changes work credit or
application writes. Unknown-denial bytes are outside session/replay byte accounting.

| Mixed-expiry operation | Instructions | Rows removed (session/replay/unknown) |
| --- | ---: | --- |
| No deadline reached | 2,567 | 0/0/0 |
| First expired session, batch limit one | 4,751 | 1/0/0 |
| Earlier expired replays, batch limit two | 12,583 | 0/2/0 |
| Later expired replay | 6,941 | 0/1/0 |
| Middle session | 4,750 | 1/0/0 |
| Late session | 4,884 | 1/0/0 |
| Unknown denial | 7,055 | 0/0/1 |
| Empty cleanup | 2,495 | 0/0/0 |

All these samples use 13-byte requests and 16-byte ordinary replies, and report
19 → 19 linear-memory pages.

The wide-scope scenario maximizes specific protocol dimensions: a 64-byte role,
thirty-two granted scopes each 64 bytes, narrowed to sixteen requested session
scopes. It does not claim maximum principal widths, numeric encodings, role count
or every other record dimension. Independent Node/OpenSSL packing/signing supplies
the credential; the installed host selects the matching fixed authority profile.
The logical footprint is independently summed from the baseline field widths:
345 + 60 extra role bytes + (16 × 65 − 5) scope bytes = 1,440 bytes. A 1,440-byte
ceiling admits and a 1,439-byte ceiling refuses before charging or allocating state.
Exact retry keeps its lease and accounting. Authorization requires all sixteen
selected scopes; requiring the seventeenth granted scope refuses without an
application write. Two-row pruning later clears session/replay bytes and preserves
the known login fence.

| Wide-scope operation | Instructions | Request bytes | Response bytes |
| --- | ---: | ---: | ---: |
| Fresh admission | 3,508,558 | 2,558 | 23 |
| Exact retry | 937,704 | 2,558 | 23 |
| Authorize sixteen scopes | 339,828 | 13 | 13 |
| Prune session and replay | 13,173 | 13 | 16 |

These samples also report 19 → 19 pages. Intervals include fixture construction
and cloning of the protected role/grants/request selection as well as Auth work;
they are not isolated algorithm timings. As before, outer Wire decoding/encoding
is excluded. The wide input is 2,558 encoded bytes. Receiver fixture byte/depth caps
stay 4,096/eight; allocation and traversal allowances rise to 524,288/32,768. The
former 131,072 allocation allowance and 8,192-node allowance independently reject
the wide frame. The host vector file grows to 39,159 bytes under a 65,536-byte
admission ceiling. Product Wire limits and retained native vectors are unchanged.

Source remains `a05b5dd` plus pending qualification work; supplier cohort matches
the preceding Platform 0.34.13/Runtime 0.23.16 observations. Rust 1.100.0, release
Wasm, PocketIC 16.1.0 NNS subnet. Fixture Wasm: 1,529,882 bytes, SHA-256
`4cd55ec86d2c5326370ca3b0942288ee9bcd3e106963f048b34c5060877670c5`.
Larger populations, other maximum-record dimensions, production host costs and
accepted operating budgets remain separate. Historical measurements retain their
original artifact identities and allowances.


Final validation passed all nineteen installed tests (five lifecycle and fourteen
measurement scenarios, sixty-three observations), plus both HTTP host tests covering
nine browser cases. Native/Wasm strict lint, formatting and independent fixture
verification passed.
