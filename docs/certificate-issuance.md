# Pure certificate issuance validation

Pending 0.4.7 provides the default-feature API
`ic_auth::certificate_issuance::validate_certificate_issuance(cert, expected_root, limits)`
for [Auth #30](https://github.com/dragginzgame/ic-auth/issues/30). It accepts the
passive `DelegationCert` contract and returns `Result<(), CertificateIssuanceError>`.
There is no clock, proof, signing callback, storage or runtime input. A successful
shape check does not authorize signing, enroll an issuer or install a delegation.

Supply `expected_root` from protected configuration, separately from the
certificate, and explicit `CertificateIssuanceLimits` with certificate TTL,
delegated token TTL, grant count and scopes-per-grant bounds. There are no implicit
Fleet defaults. Exact equality is accepted; zero bounds admit no corresponding
positive lifetime or nonempty grant collection. Hosts bound outer decoding before
constructing these passive values.

The rejection order is fixed:

1. Root mismatch (`RootPidMismatch`, including expected/found principals).
2. Validity starts before issuance, reversed expiry or zero duration
   (`InvalidCertWindow`). Duration is expiry minus validity start, not issue time.
3. Certificate duration above its host bound (`CertTtlExceeded`).
4. Zero delegated token TTL (`TokenTtlZero`), then above the host token bound
   (`TokenTtlExceeded`), then above certificate duration (`TokenTtlOutlivesCert`).
5. Empty or oversized grants (`GrantsEmpty`, `TooManyGrants`). For each grant,
   noncanonical role order, empty or oversized scopes, then scope grammar and
   strictly sorted unique scopes. Cardinality causes carry actual/maximum counts
   and affected roles. Canonical errors preserve the existing typed causes.
6. Recomputed issuer-proof binding hash mismatch (`IssuerProofBindingHashMismatch`).

`AuthRole` already enforces role grammar at construction and decoding. A Canic DTO
projection must preserve its existing invalid-role error there; do not manufacture
an unchecked library role. No role label supplies application authority. Scope
validation and hashing reuse the canonical encoding owner. Validation borrows the
certificate and does not sort, normalize or change any signed field.

Canic's current raw DTO can contain malformed role labels; its eager `protocol::cert`
projection checks every role before this passive validator can run. That projection
alone cannot preserve the old raw DTO precedence for a malformed later role combined
with an earlier Root, lifetime, grant-count or scope failure. Consumer adoption must
resolve and test this boundary before claiming equivalent refusals or retiring all
local rules. Do not bypass `AuthRole` validation or manufacture substitute signed
fields to hide a projection error. The producer's ordered contract covers validly
constructed passive roles; raw DTO admission remains a consumer acceptance gap.

Anonymous principals, current-clock acceptance, issuer approval, enrolled Root
policy and certificate freshness are deliberately separate obligations. Complete
[delegation/token verification](tokens.md) retains these stronger checks. It reuses
the lifetime and issuer-binding primitives while preserving its existing refusal
order and combined minimum token-TTL diagnostic. [Token preparation](token-preparation.md)
reuses grant validation while retaining its own errors, authority deadline,
request order, deterministic nonce and exact signed claims. Preparation continues
to assume a host-admitted proof; it does not implicitly run the new issuance API.

After publication, Canic can replace the mechanics of `delegated/cert_rules.rs`
with this call, supplying its resolved Root and limits (currently 16 grants and
32 scopes) and projecting typed errors at its DTO boundary. Keep protected Fleet
and role policy resolution, issuer approval, Root signing, management effects,
storage, endpoint authorization and current-clock checks in Canic. Construction
and pure token-material callers must both use the same adopted owner before
retiring the old rules. [Canic #491](https://github.com/dragginzgame/canic/issues/491)
owns consumer adoption; this source implementation does not establish registry
delivery or Canic runtime acceptance.
