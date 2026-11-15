# Product packages

Product packages belong here, with each directory matching its Cargo package
name. Register actual member paths explicitly in the repository-root virtual
workspace. Current packages:

- `ic-auth-protocol-types`: passive claims, validated labels and Platform principals.
- `ic-auth`: canonical claims codec, unsigned request identity and protected TTL bounds.

Both are unpublished partial implementations, not replacement authentication
engines. See the [request](../docs/development/request-boundary.md) and
[claims](../docs/development/claims-boundary.md) boundaries.

Add packages only with their first meaningful implementation. Ordinary unit and
integration tests stay with their product package. Follow the
[authentication design](../docs/design/authentication.md) and
[tooling setup](../docs/development/tooling.md) when adding members.
