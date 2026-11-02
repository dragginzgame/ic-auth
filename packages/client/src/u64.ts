/** Primitive range only; each owner applies its clock, expiry and positivity rules. */
export const isU64 = (value: unknown): value is bigint =>
  typeof value === 'bigint' && value >= 0n && value <= (1n << 64n) - 1n;
