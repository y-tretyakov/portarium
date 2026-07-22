use crate::scanner::{self, PortInfo};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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
    let dev_ports = [
        3000u16, 3001, 4000, 4200, 5173, 5174, 8000, 8080, 8888, 5432, 3306, 6379, 27017, 1420,
    ];

    // Check for conflict: same port bound twice
    let mut seen = std::collections::HashSet::new();
    for p in ports {
        if !seen.insert(p.port) {
            return TrafficState::Conflict;
        }
    }

    // Active: any known dev port is in use
    if ports.iter().any(|p| dev_ports.contains(&p.port)) {
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

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    // Build right-click menu
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

    // Build initial tray with green icon
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
            // Left click = open window
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

    // Start background watcher thread
    let app_handle = app.clone();
    let debounce = Arc::new(Mutex::new(DebounceState::new()));
    let last_ports_json = Arc::new(Mutex::new(String::new()));

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(2));

            let ports = scanner::scan_ports();
            let new_state = compute_state(&ports);
            let port_count = ports
                .iter()
                .filter(|p| {
                    [
                        3000u16, 3001, 4000, 4200, 5173, 5174, 8000, 8080, 5432, 3306, 6379, 27017,
                        1420,
                    ]
                    .contains(&p.port)
                })
                .count();

            // Update the logger with current ports and connection counts
            {
                let port_tuples: Vec<(u16, u32, String, Option<String>)> = ports
                    .iter()
                    .map(|p| {
                        let fw = crate::connections::get_framework_name(p.port);
                        (p.port, p.pid, p.process_name.clone(), fw)
                    })
                    .collect();

                // Get connection counts from the graph
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
                let graph = crate::connections::get_port_graph(&listening);
                let mut conn_counts = std::collections::HashMap::new();
                for node in &graph.nodes {
                    conn_counts.insert(node.port, node.connection_count);
                }

                let mut logger = crate::logger::LOGGER.lock().unwrap();
                let new_events = logger.update(&port_tuples, &conn_counts);

                // Emit new events to the frontend
                if !new_events.is_empty() {
                    let _ = app_handle.emit("port-events", &new_events);
                }
            }

            let should_update = {
                let mut db = debounce.lock().unwrap();

                if db.pending != new_state {
                    // State changed — reset debounce timer
                    db.pending = new_state.clone();
                    db.since = Instant::now();
                    false
                } else if db.current != new_state && db.since.elapsed() >= Duration::from_secs(4) {
                    // Stable for 4s and different from current — update
                    db.current = new_state.clone();
                    true
                } else {
                    false
                }
            };

            if should_update {
                if let Some(tray) = app_handle.tray_by_id("main") {
                    let icon_bytes = get_icon_bytes(&new_state);
                    if let Ok(icon) = tauri::image::Image::from_bytes(icon_bytes) {
                        let _ = tray.set_icon(Some(icon));
                    }
                    let tooltip = get_tooltip(&new_state, port_count);
                    let _ = tray.set_tooltip(Some(&tooltip));
                }

                // Emit event to frontend so UI stays in sync
                let _ = app_handle.emit("tray-state-changed", new_state.clone());
            }

            // Only emit port updates to frontend if data changed
            if let Ok(json) = serde_json::to_string(&ports) {
                let mut last = last_ports_json.lock().unwrap();
                if *last != json {
                    *last = json;
                    let _ = app_handle.emit("ports-updated", &ports);
                }
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
