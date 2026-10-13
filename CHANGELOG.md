# Changelog

## [0.5.10] - 2026-10-13

- Use IC Metrics for repeated qualification instruction summaries, with separate
  workloads and decode/authorization/total counts, means and maxima. Retain raw
  samples and enforce each observation's budget.
  [#49](https://github.com/dragginzgame/ic-auth/issues/49).

- Adopt Shared Tooling 0.10.0 with snapshot-owned common tool catalogs and
  independent host, IC and Rust selections. Reuse the existing reviewed pins;
  run `make install-tools` and `make tools-check` to prepare and admit them.
  [#48](https://github.com/dragginzgame/ic-auth/issues/48).

## [0.5.9] - 2026-10-13

- Avoid rewriting unchanged signature retention entries on live preparation
  retries, preserving validation, the accepted clock and the original deadline.
  [#47](https://github.com/dragginzgame/ic-auth/issues/47).

- Share canonical scope validation across token verification and live session
  checks, preserving their existing errors, rejection order and retry behavior.
  [#46](https://github.com/dragginzgame/ic-auth/issues/46).

- Remove redundant session-request byte accounting: the existing scope count
  and canonical label limits enforce the same 1,024-byte ceiling.

## [0.5.8] - 2026-10-13

- Qualify native tooling with Host 0.16.0 and Testkit 0.35.1, retaining
  Testkit-owned server admission and the library dependency boundary.
  [#45](https://github.com/dragginzgame/ic-auth/issues/45).

- Adopt Shared Tooling 0.9.13, including strict Cargo offline configuration
  checks and CI reuse of verified tool builds. Restored builds still pass explicit
  setup and offline admission; contributors reselect tools with
  `make install-rust-tools` and `make install-testkit-tools`.
  Preserve tool-lock and Cargo source-acquisition failures and their exit statuses.
  [#44](https://github.com/dragginzgame/ic-auth/issues/44).

- Add real Wasm qualification of bounded session decoding and live SDK
  authorization costs, with controller-installed authority and maximum-width
  scope records. Exercise generation, Root key/deadline, scope, missing-session
  and expiry refusals without changing the published API or stored format.
  [#43](https://github.com/dragginzgame/ic-auth/issues/43),
  [Canic #525](https://github.com/dragginzgame/canic/issues/525).

## [0.5.7] - 2026-10-12

- Adopt Shared Tooling 0.9.6 with bounded transient setup-download retries and
  stricter snapshot/release Git identities. Wait for the exact uploaded CI
  artifact to become visible before its pinned download; exhaustion and invalid
  identity remain failures, with digest and independent byte checks retained.
  [#42](https://github.com/dragginzgame/ic-auth/issues/42),
  [Shared #138](https://github.com/dragginzgame/shared-tooling/issues/138),
  [#133](https://github.com/dragginzgame/shared-tooling/issues/133),
  [#139](https://github.com/dragginzgame/shared-tooling/issues/139),
  [#63](https://github.com/dragginzgame/shared-tooling/issues/63).

## [0.5.6] - 2026-10-12

- Qualify locked native tooling at Host 0.14.10 / Testkit 0.34.6 and updated
  native TLS/macro dependencies, retaining Rust 1.88, Testkit-owned server setup,
  offline admission and protocol library boundaries.
  [#41](https://github.com/dragginzgame/ic-auth/issues/41).
- Update Canic adoption guidance for published verification/session APIs,
  protected verifier inputs and the remaining durable, lifecycle and browser
  reconciliation contracts.
  [Canic #491](https://github.com/dragginzgame/canic/issues/491),
  [#3](https://github.com/dragginzgame/ic-auth/issues/3).

## [0.5.5] - 2026-10-12

- Adopt committed Shared Tooling 0.9.2’s unified snapshot reader, including NUL
  rejection and complete validated-record output. Keep ownership/integrity
  checks separate and preserve the selected command surface.
  [#39](https://github.com/dragginzgame/ic-auth/issues/39),
  [Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133).
  Reject NUL in evidence checksum manifests and invalid consumer paths before
  tool setup. Shared installer cleanup releases both locks even if checkout
  cleanup fails, preserving original failures and failed candidates.
- Bind CI npm installation and later steps to the selected Node installation.
  Retain observed versions, paths and prefixes, and preserve the first setup
  failure through log capture. Exact pins and offline admission remain enforced.
  [#38](https://github.com/dragginzgame/ic-auth/issues/38).
- Qualify the incoming locked Host 0.14.9 / Testkit 0.34.5 pairing for native
  qualification. Library APIs, formats and compiler requirements stay unchanged.
  [#40](https://github.com/dragginzgame/ic-auth/issues/40).

## [0.5.4] - 2026-10-12

- Adopt committed Shared Tooling 0.8.4 for strict snapshot records, safer retained
  Cargo build diagnostics, consistent Rust formatter catalog selection and quiet
  successful PCRE2 checks. Refreshes replace payloads atomically and retire only
  unchanged owned files outside the selected entrypoints. Add the Cargo-tool
  freshness task document; its checker remains Shared-owned. Read workspace LOC
  as numeric JSON with `cloc.sh --json`, preserving the existing human table.
  [#37](https://github.com/dragginzgame/ic-auth/issues/37),
  [Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131),
  [#135](https://github.com/dragginzgame/shared-tooling/issues/135),
  [#136](https://github.com/dragginzgame/shared-tooling/issues/136).

- Qualify the incoming native Host 0.14.8 / Testkit 0.34.4 pairing with server
  16.1.0, including real Root/issuer certification, upgrades, complete-token/
  session and signed-ingress journeys. Testkit fixes interrupted server-download
  cleanup; setup remains explicit and checks remain offline. Auth public APIs,
  signed/stored formats and compiler requirements are unchanged.
  [#36](https://github.com/dragginzgame/ic-auth/issues/36).

## [0.5.3] - 2026-10-12

- Qualify Root role-attestation and issuer-token signature profiles in one
  host-composed store using real IC certificates, shared capacity, isolated
  expiry/cleanup and upgrade re-preparation. Give Canic evidence for replacing
  its parallel signature trees while retaining one certification-root owner.
  Public Auth APIs and signed/stored formats are unchanged.
  [#35](https://github.com/dragginzgame/ic-auth/issues/35),
  [Canic #60](https://github.com/dragginzgame/canic/issues/60).
- Qualify incoming Testkit 0.34.3 with Host 0.14.5 and its server 16.1.0 pairing;
  selected CLI preparation remains explicit and native dependencies remain
  outside both libraries.

## [0.5.2] - 2026-10-12

- Select explicit Shared Tooling snapshot entrypoints and derive their required
  companions from the reviewed commit, preserving the existing payload and
  command surface. Future refreshes reuse the recorded roots; omitted files
  require individual ownership-checked retirement.
  [#34](https://github.com/dragginzgame/ic-auth/issues/34),
  [Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131).
- Qualify the incoming native Host 0.14.5 selection with Testkit 0.34.2 and its
  admitted server 16.1.0, retaining explicit setup/offline checks and native
  dependencies outside both libraries. Auth APIs, signed/stored formats and
  compiler requirements are unchanged.
  [#34](https://github.com/dragginzgame/ic-auth/issues/34).

## [0.5.1] - 2026-10-12

- Adopt Shared Tooling 0.6.6's common Make entrypoint for setup, formatting,
  hook installation, snapshot checks and release routing, preserving Auth's
  commands, ordered MSRV/Testkit admission and complete validation gate.
  [#33](https://github.com/dragginzgame/ic-auth/issues/33),
  [Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130).
- Prepare the shared cargo-sort-derives 0.14.1 pin, which preserves literal
  `$crate` paths in macro derives and conditional derives. Auth's default
  formatting policy remains Cargo manifest sorting and rustfmt.
  [Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113).
- Seal declared Rust/frontend manifests, locks and tool/compiler pins at CI stage
  boundaries. Input drift or missing files stop later dispatch, retain original
  failures and require fresh preparation and validation. Adopt the staged-hook
  receipt fix without installing tools in the index export.
  [Shared #129](https://github.com/dragginzgame/shared-tooling/issues/129).
- Remove repeated recipe override warnings from release, tool-evidence, Testkit
  and jobserver fixtures. Substitute script effects while exercising the real
  Make recipes and retaining diagnostics, failure order and recovery evidence.
  [Shared #128](https://github.com/dragginzgame/shared-tooling/issues/128).
- Upgrade all four native Host packages to published 0.14.4, preserving incoming
  Candid 0.10.38, Testkit 0.34.1 and unrelated dependency selections. Qualify with
  isolated package MSRV, signed-byte/client contracts and
  real canister certification, upgrades and ingress. Server 16.1.0 stays selected;
  native tooling remains outside both libraries.

## [0.5.0] - 2026-10-11

- Breaking: move the canonical session request API to the default-feature
  `ic_auth::session_request` module and return `SessionRequestError` from its
  constructor. Update the former `session::SessionRequest` import and constructor
  error handling; no compatibility alias remains. Canic can reuse request hashes
  without enabling cryptographic sessions. The engine shares this owner and
  preserves request hashes, signed bytes and stored session encoding.
  [#32](https://github.com/dragginzgame/ic-auth/issues/32).

## [0.4.7] - 2026-10-11

- Add pure certificate issuance validation with explicit Root and host limits,
  typed lifetime/grant/binding refusals and unchanged signed fields. Canic can
  retire its duplicate certificate rules while retaining signing and authority.
  Verification and token preparation reuse the applicable rules without changing
  their rejection contracts. [#30](https://github.com/dragginzgame/ic-auth/issues/30).

- Add explicit compact-signature normalization so Canic can reuse K256's low-S
  conversion for management signing replies. Strict verification still refuses
  high-S input; normalization does not authenticate a reply or change authority.
  [#31](https://github.com/dragginzgame/ic-auth/issues/31).

- Qualify the incoming Testkit 0.34.0 update with its aligned Host 0.14.1 graph
  and existing server 16.1.0 pairing. The selected CLI requires explicit setup
  and offline admission; native dependencies remain outside both libraries.
  [Testkit #61](https://github.com/dragginzgame/ic-testkit/issues/61).

## [0.4.6] - 2026-10-11

- Add independently selectable chain-key public-key admission and prehash
  signature verification so Canic can remove duplicate signing-response checks.
  Reuse the same cryptographic boundary in Root-proof verification, preserving
  signed bytes, rejection order and existing token errors.
  [#29](https://github.com/dragginzgame/ic-auth/issues/29).

## [0.4.5] - 2026-10-11

- Adopt Shared Tooling 0.6.3 so standard release targets restart interrupted
  preparation after its local metadata disposition is committed in a clean
  descendant. Retain the old intent and receipts, select the requested increment
  from current metadata and require fresh validation. Exact resume stays bound
  to its saved source; library contracts and dependency selections are unchanged.
  [#28](https://github.com/dragginzgame/ic-auth/issues/28),
  [Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126).

## [0.4.4] - 2026-10-11

- Adopt Shared Tooling 0.6.1 so failed Cargo-tool setup stops with its original
  failure status and retains prior selections and failure evidence. Contributors
  must rerun explicit Rust/Testkit tool setup and offline admission; library APIs,
  formats and compiler requirements are unchanged.
  [#26](https://github.com/dragginzgame/ic-auth/issues/26),
  [Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120).

- Qualify the preserved Testkit 0.33.7 selection with Host 0.13.5 and the existing
  server 16.1.0 pairing. These patches update contributor tooling and failure
  retention, with native dependencies remaining outside both public libraries.
  [Testkit #59](https://github.com/dragginzgame/ic-testkit/issues/59),
  [Testkit #60](https://github.com/dragginzgame/ic-testkit/issues/60),
  [Host #65](https://github.com/dragginzgame/ic-host-tooling/issues/65).

## [0.4.3] - 2026-10-11

- Check completed browser token flights against each caller's captured scope and
  current claim validity. Conflicting absent/present-empty extension options
  reject without changing stored keys, clearing uncertain issuance or allocating
  another operation. Same-scope callers still share detached results; Rust APIs,
  signed bytes and the stored profile are unchanged.
  [#27](https://github.com/dragginzgame/ic-auth/issues/27).

- Qualify the preserved Host 0.13.4 native selection with Testkit 0.33.6 and the
  existing server 16.1.0 pairing. Native tools remain outside both public libraries.

## [0.4.2] - 2026-10-11

- Adopt Shared Tooling 0.6.0's shared Cargo storage and standard tool-build
  environment. Contributors must explicitly prepare/check common Rust tools and
  the selected Testkit CLI; existing PATH entries and old installations remain.
  Retain referenced shared build failures through the hosted evidence collector
  while excluding unrelated generations. The library contract is unchanged.
  [#26](https://github.com/dragginzgame/ic-auth/issues/26),
  [Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120),
  [Shared #121](https://github.com/dragginzgame/shared-tooling/issues/121).

- Add pure delegated-token preparation and finishing with explicit host authority,
  clock, operation and request bounds. Immutable prepared claims preserve Canic's
  nonce framing, signed bytes, requested TTL and typed refusal precedence. Route
  the actual certified-token/upgrade fixture through this API so Canic can replace
  its duplicate issuance helper. Signing and issuer approval remain host-owned;
  existing APIs and wire/storage formats are unchanged.
  [#25](https://github.com/dragginzgame/ic-auth/issues/25).

- Qualify the selected Testkit 0.33.6 and Host 0.13.2 native graph with separately
  prepared Testkit tools. These upstream contributor-tooling patches preserve
  runtime APIs and the server 16.1.0 pairing; native dependencies remain outside
  the public authentication libraries.
  [Testkit #58](https://github.com/dragginzgame/ic-testkit/issues/58),
  [Host #63](https://github.com/dragginzgame/ic-host-tooling/issues/63).

## [0.4.1] - 2026-10-11

- Adopt Shared Tooling 0.5.0's verified shared host/IC storage and unified Rust
  installation. Contributors must rerun host/IC setup and offline checks, prepare
  the common Rust slots and use `.tools/rust/common/bin`. Host/IC PATH entries
  stay unchanged; earlier installations remain retained. The published library
  contract is unchanged.
  Update the failure-evidence fixture for selected Cargo dispatch and retain its
  actual failed attempt and prior-slot build bytes without following links.
  [#24](https://github.com/dragginzgame/ic-auth/issues/24).

- Prepare and admit the complete tool selection before release validation,
  including missing common tool references. Setup/check failures stop before
  the gate or version changes; interrupted metadata recovery stays unchanged.
  [#24](https://github.com/dragginzgame/ic-auth/issues/24).

- Qualify the current locked Testkit 0.33.5 selection with its separately
  prepared CLI and unchanged server 16.1.0. This upstream patch changes
  contributor tooling; authentication and qualification runtime APIs are unchanged.
  [Testkit #57](https://github.com/dragginzgame/ic-testkit/issues/57).

- Correct published dependency examples and migration guidance for Auth 0.4.0's
  typed Root binding diagnostics and root-policy deadline qualification. Document
  the release's actual Testkit 0.33.4/Host 0.13.1 selections; authentication APIs
  and formats are unchanged.
  [#23](https://github.com/dragginzgame/ic-auth/issues/23),
  [#22](https://github.com/dragginzgame/ic-auth/issues/22).

## [0.4.0] - 2026-10-11

- **Breaking:** distinguish Root signature/header mismatches from enrolled-key
  policy mismatches with separate typed token-verification errors. Consumers
  must update exhaustive matches and diagnostic mappings; signed bytes,
  successful verification and stored formats are unchanged.
  [#23](https://github.com/dragginzgame/ic-auth/issues/23).

- Extend real certified-token qualification through a root-policy cutoff earlier
  than the requested session lease and signed token expiry. Retry and authorization
  reject at the cutoff, while replay protection remains until signed proof expiry.
  This qualification preserves session policy and stored formats.
  [#22](https://github.com/dragginzgame/ic-auth/issues/22).

## [0.3.8] - 2026-10-11

- Adopt Shared Tooling 0.3.12's corrected snapshot identity, independently
  checked diagnostic families and formatting-check support for relative or
  symlinked temporary directories.
  [Shared #114](https://github.com/dragginzgame/shared-tooling/issues/114),
  [Shared #110](https://github.com/dragginzgame/shared-tooling/issues/110),
  [Shared #116](https://github.com/dragginzgame/shared-tooling/issues/116).
- Select Host 0.13 for direct native file operations and Testkit 0.33.2 for
  canister qualification. Public authentication APIs and formats are unchanged.
  Testkit's Host 0.12 dependencies remain upstream-owned pending
  [Testkit #54](https://github.com/dragginzgame/ic-testkit/issues/54).

## [0.3.7] - 2026-10-11

- Adopt Shared Tooling 0.3.9 so Cargo tool setup/checks use the selected
  consumer's configuration and early fatal, LLVM and missing-file diagnostics
  remain visible in both live validation output and retained summaries.
  [Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101),
  [Shared #110](https://github.com/dragginzgame/shared-tooling/issues/110).
- Adopt the subsequent reviewed Shared snapshot's release-check temporary-path
  fix and derive-macro protection guidance. The producer's committed version
  still identifies that revision as 0.3.9.
  [Shared #112](https://github.com/dragginzgame/shared-tooling/issues/112),
  [Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113),
  [Shared #114](https://github.com/dragginzgame/shared-tooling/issues/114).
- Select the published Host 0.12.10 graph for native tooling and qualification.
  Public authentication APIs, formats and the Testkit/server pairing are unchanged.
- Accept padded macOS line counts in publication/reconciliation fixtures and
  exercise that output on every host, preserving exact dispatch assertions.
  [#19](https://github.com/dragginzgame/ic-auth/issues/19).

## [0.3.6] - 2026-10-10

- Qualify both exact, untouched package archives at the minimum Rust compiler
  through an isolated consumer. Reject incorrect package identities, forbidden
  dependency edges, changed external lock selections and changed payloads or
  inputs. Packaged manifests and locks are no longer rewritten.
  [#21](https://github.com/dragginzgame/ic-auth/issues/21).
- Adopt Shared Tooling 0.3.8's coordinated-archive guidance while preserving
  ordinary registry publication admission.
  [Shared #109](https://github.com/dragginzgame/shared-tooling/issues/109).

## [0.3.5] - 2026-10-10

- Extend real certified-token session qualification through a substituted host
  transaction refusal after verification/staging. Rejected admission leaves
  session/replay records, quotas, expiry indexes and accounting unchanged; an
  exact retry also propagates storage failure without returning authority or
  extending the committed lease. Consumer durable storage remains separate.
  Public authentication APIs and formats are unchanged.
  [#20](https://github.com/dragginzgame/ic-auth/issues/20).
- Reconcile documentation with published 0.3.4, its Testkit 0.33.1/Host 0.12.7
  selections and the current CI cancellation policy.

## [0.3.4] - 2026-10-10

- Qualify session admission from an actual certified token, including rejection
  without proof consumption, exact retry after proof expiry and live authority
  invalidation. The reference store remains volatile; consumer durable storage
  adoption is separate. Public APIs and formats are unchanged.
  [#18](https://github.com/dragginzgame/ic-auth/issues/18).
- Adopt Shared Tooling 0.3.7, including failed Git-observation, incomplete-checker
  and mandatory Bash 3.2 assertion refusal. Run actual formatting-hook
  preservation in the complete CI gate.
  [#11](https://github.com/dragginzgame/ic-auth/issues/11),
  [Shared #103](https://github.com/dragginzgame/shared-tooling/issues/103),
  [Shared #106](https://github.com/dragginzgame/shared-tooling/issues/106).
- Use the canonical lockfile-selected Cargo installer for Testkit, retaining
  offline admission and refusal of selection changes after server setup/check.
  [Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96).
- Enforce failed comparisons in owned portable release and Testkit fixtures
  explicitly, preventing Bash 3.2 from skipping required assertions.
  [#19](https://github.com/dragginzgame/ic-auth/issues/19),
  [Shared #107](https://github.com/dragginzgame/shared-tooling/issues/107).
- Keep only the newest CI run per workflow and branch or PR, cancelling older
  queued and running checks while retaining the existing host matrix and gates
  ([Shared #108](https://github.com/dragginzgame/shared-tooling/issues/108)).
- Select published Testkit 0.33.1 and the single Host 0.12.7 graph, including
  rejection of NUL-containing publication paths before parent creation. Native
  dependencies remain outside both authentication libraries.

## [0.3.3] - 2026-10-10

- Qualify complete application-token verification with an actual certified
  issuer, live authority rejection and re-preparation after canister upgrade,
  using the published bounded Merkle constructor. Public APIs and signed bytes
  are unchanged. [#18](https://github.com/dragginzgame/ic-auth/issues/18).
- Adopt Shared Tooling 0.3.4, including malformed validation-depth rejection
  and reliable premature-exit failure retention on Bash 3.2. Prepare the common
  Binaryen 133 selection explicitly; earlier bundles remain retained.
  [Shared #104](https://github.com/dragginzgame/shared-tooling/issues/104),
  [Shared #102](https://github.com/dragginzgame/shared-tooling/issues/102).
- Select Testkit 0.32.2 and a single Host 0.12.4 graph for native qualification.
  Native tooling stays outside both authentication libraries.

## [0.3.2] - 2026-10-10

- Preserve jobserver descriptors during dependency-pin validation, completing
  the owned Cargo-wrapper repair.
  [#17](https://github.com/dragginzgame/ic-auth/issues/17).
- Adopt Shared Tooling 0.3.2's jobserver handoff and explicit fixture-completion
  checks, preserving unsafe-mode refusal and failed evidence on Bash 3.2.
  [Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99),
  [Shared #103](https://github.com/dragginzgame/shared-tooling/issues/103).
- Qualify Testkit 0.32 and a single Host 0.12.2 dependency graph for native
  canister tests, removing duplicate Host 0.11 selections. Authentication APIs,
  signed bytes and stored formats are unchanged.

## [0.3.1] - 2026-10-10

- Preserve Make's shared job budget at IC Auth-owned Cargo and release entry
  points, removing closed-descriptor warnings during parallel execution while
  retaining refusal of unsafe Make modes before any effects.
  [#17](https://github.com/dragginzgame/ic-auth/issues/17).
- Adopt Shared Tooling 0.3.1's read-only platform/toolchain preflight and precise
  host-tool diagnostics, preserving setup ordering, offline checks and retained
  evidence. Qualify the selected Host filesystem/artifact 0.12.1 packages;
  authentication APIs and signed bytes are unchanged.
  [#16](https://github.com/dragginzgame/ic-auth/issues/16),
  [Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101).

## [0.3.0] - 2026-10-10

### Breaking

- Adopt Shared Tooling 0.3.0's complete toolset and ordered setup/check contract.
  Host setup always includes jq, yq, ripgrep and cloc; direct installer callers
  must remove `--with-ripgrep`/`--with-cloc`. Common host, IC and Rust tools run
  before the minimum compiler and selected Testkit CLI/server, stopping on
  failure even under parallel Make. Run explicit `make install-tools`, then
  `make tools-check`; existing matching installations and evidence are retained.
  Authentication APIs, signed bytes and stored formats are unchanged.
  [#11](https://github.com/dragginzgame/ic-auth/issues/11),
  [Shared #98](https://github.com/dragginzgame/shared-tooling/issues/98).

### Changed

- Qualify the incoming Testkit 0.30 selection for native canister tests, keeping
  its bounded build diagnostics and complete identity/metadata probes owned by
  Testkit. [Testkit #49](https://github.com/dragginzgame/ic-testkit/issues/49).

## [0.2.11] - 2026-10-10

- Add bounded chain-key Merkle batch construction so Canic can retire its local
  builder, preserving existing roots, witness ordering and signed bytes.
  [#15](https://github.com/dragginzgame/ic-auth/issues/15),
  [Canic #491](https://github.com/dragginzgame/canic/issues/491).
- Adopt the incoming Testkit 0.29 selection and unify native Host dependencies
  on 0.11, retaining Testkit-owned CLI/server setup and admission.
  [Testkit #47](https://github.com/dragginzgame/ic-testkit/issues/47).

## [0.2.10] - 2026-10-10

- Use Host 0.11 filesystem helpers in native tooling and qualification, retaining
  rejection of directory-suffixed publication targets and reporting staging
  cleanup failures alongside the original failure.
  [Host #44](https://github.com/dragginzgame/ic-host-tooling/issues/44).
- Prepare the selected Testkit CLI during release preflight and check tools
  before complete validation or standalone qualification builds.
  [#14](https://github.com/dragginzgame/ic-auth/issues/14),
  [Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96).
- Adopt Shared Tooling 0.2.13 with exact selected-tool failure diagnostics,
  preserving Make failure propagation when callers replace Make flags and
  refusing snapshot paths containing LF or CR.
  [#11](https://github.com/dragginzgame/ic-auth/issues/11),
  [Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96),
  [Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30),
  [#95](https://github.com/dragginzgame/shared-tooling/issues/95).

## [0.2.9] - 2026-10-10

- Adopt Shared Tooling 0.2.10 with concise formatting output and retained
  failure logs; authenticate exact current-run CI artifact readback.
  [#13](https://github.com/dragginzgame/ic-auth/issues/13).
- Use bounded shared registry metadata observation for publication, preserving
  exact archive acceptance and uncertain-upload reconciliation while retaining
  every observation separately. Publication requires curl 8.4.0+.
  [#12](https://github.com/dragginzgame/ic-auth/issues/12).
- Identify the locked Testkit CLI and explicit setup command when offline
  admission fails; qualify the incoming Testkit 0.28.1 selection.
  [#14](https://github.com/dragginzgame/ic-auth/issues/14).

## [0.2.8] - 2026-10-10

- Refuse browser retrieval completion when ready-token storage finishes at or
  after the original deadline, retaining committed material for subsequent
  valid cache lookup. [#3](https://github.com/dragginzgame/ic-auth/issues/3).
- Adopt Host 0.10.1's filesystem publication API and parent durability fix,
  preserving private create-only intent. Testkit 0.28.0 unifies the Host graph;
  prepare its selected CLI with `make install-testkit-tools`.
  [Host #43](https://github.com/dragginzgame/ic-host-tooling/issues/43),
  [Testkit #44](https://github.com/dragginzgame/ic-testkit/issues/44).

## [0.2.7] - 2026-10-09

- Refuse fresh browser-client request IDs and intent reservations after storage
  latency, CAS contention or TTL-backoff clearing exhausts the original operation
  deadline. [#3](https://github.com/dragginzgame/ic-auth/issues/3).
- Adopt Shared Tooling 0.2.8 Make admission fixes: use the selected snapshot's
  execution probe and support recursive Make commands with extra arguments.
  [#11](https://github.com/dragginzgame/ic-auth/issues/11),
  [Shared Tooling #30](https://github.com/dragginzgame/shared-tooling/issues/30).
- Select Host 0.9.7 for native tooling and canister qualification, retaining
  IC Testkit 0.27.2 and its PocketIC 16.1.0 server pairing.

## [0.2.6] - 2026-10-09

- Enforce the browser client's original exclusive deadline on prepare and
  reconciliation replies, preserving saved intent without TTL backoff, new
  request IDs or session invalidation after expiry.
  [#3](https://github.com/dragginzgame/ic-auth/issues/3).
- Qualify incoming Host 0.9.5 and IC Testkit 0.27.2 for native tooling and
  canister lifecycle checks, preparing the matching Testkit CLI while retaining
  its PocketIC 16.1.0 server pairing.
- Adopt Shared Tooling 0.2.7 Make failure-propagation guards and isolated release
  and newline-safe formatting checks, including required fixture companions.
  [#11](https://github.com/dragginzgame/ic-auth/issues/11),
  [Shared Tooling #30](https://github.com/dragginzgame/shared-tooling/issues/30),
  [#7](https://github.com/dragginzgame/shared-tooling/issues/7),
  [#90](https://github.com/dragginzgame/shared-tooling/issues/90).

## [0.2.5] - 2026-10-09

- Extend browser-client recovery coverage to uncertain prepare expiry across
  reloads, preserving original intent and preventing new dispatch or request
  allocation at the stored deadline. Clarify the issuer reconciliation contract
  required for Canic adoption.
  [#3](https://github.com/dragginzgame/ic-auth/issues/3),
  [Canic #507](https://github.com/dragginzgame/canic/issues/507).
- Adopt Shared Tooling 0.2.6 release and single-workspace formatting Make owners,
  preserving IC Auth's release policy, prepared tools and validation gate.
  Include both new Make owners in the isolated tool failure-evidence fixture so
  setup/check qualification reaches the intended injected failures.
  [#11](https://github.com/dragginzgame/ic-auth/issues/11).

## [0.2.4] - 2026-10-09

- Qualify the incoming Host 0.9.3 and IC Testkit 0.27.1 selections for native
  tooling and canister lifecycle checks, retaining Testkit-owned server setup.
- Extend signed-token coverage for IC trust-anchor changes and certificate
  freshness after prior success. Document complete-verifier adoption and
  effective installation deadlines for Canic.
  [Canic #491](https://github.com/dragginzgame/canic/issues/491),
  [#58](https://github.com/dragginzgame/canic/issues/58),
  [#506](https://github.com/dragginzgame/canic/issues/506).

## [0.2.3] - 2026-10-09

- Adopt the latest reviewed Shared Tooling fixes for unterminated IC-tool pin
  matrices and literal newlines in hook paths, preserving existing selections
  and failed Git-read status.
  [Shared Tooling #87](https://github.com/dragginzgame/shared-tooling/issues/87),
  [#89](https://github.com/dragginzgame/shared-tooling/issues/89).

## [0.2.2] - 2026-10-09

- Add bounded standalone root delegation-proof verification for issuer
  installation before a token exists. Reuse complete-token certificate and
  cryptographic checks with protected issuer, key, clock and limits; return
  proof evidence separately from token or issuance authorization.
  [#10](https://github.com/dragginzgame/ic-auth/issues/10),
  [Canic #491](https://github.com/dragginzgame/canic/issues/491).

## [0.2.1] - 2026-10-09

- Run the full three-host CI gate automatically for `main` pushes and pull
  requests, avoiding duplicate release-tag validation. Add manual qualification
  of a selected branch/tag while retaining the complete matrix and failure
  evidence. [#9](https://github.com/dragginzgame/ic-auth/issues/9).
- Select the matching IC Testkit 0.27.0 CLI for the incoming dependency update
  and retain Host 0.9.2 for native tooling and canister qualification.
  [IC Testkit #30](https://github.com/dragginzgame/ic-testkit/issues/30),
  [Host Tooling #39](https://github.com/dragginzgame/ic-host-tooling/issues/39).

## [0.2.0] - 2026-10-09

- **Breaking developer setup contract:** adopt Shared Tooling 0.2.0 and transfer
  PocketIC provisioning/admission to the locked IC Testkit CLI. Run explicit
  `make install-tools` (or `make install-ic-tools install-testkit-tools`) to
  prepare the new five-tool bundle and Testkit server. Offline qualification
  consumes Testkit's admitted path; the duplicate server matrix and alignment/
  binary checkers are removed. Release/publication fixtures export the current
  Testkit callers without requiring the retired matrix. Old bundles and evidence
  remain retained.
  [#7](https://github.com/dragginzgame/ic-auth/issues/7),
  [Shared Tooling #76](https://github.com/dragginzgame/shared-tooling/issues/76).
- Adopt checkout-local formatter discovery and single-document dependency
  exception admission from Shared Tooling 0.1.37/0.1.38.
  [Shared Tooling #85](https://github.com/dragginzgame/shared-tooling/issues/85),
  [#86](https://github.com/dragginzgame/shared-tooling/issues/86).
- Extend Canic Candid qualification to complete retrieved tokens, delegation
  certificates and chain-key batch proofs, preserving witness directions and
  opaque signature/key bytes while rejecting malformed nested roles/identities.
  [#8](https://github.com/dragginzgame/ic-auth/issues/8).
- Update the published-library adoption contract for 0.1.14 and record Canic's
  in-progress canonical encoding/proof adoption and remaining complete-verifier
  boundary. [Canic #491](https://github.com/dragginzgame/canic/issues/491).
- Retain the incoming Host Tooling 0.9.0 and IC Testkit 0.26.0 selections for
  native utilities and canister qualification, using Testkit's matching CLI.
  [Host Tooling #38](https://github.com/dragginzgame/ic-host-tooling/issues/38),
  [IC Testkit #38](https://github.com/dragginzgame/ic-testkit/issues/38).

## [0.1.14] - 2026-10-09

- Document the Canic adoption contract for the published libraries, with feature
  selection, protected authority inputs and durable storage/certification
  contracts. Consumer implementation and qualification remain Canic-owned.
  [Canic #491](https://github.com/dragginzgame/canic/issues/491).
- Qualify Canic's prepare request, claims/prepare response and retrieval
  envelopes in both Candid directions, preserving metadata and optional bytes
  while rejecting malformed nested identities and roles.
  [#8](https://github.com/dragginzgame/ic-auth/issues/8).

## [0.1.13] - 2026-10-09

- Adopt Shared Tooling 0.1.36's canonical installer for consumer-selected Cargo
  binaries/examples. Reject conflicting receipt documents, recheck installation
  paths after Cargo returns and retain Cargo's original failure status and build
  evidence. Existing valid installations and the formatter bundle remain usable.
  [Shared Tooling #65](https://github.com/dragginzgame/shared-tooling/issues/65).
- Adopt the common locked dependency-preparation policy; release preflight
  continues fetching the selected lock before offline validation and preserves
  explicit offline settings.
  [Shared Tooling #84](https://github.com/dragginzgame/shared-tooling/issues/84).
- Select published Host Tooling 0.8.9 for native file utilities and IC Testkit,
  retaining the current file/process contracts.
  [Host Tooling #36](https://github.com/dragginzgame/ic-host-tooling/issues/36).

## [0.1.12] - 2026-10-09

- Preserve the native MSRV lane under macOS Bash 3.2 without weakening strict
  shell checks or the package/feature matrix.
  [#4](https://github.com/dragginzgame/ic-auth/issues/4).
- Retain actual Rust-tool setup/check output and verify failure status, build
  evidence and exact uploaded/downloaded bytes in native CI.
  [#2](https://github.com/dragginzgame/ic-auth/issues/2).
- Adopt Shared Tooling 0.1.34 and retire the unused consumer fleet reporter;
  retain local workspace LOC and common setup/check commands.
  [#5](https://github.com/dragginzgame/ic-auth/issues/5).
- Use Host Tooling 0.8.8's bounded streaming hash for release file identities,
  retaining final-symlink refusal and exact digest output without buffering the
  complete file. Remove the native utility's direct artifact dependency.
  [#6](https://github.com/dragginzgame/ic-auth/issues/6).
- Qualify the selected Testkit 0.25.4 runtime with the existing pinned PocketIC
  16.1.0 server; retain the current setup until the replacement setup/check
  contract is consumer-qualified.
  [Testkit #38](https://github.com/dragginzgame/ic-testkit/issues/38).
- Preserve literal manifest paths and stop before Cargo if the selected directory
  disappears during qualification alignment checks.
  [Shared Tooling #82](https://github.com/dragginzgame/shared-tooling/issues/82).

## [0.1.11] - 2026-10-09

- Add an opt-in IndexedDB token store with atomic cross-connection compare-and-swap,
  bounded admission and completion after strict transaction commit. Retain
  uncertain issuance intent and refuse incompatible/corrupt storage without reset.
  [IC Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
- Adopt Shared Tooling 0.1.32 so verified IC tools can be reused when pin comments
  or row order change, preserving installation receipts and checksum checks.
  [Shared Tooling #79](https://github.com/dragginzgame/shared-tooling/issues/79).
- Select published Host Tooling 0.8.5 for native file operations and IC Testkit,
  preserving bounded no-follow reads and private publication evidence. Keep
  the existing PocketIC setup until Testkit's replacement contract is qualified.
  [Shared Tooling #76](https://github.com/dragginzgame/shared-tooling/issues/76).

## [0.1.10] - 2026-10-09

- Add a private browser application-token client with injected authenticated
  identities and issuer operations, scoped caching, single-flight renewal and
  generation-safe invalidation. Preserve uncertain issuance intent and use
  explicit no-effect TTL outcomes for fallback.
  [IC Auth #3](https://github.com/dragginzgame/ic-auth/issues/3).
- Generate client Candid contracts from the Rust protocol owner and add explicit
  Node/npm setup, offline lifecycle tests and cross-language wire checks.
  The client is not published and consumer adapter adoption remains pending.

## [0.1.9] - 2026-10-08

- Adopt Shared Tooling 0.1.29 and report all refused staged, unstaged and
  untracked release paths, including lockfile changes, while preserving source
  and index bytes. Initial preflight refusals identify that validation and
  version preparation have not started for that attempt.
  [shared #74](https://github.com/dragginzgame/shared-tooling/issues/74).
- Preserve exact digest and release metadata output under inherited `CDPATH`,
  using physical checkout paths throughout native tooling and release fixtures.

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
