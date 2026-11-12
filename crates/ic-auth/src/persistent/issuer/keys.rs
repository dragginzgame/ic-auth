use super::model::{IssuerTicket, Login, LoginId};
use crate::persistent::records::Key;
use ic_auth_protocol_types::Principal;

pub(super) const HEADER: u8 = 0;
pub(super) const OPERATION: u8 = 1;
pub(super) const LOGIN: u8 = 2;
pub(super) const SLOT: u8 = 3;
pub(super) const BROWSER: u8 = 4;
pub(super) const RENEWAL: u8 = 5;
pub(super) const OPERATION_EXPIRY: u8 = 6;
pub(super) const LOGIN_EXPIRY: u8 = 7;
pub(super) const ACCOUNT_OPERATIONS: u8 = 8;
pub(super) fn account_operations(account: Principal) -> Key {
    Key::new(ACCOUNT_OPERATIONS).principal(account)
}

pub(super) fn operation(account: Principal, epoch: u64, id: &[u8; 32]) -> Key {
    Key::new(OPERATION)
        .principal(account)
        .number(epoch)
        .bytes(id)
}
pub(super) fn ticket(ticket: &IssuerTicket) -> Key {
    operation(ticket.account, ticket.epoch, &ticket.operation_id)
}
pub(super) fn login(id: &LoginId) -> Key {
    Key::new(LOGIN)
        .principal(id.account)
        .number(id.epoch)
        .bytes(&id.id)
}
pub(super) fn slot(account: Principal, slot: u8) -> Key {
    Key::new(SLOT).principal(account).bytes(&[slot])
}
pub(super) fn browser(account: Principal, browser: &[u8; 32]) -> Key {
    Key::new(BROWSER).principal(account).bytes(browser)
}
pub(super) fn renewal(id: &LoginId, sequence: u64) -> Key {
    Key::new(RENEWAL)
        .principal(id.account)
        .number(id.epoch)
        .bytes(&id.id)
        .number(sequence)
}
pub(super) fn operation_expiry(time: u64, ticket: &IssuerTicket) -> Key {
    Key::new(OPERATION_EXPIRY)
        .number(time)
        .bytes(&self::ticket(ticket).0[1..])
}
pub(super) fn login_expiry(login: &Login) -> Key {
    Key::new(LOGIN_EXPIRY)
        .number(login.deadline_ns)
        .bytes(&self::login(&login.id).0[1..])
}
