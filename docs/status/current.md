# Current handoff — 2026-11-02

## Delegated 0.32 delivery

The user requested deployment of the cleanup: standard Protocol and Engine minor
releases, exact commits/tags pushed to origin/main and all three crates published
to crates.io. No service, sibling, npm deployment or data reset is authorized.
Keep the dedicated release checkout `/tmp/ic-auth-release.qoEtNy` isolated from
ordinary edits. Development changes belong here, then fast-forward that checkout
only when its release process is stopped and source/index are admitted clean.
`target/complexity-032-delivery/` records delivery scope, source and evidence.

Protocol **0.32.0** is published and checksum-verified at
`d31dc297e5026c2e6d2b1b97af99a761b50ae686`, annotated tag `protocol-v0.32.0`.
Its complete gate passed in `validation.xgxTtJ`, under the release checkout's
`.git/release-state/validation-logs/`. Source is
`119ea01520b9c129a8dd37a7c206a5d5792a5263`; publication evidence is
`target/publication-protocol-0.32.0.ZMVFZG/` there. Engine remains **0.31.0** with
pending **0.32.0** notes; no Engine 0.32 commit, tag or upload occurred.

The first Engine gate passed locally (`validation.021air`) against the Protocol
commit, but delivery stopped during preparation because the hosted Protocol run
[41523561693](https://github.com/dragginzgame/ic-auth/actions/runs/41523561693)
failed its Apple Silicon client test. The 5 ms real deadline could precede mocked
transport entry after IndexedDB work. The repair uses Node's controlled timers,
waits for transport entry, then advances the same 5 ms deadline; production code
and timeout/cancellation assertions are unchanged. Its focused regression passes;
client-suite evidence is `target/complexity-032-delivery/controlled-clock-client-suite.log`.

The stopped Engine plan is in phase `prepare`, bound to the Protocol source.
Tracked files/index remain unchanged and no candidate tag exists. Preserve its
plan, receipt, `metadata.iS4j8p` and logs. Commit the test repair, then start a
fresh normal Engine release and complete gate from that source. Do not reuse the
older successful gate as proof of the repaired source or run exact resume with it.

Earlier attempts remain retained: `validation.DpoFIl` was interrupted for the API
documentation correction; `validation.tf4ZxL` caught a nested console assertion;
`validation.CiPdKc` caught missing read-only inventory in the Testkit substitute.
Repaired fixtures inspect retained checkpoints and preserve real Cargo inventory
while substituting product effects. Nested input-seal, Testkit and client caller
regressions pass. Do not erase these attempts or disable source-drift admission.

## Scope and frozen selections

Audit items 1–9 are implemented: shared private codec bindings and canonical
per-method guards; one login record representation; consuming checkpoint restore;
borrowed session profiling; owned role projections; checked grant scopes and
64-byte roles; one private enrollment roster; shared browser u64 checks; and
Cargo-discovered CI manifest sealing. The maintainer confirmed no retained roles
above 64 bytes need preservation. Valid signed/Wire/storage bytes and crypto/trust
checks remain unchanged; malformed portable scopes fail as `InvalidShape` at decode.
The private canister also drops unused direct Metrics and Serde-bytes declarations.

Group-scoped release validation (item 10) remains unselected. Both groups retain
the complete gate. No matched Wasm or latency improvement is claimed.
Earlier focused evidence is under `target/complexity-031/`; the later release gates
qualify the frozen package selections. Chromium remains opt-in and was not run.

The release selects Platform **0.18.0**, Types **0.17.0**, Runtime/macros/guard
**0.18.1**, Wire/derive **0.15.1**, Storage **0.65.2**, Metrics **0.27.2**, Testkit
**0.80.1**, Build **0.74.1**, Host FS/Process/Artifacts **0.74.2** and ctutils
**0.4.8**. The independent file CLI stays **0.74.1**. PocketIC server **16.1.0**
and browser SDK **0.1.1** are unchanged. Generation/review found all five browser
files unchanged. Later development dependency edits must remain outside the
frozen release checkout and are not qualified by its receipts.

Shared Tooling **0.27.28**, `8c7fae6605c69a930e3b1f14e1ff5c4c9dafbcb7`, is
source-verified (145 payloads). Rust **1.99.1** is pinned/minimum. The host retains
one Runtime/generator/guard; public capability graphs exclude Runtime, native SDKs,
Storage and Candid. Passive identities use Types directly; crypto capabilities
select Platform explicitly. The native boundary retains explicit external conversion.

## Storage, trust and retained custody

Storage 0.65 replaced Memory/stable-structures in the private checkpoint host and
requires fresh backing. Old manager/ledger images are refused without fallback or
reset. No retained backing was cleared; production needs its own reviewed whole-
image transition. No production wallet service/backend is deployed here. The host
owns checked allocation, clock, atomicity, protected generation and root publication.

Keep `AUTHNAT3`, session/login IDs `0x415554485311` / `0x415554485313` and replay
`0x415554485302`. Preserve cryptographic algorithms, signed material, certificate
and delegation checks and the TEE/non-TEE model. There is no verification-result
cache. Canic consumers must adopt Protocol 0.32's checked labels explicitly.

Protocol **0.31.0** remains at `ae8ae1dc20c26d70a026dc9481a465f4d39338fd` and
Engine **0.31.0** at `38ba68f6ad06735d94566e964c11169bf46413da`, with their group
tags and verified publications. Keep `/tmp/ic-auth-release.x4YbcZ` and all saved
release/publication custody; no uncertain upload remains. Earlier 0.30 identities,
evidence paths and historical measurements are recorded in the preceding handoff
at `d08ae8916dca83b6583e1550173328b877ddd41f`. Preserve that evidence and
`/tmp/ic-auth-release.EqLgnm` if present; recover historical attempts with their
original adapter, source, group, versions and formats. No cleanup is authorized.

[Auth #124](https://github.com/dragginzgame/ic-auth/issues/124) is closed against
`28ee29d12122cfbfe05b19a49460bd5ee79bf7c8` and passing all-host
[CI 41434941116](https://github.com/dragginzgame/ic-auth/actions/runs/41434941116).
[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns installed
adoption, issuer/renewal, guards, durable backend and browser acceptance; #720 owns
storage deduplication. No sibling checkout or deployment was changed.
