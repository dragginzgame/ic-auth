import { test } from 'node:test';
import assert from 'node:assert/strict';
import * as client from '@dragginzgame/ic-auth-client';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';

// Compile the consumer boundary, including handles reached through operation options.
function consumerSurface(journal: client.LoginJournal, options: client.CredentialRequestOptions) {
  void journal.start; void journal.inspect; void journal.beginLogout; void journal.close;
  // @ts-expect-error callers cannot reserve internal operation stages
  void journal.claimCredential;
  // @ts-expect-error completion belongs to the credential operation
  void options.journal.completeCredential;
  // @ts-expect-error cleanup tickets are internal
  void journal.claimCleanup;
  // @ts-expect-error callers cannot settle cleanup independently
  void journal.completeCleanup;
  // @ts-expect-error signing guards are internal
  void journal.assertActive;
  // @ts-expect-error the package exports operations, not the signing primitive
  void client.runSigningAttempt;
  // @ts-expect-error construction must use open()
  new client.LoginJournal();
}
void consumerSurface;

test('package entrypoint keeps signing and journal stage machinery internal', () => {
  for (const name of ['Journal', 'journalOwner', 'runSigningAttempt'])
    assert.equal(Object.hasOwn(client, name), false);
  assert.deepEqual(Object.keys(client.LoginJournal), ['open']);
});

test('an imitation lifecycle handle cannot drive a cleanup effect', async () => {
  const journal: client.LoginJournal = {
    async start() {}, async beginLogout() { throw new Error('must not call'); },
    async inspect() { throw new Error('must not call'); }, close() {},
  };
  let calls = 0;
  await assert.rejects(client.cleanupOnce({ journal, generation: 'login',
    browser: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
    revoke: async () => { calls++; throw new Error('must not dispatch'); },
  }), (error: unknown) => error instanceof client.JournalError && error.code === 'configuration');
  assert.equal(calls, 0);
});
