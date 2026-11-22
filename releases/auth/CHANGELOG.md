# IC Auth changelog

The `auth` group covers `ic-auth`. Its separate ledger follows [0.48.7](../history.md#0487---2026-11-19).

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
