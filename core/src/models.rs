use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub project_path: Option<String>,
    pub project_name: Option<String>,
    pub protocol: String,
    pub start_cmd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Started,
    Stopped,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortEvent {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub framework: Option<String>,
    pub event_type: EventType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSample {
    pub connections: usize,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub project_name: Option<String>,
    pub cluster_id: Option<String>,
    pub framework: Option<String>,
    pub is_dev: bool,
    pub connection_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    #[default]
    TcpConnection,
    ProjectPeer,
    OrchestrationPeer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub active: bool,
    #[serde(default)]
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortCluster {
    pub id: String,
    pub label: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    #[serde(default)]
    pub clusters: Vec<PortCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Framework {
    pub port: u16,
    pub name: String,
    pub is_dev: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    TCP,
    UDP,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::TCP => "TCP",
            Protocol::UDP => "UDP",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_as_str_tcp() {
        assert_eq!(Protocol::TCP.as_str(), "TCP");
    }

    #[test]
    fn protocol_as_str_udp() {
        assert_eq!(Protocol::UDP.as_str(), "UDP");
    }

    #[test]
    fn protocol_equality() {
        assert_eq!(Protocol::TCP, Protocol::TCP);
        assert_ne!(Protocol::TCP, Protocol::UDP);
    }

    #[test]
    fn protocol_serialization() {
        let json = serde_json::to_string(&Protocol::TCP).unwrap();
        assert_eq!(json, "\"tcp\"");
    }

    #[test]
    fn protocol_deserialization() {
        let p: Protocol = serde_json::from_str("\"udp\"").unwrap();
        assert_eq!(p, Protocol::UDP);
    }

    #[test]
    fn port_info_serialization_roundtrip() {
        let info = PortInfo {
            port: 3000,
            pid: 1234,
            process_name: "node".into(),
            project_path: Some("/project".into()),
            project_name: Some("my-app".into()),
            protocol: "TCP".into(),
            start_cmd: Some("node server.js".into()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: PortInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.port, 3000);
        assert_eq!(deserialized.pid, 1234);
        assert_eq!(deserialized.process_name, "node");
    }

    #[test]
    fn port_event_creation() {
        let event = PortEvent {
            port: 3000,
            pid: 1234,
            process_name: "node".into(),
            framework: Some("React".into()),
            event_type: EventType::Started,
            timestamp: 1000,
        };
        assert_eq!(event.port, 3000);
        assert_eq!(event.event_type, EventType::Started);
    }

    #[test]
    fn event_type_serialization() {
        let json = serde_json::to_string(&EventType::Conflict).unwrap();
        assert_eq!(json, "\"conflict\"");
    }

    #[test]
    fn graph_node_creation() {
        let node = GraphNode {
            id: "port:3000".into(),
            port: 3000,
            pid: 1234,
            process_name: "node".into(),
            project_name: Some("my-app".into()),
            cluster_id: Some("my-app".into()),
            framework: Some("React".into()),
            is_dev: true,
            connection_count: 5,
        };
        assert_eq!(node.id, "port:3000");
        assert_eq!(node.connection_count, 5);
    }

    #[test]
    fn graph_edge_creation() {
        let edge = GraphEdge {
            source: "port:3000".into(),
            target: "port:5432".into(),
            active: true,
            edge_type: EdgeType::TcpConnection,
        };
        assert_eq!(edge.source, "port:3000");
        assert!(edge.active);
    }

    #[test]
    fn port_graph_empty() {
        let graph = PortGraph {
            nodes: vec![],
            edges: vec![],
            clusters: vec![],
        };
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.clusters.is_empty());
    }

    #[test]
    fn traffic_sample_creation() {
        let sample = TrafficSample {
            connections: 10,
            timestamp: 1234567890,
        };
        assert_eq!(sample.connections, 10);
        assert_eq!(sample.timestamp, 1234567890);
    }

    #[test]
    fn framework_creation() {
        let fw = Framework {
            port: 3000,
            name: "React".into(),
            is_dev: true,
        };
        assert_eq!(fw.port, 3000);
        assert!(fw.is_dev);
    }

    #[test]
    fn port_info_without_optionals() {
        let info = PortInfo {
            port: 8080,
            pid: 0,
            process_name: "unknown".into(),
            project_path: None,
            project_name: None,
            protocol: "UDP".into(),
            start_cmd: None,
        };
        assert!(info.project_path.is_none());
        assert!(info.start_cmd.is_none());
        assert_eq!(info.protocol, "UDP");
    }
}
