#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connections;
mod logger;
mod scanner;
mod tray;

use std::collections::HashMap;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_ports,
            kill_process,
            restart_process,
            get_port_graph,
            get_port_events,
            get_port_traffic,
        ])
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Minimize to tray instead of closing
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_ports() -> Vec<scanner::PortInfo> {
    scanner::scan_ports()
}

#[tauri::command]
fn kill_process(pid: u32) -> Result<(), String> {
    scanner::kill_pid(pid)
}

#[tauri::command]
fn restart_process(pid: u32, cmd: String, cwd: String) -> Result<(), String> {
    // Kill existing process first
    scanner::kill_pid(pid)?;

    // Small delay to let port free up
    std::thread::sleep(std::time::Duration::from_millis(800));

    // Relaunch in original working directory
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", &cmd])
            .current_dir(&cwd)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // Split command into program + args
        let mut parts = cmd.split_whitespace();
        let program = parts.next().ok_or("empty command")?;
        let args: Vec<&str> = parts.collect();

        std::process::Command::new(program)
            .args(&args)
            .current_dir(&cwd)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn get_port_graph() -> connections::PortGraph {
    let ports = scanner::scan_ports();
    let listening: Vec<(u16, u32, String, Option<String>)> = ports
        .iter()
        .map(|p| {
            (
                p.port,
                p.pid,
                p.process_name.clone(),
                p.project_name.clone(),
            )
        })
        .collect();
    connections::get_port_graph(&listening)
}

#[tauri::command]
fn get_port_events() -> Vec<logger::PortEvent> {
    let lg = logger::LOGGER.lock().unwrap();
    lg.get_events()
}

#[tauri::command]
fn get_port_traffic() -> HashMap<u16, Vec<logger::TrafficSample>> {
    let lg = logger::LOGGER.lock().unwrap();
    lg.get_all_traffic()
}
