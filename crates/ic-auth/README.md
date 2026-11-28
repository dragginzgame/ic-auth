# ic-auth

Authentication proof verification and bounded receiver and issuer state. Cryptographic primitives are supplied by IC Platform; callers own authority, time, transactions and certification publication.

## Install stateful Auth

Choose the owner roles your canister uses. Each owner requires a dedicated
stable-memory lease. For a canister with both roles and an Ed25519 receiver:

```toml
[dependencies]
ic-auth = { version = "0.52", features = ["receiver-stable", "issuer-stable", "ed25519-verification"] }
```

| Owner | Cargo feature | Fresh installation |
| --- | --- | --- |
| Receiver | `receiver-stable` | `StableReceiver::new_fresh(lease, config, now_ns)` |
| Issuer | `issuer-stable` | `StableIssuer::new_fresh(lease, config, now_ns)` |

Import the receiver types from `ic_auth::receiver_memory` and the issuer types
from `ic_auth::issuer_memory`. Ordinary operations use the owner's heap-backed
state. The upgrade hooks write and read the checkpoint.

Auth 0.52 requires fresh IC Storage 0.80 backing for stable Auth. Storage 0.79 directories and checkpoint cells cannot reopen in place.

Give each stable owner a dedicated host-owned `ic-storage` lease. Call `seal(canister_version)`
in pre-upgrade. In post-upgrade, reacquire the same lease and call
`StableReceiver::reopen(lease, config, now_ns, canister_version)` or
`StableIssuer::reopen(lease, config, now_ns, canister_version)` with the original configuration
and current trusted time. During upgrade the host may increase only the receiver’s `login.max_retained_device_slots` within the checkpoint budget. Read the canister version inside each hook. Only the
immediate successor can open the current v1 checkpoint for each owner. Pre-cut
receiver and issuer images are refused. Trap on failure so the IC rolls back the upgrade.
Receiver checkpoints hold up to 64 MiB of encoded state. Issuer checkpoints hold up to 8 MiB. Refresh the receiver's
authenticated enrollment before serving sessions. An installation with missing Auth state stays closed.

The receiver starting profile allows 20,000 session rows, 80,000 replay rows,
and 65,536 retained device slots, subject to the 32 MiB session/replay byte budget.
The starting receiver accepts logins up to one hour. The issuer still chooses
the signed login lifetime. A receiver configuration that can exceed the 64 MiB
checkpoint budget is refused before it opens a store.
Schedule bounded pruning to reclaim slots after their protection deadlines. Use `SessionTtlPolicy::default()`
for a 15-minute default and a 30-minute request ceiling, or set a host policy
explicitly. Login and issuer authority deadlines cap each lease. A current
enrollment snapshot is required at admission and on every protected call.
A same-generation refresh allows an unexpired session to resume with its
original deadline.

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
