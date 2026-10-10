# Chain-key Merkle batch construction

Published **0.2.11** includes `ic_auth::chain_key_batch::merkle_root_and_witnesses`,
tracked in [#15](https://github.com/dragginzgame/ic-auth/issues/15).
The constructor needs no optional feature,
runtime, signing key, host tool or storage backend.

Pass ordered canonical certificate hashes and a protected maximum leaf count.
Compute each leaf with `canonical::chain_key_delegation_cert_hash`; Canic retains
issuer approval, sorting by principal bytes, duplicate issuer rejection and its
64-issuer limit. The constructor preserves every input position, including
duplicate hashes, and returns one witness per position. It does not authorize
issuers, choose a limit, build a batch header or sign anything.

Empty input returns `ChainKeyBatchError::EmptyBatch`; a nonempty input above the
supplied limit returns `TooManyLeaves { found, max }`, including a zero limit.
Both checks precede tree/witness allocation. With `n` admitted leaves, each
witness has at most `ceil(log2(n))` steps. Host policy bounds the input and total
returned material; the API does not impose Canic's numeric limit on other hosts.

The frozen construction is `SHA256(0x01 || left || right)` for each internal
node. An unpaired node advances unchanged, without a duplicate sibling or extra
step. A singleton's root is its leaf hash and its witness is empty. Left and
right witness variants retain their existing protocol meanings and Candid
encoding. Changing leaf order changes the committed tree; the library never
sorts or normalizes it.

Assign the returned root to the existing batch header and zip witnesses with
the same authorized leaf order. Canic continues to own batch identity, epochs,
key enrollment, management-canister signing, persistence and retry/renewal.
The complete token and standalone delegation verifiers reconstruct these same
node bytes after their existing input-size admission, then authenticate the
header signature and live protected authority. A root or witness alone is not
an authenticated delegation.

Tests preserve the immutable Canic golden root and witness directions, check
every batch shape through its 64-leaf capacity, reject empty/over-budget input
and use a generated witness with an existing genuinely signed token without
changing its canonical proof hash. These are library checks; actual Canic
adoption and retirement remain in [Canic #491](https://github.com/dragginzgame/canic/issues/491).
