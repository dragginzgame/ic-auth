import assert from 'node:assert/strict';

export async function transportCases({ check, a, b }) {
  await check('issuer adapter uses account then browser signing, commits before returning and reuses completion', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { Ed25519KeyIdentity, retrieveCredential, signCall } = authTest;
      const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
      const account = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
      const journal = await open('transport-success'); await journal.start({ generation: 'login', caller, issuer });
      const calls = [], roles = [], signatures = [];
      let retained, reply;
      const guard = journal.assertIssuance.bind(journal);
      journal.assertIssuance = (...args) => {
        // Mutate adapter-owned evidence during postflight's real IndexedDB await.
        if (reply) reply.bytes.fill(0);
        return guard(...args);
      };
      const transport = {
        issue: async identity => {
          calls.push('issue'); roles.push(identity.getPrincipal().toText()); retained = identity;
          const signed = await signCall(identity); signatures.push(signed.body.sender_sig.length);
          reply = { bytes: new Uint8Array(frames.ed25519), untilNs: 1_000n }; return reply;
        },
        renew: async (identity, sequence) => {
          calls.push(`renew-${sequence}`); roles.push(identity.getPrincipal().toText());
          const signed = await signCall(identity); signatures.push(signed.body.sender_sig.length);
          return { bytes: frames.certified, untilNs: 2_000n };
        },
        reconcile: async () => { throw new Error('unexpected lookup'); },
      };
      const options = { journal, generation: 'login', browser, account, issuer, transport };
      const issued = await retrieveCredential(options);
      const persisted = await raw('transport-success', b => { const r = b.get('credential'); return () => Array.from(r.result); });
      const exact = issued.bytes.length === frames.ed25519.length && issued.bytes.every((v, i) => v === frames.ed25519[i]);
      const closed = await failure(signCall(retained));
      const saved = await retrieveCredential({ ...options, account: undefined });
      const renewed = await retrieveCredential({ ...options, account: undefined, refreshThrough: 0n });
      journal.close();
      return { calls, accountUsed: roles[0] === account.getPrincipal().toText(), browserUsed: roles[1] === caller.toText(),
        signatures, exact, committed: persisted.every((v, i) => v === frames.ed25519[i]), closed,
        saved: saved.sequence, renewed: renewed.sequence };
    }), { calls: ['issue', 'renew-1'], accountUsed: true, browserUsed: true, signatures: [64, 64],
      exact: true, committed: true, closed: 'closed', saved: 0n, renewed: 1n });
  });

  await check('lost issuer reply propagates unchanged; subsequent calls only reconcile the retained sequence', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { Ed25519KeyIdentity, retrieveCredential, signCall } = authTest;
      const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
      const account = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
      let journal = await open('transport-lost'); await journal.start({ generation: 'login', caller, issuer });
      const lost = new Error('unknown issuer effect'); const calls = [];
      const transport = {
        issue: async identity => { calls.push('issue'); await signCall(identity); throw lost; },
        renew: async () => { throw new Error('unexpected renewal'); },
        reconcile: async (identity, sequence) => {
          calls.push(`lookup-${sequence}`);
          if (identity.getPrincipal().toText() !== account.getPrincipal().toText()) throw new Error('wrong signer');
          await signCall(identity); return { bytes: frames.ed25519, untilNs: 700n };
        },
      };
      const request = extra => retrieveCredential({ journal, generation: 'login', browser, account, issuer, transport, ...extra });
      const same = await request().then(() => false, e => e === lost);
      journal.close(); journal = await open('transport-lost');
      const absent = await failure(request({ account: undefined }));
      const recovered = await request(); journal.close();
      return [same, absent, calls, recovered.sequence, recovered.untilNs];
    }), [true, 'account_required', ['issue', 'lookup-0'], 0n, 700n]);
  });

  await check('issuer/caller binding and pre-cancellation refuse before reserving work', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { Ed25519KeyIdentity, Principal, retrieveCredential } = authTest;
      const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
      const other = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
      const journal = await open('transport-binding'); await journal.start({ generation: 'login', caller, issuer });
      const receiver = Principal.fromUint8Array(Uint8Array.of(2));
      await journal.addDestination('login', receiver, 1_000n);
      let calls = 0;
      const transport = { issue: async () => { calls++; throw new Error('unexpected dispatch'); }, renew: async () => {}, reconcile: async () => {} };
      const request = extra => retrieveCredential({ journal, generation: 'login', browser, account: other, issuer, transport, ...extra });
      const refused = [await failure(request({ issuer: receiver })), await failure(request({ browser: other })),
        await failure(request({ signal: AbortSignal.abort() })), await failure(request({ maximumBytes: 0 }))];
      const unknown = (await journal.inspect()).issuanceUnknown; journal.close();
      return [refused, calls, unknown];
    }), [['not_current', 'not_current', 'cancelled', 'configuration'], 0, false]);
  });

  await check('captured issuer and account identity survive awaits; substituted destination cannot sign', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { Ed25519KeyIdentity, Principal, retrieveCredential, signCall } = authTest;
      const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
      const account = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
      const journal = await open('transport-mutation'); await journal.start({ generation: 'login', caller, issuer });
      const target = Principal.fromText(issuer.toText());
      const transport = { issue: async identity => {
        const wrong = await failure(signCall(identity, target));
        if (wrong !== 'wrong_destination') throw new Error('mutated target escaped');
        await signCall(identity); return { bytes: frames.ed25519, untilNs: 1_000n };
      }, renew: async () => {}, reconcile: async () => {} };
      const pending = retrieveCredential({ journal, generation: 'login', browser, account, issuer: target, transport });
      target.toUint8Array()[0] = 2;
      const value = await pending; journal.close();
      const second = await open('transport-account-change'); await second.start({ generation: 'login', caller, issuer });
      let principal = account.getPrincipal(), calls = 0;
      const mutable = { getPrincipal: () => principal, transformRequest: r => account.transformRequest(r) };
      const changed = retrieveCredential({ journal: second, generation: 'login', browser, account: mutable, issuer,
        transport: { ...transport, issue: async () => { calls++; throw new Error('must refuse'); } } });
      principal = browser.getPrincipal(); const code = await failure(changed); second.close();
      return [value.sequence, code, calls];
    }), [0n, 'identity_changed', 0]);
  });

  await check('another tab logs out while an issuer reply is suspended; late evidence stays absent', async () => {
    await a.evaluate(async () => {
      const { Ed25519KeyIdentity, retrieveCredential, signCall } = authTest;
      globalThis.transportJournal = await open('transport-logout');
      await transportJournal.start({ generation: 'login', caller, issuer });
      let entered; const ready = new Promise(resolve => { entered = resolve; });
      const gate = new Promise(resolve => { globalThis.resumeIssuer = resolve; });
      const transport = { issue: async identity => {
        await signCall(identity); entered(); await gate;
        return { bytes: frames.ed25519, untilNs: 1_000n };
      }, renew: async () => {}, reconcile: async () => {} };
      globalThis.pendingIssuer = failure(retrieveCredential({ journal: transportJournal, generation: 'login', issuer, transport,
        browser: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)), account: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)) }));
      await ready;
    });
    await b.evaluate(async () => { const j = await open('transport-logout'); await j.beginLogout('login', 1_000n); j.close(); });
    assert.deepEqual(await a.evaluate(async () => {
      resumeIssuer(); const code = await pendingIssuer;
      const state = await transportJournal.inspect(); transportJournal.close();
      const evidence = await raw('transport-logout', b => { const r = b.count('credential'); return () => r.result; });
      return [code, state.phase, state.issuanceUnknown, evidence];
    }), ['not_current', 'cleanup', true, 0]);
  });

  await check('advancing the sequence in another tab fences an older issuer signer', async () => {
    await a.evaluate(async () => {
      const { Ed25519KeyIdentity, retrieveCredential, signCall } = authTest;
      globalThis.transportJournal = await open('transport-sequence');
      await transportJournal.start({ generation: 'login', caller, issuer });
      let entered; const ready = new Promise(resolve => { entered = resolve; });
      const gate = new Promise(resolve => { globalThis.resumeIssuer = resolve; });
      globalThis.signatureEscaped = false;
      const transport = { issue: async identity => {
        entered(); await gate; await signCall(identity); signatureEscaped = true;
        return { bytes: frames.ed25519, untilNs: 1_000n };
      }, renew: async () => {}, reconcile: async () => {} };
      globalThis.pendingIssuer = failure(retrieveCredential({ journal: transportJournal, generation: 'login', issuer, transport,
        browser: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)), account: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)) }));
      await ready;
    });
    await b.evaluate(async () => {
      const j = await open('transport-sequence');
      await j.completeCredential('login', 0n, frames.ed25519, 1_000n);
      await j.claimCredential('login', { accountAvailable: false, refreshThrough: 0n }); j.close();
    });
    assert.deepEqual(await a.evaluate(async () => {
      resumeIssuer(); const code = await pendingIssuer;
      const state = await transportJournal.inspect(); transportJournal.close();
      return [code, signatureEscaped, state.sequence];
    }), ['not_current', false, 1n]);
  });

  await check('cancellation and five-second timeout leave pending lookup-only work despite ignored signals', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { Ed25519KeyIdentity, retrieveCredential, signCall } = authTest;
      const results = [];
      for (const mode of ['cancel', 'timeout']) {
        const journal = await open(`transport-${mode}`); await journal.start({ generation: 'login', caller, issuer });
        const abort = new AbortController(); let resume, entered, retained, calls = 0;
        const gate = new Promise(resolve => { resume = resolve; }), ready = new Promise(resolve => { entered = resolve; });
        const transport = { issue: async identity => {
          calls++; retained = identity; entered(); await gate;
          return { bytes: frames.ed25519, untilNs: 1_000n };
        }, renew: async () => {}, reconcile: async () => {} };
        const pending = failure(retrieveCredential({ journal, generation: 'login', issuer, transport, signal: abort.signal,
          browser: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)), account: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)) }));
        await ready; if (mode === 'cancel') abort.abort();
        const code = await pending; resume();
        const lateSigner = await failure(signCall(retained));
        const claim = await journal.claimCredential('login', { accountAvailable: true });
        const evidence = await raw(`transport-${mode}`, b => { const r = b.count('credential'); return () => r.result; });
        journal.close(); results.push([code, lateSigner, calls, claim.kind, evidence]);
      }
      return results;
    }), [['cancelled', 'cancelled', 1, 'reconcile', 0], ['timeout', 'timeout', 1, 'reconcile', 0]]);
  });

  await check('failed completion commit cannot return evidence; recovery uses lookup without reissuing', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { Ed25519KeyIdentity, retrieveCredential } = authTest;
      const journal = await open('transport-commit'); await journal.start({ generation: 'login', caller, issuer });
      const calls = [], reply = () => ({ bytes: frames.ed25519, untilNs: 1_000n });
      const transport = { issue: async () => { calls.push('issue'); return reply(); },
        reconcile: async () => { calls.push('lookup'); return reply(); }, renew: async () => {} };
      const options = { journal, generation: 'login', issuer, transport,
        browser: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)), account: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)) };
      const put = IDBObjectStore.prototype.put;
      IDBObjectStore.prototype.put = function(value, key) {
        const r = put.call(this, value, key);
        if (key === 'credential') r.addEventListener('success', () => this.transaction.abort());
        return r;
      };
      let refused;
      try { refused = await failure(retrieveCredential(options)); }
      finally { IDBObjectStore.prototype.put = put; }
      const recovered = await retrieveCredential(options);
      const bounded = await failure(retrieveCredential({ ...options, maximumBytes: 1 }));
      journal.close(); return [refused, calls, recovered.sequence, bounded];
    }), ['storage_unavailable', ['issue', 'lookup'], 0n, 'invalid_material']);
  });
}
