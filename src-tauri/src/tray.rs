use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portarium_core::frameworks;
use portarium_core::models::PortInfo;
use portarium_core::service::PortariumService;
use serde::Serialize;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, PartialEq, Debug, Serialize)]
pub enum TrafficState {
    Clear,
    Active,
    Conflict,
}

struct DebounceState {
    pending: TrafficState,
    since: Instant,
    current: TrafficState,
}

impl DebounceState {
    fn new() -> Self {
        Self {
            pending: TrafficState::Clear,
            since: Instant::now(),
            current: TrafficState::Clear,
        }
    }
}

fn compute_state(ports: &[PortInfo]) -> TrafficState {
    let mut seen = std::collections::HashSet::new();
    for p in ports {
        if !seen.insert(p.port) {
            return TrafficState::Conflict;
        }
    }

    if ports.iter().any(|p| frameworks::is_dev_port(p.port)) {
        return TrafficState::Active;
    }

    TrafficState::Clear
}

fn get_icon_bytes(state: &TrafficState) -> &'static [u8] {
    match state {
        TrafficState::Clear => include_bytes!("../icons/tray-green.png"),
        TrafficState::Active => include_bytes!("../icons/tray-yellow.png"),
        TrafficState::Conflict => include_bytes!("../icons/tray-red.png"),
    }
}

fn get_tooltip(state: &TrafficState, port_count: usize) -> String {
    match state {
        TrafficState::Clear => "Portarium — All clear".into(),
        TrafficState::Active => format!(
            "Portarium — {} dev port{} active",
            port_count,
            if port_count == 1 { "" } else { "s" }
        ),
        TrafficState::Conflict => "Portarium — ⚠ Port conflict detected!".into(),
    }
}

pub fn setup_tray(app: &AppHandle, service: Arc<Mutex<PortariumService>>) -> tauri::Result<()> {
    let open = MenuItemBuilder::new("Open Portarium")
        .id("open")
        .build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::new("Quit").id("quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&open)
        .item(&separator)
        .item(&quit)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(tauri::image::Image::from_bytes(include_bytes!(
            "../icons/tray-green.png"
        ))?)
        .tooltip("Portarium — Starting…")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        })
        .build(app)?;

    let app_handle = app.clone();
    let debounce = Arc::new(Mutex::new(DebounceState::new()));
    let last_ports_json = Arc::new(Mutex::new(String::new()));

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(2));

        let (ports, new_events) = {
            let mut svc = match service.lock() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let new_events = svc.scan_and_log().unwrap_or_default();

            let ports = match svc.get_ports() {
                Ok(p) => p,
                Err(_) => continue,
            };

            (ports, new_events)
        };

        let new_state = compute_state(&ports);
        let port_count = ports
            .iter()
            .filter(|p| frameworks::is_dev_port(p.port))
            .count();

        if !new_events.is_empty() {
            let _ = app_handle.emit("port-events", &new_events);
        }

        let should_update = {
            let mut db = match debounce.lock() {
                Ok(d) => d,
                Err(_) => continue,
            };

            if db.pending != new_state {
                db.pending = new_state.clone();
                db.since = Instant::now();
                false
            } else if db.current != new_state && db.since.elapsed() >= Duration::from_secs(4) {
                db.current = new_state.clone();
                true
            } else {
                false
            }
        };

        if should_update {
            if let Some(tray_item) = app_handle.tray_by_id("main") {
                let icon_bytes = get_icon_bytes(&new_state);
                if let Ok(icon) = tauri::image::Image::from_bytes(icon_bytes) {
                    let _ = tray_item.set_icon(Some(icon));
                }
                let tooltip = get_tooltip(&new_state, port_count);
                let _ = tray_item.set_tooltip(Some(&tooltip));
            }

            let _ = app_handle.emit("tray-state-changed", new_state.clone());
        }

        if let Ok(json) = serde_json::to_string(&ports) {
            let mut last = match last_ports_json.lock() {
                Ok(l) => l,
                Err(_) => continue,
            };
            if *last != json {
                *last = json;
                let _ = app_handle.emit("ports-updated", &ports);
            }
        }
    });

    Ok(())
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
    }
}
