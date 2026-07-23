use portarium_core::PortariumConfig;

#[test]
fn config_default_values() {
    let config = PortariumConfig::default();
    assert_eq!(config.scanner.poll_interval_secs, 2);
    assert!(config.scanner.enabled);
    assert_eq!(config.logger.max_events, 200);
    assert_eq!(config.logger.max_traffic_samples, 30);
    assert!(config.graph.enabled);
}

#[test]
fn config_json_roundtrip() {
    let config = PortariumConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    let deserialized: PortariumConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(
        config.scanner.poll_interval_secs,
        deserialized.scanner.poll_interval_secs
    );
    assert_eq!(config.logger.max_events, deserialized.logger.max_events);
    assert_eq!(config.graph.enabled, deserialized.graph.enabled);
}

#[test]
fn config_custom_values() {
    let json = r#"{
        "scanner": { "poll_interval_secs": 10, "enabled": false },
        "logger": { "max_events": 500, "max_traffic_samples": 100 },
        "graph": { "enabled": false }
    }"#;
    let config: PortariumConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.scanner.poll_interval_secs, 10);
    assert!(!config.scanner.enabled);
    assert_eq!(config.logger.max_events, 500);
    assert_eq!(config.logger.max_traffic_samples, 100);
    assert!(!config.graph.enabled);
}
