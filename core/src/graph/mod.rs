use std::collections::{HashMap, HashSet};
use std::process::Command;

use crate::frameworks;
use crate::models::{EdgeType, GraphEdge, GraphNode, PortGraph};

type ListeningEntry = (u16, u32, String, Option<String>);

pub fn build_port_graph(listening: &[ListeningEntry]) -> PortGraph {
    let connections = get_active_connections();

    let mut node_map: HashMap<u16, GraphNode> = HashMap::new();
    for (port, pid, process_name, project_name) in listening {
        let cluster_id = project_name.clone();
        node_map.insert(
            *port,
            GraphNode {
                id: format!("port:{port}"),
                port: *port,
                pid: *pid,
                process_name: process_name.clone(),
                project_name: project_name.clone(),
                cluster_id,
                framework: frameworks::get_framework(*port),
                is_dev: frameworks::is_dev_port(*port),
                connection_count: 0,
            },
        );
    }

    let mut pid_to_port: HashMap<u32, Vec<u16>> = HashMap::new();
    for (port, pid, _, _) in listening {
        pid_to_port.entry(*pid).or_default().push(*port);
    }

    let mut edges: Vec<GraphEdge> = Vec::new();
    let mut seen_edges: HashSet<(u16, u16)> = HashSet::new();

    for (src_port, dst_port, src_pid, dst_pid) in &connections {
        let src_listen = node_map.contains_key(src_port);
        let dst_listen = node_map.contains_key(dst_port);

        if src_listen && dst_listen {
            add_edge(
                &mut edges,
                &mut seen_edges,
                &mut node_map,
                *src_port,
                *dst_port,
            );
            continue;
        }

        if dst_listen {
            if let Some(src_ports) = pid_to_port.get(src_pid) {
                for sp in src_ports {
                    if *sp != *dst_port {
                        add_edge(&mut edges, &mut seen_edges, &mut node_map, *sp, *dst_port);
                    }
                }
            }
        }
        if src_listen {
            if let Some(dst_ports) = pid_to_port.get(dst_pid) {
                for dp in dst_ports {
                    if *dp != *src_port {
                        add_edge(&mut edges, &mut seen_edges, &mut node_map, *src_port, *dp);
                    }
                }
            }
        }
    }

    let clusters = build_clusters(&node_map);

    PortGraph {
        nodes: node_map.into_values().collect(),
        edges,
        clusters,
    }
}

fn build_clusters(node_map: &HashMap<u16, GraphNode>) -> Vec<crate::models::PortCluster> {
    let mut cluster_map: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for node in node_map.values() {
        if let Some(ref cid) = node.cluster_id {
            cluster_map.entry(cid.clone()).or_default().push(node.id.clone());
        }
    }
    cluster_map
        .into_iter()
        .map(|(id, node_ids)| crate::models::PortCluster {
            label: id.clone(),
            node_ids,
            id,
        })
        .collect()
}

fn add_edge(
    edges: &mut Vec<GraphEdge>,
    seen: &mut HashSet<(u16, u16)>,
    node_map: &mut HashMap<u16, GraphNode>,
    a: u16,
    b: u16,
) {
    let key = if a < b { (a, b) } else { (b, a) };
    if seen.insert(key) {
        edges.push(GraphEdge {
            source: format!("port:{a}"),
            target: format!("port:{b}"),
            active: true,
            edge_type: EdgeType::TcpConnection,
        });
        if let Some(n) = node_map.get_mut(&a) {
            n.connection_count += 1;
        }
        if let Some(n) = node_map.get_mut(&b) {
            n.connection_count += 1;
        }
    }
}

type Connection = (u16, u16, u32, u32);

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
    let output = match Command::new("netstat").args(["-ano"]).output() {
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
    let output = match Command::new("lsof")
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
    let output = match Command::new("ss")
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

    fn make_entry(port: u16, pid: u32, name: &str, project: Option<&str>) -> ListeningEntry {
        (port, pid, name.into(), project.map(String::from))
    }

    #[test]
    fn build_empty_graph() {
        let graph = build_port_graph(&[]);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn build_graph_with_single_port() {
        let listening = vec![make_entry(3000, 1234, "node", None)];
        let graph = build_port_graph(&listening);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].port, 3000);
        assert_eq!(graph.nodes[0].pid, 1234);
    }

    #[test]
    fn build_graph_multiple_ports() {
        let listening = vec![
            make_entry(3000, 1234, "node", None),
            make_entry(3001, 5678, "python", Some("myapp")),
        ];
        let graph = build_port_graph(&listening);
        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn node_framework_detection() {
        let listening = vec![make_entry(5173, 1234, "node", None)];
        let graph = build_port_graph(&listening);
        let node = &graph.nodes[0];
        assert!(node.is_dev);
        assert_eq!(node.framework.as_deref(), Some("Vite"));
    }

    #[test]
    fn edge_added_for_known_framework() {
        let listening = vec![make_entry(5173, 1234, "node", None)];
        let graph = build_port_graph(&listening);
        for node in &graph.nodes {
            assert_eq!(node.id, format!("port:{}", node.port));
        }
    }

    #[test]
    fn build_graph_duplicate_ports_deduped() {
        let listening = vec![
            make_entry(3000, 1234, "node", None),
            make_entry(3000, 1234, "node", None),
        ];
        let graph = build_port_graph(&listening);
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn build_graph_multiple_ports_same_pid() {
        let listening = vec![
            make_entry(3000, 1234, "node", None),
            make_entry(3001, 1234, "node", None),
        ];
        let graph = build_port_graph(&listening);
        assert_eq!(graph.nodes.len(), 2);
    }

    proptest::proptest! {
        #[test]
        fn graph_nodes_have_unique_ports(entries in proptest::collection::vec(
            (1u16..65535u16, 1u32..99999u32),
            0..30,
        )) {
            let listening: Vec<ListeningEntry> = entries.iter()
                .map(|(port, pid)| make_entry(*port, *pid, "test", None))
                .collect();
            let graph = build_port_graph(&listening);
            let mut ports: Vec<u16> = graph.nodes.iter().map(|n| n.port).collect();
            ports.sort();
            ports.dedup();
            assert_eq!(graph.nodes.len(), ports.len());
        }
    }
}
