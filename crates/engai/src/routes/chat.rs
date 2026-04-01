use axum::{
    extract::{Query, State, WebSocketUpgrade, ws::Message},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::StreamExt;
use serde::Deserialize;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(ws_handler))
        .route("/history", get(get_history))
        .route("/send", get(send_message))
}

#[derive(Deserialize)]
struct HistoryQuery {
    limit: Option<i64>,
}

async fn get_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> ApiResult<Json<Vec<engai_core::models::ChatEntry>>> {
    let entries = state
        .chat_service
        .get_recent(query.limit.unwrap_or(50))
        .await?;
    Ok(Json(entries))
}

#[derive(Deserialize)]
struct SendMessageQuery {
    content: String,
}

async fn send_message(
    State(state): State<AppState>,
    Query(query): Query<SendMessageQuery>,
) -> ApiResult<Json<engai_core::models::ChatEntry>> {
    state.chat_service.add_message("user", &query.content).await?;
    
    let recent = state.chat_service.get_recent(20).await.unwrap_or_default();
    
    let mut messages: Vec<engai_core::ai::ChatMessage> = recent
        .into_iter()
        .rev()
        .map(|e| engai_core::ai::ChatMessage {
            role: e.role,
            content: e.content,
        })
        .collect();
    
    messages.insert(
        0,
        engai_core::ai::ChatMessage {
            role: "system".to_string(),
            content: "You are an English learning assistant. Help the user improve their English skills."
                .to_string(),
        },
    );
    
    let response = state.ai_service.chat_completion(&messages).await?;
    
    state.chat_service.add_message("assistant", &response).await?;
    
    let entries = state.chat_service.get_recent(1).await?;
    let entry = entries.into_iter().next().unwrap();
    
    Ok(Json(entry))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: axum::extract::ws::WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Close(_) = msg {
            break;
        }
        let text = match msg.to_text() {
            Ok(t) => t.to_string(),
            Err(_) => continue,
        };

        let input: ChatInput = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Err(e) = state.chat_service.add_message("user", &input.content).await {
            let _ = socket
                .send(Message::Text(
                    format!("{{\"error\":\"{}\"}}", e).into(),
                ))
                .await;
            continue;
        }

        let recent = state.chat_service.get_recent(20).await.unwrap_or_default();

        let mut messages: Vec<engai_core::ai::ChatMessage> = recent
            .into_iter()
            .rev()
            .map(|e| engai_core::ai::ChatMessage {
                role: e.role,
                content: e.content,
            })
            .collect();

        messages.insert(
            0,
            engai_core::ai::ChatMessage {
                role: "system".to_string(),
                content: "You are an English learning assistant. Help the user improve their English skills."
                    .to_string(),
            },
        );

        let full_response = match state.ai_service.chat_completion_stream(messages).await {
            Ok(stream) => {
                let mut full = String::new();
                futures::pin_mut!(stream);
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            full.push_str(&chunk);
                            let resp = serde_json::json!({
                                "role": "assistant",
                                "content": chunk,
                            });
                            let _ = socket.send(Message::Text(resp.to_string().into())).await;
                        }
                        Err(e) => {
                            let _ = socket
                                .send(Message::Text(
                                    format!("{{\"error\":\"{}\"}}", e).into(),
                                ))
                                .await;
                            break;
                        }
                    }
                }
                full
            }
            Err(e) => {
                let _ = socket
                    .send(Message::Text(
                        format!("{{\"error\":\"{}\"}}", e).into(),
                    ))
                    .await;
                continue;
            }
        };

        let _ = state.chat_service.add_message("assistant", &full_response).await;
    }
}

#[derive(Deserialize)]
struct ChatInput {
    #[allow(dead_code)]
    role: String,
    content: String,
}
