import assert from 'node:assert/strict';

export async function installedFlowCases({ check, a }) {
  const env = process.env;
  const bytes = name => {
    const value = env[name];
    assert(value && /^(?:[0-9a-f]{2})+$/.test(value), `Missing prepared ${name}`);
    return Array.from(Buffer.from(value, 'hex'));
  };
  await a.evaluate(async config => {
    const { PublicJournal, SessionCoordinator, Ed25519KeyIdentity, Principal, httpUpdateTransport } = authTest;
    const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
    const account = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(8));
    const target = Principal.fromText(config.canister);
    const nowNs = () => BigInt(Date.now()) * 1_000_000n;
    const journal = await PublicJournal.open(indexedDB, 'installed-flow', { id: 'fixture-unix', nowNs });
    await journal.start({ generation: 'journey', caller: browser.getPrincipal(), issuer: target });
    const transport = methodName => httpUpdateTransport({ host: new URL(config.gateway), rootKey: new Uint8Array(config.root), methodName });
    const credential = transport('browser_fixture_credential');
    const admit = transport('establish_portable_session');
    const write = transport('browser_fixture_write');
    const revoke = transport('logout_portable_session');
    // Conservative fixed-fixture mapping, captured BEFORE initial issuance. Never reset on lookup.
    // The host issues a 60s proof; this entire test uses only the first 30s from this earlier instant.
    const untilNs = nowNs() + 30_000_000_000n;
    const counts = { issue: 0, lookup: 0, admit: 0, logout: 0 };
    const options = { journal, generation: 'journey', browser, account, issuer: target, target, scopes: ['write'],
      transport: {
        issue: async (identity, signal) => {
          counts.issue++;
          await credential(target, identity, new Uint8Array(config.issue), signal);
          // The host committed a real signature, but its consumer lost the authenticated reply.
          throw flow.lostReply;
        },
        reconcile: async (identity, sequence, signal) => {
          if (sequence !== 0n) throw new Error('unexpected sequence');
          counts.lookup++;
          return { bytes: await credential(target, identity, new Uint8Array(config.lookup), signal), untilNs };
        },
        renew: async () => { throw new Error('no renewal belongs in this fixed journey'); },
      },
      admit: async (destination, identity, request, signal) => {
        counts.admit++;
        flow.admissionRequest = new Uint8Array(request);
        return admit(destination, identity, request, signal);
      },
    };
    const coordinator = new SessionCoordinator({ ...options,
      // This fixture explicitly shares Unix nanoseconds with its live host; retain a one-second margin.
      refreshAt: receipt => receipt.expires_at_ns - 1_000_000_000n });
    globalThis.flow = { coordinator, lostReply: new Error('lost issuer reply'), options, counts, target, browser, account, nowNs, journal,
      // Deliberate raw caller probes test server authorization independently of local journal guards.
      write: (identity, signal = AbortSignal.timeout(5_000)) => write(target, identity, new Uint8Array(config.write), signal),
      replay: () => admit(target, browser, flow.admissionRequest, AbortSignal.timeout(5_000)),
      revoke: async (...args) => { counts.logout++; return revoke(...args); },
      lookup: () => credential(target, account, new Uint8Array(config.lookup), AbortSignal.timeout(5_000)),
    };
  }, { gateway: env.AUTH_HTTP_GATEWAY, canister: env.AUTH_HTTP_CANISTER, root: bytes('AUTH_HTTP_ROOT'),
    issue: bytes('AUTH_FLOW_ISSUE'), lookup: bytes('AUTH_FLOW_LOOKUP'), write: bytes('AUTH_FLOW_WRITE') });

  await check('installed lookup cannot create issuance and an unauthenticated session cannot write', async () => {
    assert.equal(await a.evaluate(async () => {
      try { await flow.lookup(); return false; } catch { return true; }
    }), true);
    assert.deepEqual(await a.evaluate(async () => Array.from(await flow.write(flow.browser))), bytes('AUTH_FLOW_WRITE_DENIED'));
  });
  await check('lost installed issuer reply reconciles the same operation before real Wire admission', async () => {
    const result = await a.evaluate(async () => {
      const { retrieveCredential, establishSession } = authTest;
      let lost;
      try { await retrieveCredential(flow.options); } catch (error) {
        if (error !== flow.lostReply) throw error;
        lost = true;
      }
      const pending = await flow.journal.inspect();
      let wrongScope;
      try { await establishSession({ ...flow.options, scopes: ['read'] }); } catch (error) { wrongScope = error.refusal; }
      flow.receipt = await flow.coordinator.ensureSession(flow.target);
      const saved = await flow.journal.inspect();
      return { lost, sequence: String(pending.sequence), unknown: pending.issuanceUnknown, wrongScope,
        caller: authTest.Principal.fromUint8Array(flow.receipt.caller).toText(),
        expectedCaller: flow.browser.getPrincipal().toText(),
        subject: authTest.Principal.fromUint8Array(flow.receipt.subject).toText(),
        expectedSubject: flow.account.getPrincipal().toText(), phase: saved.phase, counts: flow.counts };
    });
    assert.equal(result.lost, true);
    assert.equal(result.sequence, '0'); assert.equal(result.unknown, true);
    assert.equal(result.wrongScope, 'ScopeRejected');
    assert.equal(result.caller, result.expectedCaller); assert.equal(result.subject, result.expectedSubject);
    assert.equal(result.phase, 'active');
    assert.deepEqual(result.counts, { issue: 1, lookup: 1, admit: 2, logout: 0 });
  });
  await check('only the admitted browser commits an application write; exact admission retry retains its lease', async () => {
    assert.deepEqual(await a.evaluate(async () => Array.from(await flow.write(flow.account))), bytes('AUTH_FLOW_WRITE_DENIED'));
    assert.deepEqual(await a.evaluate(async () => Array.from(await flow.coordinator.call(flow.target, flow.write))), bytes('AUTH_FLOW_WRITE_OK'));
    assert.equal(await a.evaluate(async () => {
      const before = flow.counts.admit;
      await flow.coordinator.ensureSession(flow.target);
      if (flow.counts.admit !== before) throw new Error('live readiness repeated admission');
      const retried = await flow.coordinator.establish(flow.target);
      return retried.expires_at_ns === flow.receipt.expires_at_ns && flow.counts.issue === 1 && flow.counts.lookup === 1;
    }), true);
  });
  await check('durable local logout and certified remote cleanup fence both new calls and captured credential replay', async () => {
    const result = await a.evaluate(async () => {
      await flow.journal.beginLogout('journey', flow.nowNs() + 60_000_000_000n);
      const before = flow.counts.admit;
      let refused;
      try { await flow.coordinator.ensureSession(flow.target); } catch (error) { refused = error.code; }
      const cleanup = { journal: flow.journal, generation: 'journey', browser: flow.browser, revoke: flow.revoke };
      const finished = await authTest.cleanupUntilFinished(cleanup);
      return { refused, noAdmission: before === flow.counts.admit,
        finished: finished.phase, confirmations: finished.destinations.map(t => t.confirmedUntilNs !== undefined), counts: flow.counts };
    });
    assert.equal(result.refused, 'not_current'); assert.equal(result.noAdmission, true);
    assert.equal(result.finished, 'finished'); assert.deepEqual(result.confirmations, [true]);
    assert.equal(result.counts.logout, 1);
    assert.deepEqual(await a.evaluate(async () => Array.from(await flow.write(flow.browser))), bytes('AUTH_FLOW_WRITE_DENIED'));
    assert.deepEqual(await a.evaluate(async () => Array.from(await flow.replay())), bytes('AUTH_FLOW_REVOKED'));
    await a.evaluate(() => flow.journal.close());
  });
}
