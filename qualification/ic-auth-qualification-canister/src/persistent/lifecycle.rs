//! Controller-bound host effects around the real durable issuer. Signing is
//! supplied by the test's native key or this fixture's existing certification
//! owner. No shadow journal, copied renewal hash or private-key store is added.
use super::*;
use ic_auth_qualification_contracts::persistent::{IssuerAction, StoredCredential};

pub(super) fn issuer(
    action: IssuerAction,
    login: u32,
    browser_public_key: [u8; 32],
    key_binding_id: [u8; 32],
    certified: bool,
    sequence: u64,
    now: u64,
) -> Result<Report, Refusal> {
    ISSUER.with(|state| {
        let mut state = state.borrow_mut();
        let store = state.as_mut().ok_or(Refusal::ReceiverUnavailable)?;
        let mut original = request(1, login);
        original.browser_public_key = browser_public_key;
        let account = account(99);
        let browser = ic_auth::portable::browser_principal(&browser_public_key);
        let authority = IssuerAuthority {
            key_binding_id,
            ..authority(certified)
        };
        let input = RenewRequest {
            account,
            login_epoch: 1,
            login_id: original.login_id,
            sequence,
            grants: grants(),
        };
        let run = || -> Result<_, StoreError> {
            let status = if matches!(action, IssuerAction::Prepare) {
                if sequence == 0 {
                    store.prepare(
                        account,
                        original.clone(),
                        IssuerPolicy {
                            login_ttl_ns: 120_000_000_000,
                            proof_ttl_ns: 2_000_000_000,
                            ..policy()
                        },
                        authority,
                        now,
                    )?
                } else {
                    let selected = store.preflight_renewal(browser, &input)?;
                    // Fixture application authorization accepts exactly the
                    // retained account/grants/policy, before any reservation.
                    if selected.account != account
                        || selected.request.grants != grants()
                        || selected.policy.login_ttl_ns != 120_000_000_000
                    {
                        return Err(StoreError::Denied);
                    }
                    store.renew(browser, input.clone(), authority, now)?
                }
            } else if sequence == 0 {
                store
                    .retrieve(account, 1, original.operation_id)?
                    .ok_or(StoreError::Denied)?
            } else {
                store
                    .retrieve_renewal(browser, &input)?
                    .ok_or(StoreError::Denied)?
                    .status
            };
            let credential = match status {
                IssuerStatus::Completed(result) => StoredCredential {
                    claims: ic_wire::Bytes::new(result.claims).map_err(|_| StoreError::Invalid)?,
                    signature: result
                        .signature
                        .map(ic_wire::Bytes::new)
                        .transpose()
                        .map_err(|_| StoreError::Invalid)?,
                    completed: true,
                },
                IssuerStatus::Pending(ticket) => {
                    if matches!(action, IssuerAction::Prepare) {
                        store.challenge(&ticket, authority, [4; 16], now)?;
                        store.approve(&ticket, authority, [5; 32], now)?;
                    }
                    if let IssuerAction::Complete { signature } = action {
                        let result = store.complete(
                            &ticket,
                            authority,
                            signature.map(ic_wire::Bytes::into_vec),
                            now,
                        )?;
                        if certified {
                            let claims = ic_auth::portable::decode_claims(&result.claims)
                                .expect("retained canonical claims");
                            let digest =
                                ic_auth::portable::claims_hash(&claims).expect("claims digest");
                            // One IC transaction commits the result and the leaf
                            // through the fixture's existing certification owner.
                            crate::prepare(ic_auth_qualification_contracts::PrepareInput {
                                input: ic_auth_qualification_contracts::SignatureInput {
                                    seed: ic_wire::Bytes::new(b"qualification-identity".to_vec())
                                        .expect("seed"),
                                    domain: ic_wire::Bytes::new(
                                        ic_auth::portable_certified::CERTIFIED_SIGNATURE_DOMAIN
                                            .to_vec(),
                                    )
                                    .expect("domain"),
                                    message: ic_wire::Bytes::new(digest.to_vec()).expect("digest"),
                                },
                                retention_ns: 60_000_000_000,
                            })
                            .expect("atomic result/certification publication");
                        }
                        StoredCredential {
                            claims: ic_wire::Bytes::new(result.claims)
                                .map_err(|_| StoreError::Invalid)?,
                            signature: result
                                .signature
                                .map(ic_wire::Bytes::new)
                                .transpose()
                                .map_err(|_| StoreError::Invalid)?,
                            completed: true,
                        }
                    } else if matches!(action, IssuerAction::Recover) {
                        // Recovery may read an old pending identity, but must not
                        // use it to obtain fresh signing bytes after restart.
                        assert!(store.intent(&ticket).is_ok());
                        StoredCredential {
                            claims: ic_wire::Bytes::new(vec![]).expect("empty"),
                            signature: None,
                            completed: false,
                        }
                    } else {
                        let claims = store.claims_for_signing(&ticket, authority, now)?;
                        StoredCredential {
                            claims: ic_wire::Bytes::new(claims).map_err(|_| StoreError::Invalid)?,
                            signature: None,
                            completed: false,
                        }
                    }
                }
            };
            Ok(credential)
        };
        let credential = run().map_err(|_| Refusal::Session)?;
        Ok(Report {
            credential: Some(credential),
            logins: store.occupancy().logins,
            operations: store.occupancy().operations,
            epoch: store.epoch(),
            ..Report::default()
        })
    })
}
