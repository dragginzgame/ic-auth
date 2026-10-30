# IC Auth Wire

Bounded IC Wire contracts for portable credentials, session admission, local
session reads and logout. This package depends on passive Auth types and published
IC Wire; it has no Candid, CDK, crypto, Canic, Toko or storage dependency.

Convert domain credentials with `model::PortableCredential::try_from`, then use
`CredentialEnvelope::encode_request` for detached data. Conversion back
retains the signed claim bytes and validates checked identifier syntax. It does
not authenticate any proof. Admission still requires the complete Auth session
engine against protected host authority, caller, clock and atomic storage.

Auth 0.22 boxes `model::PortableProof::Certified` to keep the transport
enum compact. Direct Rust constructors use `Box::new`; domain conversions,
generated browser types, encoded bytes and transport limits are unchanged.

Auth 0.23 replaces `Bytes<29>` Principal fields with Platform-owned
`Principal`. Rust DTO callers pass those values directly; convert explicitly
from external SDK identities. Native IC Wire still encodes bounded principal
bytes, with the same method IDs, field order and generated browser contracts.
Human-readable Serde now uses Principal text instead of byte arrays; it is not
the application transport. No stored record or browser journal format changes.

Pending Auth 0.24 changes `ReadSession` to typed `model::SessionReadRefusal`
responses. Its identity becomes `0x415554480013`; the former
`0x415554480003` request/reply frames are refused. Update clients and receivers
together and regenerate bindings. Categories cover missing/expired sessions,
unavailable authority, rejected scopes, revocation and internal unavailability.
Keep detailed errors host-side; success still requires complete live checks.
Credential/admission/logout identities, signed bytes and stored records are
unchanged.

The method IDs, Serde field/variant declaration order, bounded field types and
explicit native traversal allowances are the transport contract. Consumers own actual endpoint names, ingress authentication, raw
argument/reply allocation, dispatch, cancellation and retries. No endpoint is
installed by linking this library. Response encoding failure after mutation must
trap so local effects roll back. Never retry an unknown issuance as new work.

`typescript` enables the upstream host generator and Platform's Principal Wire
metadata only. Canister dependencies
should leave it disabled. Generated TypeScript and Rust use the same layouts and
IC Wire's bounded codec with strict primitive parsing. Wire 0.7 does not
reserialize decoded values: maps, sets and custom Serde visitors may normalize
input. Auth transport uses order-preserving lists and opaque signed bytes;
semantic scope/grant validation remains in Auth. Retain original bytes for exact
retries, signatures and hashes. Signed claims and host-owned session records have
independent contracts; public transport does not choose their formats.

The earlier Auth 0.9 application-transport cut retired the old token-store reader
after the maintainer confirmed no deployed records or unresolved issuance effects
used that old format. There is no Candid fallback. Current portable logout
journals retain unknown issuance and durable invalidation; this migration does
not authorize discarding them.

Auth 0.16 exposes IC Wire 0.7 through `wire`; Auth 0.15 used Wire 0.6. Align Rust
consumers sharing these types. Wire 0.7 removes `Error::Noncanonical`; invalid
primitive encodings return `Malformed`. Encoder bytes and method IDs are
unchanged by this adoption. Earlier
Auth 0.14 uses Wire 0.5. The finite `Method` trait, serializer derive and typed
retained frames are removed. Use the contract's pure `encode_request`,
`decode_request`, `encode_response` and `decode_response` functions. They bind
the concrete DTO, method identity and limits to Wire's native codec and return
ordinary owned bytes. Retain those bytes unchanged across uncertain effects.
Generic native Testkit callers supply `WireCallSpec`; no replacement method trait
is provided. `typescript` derives host-only Serde metadata for regeneration.

Admission keeps its name but changes its ID from `0x415554480002` to
`0x415554480012`. Serde uses zero-based refusal tags; the new ID prevents
misinterpreting old tags. Update admission clients and receivers together.
Credential, read and logout IDs and their frozen bytes remain unchanged, as do
signed claims, session/replay records and browser logout journals. No fallback
reader, implicit reissuance or storage reset is introduced.

Hosts project `SessionError::admission_refusal()` into `model::AdmissionRefusal`
and retain detailed diagnostics locally. `NotYetValid` is a same-credential retry
hint, never permission for early admission or extending the original deadline.
