# IC Auth browser client

Private ESM package for portable session admission and durable logout. It is
not published on npm. Local archives are built from IC Auth's reviewed source;
the package's `0.0.0` version is not a release identity. Use the archive integrity
and the producer's source receipt to select an exact build.

Import the public package root:

```ts
import {
  PortableAdmissionClient,
  PortableLogoutClient,
  IndexedDbLogoutStore,
  httpAdmissionTransport,
  httpLogoutTransport,
} from '@dragginzgame/ic-auth-client';
```

The package contains compiled JavaScript, TypeScript declarations and the
Rust-owned generated Wire codecs. Its SDK boundary is
`@tokostudios/ic-platform`. Declare Platform directly if your application imports
its identities, Principals or Agent APIs. Internal paths are not exported.

Use separate account and fresh browser identities. The account authenticates
initial issuance; the retained browser identity renews the original login, admits
and calls receivers. Account identity may be omitted after initial completion. Consumers
provide `issue`, browser-bound `renew` and sequence-specific lookup-only `reconcile`, protected issuer/root configuration,
application-owned IndexedDB, clocks and key custody. The HTTP transports supply
receiver admission/logout, not issuer endpoints. Preserve exact issuance intent,
original deadlines and unknown effects. Authentication grants no resource ownership.

From an explicitly prepared, source-pinned IC Auth checkout:

```sh
make test-client-package
```

This builds and qualifies the same archive in a fresh directory under
`target/browser-client-packages/`. Use `make pack-client` only when an unqualified
archive is sufficient; running it first is unnecessary.
`delivery.json` records the Git revision, dirty-source observation, exact input
manifest, compiler/tool selections, archive integrity and qualification performed.
It is local build evidence, not a signature or production acceptance. A dirty
candidate must be rebuilt and qualified from the intended delivered commit.

To run the installed archive's public API in real Chromium, select an already
prepared executable explicitly:

```sh
IC_AUTH_BROWSER_EXECUTABLE=/absolute/path/to/chromium make test-client-package-browser
```

This includes ordinary archive qualification and checks real IndexedDB admission,
logout invalidation and journal reopen before cleanup. The receipt is marked
`packed-consumer-chromium`; `browser-proof.json` records browser version and
executable/bundle hashes. No browser is downloaded and no personal profile is used.

Consumers can install the selected archive using npm's `file:` dependency and
commit their resulting package lock, retaining its integrity plus the producer
receipt. Keep the archive at a durable consumer-owned path or artifact location;
an ignored producer `target` path is not a distribution service. Do not depend on
a moving sibling checkout or use a repository-root Git npm dependency: this
package lives in a subdirectory and requires explicit generation/build admission.

The ordinary consumer proof uses the producer-selected Node/npm and TypeScript,
strict declarations and a browser bundle, with injected issuer/receiver calls
and fake IndexedDB. The opt-in Chromium proof adds real browser storage; neither
establishes an older compiler floor, Canic endpoint integration, production
issuer behavior or TEE custody.
The full maintained contract is in `docs/protocol/browser-client.md` in the source
repository; Canic production acceptance remains #491/#507 there.

The remembered-login journal retains one bounded exact credential separately from
its small guard metadata. Completed proof expiry permits a new renewal sequence;
lost replies require exact lookup. The old portable-journal profile is retired
with maintainer confirmation of no retained records; unknown profiles are refused,
never reset. Auth supports protected login windows up to 90 days, with short
proofs and receiver leases. Hosts own persistent key custody and issuer bindings.
`sessionReceipt(receiver)` returns detached scheduling metadata in receiver time;
receiver authorization remains authoritative. No background timer or application
mutation retry is installed. Known-login logout is confirmed only after committed
denial; unseen-login denial stays explicitly unconfirmed. Local cleanup timeouts
do not prove remote expiry. Canic production adoption remains consumer-owned.

Use `ensureSession(receiver, refreshTime)` to obtain a receiver-bound SDK identity
that prepares a lease before each new call/query, including after browser resume.
The host supplies a conservative receiver-expiry-to-journal-clock mapping; Auth
does not infer one. Live lease hints skip proof work and network admission. Short
leases refresh the completed credential sequence even before proof expiry; pending
sequences remain lookup-only. After reopen, an authenticated expiry refusal allows
one fresh-proof admission retry. Polling already submitted requests never renews,
and application operations are never replayed. `establish` remains explicit forced
admission. The remembered device key and actual issuer binding remain host-owned.
