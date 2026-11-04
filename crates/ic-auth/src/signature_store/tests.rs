use super::*;

#[test]
fn output_refuses_before_growing_past_the_host_budget() {
    let mut output = SignatureOutput {
        bytes: Vec::new(),
        maximum: 17,
    };
    output.write_all(b"abc").unwrap();
    output.write_all(&[7; 14]).unwrap();
    assert_eq!(output.bytes.len(), 17);
    assert!(output.bytes.capacity() <= 17);
    let before = output.bytes.clone();
    let capacity = output.bytes.capacity();
    assert_eq!(
        output.write_all(&[0; 4096]).unwrap_err().kind(),
        io::ErrorKind::WriteZero
    );
    assert_eq!(output.bytes, before);
    assert_eq!(output.bytes.capacity(), capacity);
    output.write_all(&[]).unwrap();
}

#[test]
fn retries_and_expired_repreparation_keep_exactly_one_expiration_entry() {
    let mut store = SignatureStore::new(
        Principal::from_slice(&[1]),
        SignatureLimits {
            max_signatures: 1,
            max_message_bytes: 32,
            max_certificate_bytes: 1024,
            max_signature_bytes: 2048,
        },
    )
    .unwrap();
    let inputs = SignatureInputs {
        seed: b"seed",
        domain: b"domain",
        message: b"message",
    };
    store.prepare(inputs, 0, 2000).unwrap();
    for now in 1..2000 {
        let prepared = store.prepare(inputs, now, 2000).unwrap();
        assert_eq!(prepared.retrieval_expires_at_ns, 2000);
        assert_eq!(store.expirations.len(), 1);
    }
    store.prepare(inputs, 2000, 2000).unwrap();
    assert_eq!(store.expirations.len(), 1);
    assert_eq!(store.prune(2000, 128), Ok(0));
    assert_eq!(store.prune(4000, 128), Ok(1));
    assert!(store.expirations.is_empty());
    assert!(store.is_empty());
}
