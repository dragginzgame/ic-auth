# Delegated-token preparation

Released **0.4.2** adds `ic_auth::token_preparation` with the default library
selection, including `default-features = false`. Both packages are published.
[Auth #25](https://github.com/dragginzgame/ic-auth/issues/25) requests
the extraction of Canic's pure issuance helper; the
[Canic adoption contract](canic-adoption.md) describes consumer ownership.

`prepare_delegated_token(PrepareDelegatedTokenInput)` receives an already
installed, host-admitted `DelegationProof`, the independently protected authority
deadline, authenticated presenter, retained 32-byte operation ID, host clock and
requested audience, grants, TTL and extension. `TokenPreparationLimits` supplies
explicit host-selected grant and per-grant scope bounds; there are no fleet
defaults. The presenter is also the subject. A submitted certificate or a
successful preparation result cannot establish authority to issue a token.

The request rejects in this order:

1. Certificate not-before and the exclusive minimum of certificate expiry and
   protected authority expiry.
2. Zero TTL, certificate maximum TTL, checked expiry arithmetic, then expiry
   beyond either deadline. Expiry equal to the deadline is permitted; preparation
   at that deadline is refused. An overlong request rejects without shortening
   its requested TTL or rewriting the certificate.
3. Empty or excessive grants, then each grant's strictly increasing role,
   nonempty/bounded scopes and canonical scope labels/order.
4. Audience narrowing, then role/scope narrowing against the admitted certificate.
5. Canonical certificate and claims encoding, including the 4096-byte extension
   bound. An absent extension and a present empty extension remain distinct.

`PrepareDelegatedTokenError` preserves typed causes. Host shape bounds have their
own variants; role/scope canonical refusals use `CanonicalAuthError`. Applications
project these causes into their endpoint errors without reimplementing validation.
Pending 0.4.7 shares the grant primitive with
[certificate issuance validation](certificate-issuance.md), preserving this request
order and public causes. It does not implicitly revalidate the host-admitted proof
as a new certificate issuance.

The immutable `PreparedDelegatedToken` exposes `claims()`, `claims_hash()` and
`proof()`. It retains the original proof and the exact canonical claims/hash.
The nonce is the first 16 bytes of SHA-256 over this unchanged concatenation:

```text
canic-token-nonce-v1 || presenter.as_slice() || operation_id[32]
                   || issuer.as_slice() || canonical_cert_hash[32]
```

Principal bytes have no length prefixes in this nonce domain. Claim encoding
retains its existing domain, lengths and signed extension presence. Fixed nonce,
full claims-byte and hash vectors exercise this contract independently of the
production encoder.

The host retains the original operation inputs, including clock and installed
proof, or the prepared result across retries. Reusing an operation ID with a new
clock can preserve the nonce while changing signed claims. This function does not
provide idempotency storage or permission to repeat an uncertain signing effect.

After authorized issuer signing/retrieval, call
`finish_delegated_token(prepared, issuer_proof)`. It consumes the prepared value
and attaches host-provided material verbatim. It does not authenticate that
material. Recipients still require complete [token verification](tokens.md).
Root enrollment, issuer approval, active-proof resolution, signing authorization,
operation custody, runtime clock, certification-root publication and application
authorization remain with the host. Neither function reads storage or runtime
state, allocates stable memory, publishes a certification root or creates a
session.

`make test-protocol` covers the byte contract, deterministic retained retries,
exclusive deadlines, overflow and refusal precedence without signing features.
The first [Testkit token journey](ic-testkit-qualification.md) uses preparation
and finishing with an actual certified issuer, then verification and upgrade.
This reference journey does not qualify a deployed issuer endpoint or Canic's
adapter.
