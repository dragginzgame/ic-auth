# Root-delegation signing commitments

`delegation_encoding` owns the current certificate bytes and hash, issuer-binding
hash, root batch-header hash, issuer-leaf hash and ordered derivation-path hash.
The passive protocol records factor shared issuer authorization fields into
`IssuerDelegation`, used by `DelegationCertificate` and `DelegationLeaf`.
`RootBatchHeader` carries the root-signed header. These are domain records, not
Serde/Wire DTOs. No transport layout or endpoint owner changes.

The profile is IC canister-signature issuer binding, Fleet audience and
ECDSA/secp256k1 root signing. Existing domains, tags, big-endian widths, order and
length framing remain fixed. The certificate includes issue-time metadata; the
root leaf does not. Root verification must still check their full correspondence,
including that metadata's consistency. A submitted binding hash is preserved,
never recomputed into agreement with its submitted seed.

Claims and delegations share grant validation and byte emission. Delegations admit
1–16 roles and 1–32 scopes per role with strict ordering, preserving the broader
protected grant set rather than applying fresh credentials' 32-total-entry cap.
The new bounded encoder refuses empty or oversized grants before emitting bytes;
the old general canonical helper deferred those cardinality refusals to delegation
verification. Root names are at most 128 UTF-8 bytes; paths have at most 32
components of at most 256 bytes. Empty names, paths and components remain
structurally encodable. These bounds make the u32 framing lengths finite without
an unbounded intermediate buffer. A single private field traversal owns each
layout's length calculation and hash input.

Encoding does not validate schema support, anonymous authority, validity windows,
issuer/seed agreement, enrolled keys, root signatures, Merkle membership or
current authority. It returns no verified evidence. Full certified credential
verification must still perform those checks, bind claims and meter aggregate
proof material/work before accepting the issuer signature.

`make test-delegation-encoding` runs six cases: frozen bytes and hashes, individual
field binding, strict grant refusal and maximum cardinality, root metadata/path
bounds, and preservation of untrusted binding/time fields. The existing nine
claims tests also pass after sharing grant emission. Three hashes are literal
reference vectors; Node/OpenSSL independently assembles all six fixture records.
See [provenance](../../crates/ic-auth/tests/fixtures/README.md#root-delegation-signing-commitments).
The script is an optional independent fixture check; Rust tests need no Node.

The codec-only graph retains no crypto verifier, storage, Canic or runtime
executor. Lint/docs and Wasm library compilation cover these APIs; this is signing
commitment evidence, not full proof verification or an installed-host result.
