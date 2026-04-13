use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyCode, KeyModifiers),
    Tick,
}

pub async fn poll_event_async(timeout: Duration) -> Option<AppEvent> {
    tokio::task::spawn_blocking(move || poll_event_blocking(timeout))
        .await
        .ok()?
}

fn poll_event_blocking(timeout: Duration) -> Option<AppEvent> {
    if event::poll(timeout).ok()? {
        match event::read().ok()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                Some(AppEvent::Key(key.code, key.modifiers))
            }
            Event::Resize(_, _) => Some(AppEvent::Tick),
            _ => None,
        }
    } else {
        Some(AppEvent::Tick)
    }
}
