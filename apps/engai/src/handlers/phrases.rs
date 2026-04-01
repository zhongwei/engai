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
) -> ApiResult<Json<Vec<crate::models::Phrase>>> {
    let phrases = state
        .phrase_service
        .list_phrases(
            params.search.as_deref(),
            params.familiarity_gte,
            params.limit.unwrap_or(50),
            params.offset.unwrap_or(0),
        )
        .await?;
    Ok(Json(phrases))
}

async fn create_phrase(
    State(state): State<AppState>,
    Json(body): Json<CreatePhraseBody>,
) -> ApiResult<Json<crate::models::Phrase>> {
    let phrase = state
        .phrase_service
        .add_phrase(&body.phrase, body.meaning.as_deref())
        .await?;
    Ok(Json(phrase))
}

async fn get_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<crate::models::Phrase>> {
    let p = state.phrase_service.get_phrase_by_id(id).await?;
    Ok(Json(p))
}

async fn update_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePhraseBody>,
) -> ApiResult<Json<crate::models::Phrase>> {
    let updated = state
        .phrase_service
        .update_phrase(id, body.phrase.as_deref(), body.meaning.as_deref(), body.familiarity, None, None, None)
        .await?;
    Ok(Json(updated))
}

async fn delete_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    state.phrase_service.delete_phrase(id).await?;
    Ok(Json(json!({ "deleted": true })))
}

async fn explain_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let phrase_text = match state.phrase_service.get_phrase_by_id(id).await {
        Ok(p) => p.phrase,
        Err(_) => format!("phrase_{}", id),
    };
    let svc = state.phrase_service.clone();
    let stream = async_stream::stream! {
        match svc.explain_phrase(&phrase_text).await {
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
) -> ApiResult<Json<Vec<crate::models::Example>>> {
    let examples = state.phrase_service.get_examples(id).await?;
    Ok(Json(examples))
}
