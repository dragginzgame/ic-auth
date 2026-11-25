# ic-auth

Authentication proof verification and bounded receiver and issuer state. Cryptographic primitives are supplied by IC Platform; callers own authority, time, transactions and certification publication.

## Select storage on fresh installation

Choose storage independently for the receiver and issuer. For example, a host
with both owners and an Ed25519 receiver can use this dependency:

```toml
[dependencies]
ic-auth = { version = "0.50.0", features = ["receiver-stable", "issuer-stable", "ed25519-verification"] }
```

| Owner | Volatile feature and constructor | Stable feature and constructor |
| --- | --- | --- |
| Receiver | `receiver-memory` · `MemoryReceiver::new_fresh(config, now_ns)` | `receiver-stable` · `StableReceiver::new_fresh(lease, config, now_ns)` |
| Issuer | `issuer-memory` · `MemoryIssuer::new_fresh(config, now_ns)` | `issuer-stable` · `StableIssuer::new_fresh(lease, config, now_ns)` |

Import the receiver types from `ic_auth::receiver_memory` and the issuer types
from `ic_auth::issuer_memory`. Each stable feature includes its memory feature,
so a single build can select either constructor from trusted install configuration.
Use `state()` on a stable owner to run the same operations as its volatile counterpart.

Auth 0.50 requires fresh IC Storage 0.80 backing for stable Auth. Storage 0.79 directories and checkpoint cells cannot reopen in place.

Give each stable owner a dedicated host-owned `ic-storage` lease. Call `seal(canister_version)`
in pre-upgrade. In post-upgrade, reacquire the same lease and call
`StableReceiver::reopen(lease, config, now_ns, canister_version)` or
`StableIssuer::reopen(lease, config, now_ns, canister_version)` with the original configuration
and current trusted time. Read the canister version inside each hook. Only the
immediate successor can open the v3 checkpoint. Older checkpoint formats refuse. Trap on failure so the IC rolls back the upgrade.
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

When a renewal is fenced by an upgrade or signer change, reconcile its original
sequence while its operation is retained. After the host prunes the operation at
the later of its admission and receipt deadlines plus reconciliation retention,
the original browser can request the next sequence with its original login and
grants. Use `preflight_renewal` to check that sequence and recheck application
permission before preparing it. A missing lookup alone cannot establish eligibility.

On the issuer, prune abandoned preparations to reclaim device slots. Revocation
also makes a slot reclaimable once all retained operations for that login have
been pruned. Newly idle logins are removed on a subsequent bounded prune call.

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
