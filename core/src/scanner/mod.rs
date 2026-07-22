use std::collections::HashSet;
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

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(crate) struct UnixScanner;

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl ScannerBackend for UnixScanner {
    fn scan(&mut self, sys: &mut System) -> Result<Vec<PortInfo>> {
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

            let mut process_name = parts[0].to_string();
            let pid: u32 = match parts[1].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let name = parts[parts.len() - 1];
            let port: u16 = match name.rsplit(':').next().and_then(|p| p.parse().ok()) {
                Some(p) => p,
                None => continue,
            };

            if seen.contains(&(port, pid)) {
                continue;
            }
            seen.insert((port, pid));

            let mut project_path = None;
            let mut start_cmd = None;

            if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
                let sys_name = process.name().to_string();
                if !sys_name.is_empty() {
                    process_name = sys_name;
                }

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
