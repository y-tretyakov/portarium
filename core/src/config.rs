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
