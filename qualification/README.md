# Independent qualification

Separate conformance, benchmark, fixture and test-support packages belong here,
with directories matching their Cargo package names. They consume product
packages; product packages must never depend on them.

`ic-auth-wire-conformance` uses published `ic-wire` 0.24.1 against 37 fixed
browser frames and 296 resource thresholds retained from reference 0.22.2.
It owns its own dependency catalog and lock. Its private version is independent
of product releases. The old codec is no longer a dependency in either workspace.

Run `make qualify-wire` and `make lint-wire-qualification` from the repository
root. For browser generation and comparison, use `make qualify-wire-browser`;
generated artifacts live under `qualification/target/wire-browser`.

`ic-auth-ed25519-conformance` deliberately activates Dalek's legacy scalar
compatibility as a dev-only negative control. It proves that upstream accepts
the fixture in that graph while Auth retains canonical-scalar refusal and
accepts the valid signature. Use `make qualify-ed25519` and
`make lint-ed25519-qualification`. Product verification remains owned by Platform;
no product package depends on this feature activator.

`ic-auth-certified-conformance` builds real BLS certificate fixtures independently
of Auth to check issuer-signature binding, trust chains and freshness. Use
`make qualify-certified-issuer` and `make lint-certified-qualification`.
It also checks independent OpenSSL-signed root grants with
`make qualify-root-delegation`. `make qualify-certified` composes both with claims
policy and work charging; `make qualify-shared-verification` checks real proofs
sharing a budget across mechanisms. These checks establish no session or storage.
`make qualify-wire-admission` checks consuming Wire/domain projection into both
verifiers, unsigned request policy and failure charging with native host drivers.

`ic-auth-host-conformance` installs `ic-auth-host-fixture` as actual Wasm to check
issuer/receiver transaction rollback, certificates, terminal upgrade-loss refusal
and installation-bound enrollment calls with atomic key/metadata publication. See
[setup and evidence](ic-auth-host-conformance/README.md); run `make qualify-host`
with a prepared PocketIC 16.x server, and `make lint-host`.

The shared `make deps-list`, `make deps-update` and `toko-cloc` commands discover
this workspace automatically. See
[tooling setup](../docs/development/tooling.md) for compiler and tooling selection.
