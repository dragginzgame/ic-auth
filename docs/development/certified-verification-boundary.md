# Complete certified proof verification

Enable `ic-auth/certified-verification` and call `verification::verify_certified`.
It verifies a root-authorized delegation and an issuer IC canister signature over
canonical claims, using explicit protected host context. Success returns the same
immutable `VerifiedClaims` as Ed25519. It is proof evidence at one snapshot, not
a session, replay decision, installed enrollment or application authorization.

## Cryptographic compatibility

The cryptographic schemes and signed formats are preserved from the pinned
original Auth implementation:

| Proof | Preserved operation | Primitive owner |
| --- | --- | --- |
| Ed25519 | Ordinary strict Ed25519 over SHA-256 of exact canonical claims; canonical S and weak-point refusal | Platform's Ed25519 facade |
| Root batch | Compact low-S secp256k1 ECDSA over the canonical domain/length/header SHA-256 digest | Platform's ECDSA facade |
| Issuer | IC canister signature over `0x1b || ic-auth-portable-credential || claims_digest`, with authenticated certificate/witness and network BLS root | Platform's IC signature verifier |
| Commitments | Existing schema-2 bytes, certificate/leaf/header domains, issuer seed/path hashes and ordered Merkle node framing | Auth protocol framing, published SHA-256 |

This rewrite changes orchestration and ownership while retaining those contracts.
It introduces no cryptographic primitive or alternate runtime. Retained reference
hash/byte fixtures, independent OpenSSL root/Ed25519 signatures and Platform-built
real BLS certificates check the implemented boundaries. They are not proof of
installed network behavior or exhaustive compatibility across all old consumers.

## Admission and retained work

`CertifiedContext` combines the common protected `VerificationContext`, enrolled
`RootKeyPolicy`, raw network root and explicit limits. The host must supply one
current transaction snapshot. Submitted algorithms, keys, labels and times never
select their own trusted enrollment.

| Stage | Refusal | Budget effect |
| --- | --- | --- |
| Protected mechanism | Wrong verifier selection | None |
| Complete variable material, issuer CBOR and witness caps | `InputTooLarge` | None |
| Canonical claims and shared caller/audience/grant/window/reset checks | Encoding/context | None |
| Certificate hash and claims/enrollment binding | Encoding/binding | None |
| Shared receiver-wide allowance | Exhausted/rollback | Retain normal refill/observation rules; rollback changes nothing |
| Root authorization, low-S ECDSA and leaf inclusion | Root | One certified cost retained |
| Issuer binding, BLS certificate and authenticated freshness | Issuer | Same single cost retained |
| Both succeed | Immutable `VerifiedClaims` | Same single cost retained |

The aggregate byte allowance is at most 131072 and includes encoded claims,
issuer CBOR/DER and root variable material. Root grants count twice where carried
twice; entry and witness accounting is shared with standalone root verification.
Independent signature-size and witness-step limits still apply. Outer transport
decoding remains host-bounded; this verifier accepts borrowed decoded material.

Before charging, the certificate hash must match the signed/enrolled key binding;
issuer and audience must agree; claims grants must be a role-wise subset of the
certificate; proof time must fit certificate lifetime and delegated token TTL;
enrolled authority must fit certificate and root-key expiry. The common context
owner validates claims once for both mechanisms. Canonical claims are hashed in
their exact received form. A shared private root-verification core avoids computing
the certificate commitment twice.

The charge occurs before root authorization, including its policy and malformed
signature refusals, matching the reference certified path. An issuer failure does
not refund root work or charge a second attempt. The host must retain one budget
across calls and mechanisms; recreating it, trapping ordinary refusals or losing
revocation state does not preserve the receiver contract.

## Evidence and scope

`make qualify-certified` runs seven core complete-proof cases, two Wire admission
cases and the five API
compile-fail cases available in a certified-only product graph. Positive claims
and root signatures are assembled independently with Node/OpenSSL; issuer BLS
fixtures sign the retained claims digest and literal domain without calling Auth's
encoder or message builder. Cases cover exact aggregate boundaries, separate caps,
preflight no-mutation, certificate narrowing, root-before-issuer order, charged
refusals, digest mutation, freshness and budget exhaustion/rollback.
It also checks captured session lease policy. With `receiver-memory`, qualification
uses actual admission/retry/replacement instead of assembling public policy facts;
see the [receiver coverage mapping](receiver-policy-boundary.md#coverage-after-api-narrowing).

`make qualify-shared-verification` enables the qualification package's optional
Ed25519 comparison and passes independently signed Ed25519 and certified proofs
through the same five-unit bucket: costs three and two exhaust it exactly.
The default certified qualification consumer has no Ed25519 verifier dependency.
Existing root, issuer and Ed25519 cases remain in their focused targets.

Product graphs remain selectable: codec-only has neither verifier; certified-only
has ECDSA/BLS without Ed25519; Ed25519-only has no ECDSA/BLS. Lint, docs and native/
Wasm compilation cover combined features. No Canic, storage, Toko runtime or
executor is introduced. Canic continues to own endpoint wiring using IC Wire.

Consuming Wire-to-domain projection and native host-driver verification are covered
by [Wire qualification](wire-boundary.md#consuming-domain-projection).
Receiver/session policy, replay/revocation, atomic
volatile/stable backends, actual host/SDK lifecycle and consumer adoption remain
open. The successful complete-proof cases do not establish those replacement gates.
