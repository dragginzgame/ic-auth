export type ClientErrorCode =
  | 'configuration' | 'stale_generation' | 'retrieval_expired' | 'transport'
  | 'invalid_material' | 'storage_conflict' | 'timeout'
  | 'storage_capacity' | 'storage_corrupt' | 'storage_unavailable';

export class ClientError extends Error {
  constructor(public readonly code: ClientErrorCode, options?: ErrorOptions) {
    super(code, options);
    this.name = 'ClientError';
  }
}
