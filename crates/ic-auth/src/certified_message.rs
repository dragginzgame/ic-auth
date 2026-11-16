/// Existing portable credential IC signature domain, not an ingress delegation.
pub const CERTIFIED_SIGNATURE_DOMAIN: &[u8] = b"ic-auth-portable-credential";
const MESSAGE_BYTES: usize = 1 + CERTIFIED_SIGNATURE_DOMAIN.len() + 32;

/// Construct the fixed IC length/domain/digest framing without heap allocation.
#[must_use]
pub fn certified_signature_message(claims_digest: [u8; 32]) -> [u8; MESSAGE_BYTES] {
    let mut message = [0; MESSAGE_BYTES];
    message[0] = CERTIFIED_SIGNATURE_DOMAIN.len() as u8;
    message[1..1 + CERTIFIED_SIGNATURE_DOMAIN.len()].copy_from_slice(CERTIFIED_SIGNATURE_DOMAIN);
    message[1 + CERTIFIED_SIGNATURE_DOMAIN.len()..].copy_from_slice(&claims_digest);
    message
}
