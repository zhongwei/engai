use crossterm::event::KeyCode;

use super::app::App;
use crate::api::ApiClient;

pub async fn handle_key(app: &mut App, client: &ApiClient, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if app.chat_loading || app.chat_input.trim().is_empty() {
                return;
            }

            let input = app.chat_input.trim().to_string();
            app.chat_input.clear();

            app.chat_loading = true;

            match client.chat(&input).await {
                Ok(response) => {
                    if let Ok(msgs) = client.get_chat_history(50).await {
                        app.chat_messages = msgs.into_iter().rev().collect();
                    }
                    let _ = response;
                    app.chat_error = None;
                }
                Err(e) => {
                    app.chat_error = Some(format!("API error: {}", e));
                }
            }

            app.chat_loading = false;
        }
        KeyCode::Char(c) => {
            app.chat_input.push(c);
        }
        KeyCode::Backspace => {
            app.chat_input.pop();
        }
        _ => {}
    }
}
