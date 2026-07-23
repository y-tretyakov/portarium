use portarium_core::{EventType, PortEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::action::Screen;
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Dashboard => render_dashboard(frame, app),
        Screen::Detail => render_detail(frame, app),
        Screen::Graph => render_graph(frame, app),
    }
    if app.help_visible {
        render_help(frame, frame.area());
    }
}

fn render_dashboard(frame: &mut Frame, app: &App) {
    let [header, main, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let title = Line::from(" Portarium ".bold().cyan());
    frame.render_widget(title, header);

    let status = &app.status;
    let status_text = if status.is_empty() {
        Line::from(vec![
            " ? ".bold().cyan(),
            "help ".dim(),
            format!(" {} ports ", app.ports.len()).dim(),
        ])
    } else {
        Line::from(vec![Span::raw(status).dim()])
    };
    frame.render_widget(status_text, footer);

    render_port_table(frame, main, app);
}

fn render_port_table(frame: &mut Frame, area: Rect, app: &App) {
    let header_cells = ["Port", "PID", "Process", "Framework", "Status"]
        .iter()
        .map(|h| Cell::from(*h).bold().cyan());
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = app
        .ports
        .iter()
        .map(|p| {
            let framework = p
                .project_name
                .as_deref()
                .or(p.project_path.as_deref())
                .unwrap_or("-");
            let status_style = if p.pid > 0 {
                "Active".green()
            } else {
                "Stopped".red()
            };
            let cells = vec![
                Cell::from(p.port.to_string()),
                Cell::from(p.pid.to_string()),
                Cell::from(p.process_name.as_str()),
                Cell::from(framework),
                Cell::from(status_style),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Length(10),
    ];

    let mut state = ratatui::widgets::TableState::new().with_selected(Some(app.selected_index));
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(Style::new().bold().cyan())
        .highlight_symbol("> ");

    frame.render_stateful_widget(table, area, &mut state);
}

fn render_detail(frame: &mut Frame, app: &App) {
    let [header, main, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let title = Line::from(" Port Detail ".bold().cyan());
    frame.render_widget(title, header);

    let back_text = Line::from(" Esc ".bold().cyan());
    frame.render_widget(back_text, footer);

    if let Some(ref port) = app.selected_port {
        let info_lines = vec![
            Line::from(vec!["Port: ".bold(), port.port.to_string().cyan()]),
            Line::from(vec!["PID:  ".bold(), port.pid.to_string().green()]),
            Line::from(vec!["Process: ".bold(), port.process_name.as_str().into()]),
            Line::from(vec!["Protocol: ".bold(), port.protocol.as_str().into()]),
        ];

        let info = Paragraph::new(info_lines).block(
            Block::default()
                .title(" Info ")
                .borders(Borders::ALL)
                .cyan(),
        );

        let [info_area, events_area] =
            Layout::vertical([Constraint::Length(6), Constraint::Fill(1)]).areas(main);

        frame.render_widget(info, info_area);

        let port_events: Vec<&PortEvent> =
            app.events.iter().filter(|e| e.port == port.port).collect();

        let event_items: Vec<ListItem> = port_events
            .iter()
            .map(|e| {
                let line = Line::from(vec![
                    format!("[{}] ", e.timestamp).dim(),
                    format!(
                        "{} ",
                        match e.event_type {
                            EventType::Started => "START",
                            EventType::Stopped => "STOP",
                            EventType::Conflict => "CONFLICT",
                        }
                    )
                    .bold(),
                    e.process_name.as_str().into(),
                ]);
                ListItem::new(line)
            })
            .collect();

        let events_list = List::new(event_items).block(
            Block::default()
                .title(" Timeline ")
                .borders(Borders::ALL)
                .cyan(),
        );
        frame.render_widget(events_list, events_area);
    } else {
        let no_select =
            Paragraph::new("No port selected").block(Block::default().borders(Borders::ALL).cyan());
        frame.render_widget(no_select, main);
    }
}

fn render_graph(frame: &mut Frame, app: &App) {
    let [header, main, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let title = Line::from(" Port Graph ".bold().cyan());
    frame.render_widget(title, header);

    let back_text = Line::from(" Esc ".bold().cyan());
    frame.render_widget(back_text, footer);

    let items: Vec<ListItem> = if let Some(ref graph) = app.graph {
        graph
            .nodes
            .iter()
            .map(|node| {
                let line = Line::from(vec![
                    format!(":{} ", node.port).cyan(),
                    node.process_name.as_str().into(),
                    format!(" ({})", node.pid).dim(),
                    format!(" — {} connections", node.connection_count).green(),
                ]);
                ListItem::new(line)
            })
            .collect()
    } else {
        vec![ListItem::new(Line::from("No graph data".dim()))]
    };

    let list = List::new(items).block(
        Block::default()
            .title(" Nodes ")
            .borders(Borders::ALL)
            .cyan(),
    );
    frame.render_widget(list, main);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 50, area);
    frame.render_widget(Clear, popup_area);

    let help_lines = vec![
        Line::from(" Keybindings ".bold().cyan()),
        Line::from(""),
        Line::from(vec![" q/Ctrl+C ".bold(), "Quit".dim()]),
        Line::from(vec![" ↑/↓ ".bold(), "Navigate".dim()]),
        Line::from(vec![" Enter ".bold(), "Select".dim()]),
        Line::from(vec![" Esc/Backspace ".bold(), "Back".dim()]),
        Line::from(vec![" ? ".bold(), "Toggle help".dim()]),
        Line::from(vec![" 1/2/3 ".bold(), "Screens D/G/R".dim()]),
        Line::from(vec![" r ".bold(), "Scan".dim()]),
    ];

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .cyan(),
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(help, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, center, _] = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .areas(area);

    let [_, center, _] = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .areas(center);

    center
}
