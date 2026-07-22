mod tray;

use std::sync::{Arc, Mutex};

use portarium_core::config::PortariumConfig;
use portarium_core::models::*;
use portarium_core::service::PortariumService;

fn err_string(e: portarium_core::Error) -> String {
    e.to_string()
}

pub struct AppState {
    pub service: Arc<Mutex<PortariumService>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let service = Arc::new(Mutex::new(
        PortariumService::new(PortariumConfig::default()),
    ));
    let tray_service = service.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { service })
        .invoke_handler(tauri::generate_handler![
            get_ports,
            kill_process,
            restart_process,
            get_port_graph,
            get_port_events,
            get_port_traffic,
        ])
        .setup(move |app| {
            tray::setup_tray(app.handle(), tray_service)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_ports(state: tauri::State<AppState>) -> Result<Vec<PortInfo>, String> {
    let mut service = state.service.lock().map_err(|e| e.to_string())?;
    service.get_ports().map_err(err_string)
}

#[tauri::command]
fn kill_process(state: tauri::State<AppState>, pid: u32) -> Result<(), String> {
    let service = state.service.lock().map_err(|e| e.to_string())?;
    service.kill(pid).map_err(err_string)?;
    Ok(())
}

#[tauri::command]
fn restart_process(
    state: tauri::State<AppState>,
    pid: u32,
    cmd: String,
    cwd: String,
) -> Result<(), String> {
    let service = state.service.lock().map_err(|e| e.to_string())?;
    service.kill(pid).map_err(err_string)?;

    std::thread::sleep(std::time::Duration::from_millis(800));

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
fn get_port_graph(state: tauri::State<AppState>) -> Result<PortGraph, String> {
    let mut service = state.service.lock().map_err(|e| e.to_string())?;
    service.get_graph().map_err(err_string)
}

#[tauri::command]
fn get_port_events(state: tauri::State<AppState>) -> Result<Vec<PortEvent>, String> {
    let service = state.service.lock().map_err(|e| e.to_string())?;
    Ok(service.get_events())
}

#[tauri::command]
fn get_port_traffic(
    state: tauri::State<AppState>,
) -> Result<std::collections::HashMap<u16, Vec<TrafficSample>>, String> {
    let service = state.service.lock().map_err(|e| e.to_string())?;
    Ok(service.get_all_traffic())
}
