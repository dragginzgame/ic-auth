use super::*;
pub(super) const HEADER: u8 = 0;
pub(super) const REPLAY: u8 = 2;
pub(super) const LOGIN: u8 = 3;
pub(super) const SESSION_SLOT: u8 = 4;
pub(super) const LOGIN_SLOT: u8 = 5;
pub(super) const REPLAY_COUNT: u8 = 6;
pub(super) const SESSION_EXPIRY: u8 = 7;
pub(super) const REPLAY_EXPIRY: u8 = 8;
pub(super) const LOGIN_EXPIRY: u8 = 9;
pub(super) fn replay(fingerprint: &[u8; 32]) -> Key {
    Key::new(REPLAY).bytes(fingerprint)
}
pub(super) fn login(caller: Principal) -> Key {
    Key::new(LOGIN).principal(caller)
}
pub(super) fn slot(kind: u8, issuer: Principal, subject: Principal, slot: u8) -> Key {
    Key::new(kind)
        .principal(issuer)
        .principal(subject)
        .bytes(&[slot])
}
pub(super) fn replay_count(account: (Principal, Principal)) -> Key {
    Key::new(REPLAY_COUNT)
        .principal(account.0)
        .principal(account.1)
}
pub(super) fn session_expiry(session: &Session) -> Key {
    Key::new(SESSION_EXPIRY)
        .number(session.expires_at_ns())
        .principal(session.caller())
}
pub(super) fn replay_expiry(replay: &Replay) -> Key {
    Key::new(REPLAY_EXPIRY)
        .number(replay.remove_at_ns())
        .bytes(&replay.fingerprint())
}
pub(super) fn login_expiry(change: &LoginRevocationChange) -> Key {
    Key::new(LOGIN_EXPIRY)
        .number(change.until_ns())
        .principal(change.caller())
}
