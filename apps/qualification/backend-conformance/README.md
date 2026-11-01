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

After the matching Auth libraries are published, `make pack-backend-conformance`
creates `target/package/ic-auth-backend-conformance-VERSION.crate` without an
upload. Cargo includes the source, tests, README, license, lock and VCS metadata,
and normalizes workspace/path dependencies to registry dependencies. Retain the
archive and its integrity with the consumer's qualification inputs. A dirty VCS
receipt describes a candidate, not a delivered revision.

Extract this source archive into the consumer's test-support directory and add
it as a **development dependency**. Use the archive's normalized `Cargo.toml`;
depending on the producer's working workspace can introduce a second Git/path
identity for Auth alongside the consumer's registry dependency.

For the 0.28 cohort:

```toml
# Append this path to the consumer's existing workspace exclusions. The foreign
# source bundle must not become a consumer-owned release-group member.
[workspace]
exclude = ["test-support/ic-auth-backend-conformance-0.28.0"]

[dev-dependencies]
ic-auth-backend-conformance = { path = "test-support/ic-auth-backend-conformance-0.28.0" }
ic-auth = { version = "=0.28.0", default-features = false, features = ["session-restore", "portable-signature-verification"] }
```

Implement `ic_auth_backend_conformance::Backend` on a test driver for the actual
adapter and call `assert_backend_contract::<YourDriver>()` from a native test.
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
generation invalidation and physical count/byte capacity. They do not establish
crash atomicity, rollback protection, durable clocks/authority, cleanup bounds or
installed endpoint acceptance. The host owns those additional proofs.
