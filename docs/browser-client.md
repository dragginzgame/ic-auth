# Portable browser session client

The private `packages/client/` package supplies one portable admission coordinator
and durable logout worker for [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
It is not published on npm. A production issuer adapter, wallet login and final
Canic/Toko integration remain consumer/service work.

## Application transport hard cut

Released Auth 0.13 uses `crates/ic-auth-wire/` with published IC Wire 0.5.
Historical Auth 0.10 uses Wire 0.4 and Auth 0.9 uses Wire 0.3; consumers sharing
Rust Wire types must align that dependency identity. Wire 0.5 separates owned
deserialization from `Wire`; generic decoders require an explicit Serde bound.
Auth keeps owned DTOs, existing frame bytes and generated browser contracts; no
browser storage reset or journal migration is needed. See [consumer selection](canic-adoption.md#wire-transport).
Rust field/variant IDs, bounded types and method identities generate both Rust
codecs and TypeScript contracts. Application traffic has no Candid selector or
fallback. Canonical signed claims and session/replay/login records keep their
independent existing bytes. Platform ingress certificates and management protocols
remain distinct from this application codec.

The maintainer confirmed on 2026-10-17 that there are no deployed old token-store
records, including unresolved old issuance. The former `TokenSessionClient`,
prepare/retrieve transport, token caches, memory/IndexedDB token stores and their
format are retired. There is no migration, dual reader or automatic database reset.
This disposition applies to that never-deployed format; it does not permit losing
unknown effects in new portable journals. Canic adoption stays in
[Canic #491](https://github.com/dragginzgame/canic/issues/491).

Generated `credential.ts`, `establish.ts`, `read.ts` and `logout.ts` import the
canonical upstream `wire-runtime.ts`. They are generated artifacts, not handwritten
schemas. The optional Rust `typescript` capability belongs to native tooling,
not the canister dependency selection. `make generate-client-contracts` explicitly
updates artifacts; `make test-client` compares them offline and refuses stale files.

IC Wire 0.3.4 supplies the strict-indexing generator fix and owned bounded buffer
accessors. Generated and authored TypeScript now compile together with strict
indexing and exact optional-property checks; the temporary generated-only project
is removed. Rust projections recover owned byte/text buffers without copying.
[Wire #3](https://github.com/dragginzgame/ic-wire/issues/3) and
[Wire #4](https://github.com/dragginzgame/ic-wire/issues/4) retain upstream history.
No generated runtime is patched downstream.

## Portable credential transport

`credentialBytes(value, maximum)` and `credentialFromBytes(bytes, maximum)` handle
a method-bound data envelope, not an issuer endpoint or signature verification.
The credential carries opaque canonical signed claim bytes and either a certified
proof or an Ed25519 signature. Protected receiver enrollment selects the authority;
submitted fields cannot install a key or choose trust.

A maximum must be a positive safe integer. The outer cap precedes input copying;
IC Wire enforces finite fields, frame identity/direction, exact consumption,
canonical encoding, UTF-8 and traversal bounds. Decoding copies the admitted view
into an ordinary Uint8Array so Buffer/subarray inputs cannot retain aliases.
Generated records may have null prototypes; consumers should use their declared
fields rather than assume Object prototype methods. u64 fields remain bigint.

Consumers own endpoint exports and transport. Use the corresponding Method's
request encoder and response decoder around the raw SDK call; its NAME is a
reference binding, not endpoint installation. Hosts admit raw argument lengths
before copying/decoding and authenticate ingress independently. A credential or
successful decode is never a session or application resource grant.

## Portable admission coordinator

Construct `PortableAdmissionClient` with an existing `PortableLogoutClient`, a
fresh generation, explicit `{ browser, account }` identities, protected issuer and
`PortableAdmissionTransport`. `establish(receiver)` records that destination
durably before admission and returns a generation-guarded identity on success.
Ordinary calls carry no proof; receivers check their caller-bound local session.

Pending 0.14 requires the identity object instead of the former single signer.
`account` authenticates `issue`/`reconcile` to the issuer; `browser` is the fresh,
directly authenticated receiver login and the only identity returned for ordinary
calls. Start the logout journal with `browser` and use that identity for receiver
logout. Do not use a different delegated signing key as proof of a different
principal: ordinary ingress delegation preserves its delegating caller.

Each transport attempt receives a temporary signer restricted to its issuer or
receiver destination. Durable active-generation, fixed-principal and cancellation
checks surround signing and completion for both roles. The account signer is not
returned as receiver authority. Applications retain their underlying keys;
signatures already produced and requests already accepted cannot be recalled.
The account signer is separate from the browser cleanup key, so application
key-disposal policy must distinguish them too.
Cleanup uses the retained browser identity without requiring the account provider
to remain signed in. The issuer's logout endpoint must resolve that browser login
to its original issuance/account binding; it cannot treat the browser caller as a
new account subject. This does not provide a production issuer logout endpoint.

The journal atomically claims issuance before dispatch. The first claimant calls
`issue`; overlapping instances, reloads and uncertain attempts use `reconcile`.
The sole journal claim API is `claimIssuance(generation)`: `true` permits the first
issuance attempt, while `false` requires reconciliation. Direct adapters must
check that result. The redundant `issuing()` wrapper is removed; this changes no
stored journal bytes and does not clear unknown issuance evidence.
Reconciliation must recover the original material or refuse unresolved; it must
not issue a replacement, extend a deadline or infer no effect from absent/pruned
evidence. The transport owns exact intent persistence and the crash boundary
between journal claim and issuer dispatch. It must authenticate admission replies.
A production issuer/reconciliation adapter remains unimplemented here.
Its exact saved intent must also bind the original account caller and browser
public key. Reopening with a different account must not recover or reissue for
the old intent. The logout journal's unchanged profile does not store that issuer
request/account binding; this remains the issuer adapter's existing intent owner.
Do not recover it by replacing the account caller with the browser principal.

`httpAdmissionTransport({ host, methodName, scopes, ttlSecs?, rootKey?, fetch? })`
supplies the receiver `admit` operation. Compose it with the application's existing
`issue` and non-issuing `reconcile` operations; it installs no issuer endpoint or
recovery policy. The method must implement the shared `EstablishSession` Wire
contract. Scopes and requested TTL are fixed when the adapter is constructed.
The adapter encodes through the generated codec, uses the current guarded signer,
submits one update, authenticates the reply through the SDK and decodes the bounded
response. HTTP acceptance alone is insufficient; a receiver error refuses and a
successful reply must name the same caller. Root trust is explicit, never fetched
from credentials or an endpoint. The coordinator retains retry and logout rules.

Pending Auth 0.14 changes only admission's error reply from text to the bounded
`AdmissionRefusal` enum. Receiver adapters use the core
`SessionError::admission_refusal()` projection and the Wire enum conversion.
The browser throws `AdmissionError` (`code === 'admission_refused'`) with the
generated `refusal` tag. Network failures remain transport failures. Upgrade
receiver encoders and generated clients together; there is no text-error fallback.
Successful admission replies, requests, credential bytes, journal/storage formats
and read/logout contracts are unchanged.

`NotYetValid` is a retry hint, not evidence that the submitted proof is valid.
The caller may explicitly retry the identical credential with bounded backoff
inside its original proof/operation deadline and active login generation. Neither
the transport nor coordinator automatically retries or renews. Expiry, invalid
signatures, authority failures, reset and other refusal tags must not be treated
as future-start hints. Never parse diagnostic strings to select retry policy.

For Canic's own command envelope, its adapter remains Canic-owned; do not wrap
Candid in Wire or copy an Auth codec. Freeze the production issuer's exact saved
request identity, request bytes/grants, original credential and original admission
deadline before supplying `issue`/`reconcile`. Missing/pruned evidence stays
unresolved and must not allocate, sign, renew or extend a deadline. Acceptance
remains in [Auth #3](https://github.com/dragginzgame/ic-auth/issues/3),
[Canic #491](https://github.com/dragginzgame/canic/issues/491) and
[Canic #507](https://github.com/dragginzgame/canic/issues/507); the reference SDK
receiver test does not establish production issuer or browser acceptance.

One detached credential is cached only until its original admission deadline and
reused across receivers. The transport supplies `admissionUntilNs` conservatively
in the journal clock domain; recovery never resets that deadline relative to
retrieval. Concurrent calls for one receiver share a bounded in-flight admission;
other receivers proceed independently. Completed calls are not session authority.
There is no automatic renewal or application-operation retry.

Calls last at most five seconds and receive a cancellation signal and temporary
identity. Durable generation/lease checks surround signing and completion. Logout
rejects late results; already signed/accepted requests cannot be recalled.
Receiver proof expiry, authority, replay, scope and atomicity checks remain mandatory.

## Durable logout and guarded identities

`IndexedDbLogoutStore` uses one dedicated application-owned database per login.
The unchanged `ic-auth-logout-journal-v1` profile stores no secret or credential:
caller, issuer, generation, clock, destination/deadline inventory, attempts,
acknowledgements and conservative unknown-issuance state. Opening and strict transaction lifecycle now live directly with the sole journal
owner; no generic token-store callbacks remain. Strict transactions
commit durable invalidation before any network fan-out; corruption, profile drift,
clock rollback and conflicting state refuse instead of clearing records.

The inventory is bounded at 64 destinations including the issuer, with at most
eight attempts each. `begin` commits cleanup and its finite deadline first.
`cleanup` persists a dispatch lease/attempt before calling each destination.
Capacity and uncertain replies do not acknowledge revocation; a final attempt is
reserved until after the admission window. Crash/reload and suspended tabs share
the journal. Only authenticated `Revoked` replies acknowledge that destination;
issuer confirmation does not erase historical unknown issuance.

`httpLogoutTransport` sends one raw Wire update per attempt using the current
identity, explicit endpoint and protected gateway/root configuration. SDK
certificate/read-state verification authenticates the exact reply. HTTP acceptance
alone is insufficient. The worker owns retry/disposal; the transport does not
fetch trust, resubmit an update, cache a signer or install endpoint policy.

`guardedIdentity` checks durable generation at signing. A cleanup identity is
restricted to its current target/lease and refuses after cancellation or deadline.
The host owns idempotent key disposal. Browser logout cannot promise synchronous
fleet-wide revocation of offline destinations; receiver limits, proof/session
lifetimes and authority freshness continue to bound the residual window.

## Qualification and consumer boundary

`make test-client` checks generation, Rust/TypeScript vectors, bounded/canonical
rejections, detached byte views, admission flights and durable logout/cancellation.
`make test-qualification` exercises real certified SDK replies across two receivers,
caller mismatch, exact retry, proof-free reads, retained-key revocation, both proof
mechanisms, certification and upgrades. This is a reference host with prepared
proofs and fake IndexedDB in Node, not production issuer or Canic acceptance.

The native [logout fixture](../packages/client/test/browser/logout.html) remains
available for application-owned browser checks. No new native Chromium run is
claimed by Node/Testkit evidence. Stable versus volatile receiver storage remains
a separate host choice: the transport owns neither stable memory nor reset policy.
Canic owns protected enrollment, its atomic backend, endpoint integration and final
scoped guards. Toko owns account policy and application resources. Account subject
is issuer-scoped; browser principal/generation is the login identity.
