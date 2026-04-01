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
pub struct CreatePhraseBody {
    pub phrase: String,
    pub meaning: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePhraseBody {
    pub phrase: Option<String>,
    pub meaning: Option<String>,
    pub familiarity: Option<i32>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_phrases).post(create_phrase))
        .route("/{id}", get(get_phrase).put(update_phrase).delete(delete_phrase))
        .route("/{id}/explain", get(explain_phrase))
        .route("/{id}/examples", get(get_examples))
}

async fn list_phrases(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> ApiResult<Json<Vec<engai_core::models::Phrase>>> {
    let phrases = state
        .phrase_repo
        .list_phrases(
            params.search.as_deref(),
            params.familiarity_gte,
            params.limit.unwrap_or(50),
            params.offset.unwrap_or(0),
        )
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(phrases))
}

async fn create_phrase(
    State(state): State<AppState>,
    Json(body): Json<CreatePhraseBody>,
) -> ApiResult<Json<engai_core::models::Phrase>> {
    let phrase = state
        .phrase_repo
        .add_phrase(&body.phrase, body.meaning.as_deref())
        .await
        .map_err(|e| ApiError::bad_request(&e.to_string()))?;
    Ok(Json(phrase))
}

async fn get_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<engai_core::models::Phrase>> {
    let p = state
        .phrase_repo
        .get_phrase_by_id(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found(&format!("phrase {} not found", id)))?;
    Ok(Json(p))
}

async fn update_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePhraseBody>,
) -> ApiResult<Json<engai_core::models::Phrase>> {
    let updated = state
        .phrase_repo
        .update_phrase(id, body.phrase.as_deref(), body.meaning.as_deref(), body.familiarity, None, None, None)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found(&format!("phrase {} not found", id)))?;
    Ok(Json(updated))
}

async fn delete_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .phrase_repo
        .delete_phrase(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(json!({ "deleted": true })))
}

async fn explain_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let ai = state.ai_client.clone();
    let pe = state.prompt_engine.clone();
    let phrase_text = async move {
        let p = state.phrase_repo.get_phrase_by_id(id).await.ok().flatten()?;
        Some(p.phrase)
    }
    .await
    .unwrap_or_else(|| format!("phrase_{}", id));
    let stream = async_stream::stream! {
        match ai.explain_phrase(&phrase_text, &pe).await {
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
    Path(id): Path<i64>,
) -> ApiResult<Json<Vec<engai_core::models::Example>>> {
    let examples = state
        .example_repo
        .get_examples("phrase", id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(examples))
}
