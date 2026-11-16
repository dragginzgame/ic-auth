# Ed25519 verification boundary

Enable `ic-auth/ed25519-verification` to use `verification::verify_ed25519`.
Platform 0.34.5 owns the Ed25519 primitives, key parsing, weak-point rejection and
strict signature verification. Platform Types derives the self-authenticating
Principal from the standard Ed25519 DER key. Auth owns canonical claims, protected
context policy, the explicit canonical-scalar admission and attempt charging.
No cryptographic algorithm, signer custody or executor is implemented here.

The function decodes once and hashes the exact accepted claims bytes, without
re-encoding them. It verifies ordinary strict Ed25519 over that SHA-256 digest.
It never selects a trusted key or mechanism from untrusted proof fields.
`Ed25519Key` admits key shape and retains its SHA-256 binding; the host still
owns enrollment, key validity/revocation and the transaction snapshot.

## Protected inputs and refusal order

`VerificationContext` contains the actual caller and clock, enrolled audience,
issuer/mechanism/key binding, checked grants, authority/enrollment deadlines,
reset floors and maximum login/proof lifetimes. Public construction cannot attest
their provenance. Supply them from the host, never from request claims alone.

The common policy preserves strict role-wise grant inclusion, positive login and
proof durations, 90-day/60-second ceilings, exclusive expiry and no skew grace.
Protected grants may contain up to 32 scopes in each of 16 roles. Fresh signed
credentials retain their narrower 32-total-entry limit. Both share one local
grant-shape validator; neither list is sorted into validity.

| Stage | Refusal | Budget effect |
| --- | --- | --- |
| Canonical decode | Encoding, shape or ordering | None |
| Protected mechanism/context, bindings, grants, windows/floors | Typed `ContextError` | None |
| Prepared issuer key vs enrolled binding | Key mismatch | None |
| Signature length / canonical S | Malformed / invalid signature | None |
| Trusted-clock refill and weighted allowance | Rollback / exhausted | Rollback preserves state; exhaustion retains observation and refill |
| Platform strict verification | Invalid signature, including small-order R | Attempt remains charged |
| Success | Immutable `VerifiedClaims` | Attempt charged |

Malformed or weak issuer keys refuse at `Ed25519Key::new`, before a key can be
used by the verifier. Context refusal priority follows the pinned reference:
bindings/grants, window consistency, reset floors, current authority, future
issuance, then proof expiry. `NotYetValid` authenticates nothing.

`VerificationBudget` starts empty and refills one work unit per configured interval,
with explicit Ed25519/certified costs and a shared capacity. The host must retain
one receiver-wide instance across callers and mechanisms. Fractional refill time,
exhausted-attempt observations and backward-clock refusal remain intact. Charges
survive ordinary verification/storage errors; an enclosing IC trap can roll back
the message. No persistence/restore format or issuer bucket is supplied here.

`VerifiedClaims` cannot be forged from decoded claims or mutated through its API.
It is proof evidence at one snapshot, never a session, replay decision, enrollment
approval or permission to perform an application operation. The receiver still
needs current enrollment generation, revocation/replay, lease narrowing and atomic
storage under the same protected snapshot. Timestamp floors do not recover lost
revocation state.

## Evidence

`make test-verification` runs eight integration cases, one weighted-budget sequence
and four compile-fail API cases. Coverage includes the retained OpenSSL signature,
exact digest/caller bytes, wrong host bindings, separate role scopes, broader
protected grants, zero/excessive limits, reset floors, exclusive time boundaries,
overflow/underflow windows, malformed claims/signatures, small-order key/R,
noncanonical S, changed signed fields and raw-claims signatures. Preflight tests
assert no budget mutation; charged failures assert retained work consumption.
The existing nine claims tests also pass after sharing grant-shape validation.
See [fixture provenance](../../crates/ic-auth/tests/fixtures/README.md) for the
repeatable Node/OpenSSL check and its SDK limitations.

The retired legacy-feature activator is removed. The supported Platform graph
still checks the independent noncanonical `S + L` fixture, weak keys, small-order R,
malformed signatures and exact charging behavior in `make test-verification`.

Clippy/Rustdoc with denied warnings and Wasm library compilation pass with the
verification feature. The default Auth graph remains codec-only. Neither the
normal nor Wasm verification graph selects Canic, Toko runtime, storage, certified
verification or another runtime executor.

The separate [certified issuer component](certified-issuer-boundary.md) now
authenticates canister signatures and certificate time. The
[root verifier](root-delegation-boundary.md) authenticates issuer grants under
protected enrollment. The [complete certified verifier](certified-verification-boundary.md)
now shares common context, budget and immutable claims evidence with Ed25519.
Receiver/session policy, both backends, actual Wasm lifecycle and
consumer cutover remain separate qualification obligations.

## Protected credential dispatch

`verification::verify_credential` accepts the shared passive `Credential` and a
host-selected `VerificationProfile`. Its Ed25519 variant borrows protected claims
context and the admitted issuer key; its certified variant borrows the full
protected root/issuer/network context. The submitted proof tag only has permission
to agree with that choice. A mismatch returns `CredentialError::ProofMismatch`
before claims decoding or budget observation, even when claims are malformed.

Matching variants call the existing complete verifier once, borrowing claims and
proof buffers without re-encoding or copying them. `CredentialError` preserves the
original typed mechanism error and its preflight/charged stage. The result remains
`VerifiedClaims`, not a session or write token. There is no verification callback,
Canic dependency or endpoint installation; the host still owns snapshot provenance.

`make test-credential-profile` checks Ed25519-only and certified-only graphs using
the existing independent signatures. Combined-feature Wire qualification now
uses the same entry point for both mechanisms, including malformed signatures,
certificate-binding refusals and charged root failures. The profile mismatch case
also checks both directions with malformed claims in the combined graph.
