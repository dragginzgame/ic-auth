# Engine changelog

Covers `ic-auth`, maintained `ic-auth-tooling`, and associated private
qualification changes. Only the first two are Engine release-group members. The independent series starts after the
[last coordinated release, 0.29.2](../history.md).

## [0.40.0] - 2026-11-12

### ⚠️ Breaking

- Limit new portable credentials to 32 role–scope entries total across up to
  16 roles. Every valid combination fits within 3,332 canonical grant-list bytes;
  retain a defensive 4 KiB cap and unchanged signed encoding. Reject excess before signature
  verification or new issuer reservations; preserve completed-result recovery,
  protected enrollment bounds and existing stored session formats.

### 🧪 Qualification

- Add optional complete `ReceiverStore` conformance scenarios in private bundle
  0.36.2, using host checked backing, durable reopen and transaction rejection
  hooks without exporting or reconstructing the private login ledger
  ([#134](https://github.com/dragginzgame/ic-auth/issues/134)).

## [0.39.4] - 2026-11-12

### ⬆️ Dependencies

- Adopt published Toko Tooling 0.39.18 in both workspaces and refresh the
  recorded snapshot, preserving Auth's product qualification and frozen delivery.

### ⚡ Performance

- Use Storage 0.73.6's discard-value mutations for Auth's already-prepared
  record commits, preserving old-state admission, full Principal keys, synchronous
  host rollback and stored bytes. Follow-through for
  [#132](https://github.com/dragginzgame/ic-auth/issues/132).
- Omit the unchanged synthetic release-selector regression for reviewed
  product/data-only changes while retaining product, native archive, tooling
  and release/recovery obligations. Unknown or infrastructure changes still
  require the full roster; report the selection reason explicitly
  ([#133](https://github.com/dragginzgame/ic-auth/issues/133)).

## [0.39.3] - 2026-11-12

### ✨ Features

- Complete #132's missing renewal and inspection APIs: browser-authenticated
  lookup-only renewal recovery, original-request/policy preflight before host
  application authorization, and caller-ordered session inspection with an
  exclusive cursor and a scan budget across retained fences. Recovery preserves
  expired/interrupted originals without reserving work or advancing time.
  Signed claims, stored formats, verification and host trust boundaries are unchanged.

### 🧪 Testing

- Qualify durable issuer-to-receiver lifecycles with real certified and Ed25519
  proofs, device capacity/overflow, retries, renewal, logout, authority invalidation,
  same-release upgrade, interrupted callbacks and bounded cleanup. Extend the
  100,000-record probes to preflight, recovery and bounded inspection. Retain the
  1.3M ordinary-authentication instruction ceiling.
- Preserve incoming Storage 0.73.5 and qualification Metrics 0.29.12/Entropy
  0.1.10; keep focused evidence separate from the published 0.39.2 release gates.
- Adopt incoming Wire 0.21.5 in both locks and regenerate all five browser
  artifacts. Auth bindings are unchanged; Wire's runtime reduces varint and
  record-field allocation work while preserving byte and input-admission rules.

## [0.39.2] - 2026-11-12

### ✨ Features

- Implement optional Auth-owned persistent issuer and receiver stores for #132.
  Index selected logins/operations, reserve readable completion capacity, enforce
  account/device quotas and use reference-counted, bounded expiry cleanup. Keep
  caller sessions separate from maintenance indexes and retain one copy of each
  active binding. Host memory governance, clocks, transactions and certification
  publication remain explicit. Signed claims and verification policy are unchanged.
  The issue remains open for full issuer key/enrollment custody and qualification;
  these changes do not establish Canic adoption or complete #132 acceptance.

### 🧪 Testing

- Add native 100,000-login and installed issuer/ordinary-authentication scaling
  probes, with cold stable-memory access charged to authentication before any
  diagnostic read. Keep bookkeeping separate from initial proof cryptography.
  Accept the measured 1.223M ordinary-call cost provisionally with a 1.3M
  regression ceiling, including after upgrade, on published Storage 0.73.4.
  Further Storage optimization is not a prerequisite for this Auth batch.
  Optimize native fixture curve arithmetic without changing test assertions,
  overflow checks, record counts or the measured canister release profile.
- Update isolated all-feature package admission fixtures for optional Storage
  and its IC0 backing; retain default/passive exclusions and executor/guard
  refusal checks.
- Preserve incoming Platform 0.31.3, Runtime 0.21.3, Types 0.19.5, Host 0.76.5
  and Wire 0.21.4 in both locks. Regenerate all five browser files: the four Auth
  bindings are unchanged; Wire's runtime optimizes length and packed-boolean
  decoding. Retain client/interoperability qualification in the release gate.

- Adopt qualification Storage 0.73.4 and Testkit 0.101, plus current compatible
  Entropy, Metrics, Platform, Runtime and Wire selections. Keep one host executor/
  guard and the native PocketIC re-export. The private fixture uses fresh Storage
  backing; previous backing/cell images refuse without clearing retained bytes.
  Auth's signed claims, session records and checkpoint payload identity are unchanged.

### 🛠️ Tooling

- Adopt Toko Tooling 0.39.17 in both workspaces. Bound LOC table labels to the
  terminal width and provide `--wide` for full names; numeric counts and complete
  JSON labels are unchanged. Adopt explicit supplier-freshness review and
  disposition requirements for product, native-tool and qualification selections.
- Move modules with child directories to `mod.rs` under the new layout rule,
  including qualification tests. Preserve public module paths and remove an
  empty session-test directory, retaining the existing standalone test target.
  Adopt the upstream dependency-update companion fix in release fixtures.
- Declare Auth's dependency-update Make companion and use Shared's fixture
  copier for tool-evidence tests, so partial fixtures include every declared
  helper before exercising setup failures.
- Expose Shared's `deps-update` command across the product and qualification
  workspaces. It runs sequential Cargo updates with normal output, preserves
  manifest requirements and retains the Make execution guard. Updates run only
  when invoked explicitly; a later failure does not roll back an earlier lock.

## [0.39.1] - 2026-11-11

### ⬆️ Dependencies

- Select Platform 0.31.1 in the product lock without changing Auth's verification
  policy or enabling the new prepared-root API. The independent qualification
  lock retains Platform 0.31.0.

### 🧪 Testing

- Adopt Testkit 0.100.2 and import its native-only upstream PocketIC re-export,
  removing Auth's redundant direct client dependency. Keep upstream type identity,
  managed server custody and transport behavior unchanged.
  [#131 — Use Testkit's PocketIC re-export and remove direct client declarations](https://github.com/dragginzgame/ic-auth/issues/131).
  Reprepare the lock-selected CLI with `make install-testkit-tools`.

## [0.39.0] - 2026-11-11

### ⬆️ Dependencies

- Adopt Build 0.83, Host libraries/file CLI 0.76.4 and Platform 0.31 with
  Types 0.19.3 and Runtime/macros/guard 0.21.1. Keep the bounded external-lock
  admission and host-owned execution/verification boundaries.
  Platform's textual DID retirement affects no Auth endpoint or tooling caller;
  self-authenticating Principal bytes retain their existing SHA-224 framing.
- Select Wire 0.21.1 with the coordinated Protocol Rust identity cut. Qualify
  redundant transport lengths without changing opaque signed bytes or the
  separate canonical Auth claim encoding. Regenerate the browser runtime;
  all four explicitly named Auth bindings remain unchanged.
- Adopt Platform's BLS wrapper/error identity and retain checked certificate,
  delegation and signed-byte verification through its maintained boundary.
- Align native qualification on Testkit 0.100 and upstream PocketIC 16.1 through
  a non-Wasm development dependency. Copy canonical Principal bytes at its
  boundaries, retain managed server custody and await best-effort close. Upstream
  transport panics propagate; the retired client deadline/error contract is removed.
- Adopt Toko Tooling 0.39.10's authenticated snapshot, native qualification
  dependency admission and scoped Platform Digest adapter admission while
  retaining Auth's Protocol/Engine release gates.

## [0.38.0] - 2026-11-10

### 🛠️ Tooling

- Adopt published Toko Tooling 0.39.7 in both workspaces; keep shared references
  upstream and retain explicit shared release opt-in and Auth's receipt helper.
- Group Auth documentation into protocol, development, integration and performance
  topics with a documentation index and updated links.
  Preserve the Protocol/Engine release plans and unrelated dependency selections.

The bounded multi-device contract replaces the earlier one-device proposal.
Coordinate Protocol/issuer adoption; see the
[account/device audit](../../docs/design/account-device-capacity-audit.md).

### ⚠️ Breaking

- Charge replay capacity to `(issuer, subject)` across browser keys. Replay records
  retain authenticated account ownership after session removal; restore rebuilds
  aggregate counts and rejects mismatched session/replay owners. Rename the limit
  and occupancy projection to `max_replays_per_account` and `replays_for_account`;
  `SessionReader::occupancy` now takes the issuer-qualified account.
- Retire Auth 0.37 session, replay and login record identities together, and
  reject the old private qualification checkpoint envelope even when empty.
  No fallback reader or automatic reset is provided. This approved format cut
  accompanies the multi-device contract.
- Support eight signed device slots per issuer-qualified account, with an optional
  tighter host limit. Verified higher generations atomically replace a slot using
  a fresh browser key; renewal preserves its binding/window. Logout and expiry
  retain slot fences. Time-based pruning never removes known fences.
- Expose `LoginRevocationChange::displaced_caller()` for atomic removal of old
  session/login rows. Preserve replay history and other devices. Restore rejects
  duplicate slots and incompatible host limits; old keys cannot resurrect them.

## [0.37.0] - 2026-11-09

### ⚠️ Breaking

- Adopt Platform 0.29, canonical Types 0.19 and Wire 0.19 with the matching Protocol
  identity cut. Preserve signed bytes, strict verification and stored formats.
  Native Agent calls explicitly copy validated Principal bytes into inferred
  supplier identities after upstream conversion impls were retired.

### ⬆️ Dependencies

- Update direct Platform selections to 0.29.1 in both workspaces, retaining
  the compatible 0.29 minimum. Types 0.19 and
  Runtime 0.21 stay unchanged. New upstream Candid decoder APIs are unselected by
  the public libraries and Wasm host; native Testkit explicitly selects that
  codec. Crypto, signed bytes and owned Wire endpoints are unchanged.

- Adopt crates.io Toko Tooling 0.38.2, including its `toko-tooling-*` commands,
  `TOKO-TOOLING.md` baseline, `.toko-tooling.snapshot` receipt and renamed LOC
  metadata. Preserve Auth's strict Make execution admission with scoped guards;
  harmless help/catalog dry runs remain available. Dependency reports distinguish
  conditional, development/build and qualification declarations. Move six private test-support/host
  packages into an independent `qualification/` workspace with its own catalog
  and lock. Keep product packages in `crates/` and private versions independent.
  Adopt Shared's atomic two-lock group preparation with both-group rollback,
  input-sealing and product-resolution isolation checks.
- Update direct SHA-256 hashing to RustCrypto `sha2` 0.11.1, preserving canonical
  inputs, digest bytes and signature policy. Retain upstream-only older versions
  independently; native artifact diagnostics keep their existing hexadecimal output.
- Align the private host on Runtime/macros/guard 0.21 with Storage 0.71.1,
  resolving the previous duplicate Runtime graph. Adopt Testkit 0.97 and its
  required Host 0.76.2 cohort, clearing the prior Wire mismatch with one Wire
  0.19 / Platform 0.29 / Types 0.19 identity. Remove redundant private PocketIC
  Principal conversions; Agent/delegation boundaries retain explicit byte copies.
- Adopt Storage 0.71 for the private qualification host. Its bucket-backed
  allocator requires fresh whole backing; retired snapshot images are refused
  without clearing. Public Auth libraries, checkpoint payloads and signed bytes
  are unchanged; no production backing is reset or migrated.

### 🧪 Testing

- Define host nonce freshness, replay and restart requirements, and qualify
  published `ic-entropy` only in private backend tests. Cover real admission and
  replay, fail-closed unavailable/exhausted providers and seed reuse without
  changing verification, signed bytes or Canic's fresh challenge path.
  [#130](https://github.com/dragginzgame/ic-auth/issues/130)

### 🧹 Cleanup

- Streamline reference-store replacement, login-state checks and bounded pruning
  to avoid redundant map work and record/key copies. Preserve atomic callback
  ordering, expiry/replay retention, byte accounting and stored formats.
- Reuse signature preparation's seed lookup when selecting its existing or new
  message tree. Preserve capacity refusal, retention deadlines and certified
  roots without repeating that lookup.

## [0.36.1] - 2026-11-09

### ⬆️ Dependencies

- Qualify the incoming Metrics 0.29.8 selection for private admission diagnostics;
  retain fixed counter semantics and exclude Metrics from public library graphs.

### 🧹 Cleanup

- Validate unchanged metadata once before browser-journal callbacks, retaining
  credential-view and post-mutation corruption checks and the stored profile.
- Admit the Host file CLI once per bounded contract-generation operation while
  retaining both source hashes and effect-free refusal before compilation.

### 🛠️ Tooling

- Adopt Toko Tooling 0.36.1's corrected test-only LOC classification inside
  call-argument closures. Preserve the installed entrypoints, product release
  requirements and application dependency selections. Make the release fixture's
  pin-drift case independent of the current Tooling patch version.

## [0.36.0] - 2026-11-09

### ⚠️ Breaking

- Align the next combined delivery with Protocol's Wire 0.18 minor identity;
  consumers exchanging exposed Auth/Protocol values must select matching
  versions. Cryptographic checks, signed bytes and stored record profiles are
  unchanged.

### ⬆️ Dependencies

- Adopt Wire/derive 0.18 and matching Storage 0.70 for the private host. Keep one
  Wire identity in both native qualification and Wasm; Testkit 0.95 removes the
  temporary Wire 0.17 dev-dependency and all four budget projections.
- Preserve and qualify incoming Runtime/macros/guard 0.20.3; their upstream
  tooling adoption leaves executor behavior and generated code unchanged.

### 🛠️ Tooling

- Adopt Build 0.81 and Host FS/Process 0.76.1 for private tooling, including the
  prepared release editor and file CLI. Preserve application dependency selections,
  generated contracts and release requirements.
- Adopt Testkit 0.95 and its concrete managed instance, retaining the admitted
  PocketIC 16.1 pairing and one native Platform 0.27 / Host 0.76 identity.
  Public Auth APIs and protocol bytes are unchanged.
- Adopt Toko Tooling 0.36 through one exact Cargo pin and inherited dev-dependencies
  in every real package. Remove the metadata/dedicated-lock scheme, preserve real
  tools and qualify CI/publication refusal and Cargo input sealing. Document the
  required root exact pin in the dependency-pinning policy exceptions.
- Move all seven private tooling/qualification packages into `crates/` under their
  existing package names, preserving versions, features and publication roles.
  Update scripts, input seals, release ownership and active guides together;
  remove the future non-fleet `apps/` reservation
  ([#129](https://github.com/dragginzgame/ic-auth/issues/129)).

### 🧹 Cleanup

- Carry immutable grant-validation evidence into canonical encoding instead of a
  validation-bypass flag, preserving signed bytes and public refusal behavior.
- Separate browser journal metadata and credential transaction views, retaining
  strict atomic commits, the stored profile and unknown issuance without a
  recoverable sequence.

### 🧪 Testing

- Share deterministic certificate and browser signing-request construction while
  retaining independent vectors and scenario-specific rejection checks.
- Narrow permissive-feature qualification to proof verification and a direct
  session-admission rejection, leaving ledger/storage cases in their normal suite.
- Give release fixtures explicit scenario dispatch and a common tooling-admission
  substitute, retaining isolated recovery repositories and real tooling checks.

## [0.35.10] - 2026-11-08

### 🧹 Cleanup

- Narrow internal scope-subset checks to validated labels and fold the single-use
  request wrapper into its caller, preserving scope selection and refusal order.
- Compile the private browser-caller accessor only with sessions, removing an
  unused-method warning from standalone proof-verification builds.
- Select Shared Tooling 0.34.0 through root metadata and `toko-tooling.lock`,
  independently of application Cargo dependencies. Remove the former tooling
  dev-dependency, preserve Auth's real tools and release gates, and check the
  selection before direct publication effects. Update setup and jobserver
  fixtures for the new admission and cleanup prerequisites.
- Add compact sibling LOC progress and terminal colours for workspace, crate and
  module names, preserving plain redirected reports.
- Adopt Platform 0.27.0 without changing Auth APIs or cryptographic policy;
  Testkit's native helpers retain their compatible Platform 0.26.1 selection.
- Adopt Storage 0.69.0 in the private checkpoint host and use direct cell write
  results, preserving checkpoint formats, bounds and refusal behavior.

## [0.35.9] - 2026-11-08

### 🧹 Cleanup

- Keep signature-retention deadlines in the existing certified-tree leaves,
  removing the duplicate map while preserving certified roots, witness bytes,
  retry deadlines and bounded pruning. Share grant encoding without changing
  canonical bytes or admission checks.
- Capture browser destinations once before asynchronous work and pass immutable
  keys through private admission coordination.
- Reuse the parsed issuer key's original bytes and remove redundant private
  branches while retaining strict verification and live eligibility checks.
- Give the qualification receiver one owner for unconfigured, closed and
  configured presence, preserving checkpoint bytes and fresh-authority gating.
  Browser interoperability generation and checking share each vector declaration.
- Remove unused per-target admission deadlines from the private logout journal.
  Credential expiry and uncertain issuance recovery remain durable. The revised
  `ic-auth-login-cleanup-journal-v1` profile refuses the retired local-logout
  profile without clearing it; the maintainer confirmed no retained journals.
  This affects the unpublished client only; public Rust contracts are unchanged.

### 🔧 Changed

- Qualify the incoming locked Wire/derive 0.17.14 patch. Its generator preserves
  all five browser files byte-for-byte.
- Preserve incoming Build 0.79.1, Host 0.75.1 and Storage 0.68.11 for release
  qualification, retaining the existing native tooling and private storage APIs.

## [0.35.8] - 2026-11-08

### 🔧 Changed

- Adopt Shared Tooling 0.31.10 with workspace-member-only formatting, shared
  hash-dependency admission and crate/module LOC selection. Cache authenticated
  tool downloads before CI setup and save before tests; run one full-history
  Gitleaks scan outside the host matrix and release/publication paths.
  This is private tooling maintenance; public package versions remain unchanged.
- Adopt Platform 0.26 for the existing crypto capabilities, Wire/derive 0.17.13
  and Storage 0.68.10 for the private host. Preserve incoming Runtime, macros and
  guard 0.20.1. No Auth caller selects Platform's retired BLAKE3 facade or Wire's
  new opt-in packed ID encoding; Auth schemas, signed bytes and stored formats
  remain unchanged.
- Adopt Build 0.79 for native lock admission and the selected release editor.
  Auth does not use its retired batch configuration or outcome aliases.
- Qualify the incoming Testkit 0.93.1 native patch, retaining the unchanged
  PocketIC 16.1 pairing, explicit request budgets and managed async teardown.

## [0.35.7] - 2026-11-08

### 🔧 Changed

- Adopt Testkit 0.93 for native qualification, aligning its Platform dependency
  with Auth 0.25 and using its supplier-independent CBOR error contract.
  Explicit request budgets and managed server teardown remain unchanged.
- Qualify the incoming locked Platform 0.25.1 and Storage 0.68.8 patches with the
  existing Runtime 0.20 selection.

### 🧹 Cleanup

- Stream canonical delegation and portable-claims hashes through the existing
  SHA-256 implementation using the same field encoder as byte output. Preserve
  signed bytes, domains, structural refusals and every verification check while
  removing temporary hashing buffers.
- Use a fixed stack buffer for the certified-proof signature message, preserving
  its exact domain and digest framing without the temporary heap allocation.
- Distinguish pending issuance from completed browser evidence in the journal's
  checked types, preserving its stored profile and lookup-only recovery.
- Share private qualification configuration projections and login lifetime policy
  across fresh installation and restoration.

## [0.35.6] - 2026-11-07

### 🔧 Changed

- Adopt Platform 0.25 for Auth's crypto and native Agent adapters. Auth already
  imports ordinary entrypoints from Runtime; its public contracts and
  verification policy are unchanged.
- Align Runtime, macros and guard 0.20 with Storage 0.68.7 in the private host,
  preserving one executor/guard, canonical entrypoints and stored layouts.
- Adopt IC Build 0.78 and Host 0.75 for native tooling, including the explicitly
  selected Host file CLI. Auth does not use their retired compiler-wrapper,
  prepared-input, provenance or process-sampling APIs.
- Adopt Testkit 0.92 for native qualification's Host 0.75 error identities,
  preserving explicit request budgets and managed teardown.
- Regenerate the private browser runtime from Wire 0.17.12 while preserving
  Auth's four method bindings, transport bytes and validation budgets.

### 📚 Documentation

- Refresh the clean backend-bundle adoption instructions and Canic handoff,
  distinguishing qualified existing integration from its new adapter extraction.

## [0.35.5] - 2026-11-07

### 🩹 Fixed

- Adopt Shared Tooling 0.31.4, fixing checksum-helper parsing under macOS's
  Bash 3.2 before host-tool setup. Keep NUL-output rejection and backend failure
  propagation, and adopt its isolated grouped-release checks and LOC scratch
  handling after cleanup.
- Clear CI's Cargo target at the release-command fixture boundary, preserving
  the ordinary-shell delivery guard. Add focused execution of that actual caller.

### 🔧 Changed

- Adopt Platform 0.23.6 and its locked support dependencies. Preserve Auth's
  verification policy, signed bytes, owned protocol identities and runtime boundary.

## [0.35.4] - 2026-11-07

### 🔧 Changed

- Adopt IC Build 0.77.0 for native lock admission and the prepared release editor.
- Adopt Runtime 0.19.5 with its matching macros/guard and Storage 0.68.3 in the
  private qualification host, preserving the public Auth runtime boundary.

### 🧪 Testing

- Migrate the qualification runner to Testkit 0.90's explicit finite request
  budget, retaining the 30-second allowance, separate startup timeout and managed
  instance/server teardown ([#128](https://github.com/dragginzgame/ic-auth/issues/128)).

## [0.35.3] - 2026-11-07

### 🔧 Changed

- Adopt IC Build 0.76.0 for native lock admission and the explicitly prepared
  release editor. Public library dependencies and release custody are unchanged.
- Make the private browser journal return an explicit credential action and
  required signer, keeping issuance/recovery decisions with their durable owner.
  The stored journal format and lookup-only recovery behavior are unchanged.
- Regenerate the browser contracts from the incoming locked IC Wire 0.17.6;
  Auth method bindings are unchanged.

### 🧪 Testing

- Extend the unpublished backend conformance suite with 90-day login restoration,
  late renewal, issuer key/mechanism changes, withdrawn grants and changed
  authority/enrollment deadlines using the existing adapter hooks. Earlier source
  archives retain their original scenario set.
- Include portable claims hashing in the existing native allocation profiler.
  Production hashing, signed bytes and verification behavior are unchanged.
- Share allocation-profile capture and DHAT observation across both native
  benchmarks, preserving their commands, input custody and report schema.

### 📚 Documentation

- Refresh Canic's recorded integration progress and distinguish logout, complete
  recovery, missing state and snapshot rollback acceptance in the adoption guide.

## [0.35.2] - 2026-11-06

### 🔧 Changed

- Adopt Shared Tooling 0.31.0 with shared batch setup, saved hook context
  and lifecycle controls. Route releases through captured source, preserving
  later normal-checkout dependency edits. Prepare candidate tools and browser
  dependencies locally; recover using the saved editor selection without installs.
  Shared owns automatic single-group capture and refuses an empty roster for a
  selected group. Keep publication explicit and retain the existing gates and
  receipts.
- Migrate batch settings to Shared's `RELEASE_GROUPS`, `RELEASE_PUBLISH_TARGET`,
  `RELEASE_SETUP_TARGET`/`RELEASE_SETUP_MODE` and `RELEASE_BATCH_ACTIVE` contract.
  Retired settings have no aliases; existing batch storage paths are unchanged.
  Adopt the checksum-output admission fixes from
  [Shared Tooling PR #361](https://github.com/dragginzgame/shared-tooling/pull/361).

## [0.35.1] - 2026-11-06

### 🔧 Changed

- Adopt Shared Tooling 0.30.8 and its release-efficiency rules, including the
  frozen-batch recovery fix for generated IDs containing underscores. Frozen release
  batches remain opt-in and unqualified for Auth; existing delivery is unchanged.

### 🧪 Testing

- Extend the unpublished backend conformance suite with unknown-login logout
  failure, work-credit, retry and restoration checks. Known logout remains
  independent of unknown work credit; no driver hooks or runtime API changes.
- Refresh the source-bundle handoff and registry-consumer qualification for the
  published Engine/Protocol 0.35.0 cohort.

## [0.35.0] - 2026-11-06

### ⚠️ Breaking

- Support protected login windows up to 90 days while retaining short proofs and
  session leases. Login-ledger construction now requires an explicit maximum
  login TTL, bounding reservations, restoration and conservative logout retention;
  shorter host policies retain their shorter storage horizon.
- Replace the private browser journal profile with durable issuance/renewal
  sequences and exact result recovery. The maintainer confirmed no retained old
  records; the retired profile is refused without migration or automatic reset.
  Issuer adapters must supply browser-bound renewal and sequence-specific lookup;
  receiver adapters return the authenticated admission receipt.
- Remove the unqualified skew assumption from local revocation: retain known
  denial through the signed login deadline, report unknown denial as unconfirmed,
  and keep lost-state receivers closed pending trusted recovery. Retire the
  timestamp reset helper, login-record identity and private fixture checkpoint.
  The maintainer confirmed no retained affected Rust/browser records. Browser
  cleanup remains finite without treating local expiry as remote confirmation.
  Canic owns production adoption of the
  [#127 contract](https://github.com/dragginzgame/ic-auth/issues/127).

### ➕ Added

- Reuse retained proof evidence after browser reopen and renew completed expired
  proofs using the browser key without the account provider. Persist intent before
  dispatch and evidence before admission; unresolved sequences remain lookup-only.
  Logout fences late results and deletes evidence atomically. Ordinary signing
  reads only small metadata. Detached receipt metadata supports host scheduling;
  key custody, issuer endpoints and SDK identity wiring remain host-owned.
- Add receiver-bound readiness identities that reuse live lease hints and renew
  before signing after suspension. Support short-lease refresh before proof
  expiry and one admission retry after an authenticated expiry refusal on reopen;
  pending issuance stays lookup-only. Polling submitted requests never renews and
  application effects are never replayed. Hosts supply the clock mapping and SDK
  wiring. Expired exact recovery can refresh only after durable completion of
  the uncertain sequence. Qualify late renewal against the original 90-day deadline.

### 🔧 Dependencies

- Adopt IC Build 0.75.0, Storage 0.68.0, Testkit 0.89.0 and Wire 0.17.4 in tooling and
  qualification. Storage's collection images require fresh fixture allocation;
  retained old framing is refused without clearing. Collection checksums are
  retired upstream; Auth retains its payload identity and bounded Wire admission.
  No production Auth storage migration is performed.
- Testkit client logging is application-owned. Auth uses no retired environment
  logging setup; keep its explicit server-output custody without a replacement
  global subscriber. Wire adoption alone leaves the browser runtime unchanged;
  the separate logout hard cut updates its generated binding.
- Adopt Shared Tooling 0.30.6 and enable its CBOR source check alongside the
  existing Cargo and browser dependency/import enforcement. Retain an empty
  source allowance policy and the Protocol/Engine validation groups.

### 🧹 Cleanup

- Remove the private qualification fixture's unused direct Serde dependency;
  retain the separate feature-unification fixture's deliberate dependencies.

### 🧪 Testing

- Add clock-independent deadline/denial, unknown-result, durable recovery and
  lost-state refusal acceptance. Preserve strict future-proof rejection, bounded
  capacity/work and local ordinary authorization. The private checkpoint records
  never-configured, closed or retained receivers explicitly: first installation
  after an unrelated upgrade works, while closed state survives later upgrades.

## [0.34.1] - 2026-11-05

### 🔧 Changed

- Delegate bounded certificate decoding and standard canister-signature CBOR
  encoding to Platform, including the private qualification profiler's decoding.
  Preserve Auth's host-selected budgets, store policy, witness construction,
  certified-root binding, error variants and exact envelope bytes. Require
  published Platform 0.23.0 with its `certification-cbor` capability and remove
  Auth's direct CBOR dependency.
- Adopt published browser SDK 0.2.0 and use its typed ingress inspection and
  read-state response fixtures in transport tests. Retain signed-request and
  invalid-certificate checks, with SDK Core still at 6.1.1.

### ⬆️ Dependencies

- Adopt Testkit 0.87 for native qualification and share Platform 0.23 throughout
  the workspace. Migrate live fixtures to its async client with one caller-owned
  runtime per test and explicit, checked instance deletion before server shutdown.
  Preserve signed-ingress, Wire, lifecycle and admission assertions.
- Select Metrics 0.29.3 for private qualification; its production source and
  Auth's metrics contract are unchanged.

### 🔧 Tooling

- Adopt committed Shared Tooling 0.30.4, including LOC/formatting scratch fixes
  and numbered release-draft handling. Enforce its Candid source check across
  Auth libraries and private apps through the existing boundary gate, with no
  application-type allowances. Repair formatting-hook and saved-source recovery
  refusal fixtures after removal of the direct CBOR dependency.

### 📚 Documentation

- Replace superseded handoff/adoption journals with current contract guidance
  and Git history pointers. Compact the historical Wasm study while retaining
  its results and limits; preserve release receipts and recovery records.

## [0.34.0] - 2026-11-05

### ⚠️ Breaking

- Align the pending Engine release with Protocol 0.34's Wire 0.17 identity.
  Public Protocol types exchanged with Engine must use the matching cohort;
  signed material, session records and verification behavior remain unchanged.

### 📚 Documentation

- Document certified issuance composition with a durable consumer journal,
  including update/query separation, exact-result recovery and volatile leaf
  loss on upgrade. Refresh the Canic adoption review with recorded local
  acceptance and remaining production requirements; no runtime behavior changes.

### 🔧 Tooling

- Adopt committed Shared Tooling 0.30.0 rules and its schema-2 Rust LOC reporter.
  Declare private qualification/tooling roles and move allocation observations
  to harness-free `benches/` targets, preserving their script entrypoints.
  The corrected LOC categories are accounting changes, not removed code.
  Include the follow-up display fix: single-workspace tables use the directory
  name while JSON retains the full manifest path, omit the duplicate TOTAL row,
  and use portable exported documentation links.
- Require `test-protocol` in Engine-only release plans: it runs Engine default
  tests, despite its name. Cover failure propagation in the actual selector fixture.

### ⬆️ Dependencies

- Adopt Storage 0.67 for the private checkpoint host, including its internal
  Wire 0.17 codec. Preserve checkpoint bytes and backing layouts; no storage
  reset is required.
- Adopt Platform 0.22.1's upstream certificate and BLS implementations, restored
  packed-map/sequence envelope admission and reduced redundant BLS key parsing. Preserve
  Auth's public APIs, signature bytes, verification checks and host-owned trust
  inputs. Certificate-enabled graphs now permit the upstream helpers' transitive
  Candid/raw IC0 dependencies; default/session, wallet-signature and passive Wire
  graphs retain their exclusions. Runtime/executor ownership remains with hosts.
  Upstream packaging increases the reference canister's Wasm footprint.
- Adopt Testkit 0.86.0 with matching Wire 0.17 limits and its explicit-server
  client contract. Keep Auth's managed
  startup, retained server custody, offline CLI admission and PocketIC 16.1.0
  pairing; the existing caller already supplies the endpoint through Testkit.

## [0.33.2] - 2026-11-04

### 🩹 Fixed

- Enforce the signature-output byte limit during CBOR serialization, preserving
  envelope bytes and exact-boundary admission while stopping oversized writes.
- Bound retained Git-side release/publication state using the shared guard,
  including direct adapter calls. Keep new source extracts and gate logs under
  `target/`; preserve old recovery evidence and original command failures.

### 🧪 Testing

- Extend the reusable backend bundle with cleanup rollback/bounds and retained
  clock/generation scenarios. Consumer test drivers must implement the new
  maintenance/clock hooks and handle fallible generation updates.
- Add an opt-in native exact-retry allocation profile, with setup and signing
  excluded. Keep the existing retry behavior while measuring its claims decoding.
- Move engine and qualification unit tests and their private helpers into
  adjacent `tests.rs` modules, using ordinary Rust module discovery. Preserve
  assertions, private access and fixture bytes; retain executable API docs
  beside their declarations.

### ⬆️ Dependencies

- Adopt Metrics 0.29.1 for qualification admission instrumentation. Keep recording
  separate from report serialization, which belongs to Canic's metrics adapter;
  retain coverage that counter overflow cannot alter admission or exact retries.
- Select Storage 0.66.2 for the private qualification host, retaining its public
  APIs and stored format while removing an intermediate map-value decode copy.
- Select Host 0.74.3 and Runtime/macros/guard 0.19.1, retaining the published
  runtime behavior while adopting their test-source layout cleanup.
- Select Build 0.74.3, including its consolidated artifact-cache finalization
  and unchanged release-editor contract.
- Adopt Shared Tooling 0.29.4 at `628ccc7`, including retained-state admission,
  test-source layout and application-instrumentation ownership guidance.

### 🥾 Governance

- Retire the redundant capacity and 0.33 release checkouts after verifying that
  their work is integrated. Keep required compact evidence in the owning target
  directory and history in ordinary Git refs; keep checkout/build archives out
  of `.git`. Use the normal checkout for continued work.

## [0.33.1] - 2026-11-03

### 🔧 Changed

- Accept positive host-selected session/replay/count and encoded-byte budgets
  instead of fixed deployment ceilings, enabling Canic's 100,000-session profile.
  Keep per-write capacity/overflow checks, restoration refusal, record codec
  bounds and proof verification unchanged. Hosts still qualify backing,
  retention, cleanup and instruction costs for their selected capacity.

### 🧪 Testing

- Add maximum-shape backend conformance through admission, reopen and scoped
  authorization. Add opt-in installed-client archive qualification in Chromium,
  retaining real IndexedDB reopen/cleanup and executable/bundle hash evidence.
  Public verification and transport semantics are unchanged.

- Qualify matching 0.33 backend and private browser archive candidates, retaining
  their receipts and integrity in one local handoff. Document installation into
  consumer test support and exact browser file dependencies; require actual
  store and production transport qualification in Canic.

### ⬆️ Dependencies

- Adopt Platform 0.21.0 after moving simulator clients to Testkit. Preserve
  canonical Types 0.18, Runtime 0.19, crypto checks and the existing Protocol
  identity; this update retires the unused Platform client feature.

- Adopt Shared Tooling 0.29.2 at `04a87b3`, using pinned-commit export from the
  existing local supplier repository. Preserve normal-checkout work, native
  Testkit ownership and manifest-order refusal behavior.

- Adopt Testkit 0.84.1 and import the maintained simulator client/DTOs from
  `ic_testkit::pocket_ic`. Remove `ic-platform-pocket-ic`; keep the PocketIC
  16.1.0 server, Types 0.18 and Runtime 0.19 pairing. Its Platform 0.21 alignment
  removes the older native facade. Preserve the incoming
  locked Build 0.74.2 selection.

### 🥾 Governance

- Make routine implementation and adoption changes in the normal checkout;
  use topic branches and PRs when requested or required. Remove the additional
  release-clone requirement, coordinate active sessions before branch switches
  and preserve original recovery evidence.

## [0.33.0] - 2026-11-03

### ⚠️ Breaking

- Release validation uses source-bound Protocol/Engine plans and schema-4 receipts
  binding Shared's admitted run evidence. Resume older schema-2/3 attempts in their original
  release checkout. Session semantics, signed material and stored records are unchanged.
- Align public Principal and certificate APIs with Protocol's canonical Types 0.18
  and Platform 0.20. Keep existing verification checks and the TEE/non-TEE trust model.

### 🔧 Changed

- Keep repository CI complete while source-only group releases omit independently
  unchanged obligations. Shared, dependency, manifest, generator and uncertain
  changes retain the full roster. Opt-in native workflow qualification compares
  both mappings with its full gate on the same source and retains successful logs
  and receipts; ordinary CI runs one complete gate.
- Use Shared's evidence verifier, baseline observation and admitted Cargo group
  plans, deleting Auth's generic run parser and separate manifest discovery.
  Preserve original-source recovery, retained proof bytes and fresh-clone
  publication. Add a focused validation-receipt fixture selection
  ([#125](https://github.com/dragginzgame/ic-auth/issues/125)).

### ⬆️ Dependencies

- Adopt Shared Tooling 0.28.3 at committed revision `07f41c2`, preserving Auth's
  release groups and complete validation roster. Select Platform/PocketIC
  0.20, independent Runtime 0.19, Types 0.18, Testkit 0.83, Wire/derive 0.16,
  Metrics 0.28.2 and Storage 0.66.1. Select the file CLI at 0.74.2. Native fixtures
  handle the fallible PocketIC API explicitly without retrying uncertain effects.
- Delegate full-stage roster admission to Shared's canonical helper. Auth retains
  product mapping and source/recovery custody. Native qualification accepts exact case filters.

### 🧪 Testing

- Extend the reusable backend contract with scoped authorization, caller/issuer
  isolation and exact-expiry checks before and after reopening storage. Verify
  denied operations never invoke application callbacks or mutate stored records.

### 📚 Documentation

- Align the Canic and backend adoption guides with published Protocol/Engine
  0.32 and checked labels. Document matched backend/browser archive qualification
  and integrity-based installation without implying production integration.

## [0.32.0] - 2026-11-02

### ⚠️ Breaking

- Align grant APIs with Protocol's checked scope labels and 64-byte roles.
  `canonical::validate_scopes` accepts `ScopeLabel` slices and checks strict
  ordering; individual label admission belongs to the passive types. Malformed
  portable scope text is refused as `InvalidShape` while decoding. Signed bytes,
  verification algorithms, authority checks and valid stored formats are unchanged.

### ⬆️ Dependencies

- Adopt the incoming lock selections for Runtime 0.18.1, Testkit 0.80.1,
  Wire/derive 0.15.1, Storage 0.65.2, Metrics 0.27.2, Build 0.74.1 and Host
  FS/Process/Artifacts 0.74.2. Keep the independent file CLI pin at 0.74.1.
  Advance ctutils to 0.4.8.
  Regenerated browser contracts are unchanged.

### 🧹 Cleanup

- Remove unused direct Metrics and Serde-bytes dependencies from the private
  qualification canister. Keep metrics with the shared fixtures and certificate
  envelope bytes with Wire; tighten direct Metrics ownership accordingly.
- Share private canister codec bindings and use each method's canonical request
  ceiling before copying input. Consolidate login record serialization without
  exposing unchecked deserialization or changing its stored bytes.
- Consume checkpoint buffers during restoration and profile a borrowed session.
  Move owned role strings through fixture projections and declare the private
  enrollment field mapping once. Share primitive browser u64 admission while
  preserving each owner's clock, expiry and positivity rules.

### 🧪 Testing

- Derive the CI manifest seal from Cargo's admitted workspace inventory, including
  newly added members. Refuse incomplete discovery and changed preparation inputs;
  qualify paths with spaces and nested release logging, and preserve the complete
  CI/release stage roster. Drift fixtures inspect retained checkpoint evidence.
  Tool-admission fixtures distinguish read-only inventory from compiler/server
  dispatch while retaining their fail-fast ordering checks.
- Drive the browser admission timeout test with a controlled clock after transport
  entry. Preserve the timeout, cancellation and uncertain-issuance assertions
  without racing a 5 ms wall-clock deadline against IndexedDB scheduling on macOS.

## [0.31.0] - 2026-11-02

### ⚠️ Breaking

- Require Rust 1.99.1 for all packages, matching the pinned development/release
  compiler ([#124](https://github.com/dragginzgame/ic-auth/issues/124)).
- Align public Principal and crypto/certificate APIs with Protocol's canonical
  Types 0.17 and Platform 0.18. Consumers must select matching Rust identities;
  cryptographic checks, signed bytes and the trust model are unchanged.
- Carry shared certificate lifetime and grant errors directly in issuance errors;
  callers match `Lifetime` and `Grants` rather than duplicate flattened variants.
- Private qualification controls use typed refusal codes instead of text. All
  native callers and codec hooks change together; public Auth Wire and retained
  checkpoint/record formats are unchanged.
- Adopt Storage 0.65's consolidated allocator and checksummed collections in the
  private fixture. Fresh backing is required; previous manager/ledger images are
  refused without reset. Auth's payload/record bytes and crypto remain unchanged.

### ⬆️ Dependencies

- Replace Memory/Storage 0.63 with published `ic-storage` 0.65.0, removing both
  `ic-memory` and `ic-stable-structures` from the lock. Use checked Storage leases,
  explicit Cell load ceilings and typed write errors; retain one maintained
  Runtime/generator/guard through its direct Wasm Runtime dependency.
- Select Metrics 0.27.0 in private qualification, preserving admission counters.
  Testkit/Host retain independent arithmetic-only Metrics 0.26.1 internally; no
  counter type crosses that boundary and native report metadata stays disabled.
- Adopt Build 0.74.0 and Host FS/Process/Artifacts 0.74.1, including the explicitly
  prepared file CLI and release editor. Adopt Testkit 0.80.0 and the canonical
  independent PocketIC 0.18 client with the unchanged 16.1 server pairing. Remove
  the old Host 0.73, Metrics 0.25 and management DTO 0.8 providers.
- Adopt Shared Tooling 0.27.28 at `8c7fae6605c69a930e3b1f14e1ff5c4c9dafbcb7`.
  Retain Auth's complete CI/release roster; optional release-group plans are not
  activated. Existing tool retention and release custody remain unchanged.
- Align public Auth, Storage and Testkit with Platform 0.18, Types 0.17,
  Runtime 0.18 and Wire/derive 0.15. The new Storage and Wire releases resolve
  the prior runtime-guard and Principal-schema blockers. Preserve one host
  Runtime/generator/guard, all public capability boundaries, schemas and bytes.

### 🧹 Cleanup

- Construct fresh sessions from the engine's checked, immutable admission snapshot;
  retain complete validation for exact retries, later reads and storage commits.
- Decode checkpoints from borrowed stable payloads, and use maintained digest
  operations for digest-only certified roots. Keep witness construction separate.
- Select logout dispatch/wait/completion in one strict journal transaction while
  preserving durable leases, post-signing checks and the existing stored profile.
- Share valid native signing/context setup, count decoded replay records without
  a temporary vector, and reuse the owned Wire logout refusal conversion.
- Keep Auth-specific group/recovery/publication fixtures while leaving generic
  version arithmetic and repeated delivery permutations with Shared Tooling.
- Use one compiler setup/cache and the isolated archive feature matrix; retain
  native and Wasm Clippy, proc-macro and real runtime qualification instead of
  repeating those compilations under a second compiler.

### 🧪 Testing

- Qualify the consolidated Storage host in the boundary caller fixture, including
  missing/duplicate Storage, retired Memory/stable-structures and native-client
  refusal alongside the existing Runtime/generator/guard checks.

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
