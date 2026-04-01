use axum::{extract::State, routing::get, Json, Router};
use axum::response::IntoResponse;

use crate::error::ApiError;
use crate::state::AppState;
use engai_core::services::StatsData;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_stats))
}

async fn get_stats(State(state): State<AppState>) -> Result<Json<StatsData>, ApiError> {
    let stats = state.stats_service.get_stats().await.map_err(ApiError::from)?;
    Ok(Json(stats))
}
