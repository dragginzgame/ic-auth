# Initial request boundary

The first implementation covers passive scope labels and unsigned session
requests. This is an independent slice of architecture stage 2, not a complete
protocol implementation or authority to admit sessions. The original slice shipped
in the 0.1.0 source milestone; current versions are in the manifests and all
crates retain `publish = false`.

| Contract | Owner | Independent evidence |
| --- | --- | --- |
| Scope grammar, 1–64 UTF-8 bytes, no normalization | `ic-auth-protocol-types::ScopeLabel` | Explicit invalid cases/offsets, all ASCII bytes in three grammar positions, 64/65-byte bounds |
| 1–16 unique unsigned scopes, canonical sorting | `ic-auth::session_request::SessionRequest` | Reordered input, duplicates, count/grammar/TTL failure precedence, 1024-byte boundary |
| Exact V1 domain, u64 framing and TTL presence | `SessionRequest::hash` using published SHA-256 | Four fixed reference digests, independently reframed with Perl `Digest::SHA` |
| Host TTL bounds and overflow refusal | `SessionTtlPolicy` and `expires_at_ns` | Invalid policy pairs, exact ceiling, maximum requested TTL, exact u64 deadline/overflow boundary |

Reference evidence comes from `dragginzgame/ic-auth` at
`e52a40be75dd9809d0528189d84b43fe96952ad9`; selected file hashes are recorded in
`reference.sha256`. See the [fixture provenance](../../crates/ic-auth/tests/fixtures/README.md).
The implementation was written with the pinned source/contracts available;
this is not a claim of clean-room separation. Retained digest values predate
the replacement; new tests do not calculate expected hashes with its encoder.

The request boundary uses the canonical grant validator for narrowing, without
depending on issuer state or cryptography. Scope grammar errors are exposed as
`SessionRequestError::Scope`; their typed grammar details and refusal order are
preserved. Numeric TTL policy is one validated value for this caller. There is
no separate TTL-limit wrapper until an actual authorization caller needs it.
These are provisional Rust API choices; full source compatibility and downstream
API removal remain subject to the design's consumer inventory.

No caller, role, grant, clock or enrollment is inferred from a request. TTL
calculation still needs a trusted host input and does not cap login, key or
enrollment deadlines. Positive oversized requested TTLs preserve request identity
but refuse policy evaluation. Sorting is confined to unsigned requests; this
code must not be reused to normalize signed grants into validity.

Run `make test-request`, `make lint-protocol`, `make doc-protocol` and
`make check-protocol-wasm` for this scope. Wasm compilation does not establish
installed execution, persistence or rollback. No Storage dependency is introduced;
the planned `ic-storage` to `ic-storage-new` transition remains backend-local.

## Shared narrowing and lease calculation

`SessionRequest::evaluate_grants` admits grant shape/order, selects exactly the
protected receiver role, checks every requested scope against that grant, then
evaluates TTL with checked arithmetic. Missing roles and missing scopes refuse;
scopes from multiple roles cannot be combined. Unordered/duplicate signed roles
or scopes are refused without mutation. This helper accepts the existing bounded
grant profile; fresh claims still require their separate aggregate 32-entry cap.
It performs no proof verification and returns only a requested deadline.

Both complete verifiers construct `VerifiedClaims` through one private owner that
captures the minimum of login, authority/key and enrollment deadlines. Its
`session_deadline` method applies the same request policy at the original host
time and caps the result by that captured ceiling. Proof expiry does not cap a
live lease. A fresh enrollment can affect a newly verified proof, but cannot
lengthen a previously captured result. Oversized TTLs still refuse before capping;
a short login deadline does not make an invalid requested duration acceptable.

These calculations are not admission receipts or authorization. A receiver must
own verification and consumption within one uninterrupted transaction, recheck
authority, replay, denial and device facts, then commit identity state atomically.
No timestamp, public helper result or standalone proof is a substitute for those
rules. This increment adds no store, session constructor or commit callback.

`make test-session-policy` selects four cases: two independent request-policy
tables, an Ed25519 captured-deadline case, and a qualification case applying the
same literal deadlines to independently signed certified/Ed25519 fixtures. Cases
cover missing roles, no scope union, canonical refusal, TTL precedence and
independent login/key/enrollment caps, including leases extending past proof
expiry and previously verified evidence remaining unchanged after refresh.
The retained fixture hashes/signatures predate these helpers; expected deadlines
are literal protocol examples, not computed by the replacement implementation.
Reference contracts are `session_request.rs`, `portable/verified.rs` and
`docs/protocol/sessions.md` at the pinned reference commit. Source was consulted;
no legacy session engine or storage implementation was copied.
