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

The former proof-to-facts retry and replacement drivers are superseded by actual
receiver lifecycle qualification. Policy rules now have private unit tests;
qualification consumes only `ic_auth::receiver_memory`. The [coverage mapping](../../docs/development/receiver-policy-boundary.md#coverage-after-api-narrowing)
records the retained guarantees and the denied-slot replacement case.

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

With `--features receiver-memory`, five `memory_certified` cases drive actual
volatile admission, root/issuer failure charging, proof-expired exact retry and
known-login logout. It also checks exact unchanged logout reply frames for known
and unknown results, retries, capacity, invalid caller, clock and closed state.
`make test-memory` runs them in a certified-only graph along
with the separate Ed25519 memory sequences. This is native receiver evidence;
boot/recovery and installed-host rollback remain unqualified.

`tests/memory/mod.rs` adds two issuer-qualified accounts with equal subject IDs,
two devices on one account and two independent receivers. It exercises per-account
quota isolation, root/issuer proof failure without displacement, full-capacity
replacement, stale-device refusal, exact retry after proof/certificate-age expiry,
renewal to the original login deadline and receiver-local known/unknown logout.
Additional cases cover non-extending enrollment refresh, generation invalidation,
outage logout, terminal closure and denial before admission without identity mutation.

`verify-memory-fixture.mjs` independently assembles seven certified claims, digests
and browser identities in `fixtures/certified-memory.tsv`; its initial case must
match the previously frozen certified fixture exactly. The second issuer uses
`fixtures/certified-other-issuer.tsv`, independently signed by the existing root
fixture builder with `--certified --other-issuer`. Both issuers use the public test
keys and seed under distinct authenticated issuer identities. Rust reconstructs
the root structures and BLS-signs the literal claims domain/digest; valid claims
never come from Auth's encoder. Tests traverse the existing Wire envelope and
consuming projection without installing an endpoint. These new fixtures and traces
are authored for this design; no legacy engine code or test implementation is copied.
Normal `make test-memory` and `make verify-fixtures` verify frozen records without
regenerating signatures. This is native certified receiver evidence, not an IC
response-authentication, host transaction, durable-storage or browser SDK result.

## Volatile certified issuance

With `issuer-memory`, `tests/issuance/mod.rs` drives the public issuer API to
the existing frozen `certified.tsv` claims, including slot 1/generation 7. Its
expected claims and hash come from the independent Node/OpenSSL fixture above.
The host callback constructs the standard empty signature leaf from the actual
publication input and composes an application branch using Platform hash trees.
Platform BLS signs certificates for that root at two query times. Both complete
proofs verify under the enrolled root, with different witness/certificate bytes
and the same original credential fingerprint. No fixture or signature-store
implementation is copied from legacy.

`make test-issuer` checks this case and the frozen Node fixture; hosted
`test-qualification` includes it through the optional feature. This supplies
native boundary evidence, not installed IC transaction rollback, production leaf
retention or certificate acquisition. The host must supply those capabilities.
