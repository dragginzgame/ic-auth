# Changelog

Auth has two independent release groups:

- [Protocol](releases/protocol/CHANGELOG.md): `ic-auth-protocol-types` and `ic-auth-wire`.
- [Engine](releases/engine/CHANGELOG.md): `ic-auth` and unpublished qualification/tooling packages.

[Coordinated history through 0.29.2](releases/history.md) retains the earlier shared
releases and `v…` tag identities. Package versions remain unchanged by the split.
Select `RELEASE_GROUP=protocol` or `RELEASE_GROUP=engine` explicitly for releases
and publication; each group keeps its own notes and `<group>-v…` tags.
