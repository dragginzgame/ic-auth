use super::*;

#[test]
fn chain_key_payload_hashes_preserve_framing() {
    for length in [0, 1, 255, 256, 4096] {
        let payload: Vec<u8> = (0..length).map(|i| (i % 256) as u8).collect();
        let len = (length as u32).to_be_bytes();
        let header = [CHAIN_KEY_BATCH_HEADER_DOMAIN, &len, &payload].concat();
        let leaf = [&[0], CHAIN_KEY_DELEGATION_CERT_DOMAIN, &len, &payload].concat();
        assert_eq!(
            hash_chain_key_header_payload(&payload).unwrap(),
            <[u8; 32]>::from(Sha256::digest(header)),
        );
        assert_eq!(
            hash_chain_key_leaf_payload(&payload).unwrap(),
            <[u8; 32]>::from(Sha256::digest(leaf)),
        );
    }
}

#[test]
#[cfg(target_pointer_width = "64")]
fn length_overflow_is_fallible_before_writing() {
    let mut out = vec![7];
    assert_eq!(
        encode_len(&mut out, u32::MAX as usize + 1),
        Err(CanonicalAuthError::LengthOverflow)
    );
    assert_eq!(out, [7]);
}
