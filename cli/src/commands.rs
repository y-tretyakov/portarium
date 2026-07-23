use std::thread;
use std::time::Duration;

use color_eyre::eyre::Result;
use portarium_core::models::{EventType, PortEvent, PortGraph, PortInfo, TrafficSample};
use portarium_core::PortariumService;

pub fn list(service: &mut PortariumService, json: bool) -> Result<()> {
    let ports = service.get_ports()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&ports)?);
    } else {
        print_ports_table(&ports);
    }
    Ok(())
}

pub fn watch(service: &mut PortariumService, interval: u64, json: bool) -> Result<()> {
    loop {
        service.scan_and_log()?;
        let ports = service.get_ports()?;
        if json {
            println!("{}", serde_json::to_string_pretty(&ports)?);
        } else {
            print!("\x1B[2J\x1B[1;1H");
            print_ports_table(&ports);
        }
        thread::sleep(Duration::from_secs(interval));
    }
}

pub fn events(service: &PortariumService, port: Option<u16>, json: bool) -> Result<()> {
    let all = service.get_events();
    let filtered: Vec<&PortEvent> = match port {
        Some(p) => all.iter().filter(|e| e.port == p).collect(),
        None => all.iter().collect(),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
    } else {
        print_events_table(&filtered);
    }
    Ok(())
}

pub fn graph(service: &mut PortariumService, json: bool) -> Result<()> {
    let g = service.get_graph()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&g)?);
    } else {
        print_graph_table(&g);
    }
    Ok(())
}

pub fn traffic(service: &PortariumService, port: u16, json: bool) -> Result<()> {
    let samples = service.get_traffic(port);
    if json {
        println!("{}", serde_json::to_string_pretty(&samples)?);
    } else {
        print_traffic_table(port, &samples);
    }
    Ok(())
}

pub fn kill(service: &PortariumService, pid: u32) -> Result<()> {
    service.kill(pid)?;
    println!("Process {} killed", pid);
    Ok(())
}

pub fn restart(service: &PortariumService, pid: u32, cmd: &str, cwd: &str) -> Result<()> {
    service.restart(pid, cmd, cwd)?;
    println!("Process {} restarted", pid);
    Ok(())
}

fn print_ports_table(ports: &[PortInfo]) {
    println!(
        "{:<8} {:<8} {:<24} {:<20} {:<8}",
        "PORT", "PID", "PROCESS", "PROJECT", "PROTOCOL"
    );
    println!("{}", "-".repeat(72));
    for p in ports {
        let project = match &p.project_name {
            Some(n) => n.as_str(),
            None => "-",
        };
        println!(
            "{:<8} {:<8} {:<24} {:<20} {:<8}",
            p.port, p.pid, p.process_name, project, p.protocol
        );
    }
}

fn print_events_table(events: &[&PortEvent]) {
    println!(
        "{:<8} {:<8} {:<24} {:<12} {:<16}",
        "PORT", "PID", "PROCESS", "EVENT", "TIMESTAMP"
    );
    println!("{}", "-".repeat(72));
    for e in events {
        let event_type = match e.event_type {
            EventType::Started => "started",
            EventType::Stopped => "stopped",
            EventType::Conflict => "conflict",
        };
        println!(
            "{:<8} {:<8} {:<24} {:<12} {:<16}",
            e.port, e.pid, e.process_name, event_type, e.timestamp
        );
    }
}

fn print_traffic_table(port: u16, samples: &[TrafficSample]) {
    println!("Traffic for port {}", port);
    println!("{:<16} {:<16}", "CONNECTIONS", "TIMESTAMP");
    println!("{}", "-".repeat(34));
    for s in samples {
        println!("{:<16} {:<16}", s.connections, s.timestamp);
    }
}

fn print_graph_table(g: &PortGraph) {
    println!("NODES");
    println!(
        "{:<16} {:<8} {:<8} {:<24} {:<16} {:<16} {:<8}",
        "ID", "PORT", "PID", "PROCESS", "PROJECT", "FRAMEWORK", "CONNS"
    );
    println!("{}", "-".repeat(100));
    for n in &g.nodes {
        let project = match &n.project_name {
            Some(p) => p.as_str(),
            None => "-",
        };
        let framework = match &n.framework {
            Some(f) => f.as_str(),
            None => "-",
        };
        println!(
            "{:<16} {:<8} {:<8} {:<24} {:<16} {:<16} {:<8}",
            n.id, n.port, n.pid, n.process_name, project, framework, n.connection_count
        );
    }
    if !g.edges.is_empty() {
        println!();
        println!("EDGES");
        println!("{:<24} {:<24} {:<8}", "SOURCE", "TARGET", "ACTIVE");
        println!("{}", "-".repeat(58));
        for e in &g.edges {
            println!("{:<24} {:<24} {:<8}", e.source, e.target, e.active);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portarium_core::models::{GraphEdge, GraphNode};

    fn make_port(port: u16, pid: u32, name: &str) -> PortInfo {
        PortInfo {
            port,
            pid,
            process_name: name.into(),
            project_path: None,
            project_name: None,
            protocol: "TCP".into(),
            start_cmd: None,
        }
    }

    #[test]
    fn print_ports_table_does_not_panic() {
        let ports = vec![PortInfo {
            port: 3000,
            pid: 1234,
            process_name: "node".into(),
            project_path: Some("/project".into()),
            project_name: Some("my-app".into()),
            protocol: "TCP".into(),
            start_cmd: Some("node server.js".into()),
        }];
        print_ports_table(&ports);
    }

    #[test]
    fn print_ports_table_empty() {
        let ports: Vec<PortInfo> = vec![];
        print_ports_table(&ports);
    }

    #[test]
    fn print_ports_table_multiple_ports() {
        let ports = vec![
            make_port(3000, 1234, "node"),
            make_port(8080, 5678, "python"),
            make_port(5432, 9012, "postgres"),
        ];
        print_ports_table(&ports);
    }

    #[test]
    fn print_events_table_empty() {
        let events: Vec<&PortEvent> = vec![];
        print_events_table(&events);
    }

    #[test]
    fn print_events_table_with_events() {
        let event = PortEvent {
            port: 3000,
            pid: 1234,
            process_name: "node".into(),
            framework: Some("React".into()),
            event_type: EventType::Started,
            timestamp: 1000,
        };
        let events = vec![&event];
        print_events_table(&events);
    }

    #[test]
    fn print_graph_table_empty() {
        let g = PortGraph {
            nodes: vec![],
            edges: vec![],
        };
        print_graph_table(&g);
    }

    #[test]
    fn print_graph_table_with_nodes() {
        let g = PortGraph {
            nodes: vec![GraphNode {
                id: "port:3000".into(),
                port: 3000,
                pid: 1234,
                process_name: "node".into(),
                project_name: Some("my-app".into()),
                framework: Some("React".into()),
                is_dev: true,
                connection_count: 5,
            }],
            edges: vec![GraphEdge {
                source: "port:3000".into(),
                target: "port:5432".into(),
                active: true,
            }],
        };
        print_graph_table(&g);
    }

    #[test]
    fn print_traffic_table_empty() {
        let samples: Vec<TrafficSample> = vec![];
        print_traffic_table(3000, &samples);
    }

    #[test]
    fn print_traffic_table_with_samples() {
        let samples = vec![
            TrafficSample {
                connections: 5,
                timestamp: 1000,
            },
            TrafficSample {
                connections: 10,
                timestamp: 2000,
            },
        ];
        print_traffic_table(3000, &samples);
    }

    #[test]
    fn print_ports_table_with_missing_project() {
        let ports = vec![make_port(3000, 1234, "node")];
        print_ports_table(&ports);
    }

    #[test]
    fn print_graph_table_no_edges() {
        let g = PortGraph {
            nodes: vec![GraphNode {
                id: "port:3000".into(),
                port: 3000,
                pid: 1234,
                process_name: "node".into(),
                project_name: None,
                framework: None,
                is_dev: false,
                connection_count: 0,
            }],
            edges: vec![],
        };
        print_graph_table(&g);
    }
}
