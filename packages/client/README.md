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
issuer operations; the browser identity admits and calls receivers. Consumers
provide `issue` and lookup-only `reconcile`, protected issuer/root configuration,
application-owned IndexedDB, clocks and key custody. The HTTP transports supply
receiver admission/logout, not issuer endpoints. Preserve exact issuance intent,
original deadlines and unknown effects. Authentication grants no resource ownership.

From an explicitly prepared, source-pinned IC Auth checkout:

```sh
make pack-client
make test-client-package
```

Each command retains a fresh directory under `target/browser-client-packages/`.
`delivery.json` records the Git revision, dirty-source observation, exact input
manifest, compiler/tool selections, archive integrity and qualification performed.
It is local build evidence, not a signature or production acceptance. A dirty
candidate must be rebuilt and qualified from the intended delivered commit.

Consumers can install the selected archive using npm's `file:` dependency and
commit their resulting package lock, retaining its integrity plus the producer
receipt. Keep the archive at a durable consumer-owned path or artifact location;
an ignored producer `target` path is not a distribution service. Do not depend on
a moving sibling checkout or use a repository-root Git npm dependency: this
package lives in a subdirectory and requires explicit generation/build admission.

The current consumer proof uses the producer-selected Node/npm and TypeScript,
strict declarations and a browser bundle, with injected issuer/receiver calls
and fake IndexedDB. It does not establish an older compiler floor, real browser
storage, Canic endpoint integration, production issuer behavior or TEE custody.
The full maintained contract is in `docs/browser-client.md` in the source
repository; Canic production acceptance remains #491/#507 there.
