import assert from 'node:assert/strict';

export async function httpCases({ check, a }) {
  for (const operation of ['admission', 'logout']) {
    await check(`HTTP acceptance with an invalid certificate cannot complete browser ${operation}`, async () => {
      assert.deepEqual(await a.evaluate(async operation => {
        const { Ed25519KeyIdentity, Principal, httpUpdateTransport, IC_ROOT_KEY,
          inspectIngress, readStateResponseFixture, invalidCertificateFixture } = authTest;
        now = 100n; clockId = 'clock';
        const journal = await open(`http-${operation}`);
        await journal.start({ generation: 'login', caller, issuer });
        const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
        const requests = [];
        let submission;
        const transport = httpUpdateTransport({ host: new URL('https://gateway.invalid'),
          rootKey: Uint8Array.from(IC_ROOT_KEY.match(/../g), b => parseInt(b, 16)), methodName: 'auth_fixture',
          fetch: async (_url, init) => {
            const ingress = inspectIngress(init.body); requests.push(ingress.requestType);
            if (ingress.sender.toText() !== caller.toText() || ingress.senderSignature.length !== 64)
              throw new Error('wrong browser identity');
            if (ingress.requestType === 'call') {
              submission = ingress;
              return new Response(null, { status: 202 });
            }
            if (!ingress.paths[0][1].every((b, i) => b === submission.requestId[i]))
              throw new Error('poll is not bound to the submitted request');
            return new Response(new Uint8Array(readStateResponseFixture(invalidCertificateFixture())).buffer);
          } });
        let pending;
        if (operation === 'logout') {
          await journal.beginLogout('login', 100_000_000_000n);
          pending = authTest.cleanupOnce({ journal, generation: 'login', browser, revoke: transport });
        } else {
          await journal.claimCredential('login', { accountAvailable: true });
          await journal.completeCredential('login', 0n, frames.ed25519, 1_000_000_000n);
          pending = authTest.establishSession({ journal, generation: 'login', browser, issuer,
            target: Principal.fromUint8Array(Uint8Array.of(2)), scopes: ['read'], admit: transport,
            transport: { issue: async () => { throw new Error('unexpected issue'); },
              renew: async () => { throw new Error('unexpected renewal'); },
              reconcile: async () => { throw new Error('unexpected reconciliation'); } } });
        }
        const rejected = await pending.then(() => false, () => true);
        const snapshot = await journal.inspect(); journal.close();
        return [rejected, requests, snapshot.phase, snapshot.destinations.length,
          snapshot.destinations[0].attempts, snapshot.destinations.every(t => t.confirmedUntilNs === undefined)];
      }, operation), [true, ['call', 'read_state'], operation === 'logout' ? 'cleanup' : 'active',
        operation === 'logout' ? 1 : 2, operation === 'logout' ? 1 : 0, true]);
    });
  }
}
