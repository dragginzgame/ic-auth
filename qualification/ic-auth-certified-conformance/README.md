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
of issuer grants. This package does not yet qualify the latter, full certified
credential admission, aggregate proof/work limits, sessions or storage.
