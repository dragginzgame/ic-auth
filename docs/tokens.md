# Application-token verification

The optional `token-verification` feature and `ic_auth::token::verify_token`
are published in `0.1.4`.
The feature enables IC canister-signature verification and `k256` secp256k1
verification. Encoding-only and signature-only graphs remain independently
selectable; no selected graph imports Canic, Toko, Solana, SIWS, native Host
tooling or a CDK/storage backend.

`verify_token(token, context)` accepts the existing `DelegatedToken` wire model.
It checks complete root and issuer proofs with established cryptography and
returns a private-field `VerifiedToken`. There is no public constructor,
deserializer, signature callback or cached/semantic-only admission path.

The host supplies `TokenVerificationContext` from authenticated runtime and
protected configuration, independently of the submitted token:

| Input | Responsibility |
| --- | --- |
| `caller` | Actual authenticated transport caller; must equal presenter and subject |
| `audience`, `role` | Exact protected local network/application identities and role |
| `allowed_scopes` | Current local scope ceiling, canonically sorted and unique |
| `required_scopes` | Requirements selected by the endpoint/operation |
| `root_key` | Independently enrolled chain-key authority and current acceptance policy |
| `ic_root_public_key_raw` | Protected IC network BLS trust anchor for issuer certification |
| `now_ns`, `limits` | Trusted clock, time allowances and bounded-input/lifetime policy |

The root policy binds root principal, algorithm, exact key name, public key,
derivation-path hash and key version. Proof, registry and key-version floors and
the exclusive policy deadline are checked on every call. Network-specific key
eligibility belongs to the host: for example, a Canic mainnet adapter must reject
test-key enrollment before supplying this policy. Do not derive policy or fetch
keys using client-selected fields. A key name or an ECDSA signature alone does
not establish an IC management-canister key's provenance.

The root proof authenticates an issuer leaf in a signed chain-key batch. All
header/leaf/certificate/signature bindings are exact, including issuer principal,
seed binding, algorithm, audience, grants, validity, registry identity and proof
epoch. The leaf lies inside the batch window, whose TTL is bounded by protected
`max_revocation_latency_ns`. Merkle direction is significant. Signatures must be
the existing 64-byte `r || s` encoding with valid nonzero scalars and low-s
representation; `k256` verifies the existing canonical header hash as a prehash.
The public key must match the protected enrollment byte-for-byte.

Only after authenticating the root proof does the verifier use its issuer and
seed hash to check the issuer's IC signature. It signs the canonical claims hash
under the unchanged `canic-issuer-delegated-token` domain. The signature verifier
checks the exact seed/message tree path, certificate and subnet authority, then
host-selected certificate freshness; see [the IC signature contract](signatures.md).

Certificates and claims preserve existing protocol cardinalities: nonempty grants,
at most 16 distinct sorted roles and at most 32 distinct sorted scopes per role.
Scope grammar, the 64-byte scope bound and the 4096-byte extension bound use the
canonical owner. Token grants may narrow certificate grants, and local grant
scopes must all lie within the current protected ceiling. Required scopes must
appear in that local grant. Other granted roles confer no local role authority.
Anonymous caller/presenter/subject/issuer/root identities are rejected.

Certificate issue time cannot exceed its not-before time. Certificate/token TTLs
are positive and bounded; token validity fits within its certificate. Expiry is
exclusive. Future starts may be ahead of the host clock only within the explicit
`max_future_skew_ns` allowance; use zero for strict starts. Arithmetic rejects
invalid windows and avoids overflow at clock boundaries. There is no implicit
Canic one-minute allowance.

`max_variable_bytes` is checked before canonical encoding or cryptography. It
counts all variable string/vector bytes across claims, certificate, root leaf and
signature material, one byte per role/scope/path entry, and 33 bytes per witness
step. Fixed-width scalar/principal fields are not part of this budget. Separate
limits bound witness count and issuer CBOR bytes. This is a logical material
budget, not the outer Candid wire size. Hosts must also bound request decoding
before constructing these passive protocol values. IC keys retain the checked
short-form DER boundary from the signature capability.

The result borrows immutable claims and the local grant and exposes the claims
hash and exclusive deadline capped by the current root policy. It is evidence of
verification for this call. Do not persist it as a timeless authority or skip live
checks after policy changes. `Clone` does not refresh its validity. This API does
not consume a nonce or establish a session. The working tree's optional
[session engine](sessions.md) uses an explicit atomic host admission/replay
transaction when that flow requires it. Application resources and business
permissions remain independently checked, and these tokens are not IC ingress
delegations. No secret signing operation, clock acquisition, certification-root
write, memory allocation into host stable regions or global cache occurs here.

## Standalone root delegation verification

The pending compatible 0.2.2 batch adds
`ic_auth::token::verify_delegation_proof(proof, context)` under the existing
`token-verification` feature. It accepts `DelegationProof` directly, before
claims or an issuer signature exist. There is no fabricated token, cryptographic
callback, runtime acquisition or separate proof-verification implementation.
Complete token verification retains its public context, errors, signed bytes and
validation ordering; both calls share the same certificate rules and root engine.

The host supplies `DelegationProofVerificationContext` with its actual intended
`expected_issuer`, independently enrolled `root_key`, `now_ns` and finite
`DelegationProofVerificationLimits`. Expected issuer must come from protected
runtime/configuration, not `proof.cert.issuer_pid`. Bounds cover certificate TTL,
the delegated maximum token TTL, future skew, variable material and witness
length. `TokenVerificationLimits::into()` projects those five bounds when an
existing host already owns that policy. This call does not require an IC network
BLS key, caller, local role/scopes or issuer-signature limits because it does not
verify a completed token.

The material budget uses the accounting above across the certificate and root
proof, excluding claims and issuer signature material. It is checked before
canonical allocation and cryptography. Anonymous issuer/root, mismatched expected
issuer/root, invalid windows/TTLs and malformed/noncanonical grants reject.
The same root engine checks header/leaf/certificate/key/path/seed bindings,
Merkle direction, real low-s secp256k1 signature, current enrollment window and
version/epoch floors. Errors use `TokenVerificationError`; token-only and issuer
signature errors cannot arise from this boundary.

The private-field `VerifiedDelegationProof` borrows the validated certificate and
exposes its canonical hash and exclusive deadline capped by the current key
policy. The root signs the issuer leaf: **certificate `issued_at_ns` is checked
for consistency but is not a signed leaf field**. The returned canonical
certificate hash includes that metadata and is not separately signed by the
root. Complete token verification additionally binds the exact certificate hash
through the issuer's signed claims. Standalone success must not be described as
proof of a root-signed certificate issue timestamp.

Success means the enrolled root delegated the signed issuer authority at this
clock and live policy. It is not token verification, installation/issuance
approval, a session, an ingress delegation or an application resource grant.
The host retains issuer approval, build-network/key-name admission, audience/grant
admission, authorized issuance, renewal, storage and certification composition.
Recheck current authority and expiry on subsequent use; cloning or storing the
result does not refresh it. Canic's pre-token root-verifier replacement is
coordinated through [IC Auth #10](https://github.com/dragginzgame/ic-auth/issues/10)
and [Canic #491](https://github.com/dragginzgame/canic/issues/491).

`make test-tokens` uses real deterministic secp256k1/BLS proof chains with rejection
coverage. `make check-wasm`, `make clippy` and `make check-boundaries` include the
new feature. These establish focused native/cross-compilation evidence, not live
IC key enrollment, ingress delegation, session replay/atomicity, service lifecycle
or Canic adoption. The accepted [design](design/extraction.md) and
[Canic #491](https://github.com/dragginzgame/canic/issues/491) retain those obligations.
