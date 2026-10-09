import { HttpAgent, type Identity } from '@icp-sdk/core/agent';
import { TokenSessionClient, type ClientOptions, type IssuerOperations, type Session, type SessionSource, type TokenScope } from '../src/index.js';

/** The application's existing II SDK login supplies the authenticated identity. */
export function internetIdentityExample(
  login: (signal: AbortSignal) => Promise<Identity>,
  protectedAgentOptions: { host: string },
  issuer: (agent: HttpAgent, scope: TokenScope) => IssuerOperations,
  options: Omit<ClientOptions, 'sessions' | 'issuer'>,
): TokenSessionClient {
  let current: Session | undefined;
  let epoch = 0;
  const sessions: SessionSource = {
    current: () => current,
    connect: async signal => {
      const observed = epoch;
      const identity = await login(signal);
      if (signal.aborted || observed !== epoch) throw new Error('login cancelled');
      current = { identity, generation: crypto.randomUUID() };
      return current;
    },
    invalidate: async expected => {
      if (current?.generation !== expected) return false;
      epoch++; current = undefined; return true;
    },
  };
  return new TokenSessionClient({ ...options, sessions,
    issuer: (session, scope) => issuer(HttpAgent.createSync({ ...protectedAgentOptions, identity: session.identity }), scope),
  });
}
