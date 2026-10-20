//! Bind fixture-only Wire controls to Platform CDK raw codec hooks.
use ic_auth_qualification_contracts::{CONTROL_LIMITS, Refusal, refusal};
use ic_wire::Method;

pub(crate) fn guard() -> Result<(), String> {
    if ic_platform::ic0::msg_arg_data_size() > CONTROL_LIMITS.bytes {
        Err("wire control argument too large".into())
    } else {
        Ok(())
    }
}
pub(crate) fn decode<M: Method>(bytes: Vec<u8>) -> M::Request
where
    M::Request: serde::de::DeserializeOwned,
{
    ic_wire::decode_request::<M>(&bytes).expect("bounded fixture Wire request")
}
pub(crate) fn encode<M: Method>(value: M::Response) -> Vec<u8> {
    ic_wire::encode_response::<M>(&value)
        .expect("bounded fixture Wire response")
        .into_bytes()
}
pub(crate) fn encode_result<M, T>(value: Result<T, String>) -> Vec<u8>
where
    M: Method<Response = Result<T, Refusal>>,
{
    encode::<M>(value.map_err(refusal))
}
