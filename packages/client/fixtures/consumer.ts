import * as auth from '@dragginzgame/ic-auth-client';
import { Principal } from '@tokostudios/ic-platform/principal';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';

// Compiled and run outside the producer's package. Calls are substituted: this
// checks package/type/lifecycle composition, never installed Canic acceptance.
export async function smoke(factory: IDBFactory) {
  const browser = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(1));
  const account = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(2));
  const issuer = Principal.fromUint8Array(new Uint8Array([1]));
  const receiver = Principal.fromUint8Array(new Uint8Array([2]));
  const now = 1_000_000_000_000n;
  const counts = { issues: 0, admissions: 0, revocations: 0 };
  const check = (value: boolean) => { if (!value) throw Error('Packed consumer assertion'); };
  const material: auth.AdmissionCredential = {
    credential: { claims: new Uint8Array([1, 2]), proof: { tag: 'Ed25519', value: new Uint8Array(64) } },
    admissionUntilNs: now + 60_000_000_000n,
  };
  const encoded = auth.credentialBytes(material.credential, 131_072);
  check(auth.credentialFromBytes(encoded, 131_072).claims.length === 2);
  // Reference every runtime export so browser bundling checks the full public entry.
  check(Object.keys(auth).length > 0 && typeof auth.httpAdmissionTransport === 'function'
    && typeof auth.httpLogoutTransport === 'function');
  const store = await auth.IndexedDbLogoutStore.open(factory, 'packed-consumer');
  try {
    const journal = new auth.PortableLogoutClient(store, {
      id: 'consumer-clock', nowNs: () => now, wait: async () => { throw Error('Unexpected retry'); },
    }, {
      revoke: async (_target, identity) => {
        check(identity.getPrincipal().toText() === browser.getPrincipal().toText());
        counts.revocations++;
        return { tag: 'Revoked', value: now + 120_000_000_000n };
      },
      dispose: async () => {},
    });
    await journal.start('consumer-login', browser, issuer);
    const transport: auth.PortableAdmissionTransport = {
      issue: async identity => {
        check(identity.getPrincipal().toText() === account.getPrincipal().toText());
        counts.issues++;
        return material;
      },
      reconcile: async () => { throw Error('Unexpected replacement issuance'); },
      admit: async (target, identity) => {
        check(target.toText() === receiver.toText());
        check(identity.getPrincipal().toText() === browser.getPrincipal().toText());
        counts.admissions++;
      },
    };
    const client = new auth.PortableAdmissionClient(journal, 'consumer-login', { browser, account }, issuer, transport);
    check((await client.establish(receiver)).getPrincipal().toText() === browser.getPrincipal().toText());
    await journal.begin('consumer-login', now + 120_000_000_000n);
    try {
      await client.establish(receiver);
      throw Error('Logout did not invalidate admission');
    } catch (error) {
      check(error instanceof auth.ClientError && error.code === 'stale_generation');
    }
    check((await journal.cleanup('consumer-login', browser)).phase === 'finished');
    return counts;
  } finally { store.close(); }
}
