# IC Auth New

Read the installed [Toko Tooling baseline](TOKO-TOOLING.md). Its common rules
apply alongside the product constraints below. Tooling 0.44.4 is authenticated
by `.toko-tooling.snapshot`; [developer tooling](docs/development/tooling.md)
describes Cargo-based admission and focused checks.

Read README.md, docs/design/DESIGN.md, docs/design/authentication.md,
 docs/design/compatibility.md and
 docs/status/current.md before implementation.

- This repository is an explicitly authorized clean restart. The sibling
  `../ic-auth` is read-only reference, not an implementation destination.
- Preserve existing Wire method identities, all four codec allowances, layouts,
  generated browser interoperability and canonical signed bytes.
- Support volatile and stable authentication storage. Share semantic rules;
  retain backend-specific corruption admission and host transaction boundaries.
- Treat published `ic-storage` as interim. The intended durable destination is
  `ic-storage-new` with orthogonal persistence. Keep interim storage mechanics
  backend-local; qualify the successor's lifecycle and rollback before adoption.
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
- GitHub source milestones for this repository are authorized separately from
  production delivery. Keep every crate `publish = false`; do not publish to
  crates.io until the replacement is complete and publication is authorized.
  Deployment, consumer migration and deletion of the original repository or
  live storage remain outside the authorized scope.
