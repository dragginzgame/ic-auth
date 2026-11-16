import assert from 'node:assert/strict';

// A host identity adapter delays/rejects real SDK signatures. This qualifies
// Auth's boundary, not a wallet protocol, persistent key store or device reconnect.
export async function externalSignerCases({ check, a }) {
  for (const role of ['account', 'browser']) {
    for (const interruption of ['rejected', 'cancelled', 'timeout', 'identity-changed', 'logout']) {
      await check(`${role} external signing ${interruption} retains lookup-only recovery and fences late signing`, async () => {
        const result = await a.evaluate(async ({ role, interruption }) => {
          const { PublicJournal, Ed25519KeyIdentity, retrieveCredential, signCall } = authTest;
          const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
          const account = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
          const original = role === 'account' ? account : browser;
          const replacement = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(11));
          const name = `external-${role}-${interruption}`;
          let journal = await PublicJournal.open(indexedDB, name, { id: 'clock', nowNs: () => now });
          await journal.start({ generation: 'login', caller, issuer });
          const evidence = () => ({ bytes: frames.ed25519, untilNs: 100_000_000_000n });
          if (role === 'browser') {
            await retrieveCredential({ journal, generation: 'login', browser, account, issuer, transport: {
              issue: async identity => { await signCall(identity); return evidence(); },
              renew: async () => { throw Error('unexpected renewal'); },
              reconcile: async () => { throw Error('unexpected lookup'); },
            } });
          }
          let entered, release, changed = false, prompts = 0, escaped = 0, retained;
          const started = new Promise(resolve => { entered = resolve; });
          const approval = new Promise(resolve => { release = resolve; });
          const denied = new Error('external approval rejected');
          const external = {
            getPrincipal: () => (changed ? replacement : original).getPrincipal(),
            transformRequest: async request => {
              prompts++; entered(); await approval;
              if (interruption === 'rejected') throw denied;
              return original.transformRequest(request);
            },
          };
          let signing;
          const calls = [];
          const dispatch = async (kind, identity, sequence) => {
            calls.push([kind, sequence]); retained = identity;
            signing = signCall(identity).then(() => { escaped++; return 'escaped'; }, e => e === denied ? 'rejected' : e.code ?? e.name);
            const outcome = await signing;
            if (outcome !== 'escaped') throw outcome;
            return evidence();
          };
          const controller = new AbortController();
          const options = { journal, generation: 'login', browser: role === 'browser' ? external : browser,
            account: role === 'account' ? external : account, issuer,
            ...(role === 'browser' ? { refreshThrough: 0n } : {}),
            transport: {
              issue: identity => dispatch('issue', identity, 0n),
              renew: (identity, sequence) => dispatch('renew', identity, sequence),
              reconcile: async () => { throw Error('unexpected initial lookup'); },
            } };
          const pending = retrieveCredential({ ...options, signal: controller.signal }).then(() => 'unexpected completion', e => e.code ?? e);
          await Promise.race([started, pending.then(result => { throw Error(String(result)); })]);
          if (interruption === 'cancelled') controller.abort();
          if (interruption === 'identity-changed') changed = true;
          if (interruption === 'logout') await journal.beginLogout('login', 100_000_000_000n);
          if (interruption !== 'timeout' && interruption !== 'cancelled') release();
          const failure = await pending;
          const before = await journal.inspect();
          journal.close();
          journal = await PublicJournal.open(indexedDB, name, { id: 'clock', nowNs: () => now });
          const reopened = await journal.inspect();
          let lookups = 0, missing, recovered, recoveredRole;
          const recovery = { journal, generation: 'login', browser, account, issuer, refreshThrough: 999n,
            transport: {
              issue: async () => { throw Error('must not reissue'); },
              renew: async () => { throw Error('must not renew'); },
              reconcile: async (identity, sequence) => {
                lookups++; calls.push(['lookup', sequence]);
                recoveredRole = identity.getPrincipal().toText() === original.getPrincipal().toText();
                await signCall(identity);
                if (lookups === 1) throw Error('signer reconnected but evidence unavailable');
                return evidence();
              },
            } };
          if (interruption === 'logout') {
            missing = await authFailure(retrieveCredential(recovery));
          } else {
            missing = await retrieveCredential(recovery).then(() => 'unexpected completion', e => e.message);
            const unknown = await journal.inspect();
            if (!unknown.issuanceUnknown || unknown.sequence !== before.sequence) throw Error('lookup discarded uncertainty');
            recovered = (await retrieveCredential(recovery)).sequence;
            const saved = await retrieveCredential({ ...recovery, refreshThrough: undefined, account: undefined });
            if (saved.sequence !== recovered || saved.bytes.length !== frames.ed25519.length
              || !saved.bytes.every((value, i) => value === frames.ed25519[i])) throw Error('recovered evidence not retained');
          }
          // A cancelled/timed-out host can ignore cancellation and approve after
          // recovery completed. Its old capability must still refuse the result.
          release(); const late = await signing;
          const closed = await authFailure(signCall(retained));
          const after = await journal.inspect(); journal.close();
          return { failure, before, reopened, calls, prompts, escaped, missing, recovered,
            recoveredRole, lookups, late, closed, after };
          async function authFailure(promise) { return promise.then(() => 'accepted', e => e.code ?? e.name); }
        }, { role, interruption });
        const sequence = role === 'account' ? 0n : 1n;
        const failure = interruption === 'identity-changed' ? 'identity_changed'
          : interruption === 'logout' ? 'not_current' : interruption;
        assert.equal(result.failure, failure);
        assert.equal(result.before.sequence, sequence);
        assert.equal(result.before.issuanceUnknown, true);
        assert.deepEqual(result.reopened, result.before);
        assert.equal(result.prompts, 1); assert.equal(result.escaped, 0);
        assert.equal(result.late, failure);
        assert(['closed', 'cancelled', 'timeout'].includes(result.closed));
        assert.deepEqual(result.calls[0], [role === 'account' ? 'issue' : 'renew', sequence]);
        if (interruption === 'logout') {
          assert.equal(result.missing, 'not_current'); assert.equal(result.lookups, 0);
          assert.deepEqual(result.after, result.before);
        } else {
          assert.equal(result.missing, 'signer reconnected but evidence unavailable');
          assert.equal(result.recovered, sequence); assert.equal(result.recoveredRole, true);
          assert.deepEqual(result.calls.slice(1), [['lookup', sequence], ['lookup', sequence]]);
          assert.equal(result.after.issuanceUnknown, true); // Issuance inventory remains conservatively sticky.
        }
      });
    }
  }
}
