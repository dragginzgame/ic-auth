// Explicit reference-browser values; deliberately unverified cryptographic data.
// Shared by fixture verification only, never imported by product code.
const tag = (tag, value = null) => ({ tag, value });
const bytes = (...values) => Uint8Array.from(values);
const hash = value => Array(32).fill(value);
const audience = () => tag('Fleet', { canonical_network_id: hash(1), fleet_id: hash(2) });
const grants = () => [{ target: 'project_instance', scopes: ['write', 'read'] }];

export function ed25519Credential() {
  return { claims: bytes(0, 128, 255), proof: tag('Ed25519', Uint8Array.from({ length: 64 }, (_, i) => i)) };
}

export function certifiedCredential() {
  return {
    claims: bytes(128, 0, 255, 1),
    proof: tag('Certified', {
      delegation: {
        cert: {
          root_pid: bytes(3), issuer_pid: bytes(4), issuer_proof_alg: tag('IcCanisterSignatureV1'),
          issuer_proof_binding_hash: hash(5), issuer_proof_binding: tag('IcCanisterSignatureV1', hash(6)),
          issued_at_ns: 127n, not_before_ns: 128n, expires_at_ns: 129n, max_token_ttl_ns: 130n,
          aud: audience(), grants: grants(),
        },
        root_proof: tag('IcChainKeyBatchSignatureV1', {
          header: {
            schema_version: 257, root_canister_id: bytes(7), batch_id: hash(8),
            proof_epoch: 131n, registry_epoch: 132n, registry_hash: hash(9), tree_root: hash(10),
            not_before_ns: 133n, expires_at_ns: 134n, algorithm: tag('EcdsaSecp256k1'),
            key_id: { name: 'root-key' }, derivation_path_hash: hash(11), key_version: 135n,
          },
          delegation_cert: {
            root_canister_id: bytes(12), issuer_canister_id: bytes(13), proof_epoch: 136n,
            issuer_proof_algorithm: tag('IcCanisterSignatureV1'), issuer_proof_binding_hash: hash(14),
            issuer_proof_binding: tag('IcCanisterSignatureV1', hash(15)), max_token_ttl_ns: 137n,
            audience: audience(), grants: grants(), not_before_ns: 138n, expires_at_ns: 139n,
            registry_epoch: 140n, registry_hash: hash(16),
          },
          issuer_witness: { steps: [tag('LeftSibling', hash(17)), tag('RightSibling', hash(18))] },
          signature: {
            algorithm: tag('EcdsaSecp256k1'), key_id: { name: 'signature-key' },
            derivation_path: [bytes(), bytes(0, 128, 255)], public_key: bytes(19, 20), signature: bytes(21, 22, 23),
          },
        }),
      },
      issuer_signature: { signature_cbor: bytes(24, 25, 26, 27), public_key_der: bytes(28, 29) },
    }),
  };
}

export function admissionCases(credential, establish) {
  const cases = [];
  for (const [kind, value] of [['ed25519', ed25519Credential()], ['certified', certifiedCredential()]]) {
    for (const request of [true, false]) {
      cases.push({ name: `credential-${kind}-${request ? 'request' : 'response'}`, codec: credential, value, request });
    }
  }
  cases.push({ name: 'credential-empty', codec: credential, request: true,
    value: { claims: bytes(), proof: tag('Ed25519', bytes()) } });
  cases.push({ name: 'establish-none', codec: establish, request: true,
    value: { credential: ed25519Credential(), scopes: ['write', 'read', 'write'], ttl_secs: null } });
  cases.push({ name: 'establish-some', codec: establish, request: true,
    value: { credential: certifiedCredential(), scopes: ['', 'Read', 'read::write'], ttl_secs: { some: 2n ** 64n - 1n } } });
  cases.push({ name: 'establish-success', codec: establish, request: false,
    value: { ok: { caller: bytes(30), issuer: bytes(31), subject: bytes(32), expires_at_ns: 2n ** 64n - 1n } } });
  for (const reason of ['NotYetValid', 'Expired', 'InvalidCredential', 'AuthorityUnavailable', 'BeforeReset',
    'ScopeRejected', 'ReplayConflict', 'Revoked', 'Capacity', 'Throttled', 'Unavailable']) {
    cases.push({ name: `establish-${reason}`, codec: establish, request: false, value: { err: tag(reason) } });
  }
  return cases;
}
