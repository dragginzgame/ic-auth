# IC Auth changelog

The `auth` group covers `ic-auth`. Its separate ledger follows [0.48.7](../history.md#0487---2026-11-19).

## [0.51.1] - 2026-11-26

### 🔧 Changed

- Use IC Wire 0.31.1 for stable checkpoint encoding and Auth protocol-types 0.44.2 for principals.
- Adopt authenticated Toko Tooling 0.46.8 in both workspaces.

## [0.51.0] - 2026-11-26

### ⚠️ Breaking

- Stateful Auth requires a stable lease. Hosts select `issuer-stable` or `receiver-stable` and initialize through `StableIssuer` or `StableReceiver`. Ordinary operations use heap indexes.
- Use one current v1 checkpoint format per owner and refuse pre-cut images. The receiver stores rows in bounded chunks and rebuilds derived indexes. Fresh receivers default to 20,000 sessions, 80,000 replay rows and 65,536 device slots.
- Provide a 15-minute default session policy with a 30-minute request ceiling. The starting receiver profile accepts original logins up to one hour. The receiver starting profile gives Ed25519 more admission headroom and weights certified verification by its higher measured cost.

### 🐛 Fixed

- Reject receiver profiles that cannot fit the checkpoint byte budget. Count session and replay bytes from their compact checkpoint encoding, recompute charges on reopen, and seal directly into one image buffer.
- Treat enrollment freshness as a current authorization check. Same-generation refresh restores access through the original session deadline, and changed authority generations still invalidate old sessions.

### 📦 Dependencies

- Adopt authenticated Toko Tooling 0.46.5 in both workspaces.

## [0.50.0] - 2026-11-25

### ⚠️ Breaking

- Stable issuer and receiver checkpoints require the trusted canister version in both `seal` and `reopen`. A skipped reopen permanently fences that image. V1 and v2 images cannot upgrade in place to this format. Hosts must trap on refusal and reserve fresh initialization for a genuinely new installation. [#10](https://github.com/dragginzgame/ic-auth/issues/10)

### 🐛 Fixed

- Keep a displaced browser logged out through its promised deadline, including after stable reopen. Replacement can return capacity when the unbound denial pool is full. [#9](https://github.com/dragginzgame/ic-auth/issues/9)
- Reserve issuer credit for newly retained intents so one account’s capacity refusals leave credit available for other accounts. [#11](https://github.com/dragginzgame/ic-auth/issues/11)
- Issuer pruning releases device slots from abandoned preparations and revoked logins once their last operation leaves retention. [#12](https://github.com/dragginzgame/ic-auth/issues/12)
- A browser can prepare the next renewal after the issuer prunes its fenced predecessor. The original sequence stays consumed across upgrades. [#17](https://github.com/dragginzgame/ic-auth/issues/17)
- Stable reopen validates issuer renewal sequences, deadlines and generations, recomputes receiver session sizes and checks each application binding against its caller and credential fingerprint. [#16](https://github.com/dragginzgame/ic-auth/issues/16)
- Browser logout cleanup continues to other due destinations after a failed transport or malformed reply, while retaining each destination’s uncertainty and attempt limit. [#13](https://github.com/dragginzgame/ic-auth/issues/13)
- Restore issuer-only library test compilation and check the issuer and individual receiver verifier builds in Linux CI. [#14](https://github.com/dragginzgame/ic-auth/issues/14)
- Browser calls hold capacity until their callbacks settle and restrict each poll to the latest signed update. Plaintext gateway URLs require loopback. [#19](https://github.com/dragginzgame/ic-auth/issues/19)
- Qualify full certified admission against fresh and frozen PocketIC application-subnet delegations. [#15](https://github.com/dragginzgame/ic-auth/issues/15)

### 📚 Documentation

- Document receiver host throttling, pruning and sizing for retained device slots. [#18](https://github.com/dragginzgame/ic-auth/issues/18)

### 📦 Dependencies

- Stable issuer and receiver checkpoints now require fresh IC Storage 0.80 backing. Both memory and stable modes use IC Wire 0.30.2 or later in the 0.30 line.
- Adopt IC Platform 0.35.1 for Auth verification and Platform Runtime 0.23.27 for qualification. Both workspaces use Toko Tooling 0.46.4. Storage now owns its Cell engine, removing the transitive Platform 0.34 dependency.

## [0.49.3] - 2026-11-24

### 📦 Dependencies

- Use IC Wire 0.30.0 for stable checkpoints, renewal operation identifiers and receiver size accounting. Both workspaces use Toko Tooling 0.46.3.

## [0.49.2] - 2026-11-23

### 📚 Documentation

- Document sizing for retained receiver device slots and epoch selection for fresh sign-ins. A stable-reopen regression test covers stale epoch refusal and successful fresh issuance.

## [0.49.1] - 2026-11-22

### ✨ Added

- Receiver endpoints can use `authorize_with_scopes` to check optional permissions with `session.has_scope(&scope)` inside a guarded operation. It checks the scopes retained at session admission.

## [0.49.0] - 2026-11-22

### ⚠️ Breaking

- Bind up to 4096 optional bytes of issuer-supplied application data to each issuance operation. Verified session reads, renewal and stable reopen retain the binding. [#8 Carry an issuer-attested application user ID into verified sessions](https://github.com/dragginzgame/ic-auth/issues/8)

### 📦 Dependencies

- Adopt IC Platform 0.34.19, Platform Types 0.21.4 and qualification Runtime 0.23.25. Both workspaces use Toko Tooling 0.46.2.

## [0.48.9] - 2026-11-21

### 📦 Dependencies

- Use IC Wire 0.28.0 for stable checkpoints, renewal request identities and
  receiver session-size accounting.

### 🔧 Tooling

- Adopt Toko Tooling 0.46.0 in both workspaces. CI verifies the shared signing
  fixtures once on Linux and runs native Rust contracts on all three hosts.

## [0.48.8] - 2026-11-20

### 🔧 Tooling

- Adopt Toko Tooling 0.45.25 across both workspaces and remove the offline
  requirement from the receiver-policy test instructions.
- Declare separate release groups for Auth, Auth Wire and protocol types so Toko
  validates each package’s own version. [#7 Reconcile shared dependency-pin and inheritance audit findings](https://github.com/dragginzgame/ic-auth/issues/7)
- Use IC Wire 0.27.1 for issuer and receiver storage codecs.
