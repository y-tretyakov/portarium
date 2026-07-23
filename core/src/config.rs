use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub poll_interval_secs: u64,
    pub enabled: bool,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 2,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    pub max_events: usize,
    pub max_traffic_samples: usize,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            max_events: 200,
            max_traffic_samples: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub enabled: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortariumConfig {
    pub scanner: ScannerConfig,
    pub logger: LoggerConfig,
    pub graph: GraphConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_config_default() {
        let cfg = ScannerConfig::default();
        assert_eq!(cfg.poll_interval_secs, 2);
        assert!(cfg.enabled);
    }

    #[test]
    fn scanner_config_serialization() {
        let cfg = ScannerConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(json.contains("poll_interval_secs"));
        assert!(json.contains("enabled"));
    }

    #[test]
    fn scanner_config_deserialization() {
        let json = r#"{"poll_interval_secs": 5, "enabled": false}"#;
        let cfg: ScannerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.poll_interval_secs, 5);
        assert!(!cfg.enabled);
    }

    #[test]
    fn logger_config_default() {
        let cfg = LoggerConfig::default();
        assert_eq!(cfg.max_events, 200);
        assert_eq!(cfg.max_traffic_samples, 30);
    }

    #[test]
    fn graph_config_default() {
        let cfg = GraphConfig::default();
        assert!(cfg.enabled);
    }

    #[test]
    fn portarium_config_default() {
        let cfg = PortariumConfig::default();
        assert_eq!(cfg.scanner.poll_interval_secs, 2);
        assert_eq!(cfg.logger.max_events, 200);
        assert!(cfg.graph.enabled);
    }

    #[test]
    fn portarium_config_serialization_roundtrip() {
        let cfg = PortariumConfig::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let deserialized: PortariumConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.scanner.poll_interval_secs,
            cfg.scanner.poll_interval_secs
        );
        assert_eq!(deserialized.logger.max_events, cfg.logger.max_events);
        assert_eq!(deserialized.graph.enabled, cfg.graph.enabled);
    }
}
