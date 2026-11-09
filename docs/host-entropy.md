# Auth host entropy and nonce contract

Auth owns canonical credential bytes, verification and replay/session rules. The
host owns entropy acquisition, issuance, outstanding challenges, lifecycle and
atomic persistence. `ic-auth` and `ic-auth-protocol-types` neither acquire entropy
nor depend on `ic-entropy`. A nonce is signed input, not a seed, credential,
proof of key possession, trusted authority or an IC ingress delegation.

## Maintained consumers

| Consumer / owner | Bytes and scope | Unpredictability / nonzero policy | Lifetime and freshness |
| --- | --- | --- | --- |
| `PortableClaims::nonce`, Auth contract; issuer supplies it | Exactly 16 bytes; distinct credentials in one issuer/login must have distinct nonces. Exact issuance retries retain the original claims/nonce. Login binds issuer, browser/account and original window. | Hosts use a CSPRNG source or a qualified seeded generator, preserve uniqueness and refuse unavailable draws. The verifier does not treat the nonce as a secret or require nonzero bytes. Knowing it does not replace the issuer's signature. | Bound by original login and admission-proof deadlines. No standalone fresh-after-commitment requirement is imposed by this field; a host's enclosing protocol can require one. |
| Root registration challenge, Canic | 32-byte nonce bound to exact authority/fleet/issuer/key/operation/approval intent | Nonzero, unpredictable after retained intent. Existing `fresh_challenge` obtains a new management response after `registration::begin`; ready retries retain their challenge. | At most 300 seconds, capped by approval expiry; matching live retained intent and proof of possession are required. |
| Receiver boot/refresh challenge, Canic Auth | 32 bytes bound to the current receiver activation, installed Root and outstanding pull | Nonzero, fresh acquisition after the activation/pull decision; a buffered value is not evidence of fresh-after-commitment unpredictability. | Match the outstanding challenge and local receipt deadline; Root enrollment expires within the protected configured interval, at most 3,600 seconds. Upgrade/restore must not reopen from an old response. |
| Issuance authority-progress challenge, Canic Auth | 32 bytes per retained issuance intent; first callback wins. The first 16 bytes also populate the portable nonce. | Nonzero full challenge, fresh management acquisition after exact caller/login/request/key/deadlines are retained. A nonzero 32-byte challenge does not imply a nonzero 16-byte prefix. | Retained issuance deadline and fresh Root response are checked; retries reuse the retained challenge/result and post-await authority/account checks remain mandatory. |

The Canic observations are read-only, from HEAD
`3700c6e1bf78cb04a60dcfaf0f789d4dd992c9e0` plus its dirty checkout on 2026-11-09:
`canic-core/src/ops/auth/enrollment/{mod,registration/mod}.rs` and
`canic-auth/src/{workflow/issuer,workflow/receiver,refresh,ops/issuer/issuance}/mod.rs`,
under `crates/`. They describe inspected source, not a new composed qualification.
Canic's issuer-key preparation also obtains 32 bytes through the fresh helper;
key generation is a separate custody/cryptographic contract, not nonce expansion
qualified here. Wallet-login challenges remain planned; no implemented standalone
wallet challenge generator or new service is introduced by this contract.
Browser logout's `Math.random` is retry jitter only, not authentication entropy.

## Portable nonce and replay behavior

Auth signs/verifies SHA-256 of the complete canonical claims. Nonce bytes are
included unchanged for both Ed25519 and certified proofs. Replay identity is that
full digest, not the nonce alone or the signature/proof envelope. Receivers retain
consumed-proof evidence under their own application backend. The same credential
may be admitted independently at authorized receivers; within a receiver an exact
retry of the current live session/request returns its original receipt. After
replacement or removal, retained replay evidence refuses the old credential.
A repeated nonce with different claims is not independently detected as a nonce
collision: uniqueness is the issuer's responsibility. Nonces do not override
expiry, authority binding, logout, generation or recovery fences.

## Seeded and buffered host draws

Published `ic-entropy` 0.1.0 supplies bounded ChaCha20 expansion and a separate
bounded seed FIFO. Use a full 32-byte cryptographically secure seed; length,
nonzero bytes and a successful constructor do not establish seed quality. A
known/inspectable seed reveals its stream. Expansion increases output bytes, not
the seed's entropy; byte budgets are operational reseeding limits, not entropy
bit counts. Deterministic test seeds are public fixture data.

A host whose protocol permits ordinary nonce draws may use a seeded generator
with explicit byte/collision budgets and exclusive stream ownership. Dequeue a
buffered seed once. Use independent seeds for independent consumers; do not reuse
an issuer's key-generation seed as a nonce stream. If the pool is empty, the
generator is absent, or its byte budget cannot supply the full nonce, refuse or
defer before changing claims or signing. Never fall back to zeros, timestamps,
counters, a previous nonce/result or unseeded randomness. An explicit authorized
lookup of a retained exact issuance result is distinct from a new issuance.
Do not refund/reuse consumed output after an issuance or commit failure; retained
operation intents determine safe exact retries.

This permission does **not** change any inspected Canic `fresh_challenge` caller.
Its existing post-intent asynchronous management acquisition, nonzero admission,
current-context rechecks and receipt deadlines remain required. Do not satisfy
that contract from a prefilled seed, pool or already-running generator. A future
fresh-after-commitment adapter must acquire fresh source entropy after the exact
commitment and reject stale callbacks; causal freshness cannot be inferred from
output statistics. Review any change under
[Canic #757](https://github.com/dragginzgame/canic/issues/757).

## Restart, restore and confidentiality

Discard buffered seeds and expanded streams on restart, upgrade, restore,
reinstall, rollback or lifecycle replacement. Obtain fresh source entropy before
new security-sensitive issuance; do not reconstruct a stream from a saved seed
or reset its position/budget. A repeated seed reproduces output even when local
session/replay state has been restored. Loss or rollback of that safety state
continues to fail closed under the
[clock-independent recovery contract](design/portable-clock-audit.md). Complete
issuance intents/results, replay records, authority and original login windows
retain their existing atomicity/custody obligations.

On ordinary IC subnets, no seed confidentiality from subnet operators is assumed.
TEE hosting alone establishes neither seed secrecy nor rollback protection;
those require an independently qualified deployment/custody policy. The same
signature, authority, lifetime and replay checks apply to TEE and non-TEE hosts.

## Auth qualification and limits

The private `ic-auth-backend-conformance` package uses published `ic-entropy`
only as a dev-dependency. Its `entropy_tests` compose the existing fixture issuer
and real Ed25519 credential/session machinery. They cover generated nonce
admission/replacement, exact-result retry, replay refusal, empty/unseeded/short/
exhausted providers, explicit reseeding, stream repetition on reset and preserved
acceptance of signed all-zero nonce bytes. Refusal returns no credential and
preserves the prior claims; it does not return a predictable fallback.

These checks qualify Auth's supplied-input contract independently of Canic. They
do not establish production entropy provenance, a wallet issuer, timing freshness,
TEE confidentiality or Canic lifecycle/refill/cancellation acceptance. The adapter
and its matching composed evidence remain under Canic #757; no Canic source,
release or issue state is changed. See [Auth #130](https://github.com/dragginzgame/ic-auth/issues/130).
