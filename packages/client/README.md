# Private browser client

This private, unpublished package implements bounded SDK signing, a durable login
journal, credential retrieval and explicit receiver admission through injected
adapters. Cleanup now dispatches one journal-owned logout attempt through an
injected adapter. Readiness coordination and actual HTTP transports remain unimplemented.
Node 24.21.0 can consume its erasable TypeScript directly. Product browser
delivery/bundling has not been selected.

## Signing attempts

`runSigningAttempt` captures an existing Platform SDK identity's principal and
destination before asynchronous work. The owning journal supplies `assertCurrent`,
which runs before dispatch, before/after SDK signing and before accepting the
operation's result. Destination and principal checks also run after asynchronous
journal work. SDK read-state polling retains all authority and disposal checks.

Success, failure, timeout and external cancellation permanently close the signer
and abort its transport signal. The maximum is five seconds, including journal
waits. Late results are refused even if a provider ignores cancellation; no
replacement attempt is started. Already returned signatures or accepted requests
cannot be recalled. Transport errors propagate unchanged and never imply that an
external effect failed to happen.

## Durable local invalidation

`LoginJournal.open(indexedDB, databaseName, clock)` uses one dedicated database,
version 1, and the existing private `ic-auth-login-cleanup-journal-v1` profile.
The injected clock provides a stable `id` and checked u64 nanoseconds through
`nowNs()`. Generation and clock identifiers are nonempty strings of at most 128
characters. Clock rebinding or regression refuses an operation.

- `start({ generation, caller, issuer })` commits a new login for a self-authenticating
  browser caller. Any existing login or orphan credential prevents replacement.
- `addDestination(generation, target, admissionUntilNs)` records a receiver before
  admission. Inventory includes the issuer and is limited to 64 destinations.
  Admission expiry must leave room for the five-second attempt inside the existing
  five-minute cleanup window.
- `assertActive(generation, caller, target)` checks the persisted generation, phase,
  caller and inventory. Supply a closure calling it as `runSigningAttempt`'s guard;
  do not cache its result across asynchronous work.
- `beginLogout(generation, deadlineNs)` commits local invalidation and credential
  deletion together. The first cleanup deadline is fixed and at most five minutes
  from the current clock. Repeated calls retain it, along with destination progress,
  pending sequence and unknown issuance. Unknown issuance without a recoverable
  sequence remains unknown.
- `inspect()` returns detached, bounded metadata. Each `destinations` entry exposes
  `canister`, `attempts` and optional `confirmedUntilNs`; `close()` releases the handle.

All operations resolve only after a native `readwrite` transaction with `strict`
durability completes. Request success alone is insufficient. Refused durability,
corrupt metadata or a foreign schema/profile never trigger reset or migration.
The operation/transaction owner is private; callers cannot mutate persisted facts
through the API. Ordinary guards do not load credential bytes. A failed logout
commit does not establish durable local logout.

## Issuance and credential recovery

`claimCredential(generation, { accountAvailable, refreshThrough? })` returns one of
`issue`, `renew`, `reconcile` or `saved`, only after strict commit. Initial issuance
uses the account signer; renewal uses the browser signer. Pending work always
returns `reconcile` for the same sequence and original signer role. A second tab
or reload cannot obtain a fresh dispatch for that pending sequence. Unknown issuance
without a recoverable sequence refuses with `issuance_unrecoverable` and retains
uncertainty. An unavailable account refuses initial issuance/reconciliation.

A completed, unexpired result returns detached exact bytes and its original local
expiry. Expiry or an explicit `refreshThrough` watermark can reserve the next
sequence and atomically remove old evidence. A newer completion than the watermark
is reused; pending work is never superseded. Sequence overflow refuses without
discarding the previous completion.

`completeCredential(generation, sequence, bytes, untilNs)` captures the frame before
awaiting storage, checks its generated Wire structure and commits metadata/evidence
together. Identical retries are accepted; different bytes or expiry refuse. Stale
sequence, generation and post-logout replies cannot install evidence. Recovery
validates the metadata/evidence pair, including frame structure, before returning
bytes or reserving renewal. Corruption refuses without erasing evidence; local
logout remains available to invalidate and delete it.

The caller must map credential expiry conservatively into the injected clock; the
journal infers no clock offset. Structural decoding does not authenticate claims,
signatures or issuer provenance. The issuer adapter owns response authentication;
the receiver owns complete verification. Credential retrieval stores no signing
keys and cannot establish remote revocation.

## Issuer adapter integration

`retrieveCredential({ journal, generation, browser, account?, issuer, transport,
signal?, refreshThrough?, maximumBytes? })` executes one journal decision. A saved
completion returns detached evidence without a transport call. Otherwise, it
dispatches exactly one `issue`, `renew` or lookup-only `reconcile` call through the
injected `IssuerTransport`, using a temporary Platform SDK identity. Initial
issuance and sequence-zero lookup use the account signer; renewal and later
lookups use the browser signer. Each network attempt has the existing five-second
ceiling. Transport errors propagate unchanged and leave the reserved sequence
pending. Later retrieval reconciles that sequence; it does not retry issuance.

The adapter owns endpoint names, immutable browser-login/grant binding, authenticated
reply handling and conservative original-expiry mapping. It must use the supplied
identity and return an exact generated credential request frame plus `untilNs`.
Auth adds no issuer endpoint. Caller/issuer bytes and adapter methods are captured
before awaiting storage. `assertIssuance` binds every signing/completion guard to
the journal's original issuer and current sequence, including after another tab
starts logout or advances renewal. An inventoried receiver cannot stand in for
the issuer.

Returned frame bytes are copied before postflight journal waits and committed
before the operation returns. Cancellation, timeout, identity changes, logout and
stale sequence refuse late results. A failed completion commit returns no evidence
and leaves recovery lookup-only. The byte cap applies to fresh and saved results;
it defaults to the generated 131,072-byte policy and cannot exceed its configured
ceiling. Returned evidence retains its original expiry and conveys no session
authority; receiver admission must still check freshness and authenticate it.

This operation does not queue/coalesce callers, retry requests or implement the
admission/readiness coordinator. The adapter integration is qualified with injected
replies, native IndexedDB and real SDK signing; actual HTTP reply authentication
and production endpoint integration remain separate acceptance gates.

## Explicit receiver admission

`establishSession` accepts the credential-retrieval options plus `target`,
`scopes`, optional `ttlSecs` and an `admit` adapter. It captures the target/scopes
before asynchronous work, retrieves committed evidence, encodes the existing
EstablishSession Wire contract and strictly commits the receiver to the logout
inventory before dispatch. Only the browser identity reaches the receiver.

The injected `AdmissionTransport` receives a detached target, temporary SDK
identity, encoded request and abort signal. The host owns Canic endpoint wiring
and authenticating the corresponding update reply. It returns the raw Wire
response. Generated decoding enforces framing/bounds and detaches fields before
postflight waits; successful receipts must match the browser caller and original
issuer and carry a positive expiry. Receipt time remains in the receiver's clock
domain; Auth assumes no offset from the browser clock.

The journal's `assertAdmission` checks active generation, caller, original issuer,
inventoried destination and current completed sequence with an unexpired proof.
These checks run before/after signing and before accepting the receipt. Logout,
proof expiry or a new issuance sequence fences suspended work. A failed inventory
commit dispatches nothing. A lost response retains the receiver in the inventory.

Explicit admission makes one attempt. Every existing refusal is surfaced as an
`AdmissionError` with its `refusal` tag; transport errors remain unchanged. No
refusal, including `Expired`, triggers automatic renewal or retry in this operation.
The detached `AdmissionReceipt` is metadata, not an identity or cached authority.
The temporary signer closes when the attempt ends. Readiness, bounded caller
coordination and guarded ordinary application calls remain separate work.

## Durable cleanup attempts

The same journal owns cleanup progress in the existing private v1 record. These
operations are the storage boundary used by `cleanupOnce`, not a network
transport or independent proof of authenticated remote revocation.

- `claimCleanup(generation, caller)` requires committed local logout and the original
  browser caller. It strictly commits one attempt and a lease before returning an
  ephemeral ticket, or returns `wait` with a local scheduling time, or `finished`.
  One live lease serializes cooperating tabs; it lasts at most five seconds and
  cannot outlive the fixed cleanup deadline. The ticket carries the corresponding
  network timeout in whole milliseconds (at least one); journal guards enforce
  the exact nanosecond deadline. Each destination has eight attempts.
- `assertCleanup(generation, caller, ticket)` checks the live ticket and persisted
  caller, generation, destination, attempt, lease and deadline. Use it around SDK
  signing and response acceptance. A ticket belongs to the issuing journal handle;
  copied tickets, expired leases and consumed tickets refuse. Pair this guard with
  `runSigningAttempt` to bind the signing destination and close the signer.
- `completeCleanup(generation, caller, ticket, result)` consumes the ticket even if
  settlement fails. The trusted adapter may supply `revoked` with a positive u64
  acknowledgement only after authenticating the matching reply. That timestamp
  remains in the receiver clock domain. Every other outcome is `unconfirmed` with
  a local retry delay of 500–1,499 ms. No HTTP acceptance or elapsed time confirms
  a destination. Settlement after lease/deadline expiry refuses.

A crash or failed settlement retains the committed count and lease. Reload waits
for the lease and can reserve another attempt within the existing limits. The
worker must wait/retry explicitly; the journal starts no loop or timer. Cleanup
finishes when all destinations are confirmed or the fixed deadline expires.
Exhausting attempts waits for that deadline. Finished status retains unconfirmed
destinations and historical unknown issuance, including missing sequences.
Repeated logout cannot extend the deadline or reset progress.

This retains necessary recovery state from the reference in one journal owner.
There is no second mutable-record API, scheduler, identity facade or key store.
Automatic scheduling, HTTP reply authentication and key disposal remain deferred.

## One logout dispatch

`cleanupOnce({ journal, generation, browser, revoke, signal?, retryAfterMs? })`
resumes a previously committed logout. It returns `wait` or `finished` without a
transport call, or dispatches exactly one reserved attempt. The caller owns when
to resume; this function starts no retry loop and does not implicitly invalidate
an active login. Pre-cancellation or invalid retry configuration consumes no attempt.

The injected `LogoutTransport` receives a detached target selected by the journal,
the temporary browser identity, the generated unit request and an abort signal.
Canic owns endpoint wiring. The adapter must authenticate the exact update reply
before returning its raw bytes; HTTP acceptance alone is insufficient. Auth checks
the Rust-owned 1,024-byte browser cap, generated Wire framing and a positive Revoked
timestamp. Only Revoked confirms the selected destination. Every typed Unconfirmed,
InvalidCaller or Unavailable response commits nonconfirmation and a local retry delay
(default 1,000 ms, configurable from 500–1,499 ms). Receiver timestamps never become
browser scheduling deadlines.

The result is `settled` with the detached response and committed journal status.
Success reaches the caller only after strict settlement. Transport errors,
malformed replies, cancellation and timeout propagate unchanged or as
`invalid_material`; they leave the committed attempt/lease for later recovery.
A failed settlement also retains uncertainty. Signing always closes when the
bounded attempt ends, and late results cannot confirm after its lease/deadline.
Cancellation after a valid acknowledgement has committed may reject the call while
retaining that acknowledgement. No signing key is disposed by this operation.

## Qualification

Run from the repository root:

```sh
npm ci --prefix packages/client --ignore-scripts
make test-client
make check-client-generated
IC_AUTH_BROWSER_EXECUTABLE=/absolute/path/to/chromium make test-client-journal
```

The separate lock selects published Platform 0.2.0 and TypeScript 7.0.2. Platform
owns SDK Core; Auth imports no SDK Core or cryptographic primitive directly.
Ten Node cases use real SDK Ed25519 signing with controlled asynchronous guards,
including destination/principal mutation, late signatures/results, cancellation,
timeout and read-state polling. Six additional cases check credential/admission/logout
codecs against retained independent frames, optional TTLs, raw scopes and all
refusals, plus framing and request-bound rejection.

`make generate-client` uses the existing Rust IC Wire generator and TypeScript
compiler to produce credential/admission/logout modules and their matching runtime as JavaScript plus
declarations under `src/generated/`. `make check-client-generated` compares those
files with fresh output. Change schemas/generation at their Rust owner; do not edit
generated files. Only the three contract modules used here are adopted. Generated code
comes from IC Wire 0.24.1 and this repository's Auth schemas, not the legacy client.
Build scratch stays under root `target/client-wire/`.

Forty-four native IndexedDB cases use playwright-core 1.65.1, esbuild 0.28.2, a loopback
fixture server and an ephemeral two-tab browser profile. The fixture bundle stays
in memory. No browser is downloaded or user profile opened. Chromium 141.0.7390.37
passes concurrent creation, cross-tab logout during actual SDK signing, reopening
with unknown issuance, abort after a successful queued write, strict-durability
refusal, clock/bounds checks and corruption/schema admission. Persisted fixtures
are independently specified rather than generated by the journal implementation.
Issuance cases add concurrent reservation, lookup-only reload, exact/conflicting
completion, renewal watermarks, missing/overflowed sequences, paired-evidence
corruption and aborted reservations/completions. Credential bytes come from
[retained reference frames](../../crates/ic-auth-wire/tests/fixtures/README.md);
their cryptographic content is deliberately unverified. Eight adapter cases add
account/browser signer selection, captured destination/identity, lost replies,
cancellation, timeout, logout/sequence races and failed completion commit recovery.
Seven admission cases cover durable inventory, exact request bytes, browser-only
signing, refusal propagation, invalid receipts, aborts and logout/expiry/sequence
races. Successful replies use independent fixture packing derived from the retained
Wire frame; no replacement encoder creates those reply fixtures.
Seven cleanup cases add cross-tab reservation/reload, attempt exhaustion,
per-destination confirmation, late acknowledgements, aborted writes, u64 bounds
and suspended real SDK signing after a ticket is consumed. A reused lease timestamp
cannot revive the old signing capability. Confirmation inputs are injected trusted
adapter results; these tests do not establish HTTP reply authentication.
Seven logout adapter cases add exact request bytes and real browser signing,
all typed replies, malformed/capped/zero acknowledgements, no duplicate cross-tab
dispatch, lost responses, cancellation/timeout, aborted reservation/settlement and
late replies. Replies use the independent retained caller-contract frames; no
replacement encoder produces the acceptance oracle. HTTP authentication is still
an adapter obligation, not established by these injected responses.
Set `IC_AUTH_CLIENT_SUITE=journal`, `issuance`, `transport`, `admission`, `cleanup` or `logout` to select
a focused group; the default runs all groups.
Renderer interruption, other browsers and complete SDK transport remain unqualified.

The behavior reference is `dragginzgame/ic-auth` at
`e52a40be75dd9809d0528189d84b43fe96952ad9`, particularly
`docs/protocol/browser-client.md`, `packages/client/src/checked-identity.ts`,
`bounded-call.ts`, `logout-store.ts`, `logout.ts`, `portable-admission.ts` and
`admission-http.ts`. The tests' SDK request shape
follows its `packages/client/test/support/requests.ts`. Journal field validation,
transaction ordering and limits retain that source contract. The replacement
combines attempt lifetime and identity disposal in one owner and exposes specific
journal operations instead of the legacy public mutable-record transaction API.
