pub mod app;
pub mod event;
pub mod panel_chat;
pub mod panel_read;
pub mod panel_review;
pub mod panel_stats;
pub mod panel_vocab;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use std::time::Duration;

use crate::api::ApiClient;
use app::{App, Panel};

pub async fn run(client: ApiClient) -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, client).await;

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    client: ApiClient,
) -> Result<()> {
    let mut app = App::default();

    panel_vocab::load_vocab(&client, &mut app).await;
    panel_stats::load_stats(&client, &mut app).await;

    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        match event::poll_event(tick_rate) {
            Some(event::AppEvent::Key(code, modifiers)) => {
                handle_key(&mut app, &client, code, modifiers).await;
            }
            Some(event::AppEvent::Tick) => {
                app.clear_status_if_expired();
            }
            None => continue,
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn on_panel_enter(app: &mut App, client: &ApiClient) {
    match app.panel {
        Panel::Vocab => panel_vocab::load_vocab(client, app).await,
        Panel::Review => {
            if app.review_items.is_empty() {
                panel_review::load_review(client, app).await;
            }
        }
        Panel::Read => {
            if app.readings.is_empty() {
                panel_read::load_readings(client, app).await;
            }
        }
        Panel::Chat => {
            if app.chat_messages.is_empty() {
                match client.get_chat_history(50).await {
                    Ok(msgs) => app.chat_messages = msgs.into_iter().rev().collect(),
                    Err(_) => app.set_status("Failed to load chat history"),
                }
            }
        }
        Panel::Stats => panel_stats::load_stats(client, app).await,
    }
}

async fn handle_key(app: &mut App, client: &ApiClient, code: KeyCode, modifiers: KeyModifiers) {
    if code == KeyCode::Char('q') && app.chat_input.is_empty() {
        app.should_quit = true;
        return;
    }
    if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    if code == KeyCode::Esc {
        if app.vocab_detail.is_some() {
            app.vocab_detail = None;
            return;
        }
        if app.reading_detail.is_some() {
            app.reading_detail = None;
            return;
        }
    }

    match app.panel {
        Panel::Vocab => panel_vocab::handle_key(app, client, code).await,
        Panel::Review => panel_review::handle_key(app, client, code).await,
        Panel::Read => panel_read::handle_key(app, client, code).await,
        Panel::Chat => panel_chat::handle_key(app, client, code).await,
        Panel::Stats => panel_stats::handle_key(app, client, code).await,
    }

    if app.panel != Panel::Chat || app.chat_input.is_empty()
    {
        match code {
            KeyCode::Char('[') | KeyCode::Left => {
                app.panel = app.panel.prev();
                on_panel_enter(app, client).await;
            }
            KeyCode::Char(']') | KeyCode::Right => {
                app.panel = app.panel.next();
                on_panel_enter(app, client).await;
            }
            KeyCode::Tab => {
                if app.panel == Panel::Vocab && app.vocab_detail.is_none() {
                    panel_vocab::handle_tab(app);
                }
            }
            _ => {}
        }
    }
}
