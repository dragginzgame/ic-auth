// Real-browser qualification only. Issuance below is an explicitly injected
// native fixture; receiver calls and signing use the actual SDK and IC root.
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';
import { Principal } from '@tokostudios/ic-platform/principal';
import { Endpoint, SubmitRequestType, Expiry, type Identity } from '@tokostudios/ic-platform/agent';
import { IndexedDbLogoutStore, PortableLogoutClient, PortableAdmissionClient,
  httpAdmissionTransport, httpLogoutTransport, credentialFromBytes } from '../../src/index.js';
import { httpUpdateTransport } from '../../src/update-http.js';
import * as read from '../../src/generated/read.js';

const check = (condition: unknown, message: string): void => { if (!condition) throw new Error(message); };
const rejected = async (operation: Promise<unknown>): Promise<void> => {
  try { await operation; } catch { return; }
  throw new Error('expected refusal');
};
const key = (seed: number) => Ed25519KeyIdentity.generate(new Uint8Array(32).fill(seed));
const issuer = Principal.fromUint8Array(new Uint8Array([1]));
const destination = Principal.fromUint8Array(new Uint8Array([2]));
function request(identity: Identity, target = destination): Parameters<Identity['transformRequest']>[0] {
  return { endpoint: Endpoint.Call, request: { method: 'POST' }, body: {
    request_type: SubmitRequestType.Call, canister_id: target, method_name: 'read',
    arg: new Uint8Array(), sender: identity.getPrincipal(),
    ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}'),
  } };
}

let now = 1_000_000_000n;
let store: IndexedDbLogoutStore;
let client: PortableLogoutClient;
let failNetwork = false;
let holdNetwork = false;
let attempts = 0;
let disposed = 0;
const identity = key(1);
const clock = { id: 'controlled-browser-clock', nowNs: () => now,
  wait: async (ms: number) => { now += BigInt(Math.ceil(ms)) * 1_000_000n; await new Promise(r => setTimeout(r, 1)); } };

export async function open(name: string, start = false): Promise<void> {
  store = await IndexedDbLogoutStore.open(indexedDB, name);
  client = new PortableLogoutClient(store, clock, {
    revoke: async (target, signer, signal) => {
      attempts++;
      await signer.transformRequest(request(signer, target));
      if (holdNetwork) await new Promise<void>((_, reject) => signal.addEventListener('abort', () => reject(signal.reason), { once: true }));
      if (failNetwork) throw new Error('injected unavailable transport');
      return { tag: 'Revoked', value: 86_400_000_000_000n };
    }, dispose: async () => { disposed++; },
  }, () => 0);
  if (start) {
    await client.start('generation', identity, issuer);
    await client.destination('generation', destination, now + 2_000_000_000n);
  }
}
export const time = (value: string): void => { now = BigInt(value); };
export const state = () => client.status('generation');
export const issuance = () => client.claimIssuance('generation');
export const begin = () => client.begin('generation', 11_000_000_000n);
export const network = (hold: boolean, fail: boolean): void => { holdNetwork = hold; failNetwork = fail; };
export const counts = () => ({ attempts, disposed });
export const cleanup = () => client.cleanup('generation', identity);
export const observer = () => client.cleanup('generation', undefined);
export const close = (): void => store.close();
export const sign = () => client.guardedIdentity('generation', identity).transformRequest(request(identity));
export const abort = async (): Promise<void> => {
  await rejected(store.transact(record => { if (!record) throw new Error('missing'); record.observedNs = 99n; throw new Error('abort fixture'); }));
};
export async function abortQueuedWrite(): Promise<void> {
  const put = IDBObjectStore.prototype.put;
  IDBObjectStore.prototype.put = function (...args) {
    const result = Reflect.apply(put, this, args) as IDBRequest<IDBValidKey>;
    queueMicrotask(() => this.transaction.abort());
    return result;
  };
  try { await rejected(client.destination('generation', destination, now + 3_000_000_000n)); }
  finally { IDBObjectStore.prototype.put = put; }
}
export async function refuseRelaxedCommit(): Promise<void> {
  const transaction = IDBDatabase.prototype.transaction;
  IDBDatabase.prototype.transaction = function (names, mode, options) {
    return transaction.call(this, names, mode, { ...options, durability: 'relaxed' });
  };
  try { await rejected(client.destination('generation', destination, now + 3_000_000_000n)); }
  finally { IDBDatabase.prototype.transaction = transaction; }
}
let releaseSignature: (() => void) | undefined;
let lateResult: Promise<boolean> | undefined;
export function delayedSign(): void {
  const slow: Identity = { getPrincipal: () => identity.getPrincipal(), transformRequest: async value => {
    await new Promise<void>(resolve => { releaseSignature = resolve; });
    return identity.transformRequest(value);
  } };
  lateResult = client.guardedIdentity('generation', slow).transformRequest(request(identity)).then(() => false, () => true);
}
export const signatureWaiting = (): boolean => releaseSignature !== undefined;
export async function finishSignature(): Promise<boolean | undefined> { releaseSignature?.(); return lateResult; }

export interface LiveInput {
  issuer: string; receiver: string; root: string; now: string;
  credential: string; peerCredential: string; subject: string; until: string;
}
const unhex = (text: string): Uint8Array => Uint8Array.from(text.match(/../g) ?? [], value => Number.parseInt(value, 16));

export async function sdk(input: LiveInput): Promise<Record<string, number>> {
  const first = key(9), peer = key(10), account = key(81);
  check(account.getPrincipal().toText() === input.subject, 'account fixture mismatch');
  const targets = [Principal.fromText(input.issuer), Principal.fromText(input.receiver)];
  const rootKey = unhex(input.root), origin = BigInt(input.now), started = performance.now();
  const liveClock = { id: 'live-browser-clock', nowNs: () => origin + BigInt(Math.ceil((performance.now() - started) * 1_000_000)),
    wait: (ms: number) => new Promise<void>(resolve => setTimeout(resolve, ms)) };
  const firstStore = await IndexedDbLogoutStore.open(indexedDB, 'live-first');
  const peerStore = await IndexedDbLogoutStore.open(indexedDB, 'live-peer');
  let issued = 0, reconciled = 0, disposedKeys = 0, dropped = false, logoutCalls = 0;
  let lookupReady = false;
  const transport = httpLogoutTransport({ host: location.origin, rootKey, methodName: 'logout_portable_session',
    fetch: async (url, init) => {
      const response = await fetch(url, init);
      if (new URL(url instanceof Request ? url.url : url.toString()).pathname.endsWith('/call')) {
        logoutCalls++;
        if (!dropped) { dropped = true; throw new Error('injected lost logout response'); }
      }
      return response;
    }, dispose: async () => { disposedKeys++; },
  });
  let journal = new PortableLogoutClient(firstStore, liveClock, transport, () => 0);
  const peerJournal = new PortableLogoutClient(peerStore, liveClock, transport, () => 0);
  const admit = httpAdmissionTransport({ host: location.origin, rootKey,
    methodName: 'establish_portable_session', scopes: ['read'] });
  const session = async (target: Principal, signer: Identity) => read.decodeResponse(await httpUpdateTransport({
    host: location.origin, rootKey, methodName: 'portable_session_read',
  })(target, signer, read.encodeRequest(null), new AbortController().signal));
  const original = { credential: credentialFromBytes(unhex(input.credential), 131_072), admissionUntilNs: BigInt(input.until) };
  const probeAccount = async (signer: Identity): Promise<void> => {
    check(signer.getPrincipal().toText() === account.getPrincipal().toText(), 'wrong issuer signer');
    check('err' in await session(targets[0]!, signer), 'account inherited browser session');
  };
  const makeClient = () => new PortableAdmissionClient(journal, 'live-first', { browser: first, account }, targets[0]!, {
    issue: async signer => { await probeAccount(signer); issued++; throw new Error('injected lost issuance reply'); },
    reconcile: async signer => { await probeAccount(signer); reconciled++; if (!lookupReady) throw new Error('unresolved lookup'); return original; },
    admit,
  }, undefined, 131_072);
  let currentStore = firstStore;
  try {
    await journal.start('live-first', first, targets[0]!);
    await rejected(makeClient().establish(targets[0]!));
    check((await journal.status('live-first')).issuanceUnknown, 'lost issuance forgotten');
    firstStore.close();
    currentStore = await IndexedDbLogoutStore.open(indexedDB, 'live-first');
    journal = new PortableLogoutClient(currentStore, liveClock, transport, () => 0);
    await rejected(makeClient().establish(targets[0]!));
    lookupReady = true;
    const firstClient = makeClient();
    const guarded = await firstClient.establish(targets[0]!);
    await firstClient.establish(targets[1]!);
    const before = await session(targets[1]!, guarded);
    check('ok' in before, 'browser admission failed');
    await firstClient.establish(targets[1]!);
    const retry = await session(targets[1]!, guarded);
    check('ok' in retry && 'ok' in before && retry.ok.expires_at_ns === before.ok.expires_at_ns, 'retry extended lease');
    check(issued === 1 && reconciled === 2, 'recovery reissued');
    check((await journal.status('live-first')).targets.every(t => t.admissionUntilNs === original.admissionUntilNs), 'recovery extended proof deadline');
    await peerJournal.start('live-peer', peer, targets[0]!);
    const peerClient = new PortableAdmissionClient(peerJournal, 'live-peer', { browser: peer, account }, targets[0]!, {
      issue: async signer => { await probeAccount(signer); return { credential: credentialFromBytes(unhex(input.peerCredential), 131_072), admissionUntilNs: BigInt(input.until) }; },
      reconcile: async () => { throw new Error('unexpected peer reconcile'); }, admit,
    }, undefined, 131_072);
    const peerGuarded = await peerClient.establish(targets[0]!);
    await peerClient.establish(targets[1]!);
    await journal.begin('live-first', liveClock.nowNs() + 70_000_000_000n);
    const finished = await journal.cleanup('live-first', first);
    check(finished.phase === 'finished' && finished.issuanceUnknown, 'unknown issuance or cleanup lost');
    check(finished.targets.every(t => t.confirmedUntilNs !== undefined), 'unauthenticated logout completion');
    await rejected(session(targets[1]!, guarded));
    for (const target of targets) {
      check('err' in await session(target, first), 'retained first key bypassed logout');
      const result = await session(target, peerGuarded);
      check('ok' in result && Principal.fromUint8Array(result.ok.subject).toText() === input.subject, 'peer/account isolation failed');
    }
    await peerJournal.begin('live-peer', liveClock.nowNs() + 70_000_000_000n);
    check((await peerJournal.cleanup('live-peer', peer)).phase === 'finished', 'peer cleanup failed');
    for (const target of targets) check('err' in await session(target, peer), 'retained peer key bypassed logout');
    check(disposedKeys === 2 && logoutCalls === 5, 'lost-reply retry/disposal mismatch');
    return { issued, reconciled, disposedKeys, logoutCalls };
  } finally { currentStore.close(); peerStore.close(); }
}
