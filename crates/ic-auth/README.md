# ic-auth

Authentication proof verification and bounded receiver and issuer state. Cryptographic primitives are supplied by IC Platform; callers own authority, time, transactions and certification publication.

## Select storage on fresh installation

Choose storage independently for the receiver and issuer. For example, a host
with both owners and an Ed25519 receiver can use this dependency:

```toml
[dependencies]
ic-auth = { version = "0.48.9", features = ["receiver-stable", "issuer-stable", "ed25519-verification"] }
```

| Owner | Volatile feature and constructor | Stable feature and constructor |
| --- | --- | --- |
| Receiver | `receiver-memory` · `MemoryReceiver::new_fresh(config, now_ns)` | `receiver-stable` · `StableReceiver::new_fresh(lease, config, now_ns)` |
| Issuer | `issuer-memory` · `MemoryIssuer::new_fresh(config, now_ns)` | `issuer-stable` · `StableIssuer::new_fresh(lease, config, now_ns)` |

Import the receiver types from `ic_auth::receiver_memory` and the issuer types
from `ic_auth::issuer_memory`. Each stable feature includes its memory feature,
so a single build can select either constructor from trusted install configuration.
Use `state()` on a stable owner to run the same operations as its volatile counterpart.

Give each stable owner a dedicated host-owned `ic-storage` lease. Call `seal()`
in pre-upgrade. In post-upgrade, reacquire the same lease and call
`StableReceiver::reopen(lease, config, now_ns)` or
`StableIssuer::reopen(lease, config, now_ns)` with the original configuration
and current trusted time. Trap on failure so the IC rolls back the upgrade.
Each checkpoint holds at most 8 MiB of encoded state. Refresh the receiver's
authenticated enrollment before serving sessions. A volatile installation
that loses its state stays closed.

See the
[repository](https://github.com/dragginzgame/ic-auth) for design, features and
qualification evidence. Licensed under the included Toko Studios Network Licence.
