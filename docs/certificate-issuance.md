# Pure certificate issuance validation

Released 0.4.7 provides the default-feature API
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
with an earlier Root, lifetime, grant-count or scope failure. Canic's owner now
reports choosing projection-first refusal as an explicit pre-1.0 hard cut,
retiring its duplicate rules and testing an invalid-role/invalid-lifetime case
with zero Root/issuer proof callbacks. See the [consumer evidence](https://github.com/dragginzgame/ic-auth/issues/30#issuecomment-6120497461).
That choice is locally qualified in its held integration batch; consumer delivery
and complete session-engine acceptance remain separate. Do not bypass `AuthRole`
validation or manufacture substitute signed fields to hide a projection error.
The producer's ordered contract covers validly constructed passive roles.

Anonymous principals, current-clock acceptance, issuer approval, enrolled Root
policy and certificate freshness are deliberately separate obligations. Complete
[delegation/portable verification](delegation.md) retains these stronger checks. It reuses
the lifetime and issuer-binding primitives while preserving its existing refusal
order and combined minimum token-TTL diagnostic. The retired token-issuance path is described in the
[0.9 retirement contract](delegation.md#retired-delegated-token-lifecycle).

Canic can replace the mechanics of `delegated/cert_rules.rs`
with this call, supplying its resolved Root and limits (currently 16 grants and
32 scopes) and projecting typed errors at its DTO boundary. Keep protected Fleet
and role policy resolution, issuer approval, Root signing, management effects,
storage, endpoint authorization and current-clock checks in Canic. Construction
and verification callers must both use the same adopted owner before
retiring the old rules. [Canic #491](https://github.com/dragginzgame/canic/issues/491)
owns consumer adoption; producer qualification and publication do not establish
Canic runtime acceptance.

## Standalone scope and grant shape checks (pending 0.5.11)

`certificate_issuance::validate_grants(grants, max_grants, max_scopes_per_grant)`
exposes the same rules without requiring a certificate, expiry arithmetic or
issuer proof. `GrantValidationError` carries empty/count failures, affected roles
and existing canonical causes. Zero bounds reject nonempty corresponding values;
equality is accepted. The grant-count check precedes per-grant role ordering,
scope count, label grammar and strict scope ordering. Existing issuance and token
preparation keep their error projections and use this same owner.

`canonical::validate_scope_label_detailed` is an allocation-free `const fn`
returning `ScopeLabelError`: `Empty`, `TooLong { bytes, max }`, `EmptySegment`,
`InvalidSegmentStart { offset }` or `InvalidByte { offset }`. It enforces the
existing 64-byte grammar: colon-separated nonempty segments starting with an
ASCII lowercase letter or digit, followed by those bytes, `_` or `-`. Length
wins before left-to-right byte errors; offsets are UTF-8 byte offsets. Static
constructors can match the returned `Result` in const evaluation. The existing
`validate_scope_label` retains `EmptyScope`/`InvalidScope` error projection.
`canonical::validate_scopes` borrows a string slice and checks grammar and strict
unique byte ordering; it permits an empty list and does not impose a count bound.

These are shape checks, never resource grants. Canic retains its namespace and
resource policy, 16/32 bounds, sorting at configuration boundaries and intentionally
empty disabled configuration. Do not invoke bounded nonempty validation where
empty configuration is valid. Keep its signed registry/key-policy envelope and
encoding unchanged while reusing scope ordering. Raw role projection still needs
an explicit consumer refusal contract: eager conversion of a later malformed role
can precede an earlier scope error. No producer API bypasses passive role parsing.
[Auth #50](https://github.com/dragginzgame/ic-auth/issues/50) owns producer work;
[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns consumer adoption
and retirement of duplicate checks.
