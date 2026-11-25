# Auth protocol types changelog

The `protocol-types` group covers `ic-auth-protocol-types`. Its separate ledger follows [0.43.2](../history.md#0486---2026-11-19).

## [0.44.1] - 2026-11-25

### 📦 Dependencies

- Adopt IC Platform Types 0.21.5 for protocol principals.

## [0.44.0] - 2026-11-22

### ⚠️ Breaking

- Add `ext: Option<Vec<u8>>` to `PortableClaims`. Use `None` for pre-registration claims. [#8 Carry an issuer-attested application user ID into verified sessions](https://github.com/dragginzgame/ic-auth/issues/8)

## [0.43.3] - 2026-11-21

### 📖 Documentation

- Update the Rust API documentation for protocol values to the Toko Tooling 0.46.0 type-header format.
