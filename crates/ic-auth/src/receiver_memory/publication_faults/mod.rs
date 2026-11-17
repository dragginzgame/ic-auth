//! Test-build-only interruptions between actual receiver publication mutations.
use std::cell::Cell;

thread_local! {
    static REMAINING: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(super) fn point() {
    REMAINING.with(|remaining| match remaining.get() {
        Some(0) => std::panic::panic_any("auth-publication-fault"),
        Some(n) => remaining.set(Some(n - 1)),
        None => {}
    });
}

#[cfg(feature = "ed25519-verification")]
mod tests;
