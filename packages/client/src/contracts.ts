import type { Response } from './generated/establish.js';

export type AdmissionRefusal = Extract<Response, { err: unknown }>['err']['tag'];

export type ClientErrorCode =
  | 'configuration' | 'stale_generation' | 'retrieval_expired' | 'transport'
  | 'invalid_material' | 'storage_conflict' | 'timeout'
  | 'storage_capacity' | 'storage_corrupt' | 'storage_unavailable' | 'admission_refused';

export class ClientError extends Error {
  constructor(public readonly code: ClientErrorCode, options?: ErrorOptions) {
    super(code, options);
    this.name = 'ClientError';
  }
}

/** An authenticated receiver refusal, never evidence of a valid proof or permission to retry. */
export class AdmissionError extends ClientError {
  constructor(public readonly refusal: AdmissionRefusal) {
    super('admission_refused');
    this.name = 'AdmissionError';
  }
}
