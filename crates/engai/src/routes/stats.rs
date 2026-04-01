use axum::{extract::State, routing::get, Json, Router};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_stats))
}

async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let word_count = state
        .word_repo
        .word_count()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let phrase_count = state
        .phrase_repo
        .phrase_count()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let pending_reviews = state
        .review_repo
        .pending_review_count()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let reviewed_today = state
        .review_repo
        .review_count_today()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    Ok(Json(serde_json::json!({
        "word_count": word_count,
        "phrase_count": phrase_count,
        "pending_reviews": pending_reviews,
        "reviewed_today": reviewed_today,
    })))
}
