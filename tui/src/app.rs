use std::collections::HashMap;

use portarium_core::{PortEvent, PortGraph, PortInfo, PortariumService, TrafficSample};

use crate::action::{Action, Screen};

pub struct App {
    pub service: PortariumService,
    pub ports: Vec<PortInfo>,
    pub events: Vec<PortEvent>,
    pub traffic: HashMap<u16, Vec<TrafficSample>>,
    pub graph: Option<PortGraph>,
    pub selected_index: usize,
    pub screen: Screen,
    pub help_visible: bool,
    pub should_quit: bool,
    pub status: String,
    pub selected_port: Option<PortInfo>,
}

impl App {
    pub fn new() -> Self {
        Self {
            service: PortariumService::default(),
            ports: Vec::new(),
            events: Vec::new(),
            traffic: HashMap::new(),
            graph: None,
            selected_index: 0,
            screen: Screen::Dashboard,
            help_visible: false,
            should_quit: false,
            status: String::new(),
            selected_port: None,
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::ScanComplete(events) => {
                self.events = events;
                if let Ok(ports) = self.service.get_ports() {
                    self.ports = ports;
                }
                if let Ok(graph) = self.service.get_graph() {
                    self.graph = Some(graph);
                }
                self.traffic = self.service.get_all_traffic();
            }
            Action::SelectUp => {
                if !self.ports.is_empty() {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
            }
            Action::SelectDown => {
                let max = self.ports.len().saturating_sub(1);
                if self.selected_index < max {
                    self.selected_index += 1;
                }
            }
            Action::Enter => {
                if let Some(port) = self.selected_port_info().cloned() {
                    self.selected_port = Some(port);
                    self.screen = Screen::Detail;
                }
            }
            Action::Back => {
                self.screen = Screen::Dashboard;
                self.selected_port = None;
            }
            Action::ToggleHelp => {
                self.help_visible = !self.help_visible;
            }
            Action::ChangeScreen(screen) => {
                self.screen = screen;
            }
            Action::Error(msg) => {
                self.status = msg;
            }
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Scan | Action::Tick | Action::Kill(..) | Action::Restart(..) => {}
        }
    }

    pub fn selected_port_info(&self) -> Option<&PortInfo> {
        self.ports.get(self.selected_index)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::*;
    use portarium_core::EventType;

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
    fn app_new_has_dashboard_screen() {
        let app = App::new();
        assert_eq!(app.screen, Screen::Dashboard);
        assert!(!app.should_quit);
        assert!(app.ports.is_empty());
        assert!(app.traffic.is_empty());
        assert!(app.graph.is_none());
        assert!(!app.help_visible);
        assert_eq!(app.selected_index, 0);
        assert!(app.selected_port.is_none());
    }

    #[test]
    fn app_default_equals_new() {
        assert_eq!(App::default().should_quit, App::new().should_quit);
    }

    #[test]
    fn app_select_up_at_zero_stays_zero() {
        let mut app = App::new();
        app.update(Action::SelectUp);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn app_select_down_empty_ports_stays_zero() {
        let mut app = App::new();
        app.update(Action::SelectDown);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn app_select_down_with_ports() {
        let mut app = App::new();
        app.ports = vec![make_port(3000, 1, "a"), make_port(3001, 2, "b")];
        app.update(Action::SelectDown);
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn app_select_down_at_max_stays() {
        let mut app = App::new();
        app.ports = vec![make_port(3000, 1, "a")];
        app.update(Action::SelectDown);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn app_toggle_help() {
        let mut app = App::new();
        assert!(!app.help_visible);
        app.update(Action::ToggleHelp);
        assert!(app.help_visible);
        app.update(Action::ToggleHelp);
        assert!(!app.help_visible);
    }

    #[test]
    fn app_change_screen() {
        let mut app = App::new();
        app.update(Action::ChangeScreen(Screen::Graph));
        assert_eq!(app.screen, Screen::Graph);
    }

    #[test]
    fn app_quit() {
        let mut app = App::new();
        app.update(Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn app_back_to_dashboard() {
        let mut app = App::new();
        app.screen = Screen::Detail;
        app.selected_port = Some(make_port(3000, 1234, "node"));
        app.update(Action::Back);
        assert_eq!(app.screen, Screen::Dashboard);
        assert!(app.selected_port.is_none());
    }

    #[test]
    fn app_error_sets_status() {
        let mut app = App::new();
        app.update(Action::Error("test error".into()));
        assert_eq!(app.status, "test error");
    }

    #[test]
    fn app_enter_selects_port() {
        let mut app = App::new();
        app.ports = vec![make_port(3000, 1234, "node")];
        app.update(Action::Enter);
        assert_eq!(app.screen, Screen::Detail);
        assert_eq!(app.selected_port.as_ref().unwrap().port, 3000);
    }

    #[test]
    fn app_enter_empty_ports_does_nothing() {
        let mut app = App::new();
        app.update(Action::Enter);
        assert_eq!(app.screen, Screen::Dashboard);
        assert!(app.selected_port.is_none());
    }

    #[test]
    fn app_selected_port_info_returns_none_when_empty() {
        let app = App::new();
        assert!(app.selected_port_info().is_none());
    }

    #[test]
    fn app_selected_port_info_returns_selected() {
        let mut app = App::new();
        app.ports = vec![make_port(3000, 1234, "node")];
        let info = app.selected_port_info();
        assert!(info.is_some());
        assert_eq!(info.unwrap().port, 3000);
    }

    #[test]
    fn app_select_up_moves_up() {
        let mut app = App::new();
        app.ports = vec![
            make_port(3000, 1, "a"),
            make_port(3001, 2, "b"),
            make_port(3002, 3, "c"),
        ];
        app.selected_index = 2;
        app.update(Action::SelectUp);
        assert_eq!(app.selected_index, 1);
        app.update(Action::SelectUp);
        assert_eq!(app.selected_index, 0);
        app.update(Action::SelectUp);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn app_scan_complete_updates_events_and_traffic() {
        let mut app = App::new();
        let events = vec![PortEvent {
            port: 3000,
            pid: 1234,
            process_name: "node".into(),
            framework: Some("React".into()),
            event_type: EventType::Started,
            timestamp: 1000,
        }];
        app.update(Action::ScanComplete(events.clone()));
        assert_eq!(app.events.len(), 1);
    }

    #[test]
    fn app_action_scan_is_noop() {
        let mut app = App::new();
        app.update(Action::Scan);
        assert!(!app.should_quit);
    }

    #[test]
    fn app_action_tick_is_noop() {
        let mut app = App::new();
        app.update(Action::Tick);
        assert!(!app.should_quit);
    }

    #[test]
    fn app_changes_to_graph_screen() {
        let mut app = App::new();
        app.update(Action::ChangeScreen(Screen::Graph));
        assert_eq!(app.screen, Screen::Graph);
    }

    #[test]
    fn app_back_when_on_dashboard_stays() {
        let mut app = App::new();
        assert_eq!(app.screen, Screen::Dashboard);
        app.update(Action::Back);
        assert_eq!(app.screen, Screen::Dashboard);
    }
}
