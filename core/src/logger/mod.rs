use std::collections::HashMap;

use crate::config::LoggerConfig;
use crate::models::{EventType, PortEvent, TrafficSample};

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug, Clone)]
struct PortTraffic {
    samples: Vec<TrafficSample>,
    max_samples: usize,
}

impl PortTraffic {
    fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::new(),
            max_samples,
        }
    }

    fn push(&mut self, connections: usize) {
        let ts = now_millis();
        self.samples.push(TrafficSample {
            connections,
            timestamp: ts,
        });
        if self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }
    }
}

pub struct PortLogger {
    events: Vec<PortEvent>,
    prev_ports: HashMap<u16, (u32, String)>,
    traffic: HashMap<u16, PortTraffic>,
    first_seen: HashMap<u16, u64>,
    max_events: usize,
}

impl PortLogger {
    pub fn new(config: &LoggerConfig) -> Self {
        Self {
            events: Vec::new(),
            prev_ports: HashMap::new(),
            traffic: HashMap::new(),
            first_seen: HashMap::new(),
            max_events: config.max_events,
        }
    }

    pub fn update(
        &mut self,
        ports: &[(u16, u32, String, Option<String>)],
        conn_counts: &HashMap<u16, usize>,
    ) -> Vec<PortEvent> {
        let ts = now_millis();
        let mut new_events = Vec::new();

        let mut current: HashMap<u16, (u32, String, Option<String>)> = HashMap::new();
        for (port, pid, name, fw) in ports {
            current.insert(*port, (*pid, name.clone(), fw.clone()));
        }

        for (port, (pid, name, fw)) in &current {
            if !self.prev_ports.contains_key(port) {
                let event = PortEvent {
                    port: *port,
                    pid: *pid,
                    process_name: name.clone(),
                    framework: fw.clone(),
                    event_type: EventType::Started,
                    timestamp: ts,
                };
                self.events.push(event.clone());
                new_events.push(event);
                self.first_seen.entry(*port).or_insert(ts);
            }
        }

        for (port, (pid, name)) in &self.prev_ports {
            if !current.contains_key(port) {
                let event = PortEvent {
                    port: *port,
                    pid: *pid,
                    process_name: name.clone(),
                    framework: None,
                    event_type: EventType::Stopped,
                    timestamp: ts,
                };
                self.events.push(event.clone());
                new_events.push(event);
            }
        }

        for port in current.keys() {
            let conns = conn_counts.get(port).copied().unwrap_or(0);
            self.traffic
                .entry(*port)
                .or_insert_with(|| PortTraffic::new(20))
                .push(conns);
        }

        self.prev_ports = current
            .iter()
            .map(|(port, (pid, name, _))| (*port, (*pid, name.clone())))
            .collect();

        if self.events.len() > self.max_events {
            self.events = self.events.split_off(self.events.len() - self.max_events);
        }

        new_events
    }

    pub fn get_events(&self) -> Vec<PortEvent> {
        let mut events = self.events.clone();
        events.reverse();
        events
    }

    pub fn get_traffic(&self, port: u16) -> Vec<TrafficSample> {
        self.traffic
            .get(&port)
            .map(|t| t.samples.clone())
            .unwrap_or_default()
    }

    pub fn get_all_traffic(&self) -> HashMap<u16, Vec<TrafficSample>> {
        self.traffic
            .iter()
            .map(|(port, t)| (*port, t.samples.clone()))
            .collect()
    }

    pub fn get_first_seen(&self, port: u16) -> Option<u64> {
        self.first_seen.get(&port).copied()
    }

    pub fn log_conflict(&mut self, port: u16, pid: u32, process_name: &str) -> PortEvent {
        let ts = now_millis();
        let event = PortEvent {
            port,
            pid,
            process_name: process_name.into(),
            framework: None,
            event_type: EventType::Conflict,
            timestamp: ts,
        };
        self.events.push(event.clone());
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LoggerConfig;
    use proptest::strategy::Strategy;

    fn default_config() -> LoggerConfig {
        LoggerConfig {
            max_events: 200,
            max_traffic_samples: 30,
        }
    }

    #[test]
    fn logger_tracks_new_ports() {
        let config = default_config();
        let mut logger = PortLogger::new(&config);
        let ports = vec![(3000u16, 1234u32, "node".into(), Some("React".into()))];
        let counts = HashMap::new();

        let events = logger.update(&ports, &counts);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::Started);
        assert_eq!(events[0].port, 3000);
    }

    #[test]
    fn logger_detects_stopped_ports() {
        let config = default_config();
        let mut logger = PortLogger::new(&config);
        let ports = vec![(3000u16, 1234u32, "node".into(), Some("React".into()))];
        let counts = HashMap::new();

        logger.update(&ports, &counts);

        let events = logger.update(&[], &counts);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::Stopped);
        assert_eq!(events[0].port, 3000);
    }

    #[test]
    fn logger_ignores_unchanged_ports() {
        let config = default_config();
        let mut logger = PortLogger::new(&config);
        let ports = vec![(3000u16, 1234u32, "node".into(), Some("React".into()))];
        let counts = HashMap::new();

        logger.update(&ports, &counts);
        let events = logger.update(&ports, &counts);
        assert!(events.is_empty());
    }

    #[test]
    fn logger_tracks_traffic() {
        let config = default_config();
        let mut logger = PortLogger::new(&config);
        let ports = vec![(3000u16, 1234u32, "node".into(), None)];
        let mut counts = HashMap::new();
        counts.insert(3000u16, 5usize);

        logger.update(&ports, &counts);
        let traffic = logger.get_traffic(3000);
        assert!(!traffic.is_empty());
        assert_eq!(traffic.last().unwrap().connections, 5);
    }

    #[test]
    fn logger_get_events_returns_reverse_chronological() {
        let config = default_config();
        let mut logger = PortLogger::new(&config);
        let ports1 = vec![(3000u16, 1234u32, "node".into(), None)];
        let ports2 = vec![
            (3000u16, 1234u32, "node".into(), None),
            (3001u16, 5678u32, "python".into(), None),
        ];

        logger.update(&ports1, &HashMap::new());
        logger.update(&ports2, &HashMap::new());

        let events = logger.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].port, 3001);
        assert_eq!(events[1].port, 3000);
    }

    #[test]
    fn logger_first_seen() {
        let config = default_config();
        let mut logger = PortLogger::new(&config);
        let ports = vec![(3000u16, 1234u32, "node".into(), None)];

        logger.update(&ports, &HashMap::new());
        assert!(logger.get_first_seen(3000).is_some());
        assert!(logger.get_first_seen(3001).is_none());
    }

    #[test]
    fn logger_log_conflict() {
        let config = default_config();
        let mut logger = PortLogger::new(&config);

        let event = logger.log_conflict(3000, 1234, "node");
        assert_eq!(event.event_type, EventType::Conflict);
        assert_eq!(event.port, 3000);

        let events = logger.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::Conflict);
    }

    #[test]
    fn logger_limits_events() {
        let config = LoggerConfig {
            max_events: 5,
            max_traffic_samples: 30,
        };
        let mut logger = PortLogger::new(&config);

        for i in 0..10u16 {
            let ports = vec![(3000 + i, i as u32, "test".into(), None)];
            logger.update(&ports, &HashMap::new());
        }

        let events = logger.get_events();
        assert!(events.len() <= 5);
    }

    #[test]
    fn logger_events_are_reversed() {
        let config = default_config();
        let mut logger = PortLogger::new(&config);

        logger.update(&[(3000, 1, "a".into(), None)], &HashMap::new());
        logger.update(
            &[(3000, 1, "a".into(), None), (3001, 2, "b".into(), None)],
            &HashMap::new(),
        );

        let events = logger.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].port, 3001);
        assert_eq!(events[1].port, 3000);
    }

    proptest::proptest! {
        #[test]
        fn logger_handles_any_port_range(ports in proptest::collection::vec(
            (0u16..65535u16, 1u32..99999u32, "[a-z]{1,10}".prop_map(String::from)),
            0..20,
        )) {
            let config = default_config();
            let mut logger = PortLogger::new(&config);
            let port_refs: Vec<(u16, u32, String, Option<String>)> = ports.iter()
                .map(|(p, pid, name)| (*p, *pid, name.clone(), None))
                .collect();
            let events = logger.update(&port_refs, &HashMap::new());
            assert!(events.len() <= port_refs.len());
            for e in &events {
                assert_eq!(e.event_type, EventType::Started);
            }
        }
    }
}
