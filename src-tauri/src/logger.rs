use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single port event (started, stopped, etc.)
#[derive(Serialize, Clone, Debug)]
pub struct PortEvent {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub framework: Option<String>,
    pub event_type: String, // "started" | "stopped" | "conflict"
    pub timestamp: u64,     // unix millis
}

/// Snapshot of traffic for a single port at a point in time
#[derive(Serialize, Clone, Debug)]
pub struct TrafficSample {
    pub connections: usize,
    pub timestamp: u64,
}

/// Per-port traffic history
#[derive(Clone, Debug)]
struct PortTraffic {
    samples: Vec<TrafficSample>,
}

impl PortTraffic {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    fn push(&mut self, conns: usize) {
        let ts = now_millis();
        self.samples.push(TrafficSample {
            connections: conns,
            timestamp: ts,
        });
        // Keep last 30 samples (~60 seconds at 2s interval)
        if self.samples.len() > 30 {
            self.samples.remove(0);
        }
    }
}

/// Global store for port events and traffic
pub struct PortLogger {
    events: Vec<PortEvent>,
    prev_ports: HashMap<u16, (u32, String)>, // port -> (pid, process_name)
    traffic: HashMap<u16, PortTraffic>,
    first_seen: HashMap<u16, u64>,
}

impl PortLogger {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            prev_ports: HashMap::new(),
            traffic: HashMap::new(),
            first_seen: HashMap::new(),
        }
    }

    /// Call this every scan cycle with the current port list and connection counts.
    /// Returns any new events generated.
    pub fn update(
        &mut self,
        ports: &[(u16, u32, String, Option<String>)],
        conn_counts: &HashMap<u16, usize>,
    ) -> Vec<PortEvent> {
        let ts = now_millis();
        let mut new_events = Vec::new();

        // Build current port set
        let mut current: HashMap<u16, (u32, String, Option<String>)> = HashMap::new();
        for (port, pid, name, fw) in ports {
            current.insert(*port, (*pid, name.clone(), fw.clone()));
        }

        // Detect new ports (started)
        for (port, (pid, name, fw)) in &current {
            if !self.prev_ports.contains_key(port) {
                let event = PortEvent {
                    port: *port,
                    pid: *pid,
                    process_name: name.clone(),
                    framework: fw.clone(),
                    event_type: "started".into(),
                    timestamp: ts,
                };
                self.events.push(event.clone());
                new_events.push(event);
                self.first_seen.entry(*port).or_insert(ts);
            }
        }

        // Detect removed ports (stopped)
        for (port, (pid, name)) in &self.prev_ports {
            if !current.contains_key(port) {
                let event = PortEvent {
                    port: *port,
                    pid: *pid,
                    process_name: name.clone(),
                    framework: None,
                    event_type: "stopped".into(),
                    timestamp: ts,
                };
                self.events.push(event.clone());
                new_events.push(event);
            }
        }

        // Update traffic samples
        for port in current.keys() {
            let conns = conn_counts.get(port).copied().unwrap_or(0);
            self.traffic
                .entry(*port)
                .or_insert_with(PortTraffic::new)
                .push(conns);
        }

        // Update prev_ports
        self.prev_ports = current
            .iter()
            .map(|(port, (pid, name, _))| (*port, (*pid, name.clone())))
            .collect();

        // Trim events to last 200
        if self.events.len() > 200 {
            self.events = self.events.split_off(self.events.len() - 200);
        }

        new_events
    }

    pub fn get_events(&self) -> Vec<PortEvent> {
        // Return in reverse chronological
        let mut events = self.events.clone();
        events.reverse();
        events
    }

    #[expect(dead_code)]
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

    #[expect(dead_code)]
    pub fn get_first_seen(&self, port: u16) -> Option<u64> {
        self.first_seen.get(&port).copied()
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// Global singleton
lazy_static::lazy_static! {
    pub static ref LOGGER: Mutex<PortLogger> = Mutex::new(PortLogger::new());
}
