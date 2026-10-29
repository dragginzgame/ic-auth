# Retained Memory 0.55 format refusal

`ledger_commit_store_single.cbor.hex` is the opaque current-ledger fixture from
the published `ic-memory` **0.55.2** archive, path
`fixtures/current/ledger_commit_store_single.cbor.hex`. Its archive SHA-256 is
`dafd28d909c49f6c2bf9e9bd048c3bf7cea6a3378c4a6ce3d70f11cd0f603cb1`.

It is a valid older Cell value with the `ICMKEY01` protected marker. The Auth
qualification test installs it in isolated native backing and checks that the
new runtime refuses it without changing any backing bytes. The test includes
an application Cell to guard against clearing only the allocation ledger.
It supplies no legacy decoder, migration or permission to discard retained data.
