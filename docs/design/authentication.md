# Authentication design

The detailed [architecture and qualification sequence](DESIGN.md) owns the
replacement plan. This overview summarizes its behavior and ownership boundaries.

## Required behavior

One application authentication flow admits a portable credential, then authorizes
ordinary calls through a caller-bound local session. Enrolled Ed25519 and
certified/BLS proofs converge on the same session rules. Each receiver admits
independently; a wallet proof, ingress delegation and application resource grant
remain different authorities.

A fresh browser principal identifies a device login. `(issuer, subject)` qualifies
an account. Preserve device-slot generation fencing, caller binding, authority
generation invalidation, grant narrowing, expiry, replay consumption, exact retry,
logout denial and bounded retention. Renewal never extends the original absolute
login deadline. Issuer signing operations retain intent/completion/recovery state;
external signing success must not be inferred from an interrupted response.

The private browser admission/logout coordinator remains in replacement scope:
SDK identity and issuer boundaries are injected; unknown issuance and durable
logout uncertainty must survive interruption. No production wallet issuer or
service is implied by these contracts.

## Ownership

| Owner | Responsibility |
| --- | --- |
| Passive protocol types | Validated identifiers and data; no storage, CDK or crypto effects |
| Wire adapter | Existing bounded methods and domain projections; generated browser codecs |
| Auth verification | Canonical bytes, protected verifier selection, proof checks and work budgets |
| Session engine | One set of authorization, replacement, retry and revocation rules |
| Volatile backend | Indexed bounded memory state and explicit loss/recovery semantics |
| Future persistence backend | Qualified published `ic-storage` integration, with backend-specific admission and host rollback |
| Issuer state | Bounded login, issuance/renewal intent, completion and recovery semantics |
| Host | Authenticated caller, trusted clock, enrollment, atomic transaction and certification root |
| Application | Users, account linking, role policy and resource ownership |

The three current product packages preserve passive protocol, Wire and Auth boundaries; issuer and receiver state remain separate optional modules within Auth.
A module earns its own abstraction only when an actual caller or independent
invariant requires it. Do not reproduce the old module tree automatically.

Canic owns application endpoint installation and dispatch using IC Wire. Auth's
Wire package supplies contract schemas, fixed identities and resource allowances;
it installs no endpoints. Canic supplies authenticated host inputs and transaction
boundaries without becoming a dependency of the Auth crates. Canic receiver hosts
own ingress throttling before fresh proof verification, periodic bounded pruning,
and capacity for retained device slots. See the [receiver scheduling and sizing guidance](../development/memory-receiver-boundary.md#host-scheduling-and-sizing).

## Storage

Hosts select volatile Auth or stable upgrade checkpoints independently for each
receiver and issuer. Both use the same bounded state transitions between upgrades.
See [checkpoint lifecycle and host obligations](DESIGN.md#stable-upgrade-checkpoints).

Storage mechanism must not select different authentication policy. Share pure decisions where that
reduces duplicate maintenance; do not force physical reads, indexing or mutation
through a universal framework. Stable reads validate persisted state; memory
invariants need not imitate serialized corruption handling.

Session/replay/login mutations commit atomically. Interrupted memory publication
must close the receiver; host transactions own rollback.
An ordinary `Err` can retain earlier writes.
Clock observations and attempted-work charges preserve the existing refusal
semantics. State loss does not reset replay/revocation evidence into authority.
Bounded pruning and indexed lookup remain requirements, not optional optimization.

Supporting volatile and stable storage does not require retaining the pre-recode
custom commit-callback API. The authorized hard cut assumes no existing consumers
need that API. New host boundaries must independently prove atomicity and recovery;
no legacy implementation or compatibility graph is required for qualification.

## Browser-bound issuer logout

`MemoryIssuer::revoke_browser` verifies a host-authenticated caller against the
retained browser key for the selected account/login epoch/id. Selection fields
never establish authority. Revocation and binding verification are synchronous;
all pending effect stages and renewal admission observe the same revocation flag.
The operation retains completed/unknown evidence, counts and other logins. It
returns the original login deadline unchanged on repeats, including retained old
epochs and expired logins; this is not receiver-wide logout or proof of remote
expiry. Unknown/pruned binding, closed state and clock regression refuse.

Canic owns transport schemas, authenticated host inputs and mounted handlers:
[Canic #16](https://github.com/dragginzgame/canic/issues/16) consumes this boundary,
[Canic #10](https://github.com/dragginzgame/canic/issues/10) owns issuer integration,
and [Canic #11](https://github.com/dragginzgame/canic/issues/11) owns the real browser
journey. [Canic #17](https://github.com/dragginzgame/canic/issues/17) owns generated
issuer codecs and the adapter to Auth's existing browser lifecycle. Receiver logout
keeps its unit-input Wire contract. An issuer adapter
must bind its original account/login selectors separately and preserve every
receiver in the browser cleanup inventory.

## Browser key custody

After terminal cleanup, `disposeLoginKey` verifies the durable generation and public
caller before invoking host custody. The host must conditionally delete that exact
binding, tolerate repeated/concurrent calls and preserve replacement keys. No private
signer is required to retry after a lost deletion reply. Terminal status preserves
unconfirmed destinations and unknown issuance, allowing the host to refuse deletion.
Cancellation fences invocation only; an invoked callback reports its actual outcome.
Auth retains the journal and makes no claim of all-copy key erasure.

## Deliberate exclusions

No Canic fleet policy, Toko records, Solana RPC, new wallet-login service or custom
cryptography. No release automation, npm publication or deployment machinery until
there is a qualified replacement to deliver. Exclude historical compatibility
readers and old token flows; incompatible stored backing must refuse without reset.

## Receiver recovery after complete volatile loss

The supported decision for an upgraded receiver with a missing predecessor is
**closed**. This includes installations where the application database, controller
identity, Root enrollment service and issuer signing key all survived. At the
upgrade boundary every former session becomes unusable. There is no authorized
caller or retry that can turn this state into a fresh receiver for the same
domain. Canic must retain the closed lifecycle on repeated initialization,
refresh and interrupted upgrade recovery. Auth's fresh constructor is for an
actual first installation.

Consider two executions of the same receiver with the same network/fleet
audience, issuer, signing key, grant generation, verified caller and signed
credential. In both, the caller admitted the credential and completed a
protected write. In one, the session remained live. In the other, logout
committed before upgrade, but the caller received no acknowledgement. Losing
the heap erases the only difference relevant to authorization. Replaying the
credential or obtaining another grant from the same Root yields identical
inputs in both histories. Choosing to admit on those inputs would undo the
committed logout in the second history. A delayed receipt establishes the
outcome of one operation at its original deadline, never the completeness
of all operations through the upgrade.

Waiting until that credential, proof, session, or grant expires cannot prove a
global cutoff. Other devices and issuers may have later deadlines. Issuer
renewals, outstanding issuance and delayed callbacks can cross the upgrade.
A new challenge or grant generation authenticates enrollment but does not
establish a new login or invalidate every predecessor credential. Even a newly
signed credential can belong to an old login. Auth has no independent record
of a cutover that covers them all.

Selecting volatile storage makes complete state loss terminal for that installation.
Hosts that need continuity across upgrades must select stable storage before the
loss and retain the checkpoint. [Auth #6](https://github.com/dragginzgame/ic-auth/issues/6)

The [installed witness](../../qualification/ic-auth-host-conformance/README.md#complete-loss-recovery-witness)
qualifies this decision for [Auth #4](https://github.com/dragginzgame/ic-auth/issues/4)
and [Auth #5](https://github.com/dragginzgame/ic-auth/issues/5).

## Issuer-attested application extension

A host can supply up to 4096 opaque bytes in `IssuerPolicy::ext`. The issuer
signs their exact length and contents. The application owns their format and
meaning. Auth preserves the distinction between `None` and `Some(vec![])`.

For Toko registration, the issuer looks up the authenticated subject in trusted
registration records. On the first sign-in it supplies `None`. After registration
it prepares a new operation on the same login with `Some(user_id.to_vec())`.
A present extension is fixed for that login. Renewal retains the bytes, and an
exact retry returns the binding captured by the original operation.

For a registered-user action, the receiver reads `SessionReceipt::ext` inside
its `authorize` projection and requires the application's 16-byte ID format.
Missing or malformed data confers no registered-user authority. Each endpoint
uses the verified local session. The browser also receives a copy for display.

Upgrade receivers before issuing extended credentials. Deploy the updated Wire
session-read and admission reply codecs together with their clients. Success
replies append optional length-prefixed bytes. Existing stable checkpoints
reopen with absent extensions. Extended checkpoints use the same memory leases.

### Selecting the issuer epoch

For a fresh login, use `MemoryIssuer::epoch()` for both `IssueRequest::epoch`
and `login_epoch`. A stable host reads the same value through
`StableIssuer::as_ref()`. Reopen advances the epoch, and pruning consumed
operation IDs can advance it between upgrades. A host whose clients construct
issue requests must expose the current value through its issuer adapter.
Persist the selected epoch with the operation ID for retries and reconciliation.
Renewal and revocation continue to identify the original login by its login
epoch and ID.
