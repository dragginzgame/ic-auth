# Certified issuer qualification

This private package constructs real BLS-signed IC certificates with published
Platform 0.33.7 primitives and CBOR fixture encoders. It tests Auth's optional
`certified-issuer-verification` component. No product depends on this package;
fixture signing, certificate encoding and subnet-range construction stay here.

Fixtures use deterministic test scalars 1 and 2, a test canister, a fixture seed,
and literal `\x1bic-auth-portable-credential` followed by 32 bytes of `0x07`.
They never call Auth to obtain the message being signed. Certificate tree paths,
`\x0Dic-state-root` and empty signature leaves follow the IC protocol. This is
independent fixture construction, not an independent cryptographic implementation
or evidence from an installed canister/network.

The retained contract cases come from `dragginzgame/ic-auth` at
`e52a40be75dd9809d0528189d84b43fe96952ad9`:

- `crates/ic-auth/src/portable_certified.rs`: portable signing domain, fixed
  zero future grace and the 131072-byte hard ceiling.
- `crates/ic-auth/src/canister_signature.rs`: protected signer/seed/network,
  short DER bounds and signing-certificate age policy.
- `crates/ic-auth/tests/canister_signature.rs` and `tests/certificates/mod.rs`:
  certificate construction pattern, complete/nonminimal time acceptance,
  subnet range formats, nested-delegation refusal and identity-point forgery.

Their committed bytes are recorded in the root `reference.sha256`. Cases are
re-expressed around the narrower fixed-domain component; no old runtime or
verifier is a dependency. The zero-future-grace profile intentionally differs from
that reference's configurable generic helper.

Run `make qualify-certified-issuer lint-certified-qualification`. Eight cases
cover literal signing bytes, exact host and hard byte limits, protected bindings,
inclusive age/zero future grace/u64 extremes, malformed authenticated time and
BLS-before-time ordering, changed digest/domain/witness/nonempty leaf, truncated
DER/malformed CBOR, identity-point forgery, and both subnet range layouts with
out-of-range and nested-delegation controls.

IC root-to-subnet certificate delegation is distinct from Auth root authorization
of issuer grants. The root cases below qualify that second component separately;
full certified credential admission, aggregate work charging, sessions and storage
remain outside these component tests.

## Root authorization

`tests/root_delegation.rs` adds eight cases for the optional
`root-delegation-verification` component. The frozen `root-delegation.tsv` fixture
is constructed by `tests/verify-root-fixture.mjs` using Node buffers and OpenSSL,
without importing Auth or Platform. It builds the certificate, issuer leaf,
two ordered Merkle siblings and batch header from literal protocol fields, then
signs the header domain/length/payload using SHA-256 and ECDSA/secp256k1.

The fixture scalar is the publicly known test value 1. Both SEC1 encodings are
retained. The compact low-S signature is paired with its equivalent high-S
control; OpenSSL must actually accept both, while Auth refuses high-S. Frozen
signatures are verified without regeneration by default. `--generate` explicitly
replaces the fixture and produces a fresh randomized OpenSSL signature; ordinary
Rust tests never invoke it or generate their expectations from Auth.

Run `make qualify-root-delegation lint-certified-qualification` and optionally
`node qualification/ic-auth-certified-conformance/tests/verify-root-fixture.mjs`.
The pure root component returns immutable certificate evidence. Two compile-fail
cases ensure it cannot be fabricated from a certificate or mutate that certificate.
It has no receiver budget or session effects.

Behavior was re-expressed from the pinned reference's
`crates/ic-auth/src/delegation/{mod,root,rules}.rs`,
`crates/ic-auth/src/chain_key_signature.rs` and
`crates/ic-auth/src/chain_key_batch.rs`, plus the existing canonical byte contract.
Their bytes are recorded in `reference.sha256`. This is a new bounded profile and
borrowed domain API, not a promise to replace every historical helper API. No old
runtime, general tree builder or signing-custody implementation is imported.
These root-only cases establish neither complete credential verification nor
shared-budget behavior; those are covered by the composition cases below.
Installed-host qualification remains outstanding.

## Complete certified proofs

`tests/certified.rs` composes the real root and issuer cryptographic boundaries
with common claims policy and shared-budget charging. Its positive
`fixtures/certified.tsv` is generated independently by the same OpenSSL script's
`--certified` profile: it uses an actual seed digest, canonical schema-2 claims,
caller DER/SHA-224 identity and independently signed ECDSA/Ed25519 digests.
The script's default check verifies frozen signatures; `--generate --certified`
explicitly replaces only this profile's fixture. Run the non-generating check:

```sh
node qualification/ic-auth-certified-conformance/tests/verify-root-fixture.mjs --certified
```

The Rust issuer fixture signs the frozen claims digest and literal portable domain
using Platform's BLS fixture primitive. Positive expected claims/signatures never
come from Auth encoding. Negative policy tests may decode and modify retained
claims to isolate rejection stages, without treating those unsigned mutations as
valid crypto fixtures. `support/root.rs` shares only reconstruction of retained
root fixture data between root and complete-proof tests.

Run `make qualify-certified qualify-shared-verification`. Seven complete-proof
cases cover acceptance, exact budgets, uncharged policy/binding refusals, retained
charges for root/issuer failures, changed claims, freshness and rollback. The
optional `ed25519-comparison` qualification feature adds one real cross-mechanism
budget case; it does not change product feature defaults. No fixture installs
sessions, consumes replay state or qualifies an actual host transaction.

The full orchestration preserves the checks and charging boundary in the pinned
reference `crates/ic-auth/src/portable_certified.rs`, already in `reference.sha256`.
Its public API is newly organized around borrowed proof views and common protected
context. Complete proof verification is now implemented; receiver/session and
installed-host replacement qualification remain outstanding.

## Wire/domain integration

`make test-receiver-policy` additionally selects the `receiver_retry` driver:
independently verified certified/Ed25519 evidence is projected into borrowed
retained facts, then assessed after proof expiry and at lease expiry. This is
native policy integration, not a backend admission/commit or lifecycle test.
The [receiver boundary](../../docs/development/receiver-policy-boundary.md)
records the host and storage obligations that remain.

`make qualify-wire-admission` selects three `wire_` cases in `tests/certified.rs`.
These qualification-only host drivers build Wire records from the independent
crypto fixtures, decode with the existing contracts, consume them into passive
domain records and explicitly invoke the protected verifier. They also route
unsigned scope/TTL input through `SessionRequest`. Binding/crypto failure charging
and short Ed25519 refusal remain observable across this boundary.

Wire encoding/decoding and projection are not independent oracles for each other:
`ic-auth-wire/tests/projection.rs` separately compares every projected field to
literal retained browser-fixture values. Crypto acceptance uses the independent
signatures described above. The Wire crate gains no Auth/crypto dependency and
Auth gains no Wire dependency; both share protocol-types. No endpoint installation
or receiver state is part of these drivers.
