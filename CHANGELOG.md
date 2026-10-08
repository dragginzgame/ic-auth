# Changelog

## [0.1.8] - 2026-10-08

- Declare and check Rust 1.88 as the supported package minimum, independently of
  the development compiler. Verify isolated public package payloads on native
  and Wasm, with supported features and explicit minimum-compiler setup/CI.
- Adopt Shared Tooling 0.1.28's installer-link admission and simulation-only
  release-runner fixtures, plus its readable maintenance catalog.
  [shared #75](https://github.com/dragginzgame/shared-tooling/issues/75),
  [shared #70](https://github.com/dragginzgame/shared-tooling/issues/70).

## [0.1.7] - 2026-10-08

- Add internal `ic-testkit` qualification for composed certification, protected
  metadata upgrades and real signed ingress. Fresh session keys retain the same
  fixture principal; invalid delegations and non-owner operations are rejected.
  The fixture does not implement wallet login or a stable session backend.
- Adopt current Shared Tooling for physical script paths and complete failure
  evidence, including Rust tool builds; keep release fixtures aligned with the
  actual snapshot and complete workspace package set.
  [shared #67](https://github.com/dragginzgame/shared-tooling/issues/67),
  [shared #68](https://github.com/dragginzgame/shared-tooling/issues/68).

## [0.1.6] - 2026-10-08

- Add optional bounded canister-signature preparation and retrieval with exact
  retry deadlines, indexed expiry cleanup and explicit host-owned certification
  composition. Keep signing decisions, runtime clock/certificates and lifecycle
  with the host; preserve existing Canic signature bytes.

## [0.1.5] - 2026-10-08

- Add optional local session admission with atomic replay consumption, exact
  retries without extending authority, live caller/scope/authority checks and
  replay-preserving logout. Provide bounded volatile storage and explicit host
  transaction contracts; durable canister adoption remains separate.
- Adopt Shared Tooling 0.1.26, preserving concurrently changed symbolic Git
  tracking refs during confirmed release reconciliation.
- Use published `ic-host-fs` and `ic-host-artifacts` 0.7 for native file and
  publication evidence operations; authentication library graphs remain separate.

## [0.1.4] - 2026-10-08

- Add optional complete application-token verification: authenticate root
  chain-key grants and issuer signatures, bind the actual caller, and enforce
  protected audience, role, scope, lifetime and authority policy on every call.
  Preserve existing signed bytes and keep verification free of wallet, host
  tooling and storage dependencies. Session admission/replay remain separate.
- Isolate parallel native-tooling test fixtures even when macOS clock readings
  coincide, preventing one test's input from replacing another's evidence.

## [0.1.3] - 2026-10-08

- Add optional IC canister-signature verification with protected signer, seed
  and network trust inputs, bounded proof decoding and host-controlled certificate
  freshness. Use DFINITY's cryptography; preserve Canic's signed domains.
  Complete token verification and session admission remain pending.
- Adopt Shared Tooling 0.1.25, including final release/tag rechecks, hardened
  evidence archive paths and
  confirmed remote-tracking reconciliation. Use published `ic-host-fs` and
  `ic-host-artifacts` for bounded file identities and durable publication
  evidence. Encoding-only dependency graphs and signed-byte contracts are unchanged.
  [#1](https://github.com/dragginzgame/ic-auth/issues/1)
- Configure the complete CI gate on Linux and macOS Intel/Apple Silicon,
  with retained failure evidence. Native macOS qualification awaits those jobs.

## [0.1.2] - 2026-10-08

### Breaking

- Rename the types package to `ic-auth-protocol-types` to avoid the existing,
  unrelated `ic_auth_types` crate on crates.io. Consumers must update their Cargo
  dependency and Rust imports to `ic_auth_protocol_types`; Candid contracts and
  signed bytes are unchanged. Update release and publication tooling to use
  the renamed package. [#1](https://github.com/dragginzgame/ic-auth/issues/1)

The maintainer selected `0.1.2` for this rename as an exception to the usual
pre-1.0 minor increment for breaking changes.

## [0.1.1] - 2026-10-08

- Add standard patch, minor and major releases with recoverable metadata
  preparation and atomic branch/tag delivery, plus crates.io publication and
  a publication dry run that verifies package builds without uploading.
  [#1](https://github.com/dragginzgame/ic-auth/issues/1)

## [0.1.0]

Initial bootstrap recorded in commit
[`7a03102`](https://github.com/dragginzgame/ic-auth/commit/7a03102e52b7a400588530997a9f7fb91c83de9d)
on 2026-10-08; this version was not tagged or published.

- Bootstrap the unpublished Rust workspace with shared setup, formatting hooks,
  locked validation and CI configuration.
- Introduce passive application-token contracts and canonical encoding adapted
  from Canic, preserving existing Candid identities and signed hash vectors.
  Full verification, sessions and wallet login remain pending under
  [Canic #491](https://github.com/dragginzgame/canic/issues/491).
