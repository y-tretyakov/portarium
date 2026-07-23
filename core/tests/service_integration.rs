use portarium_core::PortariumService;

#[test]
fn service_default_creation() {
    let service = PortariumService::default();
    assert_eq!(service.config().scanner.poll_interval_secs, 2);
    assert!(service.get_events().is_empty());
}

#[test]
fn service_log_and_retrieve_events() {
    let mut service = PortariumService::default();
    let event = service.log_conflict(3000, 1234, "node");
    assert_eq!(event.port, 3000);

    let events = service.get_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].port, 3000);
}

#[test]
fn service_multiple_logs() {
    let mut service = PortariumService::default();
    service.log_conflict(3000, 1234, "node");
    service.log_conflict(3001, 5678, "python");
    service.log_conflict(8080, 9012, "java");

    let events = service.get_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn service_traffic_empty_for_unknown() {
    let service = PortariumService::default();
    assert!(service.get_traffic(9999).is_empty());
    assert!(service.get_all_traffic().is_empty());
}
