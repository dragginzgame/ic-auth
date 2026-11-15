# Wire boundary

The 0.2.0 development implementation supplies all four contracts in `ic-auth-wire`.
The published Wire codec owns framing, Serde/Postcard and resource admission.
Transport owns no endpoint, authenticated caller, authority enrollment, session
state or storage. Canonical signed claims remain opaque bytes. Scope text remains
unnormalized, including duplicates and grammatically invalid text; semantic
admission is separate.

| Contract | Fixed ID | Request allowances | Response allowances |
| --- | --- | --- | --- |
| `credential` | `0x415554480001` | 524288 / 2097152 / 65536 / 32 | 524288 / 2097152 / 65536 / 32 |
| `establish_portable_session` | `0x415554480012` | 524288 / 2097152 / 65536 / 32 | 4096 / 16384 / 128 / 8 |
| `portable_session_read` | `0x415554480013` | 11 / 128 / 8 / 4 | 4096 / 16384 / 128 / 8 |
| `logout_portable_session` | `0x415554480014` | 11 / 128 / 8 / 4 | 4096 / 16384 / 128 / 8 |

Allowances are bytes / allocation / nodes / depth. The browser credential cap
still defaults to 131072 bytes, configurable up to 1048576; the codec's lower
allowances continue to apply. Browser logout replies retain their separate 1024
byte cap. Individually legal field sizes can exceed aggregate allocation when
combined. Neither browser retention nor field limits waive that refusal.

The nested certified proof retains root delegation, chain-key batch header/leaf,
left/right witness tags, root signature and issuer canister-signature layouts.
Ed25519 remains tag one with at most 64 bytes; exact signature length and validity
are verifier obligations. The existing record/variant declarations are protocol
constraints; the narrow forwarding implementation, generator adapter and tests
are newly written. This is source-informed reimplementation, not a clean-room
claim. No legacy generic projection framework or old runtime is imported.

## Evidence and commands

`make test-wire` runs 14 product integration cases, including 37 fixed frames,
all reply variants, nested proof order, wrong identities/directions, truncation,
trailing material, invalid tags/principals/u64 and oversize inputs. Claims,
signatures, requested-scope counts and scope text lengths have below/at/above
field tests. Large certified material proves aggregate allocation refusal and
the separate browser retention policy. Redundant outer Postcard varints retain
reference acceptance; signed claims have their own strict codec.

`make qualify-wire` runs the independent qualification package using published
Wire 0.24.1. Its 296 fixed thresholds were measured from reference Wire 0.22.2
before removing that dependency, separately for encoding/decoding and all four
allowances on the 37 frames. It asserts below/at/above acceptance against these
retained expectations, not a second invocation of the current codec. Large
certified material also exercises successful admission and aggregate-allocation
refusal. See [threshold provenance](../../qualification/ic-auth-wire-conformance/tests/fixtures/README.md).
This tests codec supplier compatibility for the selected schemas and
corpus, not an exhaustive proof over every possible input or the old Auth engine.

`make qualify-wire-browser` generates all four modules and the matching supplier
runtime into `qualification/target/wire-browser`, then compares them with the
exact reference browser code and 37 retained frames. Node 24.21.0 and the pinned
Git objects are required; override `AUTH_REFERENCE_REPO` for another reference
checkout. The script authenticates old sources against `reference.sha256` and
runs them in disposable directories. Decoded byte fields must remain detached
after mutation of their input subarrays. Public method names, update-mode
metadata and retention constants remain compatible. Generated source text may
differ with the new supplier; bytes, public value shapes and tested behavior agree.

The optional `typescript` feature supplies native-only Rust metadata derives.
The generator adds the existing public convenience exports around supplier output;
it does not edit generated schema/runtime internals. Generation subtracts the
11-byte frame header from the new supplier's payload-only byte allowance input.
No browser runtime or metadata generator is selected in the default product graph;
Wasm excludes the host metadata even when the feature is forwarded.

See [fixture provenance](../../crates/ic-auth-wire/tests/fixtures/README.md).
The generated modules also passed TypeScript 7.0.2 with `--strict --noEmit`,
`--target ES2022 --module NodeNext --moduleResolution NodeNext`, using the
already-installed compiler in the read-only reference client's `node_modules`.
The browser Make target itself performs Node codec execution, not static checking;
no TypeScript installer or frontend package is introduced here.
Neither check establishes SDK-certified transport or actual browser lifecycle.
Consumer API inventory, real browser/SDK use and exhaustive/adversarial proof
acceptance remain open. Native proof admission is covered below.

## Consuming domain projection

`ic_auth_protocol_types::Credential::try_from(wire_credential)` consumes a decoded
`model::PortableCredential` and returns passive owned proof material. Projection
belongs to `ic-auth-wire`; the shared owned records belong to protocol-types.
Auth remains independent of Wire. No extra runtime package, verifier dependency
or endpoint installer is introduced into the transport crate.

The conversion moves claims, signature/CBOR/DER, key/path buffers and label
strings. Bounded lists become domain containers; container allocation may still
occur, so this is not a zero-allocation claim. Domain role/scope grammar is checked
using its existing owners. Signed order, duplicates, empty grants, unsupported
schema values, separate certificate/leaf commitments and malformed signature
lengths remain untrusted verifier inputs. No claims are decoded/re-encoded, no
binding hash is repaired and no submitted key selects trusted enrollment.

The input already has Wire field bounds. Hosts must still decode through the
contract's aggregate allowances and enforce raw-argument bounds. Projection does
not replace the complete verifier's stricter 131072-byte aggregate proof limit;
that admission runs before canonical claims decoding and cryptographic work.
Native producers may construct passive domain records directly, so verifiers
continue to check their own bounds independently of transport.

`RootDelegationProof::from(&root_material)` and
`CertifiedProof::from(&certified_material)` borrow these records without copying
or establishing authority. The host selects the verifier from protected
enrollment, rejects mismatched submitted proof variants, and supplies caller,
clock, keys, budget and transaction snapshot. Canic owns that endpoint wiring.
Unsigned `SessionInput.scopes` and TTL still go through Auth's existing
`SessionRequest::new`; signed grant lists are never normalized with that policy.

`make test-wire-projection` runs five cases against retained browser records:
every certified field, Ed25519/short signatures, grammar refusal in certificate
and leaf, preservation of duplicate/empty grants, and ownership of large buffers
and label strings. Expected projection fields are literal values from the existing
reference frame corpus. The schema and generated browser layouts are unchanged.

`make qualify-wire-admission` runs three qualification host-driver cases. An
independently signed root/issuer proof passes through `EstablishSession` encoding,
decoding, consuming projection and complete verification; unsigned request policy
is evaluated separately. Negative bindings stay uncharged and failed crypto stays
charged. A real independent Ed25519 signature also passes, while 63-byte material
refuses before charging. The drivers install no endpoints or sessions and do not
establish actual Canic/SDK or host transaction behavior.
