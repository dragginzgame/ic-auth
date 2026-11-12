# Account and device capacity audit — 2026-11-10

Status: the receiver contract is now implemented locally; publication and issuer
adoption remain outstanding. It supersedes the rejected one-device restriction,
not published 0.37 behavior. The findings and prototype below record the design
investigation. Canic inspection and all sibling activity remained read-only.

The implementation signs a slot (0–7) and nonzero generation in schema 2, retains
known slot fences after expiry/logout, and atomically replaces only a verified
higher-generation slot with a different browser key. The host can tighten the
eight-slot ceiling through `with_device_limit` before admission/restore. Replay
records retain account ownership. Durable callbacks receive the displaced caller
for atomic old-row removal; failed callbacks leave the prior device intact.
There is no account retirement timer or implicit deletion of known fences.
See [the handoff](../status/current.md) for current evidence and adoption limits.

On 2026-11-10 the maintainer explicitly authorized retiring the old Auth format.
The local storage boundary now rejects all three Auth 0.37 record identities and
the old qualification checkpoint envelope without clearing backing. This removes
the retained-format compatibility requirement for the new Auth implementation;
it does not authorize resetting Canic state.

## Required identity and behavior

One person can use several devices under the same authenticated account principal.
Each device keeps its own browser key; sharing a private key across devices is
unnecessary. Auth account identity remains `(issuer, subject)` within the installed
audience. Equal subjects asserted by different issuers are not automatically the
same account. Account linking remains application-owned.

The account needs a bounded aggregate of device state. One outer map entry is not
a storage bound if it contains an unbounded vector, map, history or journal.
Multiple physical indexed rows are also acceptable when their aggregate count and
bytes are bounded per account and committed atomically.

## Findings from the current code

| Finding | Consequence | Required correction |
| --- | --- | --- |
| The unpublished account indexes permit one browser caller. | Second-device login is refused; known logout can retain the exclusion for 90 days. | Supersede this restriction; passing tests do not validate the rejected product behavior. |
| Published sessions are caller-keyed and renewal replaces a caller's session. | Device separation already works, but caller counts do not bound one account. | Retain independent caller authentication and add account-wide accounting. |
| Known login reservations and tombstones survive to the signed login deadline. | Limiting only active sessions misses retained logout storage. Repeated login/logout can fill a finite retained-login allowance. | Bound active and retained state separately; define safe reuse before choosing a device count. |
| Published replay records retain caller/fingerprint/generation, without account identity. | Once a session disappears, a new account quota cannot safely infer replay ownership from that session. | Implemented locally: retain issuer/subject in each replay, aggregate the quota across keys and rebuild account counts on restore. The slot protocol remains pending. |
| Session byte budgets exclude heap/index overhead. | Record counts alone are not an actual-memory estimate. | Bound encoded bytes and measure indexes, reconstruction and transient allocations. |
| Canic issuer login selection uses account/caller + epoch + login ID. | One account can consume its 64 retained-login slots. | Issuer-owned per-account admission, logical-login reuse and lifecycle controls are needed separately. |
| Canic also has a separate globally bounded issuance journal (32 rows). | Fixing its login map alone leaves an issuance-history exhaustion path. | Bound outstanding and retained issuance work per account, preserving retry evidence. |
| Unknown logout authenticates a browser caller but cannot establish an account. | Such traffic cannot honestly be charged to a claimed account. | Preserve its separate global pool and work budget; do not trust submitted account fields. |
| Browser journal limits are local to a dedicated database. | They do not constrain adversarial canister requests. | Enforce authoritative limits on the receiver and issuer. |

Sources: [session limits/records](../../crates/ic-auth/src/session/model.rs),
[reference backend](../../crates/ic-auth/src/session/memory.rs),
[login ledger](../../crates/ic-auth/src/portable/revocation/mod.rs),
[signed claims](../../crates/ic-auth-protocol-types/src/portable.rs), and
[browser journal](../../packages/client/src/logout-store.ts).
Read-only issuer evidence is in Canic's
`crates/canic-auth/src/ops/issuer/issuance/{mod.rs,retention/mod.rs}`.

## Recommended design direction

Use host-configured per-account limits for active devices, retained security
metadata, replay records and total bytes, in addition to global limits. Eight
device slots is an illustrative sizing input, not an approved default. A device
means a browser-key binding; the server cannot reliably identify physical hardware.

For reusable slots, investigate an issuer-authenticated `(slot, generation)` in
the signed credential. The slot is drawn from a bounded range; its generation
advances under authenticated account control when replacing a device binding.
Browser renewal must not allocate slots or advance generations on its own.

Each receiver retains a monotonic generation and revoked/live state for each
account slot. After authenticating a newer generation, it can replace the slot's
binding atomically. Older generations then refuse; equal generations must keep
the same browser binding and original login window, and cannot undo logout.
Generation overflow refuses rather than wrapping. All existing proof, issuer,
authority, audience, scope and lifetime checks remain necessary.

This is a candidate protocol, not a claim that a counter alone solves revocation.
Current claims have no signed slot/generation. Introducing them changes signed
bytes and needs an explicit protocol decision, both proof mechanisms, issuer
support, client integration and retained-state disposition. Auth cannot complete
issuer behavior by itself.

Generation metadata cannot simply be pruned or reset: doing so could admit old
credentials again. Keep bounded per-slot metadata for the retained account; global
account capacity and authenticated account retirement/recovery are separate
requirements. The existing clock/lost-state contract remains binding.

No receiver-to-issuer call is required for ordinary authorization or proposed
local generation checks. Browser-delivered credentials can carry new generations.
A receiver that has not received the replacement/revocation cannot know it happened.
Immediate fleet-wide logout or device removal is therefore not promised; fan-out,
existing leases and proof acceptance windows remain explicit limitations. Local
logout acknowledges the exact known binding, not unseen future logins.

At a full device allowance, refuse additional device admission without evicting
another live device. An account-authenticated device-management flow may explicitly
replace a selected slot once the replacement protocol is qualified. Re-login after
logout must use a fresh authenticated generation, not wait out the old 90-day login.

## Bounds and performance acceptance

Let A be the maximum retained accounts, D the device slots per account, and R the
aggregate replay allowance per account. Require at most A×D session bindings and
A×D compact slot records, and at most min(global replay limit, A×R) replay rows.
Also enforce per-account/global byte ceilings and a separate unknown-logout limit.
These formulas exclude host allocation overhead and issuer history, which need
their own budgets. A limit controls one authenticated account, not a human who can
obtain many accounts; account admission policy remains necessary.

Keep indexed caller lookup for ordinary requests and indexed account accounting.
Do not scan all sessions or serialize an entire account history on every call.
Compare matched builds for admission, renewal, ordinary reads, logout, slot reuse,
full-capacity refusal, bounded pruning and restore. Measure Wasm bytes, instructions,
encoded storage, heap/index memory and peak reconstruction allocation. No performance
improvement or acceptable overhead has been established by this audit.

## Required acceptance cases before implementation is called complete

- Several devices on one account work concurrently; logout of one preserves others.
- Renewal and additional destinations reuse the same logical device login.
- Device/key churn cannot grow account storage beyond its count and byte budgets.
- Repeated logout and fresh login reuse slots without waiting 90 days; old proofs
  and retained old keys cannot resurrect an earlier generation.
- Concurrent replacement, old async issuer callbacks, lost replies and exact retries
  preserve the selected generation and original effects.
- Old-device logout cannot revoke a replacement device or another account.
- Account quotas include retained replay/denial evidence; another account still
  admits after an attacker saturates its own allowance.
- Failed commits, upgrade restoration, clock rollback, backup rollback, state loss
  and generation overflow preserve denial and ownership; missing state stays closed.
- Unknown logout flooding remains bounded independently of authenticated accounts.

Audit method: source and contract review. Earlier focused tests establish behavior
of the unpublished one-device implementation only. They do not qualify this design.

## Hard-cut feasibility and first performance measurements

The maintainer authorized investigating a hard cut after this audit. The candidate
requires signed device-slot and generation fields, a new signed-format identity,
bounded codecs, both verifier paths, account-aware replay ownership and durable
slot high-water marks. Issuer slot allocation must authenticate the account, bind
the browser key, preserve exact retries and reject stale asynchronous callbacks.
Fresh login advances a selected slot; ordinary renewal retains its generation and
original window. Receiver admission authenticates a newer generation before
atomically replacing the slot's caller/session and publishing its high-water mark.
Unauthenticated preflight cannot advance a counter or invalidate another device.

This can use the existing maintained cryptography. No additional signature check
is inherently required by adding fields to the existing signed claims. Extra
encoding/hash work is expected, but its size and cost are not measured yet. The
issuer and receiver cannot independently choose incompatible slot semantics.
Hard-cut authorization does not authorize deleting Canic's retained production
state; consumer inventory and disposition still precede rollout.

An isolated [native prototype](../../crates/ic-auth-tooling/benches/device_slots.rs)
compares identical caller indexes with/without an additional account-slot lookup.
Eight slots each retain a u64 generation and revoked flag. The model omits full
session payloads, browser-binding replacement, replay, proof verification, Wire
encoding, stable writes, atomic transactions and recovery. It is not production
authorization code or an implementation of the proposed protocol.

Command: `cargo bench --locked --offline -p ic-auth-tooling --bench device_slots`.
Optimized native build on AMD Threadripper 7970X, Rust 1.99.1. Timing values are
medians of seven batches of 100,000 deterministic dispersed operations. DHAT tracks
live heap allocations; setup and destruction are outside lookup timing. The host
was not isolated from other workloads. These are indicative native observations,
not confidence intervals, Wasm instructions, IC cycles or production latency.

| Accounts | Devices | Caller lookup only | Lookup + slot check | Slot metadata update only | Additional account index heap |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 8,000 | 85 ns | 167 ns | 84 ns | 192,128 B |
| 10,000 | 80,000 | 120 ns | 325 ns | 136 ns | 1,902,272 B |
| 100,000 | 800,000 | 276 ns | 801 ns | 347 ns | 18,986,304 B |

The extra index is approximately 190 bytes/account in this layout, around 18.1 MiB
for 100,000 accounts. The shared caller index alone uses 57,602,368 bytes at that
scale; neither number includes real session payloads. The native slot struct is
16 bytes with padding. Wire and stable-memory sizes require their actual layouts.
The measured 0.525 microsecond extra lookup at the largest size is not a prediction
of the final authorization overhead: the final design may combine or replace
existing ledger lookups. Whole-account rewrites could dominate stable-storage cost,
so retain narrow atomic slot updates and measure the actual adapter.

Raw output: `target/device-slots-profile.log`. Product lock SHA-256 during the run:
`b06339f6727f0cec673fbc9e3db6a5729d53f19bee69abafcc6a430c5d978846`.
No Wasm-size delta or cryptographic speedup is claimed. The next meaningful
performance gate needs an executable protocol prototype and matched installed
Wasm measurements, including churn/restore at the configured limits.
