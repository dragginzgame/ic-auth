# Auth Wire changelog

The `wire` group covers `ic-auth-wire`. Its separate ledger follows [0.45.0](../history.md#0485---2026-11-19).

## [0.48.0] - 2026-11-22

### ⚠️ Breaking

- Append the optional bounded application extension to successful session admission and read replies. Update host encoders and browser decoders together. [#8 Carry an issuer-attested application user ID into verified sessions](https://github.com/dragginzgame/ic-auth/issues/8)

## [0.47.0] - 2026-11-21

### ⚠️ Breaking

- Auth Wire now re-exports IC Wire 0.28.0. Consumers passing Wire types across
  crate boundaries should use the re-export or align their direct dependency to 0.28.

## [0.46.0] - 2026-11-20

### ⚠️ Breaking

- Re-export IC Wire 0.27.1. Consumers passing Wire types across crate boundaries
  should use the re-export or update their direct dependency to 0.27.
