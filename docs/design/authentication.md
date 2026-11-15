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
| Future persistence backend | Qualified `ic-storage-new` orthogonal persistence, with backend-specific admission and host rollback |
| Issuer state | Bounded login, issuance/renewal intent, completion and recovery semantics |
| Host | Authenticated caller, trusted clock, enrollment, atomic transaction and certification root |
| Application | Users, account linking, role policy and resource ownership |

The future package split should preserve passive protocol and Wire boundaries.
A module earns its own abstraction only when an actual caller or independent
invariant requires it. Do not reproduce the old module tree automatically.

Canic owns application endpoint installation and dispatch using IC Wire. Auth's
Wire package supplies contract schemas, fixed identities and resource allowances;
it installs no endpoints. Canic supplies authenticated host inputs and transaction
boundaries without becoming a dependency of the Auth crates.

## Storage

Volatile authentication is the current implementation milestone. Stable storage
is explicitly deferred to a future milestone while Storage is being developed;
no interim `ic-storage` adapter is planned. The intended durable backend remains
`ic-storage-new` with orthogonal persistence. Its future integration depends on
qualified supplier capabilities, not an assumed API. Keep semantic rules reusable
and avoid a speculative universal storage abstraction. See the detailed
[storage transition](DESIGN.md#intended-orthogonal-persistence-destination).

Storage mechanism must not select different authentication policy. Share pure decisions where that
reduces duplicate maintenance; do not force physical reads, indexing or mutation
through a universal framework. Stable reads validate persisted state; memory
invariants need not imitate serialized corruption handling.

Session/replay/login mutations commit atomically. Interrupted memory publication
must close the receiver; host transactions own rollback.
An orthogonal-persistence access scope does not itself provide rollback;
ordinary `Err` must not be treated as undoing earlier writes.
Clock observations and attempted-work charges preserve the existing refusal
semantics. State loss does not reset replay/revocation evidence into authority.
Bounded pruning and indexed lookup remain requirements, not optional optimization.

Supporting volatile and stable storage does not itself decide the future public
custom commit-callback API. Preserve its demonstrated guarantees in the acceptance
reference; do not promise or remove downstream API compatibility without reviewing
actual consumers. Rust API simplification may differ from Wire compatibility.

## Deliberate exclusions

No Canic fleet policy, Toko records, Solana RPC, new wallet-login service or custom
cryptography. No release automation, npm publication or deployment machinery until
there is a qualified replacement to deliver. Exclude historical compatibility
readers and old token flows; incompatible stored backing must refuse without reset.
