# Product packages

The virtual root workspace contains three product libraries:

- `ic-auth-protocol-types`: passive validated protocol values and Platform principals.
- `ic-auth`: proof verification and bounded volatile receiver/issuer decisions.
- `ic-auth-wire`: credential, admission, read and logout transport contracts.

All three are authorized for crates.io publication. See the
[publication scope](../docs/development/publication.md) and
[current status](../docs/status/current.md) for qualification and integration limits.
Ordinary tests stay with their product package; independent qualification packages
remain in the separate, nonpublishable `qualification/` workspace.
