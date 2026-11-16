# Canonical claims boundary

`ic-auth::claims` implements the current schema-2 portable-credential bytes:
bounded decode, structural encode and streaming SHA-256. Passive claim fields,
audience bytes and role labels live in `ic-auth-protocol-types`. Principal identity
comes from published `ic-platform-types` 0.21.2 with default features disabled;
there is no local identity, cryptography, runtime or storage implementation.

This is newly written, source-informed code for the specified boundary. The
retained [250-byte vector and digest](../../crates/ic-auth/tests/fixtures/README.md#schema-2-claims)
provide independent evidence. No old engine, issuer or revocation modules were
copied. One private field traversal feeds both encoding and hashing; a private
claims cursor owns checked slice reads. Neither is a public serialization or
storage framework.

| Preserved obligation | Evidence |
| --- | --- |
| Domain, schema, mechanism and field order | Exact retained bytes/digest; both mechanism tags; every semantic field changes the fingerprint |
| No partial, extended or legacy interpretation | Every truncated prefix, retired schema, unknown tags, trailing bytes and malformed lengths refuse |
| Bounded allocation admission | Principal/label/count limits, entire-blob and remaining-grant byte caps, aggregate entry refusal before reading excess payload |
| Signed grant order | Duplicate/descending scope cases and duplicate role cases refuse without normalization |
| Fresh grant ceilings | All 1–16 role distributions with 32 total maximum-length scopes; 3,332-byte worst-case grant list; 33rd entry refused |
| Structural identity checks | Slots 0–7, nonzero generation, nonanonymous subject, empty/full-length principal cases and refusal order |
| Canonical representation | Accepted mutations of the independent fixture re-encode exactly; no alternate spelling is accepted |
| Authority separation | Invalid time windows and anonymous issuers remain structural input; no trusted context is manufactured |

The public byte ceiling remains 65,536, while the decoder bounds the remaining
grant payload to 4,096 before allocation. Valid fresh grants fit in at most
3,332 bytes. Every label is at most 64 bytes; count checks bound allocations to
16 grants and 32 aggregate scope entries. Per-role and aggregate scope limits
have separate owners even though both currently equal 32.

The codec's `ClaimsError` is a provisional Rust API. It preserves structural
refusals without depending on the old certificate-issuance/canonical error tree.
Future Wire projection still needs its own exact refusal mapping and tests.
No text/Serde projection for audience IDs, credential proof transport, browser
principal derivation, signature verification or authority admission is supplied
by this slice. Retained completed issuer results may need their separately
qualified recovery bounds; the fresh codec must not be used to discard them.

Run `make test-claims` for the new boundary and `make test-request` for the
existing request contract. `make lint-protocol`, `make doc-protocol` and
`make check-protocol-wasm` cover the two implemented protocol packages; Wasm
compilation is not installed execution. Current independent fixtures and generated Rust/browser interoperability own
encoding acceptance; old/new differential decoding is no longer required.
