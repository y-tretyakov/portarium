use std::collections::{BTreeMap, HashMap, HashSet};

use crate::frameworks;
use crate::models::{EdgeType, GraphEdge, GraphNode, PortCluster, PortGraph, PortInfo};

type Connection = (u16, u16, u32, u32);

pub struct GraphBuilder;

impl GraphBuilder {
    pub fn build(listening: &[PortInfo]) -> PortGraph {
        let mut node_map = Self::build_nodes(listening);
        let mut seen_edges: HashSet<(u16, u16)> = HashSet::new();
        let mut edges: Vec<GraphEdge> = Vec::new();

        // Strategy 1: TCP connections (base layer)
        let tcp_connections = get_active_connections();
        Self::apply_tcp_strategy(&mut edges, &mut seen_edges, &mut node_map, &tcp_connections);

        // Strategy 2: Project grouping — connect nodes sharing a project
        Self::apply_project_strategy(&mut edges, &mut seen_edges, &node_map);

        // Strategy 3: Orchestration awareness (Docker Compose, etc.)
        Self::apply_orchestration_strategy(&mut edges, &mut seen_edges, &node_map);

        let clusters = Self::build_clusters(&node_map);

        PortGraph {
            nodes: node_map.into_values().collect(),
            edges,
            clusters,
        }
    }

    fn build_nodes(listening: &[PortInfo]) -> HashMap<String, GraphNode> {
        let mut node_map: HashMap<String, GraphNode> = HashMap::new();
        for info in listening {
            let id = format!("port:{}", info.port);
            let cluster_id = info
                .project_name
                .clone()
                .or_else(|| detect_docker_compose_project(info));
            node_map.insert(
                id.clone(),
                GraphNode {
                    id,
                    port: info.port,
                    pid: info.pid,
                    process_name: info.process_name.clone(),
                    project_name: info.project_name.clone(),
                    cluster_id,
                    framework: frameworks::get_framework(info.port),
                    is_dev: frameworks::is_dev_port(info.port),
                    connection_count: 0,
                },
            );
        }
        node_map
    }

    fn apply_tcp_strategy(
        edges: &mut Vec<GraphEdge>,
        seen_edges: &mut HashSet<(u16, u16)>,
        node_map: &mut HashMap<String, GraphNode>,
        connections: &[Connection],
    ) {
        let mut pid_to_port: HashMap<u32, Vec<u16>> = HashMap::new();
        for node in node_map.values() {
            pid_to_port.entry(node.pid).or_default().push(node.port);
        }

        for (src_port, dst_port, src_pid, dst_pid) in connections {
            let src_id = format!("port:{src_port}");
            let dst_id = format!("port:{dst_port}");
            let src_listen = node_map.contains_key(&src_id);
            let dst_listen = node_map.contains_key(&dst_id);

            if src_listen && dst_listen {
                Self::add_edge(
                    edges,
                    seen_edges,
                    node_map,
                    &src_id,
                    &dst_id,
                    EdgeType::TcpConnection,
                );
                continue;
            }

            if dst_listen {
                if let Some(src_ports) = pid_to_port.get(src_pid) {
                    for sp in src_ports {
                        let sp_id = format!("port:{sp}");
                        if sp_id != dst_id {
                            Self::add_edge(
                                edges,
                                seen_edges,
                                node_map,
                                &sp_id,
                                &dst_id,
                                EdgeType::TcpConnection,
                            );
                        }
                    }
                }
            }
            if src_listen {
                if let Some(dst_ports) = pid_to_port.get(dst_pid) {
                    for dp in dst_ports {
                        let dp_id = format!("port:{dp}");
                        if dp_id != src_id {
                            Self::add_edge(
                                edges,
                                seen_edges,
                                node_map,
                                &src_id,
                                &dp_id,
                                EdgeType::TcpConnection,
                            );
                        }
                    }
                }
            }
        }
    }

    fn apply_project_strategy(
        edges: &mut Vec<GraphEdge>,
        seen_edges: &mut HashSet<(u16, u16)>,
        node_map: &HashMap<String, GraphNode>,
    ) {
        let mut project_groups: BTreeMap<String, Vec<&GraphNode>> = BTreeMap::new();
        for node in node_map.values() {
            if let Some(ref pn) = node.project_name {
                project_groups.entry(pn.clone()).or_default().push(node);
            }
        }

        for group in project_groups.values() {
            if group.len() < 2 {
                continue;
            }
            for i in 0..group.len() {
                for j in i + 1..group.len() {
                    Self::add_edge(
                        edges,
                        seen_edges,
                        node_map,
                        &group[i].id,
                        &group[j].id,
                        EdgeType::ProjectPeer,
                    );
                }
            }
        }
    }

    fn apply_orchestration_strategy(
        edges: &mut Vec<GraphEdge>,
        seen_edges: &mut HashSet<(u16, u16)>,
        node_map: &HashMap<String, GraphNode>,
    ) {
        let mut cluster_groups: BTreeMap<String, Vec<&GraphNode>> = BTreeMap::new();
        for node in node_map.values() {
            if let Some(ref cid) = node.cluster_id {
                if cid != node.project_name.as_deref().unwrap_or("") {
                    cluster_groups.entry(cid.clone()).or_default().push(node);
                }
            }
        }

        for group in cluster_groups.values() {
            if group.len() < 2 {
                continue;
            }
            for i in 0..group.len() {
                for j in i + 1..group.len() {
                    Self::add_edge(
                        edges,
                        seen_edges,
                        node_map,
                        &group[i].id,
                        &group[j].id,
                        EdgeType::OrchestrationPeer,
                    );
                }
            }
        }
    }

    fn add_edge(
        edges: &mut Vec<GraphEdge>,
        seen: &mut HashSet<(u16, u16)>,
        _node_map: &HashMap<String, GraphNode>,
        a: &str,
        b: &str,
        edge_type: EdgeType,
    ) {
        let a_port = a
            .strip_prefix("port:")
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        let b_port = b
            .strip_prefix("port:")
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        let key = if a_port < b_port {
            (a_port, b_port)
        } else {
            (b_port, a_port)
        };
        if seen.insert(key) {
            edges.push(GraphEdge {
                source: a.to_string(),
                target: b.to_string(),
                active: true,
                edge_type,
            });
        }
    }

    fn build_clusters(node_map: &HashMap<String, GraphNode>) -> Vec<PortCluster> {
        let mut cluster_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for node in node_map.values() {
            if let Some(ref cid) = node.cluster_id {
                cluster_map
                    .entry(cid.clone())
                    .or_default()
                    .push(node.id.clone());
            }
        }
        cluster_map
            .into_iter()
            .map(|(id, node_ids)| PortCluster {
                label: id.clone(),
                node_ids,
                id,
            })
            .collect()
    }
}

fn detect_docker_compose_project(info: &PortInfo) -> Option<String> {
    if let Some(ref cmd) = info.start_cmd {
        if cmd.contains("com.docker.compose.project") {
            for part in cmd.split_whitespace() {
                if let Some(val) = part.strip_prefix("com.docker.compose.project=") {
                    return Some(val.to_string());
                }
            }
        }
    }
    if info.process_name == "docker-proxy" {
        return info.project_name.clone();
    }
    None
}

fn get_active_connections() -> Vec<Connection> {
    #[cfg(target_os = "windows")]
    return get_connections_windows();
    #[cfg(target_os = "macos")]
    return get_connections_macos();
    #[cfg(target_os = "linux")]
    return get_connections_linux();
}

#[cfg(target_os = "windows")]
fn get_connections_windows() -> Vec<Connection> {
    let output = match std::process::Command::new("netstat")
        .args(["-ano"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut raw_conns: Vec<(u16, u16, u32)> = Vec::new();
    for line in stdout.lines() {
        if !line.contains("ESTABLISHED") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }
        let src_port = parts[1]
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok());
        let dst_port = parts[2]
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok());
        let pid: Option<u32> = parts[4].parse().ok();
        if let (Some(s), Some(d), Some(p)) = (src_port, dst_port, pid) {
            if p == 0 {
                continue;
            }
            raw_conns.push((s, d, p));
        }
    }
    let mut port_pid: HashMap<u16, u32> = HashMap::new();
    for (s, _d, pid) in &raw_conns {
        port_pid.insert(*s, *pid);
    }
    let mut conns = Vec::new();
    for (s, d, src_pid) in &raw_conns {
        let dst_pid = port_pid.get(d).copied().unwrap_or(0);
        conns.push((*s, *d, *src_pid, dst_pid));
    }
    conns
}

#[cfg(target_os = "macos")]
fn get_connections_macos() -> Vec<Connection> {
    let output = match std::process::Command::new("lsof")
        .args(["-iTCP", "-sTCP:ESTABLISHED", "-n", "-P"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut conns = Vec::new();
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }
        let pid: u32 = match parts[1].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let name = parts[parts.len() - 1];
        if !name.contains("->") {
            continue;
        }
        let mut sides = name.split("->");
        let src = sides
            .next()
            .and_then(|s| s.rsplit(':').next())
            .and_then(|p| p.parse::<u16>().ok());
        let dst = sides
            .next()
            .and_then(|s| s.rsplit(':').next())
            .and_then(|p| p.parse::<u16>().ok());
        if let (Some(s), Some(d)) = (src, dst) {
            conns.push((s, d, pid, 0));
        }
    }
    conns
}

#[cfg(target_os = "linux")]
fn get_connections_linux() -> Vec<Connection> {
    let output = match std::process::Command::new("ss")
        .args(["-tnp", "state", "established"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut conns = Vec::new();
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }
        let src = parts[3]
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok());
        let dst = parts[4]
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok());
        let pid: u32 = parts[5]
            .split("pid=")
            .nth(1)
            .and_then(|s| s.split(&[',', ')']).next())
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);
        if let (Some(s), Some(d)) = (src, dst) {
            conns.push((s, d, pid, 0));
        }
    }
    conns
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_port_info(port: u16, pid: u32, name: &str, project: Option<&str>) -> PortInfo {
        PortInfo {
            port,
            pid,
            process_name: name.into(),
            project_path: project.map(|p| format!("/projects/{p}")),
            project_name: project.map(String::from),
            protocol: "TCP".into(),
            start_cmd: None,
        }
    }

    #[test]
    fn empty_listening() {
        let graph = GraphBuilder::build(&[]);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.clusters.is_empty());
    }

    #[test]
    fn single_node_no_cluster() {
        let ports = vec![make_port_info(3000, 100, "node", None)];
        let graph = GraphBuilder::build(&ports);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.clusters.len(), 0);
    }

    #[test]
    fn project_group_creates_cluster() {
        let ports = vec![
            make_port_info(3000, 100, "node", Some("myapp")),
            make_port_info(3001, 101, "node", Some("myapp")),
        ];
        let graph = GraphBuilder::build(&ports);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.clusters.len(), 1);
        assert_eq!(graph.clusters[0].label, "myapp");
        assert_eq!(graph.clusters[0].node_ids.len(), 2);
    }

    #[test]
    fn project_group_creates_edges() {
        let ports = vec![
            make_port_info(3000, 100, "node", Some("myapp")),
            make_port_info(3001, 101, "node", Some("myapp")),
        ];
        let graph = GraphBuilder::build(&ports);
        let project_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::ProjectPeer)
            .collect();
        assert!(!project_edges.is_empty());
    }

    #[test]
    fn multiple_projects_separate_clusters() {
        let ports = vec![
            make_port_info(3000, 100, "node", Some("frontend")),
            make_port_info(4000, 200, "python", Some("backend")),
        ];
        let graph = GraphBuilder::build(&ports);
        assert_eq!(graph.clusters.len(), 2);
    }

    #[test]
    fn cluster_id_matches_project_name() {
        let ports = vec![make_port_info(3000, 100, "node", Some("myapp"))];
        let graph = GraphBuilder::build(&ports);
        let node = &graph.nodes[0];
        assert_eq!(node.cluster_id.as_deref(), Some("myapp"));
    }

    #[test]
    fn node_without_project_has_no_cluster() {
        let ports = vec![make_port_info(5432, 200, "postgres", None)];
        let graph = GraphBuilder::build(&ports);
        assert!(graph.nodes[0].cluster_id.is_none());
        assert!(graph.clusters.is_empty());
    }

    #[test]
    fn framework_detection_works() {
        let ports = vec![make_port_info(5173, 100, "node", None)];
        let graph = GraphBuilder::build(&ports);
        assert_eq!(graph.nodes[0].framework.as_deref(), Some("Vite"));
        assert!(graph.nodes[0].is_dev);
    }

    #[test]
    fn detect_docker_compose_from_cmdline() {
        let info = PortInfo {
            port: 8080,
            pid: 42,
            process_name: "docker-proxy".into(),
            project_path: None,
            project_name: None,
            protocol: "TCP".into(),
            start_cmd: Some("-host-port 8080 com.docker.compose.project=my_stack".into()),
        };
        let project = detect_docker_compose_project(&info);
        assert_eq!(project.as_deref(), Some("my_stack"));
    }

    #[test]
    fn detect_docker_compose_no_match() {
        let info = PortInfo {
            port: 8080,
            pid: 42,
            process_name: "nginx".into(),
            project_path: None,
            project_name: Some("web".into()),
            protocol: "TCP".into(),
            start_cmd: Some("nginx -g daemon off".into()),
        };
        let project = detect_docker_compose_project(&info);
        assert!(project.is_none());
    }
}
