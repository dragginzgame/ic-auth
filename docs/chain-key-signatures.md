# Pure chain-key signature checks

Pending **0.4.6** adds `ic_auth::chain_key_signature` under the independent
`chain-key-verification` feature. [Auth #29](https://github.com/dragginzgame/ic-auth/issues/29)
owns producer qualification; [Canic #491](https://github.com/dragginzgame/canic/issues/491)
owns consumer adoption. This API is implemented in the working tree and is not
yet a published dependency selection.

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

The complete [Root/token verifier](tokens.md) reuses the same parsed-material
boundary after its existing protected policy, binding, time and witness checks.
It preserves `RootSignatureInvalid` for malformed/invalid material and
`HighSSignature` for high-S signatures. Parsing remains before canonical header
hash evaluation; signed bytes and existing rejection ordering are unchanged.

`token-verification` enables both `chain-key-verification` and
`canister-signature-verification`. The isolated chain-key feature adds K256
without certificate verification, CDK, storage or host tooling. It performs no
management call, signing, clock acquisition or runtime write. Default features
remain empty.

`make test-chain-key-signatures` exercises real keys/signatures, compressed and
uncompressed controls, changed hash/key/signature, malformed encodings/scalars,
high-S refusal and rejection precedence. `make test-tokens` retains complete
proof controls and shared standalone/token rejection checks. Native/Wasm,
isolated package MSRV and dependency-boundary checks include the new capability.

After publication, Canic can forward its existing ECDSA capability to
`ic-auth/chain-key-verification`, replace its isolated signing-response verifier
and configured-key admission with these functions, and retain its unsupported
algorithm/feature-disabled refusals and endpoint error projection. Audit all
remaining production and test K256 uses before removing its direct dependency;
the reviewed signing adapter explicitly normalizes management replies before
verification, so that use still needs K256. Its configuration validation also
parses keys and can use this admission function. This API does not change the
consumer's explicit normalization policy. K256 remains transitive. No consumer
change or deployment is implied here.
