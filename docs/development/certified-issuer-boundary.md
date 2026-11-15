# Certified issuer-signature boundary

Enable `ic-auth/certified-issuer-verification` for
`issuer_signature::verify_issuer_signature`. This component authenticates the
fixed portable-domain claims digest and returns the certificate's signing time.
It cannot produce `VerifiedClaims` or admit a credential/session.

Platform 0.33.7 owns DER/CBOR parsing, hash-tree paths, IC root-to-subnet trust-chain
verification and BLS cryptography. Auth owns the fixed domain/digest framing,
protected issuer and seed commitment, raw network-root length, signature byte
admission and signing-certificate freshness. Host inputs supply authority and time;
proof fields never select an enrolled issuer or network root.

Admission order is signature bytes, 20–129-byte DER parsing, enrolled canister,
SHA-256 seed commitment, 96-byte raw network root, Platform authentication and
then time policy. The signature ceiling is the smaller of the protected host
limit and 131072 bytes. Age is inclusive, with no future grace or wrapping
subtraction. Complete nonminimal LEB128 certificate times remain accepted by the
supplier; missing, truncated, overflowed and trailing time material refuse.
An IC root-to-subnet delegation certificate need not itself have a recent time;
the authenticated signing certificate must satisfy the current age window.

The component has no mutable state and performs no work-budget charging. Full
certified credential verification must first bound aggregate variable material
(claims, issuer proof and Auth delegation), enforce independent witness limits,
check the common protected claims context and charge the shared budget before
cryptography. It must also authenticate Auth root authorization of the issuer's
grants and bind that authorization to the claims. The separate
[root verifier](root-delegation-boundary.md) now supplies that authorization
component. IC subnet delegation alone establishes none of those application grants.
The [complete certified verifier](certified-verification-boundary.md) now composes
these checks with common claims policy and aggregate proof/work charging.

`make qualify-certified-issuer` runs eight cases with real BLS-signed certificates
assembled outside Auth, including two subnet-range layouts and adversarial
controls. See [fixture provenance and scope](../../qualification/ic-auth-certified-conformance/README.md).
The paired `make lint-certified-qualification` checks the fixture consumer.
Protocol lint/docs/Wasm commands include this component alongside Ed25519.
Wasm compilation is not installed-canister or lifecycle qualification.

The optional features are separate: codec-only Auth selects neither verifier;
certified-only Auth selects no Ed25519; Ed25519-only Auth selects no BLS. Fixture
CBOR encoders and subnet-range builders remain qualification-only. No Canic,
Toko runtime, storage or executor dependency is introduced. Canic continues to
own endpoint wiring using IC Wire.
