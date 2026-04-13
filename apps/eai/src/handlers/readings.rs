use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Json, Router,
};
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::json;

use crate::error::ApiResult;
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
) -> ApiResult<Json<Vec<crate::models::Reading>>> {
    let readings = state
        .reading_service
        .list_readings(params.limit.unwrap_or(50), params.offset.unwrap_or(0))
        .await?;
    Ok(Json(readings))
}

async fn create_reading(
    State(state): State<AppState>,
    Json(body): Json<CreateReadingBody>,
) -> ApiResult<Json<crate::models::Reading>> {
    let reading = state
        .reading_service
        .add_reading(body.title.as_deref(), &body.content, body.source.as_deref())
        .await?;
    Ok(Json(reading))
}

async fn get_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<crate::models::Reading>> {
    let reading = state.reading_service.get_reading(id).await?;
    Ok(Json(reading))
}

async fn delete_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    state.reading_service.delete_reading(id).await?;
    Ok(Json(json!({ "deleted": true })))
}

async fn analyze_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let content = match state.reading_service.get_reading(id).await {
        Ok(r) => r.content,
        Err(_) => String::new(),
    };
    let svc = state.reading_service.clone();
    let stream = async_stream::stream! {
        match svc.analyze_reading(&content).await {
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
