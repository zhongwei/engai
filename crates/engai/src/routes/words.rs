use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Json, Router,
};
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::json;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListParams {
    pub search: Option<String>,
    pub familiarity_gte: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateWordBody {
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateWordBody {
    pub word: Option<String>,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
    pub familiarity: Option<i32>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_words).post(create_word))
        .route("/{word}", get(get_word).put(update_word).delete(delete_word))
        .route("/{word}/explain", get(explain_word))
        .route("/{word}/examples", get(get_examples))
}

async fn list_words(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> ApiResult<Json<Vec<engai_core::models::Word>>> {
    let words = state
        .word_repo
        .list_words(
            params.search.as_deref(),
            params.familiarity_gte,
            params.limit.unwrap_or(50),
            params.offset.unwrap_or(0),
        )
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(words))
}

async fn create_word(
    State(state): State<AppState>,
    Json(body): Json<CreateWordBody>,
) -> ApiResult<Json<engai_core::models::Word>> {
    let word = state
        .word_repo
        .add_word(&body.word, body.phonetic.as_deref(), body.meaning.as_deref())
        .await
        .map_err(|e| ApiError::bad_request(&e.to_string()))?;
    Ok(Json(word))
}

async fn get_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Json<engai_core::models::Word>> {
    let w = state
        .word_repo
        .get_word(&word)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found(&format!("word '{}' not found", word)))?;
    Ok(Json(w))
}

async fn update_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
    Json(body): Json<UpdateWordBody>,
) -> ApiResult<Json<engai_core::models::Word>> {
    let current = state
        .word_repo
        .get_word(&word)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found(&format!("word '{}' not found", word)))?;
    let updated = state
        .word_repo
        .update_word(
            current.id,
            body.word.as_deref(),
            body.phonetic.as_deref(),
            body.meaning.as_deref(),
            body.familiarity,
            None,
            None,
            None,
        )
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::internal("failed to update word"))?;
    Ok(Json(updated))
}

async fn delete_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let current = state
        .word_repo
        .get_word(&word)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found(&format!("word '{}' not found", word)))?;
    state
        .word_repo
        .delete_word(current.id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(json!({ "deleted": true })))
}

async fn explain_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let ai = state.ai_client.clone();
    let pe = state.prompt_engine.clone();
    let word_clone = word.clone();
    let stream = async_stream::stream! {
        match ai.explain_word(&word_clone, &pe).await {
            Ok(text) => {
                yield Ok(Event::default().data(text));
            }
            Err(e) => {
                yield Ok(Event::default().data(format!("Error: {}", e)));
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn get_examples(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Json<Vec<engai_core::models::Example>>> {
    let w = state
        .word_repo
        .get_word(&word)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found(&format!("word '{}' not found", word)))?;
    let examples = state
        .example_repo
        .get_examples("word", w.id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(examples))
}
