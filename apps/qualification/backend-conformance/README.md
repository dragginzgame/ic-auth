# Auth backend conformance

Unpublished native qualification support for custom `ic_auth::session::SessionStore`
adapters. This package depends only on public Auth/protocol APIs and Platform's
Ed25519 capability. It has no private endpoint DTOs, metrics, simulator, runtime,
certificate verifier or storage dependencies.

Run `make test-backend-conformance` from the Auth workspace. The same assertions
exercise reference commit callbacks and an independent encoded-record adapter,
and detect a deliberately broken write-then-error implementation. The examples
are qualification mechanics, not production stores.

## Consumer use

After the selected Engine and Protocol dependencies are published, `make pack-backend-conformance`
creates `target/package/ic-auth-backend-conformance-VERSION.crate` without an
upload. Cargo includes the source, tests, README, license, lock and VCS metadata,
and normalizes workspace/path dependencies to registry dependencies. Retain the
archive and its integrity with the consumer's qualification inputs. A dirty VCS
receipt describes a candidate, not a delivered revision.

`make test-backend-package` qualifies a fresh archive in a separate consumer
workspace, using the exact registry Engine and Protocol versions and the source bundle
only as a dev dependency. It compiles a trait/error identity witness, runs all
packaged scenarios, checks invalid source/dependency graph variants and seals source,
archive and consumer inputs. The printed evidence directory retains logs,
metadata and `delivery.json` with source/dirty state, separate Engine/Protocol
versions and archive SHA-256. The bundle follows Engine's version; Protocol can
have a different version and remains one canonical identity in the graph.
This is opt-in delivery qualification, outside ordinary CI. Prepare matching
published registry packages explicitly first. If the offline metadata stage
reports a missing download, run `cargo fetch --locked --manifest-path
<printed-evidence>/consumer/Cargo.toml`, then rerun the check into a fresh retained
directory. The check itself never downloads or uploads.

Current source targets the pending Auth Protocol/Engine 0.30.0 cohort with
Platform 0.17 and Types 0.16. Publish those Auth suppliers before qualifying this
source bundle against the registry. Own metadata still says 0.29.2 until release;
that is not evidence that the registry's 0.29.2 has the current dependency graph.
The published 0.29.2 baseline uses Platform 0.16/Types 0.15. Keep the single-Platform/
Types admission intact; it refuses mixing that baseline with this newer bundle.

Extract this source archive into the consumer's test-support directory and add
it as a **development dependency**. Use the archive's normalized `Cargo.toml`;
depending on the producer's working workspace can introduce a second Git/path
identity for Auth alongside the consumer's registry dependency.

Historical example for the released 0.29.2 bundle (the pending 0.30 bundle requires
its own selected archive/version after publication):

```toml
# Append this path to the consumer's existing workspace exclusions. The foreign
# source bundle must not become a consumer-owned release-group member.
[workspace]
exclude = ["test-support/ic-auth-backend-conformance-0.29.2"]

[dev-dependencies]
ic-auth-backend-conformance = { path = "test-support/ic-auth-backend-conformance-0.29.2" }
ic-auth = { version = "=0.29.2", default-features = false, features = ["session-restore", "portable-signature-verification"] }
```

Implement `ic_auth_backend_conformance::Backend` on a test driver for the actual
adapter and call `assert_backend_contract::<YourDriver>()` from a native test.
The complete entrypoint always runs all four named scenarios:
`assert_admission_retry`, `assert_logout_reopen`, `assert_replacement_generation`
and `assert_capacity`. Each uses fresh disposable backing and can also be called
from a separately named native test for clearer failure diagnosis. A subset is
not complete backend conformance.
`fresh` supplies isolated backing with the requested generation and bounds;
`fail_next_commit` injects a normal error before mutations; `reopen` reconstructs
the adapter from persisted representation; `advance_generation` persists the new
protected generation; `retained_logins` returns the complete checkpointed ledger.
Do not reset or erase production backing to run these scenarios.

Check `cargo tree -i ic-auth` and the protocol/Platform identities: the harness and
adapter must use the same selected packages. Registry identity is essential to
implementing the same Rust traits; matching version text alone is insufficient.

Scenarios cover admission/replacement, failed commits, spent verification credit,
exact retry after proof expiry, logout, replay/denial retention, reopening,
generation invalidation and physical session/replay count/byte capacity. Replay
capacity covers receiver-wide and per-caller exhaustion, preserves original exact
retry at saturation, and checks that per-caller saturation leaves other callers
admissible. Deliberately broken admission and logout writes prove that returned
errors cannot publish mutations. These checks do not establish
crash atomicity, rollback protection, durable clocks/authority, cleanup bounds or
installed endpoint acceptance. The host owns those additional proofs.
