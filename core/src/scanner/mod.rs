use std::collections::{HashMap, HashSet};
use std::process::Command;

use sysinfo::System;

use crate::error::Result;
use crate::models::PortInfo;

pub(crate) trait ScannerBackend: Send {
    fn scan(&mut self, sys: &mut System) -> Result<Vec<PortInfo>>;
    fn kill(pid: u32) -> Result<()>
    where
        Self: Sized;
}

// ── Linux scanner: reads /proc/net/tcp + docker-proxy cmdline matching ──
#[cfg(target_os = "linux")]
pub(crate) struct UnixScanner;

#[cfg(target_os = "linux")]
impl ScannerBackend for UnixScanner {
    fn scan(&mut self, sys: &mut System) -> Result<Vec<PortInfo>> {
        let mut ports: Vec<PortInfo> = Vec::new();
        let mut seen: HashSet<(u16, u32)> = HashSet::new();

        // 1. Try lsof first for user-visible processes (gives richer info)
        let lsof_ports = scan_lsof(sys)?;
        for p in &lsof_ports {
            seen.insert((p.port, p.pid));
        }
        ports.extend(lsof_ports);

        // 2. Supplement with /proc/net/tcp to catch Docker / root-owned ports
        let tcp_ports = scan_proc_net_tcp(sys, &mut seen)?;
        ports.extend(tcp_ports);

        // 3. Match Docker ports to Compose projects
        let port_pids: Vec<(u16, u32)> = ports.iter().map(|p| (p.port, p.pid)).collect();
        let docker_projects = match_docker_ports_to_project(&port_pids);
        for p in &mut ports {
            if p.project_name.is_none() {
                if let Some(project) = docker_projects.get(&p.port) {
                    p.project_name = Some(project.clone());
                    p.project_path = Some(format!("/docker/compose/{project}"));
                }
            }
        }

        ports.sort_by_key(|p| p.port);
        Ok(ports)
    }

    fn kill(pid: u32) -> Result<()> {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));

        if process_exists_unix(pid) {
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }
        Ok(())
    }
}

// ── macOS scanner: uses lsof (existing behaviour) ──
#[cfg(target_os = "macos")]
pub(crate) struct UnixScanner;

#[cfg(target_os = "macos")]
impl ScannerBackend for UnixScanner {
    fn scan(&mut self, sys: &mut System) -> Result<Vec<PortInfo>> {
        scan_lsof(sys)
    }

    fn kill(pid: u32) -> Result<()> {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));

        if process_exists_unix(pid) {
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }
        Ok(())
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn process_exists_unix(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn scan_lsof(sys: &mut System) -> Result<Vec<PortInfo>> {
    let output = Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-n", "-P"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports: Vec<PortInfo> = Vec::new();
    let mut seen: HashSet<(u16, u32)> = HashSet::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let pid: u32 = match parts[1].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        // The NAME column may be followed by (LISTEN) as a separate token,
        // so scan backwards from the end to find the address:port token.
        let port: u16 = match parts
            .iter()
            .rev()
            .find_map(|t| t.rsplit(':').next().and_then(|p| p.parse().ok()))
        {
            Some(p) => p,
            None => continue,
        };

        if seen.contains(&(port, pid)) {
            continue;
        }
        seen.insert((port, pid));

        let process_name = parts[0].to_string();
        let (project_path, start_cmd) = get_process_info(sys, pid);

        let project_name = extract_project_name(&project_path);

        ports.push(PortInfo {
            port,
            pid,
            process_name,
            project_path,
            project_name,
            protocol: "TCP".into(),
            start_cmd,
        });
    }

    Ok(ports)
}

/// Read `/proc/net/tcp` (world-readable on Linux) and discover listening ports
/// that were missed by `lsof` (e.g. Docker-mapped ports owned by root).
#[cfg(target_os = "linux")]
fn scan_proc_net_tcp(
    sys: &mut System,
    seen: &mut HashSet<(u16, u32)>,
) -> Result<Vec<PortInfo>> {
    let mut ports: Vec<PortInfo> = Vec::new();
    // Track ports we already found via lsof (by port number alone)
    let mut seen_ports: HashSet<u16> = seen.iter().map(|(p, _)| *p).collect();

    let tcp_content = match std::fs::read_to_string("/proc/net/tcp") {
        Ok(c) => c,
        Err(_) => {
            // /proc/net/tcp is not available (unlikely on Linux) – skip silently
            return Ok(ports);
        }
    };

    // Pre-build a map of host-port → PID from docker-proxy cmdlines
    let docker_proxy_map = build_docker_proxy_map();

    for line in tcp_content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        // 0A = TCP_LISTEN
        if parts[3] != "0A" {
            continue;
        }

        let port_hex = parts[1].split(':').nth(1).unwrap_or("0");
        let port = match u16::from_str_radix(port_hex, 16) {
            Ok(p) if p != 0 => p,
            _ => continue,
        };

        if seen_ports.contains(&port) {
            continue;
        }
        seen_ports.insert(port);

        let mut pid = 0u32;
        let mut process_name = String::new();
        let mut start_cmd: Option<String> = None;

        // Try docker-proxy matching
        if let Some(dp) = docker_proxy_map.get(&port) {
            pid = dp.pid;
            process_name = "docker-proxy".into();
            start_cmd = dp.cmdline.clone();
        }

        seen.insert((port, pid));

        if pid != 0 {
            if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
                let sys_name = process.name().to_string();
                if !sys_name.is_empty() {
                    process_name = sys_name;
                }
                if start_cmd.is_none() {
                    let cmd = process.cmd().join(" ");
                    if !cmd.trim().is_empty() {
                        start_cmd = Some(cmd);
                    }
                }
            }
        }

        if process_name.is_empty() {
            process_name = if pid != 0 {
                format!("PID {pid}")
            } else {
                "unknown".into()
            };
        }

        let project_path = None;
        let project_name = extract_project_name(&project_path);

        ports.push(PortInfo {
            port,
            pid,
            process_name,
            project_path,
            project_name,
            protocol: "TCP".into(),
            start_cmd,
        });
    }

    Ok(ports)
}

#[cfg(target_os = "linux")]
struct DockerProxyEntry {
    pid: u32,
    cmdline: Option<String>,
}

/// Scan `/proc/*/cmdline` (world-readable) to find docker-proxy processes
/// and map their host-port to a PID.
#[cfg(target_os = "linux")]
fn build_docker_proxy_map() -> HashMap<u16, DockerProxyEntry> {
    let mut map = HashMap::new();

    let proc_dir = match std::fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return map,
    };

    for entry in proc_dir.flatten() {
        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(s) => s,
            None => continue,
        };
        if !name_str.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }

        let pid: u32 = match name_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let cmdline_path = entry.path().join("cmdline");
        let cmdline_bytes = match std::fs::read(&cmdline_path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // cmdline uses \0 as separator; we join with space for parsing
        let cmdline: String = cmdline_bytes
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .filter_map(|s| std::str::from_utf8(s).ok())
            .collect::<Vec<_>>()
            .join(" ");

        if !cmdline.contains("docker-proxy") {
            continue;
        }

        // Extract host-port from: -host-port <PORT>
        if let Some(pos) = cmdline.find("-host-port") {
            let after = &cmdline[pos + "-host-port".len()..];
            let port_str = after.trim_start().split_whitespace().next().unwrap_or("");
            if let Ok(port) = port_str.parse::<u16>() {
                let cmdline_full = Some(cmdline);
                map.entry(port).or_insert(DockerProxyEntry {
                    pid,
                    cmdline: cmdline_full,
                });
            }
        }
    }

    map
}

/// Fetch process info from sysinfo (works for accessible processes).
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_process_info(sys: &mut System, pid: u32) -> (Option<String>, Option<String>) {
    let mut project_path = None;
    let mut start_cmd = None;

    if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
        if let Some(cwd) = process.cwd() {
            project_path = find_project_root(cwd).map(|p| p.to_string_lossy().to_string());
        }

        let cmd = process.cmd().join(" ");
        if !cmd.trim().is_empty() {
            start_cmd = Some(cmd);
        }
    }

    if project_path.is_none() {
        project_path = get_project_path_unix_fallback(pid);
    }

    // If no project found via cwd, try Docker container labels
    if project_path.is_none() {
        if let Some(compose_project) = detect_docker_compose_for_pid(pid) {
            project_path = Some(format!("/docker/compose/{compose_project}"));
        }
    }

    (project_path, start_cmd)
}

/// Detect if a process belongs to a Docker Compose project by checking
/// `/proc/<pid>/cgroup` and then matching with `docker inspect`.
#[cfg(target_os = "linux")]
fn detect_docker_compose_for_pid(pid: u32) -> Option<String> {
    // Read /proc/<pid>/cgroup to get container ID
    let cgroup_path = format!("/proc/{pid}/cgroup");
    let cgroup = std::fs::read_to_string(&cgroup_path).ok()?;

    // Extract container ID from cgroup entries like:
    // 1:name=systemd:/docker/<container_id>
    // 0::/system.slice/docker-<container_id>.scope
    let container_id = cgroup
        .lines()
        .filter_map(|line| {
            if let Some(suffix) = line.rsplit('/').next() {
                if suffix.len() == 64 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Some(suffix.to_string());
                }
                // Also handle docker-<id>.scope format
                if let Some(rest) = suffix.strip_prefix("docker-") {
                    if let Some(id) = rest.strip_suffix(".scope") {
                        if id.len() == 64 && id.chars().all(|c| c.is_ascii_hexdigit()) {
                            return Some(id.to_string());
                        }
                    }
                }
            }
            None
        })
        .next()?;

    // Try docker inspect to get labels (truncated container ID is often sufficient)
    let output = std::process::Command::new("docker")
        .args([
            "inspect",
            &container_id,
            "--format",
            "{{.Config.Labels}}",
        ])
        .output()
        .ok()?;

    let labels = String::from_utf8_lossy(&output.stdout);
    for part in labels.split_whitespace() {
        if let Some(project) = part.strip_prefix("com.docker.compose.project:") {
            let project = project.trim_matches(',');
            if !project.is_empty() {
                return Some(project.to_string());
            }
        }
    }

    None
}

#[cfg(not(target_os = "linux"))]
fn detect_docker_compose_for_pid(_pid: u32) -> Option<String> {
    None
}

/// Try to detect Docker Compose project by running `docker ps` and matching
/// exposed ports to local listening ports.
#[cfg(target_os = "linux")]
pub(crate) fn match_docker_ports_to_project(ports: &[(u16, u32)]) -> HashMap<u16, String> {
    use std::collections::HashMap;

    let mut result = HashMap::new();

    let output = match std::process::Command::new("docker")
        .args([
            "ps",
            "--format",
            "{{.ID}}\t{{.Ports}}\t{{.Label \"com.docker.compose.project\"}}",
        ])
        .output()
    {
        Ok(o) => o,
        Err(_) => return result,
    };

    if !output.status.success() {
        return result;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let mut parts = line.splitn(3, '\t');
        let _id = match parts.next() { Some(id) => id, None => continue };
        let port_str = match parts.next() { Some(s) => s, None => continue };
        let project = parts.next().unwrap_or("");

        if project.is_empty() {
            continue;
        }

        // Parse port mappings like "0.0.0.0:5432->5432/tcp"
        for mapping in port_str.split(',') {
            let mapping = mapping.trim();
            if let Some(host_part) = mapping.split("->").next() {
                if let Some(host_port) = host_part.rsplit(':').next() {
                    if let Ok(port) = host_port.parse::<u16>() {
                        if ports.iter().any(|(p, _)| *p == port) {
                            result.entry(port).or_insert_with(|| project.to_string());
                        }
                    }
                }
            }
        }
    }

    result
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn match_docker_ports_to_project(_ports: &[(u16, u32)]) -> HashMap<u16, String> {
    HashMap::new()
}

#[cfg(target_os = "windows")]
pub(crate) struct WindowsScanner;

#[cfg(target_os = "windows")]
impl ScannerBackend for WindowsScanner {
    fn scan(&mut self, sys: &mut System) -> Result<Vec<PortInfo>> {
        let output = Command::new("netstat").args(["-ano"]).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ports: Vec<PortInfo> = Vec::new();
        let mut seen: HashSet<(u16, u32)> = HashSet::new();

        for line in stdout.lines() {
            if !line.contains("LISTENING") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            let port: u16 = match parts[1].rsplit(':').next().and_then(|p| p.parse().ok()) {
                Some(p) => p,
                None => continue,
            };

            let pid: u32 = match parts[4].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if pid == 0 || seen.contains(&(port, pid)) {
                continue;
            }
            seen.insert((port, pid));

            let mut process_name = format!("PID {pid}");
            let mut project_path = None;
            let mut start_cmd = None;

            if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
                let p_name = process.name().to_string();
                if !p_name.trim().is_empty() {
                    process_name = p_name;
                }

                let cmd_str = process.cmd().join(" ");
                if !cmd_str.trim().is_empty() {
                    start_cmd = Some(cmd_str.trim().to_string());
                }

                if let Some(cwd) = process.cwd() {
                    project_path = find_project_root(cwd).map(|p| p.to_string_lossy().to_string());
                }

                if project_path.is_none() {
                    if let Some(exe) = process.exe() {
                        if let Some(parent) = exe.parent() {
                            project_path =
                                find_project_root(parent).map(|p| p.to_string_lossy().to_string());
                        }
                    }
                }
            }

            let project_name = extract_project_name(&project_path);

            ports.push(PortInfo {
                port,
                pid,
                process_name,
                project_path,
                project_name,
                protocol: "TCP".into(),
                start_cmd,
            });
        }

        ports.sort_by_key(|p| p.port);
        Ok(ports)
    }

    fn kill(pid: u32) -> Result<()> {
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
            .map_err(|e| crate::error::Error::Process(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(crate::error::Error::Process(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
}

pub struct PortScanner {
    sys: System,
    backend: Box<dyn ScannerBackend>,
}

impl PortScanner {
    pub fn new() -> Self {
        let sys = System::new();
        #[cfg(target_os = "windows")]
        let backend = Box::new(WindowsScanner);
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        let backend = Box::new(UnixScanner);
        Self { sys, backend }
    }

    pub fn scan(&mut self) -> Result<Vec<PortInfo>> {
        self.sys.refresh_processes();
        self.backend.scan(&mut self.sys)
    }

    pub fn kill(&self, pid: u32) -> Result<()> {
        #[cfg(target_os = "windows")]
        return WindowsScanner::kill(pid);
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        return UnixScanner::kill(pid);
    }

    pub fn restart(&self, pid: u32, cmd: &str, cwd: &str) -> Result<()> {
        self.kill(pid)?;

        std::thread::sleep(std::time::Duration::from_millis(800));

        let mut parts = cmd.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| crate::error::Error::Process("empty command".into()))?;
        let args: Vec<&str> = parts.collect();

        Command::new(program)
            .args(&args)
            .current_dir(cwd)
            .spawn()
            .map_err(|e| crate::error::Error::Process(e.to_string()))?;

        Ok(())
    }
}

impl Default for PortScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_project_path_unix_fallback(pid: u32) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let cwd = std::fs::read_link(format!("/proc/{pid}/cwd")).ok()?;
        find_project_root(&cwd).map(|p| p.to_string_lossy().to_string())
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("lsof")
            .args(["-p", &pid.to_string(), "-a", "-d", "cwd", "-Fn"])
            .output()
            .ok()?;

        let s = String::from_utf8_lossy(&output.stdout);
        let cwd = s
            .lines()
            .find(|l| l.starts_with('n') && l.len() > 1)
            .map(|l| l[1..].to_string())?;

        find_project_root(std::path::Path::new(&cwd)).map(|p| p.to_string_lossy().to_string())
    }
}

const PROJECT_MARKERS: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "requirements.txt",
    "pom.xml",
    "build.gradle",
    ".git",
];

fn find_project_root(start: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut dir = start.to_path_buf();
    for _ in 0..6 {
        for marker in PROJECT_MARKERS {
            if dir.join(marker).exists() {
                return Some(dir);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn extract_project_name(path: &Option<String>) -> Option<String> {
    path.as_ref()
        .and_then(|p| std::path::Path::new(p).file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_project_root_returns_none_for_empty_path() {
        let tmp = tempfile::tempdir().unwrap();
        let result = find_project_root(tmp.path());
        assert!(result.is_none());
    }

    #[test]
    fn find_project_root_detects_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();
        let result = find_project_root(tmp.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), tmp.path());
    }

    #[test]
    fn find_project_root_walks_upward() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(tmp.path().join(".git"), "").unwrap();
        let result = find_project_root(&sub);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), tmp.path());
    }

    #[test]
    fn find_project_root_stops_after_six_levels() {
        let tmp = tempfile::tempdir().unwrap();
        let deep = tmp
            .path()
            .join("a")
            .join("b")
            .join("c")
            .join("d")
            .join("e")
            .join("f")
            .join("g");
        std::fs::create_dir_all(&deep).unwrap();
        let result = find_project_root(&deep);
        assert!(result.is_none());
    }

    #[test]
    fn extract_project_name_returns_none_for_none_path() {
        assert_eq!(extract_project_name(&None), None);
    }

    #[test]
    fn extract_project_name_returns_dir_name() {
        let path = Some("/home/user/my-project".into());
        assert_eq!(extract_project_name(&path), Some("my-project".into()));
    }

    #[test]
    fn extract_project_name_handles_trailing_slash() {
        let path = Some("/home/user/my-project/".into());
        assert_eq!(extract_project_name(&path), Some("my-project".into()));
    }
}
