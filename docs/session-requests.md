# Canonical session requests

Released **0.5.0** exposes `ic_auth::session_request::{SessionRequest, SessionRequestError}`
without optional features, including `default-features = false`.
[Auth #32](https://github.com/dragginzgame/ic-auth/issues/32) owns this extraction;
[Canic #491](https://github.com/dragginzgame/canic/issues/491) owns consumer adoption.

`SessionRequest::new(scopes, ttl_secs)` takes ownership of the request and accepts
1–16 canonical scope labels, at most 64 bytes each and 1024 bytes together. It
sorts labels without rewriting them, rejects duplicates and rejects `Some(0)`.
Count/bounds are checked before label grammar; label failures precede duplicate
and zero-TTL failures. Shape failures return `InvalidRequest`; grammar failures
retain `Canonical(CanonicalAuthError)`. Accessors borrow the sorted `scopes()`
and return the exact optional `ttl_secs()`.

`hash()` preserves SHA-256 over this exact framing:

```text
canic-application-session-request-v1
|| scope_count:u64be
|| (scope_byte_length:u64be || scope_bytes)*
|| (0:u8 for None | 1:u8 || ttl_seconds:u64be for Some)
```

An omitted TTL differs from an explicit default TTL. Any positive u64 TTL is
representable; protected host defaults and lease ceilings are separate admission
decisions. The frozen hash for sorted `read`, `write` with `Some(30)` remains
`d99fc919563d8dba6db966bacbb256f2ba2437e64d12353106409934cc678551`.
Request identity does not authenticate a token, enroll a key, read a clock,
consume replay state or grant scopes/resources. Hosts bound outer decoding before
allocating submitted vectors.

## Consumer cut

The former `ic_auth::session::SessionRequest` import is removed in 0.5.0.
Import from `session_request` and handle `SessionRequestError` at construction;
the old constructor returned the broader `SessionError`. There is no alias.
The optional [session engine](sessions.md) accepts this same request owner and
retains its existing admission, policy, transaction and stored-record contracts.
Request hashes, signed bytes and stored session encoding do not change.

Canic can replace its local establishment-request hash with
`SessionRequest::new(...).hash()` in its always-compiled endpoint adapter without
enabling cryptographic session features. Keep its existing typed scope/TTL
prechecks and diagnostic order before projection. Protected generation, request
authentication, operation identity, replay handling and committed lease policy
remain Canic-owned; this extraction does not qualify direct session-engine adoption.

## Qualification

`make test-protocol` exercises frozen hashes, absent/explicit TTL, scope order,
exact bounds, duplicate/grammar refusals and detached input. `make test-sessions`
retains the real proof, retry, authority and transaction controls. The isolated
normalized-package consumer in `make check-msrv` calls the public request API on
native and Wasm with default features disabled. Its normal/build graphs and
`make check-boundaries` reject K256, certificate verification, CDK and stable
storage in that pure selection. Native Host and Testkit remain outside the libraries.
