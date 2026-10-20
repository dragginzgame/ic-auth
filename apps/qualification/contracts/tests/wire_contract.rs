//! Refuse the retired transport and wrong method/direction before handler admission.
use ic_auth_qualification_contracts::*;
use ic_wire::{
    Bytes, Error, Method, decode_request, decode_response, encode_request, encode_response,
};

fn refusal_contract<M, T>()
where
    M: Method<Response = Result<T, Refusal>>,
    T: serde::de::DeserializeOwned,
{
    let encoded = encode_response::<M>(&Err(refusal("controller required".into()))).unwrap();
    assert!(
        matches!(decode_response::<M>(encoded.as_bytes()), Ok(Err(error)) if error.as_str() == "controller required")
    );
}
#[test]
fn controls_refuse_old_transport_wrong_identity_and_unbounded_requests() {
    let owner = Bytes::new(vec![1, 2, 3]).unwrap();
    let arg = encode_request::<Initialize>(&owner).unwrap();
    assert_eq!(decode_request::<Initialize>(arg.as_bytes()).unwrap(), owner);
    assert!(matches!(
        decode_request::<Initialize>(b"DIDL\x00\x00"),
        Err(Error::Identity | Error::Malformed)
    ));
    assert!(decode_request::<Upgrade>(arg.as_bytes()).is_err());
    let empty = encode_request::<Upgrade>(&()).unwrap();
    assert!(matches!(
        decode_request::<AssetVersion>(empty.as_bytes()),
        Err(Error::Identity)
    ));
    assert!(matches!(
        decode_response::<OwnerOperation>(arg.as_bytes()),
        Err(Error::Identity)
    ));
    let mut trailing = arg.into_bytes();
    trailing.push(0);
    assert!(decode_request::<Initialize>(&trailing).is_err());
    assert!(decode_request::<Initialize>(&vec![0; CONTROL_LIMITS.bytes + 1]).is_err());
    refusal_contract::<Prepare, _>();
    refusal_contract::<Retrieve, _>();
    refusal_contract::<Prune, _>();
    refusal_contract::<SetAsset, _>();
    refusal_contract::<OwnerOperation, _>();
    refusal_contract::<InstallReceiver, _>();
    refusal_contract::<InvalidateAuthority, _>();
    refusal_contract::<ProfileSession, _>();
    refusal_contract::<EnableCheckpoint, _>();
    refusal_contract::<ConfirmAuthority, _>();
}
#[test]
fn fixture_diagnostics_keep_admission_reply_opaque_and_bounded() {
    let reply = model::AdmissionProfile {
        reply: Bytes::new(vec![1; 4096]).unwrap(),
        instructions: 7,
    };
    let encoded = encode_response::<ProfileAdmission>(&reply).unwrap();
    assert_eq!(
        decode_response::<ProfileAdmission>(encoded.as_bytes()).unwrap(),
        reply
    );
    let error = refusal("🚫".repeat(100));
    assert!(error.as_str().len() <= 256);
    assert!("🚫".repeat(100).starts_with(error.as_str()));
}
