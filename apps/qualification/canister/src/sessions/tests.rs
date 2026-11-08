use super::*;

#[test]
fn unavailable_receiver_refuses_before_any_operation_with_owned_categories() {
    for snapshot in [Snapshot::Closed, Snapshot::Unconfigured, Snapshot::Closed] {
        let unconfigured = matches!(&snapshot, Snapshot::Unconfigured);
        restore(snapshot);
        assert_eq!(matches!(capture(), Snapshot::Unconfigured), unconfigured);
        assert_eq!(matches!(capture(), Snapshot::Closed), !unconfigured);
        let refusal: Result<(), AdmissionRefusal> = with_receiver(|_| panic!("admission ran"));
        assert_eq!(refusal, Err(AdmissionRefusal::Unavailable));
        let refusal: Result<(), SessionReadRefusal> = with_receiver(|_| panic!("read ran"));
        assert_eq!(refusal, Err(SessionReadRefusal::Unavailable));
        let refusal: Result<(), Refusal> = with_receiver(|_| panic!("control ran"));
        assert_eq!(refusal, Err(Refusal::ReceiverUnavailable));
    }
}
