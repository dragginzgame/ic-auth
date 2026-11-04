use super::*;

#[test]
fn login_caller_requires_complete_self_authenticating_shape() {
    assert!(is_login_caller(browser_principal(&[42; 32])));
    for length in 0..=29 {
        for class in [0, 1, 2, 3, 4, 255] {
            let mut bytes = vec![7; length];
            if let Some(last) = bytes.last_mut() {
                *last = class;
            }
            assert_eq!(
                is_login_caller(Principal::from_slice(&bytes)),
                length == 29 && class == 2
            );
        }
    }
}
