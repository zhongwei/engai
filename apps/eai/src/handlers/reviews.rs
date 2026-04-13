use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::state::AppState;
use crate::services::{ReviewEntry, ReviewResult, ReviewStats};

#[derive(Deserialize)]
pub struct ReviewSubmit {
    pub quality: i32,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/today", get(today_reviews))
        .route("/{target_type}/{id}", post(submit_review))
        .route("/stats", get(review_stats))
}

async fn today_reviews(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<ReviewEntry>>> {
    let items = state.review_service.get_today_reviews().await?;
    Ok(Json(items))
}

async fn submit_review(
    State(state): State<AppState>,
    Path((target_type, id)): Path<(String, i64)>,
    Json(body): Json<ReviewSubmit>,
) -> ApiResult<Json<ReviewResult>> {
    let result = state
        .review_service
        .submit_review(&target_type, id, body.quality)
        .await?;
    Ok(Json(result))
}

async fn review_stats(
    State(state): State<AppState>,
) -> ApiResult<Json<ReviewStats>> {
    let stats = state.review_service.get_review_stats().await?;
    Ok(Json(stats))
}
