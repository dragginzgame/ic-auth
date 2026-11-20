# IC Auth changelog

The `auth` group covers `ic-auth`. Its separate ledger follows [0.48.7](../history.md#0487---2026-11-19).

## [0.48.8] - 2026-11-20

### 🔧 Tooling

- Adopt Toko Tooling 0.45.25 across both workspaces and remove the offline
  requirement from the receiver-policy test instructions.
- Declare separate release groups for Auth, Auth Wire and protocol types so Toko
  validates each package’s own version. [#7 Reconcile shared dependency-pin and inheritance audit findings](https://github.com/dragginzgame/ic-auth/issues/7)
- Use IC Wire 0.27.1 for issuer and receiver storage codecs.
