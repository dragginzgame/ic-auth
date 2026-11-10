use super::*;
use ic_auth_protocol_types::CanonicalId;

#[test]
fn live_replacement_preserves_binding_pool_denial_and_retention() {
    let binding = LoginBinding {
        device_slot: 0,
        device_generation: 1,
        issuer: Principal::from_slice(&[1]),
        subject: Principal::from_slice(&[2]),
        started_at_ns: 1,
        deadline_ns: 10,
    };
    let change = |state, until_ns| {
        LoginRevocationChange(
            LoginData {
                audience: AudienceId {
                    canonical_network_id: CanonicalId::from_bytes([1; 32]),
                    fleet_id: CanonicalId::from_bytes([2; 32]),
                },
                caller: Principal::from_slice(&[3]),
                record: Record { state, until_ns },
            },
            None,
        )
    };
    let reserved = change(LoginRecordState::Reserved(binding), 11);
    let revoked = change(LoginRecordState::Revoked(Some(binding)), 12);
    assert_eq!(reserved.check_replacement(&reserved, 10), Ok(()));
    assert_eq!(revoked.check_replacement(&reserved, 10), Ok(()));
    for proposed in [
        change(LoginRecordState::Reserved(binding), 13),
        change(LoginRecordState::Revoked(Some(binding)), 11),
        change(LoginRecordState::Revoked(None), 13),
        change(
            LoginRecordState::Revoked(Some(LoginBinding {
                subject: Principal::from_slice(&[9]),
                ..binding
            })),
            13,
        ),
    ] {
        assert_eq!(
            proposed.check_replacement(&revoked, 11),
            Err(LoginRevocationError::InvalidPolicy)
        );
        assert_eq!(
            proposed.check_replacement(&revoked, 12),
            Err(LoginRevocationError::InvalidPolicy),
            "expiry never erases a retained slot fence"
        );
    }
    let unknown = change(LoginRecordState::Revoked(None), 12);
    assert_eq!(unknown.check_replacement(&unknown, 11), Ok(()));
    assert_eq!(
        reserved.check_replacement(&unknown, 11),
        Err(LoginRevocationError::InvalidPolicy)
    );
    let mut wrong_key = change(LoginRecordState::Reserved(binding), 13);
    wrong_key.0.caller = Principal::from_slice(&[4]);
    assert_eq!(
        wrong_key.check_replacement(&revoked, 12),
        Err(LoginRevocationError::InvalidPolicy)
    );
    wrong_key.0.caller = revoked.0.caller;
    wrong_key.0.audience.fleet_id = CanonicalId::from_bytes([3; 32]);
    assert_eq!(
        wrong_key.check_replacement(&revoked, 12),
        Err(LoginRevocationError::InvalidPolicy)
    );
}
