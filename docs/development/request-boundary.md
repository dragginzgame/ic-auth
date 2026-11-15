# Initial request boundary

The first implementation covers passive scope labels and unsigned session
requests. This is an independent slice of architecture stage 2, not a complete
protocol implementation or authority to admit sessions. Both crates retain their
intended package names with provisional version 0.1.0 and `publish = false`.

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

The request boundary no longer depends on a general credential codec error or
issuer/grant-selection code. Scope errors are exposed directly as
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
