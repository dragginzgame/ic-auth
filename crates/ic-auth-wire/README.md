# IC Auth Wire

Bounded IC Wire contracts for portable credentials, session admission, local
session reads and logout. This package depends on passive Auth types and published
IC Wire; it has no Candid, CDK, crypto, Canic, Toko or storage dependency.

Convert domain credentials with `model::PortableCredential::try_from`, then use
`wire::encode_request::<CredentialEnvelope>` for detached data. Conversion back
retains the signed claim bytes and validates checked identifier syntax. It does
not authenticate any proof. Admission still requires the complete Auth session
engine against protected host authority, caller, clock and atomic storage.

The method IDs, explicit field/variant IDs and finite bounds are the transport
contract. Consumers own actual endpoint names, ingress authentication, raw
argument/reply allocation, dispatch, cancellation and retries. No endpoint is
installed by linking this library. Response encoding failure after mutation must
trap so local effects roll back. Never retry an unknown issuance as new work.

`typescript` enables the upstream host generator only. Canister dependencies
should leave it disabled. Generated TypeScript and Rust use the same layouts and
IC Wire's canonical bounded codec. Signed claims and session records retain their
existing independent formats.

This is a coordinated application-transport hard cut. There is no Candid fallback
or previous browser token-store reader. The maintainer confirmed there are no
deployed old records or unresolved old issuance effects to preserve. New portable
logout journals continue to retain unknown issuance and durable invalidation.

Released Auth 0.13 exposes IC Wire 0.5 through `wire`; historical Auth 0.10 uses
Wire 0.4. Align consumers sharing these Rust types. `Wire` no longer implies
owned deserialization: generic decoders of owned replies/requests need explicit
`serde::de::DeserializeOwned` bounds; encoding-only callers need no new bound.
Auth's concrete DTOs remain owned.

Pending Auth 0.14 changes `EstablishSession::Response` to
`Result<SessionView, model::AdmissionRefusal>`. Hosts convert the core
`SessionError::admission_refusal()` result with `.into()` and retain detailed
errors locally. Regenerate and update receiver/client reply handling together;
there is no text-error fallback. The method IDs, request and successful-reply
bytes, read/logout contracts, signed claims and retained records are unchanged.
`NotYetValid` permits no early admission or deadline extension: it is only a
bounded same-credential retry hint. The enum does not implement retry policy.
