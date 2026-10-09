//! Generate data contracts from their Rust owner; this declares no endpoints.
use candid::{Decode, Encode, pretty::candid::compile, types::internal::TypeContainer};
use ic_auth_protocol_types::{
    AudienceId, AuthRequestMetadata, CanonicalId, DelegatedRoleGrant, DelegatedToken,
    DelegatedTokenGetRequest, DelegatedTokenPrepareRequest, DelegatedTokenPrepareResponse,
    DelegationAudience,
};

fn request_vector() -> DelegatedTokenPrepareRequest {
    DelegatedTokenPrepareRequest {
        metadata: Some(AuthRequestMetadata {
            request_id: [7; 32],
            ttl_ns: 1_000_000_000,
        }),
        aud: DelegationAudience::Fleet(AudienceId {
            canonical_network_id: CanonicalId::from_bytes([0xaa; 32]),
            fleet_id: CanonicalId::from_bytes([0xbb; 32]),
        }),
        grants: vec![DelegatedRoleGrant {
            target: "users".parse().expect("canonical role"),
            scopes: vec!["read".into()],
        }],
        ttl_ns: 9_007_199_254_740_993,
        ext: Some(vec![0, 255]),
    }
}

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => {
            let mut types = TypeContainer::new();
            types.add::<DelegatedToken>();
            types.add::<DelegatedTokenGetRequest>();
            types.add::<DelegatedTokenPrepareRequest>();
            types.add::<DelegatedTokenPrepareResponse>();
            print!("{}", compile(&types.env, &None));
        }
        [mode] if mode == "--request-vector" => {
            for byte in Encode!(&request_vector()).expect("encode fixture") {
                print!("{byte:02x}");
            }
            println!();
        }
        [mode, path] if mode == "--check-request" => {
            let bytes = std::fs::read(path).expect("read TS fixture");
            let actual = Decode!(&bytes, DelegatedTokenPrepareRequest).expect("decode TS fixture");
            assert_eq!(actual, request_vector());
        }
        _ => panic!("usage: export_candid [--request-vector | --check-request PATH]"),
    }
}
