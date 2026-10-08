# IC canister-signature verification

The working tree adds `ic-auth`'s optional `canister-signature-verification`
feature and `ic_auth::canister_signature`. This API is not in the published
`0.1.2` packages. Encoding-only consumers leave the feature disabled.

`verify_canister_signature(message, proof, policy)` verifies the exact message
against `IcCanisterSignatureProofV1`. The host constructs
`CanisterSignaturePolicy` independently of the request:

| Protected input | Meaning |
| --- | --- |
| `signing_canister` | Approved signer principal |
| `seed_hash` | Approved SHA-256 seed binding, from protected configuration or an already authenticated delegation certificate |
| `ic_root_public_key_raw` | Network trust anchor as 96 raw BLS key bytes |
| `now_ns` | Trusted host clock, Unix epoch nanoseconds |
| `max_certificate_age_ns`, `max_future_skew_ns` | Inclusive signing-certificate freshness limits |
| `max_signature_bytes`, `max_message_bytes` | Host-selected finite input limits |

Never fill the expected signer, seed hash or trust anchor from the submitted
proof. A client cannot enroll a signing key or network by presenting a proof.
Read protected policy again after configuration or authority changes; this
operation has no cache or persisted success state.

The verifier checks input sizes, DER integrity and protected signer/seed binding
before invoking [DFINITY's verifier](https://docs.rs/ic-signature-verification/0.3.0).
That owner checks the certified-data digest, exact signature-tree message path,
empty signature leaf, BLS signature, subnet key delegation and signing-canister
range. Only after those checks does IC Auth interpret the authenticated `/time`
leaf and enforce the host's freshness window. Time decoding rejects missing,
truncated, overflowing or trailing data. Differences avoid arithmetic overflow
at the `u64` clock boundaries. The older root certificate in a subnet delegation
is authenticated but is not subject to the leaf certificate's freshness window.

The supported key format is DFINITY's short-form DER encoding, 20–129 bytes.
The upstream parser is guarded against short inputs; re-encoding must reproduce
the exact DER bytes, including lengths, tags and unused-bit padding. Long-form
DER keys are rejected. Both existing Canic seeds fit this format. CBOR decoding
keeps the upstream decoder's recursion limit and occurs only after the host's
signature byte limit is checked. Hosts still bound their outer request decoding;
passive protocol vectors do not provide that admission boundary.

`domain_separated_message(domain, payload_hash)` constructs the existing IC
one-byte domain length, domain bytes and 32-byte payload hash. It rejects domains
longer than 255 bytes. Canic adapters retain their domain selection:
`canic-issuer-delegated-token` for issuer proofs and
`canic-root-role-attestation` for role attestations. The payload's canonical hash
must use its existing encoder. These messages are distinct from standard IC
ingress delegation messages and wallet challenges.

A successful result authenticates only the exact message under the selected
trust context. It does not check application-token validity, audience, subject,
presenter, grants, scope, epochs, revocation or resource ownership, and does not
consume a replay record or admit a session. Complete token/session verification
remains required before an application grants access.

This module has no runtime effects: no clock acquisition, memory-region
allocation, management calls, signing, certification-root publication or global
state. The upstream key package brings `ic0` transitively; its signing map is
not used. `ic-host-*` remains in the native tooling application, and the selected
verification graph contains no Canic, Toko, Solana, SIWS, CDK or storage backend.

`make test-signatures` exercises real BLS-signed certificates with deterministic
fixture keys, both upstream subnet-range formats and explicit rejection cases.
`make check-wasm` compiles default and verification-selected graphs, and
`make check-boundaries` inspects both. These checks do not qualify real IC
ingress delegation, service lifecycle, native macOS execution or Canic adoption.
The accepted [extraction design](design/extraction.md) and
[Canic #491](https://github.com/dragginzgame/canic/issues/491) retain that scope.
