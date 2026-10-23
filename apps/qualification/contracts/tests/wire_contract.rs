//! Refuse the retired transport and wrong method/direction before handler admission.
use ic_auth_qualification_contracts::*;
use ic_wire::{Bytes, Error};

macro_rules! refusal_contract { ($method:ty) => {{
 let encoded = <$method>::encode_response(&Err(refusal("controller required".into()))).unwrap();
 assert!(matches!(<$method>::decode_response(&encoded), Ok(Err(error)) if error.as_str() == "controller required"));
}}; }
#[test]
fn controls_refuse_old_transport_wrong_identity_and_unbounded_requests() {
    let owner = Bytes::new(vec![1, 2, 3]).unwrap();
    let arg = Initialize::encode_request(&owner).unwrap();
    assert_eq!(Initialize::decode_request(arg.as_slice()).unwrap(), owner);
    assert!(matches!(
        Initialize::decode_request(b"DIDL\x00\x00"),
        Err(Error::Identity | Error::Malformed)
    ));
    assert!(Upgrade::decode_request(arg.as_slice()).is_err());
    let empty = Upgrade::encode_request(&()).unwrap();
    assert!(matches!(
        AssetVersion::decode_request(empty.as_slice()),
        Err(Error::Identity)
    ));
    assert!(matches!(
        OwnerOperation::decode_response(arg.as_slice()),
        Err(Error::Identity)
    ));
    let mut trailing = arg;
    trailing.push(0);
    assert!(Initialize::decode_request(&trailing).is_err());
    assert!(Initialize::decode_request(&vec![0; CONTROL_LIMITS.bytes + 1]).is_err());
    refusal_contract!(Prepare);
    refusal_contract!(Retrieve);
    refusal_contract!(Prune);
    refusal_contract!(SetAsset);
    refusal_contract!(OwnerOperation);
    refusal_contract!(InstallReceiver);
    refusal_contract!(InvalidateAuthority);
    refusal_contract!(ProfileSession);
    refusal_contract!(EnableCheckpoint);
    refusal_contract!(ConfirmAuthority);
    refusal_contract!(SetSessionAsset);
}
#[test]
fn fixture_diagnostics_keep_admission_reply_opaque_and_bounded() {
    let reply = model::AdmissionProfile {
        reply: Bytes::new(vec![1; 4096]).unwrap(),
        instructions: 7,
    };
    let encoded = ProfileAdmission::encode_response(&reply).unwrap();
    assert_eq!(
        ProfileAdmission::decode_response(encoded.as_slice()).unwrap(),
        reply
    );
    let error = refusal("🚫".repeat(100));
    assert!(error.as_str().len() <= 256);
    assert!("🚫".repeat(100).starts_with(error.as_str()));
}
