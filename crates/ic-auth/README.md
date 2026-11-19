# ic-auth

Authentication proof verification and bounded receiver and issuer state. Cryptographic primitives are supplied by IC Platform; callers own authority, time, transactions and certification publication.

Use `receiver-memory` or `issuer-memory` for volatile state. Select
`receiver-stable` (with a verifier) or `issuer-stable` for upgrade persistence.
Create each stable owner on a fresh host-owned `ic-storage` lease. Call `seal`
in pre-upgrade and `reopen` in post-upgrade with the same configuration. Trap
on failure. Each checkpoint holds at most 8 MiB of encoded state. The receiver
requires fresh authenticated enrollment after reopening. See the
[repository](https://github.com/dragginzgame/ic-auth) for design, features and
qualification evidence. Licensed under the included Toko Studios Network Licence.
