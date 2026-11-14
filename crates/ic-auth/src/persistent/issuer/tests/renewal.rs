use super::*;

#[test]
fn fresh_renewals_share_durable_budget_but_recovery_does_not_charge() {
    let backing = Backing::new(VectorMemory::default());
    let mut cfg = config();
    cfg.issuance_burst = 2;
    cfg.issuance_refill_interval_ns = 10;
    let mut store = IssuerStore::create(memory(backing.clone()), cfg.clone(), 0).unwrap();
    for n in 1..=2 {
        let ticket = pending(
            store
                .prepare(account(n), request(1, n), policy(), authority(), 1)
                .unwrap(),
        );
        finish(&mut store, &ticket, 1);
    }
    let browser = crate::portable::browser_principal(&request(1, 1).browser_public_key);
    assert_eq!(
        store.prepare(account(3), request(1, 3), policy(), authority(), 1),
        Err(StoreError::Throttled)
    );
    assert!(matches!(
        store.prepare(account(1), request(1, 1), policy(), authority(), 1),
        Ok(IssuerStatus::Completed(_))
    ));
    assert_eq!(
        store.renew(browser, input(), authority(), 1),
        Err(StoreError::Throttled)
    );
    // Invalid callers cannot consume the newly refilled credit.
    assert_eq!(
        store.renew(Principal::anonymous(), input(), authority(), 10),
        Err(StoreError::Denied)
    );
    let ticket = pending(store.renew(browser, input(), authority(), 10).unwrap());
    assert_eq!(
        store.renew(browser, input(), authority(), 10),
        Ok(IssuerStatus::Pending(ticket.clone()))
    );
    let completed = finish(&mut store, &ticket, 10);
    let mut other = input();
    other.account = account(2);
    other.login_id = request(1, 2).login_id;
    let peer = crate::portable::browser_principal(&request(1, 2).browser_public_key);
    drop(store);
    let mut store = IssuerStore::open(memory(backing.clone()), &cfg, 10).unwrap();
    store.restart(10).unwrap();
    backing.reset();
    assert_eq!(
        store.renew(peer, other.clone(), authority(), 10),
        Err(StoreError::Throttled)
    );
    assert_eq!(backing.counts().writes, 0);
    assert_eq!(
        store.renew(browser, input(), authority(), 10),
        Ok(IssuerStatus::Completed(completed))
    );
    assert!(store.retrieve_renewal(browser, &input()).unwrap().is_some());
    assert_eq!(backing.counts().writes, 0);
    assert!(store.renew(peer, other, authority(), 20).is_ok());
    assert_eq!(
        store.renew(browser, input(), authority(), 19),
        Err(StoreError::Clock)
    );
}

#[test]
fn issuer_rejects_retired_schema_and_zero_renewal_policy() {
    let backing = Backing::new(VectorMemory::default());
    let mut cfg = config();
    cfg.issuance_burst = 0;
    assert!(matches!(
        IssuerStore::create(memory(backing.clone()), cfg, 0),
        Err(StoreError::Invalid)
    ));
    let mut store = IssuerStore::create(memory(backing.clone()), config(), 0).unwrap();
    let mut header = store.header.clone();
    header.schema = 0x41555448495301;
    store.publish(header, Writes::new()).unwrap();
    drop(store);
    assert!(matches!(
        IssuerStore::open(memory(backing), &config(), 0),
        Err(StoreError::Corrupt)
    ));
}

#[test]
fn later_capacity_refusal_does_not_refund_renewal_work() {
    let backing = Backing::new(VectorMemory::default());
    let mut cfg = config();
    cfg.max_operations = 1;
    cfg.max_operations_per_account = 1;
    cfg.issuance_burst = 1;
    cfg.issuance_refill_interval_ns = 10;
    let mut store = IssuerStore::create(memory(backing.clone()), cfg.clone(), 0).unwrap();
    let ticket = pending(
        store
            .prepare(account(1), request(1, 1), policy(), authority(), 1)
            .unwrap(),
    );
    finish(&mut store, &ticket, 1);
    let browser = crate::portable::browser_principal(&request(1, 1).browser_public_key);
    assert_eq!(
        store.renew(browser, input(), authority(), 10),
        Err(StoreError::Capacity)
    );
    assert_eq!(store.occupancy().operations, 1);
    drop(store);
    let mut store = IssuerStore::open(memory(backing), &cfg, 10).unwrap();
    assert_eq!(
        store.renew(browser, input(), authority(), 10),
        Err(StoreError::Throttled)
    );
    assert_eq!(store.retrieve_renewal(browser, &input()), Ok(None));
}

fn input() -> RenewRequest {
    RenewRequest {
        account: account(1),
        login_epoch: 1,
        login_id: request(1, 1).login_id,
        sequence: 1,
        grants: grants(),
    }
}

#[test]
fn renewal_preflight_and_recovery_preserve_originals_without_writes() {
    for mechanism in [PortableMechanism::Ed25519, PortableMechanism::Certified] {
        let backing = Backing::new(VectorMemory::default());
        let mut store = IssuerStore::create(memory(backing.clone()), config(), 1).unwrap();
        let authority = IssuerAuthority {
            mechanism,
            ..authority()
        };
        let finish = |store: &mut IssuerStore<Backing>, ticket: &IssuerTicket, now| {
            store.challenge(ticket, authority, [4; 16], now).unwrap();
            store.approve(ticket, authority, [5; 32], now).unwrap();
            store
                .complete(
                    ticket,
                    authority,
                    (mechanism == PortableMechanism::Ed25519).then(|| vec![6; 64]),
                    now,
                )
                .unwrap()
        };
        let original = request(1, 1);
        let first = pending(
            store
                .prepare(account(1), original.clone(), policy(), authority, 1)
                .unwrap(),
        );
        finish(&mut store, &first, 1);
        let browser = crate::portable::browser_principal(&original.browser_public_key);
        let input = input();
        let counts = store.occupancy();
        backing.reset();
        let selected = store.preflight_renewal(browser, &input).unwrap();
        assert_eq!(selected.account, account(1));
        assert_eq!(selected.policy, policy());
        assert_eq!(
            selected.request.browser_public_key,
            original.browser_public_key
        );
        assert_eq!(selected.request.grants, original.grants);
        assert_eq!(store.retrieve_renewal(browser, &input), Ok(None));
        for wrong in [Principal::anonymous(), account(9)] {
            assert_eq!(
                store.preflight_renewal(wrong, &input),
                Err(StoreError::Denied)
            );
            assert_eq!(
                store.retrieve_renewal(wrong, &input),
                Err(StoreError::Denied)
            );
        }
        let mut changed = input.clone();
        changed.grants[0].scopes = vec!["write".parse().unwrap()];
        assert_eq!(
            store.retrieve_renewal(browser, &changed),
            Err(StoreError::Denied)
        );
        changed = input.clone();
        changed.account = account(2);
        assert_eq!(
            store.retrieve_renewal(browser, &changed),
            Err(StoreError::Denied)
        );
        changed = input.clone();
        changed.sequence = 0;
        assert_eq!(
            store.preflight_renewal(browser, &changed),
            Err(StoreError::Invalid)
        );
        changed.sequence = 2;
        assert_eq!(
            store.preflight_renewal(browser, &changed),
            Err(StoreError::Conflict)
        );
        assert_eq!(store.occupancy(), counts);
        assert_eq!(store.observed_at_ns(), 1);
        assert_eq!((backing.counts().writes, backing.counts().grows), (0, 0));

        let ticket = pending(store.renew(browser, input.clone(), authority, 2).unwrap());
        assert_eq!(store.intent(&ticket).unwrap().request, selected.request);
        backing.reset();
        let pending = store.retrieve_renewal(browser, &input).unwrap().unwrap();
        assert_eq!(pending.status, IssuerStatus::Pending(ticket.clone()));
        assert_eq!(pending.intent, store.intent(&ticket).unwrap());
        assert_eq!(store.preflight_renewal(browser, &input), Ok(selected));
        assert_eq!(backing.counts().writes, 0);
        let completed = finish(&mut store, &ticket, 3);
        drop(store);
        let mut store = IssuerStore::open(memory(backing.clone()), &config(), 2000).unwrap();
        store.restart(2000).unwrap();
        store.revoke(account(1), 1, input.login_id, 2000).unwrap();
        backing.reset();
        let recovered = store.retrieve_renewal(browser, &input).unwrap().unwrap();
        assert_eq!(recovered.status, IssuerStatus::Completed(completed));
        assert_eq!(recovered.intent.request, pending.intent.request);
        assert_eq!(
            store.preflight_renewal(browser, &input),
            Err(StoreError::Denied)
        );
        assert_eq!(store.observed_at_ns(), 2000);
        assert_eq!((backing.counts().writes, backing.counts().grows), (0, 0));
        assert_eq!(
            store.renew(browser, input, authority, 2000),
            Err(StoreError::Denied)
        );
    }
}

#[test]
fn renewal_preflight_does_not_authorize_after_expiry_or_recreate_pruned_sequences() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing.clone()), config(), 1).unwrap();
    let first = pending(
        store
            .prepare(account(1), request(1, 1), policy(), authority(), 1)
            .unwrap(),
    );
    finish(&mut store, &first, 1);
    let browser = crate::portable::browser_principal(&request(1, 1).browser_public_key);
    let input = input();
    let selected = store.preflight_renewal(browser, &input).unwrap();
    let ticket = pending(store.renew(browser, input.clone(), authority(), 2).unwrap());
    finish(&mut store, &ticket, 2);
    store.prune(203, 32).unwrap();
    backing.reset();
    assert_eq!(store.retrieve_renewal(browser, &input), Ok(None));
    assert_eq!(
        store.preflight_renewal(browser, &input),
        Err(StoreError::EpochClosed)
    );
    assert_eq!(backing.counts().writes, 0);
    assert_eq!(
        store.renew(browser, input.clone(), authority(), 203),
        Err(StoreError::EpochClosed)
    );
    let next = RenewRequest {
        sequence: 2,
        ..input
    };
    assert!(store.preflight_renewal(browser, &next).is_ok());
    assert_eq!(
        store.renew(browser, next, authority(), 1001),
        Err(StoreError::Expired)
    );
    assert_eq!(selected.policy, policy());
}

#[test]
fn renewal_lookup_rejects_corrupt_index_binding_without_writing() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing.clone()), config(), 1).unwrap();
    let original = request(1, 1);
    let first = pending(
        store
            .prepare(account(1), original.clone(), policy(), authority(), 1)
            .unwrap(),
    );
    finish(&mut store, &first, 1);
    let browser = crate::portable::browser_principal(&original.browser_public_key);
    let input = input();
    let ticket = pending(store.renew(browser, input.clone(), authority(), 2).unwrap());
    let id = LoginId {
        account: account(1),
        epoch: 1,
        id: original.login_id,
    };
    for corrupt in [
        first,
        IssuerTicket {
            boot: ticket.boot + 1,
            ..ticket
        },
    ] {
        let mut writes = Writes::new();
        writes.put(keys::renewal(&id, 1), &corrupt).unwrap();
        store.records.commit(writes);
        backing.reset();
        assert_eq!(
            store.retrieve_renewal(browser, &input),
            Err(StoreError::Corrupt)
        );
        assert_eq!(
            store.preflight_renewal(browser, &input),
            Err(StoreError::Corrupt)
        );
        assert_eq!(backing.counts().writes, 0);
    }
}
