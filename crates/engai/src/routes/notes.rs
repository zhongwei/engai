use axum::{
    extract::{Path, Query, State},
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct NoteQuery {
    pub target_type: String,
    pub target_id: i64,
}

#[derive(Deserialize)]
pub struct CreateNoteBody {
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateNoteBody {
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notes).post(create_note))
        .route("/{id}", put(update_note).delete(delete_note))
}

async fn list_notes(
    State(state): State<AppState>,
    Query(params): Query<NoteQuery>,
) -> ApiResult<Json<Vec<engai_core::models::Note>>> {
    let notes = state
        .note_repo
        .get_notes(&params.target_type, params.target_id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(notes))
}

async fn create_note(
    State(state): State<AppState>,
    Json(body): Json<CreateNoteBody>,
) -> ApiResult<Json<engai_core::models::Note>> {
    let note = state
        .note_repo
        .add_note(&body.target_type, body.target_id, &body.content)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(note))
}

async fn update_note(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateNoteBody>,
) -> ApiResult<Json<engai_core::models::Note>> {
    state
        .note_repo
        .delete_note(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let note = state
        .note_repo
        .add_note(&body.target_type, body.target_id, &body.content)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(note))
}

async fn delete_note(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .note_repo
        .delete_note(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(json!({ "deleted": true })))
}
