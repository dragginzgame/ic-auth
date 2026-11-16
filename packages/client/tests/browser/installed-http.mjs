import assert from 'node:assert/strict';

export async function installedHttpCases({ check, a }) {
  const environment = process.env;
  const bytes = name => {
    const value = environment[name];
    assert(value && /^(?:[0-9a-f]{2})+$/.test(value), `Missing prepared ${name}`);
    return Array.from(Buffer.from(value, 'hex'));
  };
  assert(environment.AUTH_HTTP_GATEWAY && environment.AUTH_HTTP_CANISTER, 'Run make qualify-client-http');
  await a.evaluate(config => {
    const { Ed25519KeyIdentity, Principal, httpUpdateTransport, runSigningAttempt } = authTest;
    const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
    const canister = Principal.fromText(config.canister);
    globalThis.httpEvidence = undefined;
    globalThis.installedCall = async (mode, other = false) => {
      let calls = 0, polls = 0;
      const rootKey = mode === 'root'
        ? Uint8Array.from(authTest.IC_ROOT_KEY.match(/../g), b => parseInt(b, 16)) : new Uint8Array(config.root);
      const update = httpUpdateTransport({ host: new URL(config.gateway), rootKey,
        methodName: 'auth_fixture_control', fetch: async (input, init) => {
          const call = new URL(input.toString()).pathname.endsWith('/call');
          if (call) calls++; else polls++;
          const response = await fetch(input, init);
          if (call || !response.ok) return response;
          const body = new Uint8Array(await response.arrayBuffer());
          if (mode === 'replay') return new Response(httpEvidence.slice().buffer);
          if (mode === 'tamper') body[body.length - 1] ^= 1;
          if (mode === 'capture') httpEvidence = body.slice();
          return new Response(body.buffer);
        } });
      let outcome;
      try {
        const reply = await runSigningAttempt({ identity, target: canister, assertCurrent: async () => {},
          timeoutMs: mode === 'replay' ? 2_000 : 5_000 },
        (signer, signal) => update(canister, signer, new Uint8Array(other ? config.other : config.request), signal));
        outcome = { ok: true, reply: Array.from(reply) };
      } catch (error) { outcome = { ok: false, code: error.code ?? error.name }; }
      return { ...outcome, calls, polls };
    };
  }, { gateway: environment.AUTH_HTTP_GATEWAY, canister: environment.AUTH_HTTP_CANISTER,
    root: bytes('AUTH_HTTP_ROOT'), request: bytes('AUTH_HTTP_REQUEST'), other: bytes('AUTH_HTTP_OTHER_REQUEST') });

  await check('installed single-subnet update returns actual SDK-certified Auth fixture bytes', async () => {
    const result = await a.evaluate(() => installedCall('capture'));
    assert.equal(result.ok, true, JSON.stringify(result));
    assert.deepEqual(result.reply, bytes('AUTH_HTTP_EXPECTED'));
    assert.equal(result.calls, 1); assert(result.polls >= 1);
    assert(await a.evaluate(() => httpEvidence instanceof Uint8Array && httpEvidence.length > 0));
  });
  await check('a wrong protected root cannot authenticate the installed reply', async () => {
    const result = await a.evaluate(() => installedCall('root'));
    assert.equal(result.ok, false); assert.notEqual(result.code, 'timeout');
    assert.equal(result.calls, 1); assert(result.polls >= 1);
  });
  await check('tampering with the actual certificate response cannot return fixture bytes', async () => {
    const result = await a.evaluate(() => installedCall('tamper'));
    assert.equal(result.ok, false); assert.notEqual(result.code, 'timeout');
    assert.equal(result.calls, 1); assert(result.polls >= 1);
  });
  await check('a valid certificate for an earlier request cannot complete a different update', async () => {
    const result = await a.evaluate(() => installedCall('replay', true));
    assert.equal(result.ok, false); assert.equal(result.code, 'timeout');
    assert.equal(result.calls, 1); assert(result.polls >= 2);
  });
  await check('an untampered completion remains independently verifiable after the negative controls', async () => {
    const result = await a.evaluate(() => installedCall('plain', true));
    assert.equal(result.ok, true, JSON.stringify(result));
    assert.deepEqual(result.reply, bytes('AUTH_HTTP_EXPECTED'));
    assert.equal(result.calls, 1); assert(result.polls >= 1);
  });
}
