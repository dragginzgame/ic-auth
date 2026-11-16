# Private browser client

This private, unpublished package owns a strict IndexedDB login journal, credential
retrieval, explicit receiver admission and one-step logout cleanup. Platform owns
SDK identities and HTTP certificate verification; consumers own endpoints, trusted
configuration, issuer response mapping and key custody. Product delivery/bundling
and the complete browser lifecycle remain unqualified.

## Supported API

Import from `@dragginzgame/ic-auth-client`:

| Entry | Responsibility |
| --- | --- |
| `LoginJournal.open(indexedDB, name, clock)` | Open the dedicated journal; return a lifecycle handle |
| `journal.start({ generation, caller, issuer })` | Commit a new login bound to a self-authenticating browser principal |
| `journal.inspect()` | Return detached phase, sequence, deadline and destination progress |
| `journal.beginLogout(generation, deadlineNs)` | Commit local invalidation and credential deletion together |
| `journal.close()` | Release this database handle; does not perform logout |
| `retrieveCredential(options)` | Reuse saved evidence or make one issue/renew/lookup call, then commit exact evidence |
| `establishSession(options)` | Retrieve evidence, inventory the receiver and make one guarded Wire admission call |
| `new SessionCoordinator(options)` | Bind one login, fixed scopes/transports and an explicit receiver-to-journal deadline mapping |
| `coordinator.ensureSession(target)` | Reuse a checked hint or share bounded readiness/admission work |
| `coordinator.call(target, operation)` | Prepare readiness, then run one operation with a bounded browser signer |
| `coordinator.establish(target)` | Force one admission, sharing only an equivalent outstanding attempt |
| `cleanupOnce(options)` | Resume committed logout with at most one reserved Wire effect |
| `cleanupUntilFinished(options)` | Schedule committed logout through confirmation or its fixed deadline |
| `httpUpdateTransport({ host, rootKey, methodName, fetch? })` | Supply the shared SDK-certified raw update transport for admission or cleanup |

`LoginJournal` is the lifecycle handle type and its opening factory, not a public
constructor. Operation options accept that same handle. Internal claim, guard,
settlement and signing-attempt functions are not supported consumer APIs. They
remain available to package implementation and focused qualification through
internal source imports. The narrowed TypeScript surface is not a security boundary
against arbitrary JavaScript in the same application.

## Journal and lifetime guarantees

The injected clock supplies a stable `id` and checked u64 nanoseconds through
`nowNs()`. Clock rebinding or regression refuses. Database, generation and clock
identifiers are nonempty strings of at most 128 characters. One dedicated database
uses version 1 and the frozen private `ic-auth-login-cleanup-journal-v1` profile.
An existing login or orphan credential prevents replacement; no reset or migration
is provided.

The journal privately owns synchronous transitions within native `readwrite`
transactions with `strict` durability. Operations resolve after transaction
completion, not request success. Foreign layouts, corrupt records and unavailable
strict durability refuse without erasing evidence. Ordinary guards count credential
presence without loading its bytes. Keys and network effects are never stored or
executed by the transaction owner.

Network operations capture caller/destination and recheck current journal state
before and after SDK signing and before accepting a result. Each signing attempt
has a five-second ceiling, including its journal waits, and permanently closes on
success, failure, cancellation or timeout. Polling signatures receive fresh guards.
Already returned signatures and accepted requests cannot be recalled. Transport
failure never proves that the remote effect did not happen.

## Credential retrieval and admission

`retrieveCredential({ journal, generation, browser, account?, issuer, transport,
signal?, refreshThrough?, maximumBytes? })` returns detached `{ bytes, untilNs,
sequence }`. An unexpired saved completion requires no call. Otherwise one durable
claim selects initial account-signed issuance, browser-signed renewal, or lookup-only
reconciliation with the original sequence and signer role. Pending work never
permits reissuance. Missing/pruned evidence and unknown issuance without a
recoverable sequence cannot be interpreted as absence of an effect.

The injected `IssuerTransport` implements `issue`, `renew` and `reconcile`, uses
the supplied temporary identity, and owns immutable login/grant binding,
authenticated replies and conservative expiry mapping into the journal clock.
The operation captures bytes before asynchronous checks and commits evidence
before returning. Identical completions preserve the original expiry; conflicting,
stale, post-logout or overflowed sequences refuse. The generated frame cap defaults
to 131,072 bytes and cannot exceed its configured ceiling. Structural decoding is
not proof authentication; the receiver still verifies the credential.

`establishSession` adds `target`, `scopes`, optional `ttlSecs` and an `admit`
transport to the retrieval options. It captures request inputs, commits the target
to the bounded 64-destination cleanup inventory before dispatch, and uses only the
browser identity at the receiver. Guards bind the original issuer, current
sequence and unexpired proof. Generated decoding checks frame bounds; accepted
receipts must match caller/issuer and have positive expiry. Receipt time remains
in the receiver's clock domain and the detached receipt is not continuing authority.

Each admission makes one attempt. Every typed refusal is an `AdmissionError` with
its original tag; transport errors propagate. No refusal automatically renews or
replays an application operation. Consumer Canic endpoints own wiring and current
application authorization.

## Readiness and admission coalescing

Create one `SessionCoordinator` per login/tab with the same options as
`establishSession`, except `target` and `refreshThrough`, plus
`refreshAt(receipt, target)`. This synchronous host callback returns a conservative
exclusive deadline in the journal clock. Auth infers no receiver/browser offset.
The coordinator captures identities, issuer, scopes and adapter entrypoints; its
optional signal cancels the whole coordinator, including shared work.

`ensureSession(target)` returns a detached scheduling receipt, never a signer or
application authority. Use `call(target, async (identity, signal) => ...)` for an
ordinary application operation. It prepares readiness and invokes the callback
once with a temporary browser signer and cancellation signal. The callback owns
its transport and must pass that signal through. A live hint reads
only checked journal metadata. Proof expiry alone does not expire a receiver hint.
Reopening a coordinator has no hint and admits retained exact evidence again.

Concurrent readiness for one receiver shares preparation. Different receivers share
in-flight credential retrieval while admitting independently. Expired hints refresh
through the sequence that created the lease; a newer completed sequence can serve
another receiver without another renewal. There are at most two preparation attempts:
only an authenticated typed `Expired` refusal or an exactly recovered expired
credential permits one fresh-proof attempt. Other refusals and uncertain replies
propagate. A freshly mapped deadline already reached returns `lease_expired`
without looping. Explicit `establish` still forces one admission without that retry.

The coordinator admits at most 64 outstanding public operations, including duplicate
waiters. Overflow returns `operation_limit` before journal access; completion,
refusal and network timeout release the slot. This is a per-coordinator bound, not
cross-tab scheduling. The existing five-second network attempts remain the effect
owner. Use `journal.beginLogout` directly: local invalidation never waits for an
admission flight, and late signatures/results cannot install usable readiness.
The call signer checks the captured readiness deadline and durable login generation
before and after each SDK signature, including read-state polling, and before
returning a result. It closes on completion, cancellation or its five-second
timeout. Expiry after callback invocation refuses without renewal or replay. A
refusal after submission leaves the remote outcome uncertain; it does not undo
an application effect. Returned signatures cannot be recalled. Calls occupy the
same 64-operation budget through callback completion. Key disposal remains separate.

## Logout and cleanup

Call `beginLogout` first. Its successful strict commit establishes local
invalidation; a failed commit does not. The first deadline is fixed, positive and
at most five minutes ahead. Repeated calls preserve it and all progress, including
pending sequences and historical issuance uncertainty.

`cleanupOnce({ journal, generation, browser, revoke, signal?, retryAfterMs? })`
returns `wait`, `finished` or `settled`. Internally it commits a cross-tab lease and
attempt count before dispatch. Each destination has eight attempts; a lease lasts
at most five seconds and cannot exceed the cleanup deadline. Lost responses,
timeouts and failed settlement retain the count/lease for explicit resumption.
Consumed tickets cannot authorize another signature even after an aborted write.

Only an authenticated, matching positive `Revoked` response confirms a destination.
Its timestamp remains in the receiver clock domain. Other typed replies stay
unconfirmed; the local retry delay is 500–1,499 ms, defaulting to 1,000. The operation
starts no retry loop. Finished means all destinations confirmed **or** the fixed
deadline elapsed: inspect per-destination confirmations and retained uncertainty.
Exhausted attempts wait for that deadline.

Use `cleanupUntilFinished(options)` to schedule these steps and receive the final
`JournalStatus`. It requires an already committed `beginLogout`, captures the
browser identity and options, and sleeps for journal-calculated delays of at most
five seconds. Every
wake-up claims again, respecting other tabs, the original deadline and attempt
limits. Cancellation interrupts waits and active signing attempts. Transport,
certificate, storage and clock errors stop the run; call it again to resume the
retained progress. It never clears uncertainty or disposes of keys. Three native
cases now qualify abrupt renderer loss after committed local logout, signed cleanup
dispatch and committed confirmation. They reopen the same browser storage with a
host-supplied identity. Four further cases cover pending issuance/renewal, saved
evidence and logout fencing after reopening. Unknown work reconciles the original
sequence without reissuance, and unavailable evidence remains unresolved.
Whole-browser failure, external signer recovery and key recovery are not covered.

### Host-owned key disposal

After cleanup finishes, call `disposeLoginKey({ journal, generation, caller, dispose })`.
Retain the public caller and generation independently of the private key so a lost
deletion reply can be retried after reopening. The operation validates terminal
cleanup and the exact binding, then passes a frozen `{ generation, caller }` and
detached terminal status to the host callback. It never erases the journal.

The host must conditionally delete only that exact key, succeed when it is already
absent, and preserve replacement keys. Concurrent and repeated callbacks are
possible; this is not an exactly-once effect. Inspect unconfirmed destinations and
issuance uncertainty before choosing to delete; deadline completion is not proof
of remote logout. A callback may throw to retain custody. Cancellation prevents
invocation, but once invoked the operation awaits the actual callback outcome
without a timeout or rollback claim. Host custody owns all key copies and handles;
this API cannot guarantee erasure of JavaScript or SDK memory.

Seven native cases cover terminal/binding refusal, detached evidence, deadline
uncertainty, failed-reply retry, replacement protection, cancellation and storage/
clock failure. An eighth renderer-loss case deletes through a host test double,
loses the callback reply, then retries with only the public binding. This qualifies
coordination, not a production key store or whole-browser recovery.

## HTTP trust boundary and evidence

`httpUpdateTransport` captures an explicit gateway origin, root bytes and endpoint.
It uses Platform's certified update path, one submission and fresh read-state
signatures. Automatic root fetching, clock synchronization and update retries are
disabled; redirected requests and requests outside the selected origin/destination
refuse. Platform owns certificate parsing, verification, freshness and request-ID
binding. Use this transport through admission/cleanup, which own codecs and signing
lifetime; do not treat HTTP acceptance as an authenticated reply.

Node checks cover SDK signing, independent generated Wire frames, HTTP binding and
the package API boundary. Native Chromium checks cover strict IndexedDB, two-tab
ordering, interruption, stale signatures, exact evidence and cleanup uncertainty.
A separate single-subnet PocketIC gate authenticates actual fixture replies and
rejects wrong roots, tampering and replay of another request's certificate.
Four additional installed cases drive the supported lifecycle/operation API through
a fixed combined issuer/receiver: lost issuance reply and lookup-only reconciliation,
real Wire admission, caller-bound application writes, exact retry, local invalidation
and certified remote cleanup. The host independently counts one signature and one
write, retained fences and zero remaining sessions. Captured proof replay after
logout refuses even when a raw caller bypasses the local journal. The fixture uses
installation-owned enrollment and a local public test signer; production Canic
wiring, external signing recovery and multi-receiver coordination remain unqualified.
Production key custody and broader renderer-interruption acceptance remain open.

Run `make test-client`, `make test-client-journal` and `make check-client-generated`;
`make qualify-client-http` is the separate installed gate. Prepared tools, browser
selection, suite selectors and ownership limits are documented in
[developer tooling](../../docs/development/tooling.md). No gate downloads a browser
or uses a personal profile. Node 24.21.0 consumes the erasable TypeScript directly;
the package and Rust crates remain unpublished.
