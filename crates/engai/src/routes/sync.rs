use axum::{extract::State, routing::post, Json, Router};
use serde_json::json;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(trigger_sync))
}

async fn trigger_sync(
    State(state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    let engine = engai_core::sync::SyncEngine::new(
        state.db.clone(),
        &state.config.docs_path(),
        &state.config.prompts_path(),
    );
    engine
        .sync_all()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(json!({ "synced": true })))
}
