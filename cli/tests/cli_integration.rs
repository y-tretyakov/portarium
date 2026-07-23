use portarium_core::models::{
    EdgeType, EventType, GraphEdge, GraphNode, PortEvent, PortGraph, PortInfo, TrafficSample,
};

/// Helper to create a test PortInfo.
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
fn cli_port_info_creation() {
    let port = make_port(3000, 1234, "node");
    assert_eq!(port.port, 3000);
    assert_eq!(port.pid, 1234);
    assert_eq!(port.process_name, "node");
}

#[test]
fn cli_port_info_serialization() {
    let port = make_port(8080, 5678, "python");
    let json = serde_json::to_string(&port).unwrap();
    assert!(json.contains("8080"));
    assert!(json.contains("python"));
}

#[test]
fn cli_event_creation() {
    let event = PortEvent {
        port: 3000,
        pid: 1234,
        process_name: "node".into(),
        framework: Some("React".into()),
        event_type: EventType::Started,
        timestamp: 1000,
    };
    assert_eq!(event.event_type, EventType::Started);
    assert_eq!(event.framework, Some("React".into()));
}

#[test]
fn cli_graph_creation() {
    let graph = PortGraph {
        clusters: vec![],
        nodes: vec![GraphNode {
            id: "port:3000".into(),
            port: 3000,
            pid: 1234,
            process_name: "node".into(),
            project_name: Some("my-app".into()),
            cluster_id: Some("my-app".into()),
            framework: Some("React".into()),
            is_dev: true,
            connection_count: 5,
        }],
        edges: vec![GraphEdge {
            source: "port:3000".into(),
            target: "port:5432".into(),
            active: true,
            edge_type: EdgeType::TcpConnection,
        }],
    };
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn cli_traffic_sample_serialization() {
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
    let json = serde_json::to_string(&samples).unwrap();
    assert!(json.contains("1000"));
    assert!(json.contains("2000"));
}

#[test]
fn cli_events_filter_by_port() {
    let events = [
        PortEvent {
            port: 3000,
            pid: 1234,
            process_name: "node".into(),
            framework: None,
            event_type: EventType::Started,
            timestamp: 1000,
        },
        PortEvent {
            port: 3001,
            pid: 5678,
            process_name: "python".into(),
            framework: None,
            event_type: EventType::Started,
            timestamp: 2000,
        },
    ];

    let filtered: Vec<&PortEvent> = events.iter().filter(|e| e.port == 3000).collect();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].pid, 1234);
}
