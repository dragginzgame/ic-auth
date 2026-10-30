# Delegation and certified portable proofs

The current application path is portable credential admission into a caller-bound
local session, followed by scoped session checks. Both Ed25519 and certified
credentials use that engine. Structural decoding or Root delegation verification
alone grants neither a session nor application resource access.

## Retired delegated-token lifecycle

Released 0.9.0 removed the separate `DelegatedToken` prepare/retrieve/verify API,
its claims/transport DTOs and token-only canonical encoders. The maintainer
explicitly confirmed no deployed Rust-token users or unresolved old issuance
operations on 2026-10-17, separately from the earlier browser-store disposition.
There is no compatibility alias, old-format reader or automatic reset. Existing
published releases and downstream pinned deployments are unchanged by local source
preparation. Current portable logout journals still retain unknown issuance.

The `certified-proof-verification` feature is replaced by `certified-proof-verification`.
`RootKeyPolicy`, standalone Root verification and typed Root failures now belong
to `ic_auth::delegation`; certified credential limits belong to
`ic_auth::portable_certified::CertifiedProofLimits`. Signed Root certificate,
chain-key batch, proof hash and portable-claims bytes are unchanged. The certificate
field `max_token_ttl_ns` retains its signed identity and constrains portable
admission-proof lifetime; it does not restore the old application-token flow.

## Standalone root delegation verification

`ic_auth::delegation::verify_delegation_proof(proof, context)` is available with
`chain-key-verification`, without IC certificate/BLS verification. The host supplies
`DelegationProofVerificationContext`: expected issuer, independently enrolled
`RootKeyPolicy`, actual clock and finite `DelegationProofVerificationLimits`.
Expected authority never comes from submitted proof fields.

Bounds cover certificate lifetime, delegated maximum proof lifetime, future skew,
variable material and witness length. The verifier checks bounds before canonical
allocation/cryptography, nonanonymous Root/issuer, certificate shape and binding,
header/leaf/certificate/key/path/seed identity, Merkle direction, low-S secp256k1
signature, current key window and version/epoch floors. No cache bypasses policy.
It performs no management call, installation, signing, clock acquisition or write.

`VerifiedDelegationProof` borrows the certificate and exposes its canonical hash
and exclusive usable deadline capped by current Root policy. The Root signs the
issuer leaf: certificate `issued_at_ns` is checked for consistency, but is not a
signed leaf field. The certificate hash includes that metadata. Portable claims
bind that exact hash through `key_binding_id`; standalone Root verification must
not be described as authenticating a Root-signed certificate issuance timestamp.

## Certified portable admission

`portable_certified::verify_certified_claims` requires
`certified-proof-verification`. It composes the same Root verifier and complete IC
canister-signature verification with shared portable caller, account, issuer,
audience, grants and lifetime checks. Its protected `PortableCertifiedContext`
contains the actual ingress caller/clock, approved enrollment, Root key policy,
IC network key and `CertifiedProofLimits`. `CertifiedProofLimits::into()` projects
the applicable bounds with zero future skew. This profile has no skew option:
future Root validity starts and future signing-certificate times are refused.
Standalone Root verification retains its independently selected skew allowance.

The issuer signs `domain_separated_message(CERTIFIED_SIGNATURE_DOMAIN,
SHA256(canonical_portable_claims))`. Host admission bounds aggregate material before
decoding; cheap shape/context failures precede charging the receiver-wide
`SessionAdmissionBudget`. Failed expensive verification consumes credit. Normal
refusals and later storage failures must preserve charges rather than trap them
away. Neither proof variants nor submitted keys install trusted authority.

Verified claims feed the [session engine](sessions.md), which narrows grants to
the protected receiver role and requested scopes, caps the lease, consumes replay
and reserves login revocation atomically in host storage. Ordinary application
calls use their authenticated caller and live local session. Stable versus volatile
storage and reset floors remain host policy, independent of the proof mechanism.

## Root binding diagnostics

`DelegationProofError::RootSignatureBindingMismatch` identifies disagreement
between submitted signature metadata and its batch header.
`RootKeyPolicyBindingMismatch` identifies disagreement with independently enrolled
policy. Both carry a `delegation::DelegationField` enum; window targets use
`delegation::DelegationTarget`. Match variants such as `DelegationField::KeyId`
instead of string literals. `Display` retains the existing diagnostic labels.
Header/signature mismatches precede the
corresponding enrollment check. Other typed causes include witness, window,
canonicality, stale authority, malformed signature and high-S rejection.
Consumers project these causes; they must not reverify submitted fields to
manufacture another authority or freeze upstream diagnostic text.

## Canic verifier integration

[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns consumer adoption.
Move imports to `ic_auth::delegation`, select the current certified capability and
use portable credentials for application admission. Root installation still uses
the standalone verifier and protected expected issuer/key/network policy.
`AuthRequestMetadata`, Root proof hashes, role attestation and signature creation
remain shared independent contracts; removing the old token API does not retire
them. Canic owns issuer approval, renewal, transactions, scoped guards and endpoint
adapters. Toko owns account/resource policy and updating its pinned Canic calls.

The inspected Toko source still contains older token calls and pins Canic
`=0.110.48`; this is adoption work, not evidence contradicting the maintainer's
explicit deployment disposition. Do not edit sibling source or reset their data
from this repository. [Auth #69](https://github.com/dragginzgame/ic-auth/issues/69)
records retirement and [Toko #1803](https://github.com/dragginzgame/toko/issues/1803)
records consumer coordination.

## Qualification

`make test-certified-proofs` exercises real secp256k1/BLS proofs, malformed and
high-S signatures, witness/epoch/key/window/bounds rejection and portable budgets.
`make test-sessions` covers replay, atomic commit, logout, scope narrowing and
restoration. `make test-qualification` composes real certificates, fresh browser
principals, two receivers, signed SDK ingress, logout and upgrade behavior.
Production issuer/enrollment and final Canic/Toko acceptance remain separate.
