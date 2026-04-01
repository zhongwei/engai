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
        .word_service
        .list_words(
            params.search.as_deref(),
            params.familiarity_gte,
            params.limit.unwrap_or(50),
            params.offset.unwrap_or(0),
        )
        .await?;
    Ok(Json(words))
}

async fn create_word(
    State(state): State<AppState>,
    Json(body): Json<CreateWordBody>,
) -> ApiResult<Json<engai_core::models::Word>> {
    let word = state
        .word_service
        .add_word(&body.word, body.phonetic.as_deref(), body.meaning.as_deref())
        .await?;
    Ok(Json(word))
}

async fn get_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Json<engai_core::models::Word>> {
    let w = state.word_service.get_word(&word).await?;
    Ok(Json(w))
}

async fn update_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
    Json(body): Json<UpdateWordBody>,
) -> ApiResult<Json<engai_core::models::Word>> {
    let current = state.word_service.get_word(&word).await?;
    let updated = state
        .word_service
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
        .await?;
    Ok(Json(updated))
}

async fn delete_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.word_service.delete_word(&word).await?;
    Ok(Json(json!({ "deleted": true })))
}

async fn explain_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let svc = state.word_service.clone();
    let word_clone = word.clone();
    let stream = async_stream::stream! {
        match svc.explain_word(&word_clone).await {
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
    let examples = state.word_service.get_examples(&word).await?;
    Ok(Json(examples))
}
