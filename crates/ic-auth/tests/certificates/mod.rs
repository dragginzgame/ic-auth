//! Deterministic external-certificate mechanics; expected bytes and adversarial
//! mutations stay in their owning tests, independently of these constructors.
use ic_platform::{
    bls::PrivateKey,
    certification::{Certificate, Delegation, HashTree},
};

pub fn key(value: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = value;
    PrivateKey::deserialize(&bytes).unwrap()
}

pub fn certificate(
    tree: HashTree,
    key: &PrivateKey,
    delegation: Option<Delegation>,
) -> Certificate {
    let mut message = b"\x0Dic-state-root".to_vec();
    message.extend_from_slice(&tree.digest());
    Certificate {
        tree,
        signature: key.sign(&message).serialize().to_vec(),
        delegation,
    }
}

pub fn signature_bytes(value: &ic_platform::certification::cbor::CanisterSignature) -> Vec<u8> {
    ic_platform::certification::cbor::encode_canister_signature(
        &value.certificate,
        &value.tree,
        usize::MAX,
    )
    .unwrap()
}
