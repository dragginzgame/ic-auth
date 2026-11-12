//! Real stable-memory probes for Auth bookkeeping. Crypto is deliberately outside
//! these samples; the existing certified/Ed25519 admission suites own proof cost.
use ic_auth::persistent::{
    StoreError, StoreMemory,
    issuer::*,
    receiver::{BudgetPolicy, LoginLimits, ReceiverConfig, ReceiverMemory, ReceiverStore},
};
use ic_auth::{
    portable::{PortableAuthority, ValidatedGrants},
    portable_signature::PreparedIssuerKey,
    session::{RequiredScopes, SessionContext, SessionLimits, SessionPolicy},
    session_request::SessionTtlPolicy,
};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableMechanism, Principal,
};
use ic_auth_qualification_contracts::{
    Refusal,
    persistent::{Command, Report},
};
use ic_storage::DefaultMemoryImpl;
use std::cell::RefCell;

thread_local! {
    static MEMORY: RefCell<Option<StoreMemory<DefaultMemoryImpl>>> = const { RefCell::new(None) };
    static ISSUER: RefCell<Option<IssuerStore<DefaultMemoryImpl>>> = const { RefCell::new(None) };
    static RECEIVER_MEMORY: RefCell<Option<ReceiverMemory<DefaultMemoryImpl>>> = const { RefCell::new(None) };
    static RECEIVER: RefCell<Option<ReceiverStore<DefaultMemoryImpl>>> = const { RefCell::new(None) };
}
fn config() -> IssuerConfig {
    IssuerConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: ic_platform_runtime::cdk::api::canister_self(),
        max_logins: 100_000,
        max_operations: 100_100,
        max_operations_per_account: 64,
        max_reserved_bytes: 512 * 1024 * 1024,
        max_devices_per_account: 8,
    }
}
pub(crate) fn bootstrap(
    (memory, receiver): (
        StoreMemory<DefaultMemoryImpl>,
        ReceiverMemory<DefaultMemoryImpl>,
    ),
    restore: bool,
) {
    if restore && memory.records.size() != 0 {
        let mut store = IssuerStore::open(memory, &config(), ic_platform_runtime::cdk::api::time())
            .expect("current persistent Auth issuer");
        store
            .restart(ic_platform_runtime::cdk::api::time())
            .expect("issuer boot fence");
        ISSUER.with(|state| *state.borrow_mut() = Some(store));
    } else {
        MEMORY.with(|state| *state.borrow_mut() = Some(memory));
    }
    if restore && receiver.state.records.size() != 0 {
        let store = ReceiverStore::open(
            receiver,
            &receiver_config(),
            ic_platform_runtime::cdk::api::time(),
        )
        .expect("persistent receiver restore");
        RECEIVER.with(|state| *state.borrow_mut() = Some(store));
    } else {
        RECEIVER_MEMORY.with(|state| *state.borrow_mut() = Some(receiver));
    }
}
fn receiver_config() -> ReceiverConfig {
    ReceiverConfig::new(
        config().audience,
        SessionLimits::new(100_000, 100_000, 32, 512 * 1024 * 1024).expect("receiver limits"),
        LoginLimits {
            admitted: 100_000,
            unknown: 16,
            maximum_ttl_ns: 90 * 86_400_000_000_000,
            devices_per_account: 8,
        },
        BudgetPolicy {
            burst: 1_000_000,
            refill_interval_ns: 1,
        },
        BudgetPolicy {
            burst: 16,
            refill_interval_ns: 1,
        },
    )
    .expect("receiver configuration")
}
fn receiver_key() -> PreparedIssuerKey {
    PreparedIssuerKey::new([
        0x19, 0x7f, 0x6b, 0x23, 0xe1, 0x6c, 0x85, 0x32, 0xc6, 0xab, 0xc8, 0x38, 0xfa, 0xcd, 0x5e,
        0xa7, 0x89, 0xbe, 0x0c, 0x76, 0xb2, 0x92, 0x03, 0x34, 0x03, 0x9b, 0xfa, 0x8b, 0x3d, 0x36,
        0x8d, 0x61,
    ])
    .expect("fixed qualification issuer key")
}
fn receiver_authority<'a>(
    audience: &'a AudienceId,
    grants: &'a ValidatedGrants,
    caller: Principal,
    now_ns: u64,
) -> PortableAuthority<'a> {
    PortableAuthority {
        audience,
        issuer: config().issuer,
        mechanism: PortableMechanism::Ed25519,
        key_binding_id: receiver_key().binding(),
        grants,
        caller,
        now_ns,
        authority_not_before_ns: 0,
        authority_expires_at_ns: u64::MAX,
        enrollment_expires_at_ns: u64::MAX,
        min_login_started_at_ns: 0,
    }
}
fn authority(certified: bool) -> IssuerAuthority {
    IssuerAuthority {
        generation: 1,
        mechanism: if certified {
            PortableMechanism::Certified
        } else {
            PortableMechanism::Ed25519
        },
        key_binding_id: [7; 32],
        expires_at_ns: u64::MAX,
    }
}
fn grants() -> Vec<DelegatedRoleGrant> {
    vec![DelegatedRoleGrant {
        target: "user".parse().expect("role"),
        scopes: vec!["read".parse().expect("scope")],
    }]
}
fn request(epoch: u64, n: u32) -> IssueRequest {
    let mut id = [1; 32];
    id[..4].copy_from_slice(&n.to_be_bytes());
    IssueRequest {
        epoch,
        operation_id: id,
        login_epoch: 1,
        login_id: id,
        browser_public_key: id,
        grants: grants(),
    }
}
fn account(n: u32) -> Principal {
    Principal::from_slice(&n.to_be_bytes())
}
fn policy() -> IssuerPolicy {
    IssuerPolicy {
        login_ttl_ns: 90 * 86_400_000_000_000,
        proof_ttl_ns: 60_000_000_000,
        receipt_timeout_ns: 60_000_000_000,
        reconciliation_retention_ns: 60_000_000_000,
    }
}
fn sample<T>(
    samples: &mut Vec<(String, u64)>,
    name: &str,
    run: impl FnOnce() -> Result<T, StoreError>,
) -> Result<T, StoreError> {
    let start = ic_platform_runtime::ic0::performance_counter(0);
    let value = run()?;
    samples.push((
        name.to_string(),
        ic_platform_runtime::ic0::performance_counter(0) - start,
    ));
    Ok(value)
}
#[ic_platform_runtime::cdk::update(
    guard = "crate::control::PersistentIssuer::guard",
    decode_with = "crate::control::PersistentIssuer::decode",
    encode_with = "crate::control::PersistentIssuer::encode"
)]
fn persistent_issuer(command: Command) -> Result<Report, Refusal> {
    super::controller()?;
    let now = ic_platform_runtime::cdk::api::time();
    if let Command::FillReceiver { records } = command {
        return fill_receiver(records);
    }
    if let Command::ProfileReceiver { browser_public_key } = command {
        return profile_receiver(browser_public_key, now);
    }
    if matches!(command, Command::Initialize) {
        let memory = MEMORY
            .with(|memory| memory.borrow_mut().take())
            .ok_or(Refusal::ReceiverUnavailable)?;
        let store =
            IssuerStore::create(memory, config(), now).map_err(|_| Refusal::RestorationMismatch)?;
        ISSUER.with(|state| *state.borrow_mut() = Some(store));
        let receiver = RECEIVER_MEMORY
            .with(|memory| memory.borrow_mut().take())
            .ok_or(Refusal::ReceiverUnavailable)?;
        let receiver = ReceiverStore::create(receiver, receiver_config(), 0, now)
            .map_err(|_| Refusal::Session)?;
        RECEIVER.with(|state| *state.borrow_mut() = Some(receiver));
    }
    ISSUER.with(|state| {
        let mut state = state.borrow_mut();
        let store = state.as_mut().ok_or(Refusal::ReceiverUnavailable)?;
        let mut samples = Vec::new();
        let result = (|| {
            match command {
                Command::Initialize => (),
                Command::Fill {
                    start,
                    count,
                    certified,
                } => {
                    if count == 0 || count > 64 || start == 0 || start.checked_add(count).is_none()
                    {
                        return Err(StoreError::Invalid);
                    }
                    let mut maximum = 0;
                    for n in start..start + count {
                        let begin = ic_platform_runtime::ic0::performance_counter(0);
                        store.prepare(
                            account(n),
                            request(store.epoch(), n),
                            policy(),
                            authority(certified),
                            now,
                        )?;
                        maximum =
                            maximum.max(ic_platform_runtime::ic0::performance_counter(0) - begin);
                    }
                    samples.push(("prepare_max_individual".into(), maximum));
                }
                Command::Profile {
                    account: n,
                    certified,
                } => {
                    let original = request(1, n);
                    sample(&mut samples, "retrieve", || {
                        store.retrieve(account(n), 1, original.operation_id)
                    })?;
                    let mut input = original.clone();
                    input.epoch = store.epoch();
                    input.operation_id[31] = 2;
                    let status = sample(&mut samples, "prepare", || {
                        store.prepare(
                            account(n),
                            input.clone(),
                            policy(),
                            authority(certified),
                            now,
                        )
                    })?;
                    let IssuerStatus::Pending(ticket) = status else {
                        return Err(StoreError::Conflict);
                    };
                    sample(&mut samples, "challenge", || {
                        store.challenge(&ticket, authority(certified), [4; 16], now)
                    })?;
                    sample(&mut samples, "approve", || {
                        store.approve(&ticket, authority(certified), [5; 32], now)
                    })?;
                    sample(&mut samples, "claims", || {
                        store.claims_for_signing(&ticket, authority(certified), now)
                    })?;
                    // Opaque signature bytes measure custody only; no proof or
                    // verified identity is produced by this controller fixture.
                    sample(&mut samples, "complete_custody", || {
                        store.complete(
                            &ticket,
                            authority(certified),
                            if certified { None } else { Some(vec![6; 64]) },
                            now,
                        )
                    })?;
                    let renew = RenewRequest {
                        account: account(n),
                        login_epoch: 1,
                        login_id: original.login_id,
                        sequence: 1,
                        grants: grants(),
                    };
                    sample(&mut samples, "renew", || {
                        store.renew(
                            ic_auth::portable::browser_principal(&original.browser_public_key),
                            renew,
                            authority(certified),
                            now,
                        )
                    })?;
                }
                Command::Prune { budget } => {
                    sample(&mut samples, "prune", || store.prune(now, budget))?;
                }
                Command::FillReceiver { .. } | Command::ProfileReceiver { .. } => {
                    unreachable!("receiver commands dispatched above")
                }
            }
            Ok::<_, StoreError>(())
        })();
        result.map_err(|_| Refusal::Session)?;
        let counts = store.occupancy();
        Ok(Report {
            logins: counts.logins,
            operations: counts.operations,
            epoch: store.epoch(),
            samples,
        })
    })
}

// These are controller-owned trusted backing records produced by native real
// admission. This isolates durable lookup cost from repeated proof verification;
// no ordinary endpoint accepts records or this injected caller boundary.
fn fill_receiver(
    records: Vec<ic_auth_qualification_contracts::persistent::StoredSession>,
) -> Result<Report, Refusal> {
    if records.is_empty() || records.len() > 32 {
        return Err(Refusal::InvalidConfiguration);
    }
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let store = state.as_mut().ok_or(Refusal::ReceiverUnavailable)?;
        for record in records {
            let session = ic_auth::session::Session::decode(record.session.as_slice())
                .map_err(|_| Refusal::RestorationMismatch)?;
            let replay = ic_auth::session::Replay::decode(record.replay.as_slice())
                .map_err(|_| Refusal::RestorationMismatch)?;
            let login = ic_auth::portable::LoginRevocationChange::decode(record.login.as_slice())
                .map_err(|_| Refusal::RestorationMismatch)?;
            ic_auth::session::SessionStore::admit(store, session, replay, &login)
                .map_err(|_| Refusal::Session)?;
        }
        let counts = store.occupancy();
        Ok(Report {
            logins: counts.admitted_logins as u64,
            operations: counts.sessions as u64,
            epoch: store.generation(),
            samples: Vec::new(),
        })
    })
}
fn profile_receiver(browser_public_key: [u8; 32], now_ns: u64) -> Result<Report, Refusal> {
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let store = state.as_mut().ok_or(Refusal::ReceiverUnavailable)?;
        let audience = config().audience;
        let grants = ValidatedGrants::new(grants()).expect("qualification grants");
        let context = SessionContext {
            authority: receiver_authority(
                &audience,
                &grants,
                ic_auth::portable::browser_principal(&browser_public_key),
                now_ns,
            ),
            max_ttl: SessionTtlPolicy::new(30, 60).expect("TTL").maximum(),
            role: &"user".parse().expect("role"),
            required_scopes: RequiredScopes::NONE,
        };
        let start = ic_platform_runtime::ic0::performance_counter(0);
        store
            .authenticate(
                &context,
                SessionPolicy {
                    generation: store.generation(),
                    enabled: true,
                    subject_admissible: true,
                },
                |session| session.subject(),
            )
            .map_err(|_| Refusal::Session)?;
        let instructions = ic_platform_runtime::ic0::performance_counter(0) - start;
        let start = ic_platform_runtime::ic0::performance_counter(0);
        ic_auth::session::SessionStore::read(store, |reader| {
            reader
                .session(context.authority.caller)
                .map(|session| session.map(|session| session.subject()))
        })
        .map_err(|_| Refusal::Session)?;
        let point = ic_platform_runtime::ic0::performance_counter(0) - start;

        Ok(Report {
            logins: store.occupancy().admitted_logins as u64,
            operations: store.occupancy().sessions as u64,
            epoch: store.generation(),
            samples: vec![
                ("authenticate".into(), instructions),
                ("point_read_decode".into(), point),
            ],
        })
    })
}
