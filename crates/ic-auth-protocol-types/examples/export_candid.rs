//! Generate data contracts from their Rust owner; this declares no endpoints.
use candid::{Decode, Encode, pretty::candid::compile, types::internal::TypeContainer};
use ic_auth_protocol_types::{
    AudienceId, AuthRequestMetadata, CanonicalId, DelegatedRoleGrant, DelegatedToken,
    DelegatedTokenGetRequest, DelegatedTokenPrepareRequest, DelegatedTokenPrepareResponse,
    DelegationAudience, LogoutRequest, LogoutResult, LogoutUnavailable, PortableCredential,
    PortableProof,
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

// Deliberately opaque transport bytes, not a valid signed credential.
fn credential_vector() -> PortableCredential {
    PortableCredential {
        claims: vec![0, 1, 127, 128, 255],
        proof: PortableProof::Ed25519 {
            signature: (0..64).collect(),
        },
    }
}

fn logout_vector() -> Vec<LogoutResult> {
    vec![
        LogoutResult::Revoked {
            deny_until_ns: 9_007_199_254_740_993,
        },
        LogoutResult::InvalidCaller,
        LogoutResult::Unavailable(LogoutUnavailable::Capacity),
        LogoutResult::Unavailable(LogoutUnavailable::Storage),
        LogoutResult::Unavailable(LogoutUnavailable::Clock),
        LogoutResult::Unavailable(LogoutUnavailable::ResetPending),
    ]
}

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => {
            let mut types = TypeContainer::new();
            types.add::<DelegatedToken>();
            types.add::<PortableCredential>();
            // Candid elides names for empty records; retain the public request alias.
            let logout_request = types.add::<LogoutRequest>();
            types.env.0.insert("LogoutRequest".into(), logout_request);
            types.add::<LogoutResult>();
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
        [mode] if mode == "--credential-vector" => {
            for byte in Encode!(&credential_vector()).expect("encode credential fixture") {
                print!("{byte:02x}");
            }
            println!();
        }
        [mode, path] if mode == "--check-credential" => {
            let bytes = std::fs::read(path).expect("read TS credential fixture");
            let actual = Decode!(&bytes, PortableCredential).expect("decode TS credential fixture");
            assert_eq!(actual, credential_vector());
        }
        [mode] if mode == "--logout-vector" => {
            for byte in Encode!(&logout_vector()).expect("encode logout fixture") {
                print!("{byte:02x}");
            }
            println!();
        }
        [mode, path] if mode == "--check-logout" => {
            let bytes = std::fs::read(path).expect("read TS logout fixture");
            assert_eq!(
                Decode!(&bytes, Vec<LogoutResult>).expect("decode TS logout fixture"),
                logout_vector()
            );
        }
        _ => panic!(
            "usage: export_candid [--request-vector | --check-request PATH | --credential-vector | --check-credential PATH | --logout-vector | --check-logout PATH]"
        ),
    }
}
