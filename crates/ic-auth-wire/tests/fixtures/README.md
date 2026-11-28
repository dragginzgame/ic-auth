# Independent Wire frames

The two TSV files contain 37 sparse frames assembled by `../verify-fixtures.mjs`. It writes literal method IDs, Postcard primitives, variant tags and record extension blocks independently of the Rust and generated browser codecs.

The original case selection came from the historical browser contracts at revision `e52a40be75dd9809d0528189d84b43fe96952ad9`. Root `reference.sha256` retains their provenance. The current fixtures use the sparse method IDs and layouts introduced in Auth Wire 0.52.0.

The caller cases cover both unit requests, all read and logout outcomes, boundary principal lengths and deadlines, and the optional application extension. The admission cases exercise both credential directions and proof variants, witness steps, nested fields, scopes, optional TTL and every admission reply. The proof bytes exercise transport decoding independently of cryptographic verification.

`make qualify-wire-browser` regenerates all four current codecs, runs that
fixture check, then checks encoding/decoding, method/mode metadata, retention
constants, detached subarrays, truncated/trailing frames and redundant outer
Postcard varints. Node 24.21.0 is required; no legacy checkout is needed.
`node crates/ic-auth-wire/tests/verify-fixtures.mjs` runs fixture assembly alone.
Rust product and qualification tests consume the same checked TSVs.

Signed-claims canonicalization, SDK transport, installed browser lifecycle and
cryptographic verification remain separate boundaries.
