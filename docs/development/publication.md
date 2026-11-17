# Crates.io publication scope

The maintainer authorized publication of Auth 0.47.4. Protocol-types/Wire
remain at their already published 0.43.0 versions. All three product crates use `publish = true`; the destination is crates.io.
Publication covers the implemented volatile libraries, not production deployment.

| Product crate | Published responsibility | In-repository prerequisite |
| --- | --- | --- |
| `ic-auth-protocol-types` | Passive validated authentication protocol values | None |
| `ic-auth` | Authentication verification and bounded volatile receiver/issuer decisions | `ic-auth-protocol-types` |
| `ic-auth-wire` | Credential, admission, read and logout Wire contracts; optional TypeScript generation | `ic-auth-protocol-types` |

Publish the selected protocol-types version first. Once that exact required
version is available from crates.io, Auth and Auth Wire can publish independently;
neither depends on the other. All product-to-product Cargo dependencies must retain
explicit registry version requirements alongside their local development paths.
Changed supplier versions must be available before dependent uploads. Do not bump
or republish an unchanged product solely to make all three version numbers match.

The independent `qualification/` workspace is permanently outside this publication
scope: `ic-auth-certified-conformance`, `ic-auth-host-conformance`,
`ic-auth-host-fixture` and `ic-auth-wire-conformance` remain `publish = false`.
Product packages must not acquire dependencies on these packages. The private
`@dragginzgame/ic-auth-client` browser package is not a Rust crate and is not a
crates.io publication candidate; npm delivery requires its own future decision.
Do not introduce empty crates, endpoint installers or storage adapters to expand
this inventory.

## Upload authorization boundary

At the authorized publication milestone, review each affected product package's
actual archive and public feature/dependency surface, confirm the intended
crates.io destination, and qualify registry resolution in dependency order.
Verify package contents, repository/documentation metadata, required source and
fixtures, and consumer compilation against uploaded supplier versions. Recheck
crates.io name ownership and existing versions then; this plan does not claim
that any name or version is currently available. Confirm each upload before
publishing its dependents.

This release uses Cargo directly with exact package selectors, locked dependencies
and all features. No local publication automation is installed. Future shared
entrypoints must be adopted through the authenticated Toko exporter. Production deployment, consumer migration and live-storage changes
remain separate from crates.io delivery. Stable storage remains a future milestone;
this scope does not create a storage package or make one a prerequisite for the
currently implemented volatile crates.
