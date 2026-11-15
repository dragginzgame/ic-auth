# Root-delegation verification

Enable `ic-auth/root-delegation-verification` for `verify_root_delegation`.
The component authenticates issuer authorization in a root-signed ECDSA/secp256k1
batch under independently enrolled root-key policy and an explicit host clock.
It returns an immutable borrowed `VerifiedDelegation`, not a portable credential,
installed enrollment, session or permission to perform an application operation.

The host supplies the expected issuer, root canister, approved key name/path hash,
exact SEC1 key bytes, key version, live version/proof/registry floors, enrollment
window and revocation-latency limit. Submitted key material cannot establish any
of those inputs. Policy and floors are rechecked on every call. Compressed and
uncompressed SEC1 keys are both supported, but equivalent points do not override
exact enrolled byte identity.

The bounded current profile supports IC canister-signature issuer binding, Fleet
audience and ECDSA/secp256k1 root signing. It has zero future grace. Signature and
canonical protocol owners remain Platform and `delegation_encoding`; Auth only
assembles ordered SHA-256 Merkle nodes as `0x01 || left || right`. There is no
local ECDSA implementation, tree builder, signer service or executor.

Admission proceeds through these boundaries:

1. Variable-material accounting and field/witness caps, before canonical hashing
   or signature work. Both copies of grants count; entries, labels, siblings,
   key names, path components, public key and signature retain reference costs.
2. Nonanonymous root/issuer agreement with protected selection; consistent issue
   time, certificate/token lifetime and exclusive current validity.
3. Strict shared grant shape/order, issuer/algorithm/seed commitment and exact
   certificate binding, without repairing submitted fields.
4. Schema, root identity, root enrollment, batch/leaf windows, leaf containment,
   epochs/registry commitment and full certificate-to-leaf authorization equality.
5. Submitted signature metadata vs header, then protected key/path/version
   enrollment, live epoch/version floors and maximum batch lifetime.
6. Ordered leaf inclusion, Platform SEC1/compact-signature parsing, explicit
   high-S refusal, then ECDSA verification over the exact canonical header digest.

Variable material is capped at the smaller of the host allowance and 131072;
witness steps at the smaller of the host limit and 64. Key/path/signature field
caps follow the current transport profile. These are delegation-only bounds.
The [complete certified verifier](certified-verification-boundary.md) additionally
counts claims and issuer proof material and charges the shared weighted budget
before invoking the common root-verification core. It prepares the certificate
commitment once before charging; standalone root verification hashes its result
after authorization. This standalone operation has no mutable state or budget argument.
Its failures cannot install authorization or consume replay/session state.

The certificate issue time is checked for consistency but absent from the root
leaf signature. Its hash still includes that metadata. The issuer's signature and
claims key binding must complete this chain; root verification alone does not
prove a portable credential. The returned expiry is the smaller of certificate
expiry and current root-key acceptance expiry, with no continuing freshness claim.

`make qualify-root-delegation` runs eight independent fixture cases and two
compile-fail API cases. Tests cover real signatures, live policy, floors, exclusive
windows, bounded lifetime/material/witness work, certificate/leaf binding, key
metadata, ordered Merkle paths, high-S refusal and exact SEC1 enrollment identity.
`make lint-certified-qualification` checks the fixture consumer. Protocol
lint/docs/Wasm checks include the new feature. Optional fixture verification uses
Node/OpenSSL; see [provenance](../../qualification/ic-auth-certified-conformance/README.md#root-authorization).

The root-only product graph selects Platform ECDSA without Ed25519 or BLS.
The issuer-only and codec-only graphs remain separately selectable. Native and
Wasm library compilation do not establish installed-host behavior or session
admission. Aggregate certified-proof charging and claims binding are implemented
by the complete coordinator; receiver/storage policy remains separate work.
