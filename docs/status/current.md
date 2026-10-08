# Current handoff — 2026-10-08

The maintainer accepted the larger Canic authentication extraction, selected the
name `ic-auth`, and confirmed `/home/adam/projects/ic-auth` as the local repository.

The local Git repository is initialized on `main` with no commits.
The public remote [dragginzgame/ic-auth](https://github.com/dragginzgame/ic-auth)
was created at the maintainer's request and is configured as `origin` using
`https://github.com/dragginzgame/ic-auth.git`. The remote is empty; no source push
has occurred.
The accepted [design](../design/extraction.md) covers signatures, application
tokens and local sessions, plus an independent Solana wallet-auth service.
[Canic #491](https://github.com/dragginzgame/canic/issues/491) remains the existing
extraction and Canic adoption tracker. New IC Auth-specific issues belong in
the public repository's issue tracker.
The [bootstrap evidence comment](https://github.com/dragginzgame/canic/issues/491#issuecomment-6055817375)
records this batch without closing the larger extraction.

The virtual Rust workspace now contains `ic-auth-types` and `ic-auth`, with one
lockfile, root-owned dependency selections and initial local version `0.1.0`.
The passive token/proof contracts and canonical encoding are adapted from the
clean Canic source at `e286b3fd98460c98670336853f80658a920966e0`; see the
[source review](../design/canic-source-review.md) for exact ownership and API
adaptations. Both packages have `publish = false`. There is no token verifier,
session engine, wallet endpoint or TypeScript client yet. Canic adoption has not
occurred; no sibling repository was modified.

The engineering snapshot contains 56 committed Shared Tooling files from
`2687f26317952c43c685f7f799ed09288dc10a67`. It was exported from a clean temporary
clone, preserving the source checkout's unrelated uncommitted work. The snapshot
verifier passed during installation. The expanded snapshot supplies shared
Make/tool setup, formatter hook and focused governance helpers. Pinned host,
Rust and IC tools are installed in this checkout and `make tools-check` passed
on Linux x86_64. Tool installation output is retained at
`/tmp/ic-auth-install-tools.log`; installation build output remains under
`.tools/rust/build/`. The repository-local formatting hook is enabled.

This starts A1 and the encoding portion of A3 without claiming either complete.
Protocol `Fleet` labels and signed domains remain unchanged. Auth role parsing
now validates at construction/deserialization; the host still owns trusted
identity projection and all authority. Remaining Canic consumer adaptation,
generated-Candid/feature coverage and canonical ownership convergence stay in
the tracker. Preserve Canic's current release boundary and keep its reinstall-only
policy out of wallet identity storage.

Focused validation passed: 17 tests across both packages, the three imported
Canic hash vectors, Candid identity/role byte compatibility, Wasm compile checks,
Clippy with warnings denied, locked metadata, transitive dependency boundaries,
formatting, snapshot integrity and local links. Initial Clippy validation found a
constant-size iterator lint; it was corrected and the check passed. No full CI,
PocketIC, live verification, service or downstream-adoption evidence is claimed.

The dependency declaration checker correctly rejects the actual bootstrap's
untracked lockfile. It passed with root inheritance enabled in an isolated staged
copy at `/tmp/ic-auth-pinning.uGjpA0`; the user's real index remains untouched.
The standard release entry points passed the shared substitute-runner routing
check, which has no release effects. Hook activation and normal `fmt-check` passed;
the full hook-adoption fixture needs a committed baseline and was not run here.

The pending `0.1.0` changelog records the initial implementation, not a finalized
or published release. There is no earlier release to increment. Release preflight
is intentionally blocked pending an initial release identity and qualified
delivery/metadata adapters. No commit, source push, release, package publication
or deployment occurred. See [development](../development.md) for actual commands
and their boundaries.

Documentation closeout identified the wallet-service contract decisions explicitly
in the design: application identity scoping, credential linking/recovery, distinct
revocation semantics and exact public protocol limits. They do not block initial
Canic library extraction, but must be settled before supported wallet login.
