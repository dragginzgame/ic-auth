use ic_auth::canonical::CanonicalAuthError;
use ic_auth::session_request::{SessionRequest, SessionRequestError};

#[test]
fn canonical_request_hash_preserves_exact_ttl_choice_and_scope_order() {
    let a = SessionRequest::new(vec!["write".into(), "read".into()], Some(30)).unwrap();
    let b = SessionRequest::new(vec!["read".into(), "write".into()], Some(30)).unwrap();
    assert_eq!(a.hash(), b.hash());
    assert_eq!(
        a.hash()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        "d99fc919563d8dba6db966bacbb256f2ba2437e64d12353106409934cc678551"
    );
    assert_ne!(
        SessionRequest::new(vec!["read".into()], None)
            .unwrap()
            .hash(),
        SessionRequest::new(vec!["read".into()], Some(1))
            .unwrap()
            .hash()
    );
    assert!(SessionRequest::new(vec![], None).is_err());
    assert!(SessionRequest::new(vec!["read".into(), "read".into()], None).is_err());
    assert!(SessionRequest::new(vec!["bad scope".into()], None).is_err());
    assert!(SessionRequest::new((0..17).map(|n| format!("s{n}")).collect(), None).is_err());
    assert!(SessionRequest::new(vec!["read".into()], Some(0)).is_err());
}

#[test]
fn absent_and_explicit_ttl_keep_independent_frozen_hashes() {
    let hex = |hash: [u8; 32]| hash.iter().map(|b| format!("{b:02x}")).collect::<String>();
    // These fixed values were independently framed outside the library encoder.
    // They cover the one-byte None/Some discriminant.
    for (ttl, expected) in [
        (
            None,
            "8c48d09f19a9ed9e142691355c3585d259ac590f462e50899ab7c6fc561136bd",
        ),
        (
            Some(30),
            "19deb7edab101b6faddbfc676618221c3145d32ba7e34c95c858e6d2d3aa5fc2",
        ),
        (
            Some(u64::MAX),
            "3ac3e779d1a2e076780f8a81878a471d2f58d9edf8d14c4c919d7aa834af285a",
        ),
    ] {
        let request = SessionRequest::new(vec!["read".into()], ttl).unwrap();
        assert_eq!(request.ttl_secs(), ttl);
        assert_eq!(hex(request.hash()), expected);
    }
}

#[test]
fn exact_scope_limits_accept_without_widening_or_changing_labels() {
    let mut scopes: Vec<String> = (0..16)
        .map(|n| format!("scope{n:02}{}", "a".repeat(57)))
        .collect();
    assert!(scopes.iter().all(|s| s.len() == 64));
    assert_eq!(scopes.iter().map(String::len).sum::<usize>(), 1024);
    let sorted = scopes.clone();
    scopes.reverse();
    let request = SessionRequest::new(scopes, None).unwrap();
    assert_eq!(request.scopes(), sorted);
    assert_eq!(request.clone(), request);
    let mut too_many = sorted.clone();
    too_many.push("read".into());
    assert_eq!(
        SessionRequest::new(too_many, None),
        Err(SessionRequestError::InvalidRequest)
    );
    let too_long = format!("{}a", sorted[0]);
    let mut over_total = sorted;
    over_total[0] = too_long.clone();
    assert_eq!(
        SessionRequest::new(over_total, None),
        Err(SessionRequestError::Canonical(
            CanonicalAuthError::InvalidScope {
                scope: too_long.clone()
            }
        ))
    );
    assert_eq!(
        SessionRequest::new(vec![too_long.clone()], None),
        Err(SessionRequestError::Canonical(
            CanonicalAuthError::InvalidScope { scope: too_long }
        ))
    );
}

#[test]
fn request_shape_and_canonical_failures_preserve_refusal_precedence() {
    use SessionRequestError as E;
    for scopes in [vec![], vec!["Bad".into(); 17]] {
        assert_eq!(SessionRequest::new(scopes, Some(0)), Err(E::InvalidRequest));
    }
    for (scopes, expected) in [
        (vec!["".into()], CanonicalAuthError::EmptyScope),
        (
            vec!["read".into(), "Bad".into(), "read".into()],
            CanonicalAuthError::InvalidScope {
                scope: "Bad".into(),
            },
        ),
        (
            vec!["read".into(), "é".into()],
            CanonicalAuthError::InvalidScope { scope: "é".into() },
        ),
    ] {
        assert_eq!(
            SessionRequest::new(scopes, Some(0)),
            Err(E::Canonical(expected))
        );
    }
    for ttl in [None, Some(1), Some(0)] {
        assert_eq!(
            SessionRequest::new(vec!["read".into(), "read".into()], ttl),
            Err(E::InvalidRequest)
        );
    }
    assert_eq!(
        SessionRequest::new(vec!["read".into()], Some(0)),
        Err(E::InvalidRequest)
    );
}

#[test]
fn request_identity_is_detached_and_each_scope_ttl_and_framing_matters() {
    let input = vec!["write".into(), "read".into()];
    let request = SessionRequest::new(input.clone(), None).unwrap();
    assert_eq!(input, ["write", "read"]);
    let original = request.hash();
    for scopes in [vec!["read".into()], vec!["read".into(), "writes".into()]] {
        assert_ne!(SessionRequest::new(scopes, None).unwrap().hash(), original);
    }
    let explicit = SessionRequest::new(input, Some(30)).unwrap();
    assert_ne!(explicit.hash(), original);
    assert_ne!(
        SessionRequest::new(vec!["readwrite".into()], None)
            .unwrap()
            .hash(),
        original
    );
    // Construction does not impose the engine's protected 1800-second ceiling.
    assert!(SessionRequest::new(vec!["read".into()], Some(u64::MAX)).is_ok());
}
