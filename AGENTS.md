# IC Auth

Read the installed [Toko Tooling baseline](TOKO-TOOLING.md). Its common rules
apply alongside the product constraints below. Tooling 0.45.7 is authenticated
by `.toko-tooling.snapshot`; [developer tooling](docs/development/tooling.md)
describes Cargo-based admission and focused checks.

Read README.md, docs/design/DESIGN.md, docs/design/authentication.md,
 docs/design/compatibility.md and
 docs/status/current.md before implementation.

- This repository is an explicitly authorized clean restart. The sibling
  `../ic-auth-legacy` is read-only reference, not an implementation destination.
- The maintainer authorized the pre-recode hard cut in Auth #2: no existing
  users/deployments require compatibility with that implementation. Maintain one
  current Auth contract, independent fixtures and Rust/browser interoperability.
  Historical Wire/API identities and layouts are provenance, not preservation
  requirements. Do not execute the legacy implementation as a test oracle or
  enable the retired legacy-compatibility qualification graph.
- Volatile authentication storage is the current milestone. Stable storage is
  deferred to a future milestone; do not build an interim `ic-storage` adapter.
  Keep semantic rules reusable without a speculative storage abstraction.
- Future stable storage belongs at the published `ic-storage` boundary. The
  supplier retired its OP experiment; do not prescribe the former `ic-storage-new`
  runtime. At the separately selected stable-storage milestone, review the current
  allocation/collection contract and qualify lifecycle, corruption admission and
  rollback. This does not expand the current volatile milestone.
- Host inputs own caller, time, authority, memory leases, transactions and
  certification-root publication. Untrusted fields never establish authority.
- Auth must not depend on Canic or Toko. Use published Platform/Types, Storage,
  Wire, Host, Build and Testkit at their established ownership boundaries. Do not
  implement cryptography or introduce a replacement runtime executor.
- Canic owns application endpoint wiring using IC Wire. Auth owns its contract
  schemas and decisions; do not add endpoint installation to `ic-auth-wire`.
- Build only required capabilities. No speculative wallet service, placeholder
  accepting endpoints, generic storage framework or legacy format readers.
- Keep product packages in a virtual root under crates/ and qualification-only
  packages in an independent qualification/ workspace. Do not create empty
  packages to imply implemented capabilities. Public packages never depend on
  qualification packages.
- Preserve provenance for retained code and fixtures. Tests must establish
  behavior independently of the replacement implementation.
- Refresh tooling only through the authenticated exporter. Preserve the
  installed snapshot and its immutable policy references; do not follow a moving
  sibling baseline. Do not add synthetic packages to hold the tooling dependency.
- Do not import historical release scripts, journals or build trees. Add only
  checks and commands that exist and have been exercised. Keep status short;
  owning GitHub issues are the tracker, not a parallel local backlog.
- The maintainer authorized crates.io publication of `ic-auth`,
  `ic-auth-protocol-types` and `ic-auth-wire`. Keep these product crates
  `publish = true` and qualification crates `publish = false`. Release validation,
  package provenance and registry verification remain required.
  Production deployment, consumer migration and deletion of the original
  repository or live storage remain outside the authorized scope.
