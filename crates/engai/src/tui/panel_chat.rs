use crossterm::event::KeyCode;

use super::app::App;
use crate::state::AppState;
use engai_core::ai::ChatMessage;

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if app.chat_loading || app.chat_input.trim().is_empty() {
                return;
            }

            let input = app.chat_input.trim().to_string();
            app.chat_input.clear();

            if let Err(e) = state.chat_repo.add_chat_message("user", &input).await {
                app.chat_error = Some(format!("DB error: {}", e));
                return;
            }

            if let Ok(msgs) = state.chat_repo.get_recent_chat(50).await {
                app.chat_messages = msgs.into_iter().rev().collect();
            }

            app.chat_loading = true;

            let recent = state.chat_repo.get_recent_chat(20).await.unwrap_or_default();
            let messages: Vec<ChatMessage> = recent
                .iter()
                .map(|r| ChatMessage {
                    role: r.role.clone(),
                    content: r.content.clone(),
                })
                .collect();

            let ai = state.ai_client.clone();
            match ai.chat_completion(&messages).await {
                Ok(response) => {
                    let _ = state.chat_repo.add_chat_message("assistant", &response).await;
                    app.chat_error = None;
                }
                Err(e) => {
                    app.chat_error = Some(format!("AI error: {}", e));
                }
            }

            if let Ok(msgs) = state.chat_repo.get_recent_chat(50).await {
                app.chat_messages = msgs.into_iter().rev().collect();
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
