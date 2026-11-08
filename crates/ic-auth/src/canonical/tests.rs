use super::*;

#[test]
fn chain_key_payload_hashes_preserve_framing() {
    for length in [0, 1, 255, 256, 4096] {
        let payload: Vec<u8> = (0..length).map(|i| (i % 256) as u8).collect();
        let len = (length as u32).to_be_bytes();
        let header = [CHAIN_KEY_BATCH_HEADER_DOMAIN, &len, &payload].concat();
        let leaf = [&[0], CHAIN_KEY_DELEGATION_CERT_DOMAIN, &len, &payload].concat();
        assert_eq!(
            hash_encoded(&[CHAIN_KEY_BATCH_HEADER_DOMAIN], true, &|out| {
                out(&payload);
                Ok(())
            })
            .unwrap(),
            <[u8; 32]>::from(Sha256::digest(header)),
        );
        assert_eq!(
            hash_encoded(&[&[0], CHAIN_KEY_DELEGATION_CERT_DOMAIN], true, &|out| {
                out(&payload);
                Ok(())
            })
            .unwrap(),
            <[u8; 32]>::from(Sha256::digest(leaf)),
        );
    }
}

#[test]
fn framed_length_overflow_refuses_before_the_hashing_pass() {
    let passes = std::cell::Cell::new(0);
    assert_eq!(
        hash_encoded(&[], true, &|out| {
            passes.set(passes.get() + 1);
            for _ in 0..65_536 {
                out(&[0; 65_536]);
            }
            Ok(())
        }),
        Err(CanonicalAuthError::LengthOverflow),
    );
    assert_eq!(passes.get(), 1);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn length_overflow_is_fallible_before_writing() {
    let mut out = vec![7];
    assert_eq!(
        encode_len(
            &mut |bytes| out.extend_from_slice(bytes),
            u32::MAX as usize + 1
        ),
        Err(CanonicalAuthError::LengthOverflow)
    );
    assert_eq!(out, [7]);
}
