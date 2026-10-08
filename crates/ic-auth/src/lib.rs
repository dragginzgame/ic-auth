//! Pure application-token encoding. No verification or runtime effects yet.
//!
//! Hashing untrusted material does not authenticate it. Applications must not
//! admit tokens using this package until complete proof verification is supplied.

pub mod canonical;
