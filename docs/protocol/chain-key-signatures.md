# Pure chain-key signature checks

Published **0.4.7** includes `ic_auth::chain_key_signature` under the independent
`chain-key-verification` feature and explicit producer normalization.
[Auth #29](https://github.com/dragginzgame/ic-auth/issues/29) and
[Auth #31](https://github.com/dragginzgame/ic-auth/issues/31) own producer qualification;
[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns consumer adoption.
Both 0.4.7 library publications are checksum-verified; the earlier incomplete
0.4.6 delivery does not block this published selection.

`validate_secp256k1_public_key(&[u8])` admits SEC1 key material using K256's
existing parser, including compressed and uncompressed keys. Admission does
not enroll a Root key or establish its authority.

`verify_secp256k1_prehash(&[u8], &[u8; 32], &[u8])` checks a compact 64-byte
`r || s` signature over the exact supplied hash. It does not hash a message,
normalize a signature, choose an algorithm or trust a key from a response.
The consumer supplies the original hash retained for its signing operation and
the independently selected key. DER and recoverable signatures are refused.

| `ChainKeySignatureError` | Rejection |
| --- | --- |
| `MalformedPublicKey` | K256 cannot admit the supplied SEC1 key |
| `MalformedSignature` | Signature encoding or scalars are invalid |
| `HighSSignature` | A parsed signature has high S; it is not normalized |
| `InvalidSignature` | The admitted key and low-S signature fail exact prehash verification |

This table also gives refusal precedence: key parsing, signature parsing,
high-S detection, then cryptographic verification. Inputs are borrowed and
unchanged. Signature validity alone is not proof of enrollment, management-call
provenance, issuer approval, wallet identity or application authorization.

## Explicit producer normalization (released 0.4.7)

`normalize_secp256k1_signature(&[u8]) -> Result<[u8; 64], ChainKeySignatureError>`
uses K256 to parse compact signature bytes and return their low-S representation.
Already-low-S bytes remain identical; repeated normalization is idempotent.
Malformed encodings/scalars return `MalformedSignature`. The input remains
unchanged.

Normalization has no public-key or hash input and cannot authenticate a
signature. A parseable signature for the wrong hash or key can normalize
successfully and still fail verification. The host owns the decision to
normalize a management signing reply and must verify the returned low-S bytes
against its independently selected key and retained signing hash.

Neither the isolated verifier nor complete proof verification calls this
function implicitly. Both continue to reject submitted high-S input. Do not
normalize untrusted proofs as an acceptance shortcut.

## Complete verification and qualification

The complete [Root delegation verifier](delegation.md) reuses the same parsed-material
boundary after its existing protected policy, binding, time and witness checks.
It preserves `RootSignatureInvalid` for malformed/invalid material and
`HighSSignature` for high-S signatures. Parsing remains before canonical header
hash evaluation; signed bytes and existing rejection ordering are unchanged.

`certified-proof-verification` enables both `chain-key-verification` and
`canister-signature-verification`. The isolated chain-key feature adds K256
without certificate verification, CDK, storage or host tooling. It performs no
management call, signing, clock acquisition or runtime write. Default features
remain empty.

`make test-chain-key-signatures` exercises real keys/signatures, compressed and
uncompressed controls, changed hash/key/signature, malformed encodings/scalars,
high-S refusal, explicit normalization/idempotence and rejection precedence.
`make test-certified-proofs` retains complete
portable certified proofs and standalone Root rejection checks. Native/Wasm,
isolated package MSRV and dependency-boundary checks include the new capability.

With published 0.4.7, Canic can forward its existing ECDSA capability to
`ic-auth/chain-key-verification`, replace its isolated signing-response verifier
and configured-key admission with these functions, and retain its unsupported
algorithm/feature-disabled refusals and endpoint error projection. Audit all
remaining production and test K256 uses before removing its direct dependency;
the reviewed signing adapter explicitly normalizes management replies before
verification. It can replace that conversion with the
new normalizer, projecting the returned array to its response vector, while
retaining the same producer policy. Its configuration validation also parses
keys and can use this admission function. If no production K256 use remains,
move the direct dependency needed for signing fixtures to test infrastructure.
K256 remains transitive. No consumer change or deployment is implied here.
