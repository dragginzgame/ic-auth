# Portable clock-contract audit — 2026-11-06

Scope: [#127 — Resolve the production clock-skew contract](https://github.com/dragginzgame/ic-auth/issues/127).
This audit supersedes the earlier recommendation to adopt receiver-bound admission
challenges immediately. Preserve portable local verification and avoid adding
receiver-to-issuer calls. The implemented hard cut uses signed deadlines for known logins, explicit
unconfirmed local denial for unknown logins and fail-closed lost-state recovery.
Canic production acceptance stays consumer-owned; its refusal is not removed here.

## Standards and applicable boundaries

[OAuth Security BCP, RFC 9700 §2.3](https://www.rfc-editor.org/rfc/rfc9700.html#section-2.3)
recommends sender-constrained and audience-restricted credentials. Auth already
checks the authenticated browser principal, audience, issuer/key, grants, replay
and protected authority. This is an architectural comparison, not OAuth compliance.

[Token Revocation, RFC 7009 §3](https://datatracker.ietf.org/doc/html/rfc7009#section-3)
explicitly supports self-contained tokens verified without contacting the issuer
and describes the state/communication tradeoffs of revocation. Online
[introspection, RFC 7662 §4](https://www.rfc-editor.org/rfc/rfc7662.html#section-4)
adds network work; caching introduces a stale-revocation window.

[DPoP, RFC 9449 §§8–9, 11.2](https://www.rfc-editor.org/rfc/rfc9449.html#section-11.2)
uses server nonces to limit proof pre-generation. Its resource-server nonce is
normally signed by the client, not a requirement for issuer reissuance per receiver.
That is not the proposed issuer-signed receiver-grant protocol. IC ingress already
authenticates possession of the browser key; avoid adding a second client-proof
layer merely to establish the same fact. A nonce alone
cannot revoke a still-valid credential when the attacker retains the browser key:
the attacker can obtain a new nonce and sign a new request. Logout and reset still
need independent state or an authenticated invalidation boundary.

[IC system time](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#system-time)
is monotonic within a canister; the specification supplies no cross-canister or
real-time skew bound. A conventional JWT clock tolerance is not evidence for one.

## Current path and cost

Auth's `establish_session`, `authorize_session` and `logout_session` in
[the engine](../../crates/ic-auth/src/session/mod.rs) are synchronous local functions.
Fresh admission performs complete verification; ordinary authorization and exact
retries reuse retained state while checking live authority, generation and denial.
There is no issuer lookup, network retry or proof verification on ordinary use.
The browser [admission coordinator](../../packages/client/src/portable-admission.ts)
reuses one exact issued credential across receiver admissions.

Read-only Canic review at HEAD `e6a061999` plus its working tree traced
`ops/auth/portable/runtime::{admit,read,logout}` through the portable adapter.
Those functions use retained enrollment and the local persistent receiver owner;
there is no await/network call in those inspected operation paths. Enrollment
refresh is separate. No Canic files or production activation were changed.

| Path | Existing architecture | Proposed issuer-signed receiver challenges |
| --- | --- | --- |
| Ordinary application call | Local session/authority/store checks | Should remain identical |
| First proof | One issuer exchange, shared across receivers | Receiver-specific issuance/recovery |
| Admission at a receiver | Browser submits proof to receiver | Obtain challenge, issuer binds it, submit grant |
| Inter-canister calls | None required in these request paths | None only if browser relays; server relay would add calls |
| New state | Existing issuer and receiver records | Challenge slots, grant recovery and logout/reset coordination |

For N receivers, a simple implementation changes the logical exchanges from
one issuance plus N admissions to approximately 3N challenge/issue/admit exchanges.
This counts logical API steps, not physical IC messages: certified issuance can
already include preparation and retrieval. Parallelism/batching changes totals
and latency but introduces its own coordination. No timing or Wasm delta was
measured for an unimplemented alternative; there is no demonstrated saving.

## Accepted clock-independent contract

- **Known login:** admission requires `now < proof_expiry <= login_deadline`;
  every session also ends by that immutable authenticated deadline. A monotonic
  receiver cannot accept that login after the deadline. Logout preserves its
  binding, denies it through that deadline and atomically removes sessions.
  Longer live denial is never shortened. No fresh proof or enrollment is needed.
- **Unknown login:** no receiver-local finite duration can bound an unseen
  issuer-ahead credential. The endpoint commits bounded temporary local denial
  until `now + host_max_login_ttl` and returns `Unconfirmed`, never `Revoked`.
  Repeating retains the same deadline/class. After pruning, a delayed credential
  may admit: this is why the acknowledgement is explicitly unconfirmed.
- **Complete durable upgrade:** restore sessions, replay, login/denial records,
  protected authority and clock/pool metadata from one trusted transaction.
  Authenticate current enrollment before readiness; deadlines are not extended.
- **Actual safety-state loss:** stay closed pending trusted complete recovery.
  Local timestamp floors, generation increments and fresh enrollment do not
  invalidate every old signed login. The timestamp reset helper is removed.
  An authenticated non-reusable recovery fence would need separate design and
  acceptance; arbitrary snapshot rollback remains independently unqualified.
- **Browser cleanup:** commit local invalidation before fan-out, retry with bounded
  concurrency/attempts/backoff and dispose local keys at the finite local budget.
  Only authenticated `Revoked` acknowledges a destination. `Unconfirmed`, lost
  replies, expiry estimates and local timeout retain uncertainty. A stable local
  scheduling clock rejects rollback but requires no safety-critical IC offset.

This changes the former unconditional unseen-login confirmation guarantee.
It adds no challenge flow, credential-bearing logout, cryptographic relaxation,
receiver-to-issuer lookup or automatic issuer rotation. Existing ingress sender,
audience, grant, authority, strict future-time, signed-byte and verification checks
remain. Both proof mechanisms share these rules. Ordinary authorization remains
local and performs no credential verification. No performance saving is claimed.

## Bounds and persistence

Known reservation/denial uses its original deadline; unknown denial uses the
positive host login ceiling, at most 90 days. Admitted and unknown pools stay
separate. Unknown work has independent receiver-wide credit; exhaustion cannot
spend admitted logout capacity. A failed atomic commit never acknowledges denial.
Repeated attempts cannot slide a live deadline. Restores reject incomplete counts,
domains, duplicates, rollback and invalid retention without publishing partial state.

Protocol logout moves from `0x415554480004` to `0x415554480014` so old variant
ordinals cannot be confused with `Unconfirmed`. Login records move from
`0x415554485313` to `0x415554485323`; session/replay identities and signed
claims do not change. The private fixture checkpoint moves from `AUTHNAT3` to
`AUTHCLK1`. Old bytes are refused without clearing. The maintainer confirmed no
retained affected Rust records and no retained private browser-journal records.
The private client selects `ic-auth-local-logout-journal-v1`; retired journals
are refused, never migrated or implicitly reset. Consumers update both endpoints
and generated clients together and own any actual deployed-record disposition.

## Acceptance

Focused claims tests enumerate signed windows for both mechanism tags, including
translated/near-limit clocks. They retain strict future-issued refusal and the
counterexample showing local timestamp reset fails. Revocation tests use real
Ed25519 proofs to show unseen credentials may become valid after unknown denial
expires, including after durable restoration. Known denial blocks exact retries
and fresh renewal through complete restore. Engine tests distinguish confirmed
and unconfirmed outcomes and retain atomicity, capacity and work isolation.

Installed fixture acceptance checks complete durable upgrade plus fresh authority,
and refuses reinstallation/fresh authority when an upgrade lost safety state.
Browser acceptance covers explicit unconfirmed replies, finite cleanup before
remote scheduling hints, crash/reopen, stale signatures and retained uncertainty.
Current execution evidence belongs in the [handoff](../status/current.md), not
this design. These are Auth contract checks, not Canic production acceptance or
a cryptographic audit. Canic #491 owns actual storage/clock/issuer/browser wiring.
