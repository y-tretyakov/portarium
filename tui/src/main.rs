mod action;
mod app;
mod tui;
mod ui;

use std::sync::Arc;
use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use tokio::select;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use action::{Action, Screen};
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();

    tui::install_panic_hook();
    let mut terminal = tui::init()?;
    let app = Arc::new(Mutex::new(App::new()));
    let mut events = EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(250));
    let (scan_tx, mut scan_rx) = tokio::sync::mpsc::channel(16);

    {
        let app = Arc::clone(&app);
        let scan_tx = scan_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                let mut app = app.lock().await;
                match app.service.scan_and_log() {
                    Ok(events) => {
                        let _ = scan_tx.send(Action::ScanComplete(events)).await;
                    }
                    Err(e) => {
                        let _ = scan_tx
                            .send(Action::Error(format!("Scan error: {}", e)))
                            .await;
                    }
                }
            }
        });
    }

    loop {
        {
            let app = app.lock().await;
            terminal.draw(|f| ui::render(f, &app))?;
        }

        select! {
            Some(Ok(event)) = events.next() => {
                if let Event::Key(key) = event {
                    if let Some(action) = map_key(key) {
                        let mut app = app.lock().await;
                        app.update(action.clone());
                        if matches!(action, Action::Quit) {
                            break;
                        }
                    }
                }
            }
            _ = tick_interval.tick() => {
                let mut app = app.lock().await;
                app.update(Action::Tick);
            }
            Some(action) = scan_rx.recv() => {
                let mut app = app.lock().await;
                app.update(action);
            }
        }
    }

    tui::restore()?;
    Ok(())
}

fn map_key(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(Action::SelectUp),
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(Action::SelectDown),
        (KeyCode::Enter, _) => Some(Action::Enter),
        (KeyCode::Esc, _) | (KeyCode::Backspace, _) => Some(Action::Back),
        (KeyCode::Char('?'), _) => Some(Action::ToggleHelp),
        (KeyCode::Char('1'), _) => Some(Action::ChangeScreen(Screen::Dashboard)),
        (KeyCode::Char('2'), _) => Some(Action::ChangeScreen(Screen::Detail)),
        (KeyCode::Char('3'), _) => Some(Action::ChangeScreen(Screen::Graph)),
        (KeyCode::Char('r'), _) => Some(Action::Scan),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    #[test]
    fn map_key_quit_q() {
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::Quit)));
    }

    #[test]
    fn map_key_quit_ctrl_c() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(matches!(map_key(key), Some(Action::Quit)));
    }

    #[test]
    fn map_key_navigate_up() {
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::SelectUp)));
    }

    #[test]
    fn map_key_navigate_down_k() {
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::SelectUp)));
    }

    #[test]
    fn map_key_navigate_down() {
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::SelectDown)));
    }

    #[test]
    fn map_key_navigate_down_j() {
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::SelectDown)));
    }

    #[test]
    fn map_key_enter() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::Enter)));
    }

    #[test]
    fn map_key_back_esc() {
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::Back)));
    }

    #[test]
    fn map_key_back_backspace() {
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::Back)));
    }

    #[test]
    fn map_key_toggle_help() {
        let key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::ToggleHelp)));
    }

    #[test]
    fn map_key_screen_dashboard() {
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        assert!(matches!(
            map_key(key),
            Some(Action::ChangeScreen(Screen::Dashboard))
        ));
    }

    #[test]
    fn map_key_screen_detail() {
        let key = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        assert!(matches!(
            map_key(key),
            Some(Action::ChangeScreen(Screen::Detail))
        ));
    }

    #[test]
    fn map_key_screen_graph() {
        let key = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE);
        assert!(matches!(
            map_key(key),
            Some(Action::ChangeScreen(Screen::Graph))
        ));
    }

    #[test]
    fn map_key_scan() {
        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        assert!(matches!(map_key(key), Some(Action::Scan)));
    }

    #[test]
    fn map_key_unknown_returns_none() {
        let key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        assert!(map_key(key).is_none());
    }

    #[test]
    fn map_key_unmodified_c_returns_none() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert!(map_key(key).is_none());
    }
}
