# Retired Memory 0.63 manager header

`manager-header.hex` contains the first 64 raw bytes produced by published
`ic-memory` 0.63.0 and `ic-storage` 0.63.0 over a fresh native backing. The probe
bootstraps `ic_auth_qualification.checkpoint.v1`, opens its checked lease and
creates a cell containing `AUTHNAT3retained checkpoint`.

This is a frozen manager-header rejection vector, not a complete backing image.
The native refusal test installs this prefix and a separate application-data
sentinel, then verifies that Storage 0.64 rejects both restore and fresh admission
without changing any bytes. No legacy decoder or reset is supplied. The earlier
0.55 ledger fixture is superseded because the entire manager format is retired.

The published package checksums are:

- Memory: `cf61c6528cdbde5bfae7c7b17573ceb992d85e52c55597f5b41c0eafa17be602`
- Storage: `3fca9d8f5962146321a269ae2c7fd2aa137d350c2d4a2c37df1c75a9517224fe`

Generation source and compiler output are retained in
`target/storage-owner-adoption/legacy/` and `legacy-build.log` for this adoption.
