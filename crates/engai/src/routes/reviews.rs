use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct ReviewItem {
    pub target_type: String,
    pub id: i64,
    pub display: String,
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
}

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
) -> ApiResult<Json<Vec<ReviewItem>>> {
    let words = state
        .db
        .get_today_review_words()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let phrases = state
        .db
        .get_today_review_phrases()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    let mut items: Vec<ReviewItem> = words
        .into_iter()
        .map(|w| ReviewItem {
            target_type: "word".to_string(),
            id: w.id,
            display: w.word,
            familiarity: w.familiarity,
            interval: w.interval,
            ease_factor: w.ease_factor,
        })
        .collect();
    items.extend(phrases.into_iter().map(|p| ReviewItem {
        target_type: "phrase".to_string(),
        id: p.id,
        display: p.phrase,
        familiarity: p.familiarity,
        interval: p.interval,
        ease_factor: p.ease_factor,
    }));
    Ok(Json(items))
}

async fn submit_review(
    State(state): State<AppState>,
    Path((target_type, id)): Path<(String, i64)>,
    Json(body): Json<ReviewSubmit>,
) -> ApiResult<Json<serde_json::Value>> {
    if body.quality < 0 || body.quality > 5 {
        return Err(ApiError::bad_request("quality must be between 0 and 5"));
    }

    let (interval, ease_factor) = match target_type.as_str() {
        "word" => {
            let w = state
                .db
                .get_word_by_id(id)
                .await
                .map_err(|e| ApiError::internal(&e.to_string()))?
                .ok_or_else(|| ApiError::not_found("word not found"))?;
            (w.interval, w.ease_factor)
        }
        "phrase" => {
            let p = state
                .db
                .get_phrase_by_id(id)
                .await
                .map_err(|e| ApiError::internal(&e.to_string()))?
                .ok_or_else(|| ApiError::not_found("phrase not found"))?;
            (p.interval, p.ease_factor)
        }
        _ => return Err(ApiError::bad_request("target_type must be 'word' or 'phrase'")),
    };

    let result = engai_core::review::calculate_next_review(body.quality, interval, ease_factor);

    match target_type.as_str() {
        "word" => {
            state
                .db
                .update_word(
                    id,
                    None,
                    None,
                    None,
                    Some(result.familiarity),
                    Some(result.next_review),
                    Some(result.interval),
                    Some(result.ease_factor),
                )
                .await
                .map_err(|e| ApiError::internal(&e.to_string()))?;
        }
        "phrase" => {
            state
                .db
                .update_phrase(
                    id,
                    None,
                    None,
                    Some(result.familiarity),
                    Some(result.next_review),
                    Some(result.interval),
                    Some(result.ease_factor),
                )
                .await
                .map_err(|e| ApiError::internal(&e.to_string()))?;
        }
        _ => unreachable!(),
    }

    state
        .db
        .add_review(&target_type, id, body.quality)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    Ok(Json(json!({
        "updated": true,
        "next_review": result.next_review,
        "interval": result.interval,
        "ease_factor": result.ease_factor,
        "familiarity": result.familiarity,
    })))
}

async fn review_stats(
    State(state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    let pending = state
        .db
        .pending_review_count()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let reviewed = state
        .db
        .review_count_today()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(json!({
        "pending_reviews": pending,
        "reviewed_today": reviewed,
    })))
}
