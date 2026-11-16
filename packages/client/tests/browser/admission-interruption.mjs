import assert from 'node:assert/strict';
import { crashAndReopen } from './interruption.mjs';

export async function admissionInterruptionCases({ check, browser, prepare }) {
  for (const boundary of ['signing', 'submitted', 'confirmed', 'logout-after-crash']) {
    await check(`renderer loss during admission at ${boundary} preserves receiver inventory and exact evidence`, async () => {
      const ownedBrowser = await browser.browserType().launch({
        executablePath: process.env.IC_AUTH_BROWSER_EXECUTABLE, headless: true });
      const context = await ownedBrowser.newContext();
      // Observations survive the renderer. This models transport submission, not
      // receiver execution, certificate authentication or remote deduplication.
      const submitted = [];
      await context.exposeFunction('recordAdmission', (target, bytes) => { submitted.push({ target, bytes }); });
      try {
        const page = await context.newPage(); await prepare(page);
        const before = await page.evaluate(async boundary => {
          const { PublicJournal, Ed25519KeyIdentity, Principal, establishSession, signCall } = authTest;
          const journal = await PublicJournal.open(indexedDB, 'admission-crash', { id: 'clock', nowNs: () => now });
          await journal.start({ generation: 'login', caller, issuer });
          const key = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
          let entered; const started = new Promise(resolve => { entered = resolve; });
          const browser = {
            getPrincipal: () => key.getPrincipal(),
            transformRequest: async request => {
              if (boundary === 'signing') { entered(); await new Promise(() => {}); }
              return key.transformRequest(request);
            },
          };
          const target = Principal.fromUint8Array(Uint8Array.of(2));
          let issuance = 0, request;
          globalThis.interruptedAdmission = establishSession({ journal, generation: 'login', browser,
            account: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)), issuer, target,
            scopes: ['read'], ttlSecs: 30n, transport: {
              issue: async identity => { issuance++; await signCall(identity, issuer); return { bytes: frames.ed25519, untilNs: 100_000_000_000n }; },
              renew: async () => { throw Error('unexpected renewal'); },
              reconcile: async () => { throw Error('unexpected lookup'); },
            },
            admit: async (target, identity, bytes) => {
              request = Array.from(bytes);
              await signCall(identity, target, bytes);
              await recordAdmission(target.toText(), request);
              if (boundary !== 'confirmed') { entered(); await new Promise(() => {}); }
              return authTest.admissionReply(caller, issuer, 100_000_000_000n);
            },
          }).then(receipt => ({ receipt }), error => ({ error: error.code ?? error.name }));
          if (boundary === 'confirmed') {
            const result = await interruptedAdmission;
            if (result.error) throw Error(result.error);
          } else {
            await Promise.race([started, interruptedAdmission.then(result => { throw Error(JSON.stringify(result)); })]);
          }
          return { status: await journal.inspect(), issuance, request, target: target.toText(), issuer: issuer.toText() };
        }, boundary);
        assert.equal(before.issuance, 1);
        assert.equal(before.status.sequence, 0n);
        assert.deepEqual(before.status.destinations.map(d => d.canister).sort(), [before.issuer, before.target].sort());
        assert.equal(submitted.length, boundary === 'signing' ? 0 : 1);
        const submissionCount = submitted.length;
        const reopened = await crashAndReopen(ownedBrowser, context, page, prepare);
        assert.equal(submitted.length, submissionCount, 'reopening must not submit admission');
        const after = await reopened.evaluate(async boundary => {
          const { PublicJournal, Ed25519KeyIdentity, Principal, establishSession, signCall } = authTest;
          const journal = await PublicJournal.open(indexedDB, 'admission-crash', { id: 'clock', nowNs: () => now });
          const retained = await journal.inspect(); let issuerCalls = 0, admissionCalls = 0;
          const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
          const forbidden = async () => { issuerCalls++; throw Error('must reuse completed credential'); };
          const options = { journal, generation: 'login', browser, issuer,
            target: Principal.fromUint8Array(Uint8Array.of(2)), scopes: ['read'], ttlSecs: 30n,
            transport: { issue: forbidden, renew: forbidden, reconcile: forbidden },
            admit: async (target, identity, bytes) => {
              admissionCalls++; await signCall(identity, target, bytes);
              await recordAdmission(target.toText(), Array.from(bytes));
              return authTest.admissionReply(caller, issuer, 100_000_000_000n);
            } };
          if (boundary === 'logout-after-crash') await journal.beginLogout('login', 100_000_000_000n);
          // An explicit admission request is allowed to resubmit exact evidence;
          // no application callback is run or replayed by this recovery.
          const result = await establishSession(options).then(() => 'accepted', e => e.code ?? e.name);
          if (boundary !== 'logout-after-crash') await journal.beginLogout('login', 100_000_000_000n);
          const revoked = [];
          const finished = await authTest.cleanupUntilFinished({ journal, generation: 'login', browser,
            revoke: async (target, identity, bytes) => {
              await signCall(identity, target, bytes); revoked.push(target.toText()); return frames['logout-Revoked-max'];
            } });
          journal.close(); return { retained, issuerCalls, admissionCalls, result, finished, revoked };
        }, boundary);
        assert.deepEqual(after.retained, before.status);
        assert.equal(after.issuerCalls, 0);
        assert.equal(after.admissionCalls, boundary === 'logout-after-crash' ? 0 : 1);
        assert.equal(after.result, boundary === 'logout-after-crash' ? 'not_current' : 'accepted');
        assert.equal(submitted.length, submissionCount + after.admissionCalls);
        for (const call of submitted) assert.deepEqual(call, { target: before.target, bytes: before.request });
        assert.deepEqual(after.revoked.sort(), [before.issuer, before.target].sort());
        assert.equal(after.finished.phase, 'finished');
        assert.equal(after.finished.issuanceUnknown, true);
        assert(after.finished.destinations.every(d => d.attempts === 1 && d.confirmedUntilNs === 0xffff_ffff_ffff_ffffn));
      } finally { await ownedBrowser.close(); }
    });
  }
}
