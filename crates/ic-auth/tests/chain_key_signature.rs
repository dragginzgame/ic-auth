#![cfg(feature = "chain-key-verification")]

use ic_auth::chain_key_signature::{
    ChainKeySignatureError as Error, normalize_secp256k1_signature, validate_secp256k1_public_key,
    verify_secp256k1_prehash,
};
use k256::ecdsa::{Signature, SigningKey, signature::hazmat::PrehashSigner};

const PREHASH: [u8; 32] = [42; 32];

fn signing_key() -> SigningKey {
    SigningKey::from_slice(&[7; 32]).unwrap()
}

fn signature() -> Signature {
    signing_key().sign_prehash(&PREHASH).unwrap()
}

fn high_s_signature() -> Signature {
    let signature = signature();
    assert!(signature.normalize_s().is_none());
    let (r, s) = signature.split_scalars();
    Signature::from_scalars(r.to_bytes(), (-*s).to_bytes()).unwrap()
}

fn malformed_signatures() -> [Vec<u8>; 6] {
    let mut recoverable = signature().to_bytes().to_vec();
    recoverable.push(0);
    [
        vec![],
        vec![1; 63],
        vec![0; 64],
        vec![0xff; 64],
        signature().to_der().as_bytes().to_vec(),
        recoverable,
    ]
}

#[test]
fn admits_compressed_and_uncompressed_keys_and_exact_prehash_signatures() {
    let key = signing_key();
    let signature = signature().to_bytes();
    for compressed in [true, false] {
        let public_key = key.verifying_key().to_encoded_point(compressed);
        assert_eq!(validate_secp256k1_public_key(public_key.as_bytes()), Ok(()));
        assert_eq!(
            verify_secp256k1_prehash(public_key.as_bytes(), &PREHASH, &signature),
            Ok(())
        );
    }
}

#[test]
fn rejects_malformed_keys_for_admission_and_verification() {
    let key = signing_key().verifying_key().to_encoded_point(true);
    let mut invalid_prefix = key.as_bytes().to_vec();
    invalid_prefix[0] = 1;
    for public_key in [
        vec![],
        vec![0],
        vec![0; 33],
        vec![0xff; 33],
        key.as_bytes()[..32].to_vec(),
        invalid_prefix,
    ] {
        assert_eq!(
            validate_secp256k1_public_key(&public_key),
            Err(Error::MalformedPublicKey)
        );
        assert_eq!(
            verify_secp256k1_prehash(&public_key, &PREHASH, &signature().to_bytes()),
            Err(Error::MalformedPublicKey)
        );
    }
}

#[test]
fn rejects_noncompact_encodings_and_out_of_range_scalars() {
    let key = signing_key().verifying_key().to_encoded_point(true);
    for signature in malformed_signatures() {
        assert_eq!(
            verify_secp256k1_prehash(key.as_bytes(), &PREHASH, &signature),
            Err(Error::MalformedSignature)
        );
    }
}

#[test]
fn explicit_normalization_preserves_low_s_and_converts_high_s_idempotently() {
    let key = signing_key().verifying_key().to_encoded_point(true);
    let low_s: [u8; 64] = signature().to_bytes().into();
    let high_s: [u8; 64] = high_s_signature().to_bytes().into();
    assert_ne!(high_s, low_s);
    for input in [low_s, high_s] {
        let unchanged = input;
        let normalized = normalize_secp256k1_signature(&input).unwrap();
        assert_eq!(normalized, low_s);
        assert_eq!(input, unchanged);
        assert_eq!(normalize_secp256k1_signature(&normalized), Ok(low_s));
        assert_eq!(
            verify_secp256k1_prehash(key.as_bytes(), &PREHASH, &normalized),
            Ok(())
        );
    }
    // Explicit producer normalization does not weaken strict verification.
    assert_eq!(
        verify_secp256k1_prehash(key.as_bytes(), &PREHASH, &high_s),
        Err(Error::HighSSignature)
    );
}

#[test]
fn normalization_rejects_malformed_encodings_and_scalars() {
    for signature in malformed_signatures() {
        assert_eq!(
            normalize_secp256k1_signature(&signature),
            Err(Error::MalformedSignature)
        );
    }
}

#[test]
fn normalization_success_does_not_authenticate_a_signature() {
    let key = signing_key().verifying_key().to_encoded_point(true);
    let normalized = normalize_secp256k1_signature(&high_s_signature().to_bytes()).unwrap();
    assert_eq!(
        verify_secp256k1_prehash(key.as_bytes(), &[0; 32], &normalized),
        Err(Error::InvalidSignature)
    );
    let mut changed_signature: [u8; 64] = signature().to_bytes().into();
    changed_signature[0] ^= 1;
    let normalized = normalize_secp256k1_signature(&changed_signature).unwrap();
    assert_eq!(
        verify_secp256k1_prehash(key.as_bytes(), &PREHASH, &normalized),
        Err(Error::InvalidSignature)
    );
}

#[test]
fn rejects_changed_hash_key_and_signature() {
    let key = signing_key().verifying_key().to_encoded_point(true);
    let other_key = SigningKey::from_slice(&[8; 32])
        .unwrap()
        .verifying_key()
        .to_encoded_point(true);
    let signature = signature().to_bytes();
    let mut changed_hash = PREHASH;
    changed_hash[0] ^= 1;
    let mut changed_signature = signature;
    changed_signature[0] ^= 1;
    for (public_key, hash, signature) in [
        (key.as_bytes(), &changed_hash, &signature),
        (other_key.as_bytes(), &PREHASH, &signature),
        (key.as_bytes(), &PREHASH, &changed_signature),
    ] {
        assert_eq!(
            verify_secp256k1_prehash(public_key, hash, signature),
            Err(Error::InvalidSignature)
        );
    }
}

#[test]
fn refuses_high_s_without_normalization_and_keeps_inputs_unchanged() {
    let key = signing_key().verifying_key().to_encoded_point(true);
    let high_s = high_s_signature().to_bytes();
    let unchanged = high_s;
    assert_eq!(
        verify_secp256k1_prehash(key.as_bytes(), &PREHASH, &high_s),
        Err(Error::HighSSignature)
    );
    assert_eq!(high_s, unchanged);
    // The same mathematical signature succeeds only when the caller explicitly
    // supplies the original low-S bytes; the verifier never rewrites them.
    assert_eq!(
        verify_secp256k1_prehash(key.as_bytes(), &PREHASH, &signature().to_bytes()),
        Ok(())
    );
}

#[test]
fn key_and_signature_shape_precede_cryptographic_rejection() {
    let key = signing_key().verifying_key().to_encoded_point(true);
    assert_eq!(
        verify_secp256k1_prehash(&[], &[0; 32], &[]),
        Err(Error::MalformedPublicKey)
    );
    assert_eq!(
        verify_secp256k1_prehash(key.as_bytes(), &[0; 32], &[]),
        Err(Error::MalformedSignature)
    );
    assert_eq!(
        verify_secp256k1_prehash(key.as_bytes(), &[0; 32], &high_s_signature().to_bytes()),
        Err(Error::HighSSignature)
    );
}
