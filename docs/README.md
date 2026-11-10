# Auth documentation

The [current handoff](status/current.md) records implementation, qualification and
publication status. The [repository README](../README.md) introduces the packages.

| Topic | Start here |
| --- | --- |
| Protocol and client contracts | [Sessions](protocol/sessions.md), [session requests](protocol/session-requests.md), [browser client](protocol/browser-client.md), [signatures](protocol/signatures.md) |
| Development and delivery | [Setup and commands](development/setup.md), [compiler qualification](development/msrv.md), [release validation](development/release-validation.md), [canister qualification](development/testkit-qualification.md) |
| Consumer integration | [Canic adoption](integration/canic-adoption.md), [host entropy](integration/host-entropy.md) |
| Design and ownership | [Extraction boundary](design/extraction.md), [portable protocol](design/0.6-auth-protocol.md), [device capacity](design/account-device-capacity-audit.md) |
| Performance evidence | [Wasm attribution](performance/wasm-attribution.md), [canonical allocations](performance/canonical-allocation-observations.md), [retry allocations](performance/retry-allocation-observations.md), [signature preparation](performance/signature-preparation.md) |

Shared engineering references stay upstream at the revision-pinned links in
[Toko Tooling](../TOKO-TOOLING.md).
