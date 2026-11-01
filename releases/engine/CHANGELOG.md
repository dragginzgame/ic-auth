# Engine changelog

Covers public `ic-auth` and the unpublished backend conformance, qualification and
native tooling packages. The independent series starts after the
[last coordinated release, 0.29.2](../history.md).

## [0.30.0] - 2026-11-01

### ⚠️ Breaking

- Align exposed Principal and codec identities with Protocol's Types 0.16 and
  Wire 0.14 generation. Consumers exchanging these Rust values must align their
  dependencies. Signed bytes, verification checks, stored formats and TEE/non-TEE
  trust policy are unchanged.

### ⬆️ Dependencies

- Adopt Platform 0.17.0, Wire/derive 0.14.2 and Testkit 0.78.0 from the registry.
  The qualification host selects Runtime/macros/guard 0.17.1 directly; native
  Testkit uses Platform's owned PocketIC client and the matching Wire limits.
- Align private admission metrics with Testkit's Metrics 0.25.0 selection,
  retaining one arithmetic provider and unchanged counters/Serde admission.
- Adopt Memory/Storage 0.62.1 in the private qualification host, aligning ledger
  codec errors with Wire 0.14 and removing the remaining Wire 0.13 providers.
  Preserve checked allocation handles, checkpoint bytes, bounds and fresh/restore
  refusals; no retained-state reset or migration is introduced.

### 🧪 Testing

- Isolate release fixture Make controls from an enclosing release's group, so
  missing-group admission and independent Engine selection remain exercised.
- Verify the three local library archives together in the read-only package gate,
  so pending Protocol identity changes do not accidentally compile Engine against
  the older registry Protocol. Selected-group publication keeps normal registry
  admission; cover package refusal and Make descriptor/unsafe-mode behavior.

### ♻️ Refactoring

- Select canonical Types directly for engine self-authentication; make Platform
  optional and enable it explicitly through crypto capabilities. Default and
  session-only graphs exclude the facade. See the
  [Protocol boundary change](../protocol/CHANGELOG.md).
- Release Engine independently of Protocol. Bind validation and publication to
  the selected group and saved policy, retaining the complete gate and uncertain
  upload recovery. Compiler/package and backend archive qualification admit
  independent Engine and Protocol versions.
- Enforce facade-free passive/default/session graphs and reject the independent
  Runtime owner in public and conformance graphs. Packaged-consumer compiler
  qualification witnesses the canonical Types identity directly.
