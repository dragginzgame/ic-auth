# Independent qualification

Separate conformance, benchmark, fixture and test-support packages belong here,
with directories matching their Cargo package names. They consume product
packages; product packages must never depend on them.

`ic-auth-wire-conformance` compares the reference's published `ic-wire` 0.22.2
with the product's 0.24.1 using fixed browser frames and the replacement schemas.
It owns its own dependency catalog and lock. Its private version is independent
of product releases. The reference codec is never a product dependency.

Run `make qualify-wire` and `make lint-wire-qualification` from the repository
root. For browser generation and comparison, use `make qualify-wire-browser`;
generated artifacts live under `qualification/target/wire-browser`.

`ic-auth-ed25519-conformance` deliberately activates Dalek's legacy scalar
compatibility as a dev-only negative control. It proves that upstream accepts
the fixture in that graph while Auth retains canonical-scalar refusal and
accepts the valid signature. Use `make qualify-ed25519` and
`make lint-ed25519-qualification`. Product verification remains owned by Platform;
no product package depends on this feature activator.

The shared `make deps-list`, `make deps-update` and `toko-cloc` commands discover
this workspace automatically. See
[tooling setup](../docs/development/tooling.md) for compiler and tooling selection.
