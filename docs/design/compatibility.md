# Compatibility and acceptance

## Fixed reference

Reference repository: https://github.com/dragginzgame/ic-auth

Reference commit: `e52a40be75dd9809d0528189d84b43fe96952ad9`.
The sibling checkout is convenient for reading; resolve reference files from this
commit, not its moving working tree. `reference.sha256` records selected file bytes.
The reference is evidence, not a dependency on the old engine in the new product.

## Preserve

- Wire method IDs, request/reply layouts, tags, byte/allocation/node/depth budgets,
  error projections and Rust/TypeScript interoperability.
- Canonical signing domains and bytes, claims schema 2, strict ordering and
  rejection behavior. Never sort untrusted signed inputs into validity.
- Credential grant bounds, browser-key/caller binding, scope narrowing, device
  fencing, protected authority generations and exclusive validity deadlines.
- Both proof mechanisms, strict signature policy and trusted enrollment inputs.
- Exact retries, replay retention across logout, interrupted issuance/renewal
  recovery and durable browser logout uncertainty.
- Bounded abuse budgets, failed-attempt charging and no mutation on applicable
  rejection paths, as established by the current contracts and tests.

Wire compatibility does not imply stable-record compatibility. No storage format
change is selected by this initialization. Before a backend cut, explicitly
choose preserved current records or a hard cut with legacy refusal; never silently
reinterpret/reset storage or erase a host's data. This repo has no migration authority.

`ic-storage` is the interim implementation dependency; `ic-storage-new` with
orthogonal persistence is the intended successor. Preserve logical Auth behavior
across that transition, without promising package/API or stored-format
compatibility. Reuse independent semantic cases and add retained-state upgrade,
schema conversion, incompatible-layout refusal and recovery/rollback evidence.
An interim stable-backend pass is not OP qualification. Selecting this direction
does not authorize importing, resetting or migrating any consumer's live state.

## Evidence required before replacement claims

| Boundary | Required evidence |
| --- | --- |
| Encoding | Independent retained golden bytes; old/new cross-decoding and generator output comparison |
| Verification | Valid and adversarial vectors for each selected mechanism, including wrong authority/caller/time |
| Backend semantics | Same behavioral contract on volatile and stable implementations; failed commits and exact retries |
| Lifecycle | Stable reopen/upgrade, corruption refusal, generation changes, replay/logout retention |
| Runtime | Native and actual Wasm host qualification; one coordinated executor/guard |
| Browser | Real SDK identity transport, Rust/TypeScript interoperability and interrupted logout recovery |
| Costs | Selected graph/size observations and comparable populated-store work; retain existing accepted ceilings |

Port independent contract tests and vectors before replacing their implementation.
Do not copy every test mechanically or delete tests to reach a size goal. Preserve
unique guarantees and explain redundant coverage removal. The initial
[request-boundary slice](../development/request-boundary.md) has focused scope,
request-hash and TTL evidence; the [claims codec](../development/claims-boundary.md)
has independent signed-byte/digest and bounded rejection evidence. Neither
completes a replacement gate above. The first
[Wire boundary](../development/wire-boundary.md) covers all four contracts with
37 independent reference frames, generated-browser interoperability and separate
reference/current native resource thresholds. This is corpus-based codec
evidence. The [Ed25519 verifier](../development/verification-boundary.md) now has
an independent signature vector, protected context/refusal cases and a deliberately
legacy-enabled scalar negative control. The
[certified issuer component](../development/certified-issuer-boundary.md) has real
BLS certificate fixtures for issuer/seed/network binding and freshness, including
subnet-range delegation controls. [Delegation signing commitments](../development/delegation-encoding-boundary.md)
have independent byte/hash evidence; the [root verifier](../development/root-delegation-boundary.md)
has independent OpenSSL evidence for enrolled authority, grant binding and proof bounds.
The [complete certified verifier](../development/certified-verification-boundary.md)
now has independent root/issuer composition and shared-budget evidence.
The [request policy](../development/request-boundary.md#shared-narrowing-and-lease-calculation)
also checks single-role narrowing and captured lease ceilings through both proof
mechanisms, without claiming session or replay admission.
The [receiver policy boundary](../development/receiver-policy-boundary.md) checks
retained-state read/retry decisions and proof-expired retries. Its fact drivers
are not evidence of backend admission, retention, rollback or host integration.
Complete adversarial acceptance, actual SDK/browser lifecycle and backend qualification
remain outstanding.

Reference entrypoints:

- `docs/design/0.6-auth-protocol.md`: signed protocol and trust contract.
- `docs/protocol/sessions.md`: session, replay, revocation and storage obligations.
- `docs/integration/persistent-auth.md`: host transaction and stable-store custody.
- `docs/protocol/browser-client.md`: browser lifecycle and uncertainty handling.
- `crates/ic-auth-wire/src/`: actual Wire definitions.
- `crates/ic-auth/tests/` and `qualification/ic-auth-backend-conformance/`:
  behavioral evidence to classify and retain.
