# ic-auth

Authentication proof verification and bounded receiver and issuer state. Cryptographic primitives are supplied by IC Platform; callers own authority, time, transactions and certification publication.

## Select storage on fresh installation

Choose storage independently for the receiver and issuer. For example, a host
with both owners and an Ed25519 receiver can use this dependency:

```toml
[dependencies]
ic-auth = { version = "0.49.2", features = ["receiver-stable", "issuer-stable", "ed25519-verification"] }
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

Set `config.login.max_admitted_logins` before receiver installation to cover the
distinct issuer, account and device-slot combinations the shard will serve.
The starting limit is 1,024. Session expiry retains each slot’s generation fence.

For a fresh login, use the current issuer epoch for both request epoch fields.
A stable host can read it with `issuer.as_ref().epoch()` after reopen. Retain the
original operation epoch for reconciliation and the login epoch for renewal and
revocation.

## Optional permissions within an endpoint

Use `authorize_with_scopes` when a response depends on an additional permission.
Pass the mandatory scopes to the receiver and check optional scopes inside the
callback. For example, a profile endpoint can include private fields only when
the admitted session holds the additional scope.

```rust
receiver.authorize_with_scopes(&authority, &[profile_read], |session| {
    let include_private = session.has_scope(&profile_private_read);
    load_profile(session.receipt.issuer, session.receipt.subject, include_private)
})
```

See the
[repository](https://github.com/dragginzgame/ic-auth) for design, features and
qualification evidence. Licensed under the included Toko Studios Network Licence.
