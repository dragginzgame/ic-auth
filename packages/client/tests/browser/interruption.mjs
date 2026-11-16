import assert from 'node:assert/strict';

export async function interruptionCases({ check, browser, prepare }) {
  for (const boundary of ['issue-pending', 'renewal-pending', 'completed', 'logout-after-crash']) {
    await check(`renderer loss at ${boundary} preserves issuance identity without reissuing`, async () => {
      const ownedBrowser = await browser.browserType().launch({
        executablePath: process.env.IC_AUTH_BROWSER_EXECUTABLE, headless: true });
      const context = await ownedBrowser.newContext();
      try {
        const page = await context.newPage(); await prepare(page);
        const before = await page.evaluate(async boundary => {
          const journal = await authTest.PublicJournal.open(indexedDB, 'renderer-issuance', { id: 'clock', nowNs: () => now });
          await journal.start({ generation: 'login', caller, issuer });
          const browser = authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
          const account = authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
          const calls = [];
          const signed = async (kind, identity, sequence) => {
            await authTest.signCall(identity, issuer);
            calls.push([kind, sequence, identity.getPrincipal().toText()]);
          };
          const options = { journal, generation: 'login', browser, account, issuer,
            transport: {
              issue: async identity => { await signed('issue', identity, 0n); return { bytes: frames.ed25519, untilNs: 10_000n }; },
              renew: async () => { throw new Error('unexpected renewal'); },
              reconcile: async () => { throw new Error('unexpected lookup'); },
            } };
          if (boundary === 'renewal-pending' || boundary === 'completed') await authTest.retrieveCredential(options);
          if (boundary !== 'completed') {
            let entered; const started = new Promise(resolve => { entered = resolve; });
            const pending = async (kind, identity, sequence) => {
              await signed(kind, identity, sequence); entered(); await new Promise(() => {});
            };
            options.transport.issue = identity => pending('issue', identity, 0n);
            options.transport.renew = (identity, sequence) => pending('renew', identity, sequence);
            globalThis.interruptedIssue = authTest.retrieveCredential({ ...options,
              ...(boundary === 'renewal-pending' ? { refreshThrough: 0n } : {}) })
              .then(() => 'unexpected completion', error => error.code ?? error.name);
            await Promise.race([started, interruptedIssue.then(result => { throw new Error(String(result)); })]);
          }
          return { status: await journal.inspect(), calls, account: account.getPrincipal().toText(), browser: caller.toText() };
        }, boundary);
        const sequence = boundary === 'renewal-pending' ? 1n : 0n;
        assert.equal(before.status.sequence, sequence);
        assert.deepEqual(before.calls, boundary === 'renewal-pending'
          ? [['issue', 0n, before.account], ['renew', 1n, before.browser]] : [['issue', 0n, before.account]]);
        const reopened = await crashAndReopen(ownedBrowser, context, page, prepare);
        const result = await reopened.evaluate(async ({ boundary, sequence }) => {
          const journal = await authTest.PublicJournal.open(indexedDB, 'renderer-issuance', { id: 'clock', nowNs: () => now });
          const retained = await journal.inspect(); const calls = [];
          const browser = authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
          const account = authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
          let failLookup = true;
          const missing = new Error('issuer evidence unavailable');
          const options = { journal, generation: 'login', browser, issuer,
            ...(boundary === 'issue-pending' || boundary === 'logout-after-crash' ? { account } : {}),
            transport: {
              issue: async () => { calls.push(['issue']); throw new Error('must never reissue'); },
              renew: async () => { calls.push(['renew']); throw new Error('must never renew pending work'); },
              reconcile: async (identity, seq) => {
                await authTest.signCall(identity, issuer);
                calls.push(['lookup', seq, identity.getPrincipal().toText()]);
                if (failLookup) throw missing;
                return { bytes: frames.ed25519, untilNs: 10_000n };
              },
            } };
          let noAccount, failed, afterFailure, evidence, denied;
          if (boundary === 'logout-after-crash') {
            await journal.beginLogout('login', 100_000_000_000n);
            denied = await failure(authTest.retrieveCredential(options));
          } else {
            if (boundary === 'issue-pending') {
              const { account: omitted, ...withoutAccount } = options;
              noAccount = await failure(authTest.retrieveCredential(withoutAccount));
            }
            if (boundary !== 'completed') {
              // A stronger refresh request must not skip unknown pending work.
              options.refreshThrough = 999n;
              failed = await authTest.retrieveCredential(options).then(() => false, error => error === missing);
              afterFailure = await journal.inspect(); failLookup = false;
            }
            evidence = await authTest.retrieveCredential(options);
            if (evidence.bytes.length !== frames.ed25519.length || !evidence.bytes.every((byte, i) => byte === frames.ed25519[i]))
              throw new Error('retained/reconciled evidence changed');
          }
          const final = await journal.inspect(); journal.close();
          return { retained, calls, noAccount, failed, afterFailure, sequence: evidence?.sequence,
            untilNs: evidence?.untilNs, denied, final };
        }, { boundary, sequence });
        assert.deepEqual(result.retained, before.status);
        assert.equal(result.final.issuanceUnknown, true);
        assert.equal(result.final.sequence, sequence);
        if (boundary === 'logout-after-crash') {
          assert.equal(result.denied, 'not_current'); assert.equal(result.final.phase, 'cleanup');
          assert.deepEqual(result.calls, []);
        } else {
          assert.equal(result.sequence, sequence); assert.equal(result.untilNs, 10_000n);
          if (boundary === 'completed') assert.deepEqual(result.calls, []);
          else {
            assert.equal(result.failed, true); assert.deepEqual(result.afterFailure, before.status);
            if (boundary === 'issue-pending') assert.equal(result.noAccount, 'account_required');
            const signer = boundary === 'renewal-pending' ? before.browser : before.account;
            assert.deepEqual(result.calls, [['lookup', sequence, signer], ['lookup', sequence, signer]]);
          }
        }
      } finally { await ownedBrowser.close(); }
    });
  }

  await check('renderer loss after host key deletion retries disposal without a private signer', async () => {
    const ownedBrowser = await browser.browserType().launch({
      executablePath: process.env.IC_AUTH_BROWSER_EXECUTABLE, headless: true });
    const context = await ownedBrowser.newContext();
    let binding, deletions = 0, invocations = 0;
    // Host custody survives the renderer; this is a conditional-delete test double.
    await context.exposeFunction('deleteHostKey', requested => {
      invocations++;
      if (binding && requested.generation === binding.generation && requested.caller === binding.caller) {
        binding = undefined; deletions++;
      }
    });
    try {
      const page = await context.newPage(); await prepare(page);
      binding = await page.evaluate(() => ({ generation: 'login', caller: caller.toText() }));
      const before = await page.evaluate(async () => {
        await seed('renderer-disposal', { ...record(), issuanceUnknown: true, issuance: { sequence: 7n } });
        const journal = await authTest.PublicJournal.open(indexedDB, 'renderer-disposal', { id: 'clock', nowNs: () => now });
        await journal.beginLogout('login', 100_000_000_000n);
        await authTest.cleanupOnce({ journal, generation: 'login',
          browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
          revoke: async () => frames['logout-Revoked-max'] });
        let entered; const started = new Promise(resolve => { entered = resolve; });
        globalThis.interruptedDisposal = authTest.disposeLoginKey({ journal, generation: 'login', caller,
          dispose: async binding => { await deleteHostKey(binding); entered(); await new Promise(() => {}); }
        }).then(() => 'unexpected completion', error => error.code ?? error.name);
        await Promise.race([started, interruptedDisposal.then(result => { throw new Error(String(result)); })]);
        return journal.inspect();
      });
      assert.equal(deletions, 1); assert.equal(binding, undefined);
      const reopened = await crashAndReopen(ownedBrowser, context, page, prepare);
      const after = await reopened.evaluate(async () => {
        const journal = await authTest.PublicJournal.open(indexedDB, 'renderer-disposal', { id: 'clock', nowNs: () => now });
        const result = await authTest.disposeLoginKey({ journal, generation: 'login', caller, dispose: deleteHostKey });
        const retained = await journal.inspect(); journal.close(); return { result, retained };
      });
      assert.equal(invocations, 2); assert.equal(deletions, 1);
      assert.deepEqual(after.result, before); assert.deepEqual(after.retained, before);
      assert.equal(before.issuanceUnknown, true);
    } finally { await ownedBrowser.close(); }
  });

  for (const boundary of ['local-logout', 'submitted', 'confirmed']) {
    await check(`renderer crash after ${boundary} preserves logout progress on reopening`, async () => {
      const ownedBrowser = await browser.browserType().launch({
        executablePath: process.env.IC_AUTH_BROWSER_EXECUTABLE, headless: true });
      const context = await ownedBrowser.newContext();
      try {
        const page = await context.newPage(); await prepare(page);
        const before = await page.evaluate(async boundary => {
          await seed('renderer-logout', { ...record(), issuanceUnknown: true, issuance: { sequence: 7n } });
          const journal = await authTest.PublicJournal.open(indexedDB, 'renderer-logout', { id: 'clock', nowNs: () => now });
          await journal.beginLogout('login', 100_000_000_000n);
          const options = { journal, generation: 'login',
            browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)) };
          let calls = 0;
          if (boundary === 'submitted') {
            let entered; const started = new Promise(resolve => { entered = resolve; });
            // Leave a genuinely live signed attempt suspended until the renderer dies.
            globalThis.interruptedCleanup = authTest.cleanupUntilFinished({ ...options,
              revoke: async (target, identity, bytes) => {
                calls++; await authTest.signCall(identity, target, bytes); entered(); await new Promise(() => {});
              } }).then(() => 'unexpected completion', error => error.code ?? error.name);
            await Promise.race([started, interruptedCleanup.then(result => { throw new Error(String(result)); })]);
          } else if (boundary === 'confirmed') {
            await authTest.cleanupOnce({ ...options, revoke: async (target, identity, bytes) => {
              calls++; await authTest.signCall(identity, target, bytes); return frames['logout-Revoked-max'];
            } });
          }
          const status = await journal.inspect();
          return { status, calls };
        }, boundary);
        assert.equal(before.status.phase, boundary === 'confirmed' ? 'finished' : 'cleanup');
        assert.equal(before.calls, boundary === 'local-logout' ? 0 : 1);
        const reopened = await crashAndReopen(ownedBrowser, context, page, prepare);
        const result = await reopened.evaluate(async boundary => {
          const journal = await authTest.PublicJournal.open(indexedDB, 'renderer-logout', { id: 'clock', nowNs: () => now });
          const retained = await journal.inspect(); let calls = 0;
          const options = { journal, generation: 'login',
            browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
            revoke: async (target, identity, bytes) => {
              calls++; await authTest.signCall(identity, target, bytes); return frames['logout-Revoked-max'];
            } };
          // Logout cannot be reversed by starting the same or a replacement login.
          const restart = await failure(journal.start({ generation: 'replacement', caller, issuer }));
          let waiting;
          if (boundary === 'submitted') {
            const step = await authTest.cleanupOnce(options);
            waiting = [step.kind, step.untilNs, calls];
            now = 5_000_000_100n;
          }
          const finished = await authTest.cleanupUntilFinished(options);
          journal.close();
          return { retained, restart, waiting, finished, calls };
        }, boundary);
        assert.deepEqual(result.retained, before.status);
        assert.equal(result.restart, 'login_exists');
        assert.equal(result.finished.phase, 'finished');
        assert.equal(result.finished.deadlineNs, 100_000_000_000n);
        assert.equal(result.finished.issuanceUnknown, true);
        assert.equal(result.finished.sequence, 7n);
        assert.equal(result.finished.destinations[0].confirmedUntilNs, 0xffff_ffff_ffff_ffffn);
        assert.equal(result.calls, boundary === 'confirmed' ? 0 : 1);
        assert.equal(result.finished.destinations[0].attempts, boundary === 'submitted' ? 2 : 1);
        if (boundary === 'submitted') assert.deepEqual(result.waiting, ['wait', 5_000_000_100n, 0]);
      } finally { await ownedBrowser.close(); }
    });
  }
}

export async function crashAndReopen(ownedBrowser, context, page, prepare) {
  const cdp = await ownedBrowser.newBrowserCDPSession();
  const { processInfo } = await cdp.send('SystemInfo.getProcessInfo');
  const renderers = processInfo.filter(p => p.type === 'renderer');
  assert(renderers.length > 0, 'dedicated browser must expose its renderer processes');
  const crashed = page.waitForEvent('crash', { timeout: 10_000 });
  // Kill only renderers reported by this dedicated test browser. SIGKILL
  // models abrupt process loss without invoking host core-dump handlers.
  for (const renderer of renderers) {
    assert(Number.isSafeInteger(renderer.id) && renderer.id > 0);
    try { process.kill(renderer.id, 'SIGKILL'); } catch (error) {
      if (error.code !== 'ESRCH') throw error;
    }
  }
  await crashed;
  await page.close();
  const reopened = await context.newPage(); await prepare(reopened);
  return reopened;
}
