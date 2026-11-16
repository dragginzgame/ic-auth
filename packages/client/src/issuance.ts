import type { Identity } from '@tokostudios/ic-platform/agent';
import { Principal } from '@tokostudios/ic-platform/principal';
import { AttemptError, runSigningAttempt } from './attempt.ts';
import { JournalError, type LoginJournal } from './journal.ts';
import { defaultMaximumBytes, maximumConfiguredBytes } from './generated/credential.js';

/** Exact generated Wire request frame and original expiry mapped to the journal clock. */
export interface IssuedCredential { readonly bytes: Uint8Array; readonly untilNs: bigint }
export interface CredentialEvidence extends IssuedCredential { readonly sequence: bigint }
/** Host adapter owns endpoints, authenticated replies and immutable login/grant binding.
 * It must use the supplied temporary SDK identity and honor cancellation where possible.
 */
export interface IssuerTransport {
  issue(identity: Identity, signal: AbortSignal): Promise<IssuedCredential>;
  renew(identity: Identity, sequence: bigint, signal: AbortSignal): Promise<IssuedCredential>;
  /** Lookup only; missing/pruned evidence never permits reissuance. */
  reconcile(identity: Identity, sequence: bigint, signal: AbortSignal): Promise<IssuedCredential>;
}
export interface CredentialRequestOptions {
  journal: LoginJournal;
  generation: string;
  browser: Identity;
  account?: Identity;
  issuer: Principal;
  transport: IssuerTransport;
  signal?: AbortSignal;
  refreshThrough?: bigint;
  maximumBytes?: number;
}

/** One journal claim and at most one bounded issuer call. No retry or admission. */
export async function retrieveCredential(options: CredentialRequestOptions): Promise<CredentialEvidence> {
  const { journal, generation, browser, account, issuer, transport, signal, refreshThrough,
    maximumBytes = defaultMaximumBytes } = options;
  const caller = browser.getPrincipal().toText();
  const accountCaller = account?.getPrincipal().toText();
  const destination = issuer.toText();
  if (issuer.isAnonymous() || (accountCaller !== undefined && Principal.fromText(accountCaller).isAnonymous())
    || !Number.isSafeInteger(maximumBytes) || maximumBytes < 1 || maximumBytes > maximumConfiguredBytes)
    throw new JournalError('configuration');
  // Capture adapter entrypoints and principal bytes before any journal await.
  const issue = transport.issue.bind(transport), renew = transport.renew.bind(transport),
    reconcile = transport.reconcile.bind(transport);
  const current = async (sequence?: bigint, needsAccount = false) => {
    const check = () => {
      if (signal?.aborted) throw new AttemptError('cancelled');
      if (browser.getPrincipal().toText() !== caller
        || (needsAccount && account?.getPrincipal().toText() !== accountCaller))
        throw new AttemptError('identity_changed');
    };
    check();
    await journal.assertIssuance(generation, Principal.fromText(caller), Principal.fromText(destination), sequence);
    check();
  };
  const capture = (value: IssuedCredential): IssuedCredential => {
    const { bytes, untilNs } = value;
    if (!(bytes instanceof Uint8Array) || bytes.length < 1 || bytes.length > maximumBytes)
      throw new JournalError('invalid_material');
    return { bytes: new Uint8Array(bytes), untilNs };
  };
  await current();
  const claim = await journal.claimCredential(generation, { accountAvailable: account !== undefined,
    ...(refreshThrough === undefined ? {} : { refreshThrough }) });
  if (claim.kind === 'saved') {
    await current(claim.sequence);
    return { ...capture(claim), sequence: claim.sequence };
  }
  const needsAccount = claim.signer === 'account';
  const identity = needsAccount ? account : browser;
  if (!identity) throw new JournalError('account_required');
  const result = await runSigningAttempt({ identity, target: Principal.fromText(destination),
    assertCurrent: () => current(claim.sequence, needsAccount), ...(signal ? { signal } : {}),
  }, async (signer, abort) => {
    const reply = claim.kind === 'issue' ? await issue(signer, abort)
      : claim.kind === 'renew' ? await renew(signer, claim.sequence, abort)
      : await reconcile(signer, claim.sequence, abort);
    // Own the reply before the attempt's asynchronous postflight journal check.
    return capture(reply);
  });
  await current(claim.sequence, needsAccount);
  await journal.completeCredential(generation, claim.sequence, result.bytes, result.untilNs);
  await current(claim.sequence, needsAccount);
  return { ...result, sequence: claim.sequence };
}
