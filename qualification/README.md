# Independent qualification

Separate conformance, benchmark, fixture and test-support packages belong here,
with directories matching their Cargo package names. They consume product
packages; product packages must never depend on them.

There are no qualification packages yet. Add `qualification/Cargo.toml` as an
independent virtual workspace and generate its own `Cargo.lock` with the first
real qualification package. Do not add an empty package or workspace to imply
acceptance evidence. Keep ordinary product tests with their owning crate.

The shared `make deps-list`, `make deps-update` and `toko-cloc` commands discover
this workspace automatically once its manifest exists. See
[tooling setup](../docs/development/tooling.md) for compiler and tooling selection.
