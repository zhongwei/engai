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
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateReadingBody {
    pub title: Option<String>,
    pub content: String,
    pub source: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_readings).post(create_reading))
        .route("/{id}", get(get_reading).delete(delete_reading))
        .route("/{id}/analyze", get(analyze_reading))
}

async fn list_readings(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> ApiResult<Json<Vec<engai_core::models::Reading>>> {
    let readings = state
        .reading_repo
        .list_readings(params.limit.unwrap_or(50), params.offset.unwrap_or(0))
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(readings))
}

async fn create_reading(
    State(state): State<AppState>,
    Json(body): Json<CreateReadingBody>,
) -> ApiResult<Json<engai_core::models::Reading>> {
    let reading = state
        .reading_repo
        .add_reading(body.title.as_deref(), &body.content, body.source.as_deref())
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(reading))
}

async fn get_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<engai_core::models::Reading>> {
    let reading = state
        .reading_repo
        .get_reading(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found(&format!("reading {} not found", id)))?;
    Ok(Json(reading))
}

async fn delete_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .reading_repo
        .delete_reading(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(json!({ "deleted": true })))
}

async fn analyze_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let ai = state.ai_client.clone();
    let pe = state.prompt_engine.clone();
    let content = async move {
        let r = state.reading_repo.get_reading(id).await.ok().flatten()?;
        Some(r.content)
    }
    .await
    .unwrap_or_default();
    let stream = async_stream::stream! {
        match ai.analyze_reading(&content, &pe).await {
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
