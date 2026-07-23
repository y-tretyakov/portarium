use std::collections::HashMap;

use crate::config::PortariumConfig;
use crate::error::Result;
use crate::frameworks;
use crate::graph_builder::GraphBuilder;
use crate::logger::PortLogger;
use crate::models::{PortEvent, PortGraph, PortInfo, TrafficSample};
use crate::scanner::PortScanner;

pub struct PortariumService {
    scanner: PortScanner,
    logger: PortLogger,
    config: PortariumConfig,
}

impl PortariumService {
    pub fn new(config: PortariumConfig) -> Self {
        Self {
            scanner: PortScanner::new(),
            logger: PortLogger::new(&config.logger),
            config,
        }
    }

    pub fn scan_and_log(&mut self) -> Result<Vec<PortEvent>> {
        let ports = self.scanner.scan()?;

        let port_tuples: Vec<(u16, u32, String, Option<String>)> = ports
            .iter()
            .map(|p| {
                let fw = frameworks::get_framework(p.port);
                (p.port, p.pid, p.process_name.clone(), fw)
            })
            .collect();

        let graph = GraphBuilder::build(&ports);
        let mut conn_counts = HashMap::new();
        for node in &graph.nodes {
            conn_counts.insert(node.port, node.connection_count);
        }

        Ok(self.logger.update(&port_tuples, &conn_counts))
    }

    pub fn get_ports(&mut self) -> Result<Vec<PortInfo>> {
        self.scanner.scan()
    }

    pub fn get_events(&self) -> Vec<PortEvent> {
        self.logger.get_events()
    }

    pub fn get_traffic(&self, port: u16) -> Vec<TrafficSample> {
        self.logger.get_traffic(port)
    }

    pub fn get_all_traffic(&self) -> HashMap<u16, Vec<TrafficSample>> {
        self.logger.get_all_traffic()
    }

    pub fn get_graph(&mut self) -> Result<PortGraph> {
        let ports = self.scanner.scan()?;
        Ok(GraphBuilder::build(&ports))
    }

    pub fn kill(&self, pid: u32) -> Result<()> {
        self.scanner.kill(pid)
    }

    pub fn restart(&self, pid: u32, cmd: &str, cwd: &str) -> Result<()> {
        self.scanner.restart(pid, cmd, cwd)
    }

    pub fn config(&self) -> &PortariumConfig {
        &self.config
    }

    pub fn log_conflict(&mut self, port: u16, pid: u32, process_name: &str) -> PortEvent {
        self.logger.log_conflict(port, pid, process_name)
    }
}

impl Default for PortariumService {
    fn default() -> Self {
        Self::new(PortariumConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_default_creates_ok() {
        let service = PortariumService::default();
        assert_eq!(service.config().scanner.poll_interval_secs, 2);
    }

    #[test]
    fn service_get_events_empty_initially() {
        let service = PortariumService::default();
        assert!(service.get_events().is_empty());
    }

    #[test]
    fn service_get_traffic_empty_for_unknown() {
        let service = PortariumService::default();
        assert!(service.get_traffic(3000).is_empty());
    }

    #[test]
    fn service_get_all_traffic_empty_initially() {
        let service = PortariumService::default();
        assert!(service.get_all_traffic().is_empty());
    }

    #[test]
    fn service_log_conflict() {
        let mut service = PortariumService::default();
        let event = service.log_conflict(3000, 1234, "node");
        assert_eq!(event.port, 3000);

        let events = service.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].port, 3000);
    }

    #[test]
    fn service_log_multiple_conflicts() {
        let mut service = PortariumService::default();
        service.log_conflict(3000, 1234, "node");
        service.log_conflict(3001, 5678, "python");
        let events = service.get_events();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn service_config_reflects_constructor() {
        let mut config = PortariumConfig::default();
        config.scanner.poll_interval_secs = 10;
        let service = PortariumService::new(config);
        assert_eq!(service.config().scanner.poll_interval_secs, 10);
    }

    #[test]
    fn service_get_events_returns_newest_first() {
        let mut service = PortariumService::default();
        service.log_conflict(3000, 1234, "node");
        service.log_conflict(3001, 5678, "python");
        let events = service.get_events();
        assert_eq!(events[0].port, 3001);
        assert_eq!(events[1].port, 3000);
    }
}
