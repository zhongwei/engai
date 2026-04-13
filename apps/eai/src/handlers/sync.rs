use axum::{extract::State, routing::post, Json, Router};
use serde_json::json;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(trigger_sync))
}

async fn trigger_sync(
    State(state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .sync_service
        .sync_all()
        .await
        .map_err(|e| crate::error::ApiError::internal(&e.to_string()))?;
    Ok(Json(json!({ "synced": true })))
}
