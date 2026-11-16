use ic_auth_protocol_types::Principal;

/// Derive the standard IC self-authenticating caller for a raw browser key.
/// This checks neither curve membership nor key possession/ingress provenance.
#[must_use]
pub fn browser_principal(public_key: &[u8; 32]) -> Principal {
    let mut der = [0; 44];
    der[..12].copy_from_slice(&[
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0,
    ]);
    der[12..].copy_from_slice(public_key);
    Principal::self_authenticating(der)
}
